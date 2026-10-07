import type { WebhookRequest } from '../types';

export function formatTimeAgo(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    const now = new Date();
    const diffSec = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffSec < 5) return 'vừa xong';
    if (diffSec < 60) return `${diffSec} giây trước`;
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)} phút trước`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)} giờ trước`;
    return date.toLocaleDateString('vi-VN');
  } catch {
    return timestamp;
  }
}

export function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
  } catch {
    return timestamp;
  }
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function formatJson(str: string): { formatted: string; isValid: boolean } {
  try {
    const parsed = JSON.parse(str);
    return { formatted: JSON.stringify(parsed, null, 2), isValid: true };
  } catch {
    return { formatted: str, isValid: false };
  }
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

export function generateCurl(request: WebhookRequest, baseUrl: string): string {
  const fullUrl = `${baseUrl}${request.path}`;
  const parts: string[] = [`curl -X ${request.method} "${fullUrl}"`];

  for (const [k, v] of Object.entries(request.headers)) {
    if (!['host', 'content-length'].includes(k.toLowerCase())) {
      parts.push(`  -H "${k}: ${v}"`);
    }
  }

  if (request.body && request.method !== 'GET') {
    const escaped = request.body.replace(/"/g, '\\"');
    parts.push(`  -d "${escaped}"`);
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
