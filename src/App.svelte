<script lang="ts">
  import { onMount } from 'svelte';
  import type { Endpoint, WebhookRequest, ResponseRule, TunnelStatus, ServerStatus, ApiForwarder } from './types';
  import { api } from './lib/api';
  import Header from './components/Header.svelte';
  import EndpointsSidebar from './components/EndpointsSidebar.svelte';
  import RequestsList from './components/RequestsList.svelte';
  import RequestDetail from './components/RequestDetail.svelte';
  import EndpointModal from './components/EndpointModal.svelte';
  import RulesModal from './components/RulesModal.svelte';
  import ReplayModal from './components/ReplayModal.svelte';
  import InstallTunnelModal from './components/InstallTunnelModal.svelte';
  import ApiForwarderModal from './components/ApiForwarderModal.svelte';

  // Core application state
  let endpoints = $state<Endpoint[]>([]);
  let requests = $state<WebhookRequest[]>([]);
  let rules = $state<ResponseRule[]>([]);
  let selectedEndpointId = $state<string | null>(null);
  let selectedRequestId = $state<string | null>(null);

  let isInstallModalOpen = $state(false);

  let tunnelStatus = $state<TunnelStatus>({
    is_running: false,
    public_url: null,
    error: null,
    started_at: null,
  });

  let forwarders = $state<ApiForwarder[]>([]);
  let isForwarderModalOpen = $state(false);

  let serverStatus = $state<ServerStatus>({
    is_running: true,
    port: 4567,
    local_base_url: 'http://localhost:4567',
  });

  let isTunnelLoading = $state(false);

  // Modal dialog states
  let isEndpointModalOpen = $state(false);
  let editingEndpoint = $state<Endpoint | null>(null);

  let isRulesModalOpen = $state(false);
  let activeRulesEndpoint = $state<Endpoint | null>(null);
  let rulePrefill = $state<Partial<ResponseRule> | null>(null);

  let isReplayModalOpen = $state(false);
  let replayTargetRequest = $state<WebhookRequest | null>(null);

  // Derived filtered requests based on selected endpoint
  let displayedRequests = $derived(
    selectedEndpointId
      ? requests.filter((r) => r.endpoint_id === selectedEndpointId)
      : requests
  );

  let activeRequest = $derived(
    requests.find((r) => r.id === selectedRequestId) || null
  );

  onMount(() => {
    loadInitialData();

    // Listen for live incoming webhooks from Rust backend
    const unlistenRequests = api.onRequestReceived((newReq) => {
      requests = [newReq, ...requests];

      // Update endpoint request count
      endpoints = endpoints.map((ep) =>
        ep.id === newReq.endpoint_id
          ? { ...ep, request_count: (ep.request_count ?? 0) + 1 }
          : ep
      );

      // Auto-select first request if none selected
      if (!selectedRequestId) {
        selectedRequestId = newReq.id;
      }
    });

    // Listen for Cloudflare tunnel status changes
    const unlistenTunnel = api.onTunnelStatusChanged((newStatus) => {
      tunnelStatus = newStatus;
      isTunnelLoading = false;
    });

    // Listen for API Forwarder tunnel status changes
    const unlistenForwarder = api.onForwarderStatusChanged((payload) => {
      forwarders = forwarders.map((f) =>
        f.id === payload.id
          ? {
              ...f,
              is_running: payload.is_running,
              public_url: payload.public_url ?? (payload.is_running ? f.public_url : null),
              error: payload.error ?? null,
              is_loading: false,
            }
          : f
      );
    });

    return () => {
      unlistenRequests();
      unlistenTunnel();
      unlistenForwarder();
    };
  });

  async function loadInitialData() {
    try {
      const [eps, reqs, sStatus, tStatus, fwds] = await Promise.all([
        api.getEndpoints(),
        api.getRequests(),
        api.getServerStatus(),
        api.getTunnelStatus(),
        api.getApiForwarders(),
      ]);

      endpoints = eps;
      requests = reqs;
      serverStatus = sStatus;
      tunnelStatus = tStatus;
      forwarders = fwds;

      if (reqs.length > 0 && !selectedRequestId) {
        selectedRequestId = reqs[0].id;
      }
    } catch (err) {
      console.error('Failed to load initial data:', err);
    }
  }

  // Tunnel control
  async function handleToggleTunnel() {
    if (tunnelStatus.is_running) {
      isTunnelLoading = true;
      try {
        tunnelStatus = await api.stopTunnel();
      } catch (err: any) {
        console.error('Lỗi khi dừng tunnel:', err);
      } finally {
        isTunnelLoading = false;
      }
      return;
    }

    // Kiểm tra xem máy đã cài cloudflared chưa
    try {
      const isInstalled = await api.checkCloudflaredInstalled();
      if (!isInstalled) {
        // Chưa có -> Mở hộp thoại xác nhận từ người dùng
        isInstallModalOpen = true;
        return;
      }
    } catch (e) {
      console.warn('Lỗi kiểm tra cloudflared:', e);
    }

    // Đã có -> Khởi động ngay
    await executeStartTunnel();
  }

  async function executeStartTunnel() {
    isTunnelLoading = true;
    try {
      tunnelStatus = await api.startTunnel();
    } catch (err: any) {
      console.error('Lỗi khi khởi động tunnel:', err);
      tunnelStatus = {
        ...tunnelStatus,
        is_running: false,
        error: err?.message || 'Không thể khởi động tunnel',
      };
    } finally {
      isTunnelLoading = false;
    }
  }

  // API Forwarder management (Multi-forwarder)
  async function handleSaveForwarder(forwarder: Partial<ApiForwarder>) {
    try {
      const saved = await api.saveApiForwarder(forwarder);
      const idx = forwarders.findIndex((f) => f.id === saved.id);
      if (idx >= 0) {
        forwarders[idx] = { ...forwarders[idx], ...saved };
      } else {
        forwarders = [...forwarders, saved];
      }
    } catch (err: any) {
      console.error('Lỗi khi lưu forwarder:', err);
      throw err;
    }
  }

  async function handleDeleteForwarder(id: string) {
    try {
      await api.deleteApiForwarder(id);
      forwarders = forwarders.filter((f) => f.id !== id);
    } catch (err) {
      console.error('Lỗi khi xóa forwarder:', err);
    }
  }

  async function handleStartForwarder(id: string, host: string, port: number) {
    try {
      const isInstalled = await api.checkCloudflaredInstalled();
      if (!isInstalled) {
        isInstallModalOpen = true;
        return;
      }
      forwarders = forwarders.map((f) =>
        f.id === id ? { ...f, is_loading: true, error: null } : f
      );
      await api.startApiForwarder(id, host, port);
    } catch (err: any) {
      console.error('Lỗi khi bật API forwarder:', err);
      forwarders = forwarders.map((f) =>
        f.id === id ? { ...f, is_loading: false, is_running: false, error: err?.message || 'Lỗi khởi chạy' } : f
      );
    }
  }

  async function handleStopForwarder(id: string) {
    try {
      forwarders = forwarders.map((f) =>
        f.id === id ? { ...f, is_loading: true } : f
      );
      await api.stopApiForwarder(id);
      forwarders = forwarders.map((f) =>
        f.id === id ? { ...f, is_loading: false, is_running: false, public_url: null } : f
      );
    } catch (err: any) {
      console.error('Lỗi khi dừng API forwarder:', err);
      forwarders = forwarders.map((f) =>
        f.id === id ? { ...f, is_loading: false } : f
      );
    }
  }

  // Endpoint management
  function openCreateEndpointModal() {
    editingEndpoint = null;
    isEndpointModalOpen = true;
  }

  function openEditEndpointModal(ep: Endpoint) {
    editingEndpoint = ep;
    isEndpointModalOpen = true;
  }

  async function handleSaveEndpoint(payload: Partial<Endpoint>) {
    try {
      if (payload.id) {
        const updated = await api.updateEndpoint(payload as Endpoint);
        endpoints = endpoints.map((e) => (e.id === updated.id ? updated : e));
      } else {
        const created = await api.createEndpoint(payload);
        endpoints = [created, ...endpoints];
        selectedEndpointId = created.id;
      }
    } catch (err) {
      console.error('Failed to save endpoint:', err);
    } finally {
      isEndpointModalOpen = false;
      editingEndpoint = null;
    }
  }

  async function handleDeleteEndpoint(id: string) {
    try {
      await api.deleteEndpoint(id);
      endpoints = endpoints.filter((e) => e.id !== id);
      requests = requests.filter((r) => r.endpoint_id !== id);
      if (selectedEndpointId === id) {
        selectedEndpointId = null;
      }
    } catch (err) {
      console.error('Failed to delete endpoint:', err);
    }
  }

  // Rules management
  async function openRulesModal(ep: Endpoint, prefill?: Partial<ResponseRule>) {
    activeRulesEndpoint = ep;
    rulePrefill = prefill || null;
    try {
      rules = await api.getRules(ep.id);
    } catch (err) {
      console.error('Failed to load rules:', err);
      rules = [];
    }
    isRulesModalOpen = true;
  }

  async function handleSaveRule(rulePayload: Partial<ResponseRule>) {
    try {
      const saved = await api.saveRule(rulePayload);
      const idx = rules.findIndex((r) => r.id === saved.id);
      if (idx >= 0) {
        rules[idx] = saved;
      } else {
        rules = [...rules, saved];
      }
    } catch (err) {
      console.error('Failed to save rule:', err);
    }
  }

  async function handleDeleteRule(id: string) {
    try {
      await api.deleteRule(id);
      rules = rules.filter((r) => r.id !== id);
    } catch (err) {
      console.error('Failed to delete rule:', err);
    }
  }

  function handleCreateRuleFromRequest(req: WebhookRequest) {
    const ep = endpoints.find((e) => e.id === req.endpoint_id);
    if (!ep) return;

    // Detect header or JSON body event
    let condType: any = 'header';
    let condField = '';
    let condVal = '';

    if (req.headers['x-github-event']) {
      condType = 'header';
      condField = 'x-github-event';
      condVal = req.headers['x-github-event'];
    } else if (req.headers['stripe-signature']) {
      condType = 'header';
      condField = 'stripe-signature';
      condVal = 'invalid';
    } else {
      try {
        const parsed = JSON.parse(req.body);
        if (parsed.type) {
          condType = 'json_body';
          condField = 'type';
          condVal = parsed.type;
        } else if (parsed.event) {
          condType = 'json_body';
          condField = 'event';
          condVal = parsed.event;
        }
      } catch {
        condType = 'method';
        condField = '';
        condVal = req.method;
      }
    }

    openRulesModal(ep, {
      name: `Phản Hồi Mẫu cho ${condVal || req.method}`,
      condition_type: condType,
      condition_field: condField,
      condition_operator: 'equals',
      condition_value: condVal,
      response_status: 200,
      response_body: JSON.stringify({ message: 'Khớp quy tắc mock phản hồi', source: 'Webhook Lab' }, null, 2),
      response_delay_ms: 0,
      is_enabled: true,
    });
  }

  // Request actions
  async function handleClearRequests() {
    if (confirm('Bạn có chắc chắn muốn xóa toàn bộ danh sách webhook đã nhận?')) {
      await api.clearRequests(selectedEndpointId || undefined);
      if (selectedEndpointId) {
        requests = requests.filter((r) => r.endpoint_id !== selectedEndpointId);
      } else {
        requests = [];
      }
      selectedRequestId = null;
    }
  }

  async function handleDeleteRequest(id: string) {
    await api.deleteRequest(id);
    requests = requests.filter((r) => r.id !== id);
    if (selectedRequestId === id) {
      selectedRequestId = requests.length > 0 ? requests[0].id : null;
    }
  }

  function handleOpenReplay(req: WebhookRequest) {
    replayTargetRequest = req;
    isReplayModalOpen = true;
  }

  async function handleSendTest(preset: string) {
    const targetEpId = selectedEndpointId || (endpoints[0]?.id ?? 'ep-default');
    try {
      const testReq = await api.sendTestWebhook(targetEpId, preset);
      selectedRequestId = testReq.id;
    } catch (err) {
      console.error('Failed to send test webhook:', err);
    }
  }

  // Layout resize state
  const DEFAULT_SIDEBAR_WIDTH = 210;
  const DEFAULT_REQUESTS_WIDTH = 300;

  let sidebarWidth = $state(
    typeof localStorage !== 'undefined'
      ? Number(localStorage.getItem('whl_sidebar_w')) || DEFAULT_SIDEBAR_WIDTH
      : DEFAULT_SIDEBAR_WIDTH
  );
  let requestsWidth = $state(
    typeof localStorage !== 'undefined'
      ? Number(localStorage.getItem('whl_requests_w')) || DEFAULT_REQUESTS_WIDTH
      : DEFAULT_REQUESTS_WIDTH
  );
  let isDraggingLeft = $state(false);
  let isDraggingRight = $state(false);

  function startResizeLeft(e: MouseEvent) {
    e.preventDefault();
    isDraggingLeft = true;
    const startX = e.clientX;
    const initialWidth = sidebarWidth;

    function onMouseMove(moveEvent: MouseEvent) {
      const delta = moveEvent.clientX - startX;
      sidebarWidth = Math.max(160, Math.min(460, initialWidth + delta));
    }

    function onMouseUp() {
      isDraggingLeft = false;
      try {
        localStorage.setItem('whl_sidebar_w', String(sidebarWidth));
      } catch {}
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  function startResizeRight(e: MouseEvent) {
    e.preventDefault();
    isDraggingRight = true;
    const startX = e.clientX;
    const initialWidth = requestsWidth;

    function onMouseMove(moveEvent: MouseEvent) {
      const delta = moveEvent.clientX - startX;
      requestsWidth = Math.max(200, Math.min(650, initialWidth + delta));
    }

    function onMouseUp() {
      isDraggingRight = false;
      try {
        localStorage.setItem('whl_requests_w', String(requestsWidth));
      } catch {}
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  function resetLeftWidth() {
    sidebarWidth = DEFAULT_SIDEBAR_WIDTH;
    try {
      localStorage.setItem('whl_sidebar_w', String(sidebarWidth));
    } catch {}
  }

  function resetRightWidth() {
    requestsWidth = DEFAULT_REQUESTS_WIDTH;
    try {
      localStorage.setItem('whl_requests_w', String(requestsWidth));
    } catch {}
  }
</script>

<div class="app-container">
  <!-- Top App Navigation & Quick Tunnel Bar -->
  <Header
    {tunnelStatus}
    {serverStatus}
    {forwarders}
    {isTunnelLoading}
    onToggleTunnel={handleToggleTunnel}
    onOpenForwarderModal={() => (isForwarderModalOpen = true)}
    onSendTest={handleSendTest}
    onClearRequests={handleClearRequests}
  />

  <!-- Main 3-Pane Split View with Draggable Resizers -->
  <main
    class="main-content"
    class:is-resizing={isDraggingLeft || isDraggingRight}
  >
    <!-- Left Pane: Endpoints Sidebar -->
    <div class="pane-sidebar" style="width: {sidebarWidth}px;">
      <EndpointsSidebar
        {endpoints}
        {selectedEndpointId}
        {tunnelStatus}
        onSelectEndpoint={(id) => (selectedEndpointId = id)}
        onOpenCreateModal={openCreateEndpointModal}
        onOpenEditModal={openEditEndpointModal}
        onOpenRulesModal={(ep) => openRulesModal(ep)}
        onDeleteEndpoint={handleDeleteEndpoint}
      />
    </div>

    <!-- Draggable Splitter 1 (Sidebar <-> Requests) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resizer-handle"
      class:dragging={isDraggingLeft}
      onmousedown={startResizeLeft}
      ondblclick={resetLeftWidth}
      title="Kéo sang trái/phải để điều chỉnh kích thước Sidebar (Nháy đúp để về mặc định)"
    >
      <div class="resizer-line"></div>
    </div>

    <!-- Center Pane: Live Requests Stream -->
    <div class="pane-requests" style="width: {requestsWidth}px;">
      <RequestsList
        requests={displayedRequests}
        {selectedRequestId}
        onSelectRequest={(id) => (selectedRequestId = id)}
        onDeleteRequest={handleDeleteRequest}
      />
    </div>

    <!-- Draggable Splitter 2 (Requests <-> Detail Inspector) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resizer-handle"
      class:dragging={isDraggingRight}
      onmousedown={startResizeRight}
      ondblclick={resetRightWidth}
      title="Kéo sang trái/phải để điều chỉnh kích thước Danh Sách (Nháy đúp để về mặc định)"
    >
      <div class="resizer-line"></div>
    </div>

    <!-- Right Pane: Request Inspector & Replay -->
    <div class="pane-detail">
      <RequestDetail
        request={activeRequest}
        {tunnelStatus}
        onOpenReplay={handleOpenReplay}
        onCreateRuleFromRequest={handleCreateRuleFromRequest}
        onDeleteRequest={handleDeleteRequest}
      />
    </div>
  </main>

  <!-- Modals -->
  <EndpointModal
    isOpen={isEndpointModalOpen}
    endpoint={editingEndpoint}
    onClose={() => (isEndpointModalOpen = false)}
    onSave={handleSaveEndpoint}
  />

  <RulesModal
    isOpen={isRulesModalOpen}
    endpoint={activeRulesEndpoint}
    {rules}
    initialPrefill={rulePrefill}
    onClose={() => {
      isRulesModalOpen = false;
      activeRulesEndpoint = null;
      rulePrefill = null;
    }}
    onSaveRule={handleSaveRule}
    onDeleteRule={handleDeleteRule}
  />

  <ReplayModal
    isOpen={isReplayModalOpen}
    request={replayTargetRequest}
    onClose={() => {
      isReplayModalOpen = false;
      replayTargetRequest = null;
    }}
  />

  <InstallTunnelModal
    isOpen={isInstallModalOpen}
    onClose={() => (isInstallModalOpen = false)}
    onConfirm={executeStartTunnel}
  />

  <ApiForwarderModal
    isOpen={isForwarderModalOpen}
    {forwarders}
    onClose={() => (isForwarderModalOpen = false)}
    onSaveForwarder={handleSaveForwarder}
    onDeleteForwarder={handleDeleteForwarder}
    onStartForwarder={handleStartForwarder}
    onStopForwarder={handleStopForwarder}
  />
</div>
