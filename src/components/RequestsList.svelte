<script lang="ts">
  import { tick, untrack } from 'svelte';
  import type { WebhookRequest } from '../types';
  import { formatTimeAgo, formatClock, formatDateTime, formatBytes, byteLength, getMethodColor, getStatusColor } from '../lib/utils';
  import type { ExportFormat } from '../lib/transfer';
  import Icons from './Icons.svelte';

  let {
    requests,
    selectedRequestId,
    onSelectRequest,
    onDeleteRequest,
    compareBase = null,
    onCancelCompare,
    onExport,
    onImport,
    keyboardEnabled = true,
  } = $props<{
    requests: WebhookRequest[];
    selectedRequestId: string | null;
    onSelectRequest: (id: string) => void;
    onDeleteRequest: (id: string) => void;
    compareBase?: WebhookRequest | null;
    onCancelCompare: () => void;
    onExport: (format: ExportFormat, list: WebhookRequest[]) => void;
    onImport: (fileText: string) => void;
    keyboardEnabled?: boolean;
  }>();

  let searchQuery = $state('');
  let methodFilter = $state('ALL');
  let statusFilter = $state('ALL');
  // Cập nhật định kỳ để nhãn "x phút trước" không bị đứng yên
  let now = $state(Date.now());

  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 15000);
    return () => clearInterval(timer);
  });

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

  let searchEl = $state<HTMLInputElement>();
  let listEl = $state<HTMLElement>();
  let fileInput = $state<HTMLInputElement>();
  let exportMenuOpen = $state(false);

  // Nhãn nhóm theo ngày
  function dayKey(ts: string): string {
    return new Date(ts).toDateString();
  }

  function dayLabel(ts: string, nowMs: number): string {
    const d = new Date(ts);
    const today = new Date(nowMs);
    const yesterday = new Date(nowMs - 86400000);
    if (d.toDateString() === today.toDateString()) return 'Hôm nay';
    if (d.toDateString() === yesterday.toDateString()) return 'Hôm qua';
    return d.toLocaleDateString('vi-VN', { weekday: 'short', day: '2-digit', month: '2-digit', year: 'numeric' });
  }

  // Đánh dấu request vừa tới để nháy sáng
  const seenIds = new Set<string>();
  let initialized = false;
  let freshIds = $state<Set<string>>(new Set());

  $effect(() => {
    const list = requests;
    untrack(() => {
      const arrived = initialized
        ? list.filter((r: WebhookRequest) => !seenIds.has(r.id) && Date.now() - Date.parse(r.timestamp) < 10000)
        : [];
      for (const r of list) seenIds.add(r.id);
      initialized = true;
      if (arrived.length === 0) return;
      const ids = arrived.map((r: WebhookRequest) => r.id);
      freshIds = new Set([...freshIds, ...ids]);
      setTimeout(() => {
        freshIds = new Set([...freshIds].filter((id) => !ids.includes(id)));
      }, 2500);
    });
  });

  async function scrollSelectedIntoView() {
    await tick();
    listEl?.querySelector('.request-card.selected')?.scrollIntoView({ block: 'nearest' });
  }

  function onWindowKeydown(e: KeyboardEvent) {
    if (!keyboardEnabled || e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('input, textarea, select, [contenteditable="true"]')) return;

    if (e.key === '/') {
      e.preventDefault();
      searchEl?.focus();
      return;
    }

    const dir = e.key === 'j' || e.key === 'ArrowDown' ? 1 : e.key === 'k' || e.key === 'ArrowUp' ? -1 : 0;
    if (dir === 0 || filteredRequests.length === 0) return;
    e.preventDefault();
    const idx = filteredRequests.findIndex((r: WebhookRequest) => r.id === selectedRequestId);
    const next = idx === -1 ? 0 : Math.min(Math.max(idx + dir, 0), filteredRequests.length - 1);
    onSelectRequest(filteredRequests[next].id);
    scrollSelectedIntoView();
  }

  function onSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      searchQuery = '';
      searchEl?.blur();
    }
  }

  function chooseExport(format: ExportFormat) {
    exportMenuOpen = false;
    onExport(format, filteredRequests);
  }

  async function handleFileChosen(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (file) onImport(await file.text());
  }
</script>

<svelte:window
  onkeydown={onWindowKeydown}
  onclick={() => (exportMenuOpen = false)}
/>

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
        <div class="export-wrap">
          <button
            class="btn-icon-sm"
            onclick={(e) => {
              e.stopPropagation();
              exportMenuOpen = !exportMenuOpen;
            }}
            disabled={filteredRequests.length === 0}
            title="Xuất các request đang hiển thị"
          >
            <Icons name="download" size={12} />
          </button>
          {#if exportMenuOpen}
            <div class="export-menu">
              <button onclick={() => chooseExport('json')}>
                <strong>JSON</strong><span>Nhập lại được vào Webhook Lab</span>
              </button>
              <button onclick={() => chooseExport('har')}>
                <strong>HAR</strong><span>Mở bằng DevTools, Postman, Insomnia…</span>
              </button>
            </div>
          {/if}
        </div>
        <button class="btn-icon-sm" onclick={() => fileInput?.click()} title="Nhập request từ file JSON / HAR">
          <Icons name="upload" size={12} />
        </button>
        <input
          bind:this={fileInput}
          type="file"
          accept=".json,.har,application/json"
          onchange={handleFileChosen}
          hidden
        />
      </div>
    </div>

    <div class="filter-bar">
      <Icons name="search" size={12} color="#64748b" />
      <input
        bind:this={searchEl}
        type="text"
        placeholder="Lọc request... ( / )"
        bind:value={searchQuery}
        onkeydown={onSearchKeydown}
        class="search-input"
      />
      {#if searchQuery}
        <button class="btn-clear-search" onclick={() => (searchQuery = '')} title="Xóa bộ lọc">
          <Icons name="x" size={11} />
        </button>
      {/if}
    </div>
  </div>

  {#if compareBase}
    <div class="compare-banner">
      <Icons name="diff" size={12} />
      <span class="compare-text">
        Chọn request để so sánh với
        <strong>{compareBase.method} {formatClock(compareBase.timestamp, now)}</strong>
      </span>
      <button class="btn-cancel-compare" onclick={onCancelCompare} title="Huỷ (Esc)">Huỷ</button>
    </div>
  {/if}

  <div class="requests-list" bind:this={listEl}>
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
      {#each filteredRequests as req, i (req.id)}
        {@const mColors = getMethodColor(req.method)}
        {@const sColors = getStatusColor(req.response_status)}
        {#if i === 0 || dayKey(filteredRequests[i - 1].timestamp) !== dayKey(req.timestamp)}
          <div class="day-separator">{dayLabel(req.timestamp, now)}</div>
        {/if}
        <div
          class="request-card"
          class:selected={selectedRequestId === req.id}
          class:fresh={freshIds.has(req.id)}
          class:compare-base={compareBase?.id === req.id}
          class:compare-pick={compareBase && compareBase.id !== req.id}
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
              <span class="req-time" title={`Nhận lúc ${formatDateTime(req.timestamp)}`}>
                <Icons name="clock" size={10} />
                <span class="req-clock">{formatClock(req.timestamp, now)}</span>
                <span class="req-ago">{formatTimeAgo(req.timestamp, now)}</span>
              </span>
              <span class="req-sep">•</span>
              <span class="req-size">{formatBytes(byteLength(req.body))}</span>
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
  .export-wrap {
    position: relative;
  }

  .btn-icon-sm {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 3px 5px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .btn-icon-sm:hover:not(:disabled) {
    color: var(--text-main);
    border-color: var(--border-medium);
  }

  .btn-icon-sm:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .export-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 20;
    min-width: 220px;
    background: var(--bg-panel);
    border: 1px solid var(--border-medium);
    border-radius: 6px;
    box-shadow: var(--shadow-lg);
    padding: 4px;
    display: flex;
    flex-direction: column;
  }

  .export-menu button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 5px 8px;
    color: var(--text-main);
    font-size: 11px;
    cursor: pointer;
    text-align: left;
  }

  .export-menu button span {
    font-size: 10px;
    color: var(--text-dim);
  }

  .export-menu button:hover {
    background: var(--bg-card-hover);
  }

  .compare-banner {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    font-size: 11px;
    color: #93c5fd;
    background: rgba(59, 130, 246, 0.12);
    border-bottom: 1px solid rgba(59, 130, 246, 0.3);
  }

  .compare-text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .compare-text strong {
    font-family: var(--font-mono);
  }

  .btn-cancel-compare {
    background: transparent;
    border: 1px solid rgba(59, 130, 246, 0.4);
    border-radius: 4px;
    color: #93c5fd;
    font-size: 10.5px;
    padding: 1px 7px;
    cursor: pointer;
  }

  .day-separator {
    position: sticky;
    top: 0;
    z-index: 2;
    padding: 3px 10px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--text-dim);
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-subtle);
  }

  .request-card.fresh {
    animation: fresh-flash 2.5s ease-out;
  }

  @keyframes fresh-flash {
    0% {
      background: rgba(52, 211, 153, 0.22);
      box-shadow: inset 3px 0 0 #34d399;
    }
    100% {
      background: transparent;
      box-shadow: inset 3px 0 0 transparent;
    }
  }

  .request-card.compare-base {
    box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.6);
  }

  .request-card.compare-pick:hover {
    background: rgba(59, 130, 246, 0.1);
  }

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

  .req-time {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .req-clock {
    font-family: var(--font-mono);
    color: #a5b4fc;
  }

  .req-ago {
    color: var(--text-dim);
  }

  .req-ip {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .req-meta-left {
    min-width: 0;
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
