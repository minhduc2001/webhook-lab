<script lang="ts">
  import type { WebhookRequest, TunnelStatus } from '../types';
  import {
    generateCurl,
    generateFetchCode,
    copyToClipboard,
    getMethodColor,
    getStatusColor,
    formatDateTime,
    formatTimeAgo,
    formatBytes,
    byteLength,
  } from '../lib/utils';
  import Icons from './Icons.svelte';
  import BodyViewer from './BodyViewer.svelte';
  import SignaturePanel from './SignaturePanel.svelte';

  let {
    request,
    tunnelStatus,
    onOpenReplay,
    onCreateRuleFromRequest,
    onDeleteRequest,
    onStartCompare,
    isComparing = false,
  } = $props<{
    request: WebhookRequest | null;
    tunnelStatus: TunnelStatus;
    onOpenReplay: (req: WebhookRequest) => void;
    onCreateRuleFromRequest: (req: WebhookRequest) => void;
    onDeleteRequest: (id: string) => void;
    onStartCompare: (req: WebhookRequest) => void;
    isComparing?: boolean;
  }>();

  let activeTab = $state<'body' | 'headers' | 'query' | 'response' | 'overview' | 'code'>('body');
  let copiedCurl = $state(false);
  let copiedFetch = $state(false);
  let copiedHeaderKey = $state<string | null>(null);
  let now = $state(Date.now());

  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 15000);
    return () => clearInterval(timer);
  });

  let activeBaseUrl = $derived(
    tunnelStatus.is_running && tunnelStatus.public_url
      ? tunnelStatus.public_url
      : 'http://localhost:4567'
  );

  let bodyBytes = $derived(request ? byteLength(request.body) : 0);

  let curlCode = $derived(
    request ? generateCurl(request, activeBaseUrl) : ''
  );

  let fetchCode = $derived(
    request ? generateFetchCode(request, activeBaseUrl) : ''
  );

  async function handleCopyCurl() {
    if (!curlCode) return;
    const ok = await copyToClipboard(curlCode);
    if (ok) {
      copiedCurl = true;
      setTimeout(() => (copiedCurl = false), 2000);
    }
  }

  async function handleCopyFetch() {
    if (!fetchCode) return;
    const ok = await copyToClipboard(fetchCode);
    if (ok) {
      copiedFetch = true;
      setTimeout(() => (copiedFetch = false), 2000);
    }
  }

  function responseContentType(req: WebhookRequest): string {
    const entry = Object.entries(req.response_headers ?? {}).find(([k]) => k.toLowerCase() === 'content-type');
    return entry?.[1] ?? '';
  }

  async function handleCopyHeader(key: string, val: any) {
    const ok = await copyToClipboard(`${key}: ${val}`);
    if (ok) {
      copiedHeaderKey = key;
      setTimeout(() => (copiedHeaderKey = null), 1500);
    }
  }
</script>

<div class="detail-pane">
  {#if !request}
    <div class="empty-state">
      <div class="empty-icon">
        <Icons name="code" size={24} />
      </div>
      <h3>Chọn Một Request</h3>
      <p>Xem chi tiết headers, payload và phản hồi</p>
    </div>
  {:else}
    {@const mColors = getMethodColor(request.method)}
    {@const sColors = getStatusColor(request.response_status)}

    <!-- Header / Inspector Actions -->
    <div class="detail-header">
      <div class="detail-headline">
        <span
          class="method-tag"
          style="background: {mColors.bg}; color: {mColors.text}; border: 1px solid {mColors.border}; font-size: 12px; padding: 2px 8px;"
        >
          {request.method}
        </span>
        <span class="detail-path-title">{request.path}</span>
        <span
          class="status-tag"
          style="background: {sColors.bg}; color: {sColors.text}; border: 1px solid {sColors.border};"
        >
          {request.response_status}
        </span>
        <span class="detail-meta-chip" title="Thời gian xử lý">
          <Icons name="zap" size={11} />
          {request.response_time_ms}ms
        </span>
        <span class="detail-meta-chip received" title={`Nhận lúc ${formatDateTime(request.timestamp)} (${request.timestamp})`}>
          <Icons name="clock" size={11} />
          {formatDateTime(request.timestamp, false)}
          <span class="received-ago">· {formatTimeAgo(request.timestamp, now)}</span>
        </span>
      </div>

      <div class="detail-actions">
        <button
          class="btn-primary"
          onclick={() => onOpenReplay(request)}
          title="Bắn lại webhook này sang server của bạn"
        >
          <Icons name="play" size={12} color="#ffffff" />
          <span>Replay</span>
        </button>

        <button
          class="btn-secondary"
          onclick={() => onCreateRuleFromRequest(request)}
          title="Tạo quy tắc phản hồi mock từ request này"
        >
          <Icons name="sliders" size={12} />
          <span>Tạo Rule</span>
        </button>

        <button
          class="btn-secondary"
          onclick={handleCopyCurl}
          title="Sao chép dưới dạng lệnh cURL"
        >
          <Icons name={copiedCurl ? 'check' : 'copy'} size={12} />
          <span>{copiedCurl ? 'Đã chép' : 'cURL'}</span>
        </button>

        <button
          class="btn-secondary"
          class:active-toggle={isComparing}
          onclick={() => onStartCompare(request)}
          title="So sánh với request khác (chọn request thứ hai trong danh sách)"
        >
          <Icons name="diff" size={12} />
          <span>So sánh</span>
        </button>

        <button
          class="btn-secondary danger"
          onclick={() => onDeleteRequest(request.id)}
          title="Xóa request này"
        >
          <Icons name="trash" size={12} color="#f87171" />
        </button>
      </div>
    </div>

    <!-- Tabs Nav -->
    <nav class="tabs-nav">
      <button
        class="tab-btn"
        class:active={activeTab === 'body'}
        onclick={() => (activeTab = 'body')}
      >
        Body ({formatBytes(bodyBytes)})
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'headers'}
        onclick={() => (activeTab = 'headers')}
      >
        Headers ({Object.keys(request.headers).length})
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'query'}
        onclick={() => (activeTab = 'query')}
      >
        Query ({Object.keys(request.query_params).length})
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'response'}
        onclick={() => (activeTab = 'response')}
      >
        Phản Hồi
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'overview'}
        onclick={() => (activeTab = 'overview')}
      >
        Chi Tiết
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'code'}
        onclick={() => (activeTab = 'code')}
      >
        Code
      </button>
    </nav>

    <!-- Tab Contents -->
    <div class="detail-content">
      {#if activeTab === 'body'}
        {#key request.id}
          <BodyViewer body={request.body} contentType={request.content_type} emptyText="<Request không có body>" />
        {/key}

      {:else if activeTab === 'headers'}
        <div class="headers-tab-container">
          {#key request.id}
            <SignaturePanel {request} />
          {/key}
          <table class="headers-table">
            <thead>
              <tr>
                <th style="width: 32%;">Tên Header</th>
                <th>Giá Trị</th>
                <th style="width: 50px; text-align: center;">Chép</th>
              </tr>
            </thead>
            <tbody>
              {#each Object.entries(request.headers) as [key, val]}
                <tr>
                  <td class="header-key">{key}</td>
                  <td class="header-val">{val}</td>
                  <td style="text-align: center;">
                    <button
                      class="btn-copy-sm"
                      onclick={() => handleCopyHeader(key, val)}
                      title="Sao chép giá trị header"
                    >
                      <Icons
                        name={copiedHeaderKey === key ? 'check' : 'copy'}
                        size={11}
                        color={copiedHeaderKey === key ? '#34d399' : '#64748b'}
                      />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

      {:else if activeTab === 'query'}
        <div class="query-tab-container">
          {#if Object.keys(request.query_params).length === 0}
            <div class="empty-inline">Không có tham số query nào trên đường dẫn URL.</div>
          {:else}
            <table class="headers-table">
              <thead>
                <tr>
                  <th style="width: 30%;">Tham Số</th>
                  <th>Giá Trị</th>
                </tr>
              </thead>
              <tbody>
                {#each Object.entries(request.query_params) as [key, val]}
                  <tr>
                    <td class="header-key">{key}</td>
                    <td class="header-val">{val}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>

      {:else if activeTab === 'response'}
        <div class="response-tab-container">
          {#if request.is_simulated_error}
            <div class="simulated-error-alert">
              <Icons name="zap" size={16} color="#fbbf24" />
              <div class="alert-text-col">
                <strong>Mô Phỏng Lỗi Ngẫu Nhiên (Chaos Engine)</strong>
                <span>Request này đã được chọn ngẫu nhiên để trả về lỗi nhằm kiểm tra cơ chế thử lại (retry) của hệ thống.</span>
              </div>
            </div>
          {/if}

          <div class="response-summary-card">
            <div class="resp-item">
              <span class="resp-label">Mã HTTP Trả Về</span>
              <span
                class="status-tag"
                style="background: {sColors.bg}; color: {sColors.text}; border: 1px solid {sColors.border}; font-size: 13px;"
              >
                {request.response_status}
              </span>
            </div>
            <div class="resp-item">
              <span class="resp-label">Time</span>
              <span class="resp-value">{request.response_time_ms} ms</span>
            </div>
            <div class="resp-item">
              <span class="resp-label">Quy Tắc / Nguyên Nhân</span>
              <span class="resp-value">
                {request.is_simulated_error
                  ? '⚡ Lỗi Mô Phỏng Ngẫu Nhiên (Chaos)'
                  : request.matched_rule_name
                    ? `Quy tắc: ${request.matched_rule_name}`
                    : 'Phản Hồi Mặc Định Của Endpoint'}
              </span>
            </div>
          </div>

          {#if request.response_body == null}
            <div class="empty-inline">Request này được ghi nhận trước khi app lưu nội dung phản hồi.</div>
          {:else}
            {@const respHeaders = Object.entries(request.response_headers ?? {})}
            {#if respHeaders.length > 0}
              <div class="section-label">Headers phản hồi ({respHeaders.length})</div>
              <table class="headers-table">
                <tbody>
                  {#each respHeaders as [key, val]}
                    <tr>
                      <td class="header-key" style="width: 32%;">{key}</td>
                      <td class="header-val">{val}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
            <div class="section-label">Body phản hồi</div>
            {#key request.id}
              <BodyViewer
                body={request.response_body}
                contentType={responseContentType(request)}
                emptyText="<Phản hồi không có body>"
              />
            {/key}
          {/if}
        </div>

      {:else if activeTab === 'overview'}
        <div class="overview-grid">
          <div class="overview-card">
            <span class="card-label">Received</span>
            <span class="card-val">{formatDateTime(request.timestamp)} · {formatTimeAgo(request.timestamp, now)}</span>
            <span class="card-sub mono">{request.timestamp}</span>
          </div>
          <div class="overview-card">
            <span class="card-label">Địa Chỉ IP Người Gửi</span>
            <span class="card-val">{request.ip_address}</span>
          </div>
          <div class="overview-card">
            <span class="card-label">Kích Thước Dữ Liệu</span>
            <span class="card-val">{formatBytes(bodyBytes)} ({bodyBytes} bytes)</span>
          </div>
          <div class="overview-card">
            <span class="card-label">Mã Request ID</span>
            <span class="card-val mono">{request.id}</span>
          </div>
        </div>

      {:else if activeTab === 'code'}
        <div class="code-snippets-container">
          <div class="snippet-block">
            <div class="snippet-header">
              <span>Lệnh cURL (Sao chép để bắn lại từ terminal)</span>
              <button class="btn-copy-toolbar" onclick={handleCopyCurl}>
                <Icons name={copiedCurl ? 'check' : 'copy'} size={12} />
                <span>{copiedCurl ? 'Đã sao chép' : 'Sao chép'}</span>
              </button>
            </div>
            <pre class="code-box"><code>{curlCode}</code></pre>
          </div>

          <div class="snippet-block">
            <div class="snippet-header">
              <span>Mã Nguồn JavaScript (Fetch API)</span>
              <button class="btn-copy-toolbar" onclick={handleCopyFetch}>
                <Icons name={copiedFetch ? 'check' : 'copy'} size={12} />
                <span>{copiedFetch ? 'Đã sao chép' : 'Sao chép'}</span>
              </button>
            </div>
            <pre class="code-box"><code>{fetchCode}</code></pre>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .detail-meta-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-dim);
    background: rgba(255, 255, 255, 0.04);
    padding: 2px 7px;
    border-radius: 4px;
  }

  .detail-meta-chip.received {
    color: #a5b4fc;
    background: rgba(99, 102, 241, 0.1);
    white-space: nowrap;
  }

  .received-ago {
    color: var(--text-dim);
  }

  .section-label {
    margin: 12px 0 6px;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
  }

  .btn-secondary.active-toggle {
    background: rgba(59, 130, 246, 0.18);
    border-color: rgba(59, 130, 246, 0.45);
    color: #93c5fd;
  }

  .card-sub {
    font-size: 10px;
    color: var(--text-dim);
  }

  .card-sub.mono {
    font-family: var(--font-mono);
  }

  .btn-copy-sm {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 3px;
    border-radius: 3px;
  }

  .btn-copy-sm:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .response-summary-card {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 8px;
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 10px 12px;
  }

  .resp-item {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .resp-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
  }

  .simulated-error-alert {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(245, 158, 11, 0.1);
    border: 1px solid rgba(245, 158, 11, 0.3);
    border-radius: 6px;
    padding: 7px 10px;
    margin-bottom: 8px;
  }

  .alert-text-col {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .alert-text-col strong {
    font-size: 11.5px;
    color: #fbbf24;
  }

  .alert-text-col span {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .resp-value {
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text-main);
  }

  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 8px;
  }

  .overview-card {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .card-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
  }

  .card-val {
    font-size: 11.5px;
    color: var(--text-main);
  }

  .card-val.mono {
    font-family: var(--font-mono);
    font-size: 11px;
    color: #93c5fd;
  }

  .snippet-block {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }

  .snippet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
  }
</style>
