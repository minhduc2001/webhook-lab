<script lang="ts">
  import { untrack } from 'svelte';
  import type { WebhookRequest } from '../types';
  import {
    SIGNATURE_SCHEMES,
    detectScheme,
    verifyCustom,
    type CustomSignatureConfig,
    type SignatureResult,
  } from '../lib/signature';
  import Icons from './Icons.svelte';

  let { request } = $props<{ request: WebhookRequest }>();

  interface SavedConfig {
    secret: string;
    custom: CustomSignatureConfig;
  }

  // Secret lưu cục bộ theo từng endpoint để không phải nhập lại
  const storageKey = untrack(() => `webhook-lab:signature:${request.endpoint_id}`);

  function loadConfig(): Partial<SavedConfig> {
    try {
      return JSON.parse(localStorage.getItem(storageKey) ?? '{}');
    } catch {
      return {};
    }
  }

  function guessSignatureHeader(headers: Record<string, string>): string {
    return Object.keys(headers).find((k) => /sign|hmac/i.test(k)) ?? '';
  }

  const saved = loadConfig();
  const detected = untrack(() => detectScheme(request.headers));

  let schemeId = $state(detected?.id ?? 'custom');
  let secret = $state(saved.secret ?? '');
  let showSecret = $state(false);
  let expanded = $state(detected !== null);
  let custom = $state<CustomSignatureConfig>({
    algo: saved.custom?.algo ?? 'SHA-256',
    encoding: saved.custom?.encoding ?? 'hex',
    header: untrack(() => guessSignatureHeader(request.headers)) || saved.custom?.header || '',
  });
  let result = $state<SignatureResult | null>(null);
  let error = $state<string | null>(null);

  let headerNames = $derived(Object.keys(request.headers));

  $effect(() => {
    const config: SavedConfig = { secret, custom: { ...custom } };
    try {
      localStorage.setItem(storageKey, JSON.stringify(config));
    } catch {
      // localStorage có thể bị chặn; chỉ mất tiện ích ghi nhớ
    }
  });

  let runId = 0;
  $effect(() => {
    const id = ++runId;
    const s = secret;
    const scheme = SIGNATURE_SCHEMES.find((x) => x.id === schemeId);
    const cfg = { ...custom };
    const { headers, body } = request;

    if (!s || (!scheme && !cfg.header)) {
      result = null;
      error = null;
      return;
    }

    const pending = scheme ? scheme.verify(s, headers, body) : verifyCustom(cfg, s, headers, body);
    pending
      .then((r) => {
        if (id !== runId) return;
        result = r;
        error = null;
      })
      .catch((e) => {
        if (id !== runId) return;
        result = null;
        error = String(e?.message ?? e);
      });
  });
</script>

<div class="sig-panel" class:valid={result?.valid} class:invalid={result && !result.valid}>
  <button class="sig-head" onclick={() => (expanded = !expanded)}>
    <Icons name="shield" size={12} color={result ? (result.valid ? '#34d399' : '#f87171') : '#94a3b8'} />
    <span class="sig-title">Kiểm tra chữ ký</span>
    {#if detected}
      <span class="sig-detected">{detected.label}</span>
    {/if}
    {#if result}
      <span class="sig-status" class:ok={result.valid}>
        {result.valid ? '✓ Hợp lệ' : '✗ Không khớp'}
      </span>
    {:else if error}
      <span class="sig-status">Lỗi</span>
    {:else if expanded}
      <span class="sig-hint">Nhập secret để kiểm tra</span>
    {/if}
    <span class="sig-chevron" class:open={expanded}>▸</span>
  </button>

  {#if expanded}
    <div class="sig-body">
      <div class="sig-row">
        <select class="sig-input sig-select" bind:value={schemeId}>
          {#each SIGNATURE_SCHEMES as s}
            <option value={s.id}>{s.label}</option>
          {/each}
          <option value="custom">Tuỳ chỉnh (HMAC body)</option>
        </select>
        <input
          class="sig-input sig-secret"
          type={showSecret ? 'text' : 'password'}
          placeholder="Webhook secret"
          bind:value={secret}
          spellcheck="false"
          autocomplete="off"
        />
        <button class="sig-btn" onclick={() => (showSecret = !showSecret)}>
          {showSecret ? 'Ẩn' : 'Hiện'}
        </button>
      </div>

      {#if schemeId === 'custom'}
        <div class="sig-row">
          <select class="sig-input sig-select" bind:value={custom.header} title="Header chứa chữ ký">
            <option value="" disabled>Chọn header chữ ký</option>
            {#each headerNames as h}
              <option value={h}>{h}</option>
            {/each}
          </select>
          <select class="sig-input" bind:value={custom.algo}>
            <option value="SHA-256">HMAC-SHA256</option>
            <option value="SHA-512">HMAC-SHA512</option>
            <option value="SHA-1">HMAC-SHA1</option>
          </select>
          <select class="sig-input" bind:value={custom.encoding}>
            <option value="hex">hex</option>
            <option value="base64">base64</option>
          </select>
        </div>
      {/if}

      {#if error}
        <div class="sig-detail error">{error}</div>
      {:else if result}
        <div class="sig-detail">
          <div><span class="sig-label">Nhận</span><code class:match={result.valid}>{result.received || '(trống)'}</code></div>
          <div><span class="sig-label">Tính ra</span><code class:match={result.valid}>{result.expected}</code></div>
          {#if result.note}
            <div><span class="sig-label">Ghi chú</span><span>{result.note}</span></div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .sig-panel {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    margin-bottom: 8px;
    overflow: hidden;
  }

  .sig-panel.valid {
    border-color: rgba(16, 185, 129, 0.35);
  }

  .sig-panel.invalid {
    border-color: rgba(239, 68, 68, 0.35);
  }

  .sig-head {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    border: none;
    color: var(--text-main);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
    text-align: left;
  }

  .sig-head:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .sig-title {
    font-weight: 600;
  }

  .sig-detected {
    font-size: 10px;
    padding: 0 5px;
    border-radius: 3px;
    background: rgba(99, 102, 241, 0.15);
    color: #a5b4fc;
  }

  .sig-status {
    font-size: 10.5px;
    font-weight: 600;
    color: #f87171;
  }

  .sig-status.ok {
    color: #34d399;
  }

  .sig-hint {
    font-size: 10.5px;
    color: var(--text-dim);
  }

  .sig-chevron {
    margin-left: auto;
    color: var(--text-dim);
    font-size: 10px;
    transition: transform 0.12s;
  }

  .sig-chevron.open {
    transform: rotate(90deg);
  }

  .sig-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0 10px 8px;
  }

  .sig-row {
    display: flex;
    gap: 6px;
  }

  .sig-input {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 3px 6px;
    font-size: 11px;
    color: var(--text-main);
    outline: none;
    font-family: inherit;
  }

  .sig-input:focus {
    border-color: var(--border-focus);
  }

  .sig-select {
    min-width: 0;
    max-width: 200px;
  }

  .sig-secret {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
  }

  .sig-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 3px 8px;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
  }

  .sig-btn:hover {
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.1);
  }

  .sig-detail {
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .sig-detail > div {
    display: flex;
    gap: 6px;
    min-width: 0;
  }

  .sig-detail.error {
    color: #f87171;
  }

  .sig-label {
    flex-shrink: 0;
    width: 52px;
    color: var(--text-dim);
  }

  .sig-detail code {
    font-family: var(--font-mono);
    color: #fca5a5;
    word-break: break-all;
  }

  .sig-detail code.match {
    color: #86efac;
  }
</style>
