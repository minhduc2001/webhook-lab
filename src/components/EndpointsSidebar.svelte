<script lang="ts">
  import type { Endpoint, TunnelStatus } from '../types';
  import { copyToClipboard } from '../lib/utils';
  import Icons from './Icons.svelte';

  let {
    endpoints,
    selectedEndpointId,
    tunnelStatus,
    onSelectEndpoint,
    onOpenCreateModal,
    onOpenEditModal,
    onOpenRulesModal,
    onDeleteEndpoint,
  } = $props<{
    endpoints: Endpoint[];
    selectedEndpointId: string | null;
    tunnelStatus: TunnelStatus;
    onSelectEndpoint: (id: string | null) => void;
    onOpenCreateModal: () => void;
    onOpenEditModal: (ep: Endpoint) => void;
    onOpenRulesModal: (ep: Endpoint) => void;
    onDeleteEndpoint: (id: string) => void;
  }>();

  let searchQuery = $state('');
  let copiedUrlId = $state<string | null>(null);

  let filteredEndpoints = $derived(
    endpoints.filter(
      (e: Endpoint) =>
        e.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        e.slug.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  let selectedEndpoint = $derived(
    endpoints.find((e: Endpoint) => e.id === selectedEndpointId) || null
  );

  async function handleCopyWebhookUrl(e: MouseEvent, ep: Endpoint) {
    e.stopPropagation();
    const base = tunnelStatus.is_running && tunnelStatus.public_url
      ? tunnelStatus.public_url
      : 'http://localhost:4567';
    const fullUrl = `${base}/wh/${ep.slug}`;
    const ok = await copyToClipboard(fullUrl);
    if (ok) {
      copiedUrlId = ep.id;
      setTimeout(() => (copiedUrlId = null), 2000);
    }
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <span class="section-title">Endpoints ({endpoints.length})</span>
    <button class="btn-add-ep" onclick={onOpenCreateModal} title="Tạo Endpoint Mới">
      <Icons name="plus" size={12} />
      <span>Mới</span>
    </button>
  </div>

  <div class="search-box">
    <Icons name="search" size={12} color="#64748b" />
    <input
      type="text"
      placeholder="Tìm endpoint..."
      bind:value={searchQuery}
      class="ep-search-input"
    />
  </div>

  <div class="endpoints-list">
    <!-- "All Endpoints" Option -->
    <button
      class="endpoint-item"
      class:selected={selectedEndpointId === null}
      onclick={() => onSelectEndpoint(null)}
    >
      <div class="ep-header-row">
        <span class="ep-name">Tất Cả Endpoints</span>
        <span class="ep-badge">ALL</span>
      </div>
    </button>

    {#each filteredEndpoints as ep (ep.id)}
      <div
        class="endpoint-item"
        class:selected={selectedEndpointId === ep.id}
        onclick={() => onSelectEndpoint(ep.id)}
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === 'Enter' && onSelectEndpoint(ep.id)}
      >
        <div class="ep-header-row">
          <span class="ep-name" title={ep.name}>{ep.name}</span>
          <div class="ep-top-badges">
            <span class="status-code-badge" class:error={ep.default_status >= 400}>
              {ep.default_status}
            </span>
            {#if (ep.request_count ?? 0) > 0}
              <span class="count-pill">{ep.request_count}</span>
            {/if}
          </div>
        </div>

        <div class="ep-slug-row">
          <div class="slug-and-methods">
            <span class="ep-slug" title={`/wh/${ep.slug}`}>/wh/{ep.slug}</span>
            <div class="methods-mini-row">
              {#each (ep.allowed_methods || ['ANY']) as m}
                <span class="method-mini-tag">{m}</span>
              {/each}
            </div>
          </div>

          <button
            class="btn-copy-url"
            onclick={(e) => handleCopyWebhookUrl(e, ep)}
            title="Sao chép URL webhook đầy đủ"
          >
            <Icons
              name={copiedUrlId === ep.id ? 'check' : 'copy'}
              size={11}
              color={copiedUrlId === ep.id ? '#34d399' : '#64748b'}
            />
          </button>
        </div>

        <div class="ep-footer">
          <div class="ep-indicators">
            {#if ep.redirect_url}
              <span class="indicator-tag redirect" title={`Chuyển hướng HTTP sang: ${ep.redirect_url}`}>
                🔀 {ep.redirect_status || 302}
              </span>
            {/if}
            {#if ep.auto_forward_url}
              <span class="indicator-tag forward" title={`Tự động chuyển tiếp (Relay) sang: ${ep.auto_forward_url}`}>
                ↗️ Relay
              </span>
            {/if}
            {#if ep.delay_ms > 0}
              <span class="indicator-tag delay" title="Độ trễ giả lập">
                <Icons name="clock" size={10} />
                {ep.delay_ms}ms
              </span>
            {/if}
            {#if ep.error_rate_percent > 0}
              <span
                class="indicator-tag error-rate"
                title={`Mô phỏng lỗi ngẫu nhiên: ${ep.error_rate_percent}% trả về mã ${ep.error_status}`}
              >
                <Icons name="zap" size={10} color="#fbbf24" />
                {ep.error_rate_percent}% lỗi ({ep.error_status})
              </span>
            {/if}
          </div>

          <div class="ep-actions-row">
            <button
              class="btn-item-action"
              onclick={(e) => {
                e.stopPropagation();
                onOpenRulesModal(ep);
              }}
              title="Quy tắc phản hồi có điều kiện (Rules)"
            >
              <Icons name="sliders" size={12} color="#94a3b8" />
            </button>
            <button
              class="btn-item-action"
              onclick={(e) => {
                e.stopPropagation();
                onOpenEditModal(ep);
              }}
              title="Chỉnh sửa cấu hình endpoint"
            >
              <Icons name="settings" size={12} color="#94a3b8" />
            </button>
            <button
              class="btn-item-action danger"
              onclick={(e) => {
                e.stopPropagation();
                if (confirm(`Bạn có chắc chắn muốn xóa endpoint "${ep.name}"?`)) {
                  onDeleteEndpoint(ep.id);
                }
              }}
              title="Xóa endpoint này"
            >
              <Icons name="trash" size={12} color="#f87171" />
            </button>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Footer Quick Target Display -->
  <div class="sidebar-footer">
    <div class="active-url-box">
      <div class="url-type-row">
        <span class="url-type-label">URL Endpoint</span>
        <span class="env-pill" class:online={tunnelStatus.is_running}>
          {tunnelStatus.is_running ? 'HTTPS' : 'LOCAL'}
        </span>
      </div>
      <div class="target-url-display">
        {#if selectedEndpoint}
          {#if tunnelStatus.is_running && tunnelStatus.public_url}
            {tunnelStatus.public_url}/wh/{selectedEndpoint.slug}
          {:else}
            http://localhost:4567/wh/{selectedEndpoint.slug}
          {/if}
        {:else}
          {#if tunnelStatus.is_running && tunnelStatus.public_url}
            {tunnelStatus.public_url}/wh/&lt;slug&gt;
          {:else}
            http://localhost:4567/wh/&lt;slug&gt;
          {/if}
        {/if}
      </div>
    </div>
  </div>
</aside>

<style>
  .btn-add-ep {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: 4px;
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-add-ep:hover {
    background: rgba(59, 130, 246, 0.25);
    border-color: #3b82f6;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid var(--border-subtle);
  }

  .ep-search-input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-main);
    font-size: 11.5px;
    width: 100%;
  }

  .ep-top-badges {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-code-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .status-code-badge.error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  .count-pill {
    font-family: var(--font-mono);
    font-size: 9.5px;
    padding: 1px 4px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
  }

  .ep-slug-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
  }

  .slug-and-methods {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .methods-mini-row {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .method-mini-tag {
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 700;
    padding: 0 3px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.07);
    color: var(--text-muted);
  }

  .btn-copy-url {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
  }

  .btn-copy-url:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .indicator-tag {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 9.5px;
    font-family: var(--font-mono);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .indicator-tag.redirect {
    background: rgba(168, 85, 247, 0.15);
    color: #c084fc;
    border: 1px solid rgba(168, 85, 247, 0.25);
  }

  .indicator-tag.forward {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    border: 1px solid rgba(56, 189, 248, 0.25);
  }

  .indicator-tag.delay {
    background: rgba(59, 130, 246, 0.12);
    color: #60a5fa;
  }

  .indicator-tag.error-rate {
    background: rgba(245, 158, 11, 0.12);
    color: #fbbf24;
  }

  .ep-indicators {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 4px;
  }

  .ep-actions-row {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0.6;
    transition: opacity 0.15s;
  }

  .endpoint-item:hover .ep-actions-row {
    opacity: 1;
  }

  .btn-item-action {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 3px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-item-action:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .btn-item-action.danger:hover {
    background: rgba(239, 68, 68, 0.15);
  }

  .sidebar-footer {
    padding: 10px 12px;
    border-top: 1px solid var(--border-subtle);
    background: rgba(10, 14, 26, 0.6);
  }

  .active-url-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .url-type-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .url-type-label {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-dim);
  }

  .env-pill {
    font-size: 9px;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
  }

  .env-pill.online {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .target-url-display {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: #93c5fd;
    word-break: break-all;
    line-height: 1.3;
    max-height: 42px;
    overflow: hidden;
  }
</style>
