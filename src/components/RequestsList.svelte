<script lang="ts">
  import type { WebhookRequest } from '../types';
  import { formatTimeAgo, formatBytes, getMethodColor, getStatusColor } from '../lib/utils';
  import Icons from './Icons.svelte';

  let {
    requests,
    selectedRequestId,
    onSelectRequest,
    onDeleteRequest,
  } = $props<{
    requests: WebhookRequest[];
    selectedRequestId: string | null;
    onSelectRequest: (id: string) => void;
    onDeleteRequest: (id: string) => void;
  }>();

  let searchQuery = $state('');
  let methodFilter = $state('ALL');
  let statusFilter = $state('ALL');

  let filteredRequests = $derived(
    requests.filter((r: WebhookRequest) => {
      const matchMethod = methodFilter === 'ALL' || r.method.toUpperCase() === methodFilter;
      const matchStatus =
        statusFilter === 'ALL' ||
        (statusFilter === '2xx' && r.response_status >= 200 && r.response_status < 300) ||
        (statusFilter === '4xx' && r.response_status >= 400 && r.response_status < 500) ||
        (statusFilter === '5xx' && r.response_status >= 500);

      const q = searchQuery.toLowerCase();
      const matchSearch =
        !q ||
        r.path.toLowerCase().includes(q) ||
        r.ip_address.includes(q) ||
        r.body.toLowerCase().includes(q) ||
        (r.matched_rule_name && r.matched_rule_name.toLowerCase().includes(q));

      return matchMethod && matchStatus && matchSearch;
    })
  );
</script>

<div class="requests-pane">
  <div class="requests-header">
    <div class="requests-header-top">
      <span class="section-title">
        Requests ({filteredRequests.length}{#if requests.length !== filteredRequests.length}/{requests.length}{/if})
      </span>
      <div class="filters-row">
        <select class="select-sm" bind:value={methodFilter}>
          <option value="ALL">Method</option>
          <option value="POST">POST</option>
          <option value="GET">GET</option>
          <option value="PUT">PUT</option>
          <option value="PATCH">PATCH</option>
          <option value="DELETE">DELETE</option>
        </select>
        <select class="select-sm" bind:value={statusFilter}>
          <option value="ALL">Status</option>
          <option value="2xx">2xx</option>
          <option value="4xx">4xx</option>
          <option value="5xx">5xx</option>
        </select>
      </div>
    </div>

    <div class="filter-bar">
      <Icons name="search" size={12} color="#64748b" />
      <input
        type="text"
        placeholder="Lọc request..."
        bind:value={searchQuery}
        class="search-input"
      />
      {#if searchQuery}
        <button class="btn-clear-search" onclick={() => (searchQuery = '')} title="Xóa bộ lọc">
          <Icons name="x" size={11} />
        </button>
      {/if}
    </div>
  </div>

  <div class="requests-list">
    {#if filteredRequests.length === 0}
      <div class="empty-requests">
        <div class="empty-icon-box">
          <Icons name="refresh" size={22} color="#64748b" />
        </div>
        <span class="empty-title">Chờ Webhook...</span>
        <span class="empty-desc">
          Gửi HTTP request tới endpoint để bắt đầu ghi nhận
        </span>
      </div>
    {:else}
      {#each filteredRequests as req (req.id)}
        {@const mColors = getMethodColor(req.method)}
        {@const sColors = getStatusColor(req.response_status)}
        <div
          class="request-card"
          class:selected={selectedRequestId === req.id}
          onclick={() => onSelectRequest(req.id)}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && onSelectRequest(req.id)}
        >
          <div class="req-row-1">
            <div class="req-method-path">
              <span
                class="method-tag"
                style="background: {mColors.bg}; color: {mColors.text}; border: 1px solid {mColors.border};"
              >
                {req.method}
              </span>
              <span class="req-path" title={req.path}>{req.path}</span>
            </div>

            <span
              class="status-tag"
              style="background: {sColors.bg}; color: {sColors.text}; border: 1px solid {sColors.border};"
            >
              {req.response_status}
            </span>
          </div>

          <div class="req-row-2">
            <div class="req-meta-left">
              <span class="req-time" title={req.timestamp}>{formatTimeAgo(req.timestamp)}</span>
              <span class="req-sep">•</span>
              <span class="req-size">{formatBytes(req.body.length)}</span>
              <span class="req-sep">•</span>
              <span class="req-ip">{req.ip_address}</span>
            </div>

            <button
              class="btn-delete-req"
              onclick={(e) => {
                e.stopPropagation();
                onDeleteRequest(req.id);
              }}
              title="Xóa request này"
            >
              <Icons name="trash" size={11} color="#64748b" />
            </button>
          </div>

          {#if req.is_simulated_error}
            <div class="req-chaos-tag" title="Phản hồi này được sinh ngẫu nhiên bởi bộ mô phỏng lỗi Chaos">
              <Icons name="zap" size={10} color="#f59e0b" />
              <span>⚡ Lỗi Ngẫu Nhiên ({req.response_status})</span>
            </div>
          {/if}

          {#if req.matched_rule_name}
            <div class="req-rule-tag" title={`Khớp quy tắc: ${req.matched_rule_name}`}>
              <Icons name="sliders" size={10} />
              <span>Quy tắc: {req.matched_rule_name}</span>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .filters-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .select-sm {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 11px;
    color: var(--text-muted);
    outline: none;
    font-family: inherit;
  }

  .select-sm:focus {
    border-color: var(--border-focus);
  }

  .btn-clear-search {
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--text-dim);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
  }

  .btn-clear-search:hover {
    color: var(--text-main);
  }

  .empty-requests {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    gap: 10px;
    height: 100%;
  }

  .empty-icon-box {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.04);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-title {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .empty-desc {
    font-size: 11.5px;
    color: var(--text-dim);
    line-height: 1.5;
    max-width: 250px;
  }

  .req-meta-left {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    color: var(--text-dim);
  }

  .req-sep {
    color: rgba(255, 255, 255, 0.15);
  }

  .btn-delete-req {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .request-card:hover .btn-delete-req {
    opacity: 0.8;
  }

  .btn-delete-req:hover {
    opacity: 1 !important;
    background: rgba(239, 68, 68, 0.15);
  }

  .req-rule-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    align-self: flex-start;
    margin-top: 2px;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 3px;
    background: rgba(168, 85, 247, 0.12);
    color: #c084fc;
    border: 1px solid rgba(168, 85, 247, 0.25);
  }

  .req-chaos-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    align-self: flex-start;
    margin-top: 2px;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 3px;
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
</style>
