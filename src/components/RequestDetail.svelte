<script lang="ts">
  import type { WebhookRequest, TunnelStatus } from '../types';
  import {
    formatJson,
    generateCurl,
    generateFetchCode,
    copyToClipboard,
    getMethodColor,
    getStatusColor,
    formatTime,
    formatBytes,
  } from '../lib/utils';
  import Icons from './Icons.svelte';

  let {
    request,
    tunnelStatus,
    onOpenReplay,
    onCreateRuleFromRequest,
    onDeleteRequest,
  } = $props<{
    request: WebhookRequest | null;
    tunnelStatus: TunnelStatus;
    onOpenReplay: (req: WebhookRequest) => void;
    onCreateRuleFromRequest: (req: WebhookRequest) => void;
    onDeleteRequest: (id: string) => void;
  }>();

  let activeTab = $state<'body' | 'headers' | 'query' | 'response' | 'overview' | 'code'>('body');
  let isRawBody = $state(false);
  let copiedCurl = $state(false);
  let copiedBody = $state(false);
  let copiedHeaderKey = $state<string | null>(null);

  let activeBaseUrl = $derived(
    tunnelStatus.is_running && tunnelStatus.public_url
      ? tunnelStatus.public_url
      : 'http://localhost:4567'
  );

  let formattedBody = $derived(
    request ? formatJson(request.body) : { formatted: '', isValid: false }
  );

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

  async function handleCopyBody() {
    if (!request) return;
    const ok = await copyToClipboard(request.body);
    if (ok) {
      copiedBody = true;
      setTimeout(() => (copiedBody = false), 2000);
    }
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
        <Icons name="code" size={28} />
      </div>
      <h3>Chưa Chọn Request Nào</h3>
      <p>Chọn một webhook request từ danh sách bên trái để kiểm tra chi tiết headers, payload, phản hồi và bắn lại (replay).</p>
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
        <span class="detail-meta-chip">
          <Icons name="clock" size={11} />
          {request.response_time_ms}ms
        </span>
      </div>

      <div class="detail-actions">
        <button
          class="btn-primary"
          onclick={() => onOpenReplay(request)}
          title="Bắn lại webhook này sang server backend nội bộ của bạn"
        >
          <Icons name="play" size={12} color="#ffffff" />
          <span>Bắn Lại (Replay)</span>
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
        Dữ Liệu Body ({formatBytes(request.body.length)})
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
        Query Params ({Object.keys(request.query_params).length})
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'response'}
        onclick={() => (activeTab = 'response')}
      >
        Phản Hồi Đã Trả Về
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'overview'}
        onclick={() => (activeTab = 'overview')}
      >
        Tổng Quan
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'code'}
        onclick={() => (activeTab = 'code')}
      >
        Mã Lệnh Mẫu
      </button>
    </nav>

    <!-- Tab Contents -->
    <div class="detail-content">
      {#if activeTab === 'body'}
        <div class="body-tab-container">
          <div class="tab-toolbar">
            <div class="tab-toolbar-left">
              <span class="content-type-badge">{request.content_type || 'text/plain'}</span>
              {#if formattedBody.isValid}
                <span class="json-valid-badge">JSON Hợp Lệ</span>
              {/if}
            </div>
            <div class="tab-toolbar-right">
              {#if formattedBody.isValid}
                <button
                  class="btn-toggle-raw"
                  onclick={() => (isRawBody = !isRawBody)}
                >
                  {isRawBody ? 'Định dạng đẹp' : 'Dạng gốc (Raw)'}
                </button>
              {/if}
              <button class="btn-copy-toolbar" onclick={handleCopyBody}>
                <Icons name={copiedBody ? 'check' : 'copy'} size={12} />
                <span>{copiedBody ? 'Đã sao chép' : 'Sao chép'}</span>
              </button>
            </div>
          </div>

          {#if !request.body}
            <div class="empty-inline">&lt;Request Không Có Nội Dung Body&gt;</div>
          {:else}
            <pre class="code-box"><code>{isRawBody || !formattedBody.isValid ? request.body : formattedBody.formatted}</code></pre>
          {/if}
        </div>

      {:else if activeTab === 'headers'}
        <div class="headers-tab-container">
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
              <span class="resp-label">Thời Gian Xử Lý</span>
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
        </div>

      {:else if activeTab === 'overview'}
        <div class="overview-grid">
          <div class="overview-card">
            <span class="card-label">Thời Điểm Nhận</span>
            <span class="card-val">{formatTime(request.timestamp)} ({request.timestamp})</span>
          </div>
          <div class="overview-card">
            <span class="card-label">Địa Chỉ IP Người Gửi</span>
            <span class="card-val">{request.ip_address}</span>
          </div>
          <div class="overview-card">
            <span class="card-label">Kích Thước Dữ Liệu</span>
            <span class="card-val">{formatBytes(request.body.length)} ({request.body.length} bytes)</span>
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
              <button
                class="btn-copy-toolbar"
                onclick={async () => {
                  await copyToClipboard(fetchCode);
                }}
              >
                <Icons name="copy" size={12} />
                <span>Sao chép</span>
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

  .tab-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .tab-toolbar-left, .tab-toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .content-type-badge {
    font-family: var(--font-mono);
    font-size: 10.5px;
    background: rgba(255, 255, 255, 0.06);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--text-muted);
  }

  .json-valid-badge {
    font-size: 10px;
    font-weight: 600;
    color: #34d399;
    background: rgba(16, 185, 129, 0.12);
    border: 1px solid rgba(16, 185, 129, 0.25);
    padding: 1px 6px;
    border-radius: 4px;
  }

  .btn-toggle-raw, .btn-copy-toolbar {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 3px 8px;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.12s;
  }

  .btn-toggle-raw:hover, .btn-copy-toolbar:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
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

  .empty-inline {
    padding: 24px;
    text-align: center;
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: 12px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
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
