import type {
  Endpoint,
  WebhookRequest,
  ResponseRule,
  TunnelStatus,
  ApiForwarder,
  ServerStatus,
  ReplayRequestPayload,
  ReplayResponse,
} from '../types';
import { initialEndpoints, initialRequests, initialRules } from './mockData';

// Detect whether running inside Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Dynamic import Tauri invoke / listen to prevent breaking when in standard browser Vite
let tauriInvoke: any = null;
let tauriListen: any = null;

if (isTauri) {
  try {
    const core = await import('@tauri-apps/api/core');
    tauriInvoke = core.invoke;
    const event = await import('@tauri-apps/api/event');
    tauriListen = event.listen;
  } catch (err) {
    console.warn('Tauri API import failed, falling back to mock mode', err);
  }
}

// In-browser mock state (used if not in Tauri)
let mockEndpoints = [...initialEndpoints];
let mockRequests = [...initialRequests];
let mockRules = [...initialRules];
let mockTunnel: TunnelStatus = {
  is_running: false,
  public_url: null,
  error: null,
  started_at: null,
};
const mockServer: ServerStatus = {
  is_running: true,
  port: 4567,
  local_base_url: 'http://localhost:4567',
};

const listeners: ((req: WebhookRequest) => void)[] = [];

let activePendingUpdate: any = null;

export const api = {
  isTauriRuntime(): boolean {
    return !!tauriInvoke;
  },

  async getAppVersion(): Promise<string> {
    if (tauriInvoke) {
      try {
        return await tauriInvoke('get_app_version');
      } catch {
        return '0.2.0';
      }
    }
    return '0.2.0';
  },

  async checkForAppUpdates(): Promise<{
    available: boolean;
    currentVersion: string;
    version?: string;
    body?: string;
    error?: string;
  }> {
    if (!isTauri) {
      return { available: false, currentVersion: '0.2.0 (Web)' };
    }
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        activePendingUpdate = update;
        return {
          available: true,
          currentVersion: update.currentVersion,
          version: update.version,
          body: update.body || '',
        };
      }
      activePendingUpdate = null;
      return {
        available: false,
        currentVersion: '0.2.0',
      };
    } catch (err: any) {
      console.warn('Lỗi kiểm tra cập nhật:', err);
      return {
        available: false,
        currentVersion: '0.2.0',
        error: err?.message || 'Không thể kiểm tra cập nhật lúc này',
      };
    }
  },

  async installAppUpdate(): Promise<void> {
    if (!activePendingUpdate) {
      throw new Error('Không có bản cập nhật nào đang chờ');
    }
    await activePendingUpdate.downloadAndInstall();
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  },

  async getServerStatus(): Promise<ServerStatus> {
    if (tauriInvoke) {
      return await tauriInvoke('get_server_status');
    }
    return mockServer;
  },

  async getEndpoints(): Promise<Endpoint[]> {
    if (tauriInvoke) {
      return await tauriInvoke('get_endpoints');
    }
    return mockEndpoints.map((ep) => ({
      ...ep,
      request_count: mockRequests.filter((r) => r.endpoint_id === ep.id).length,
    }));
  },

  async createEndpoint(endpoint: Partial<Endpoint>): Promise<Endpoint> {
    if (tauriInvoke) {
      return await tauriInvoke('create_endpoint', { endpoint });
    }
    const newEp: Endpoint = {
      id: 'ep-' + Math.random().toString(36).substring(2, 9),
      name: endpoint.name || 'New Endpoint',
      slug: endpoint.slug || 'webhook-' + Math.random().toString(36).substring(2, 7),
      allowed_methods: endpoint.allowed_methods || ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'],
      default_status: endpoint.default_status ?? 200,
      default_headers: endpoint.default_headers || { 'Content-Type': 'application/json' },
      default_body: endpoint.default_body || '{"received": true}',
      default_content_type: endpoint.default_content_type || 'application/json',
      delay_ms: endpoint.delay_ms ?? 0,
      error_rate_percent: endpoint.error_rate_percent ?? 0,
      error_status: endpoint.error_status ?? 500,
      error_body: endpoint.error_body || null,
      is_active: true,
      created_at: new Date().toISOString(),
      request_count: 0,
    };
    mockEndpoints = [newEp, ...mockEndpoints];
    return newEp;
  },

  async updateEndpoint(endpoint: Endpoint): Promise<Endpoint> {
    if (tauriInvoke) {
      return await tauriInvoke('update_endpoint', { endpoint });
    }
    mockEndpoints = mockEndpoints.map((e) => (e.id === endpoint.id ? endpoint : e));
    return endpoint;
  },

  async deleteEndpoint(id: string): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('delete_endpoint', { id });
    }
    mockEndpoints = mockEndpoints.filter((e) => e.id !== id);
    mockRequests = mockRequests.filter((r) => r.endpoint_id !== id);
    mockRules = mockRules.filter((rl) => rl.endpoint_id !== id);
    return true;
  },

  async getRequests(endpointId?: string): Promise<WebhookRequest[]> {
    if (tauriInvoke) {
      return await tauriInvoke('get_requests', { endpointId });
    }
    if (endpointId) {
      return mockRequests.filter((r) => r.endpoint_id === endpointId);
    }
    return mockRequests;
  },

  async clearRequests(endpointId?: string): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('clear_requests', { endpointId });
    }
    if (endpointId) {
      mockRequests = mockRequests.filter((r) => r.endpoint_id !== endpointId);
    } else {
      mockRequests = [];
    }
    return true;
  },

  async deleteRequest(id: string): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('delete_request', { id });
    }
    mockRequests = mockRequests.filter((r) => r.id !== id);
    return true;
  },

  async getRules(endpointId: string): Promise<ResponseRule[]> {
    if (tauriInvoke) {
      return await tauriInvoke('get_rules', { endpointId });
    }
    return mockRules.filter((r) => r.endpoint_id === endpointId);
  },

  async saveRule(rule: Partial<ResponseRule>): Promise<ResponseRule> {
    if (tauriInvoke) {
      return await tauriInvoke('save_rule', { rule });
    }
    const existingIndex = mockRules.findIndex((r) => r.id === rule.id);
    const ruleObj: ResponseRule = {
      id: rule.id || 'rule-' + Math.random().toString(36).substring(2, 9),
      endpoint_id: rule.endpoint_id!,
      name: rule.name || 'Rule',
      priority: rule.priority ?? mockRules.length + 1,
      condition_type: rule.condition_type || 'header',
      condition_field: rule.condition_field || '',
      condition_operator: rule.condition_operator || 'equals',
      condition_value: rule.condition_value || '',
      response_status: rule.response_status ?? 200,
      response_headers: rule.response_headers || { 'Content-Type': 'application/json' },
      response_body: rule.response_body || '{"matched": true}',
      response_delay_ms: rule.response_delay_ms ?? 0,
      is_enabled: rule.is_enabled ?? true,
    };
    if (existingIndex >= 0) {
      mockRules[existingIndex] = ruleObj;
    } else {
      mockRules = [...mockRules, ruleObj];
    }
    return ruleObj;
  },

  async deleteRule(id: string): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('delete_rule', { id });
    }
    mockRules = mockRules.filter((r) => r.id !== id);
    return true;
  },

  async replayRequest(payload: ReplayRequestPayload): Promise<ReplayResponse> {
    if (tauriInvoke) {
      return await tauriInvoke('replay_request', { payload });
    }
    // Simulation in mock mode
    const startTime = performance.now();
    try {
      const res = await fetch(payload.target_url, {
        method: payload.method,
        headers: payload.headers,
        body: ['GET', 'HEAD'].includes(payload.method) ? undefined : payload.body,
      });
      const timeMs = Math.round(performance.now() - startTime);
      const text = await res.text();
      const resHeaders: Record<string, string> = {};
      res.headers.forEach((val, key) => {
        resHeaders[key] = val;
      });
      return {
        status: res.status,
        status_text: res.statusText,
        headers: resHeaders,
        body: text,
        time_ms: timeMs,
      };
    } catch (err: any) {
      return {
        status: 0,
        status_text: 'Network Error',
        headers: {},
        body: '',
        time_ms: Math.round(performance.now() - startTime),
        error: err?.message || 'Could not connect to target URL',
      };
    }
  },

  async getTunnelStatus(): Promise<TunnelStatus> {
    if (tauriInvoke) {
      return await tauriInvoke('get_tunnel_status');
    }
    return mockTunnel;
  },

  async checkCloudflaredInstalled(): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('check_cloudflared_installed');
    }
    return true;
  },

  async startTunnel(): Promise<TunnelStatus> {
    if (tauriInvoke) {
      return await tauriInvoke('start_tunnel');
    }
    mockTunnel = {
      is_running: true,
      public_url: 'https://webhook-lab-' + Math.random().toString(36).substring(2, 8) + '.trycloudflare.com',
      error: null,
      started_at: new Date().toISOString(),
    };
    return mockTunnel;
  },

  async stopTunnel(): Promise<TunnelStatus> {
    if (tauriInvoke) {
      return await tauriInvoke('stop_tunnel');
    }
    mockTunnel = {
      is_running: false,
      public_url: null,
      error: null,
      started_at: null,
    };
    return mockTunnel;
  },

  async getApiForwarders(): Promise<ApiForwarder[]> {
    if (tauriInvoke) {
      return await tauriInvoke('get_api_forwarders');
    }
    return [
      {
        id: 'fwd-1',
        name: 'Backend API',
        host: '127.0.0.1',
        port: 3000,
        created_at: new Date().toISOString(),
        is_running: false,
      },
    ];
  },

  async saveApiForwarder(forwarder: Partial<ApiForwarder>): Promise<ApiForwarder> {
    const fwdObj: ApiForwarder = {
      id: forwarder.id || 'fwd-' + Math.random().toString(36).substring(2, 9),
      name: forwarder.name || 'API Service',
      host: forwarder.host || '127.0.0.1',
      port: forwarder.port || 3000,
      created_at: forwarder.created_at || new Date().toISOString(),
      is_running: forwarder.is_running || false,
      public_url: forwarder.public_url || null,
      error: forwarder.error || null,
      started_at: forwarder.started_at || null,
    };
    if (tauriInvoke) {
      return await tauriInvoke('save_api_forwarder', { forwarder: fwdObj });
    }
    return fwdObj;
  },

  async deleteApiForwarder(id: string): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('delete_api_forwarder', { id });
    }
    return true;
  },

  async startApiForwarder(id: string, host: string, port: number): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('start_api_forwarder', { id, host, port });
    }
    return true;
  },

  async stopApiForwarder(id: string): Promise<boolean> {
    if (tauriInvoke) {
      return await tauriInvoke('stop_api_forwarder', { id });
    }
    return true;
  },

  async sendTestWebhook(endpointId: string, preset: string): Promise<WebhookRequest> {
    if (tauriInvoke) {
      return await tauriInvoke('send_test_webhook', { endpointId, preset });
    }
    const ep = mockEndpoints.find((e) => e.id === endpointId) || mockEndpoints[0];
    let body = '{}';
    let headers: Record<string, string> = {
      'content-type': 'application/json',
      'user-agent': 'WebhookLab-TestSender/1.0',
    };

    if (preset === 'stripe') {
      headers['stripe-signature'] = 't=' + Math.floor(Date.now() / 1000) + ',v1=test_sig';
      body = JSON.stringify(
        {
          id: 'evt_sim_' + Math.random().toString(36).substring(2, 9),
          object: 'event',
          type: 'payment_intent.succeeded',
          created: Math.floor(Date.now() / 1000),
          data: { object: { amount: 4999, currency: 'usd', status: 'succeeded' } },
        },
        null,
        2
      );
    } else if (preset === 'github') {
      headers['x-github-event'] = 'push';
      body = JSON.stringify(
        {
          ref: 'refs/heads/main',
          repository: { name: 'app-production', full_name: 'org/app-production' },
          commits: [{ message: 'feat: add webhook integration', author: { name: 'Developer' } }],
        },
        null,
        2
      );
    } else {
      body = JSON.stringify({ message: 'Custom test webhook', timestamp: new Date().toISOString() }, null, 2);
    }

    const testReq: WebhookRequest = {
      id: 'req-' + Math.random().toString(36).substring(2, 9),
      endpoint_id: ep.id,
      method: 'POST',
      path: `/wh/${ep.slug}`,
      query_params: { test: '1' },
      headers,
      body,
      content_type: 'application/json',
      ip_address: '127.0.0.1',
      timestamp: new Date().toISOString(),
      response_status: ep.default_status,
      response_time_ms: 6,
    };

    mockRequests = [testReq, ...mockRequests];
    listeners.forEach((fn) => fn(testReq));
    return testReq;
  },

  onRequestReceived(callback: (req: WebhookRequest) => void): () => void {
    if (tauriListen) {
      let unlistenFn: any = null;
      tauriListen('request_received', (event: any) => {
        callback(event.payload);
      }).then((unlisten: any) => {
        unlistenFn = unlisten;
      });
      return () => {
        if (unlistenFn) unlistenFn();
      };
    }

    listeners.push(callback);
    return () => {
      const idx = listeners.indexOf(callback);
      if (idx >= 0) listeners.splice(idx, 1);
    };
  },

  onTunnelStatusChanged(callback: (status: TunnelStatus) => void): () => void {
    if (tauriListen) {
      let unlistenFn: any = null;
      tauriListen('tunnel_status_changed', (event: any) => {
        callback(event.payload);
      }).then((unlisten: any) => {
        unlistenFn = unlisten;
      });
      return () => {
        if (unlistenFn) unlistenFn();
      };
    }
    return () => {};
  },

  onForwarderStatusChanged(
    callback: (payload: { id: string; is_running: boolean; public_url?: string | null; error?: string | null }) => void
  ): () => void {
    if (tauriListen) {
      let unlistenFn: any = null;
      tauriListen('forwarder_status_changed', (event: any) => {
        callback(event.payload);
      }).then((unlisten: any) => {
        unlistenFn = unlisten;
      });
      return () => {
        if (unlistenFn) unlistenFn();
      };
    }
    return () => {};
  },
};
