<script lang="ts">
  import type { WebhookRequest, ReplayResponse } from '../types';
  import { api } from '../lib/api';
  import { getStatusColor, formatJson, copyToClipboard } from '../lib/utils';
  import Icons from './Icons.svelte';

  let {
    isOpen,
    request,
    onClose,
  } = $props<{
    isOpen: boolean;
    request: WebhookRequest | null;
    onClose: () => void;
  }>();

  let targetUrl = $state('http://localhost:3000/webhook');
  let method = $state('POST');
  let headersText = $state('');
  let bodyText = $state('');
  let isLoading = $state(false);
  let replayResult = $state<ReplayResponse | null>(null);

  $effect(() => {
    if (isOpen && request) {
      targetUrl = 'http://localhost:3000/api/webhook';
      method = request.method;
      bodyText = request.body;
      replayResult = null;

      // Filter out hop-by-hop headers for replay
      const cleanHeaders: Record<string, string> = {};
      for (const [k, v] of Object.entries(request.headers)) {
        if (!['host', 'content-length', 'connection'].includes(k.toLowerCase())) {
          cleanHeaders[k] = String(v);
        }
      }
      headersText = JSON.stringify(cleanHeaders, null, 2);
    }
  });

  async function handleSendReplay() {
    if (!targetUrl.trim() || !request) return;

    isLoading = true;
    replayResult = null;

    let parsedHeaders: Record<string, string> = {};
    try {
      parsedHeaders = JSON.parse(headersText);
    } catch {
      parsedHeaders = { 'Content-Type': 'application/json' };
    }

    try {
      const res = await api.replayRequest({
        request_id: request.id,
        target_url: targetUrl.trim(),
        method,
        headers: parsedHeaders,
        body: bodyText,
      });
      replayResult = res;
    } catch (err: any) {
      replayResult = {
        status: 0,
        status_text: 'Failed',
        headers: {},
        body: '',
        time_ms: 0,
        error: err?.message || 'Error executing request replay',
      };
    } finally {
      isLoading = false;
    }
  }
</script>

{#if isOpen && request}
  <div
    class="modal-overlay"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="modal-card replay-card"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <div class="title-col">
          <h3 class="modal-title">Gửi Lại Yêu Cầu (Replay Webhook)</h3>
          <span class="modal-subtitle">Chuyển tiếp payload đã bắt được tới server backend cục bộ hoặc server test</span>
        </div>
        <button class="btn-close" onclick={onClose} aria-label="Đóng">
          <Icons name="x" size={16} />
        </button>
      </div>

      <div class="modal-body">
        <!-- Target Endpoint Configuration -->
        <div class="target-row">
          <select class="form-select method-select" bind:value={method}>
            <option value="POST">POST</option>
            <option value="GET">GET</option>
            <option value="PUT">PUT</option>
            <option value="PATCH">PATCH</option>
            <option value="DELETE">DELETE</option>
          </select>
          <input
            type="text"
            class="form-input url-input mono"
            placeholder="http://localhost:3000/api/webhook"
            bind:value={targetUrl}
          />
          <button
            class="btn-primary"
            onclick={handleSendReplay}
            disabled={isLoading || !targetUrl}
          >
            {#if isLoading}
              <Icons name="refresh" size={13} class="spin" />
              <span>Đang gửi...</span>
            {:else}
              <Icons name="play" size={13} color="#ffffff" />
              <span>Thực Thi Replay</span>
            {/if}
          </button>
        </div>

        <!-- Editable Payload and Headers -->
        <div class="editors-split">
          <div class="form-group flex-1">
            <label class="form-label" for="replay-body-input">Nội Dung Request Body (Có thể chỉnh sửa)</label>
            <textarea
              id="replay-body-input"
              rows="7"
              class="form-textarea"
              bind:value={bodyText}
            ></textarea>
          </div>

          <div class="form-group flex-1">
            <label class="form-label" for="replay-headers-input">Headers (JSON)</label>
            <textarea
              id="replay-headers-input"
              rows="7"
              class="form-textarea"
              bind:value={headersText}
            ></textarea>
          </div>
        </div>

        <!-- Replay Result Section -->
        {#if replayResult}
          {@const sColors = getStatusColor(replayResult.status)}
          <div class="replay-result-container">
            <div class="result-top-bar">
              <div class="result-headline">
                <span class="result-label">Kết Quả Phản Hồi:</span>
                {#if replayResult.status > 0}
                  <span
                    class="status-tag"
                    style="background: {sColors.bg}; color: {sColors.text}; border: 1px solid {sColors.border};"
                  >
                    {replayResult.status} {replayResult.status_text}
                  </span>
                  <span class="time-tag">{replayResult.time_ms} ms</span>
                {:else}
                  <span class="status-tag error">Kết Nối Thất Bại</span>
                {/if}
              </div>
            </div>

            {#if replayResult.error}
              <div class="error-banner">
                <Icons name="alert-circle" size={14} color="#f87171" />
                <span>{replayResult.error}</span>
              </div>
            {/if}

            {#if replayResult.body}
              <div class="result-body-box">
                <div class="result-body-header">
                  <span>Dữ Liệu Phản Hồi Từ Server Backend (Response Body)</span>
                  <button
                    class="btn-copy-sm"
                    title="Sao chép"
                    onclick={() => copyToClipboard(replayResult?.body || '')}
                  >
                    <Icons name="copy" size={11} />
                  </button>
                </div>
                <pre class="code-box"><code>{formatJson(replayResult.body).formatted}</code></pre>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" onclick={onClose}>Đóng</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .replay-card {
    max-width: 720px;
  }

  .title-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .modal-subtitle {
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 4px;
  }

  .target-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .method-select {
    width: 100px;
    font-family: var(--font-mono);
    font-weight: 700;
  }

  .url-input {
    flex: 1;
  }

  .mono {
    font-family: var(--font-mono);
  }

  .editors-split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .flex-1 {
    flex: 1;
  }

  .replay-result-container {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-medium);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .result-top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .result-headline {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .result-label {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .time-tag {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-dim);
  }

  .status-tag.error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #f87171;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.25);
    padding: 8px 12px;
    border-radius: 6px;
  }

  .result-body-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .result-body-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
  }

  .btn-copy-sm {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px;
    color: var(--text-muted);
  }

  .btn-copy-sm:hover {
    color: var(--text-main);
  }
</style>
