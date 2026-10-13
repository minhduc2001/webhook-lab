// Xuất / nhập danh sách request dưới dạng JSON (định dạng riêng) hoặc HAR 1.2
import type { WebhookRequest } from '../types';

export type ExportFormat = 'json' | 'har';

interface HarHeader {
  name: string;
  value: string;
}

interface HarEntry {
  startedDateTime: string;
  time: number;
  request: {
    method: string;
    url: string;
    httpVersion: string;
    headers: HarHeader[];
    queryString: HarHeader[];
    postData?: { mimeType: string; text: string };
    headersSize: number;
    bodySize: number;
  };
  response: {
    status: number;
    statusText: string;
    httpVersion: string;
    headers: HarHeader[];
    content: { size: number; mimeType: string; text?: string };
    redirectURL: string;
    headersSize: number;
    bodySize: number;
  };
  cache: Record<string, never>;
  timings: { send: number; wait: number; receive: number };
  serverIPAddress?: string;
  _webhookLab?: Partial<WebhookRequest>;
}

const encoder = new TextEncoder();
const toHarHeaders = (h: Record<string, string> = {}): HarHeader[] =>
  Object.entries(h).map(([name, value]) => ({ name, value }));

function fromHarHeaders(list: HarHeader[] = [], lowercase = true): Record<string, string> {
  const out: Record<string, string> = {};
  for (const { name, value } of list) out[lowercase ? name.toLowerCase() : name] = String(value);
  return out;
}

function requestUrl(req: WebhookRequest, baseUrl: string): string {
  const qs = new URLSearchParams(req.query_params).toString();
  return `${baseUrl}${req.path}${qs ? `?${qs}` : ''}`;
}

function findHeader(h: Record<string, string> = {}, name: string): string {
  return Object.entries(h).find(([k]) => k.toLowerCase() === name)?.[1] ?? '';
}

export function toHar(requests: WebhookRequest[], baseUrl: string, appVersion: string): string {
  const entries: HarEntry[] = requests.map((req) => {
    const respBody = req.response_body ?? '';
    return {
      startedDateTime: req.timestamp,
      time: req.response_time_ms,
      request: {
        method: req.method,
        url: requestUrl(req, baseUrl),
        httpVersion: 'HTTP/1.1',
        headers: toHarHeaders(req.headers),
        queryString: toHarHeaders(req.query_params),
        ...(req.body ? { postData: { mimeType: req.content_type, text: req.body } } : {}),
        headersSize: -1,
        bodySize: encoder.encode(req.body).length,
      },
      response: {
        status: req.response_status,
        statusText: '',
        httpVersion: 'HTTP/1.1',
        headers: toHarHeaders(req.response_headers),
        content: {
          size: encoder.encode(respBody).length,
          mimeType: findHeader(req.response_headers, 'content-type'),
          ...(req.response_body != null ? { text: respBody } : {}),
        },
        redirectURL: findHeader(req.response_headers, 'location'),
        headersSize: -1,
        bodySize: req.response_body != null ? encoder.encode(respBody).length : -1,
      },
      cache: {},
      timings: { send: 0, wait: req.response_time_ms, receive: 0 },
      serverIPAddress: req.ip_address,
      // Giữ thông tin riêng để nhập lại không mất dữ liệu
      _webhookLab: {
        id: req.id,
        endpoint_id: req.endpoint_id,
        ip_address: req.ip_address,
        matched_rule_id: req.matched_rule_id,
        matched_rule_name: req.matched_rule_name,
        is_simulated_error: req.is_simulated_error,
      },
    };
  });

  return JSON.stringify(
    { log: { version: '1.2', creator: { name: 'Webhook Lab', version: appVersion }, entries } },
    null,
    2
  );
}

export function toJsonExport(requests: WebhookRequest[], appVersion: string): string {
  return JSON.stringify(
    { app: 'webhook-lab', version: appVersion, exported_at: new Date().toISOString(), requests },
    null,
    2
  );
}

function randomId(): string {
  return 'req-imp-' + Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

function fromHarEntry(entry: HarEntry): WebhookRequest {
  const extra = entry._webhookLab ?? {};
  let path = '/';
  let query: Record<string, string> = fromHarHeaders(entry.request.queryString, false);
  try {
    const url = new URL(entry.request.url);
    path = url.pathname;
    if (Object.keys(query).length === 0) query = Object.fromEntries(url.searchParams);
  } catch {
    path = entry.request.url || '/';
  }
  const headers = fromHarHeaders(entry.request.headers);
  return {
    id: extra.id ?? randomId(),
    endpoint_id: extra.endpoint_id ?? '',
    method: (entry.request.method || 'GET').toUpperCase(),
    path,
    query_params: query,
    headers,
    body: entry.request.postData?.text ?? '',
    content_type: entry.request.postData?.mimeType || headers['content-type'] || 'text/plain',
    ip_address: extra.ip_address ?? entry.serverIPAddress ?? 'imported',
    timestamp: entry.startedDateTime || new Date().toISOString(),
    response_status: entry.response?.status ?? 0,
    response_time_ms: Math.round(entry.time ?? 0),
    matched_rule_id: extra.matched_rule_id ?? null,
    matched_rule_name: extra.matched_rule_name ?? null,
    is_simulated_error: extra.is_simulated_error ?? false,
    response_headers: fromHarHeaders(entry.response?.headers, false),
    response_body: entry.response?.content?.text ?? null,
  };
}

// Chuẩn hoá một request từ file JSON, bổ sung các trường thiếu
function normalize(raw: any): WebhookRequest | null {
  if (!raw || typeof raw !== 'object' || typeof raw.method !== 'string') return null;
  return {
    id: typeof raw.id === 'string' && raw.id ? raw.id : randomId(),
    endpoint_id: typeof raw.endpoint_id === 'string' ? raw.endpoint_id : '',
    method: raw.method.toUpperCase(),
    path: typeof raw.path === 'string' ? raw.path : '/',
    query_params: raw.query_params && typeof raw.query_params === 'object' ? raw.query_params : {},
    headers: raw.headers && typeof raw.headers === 'object' ? raw.headers : {},
    body: typeof raw.body === 'string' ? raw.body : raw.body == null ? '' : JSON.stringify(raw.body),
    content_type: typeof raw.content_type === 'string' ? raw.content_type : 'text/plain',
    ip_address: typeof raw.ip_address === 'string' ? raw.ip_address : 'imported',
    timestamp: typeof raw.timestamp === 'string' ? raw.timestamp : new Date().toISOString(),
    response_status: Number(raw.response_status) || 0,
    response_time_ms: Number(raw.response_time_ms) || 0,
    matched_rule_id: raw.matched_rule_id ?? null,
    matched_rule_name: raw.matched_rule_name ?? null,
    is_simulated_error: !!raw.is_simulated_error,
    response_headers: raw.response_headers && typeof raw.response_headers === 'object' ? raw.response_headers : {},
    response_body: typeof raw.response_body === 'string' ? raw.response_body : null,
  };
}

// Đọc nội dung file (JSON của Webhook Lab, mảng request, hoặc HAR)
export function parseImport(text: string): WebhookRequest[] {
  let data: any;
  try {
    data = JSON.parse(text);
  } catch {
    throw new Error('File không phải JSON hợp lệ');
  }

  let list: WebhookRequest[];
  if (Array.isArray(data?.log?.entries)) {
    list = data.log.entries.map(fromHarEntry);
  } else {
    const raws: unknown[] = Array.isArray(data) ? data : Array.isArray(data?.requests) ? data.requests : [];
    list = raws.map(normalize).filter((r): r is WebhookRequest => r !== null);
  }

  if (list.length === 0) throw new Error('Không tìm thấy request nào trong file');
  return list;
}

function stamp(d = new Date()): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
}

export function exportFileName(scope: string, format: ExportFormat): string {
  return `webhook-lab-${scope}-${stamp()}.${format}`;
}
