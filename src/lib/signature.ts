// Kiểm tra chữ ký HMAC của webhook ngay trên client (Web Crypto)

export type HashAlgo = 'SHA-1' | 'SHA-256' | 'SHA-512';
export type DigestEncoding = 'hex' | 'base64';

export interface CustomSignatureConfig {
  header: string;
  algo: HashAlgo;
  encoding: DigestEncoding;
}

export interface SignatureResult {
  valid: boolean;
  received: string;
  expected: string;
  note?: string;
}

export interface SignatureScheme {
  id: string;
  label: string;
  header: string;
  verify: (secret: string, headers: Record<string, string>, body: string) => Promise<SignatureResult>;
}

const encoder = new TextEncoder();

function getHeader(headers: Record<string, string>, name: string): string | undefined {
  const lower = name.toLowerCase();
  for (const [k, v] of Object.entries(headers)) {
    if (k.toLowerCase() === lower) return v;
  }
  return undefined;
}

async function hmac(algo: HashAlgo, secret: string, message: string, encoding: DigestEncoding): Promise<string> {
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: algo },
    false,
    ['sign']
  );
  const sig = new Uint8Array(await crypto.subtle.sign('HMAC', key, encoder.encode(message)));
  if (encoding === 'base64') {
    let bin = '';
    for (const b of sig) bin += String.fromCharCode(b);
    return btoa(bin);
  }
  return Array.from(sig, (b) => b.toString(16).padStart(2, '0')).join('');
}

// Bỏ tiền tố kiểu "sha256=", "v0=", "v1="
function stripPrefix(value: string): string {
  return value.trim().replace(/^(sha1|sha256|sha512|v0|v1)=/i, '');
}

function sameDigest(a: string, b: string, encoding: DigestEncoding): boolean {
  return encoding === 'hex' ? a.toLowerCase() === b.toLowerCase() : a === b;
}

function prefixedHex(id: string, label: string, header: string, algo: HashAlgo): SignatureScheme {
  return {
    id,
    label,
    header,
    async verify(secret, headers, body) {
      const received = stripPrefix(getHeader(headers, header) ?? '');
      const expected = await hmac(algo, secret, body, 'hex');
      return { valid: sameDigest(received, expected, 'hex'), received, expected };
    },
  };
}

export const SIGNATURE_SCHEMES: SignatureScheme[] = [
  prefixedHex('github', 'GitHub (SHA-256)', 'x-hub-signature-256', 'SHA-256'),
  prefixedHex('github-sha1', 'GitHub (SHA-1, cũ)', 'x-hub-signature', 'SHA-1'),
  {
    id: 'stripe',
    label: 'Stripe',
    header: 'stripe-signature',
    async verify(secret, headers, body) {
      const raw = getHeader(headers, 'stripe-signature') ?? '';
      const parts = raw.split(',').map((p) => p.trim().split('='));
      const t = parts.find(([k]) => k === 't')?.[1] ?? '';
      const v1s = parts.filter(([k]) => k === 'v1').map(([, v]) => v);
      const expected = await hmac('SHA-256', secret, `${t}.${body}`, 'hex');
      const ageSec = Math.round(Date.now() / 1000 - Number(t));
      return {
        valid: v1s.some((v) => sameDigest(v, expected, 'hex')),
        received: v1s.join(', '),
        expected,
        note: t ? `t=${t} (${ageSec}s trước; Stripe mặc định từ chối nếu > 300s)` : 'Thiếu t= trong header',
      };
    },
  },
  {
    id: 'shopify',
    label: 'Shopify',
    header: 'x-shopify-hmac-sha256',
    async verify(secret, headers, body) {
      const received = (getHeader(headers, 'x-shopify-hmac-sha256') ?? '').trim();
      const expected = await hmac('SHA-256', secret, body, 'base64');
      return { valid: received === expected, received, expected };
    },
  },
  {
    id: 'slack',
    label: 'Slack',
    header: 'x-slack-signature',
    async verify(secret, headers, body) {
      const received = stripPrefix(getHeader(headers, 'x-slack-signature') ?? '');
      const ts = getHeader(headers, 'x-slack-request-timestamp') ?? '';
      const expected = await hmac('SHA-256', secret, `v0:${ts}:${body}`, 'hex');
      return { valid: sameDigest(received, expected, 'hex'), received, expected };
    },
  },
];

export function detectScheme(headers: Record<string, string>): SignatureScheme | null {
  return SIGNATURE_SCHEMES.find((s) => getHeader(headers, s.header) !== undefined) ?? null;
}

// Chế độ tự cấu hình: HMAC(body) với header / thuật toán / encoding tuỳ chọn
export async function verifyCustom(
  config: CustomSignatureConfig,
  secret: string,
  headers: Record<string, string>,
  body: string
): Promise<SignatureResult> {
  const received = stripPrefix(getHeader(headers, config.header) ?? '');
  const expected = await hmac(config.algo, secret, body, config.encoding);
  return { valid: sameDigest(received, expected, config.encoding), received, expected };
}
