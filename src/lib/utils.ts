import type { WebhookRequest } from '../types';

export function formatTimeAgo(timestamp: string, now: number = Date.now()): string {
  try {
    const date = new Date(timestamp);
    const diffSec = Math.floor((now - date.getTime()) / 1000);

    if (diffSec < 5) return 'now';
    if (diffSec < 60) return `${diffSec}s ago`;
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
    return `${Math.floor(diffSec / 86400)}d ago`;
  } catch {
    return timestamp;
  }
}

function pad(n: number, len = 2): string {
  return String(n).padStart(len, '0');
}

// "13/10/2026 14:32:05.123" theo giờ địa phương
export function formatDateTime(timestamp: string, withMs = true): string {
  const d = new Date(timestamp);
  if (isNaN(d.getTime())) return timestamp;
  const date = `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
  const time = `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  return withMs ? `${date} ${time}.${pad(d.getMilliseconds(), 3)}` : `${date} ${time}`;
}

// "14:32:05" nếu cùng ngày, ngược lại "13/10 14:32:05"
export function formatClock(timestamp: string, now: number = Date.now()): string {
  const d = new Date(timestamp);
  if (isNaN(d.getTime())) return timestamp;
  const time = `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  return new Date(now).toDateString() === d.toDateString()
    ? time
    : `${pad(d.getDate())}/${pad(d.getMonth() + 1)} ${time}`;
}

const encoder = new TextEncoder();

// Số byte UTF-8 thực tế (String.length chỉ đếm ký tự UTF-16)
export function byteLength(str: string): number {
  return encoder.encode(str).length;
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function formatJson(str: string): { formatted: string; isValid: boolean; parsed?: unknown } {
  try {
    const parsed = JSON.parse(str);
    return { formatted: JSON.stringify(parsed, null, 2), isValid: true, parsed };
  } catch {
    return { formatted: str, isValid: false };
  }
}

export type BodyKind = 'json' | 'form' | 'text';

export interface ParsedBody {
  kind: BodyKind;
  value?: unknown;
}

function tryParseJson(str: string): { ok: true; value: unknown } | { ok: false } {
  try {
    return { ok: true, value: JSON.parse(str) };
  } catch {
    return { ok: false };
  }
}

// Giá trị form có thể là JSON lồng (vd. Slack gửi payload={...})
function formValue(raw: string): unknown {
  const t = raw.trim();
  if (t.startsWith('{') || t.startsWith('[')) {
    const r = tryParseJson(t);
    if (r.ok) return r.value;
  }
  return raw;
}

const FORM_LIKE = /^[\w.\-[\]%+]+=[^\s&]*(&[\w.\-[\]%+]+=[^\s&]*)*$/;

// Nhận dạng body là JSON, form-urlencoded hay text thường
export function parseBody(body: string, contentType = ''): ParsedBody {
  const trimmed = body.trim();
  if (!trimmed) return { kind: 'text' };

  const json = tryParseJson(trimmed);
  if (json.ok && json.value !== null && typeof json.value === 'object') {
    return { kind: 'json', value: json.value };
  }

  if (/x-www-form-urlencoded/i.test(contentType) || FORM_LIKE.test(trimmed)) {
    const obj: Record<string, unknown> = {};
    for (const [k, v] of new URLSearchParams(trimmed)) {
      const val = formValue(v);
      if (!(k in obj)) obj[k] = val;
      else if (Array.isArray(obj[k])) (obj[k] as unknown[]).push(val);
      else obj[k] = [obj[k], val];
    }
    if (Object.keys(obj).length > 0) return { kind: 'form', value: obj };
  }

  return json.ok ? { kind: 'json', value: json.value } : { kind: 'text' };
}

export interface TextSegment {
  text: string;
  hit: boolean;
}

// Tách chuỗi thành các đoạn khớp / không khớp với từ khoá (không phân biệt hoa thường)
export function splitMatches(text: string, query: string): TextSegment[] {
  if (!query) return [{ text, hit: false }];
  const lower = text.toLowerCase();
  const q = query.toLowerCase();
  const out: TextSegment[] = [];
  let pos = 0;
  for (let i = lower.indexOf(q); i !== -1; i = lower.indexOf(q, i + q.length)) {
    if (i > pos) out.push({ text: text.slice(pos, i), hit: false });
    out.push({ text: text.slice(i, i + q.length), hit: true });
    pos = i + q.length;
  }
  if (pos < text.length) out.push({ text: text.slice(pos), hit: false });
  return out;
}

export function countMatches(text: string, query: string): number {
  if (!query) return 0;
  return splitMatches(text, query).filter((s) => s.hit).length;
}

// Chuỗi hiển thị của một giá trị nguyên thuỷ trong cây JSON
export function jsonLeafText(value: unknown): string {
  return typeof value === 'string' ? JSON.stringify(value) : String(value);
}

export function pathKey(path: (string | number)[]): string {
  return JSON.stringify(path);
}

// Tìm trong cây: trả về các node cần mở (chứa kết quả bên trong) và tổng số kết quả
export function collectJsonMatches(root: unknown, query: string): { openPaths: Set<string>; count: number } {
  const openPaths = new Set<string>();
  let count = 0;
  if (!query) return { openPaths, count };

  function walk(value: unknown, path: (string | number)[]): boolean {
    if (value === null || typeof value !== 'object') {
      const c = countMatches(jsonLeafText(value), query);
      count += c;
      return c > 0;
    }
    let any = false;
    const entries: [string | number, unknown][] = Array.isArray(value)
      ? value.map((v, i) => [i, v])
      : Object.entries(value as Record<string, unknown>);
    for (const [k, child] of entries) {
      const keyHits = typeof k === 'string' ? countMatches(k, query) : 0;
      count += keyHits;
      const childHit = walk(child, [...path, k]);
      if (keyHits > 0 || childHit) any = true;
    }
    if (any) openPaths.add(pathKey(path));
    return any;
  }

  walk(root, []);
  return { openPaths, count };
}

// Đường dẫn dạng "data.items.0.id" giống cách Rule JSON Body đọc;
// nếu key có ký tự đặc biệt thì dùng JSON pointer "/a.b/c~1d"
export function toJsonPath(segments: (string | number)[]): string {
  const parts = segments.map(String);
  if (parts.every((p) => p !== '' && !/[./~]/.test(p))) return parts.join('.');
  return parts.map((p) => '/' + p.replace(/~/g, '~0').replace(/\//g, '~1')).join('');
}

export function getStatusColor(status: number): { bg: string; text: string; border: string; glow: string } {
  if (status >= 200 && status < 300) {
    return {
      bg: 'rgba(16, 185, 129, 0.12)',
      text: '#34d399',
      border: 'rgba(16, 185, 129, 0.3)',
      glow: 'rgba(16, 185, 129, 0.25)',
    };
  }
  if (status >= 300 && status < 400) {
    return {
      bg: 'rgba(59, 130, 246, 0.12)',
      text: '#60a5fa',
      border: 'rgba(59, 130, 246, 0.3)',
      glow: 'rgba(59, 130, 246, 0.25)',
    };
  }
  if (status >= 400 && status < 500) {
    return {
      bg: 'rgba(245, 158, 11, 0.12)',
      text: '#fbbf24',
      border: 'rgba(245, 158, 11, 0.3)',
      glow: 'rgba(245, 158, 11, 0.25)',
    };
  }
  return {
    bg: 'rgba(239, 68, 68, 0.12)',
    text: '#f87171',
    border: 'rgba(239, 68, 68, 0.3)',
    glow: 'rgba(239, 68, 68, 0.25)',
  };
}

export function getMethodColor(method: string): { bg: string; text: string; border: string } {
  switch (method.toUpperCase()) {
    case 'GET':
      return { bg: 'rgba(59, 130, 246, 0.15)', text: '#60a5fa', border: 'rgba(59, 130, 246, 0.35)' };
    case 'POST':
      return { bg: 'rgba(16, 185, 129, 0.15)', text: '#34d399', border: 'rgba(16, 185, 129, 0.35)' };
    case 'PUT':
      return { bg: 'rgba(245, 158, 11, 0.15)', text: '#fbbf24', border: 'rgba(245, 158, 11, 0.35)' };
    case 'PATCH':
      return { bg: 'rgba(168, 85, 247, 0.15)', text: '#c084fc', border: 'rgba(168, 85, 247, 0.35)' };
    case 'DELETE':
      return { bg: 'rgba(239, 68, 68, 0.15)', text: '#f87171', border: 'rgba(239, 68, 68, 0.35)' };
    default:
      return { bg: 'rgba(148, 163, 184, 0.15)', text: '#cbd5e1', border: 'rgba(148, 163, 184, 0.35)' };
  }
}

// Bọc trong nháy đơn để shell không diễn giải $, `, \ ...
function shellQuote(str: string): string {
  return "'" + str.replace(/'/g, "'\\''") + "'";
}

export function generateCurl(request: WebhookRequest, baseUrl: string): string {
  const fullUrl = `${baseUrl}${request.path}`;
  const parts: string[] = [`curl -X ${request.method} ${shellQuote(fullUrl)}`];

  for (const [k, v] of Object.entries(request.headers)) {
    if (!['host', 'content-length'].includes(k.toLowerCase())) {
      parts.push(`  -H ${shellQuote(`${k}: ${v}`)}`);
    }
  }

  if (request.body && request.method !== 'GET') {
    parts.push(`  --data-raw ${shellQuote(request.body)}`);
  }

  return parts.join(' \\\n');
}

export function generateFetchCode(request: WebhookRequest, baseUrl: string): string {
  const fullUrl = `${baseUrl}${request.path}`;
  const options: Record<string, any> = {
    method: request.method,
    headers: request.headers,
  };
  if (request.body && request.method !== 'GET') {
    try {
      options.body = JSON.parse(request.body);
    } catch {
      options.body = request.body;
    }
  }
  return `fetch('${fullUrl}', ${JSON.stringify(options, null, 2)})
  .then(res => res.json())
  .then(data => console.log(data))
  .catch(err => console.error(err));`;
}

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (err) {
    console.error('Failed to copy text:', err);
    return false;
  }
}
