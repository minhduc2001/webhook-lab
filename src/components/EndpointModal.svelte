<script lang="ts">
  import type { Endpoint } from '../types';
  import Icons from './Icons.svelte';

  let {
    isOpen,
    endpoint,
    onClose,
    onSave,
  } = $props<{
    isOpen: boolean;
    endpoint: Endpoint | null;
    onClose: () => void;
    onSave: (ep: Partial<Endpoint>) => void;
  }>();

  let name = $state('');
  let slug = $state('');
  let allowedMethods = $state<string[]>(['ANY']);
  let defaultStatus = $state(200);
  let defaultContentType = $state('application/json');
  let defaultBody = $state('{\n  "status": "ok",\n  "message": "Webhook processed successfully"\n}');
  let customHeaders = $state<{ key: string; value: string }[]>([]);

  // HTTP Redirect & Forwarding (giống webhook.site)
  let enableRedirect = $state(false);
  let redirectUrl = $state('');
  let redirectStatus = $state(302);

  let enableForward = $state(false);
  let autoForwardUrl = $state('');

  // Chaos & Random Failure
  let enableChaos = $state(false);
  let delayMs = $state(0);
  let errorRatePercent = $state(0);
  let errorStatus = $state(429);
  let errorBody = $state('{\n  "error": "Quá tải request (Rate Limit), vui lòng thử lại sau.",\n  "retry_after": 30\n}');

  const availableMethods = ['ANY', 'POST', 'GET', 'PUT', 'PATCH', 'DELETE'];

  // Preset format templates
  const formatPresets = [
    {
      id: 'json',
      name: '{ JSON }',
      type: 'application/json',
      status: 200,
      body: '{\n  "status": "ok",\n  "received": true\n}',
    },
    {
      id: 'text',
      name: 'Văn bản (Text)',
      type: 'text/plain',
      status: 200,
      body: 'OK',
    },
    {
      id: 'xml',
      name: '< XML >',
      type: 'application/xml',
      status: 200,
      body: '<?xml version="1.0" encoding="UTF-8"?>\n<response>\n  <status>ok</status>\n</response>',
    },
    {
      id: 'html',
      name: 'HTML',
      type: 'text/html',
      status: 200,
      body: '<!DOCTYPE html>\n<html>\n<body>\n  <h1>Webhook Nhận Thành Công</h1>\n</body>\n</html>',
    },
    {
      id: 'empty',
      name: '204 Rỗng (No Content)',
      type: 'text/plain',
      status: 204,
      body: '',
    },
    {
      id: 'stripe',
      name: 'Chuẩn Stripe',
      type: 'application/json',
      status: 200,
      body: '{\n  "received": true\n}',
    },
    {
      id: 'github',
      name: 'Chuẩn GitHub',
      type: 'application/json',
      status: 200,
      body: '{\n  "status": "success",\n  "queued": true\n}',
    },
  ];

  $effect(() => {
    if (isOpen) {
      if (endpoint) {
        name = endpoint.name;
        slug = endpoint.slug;
        allowedMethods = endpoint.allowed_methods && endpoint.allowed_methods.length > 0
          ? [...endpoint.allowed_methods]
          : ['ANY'];
        defaultStatus = endpoint.default_status;
        defaultContentType = endpoint.default_content_type;
        defaultBody = endpoint.default_body;
        delayMs = endpoint.delay_ms;
        errorRatePercent = endpoint.error_rate_percent;
        errorStatus = endpoint.error_status;
        errorBody = endpoint.error_body || '{\n  "error": "Mô phỏng lỗi ngẫu nhiên",\n  "status": ' + endpoint.error_status + '\n}';
        enableChaos = endpoint.error_rate_percent > 0 || endpoint.delay_ms > 0;

        enableRedirect = !!(endpoint.redirect_url && endpoint.redirect_url.trim());
        redirectUrl = endpoint.redirect_url || '';
        redirectStatus = endpoint.redirect_status || 302;

        enableForward = !!(endpoint.auto_forward_url && endpoint.auto_forward_url.trim());
        autoForwardUrl = endpoint.auto_forward_url || '';

        // Load custom headers
        const hdrs: { key: string; value: string }[] = [];
        for (const [k, v] of Object.entries(endpoint.default_headers || {})) {
          if (!k.toLowerCase().includes('content-type')) {
            hdrs.push({ key: String(k), value: String(v) });
          }
        }
        customHeaders = hdrs;
      } else {
        const randSlug = 'hook-' + Math.random().toString(36).substring(2, 7);
        name = 'Endpoint Mới';
        slug = randSlug;
        allowedMethods = ['POST'];
        defaultStatus = 200;
        defaultContentType = 'application/json';
        defaultBody = '{\n  "status": "ok",\n  "message": "Webhook nhận thành công"\n}';
        customHeaders = [];
        enableRedirect = false;
        redirectUrl = '';
        redirectStatus = 302;
        enableForward = false;
        autoForwardUrl = '';
        enableChaos = false;
        delayMs = 0;
        errorRatePercent = 0;
        errorStatus = 429;
        errorBody = '{\n  "error": "Quá tải request (Rate Limit), vui lòng thử lại sau.",\n  "retry_after": 30\n}';
      }
    }
  });

  function toggleMethod(m: string) {
    if (m === 'ANY') {
      allowedMethods = ['ANY'];
      return;
    }

    let current = allowedMethods.filter((item) => item !== 'ANY');
    if (current.includes(m)) {
      current = current.filter((item) => item !== m);
      if (current.length === 0) {
        current = ['ANY'];
      }
    } else {
      current.push(m);
    }
    allowedMethods = current;
  }

  function applyPreset(p: typeof formatPresets[0]) {
    defaultContentType = p.type;
    defaultStatus = p.status;
    defaultBody = p.body;
  }

  function addHeaderRow() {
    customHeaders = [...customHeaders, { key: '', value: '' }];
  }

  function removeHeaderRow(index: number) {
    customHeaders = customHeaders.filter((_, i) => i !== index);
  }

  function handleSubmit() {
    if (!name.trim() || !slug.trim()) return;

    const headersMap: Record<string, string> = {
      'Content-Type': defaultContentType,
      'X-Powered-By': 'WebhookLab',
    };
    for (const h of customHeaders) {
      if (h.key.trim()) {
        headersMap[h.key.trim()] = h.value.trim();
      }
    }

    onSave({
      id: endpoint ? endpoint.id : undefined,
      name: name.trim(),
      slug: slug.trim().toLowerCase().replace(/[^a-z0-9-_]/g, '-'),
      allowed_methods: allowedMethods,
      default_status: Number(defaultStatus),
      default_content_type: defaultContentType,
      default_body: defaultBody,
      default_headers: headersMap,
      redirect_url: enableRedirect && redirectUrl.trim() ? redirectUrl.trim() : null,
      redirect_status: enableRedirect ? Number(redirectStatus) : 302,
      auto_forward_url: enableForward && autoForwardUrl.trim() ? autoForwardUrl.trim() : null,
      forward_response: false,
      delay_ms: enableChaos ? Number(delayMs) : 0,
      error_rate_percent: enableChaos ? Number(errorRatePercent) : 0,
      error_status: Number(errorStatus),
      error_body: enableChaos ? errorBody : null,
      is_active: true,
    });
  }
</script>

{#if isOpen}
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
      class="modal-card ep-config-card"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <div class="modal-title-col">
          <h3 class="modal-title">{endpoint ? 'Cấu Hình Endpoint' : 'Tạo Webhook Endpoint Mới'}</h3>
          <span class="modal-subtitle">
            Địa chỉ URL cục bộ: <code class="mono-tag">http://localhost:4567/wh/{slug || '<slug>'}</code>
          </span>
        </div>
        <button class="btn-close" onclick={onClose}>
          <Icons name="x" size={16} />
        </button>
      </div>

      <div class="modal-body">
        <!-- 1. Thông tin cơ bản: Tên & Slug -->
        <div class="form-row-2">
          <div class="form-group">
            <label class="form-label" for="ep-name-input">Tên Endpoint</label>
            <input
              id="ep-name-input"
              type="text"
              class="form-input"
              placeholder="VD: Webhook Thanh Toán Stripe"
              bind:value={name}
            />
          </div>

          <div class="form-group">
            <label class="form-label" for="ep-slug-input">Đường dẫn Slug (/wh/&lt;slug&gt;)</label>
            <input
              id="ep-slug-input"
              type="text"
              class="form-input mono"
              placeholder="VD: stripe-checkout"
              bind:value={slug}
            />
          </div>
        </div>

        <!-- 2. Phương thức HTTP được phép (POST, GET, PUT,...) -->
        <div class="config-section">
          <div class="section-header-row">
            <span class="section-title-label">Phương Thức HTTP Cho Phép</span>
            <span class="section-desc-label">Chặn các method khác với mã 405 Method Not Allowed</span>
          </div>

          <div class="methods-selector-row">
            {#each availableMethods as m}
              <button
                type="button"
                class="btn-method-pill"
                class:active={allowedMethods.includes(m)}
                onclick={() => toggleMethod(m)}
              >
                {m === 'ANY' ? '★ Tất Cả (ANY)' : m}
              </button>
            {/each}
          </div>
        </div>

        <!-- 3. Chuyển hướng & Chuyển tiếp Webhook (giống Webhook.site) -->
        <div class="config-section redirect-box">
          <div class="section-header-row">
            <span class="section-title-label">Chuyển Hướng & Tự Động Chuyển Tiếp (Relay)</span>
            <span class="section-desc-label">Tính năng cao cấp tương tự Webhook.site</span>
          </div>

          <!-- HTTP Redirect Mode -->
          <div class="sub-toggle-card" class:active-card={enableRedirect}>
            <div class="sub-toggle-header">
              <label class="sub-toggle-title">
                <input type="checkbox" bind:checked={enableRedirect} />
                <span>Chuyển Hướng HTTP (HTTP Redirect 301 / 302 / 307)</span>
              </label>
              <span class="sub-toggle-hint">Trả về mã redirect kèm header Location cho client</span>
            </div>

            {#if enableRedirect}
              <div class="form-row-2 mt-2">
                <div class="form-group">
                  <label class="form-label" for="ep-redirect-url">URL Đích Chuyển Hướng Đến (Location)</label>
                  <input
                    id="ep-redirect-url"
                    type="text"
                    class="form-input mono"
                    placeholder="https://example.com/thank-you"
                    bind:value={redirectUrl}
                  />
                </div>
                <div class="form-group">
                  <label class="form-label" for="ep-redirect-status">Mã Chuyển Hướng (Status)</label>
                  <select id="ep-redirect-status" class="form-select" bind:value={redirectStatus}>
                    <option value={302}>302 Found (Chuyển hướng tạm thời)</option>
                    <option value={301}>301 Moved Permanently (Vĩnh viễn)</option>
                    <option value={307}>307 Temporary Redirect (Giữ nguyên POST/GET)</option>
                    <option value={308}>308 Permanent Redirect (Giữ nguyên Method)</option>
                  </select>
                </div>
              </div>
            {/if}
          </div>

          <!-- Auto-Forwarding Webhook Relay -->
          <div class="sub-toggle-card" class:active-card={enableForward}>
            <div class="sub-toggle-header">
              <label class="sub-toggle-title">
                <input type="checkbox" bind:checked={enableForward} />
                <span>Tự Động Chuyển Tiếp (Auto-Forwarding / Relay Webhook)</span>
              </label>
              <span class="sub-toggle-hint">Tự động bắn bản sao payload sang backend nội bộ của bạn</span>
            </div>

            {#if enableForward}
              <div class="form-group mt-2">
                <label class="form-label" for="ep-forward-url">URL Backend Đích Nhận Bản Sao (Local / Server)</label>
                <input
                  id="ep-forward-url"
                  type="text"
                  class="form-input mono"
                  placeholder="http://localhost:3000/api/webhook"
                  bind:value={autoForwardUrl}
                />
                <span class="field-hint">
                  Ngay khi Webhook Lab nhận được request, nó sẽ tự động gửi tiếp dữ liệu nguyên vẹn sang URL này.
                </span>
              </div>
            {/if}
          </div>
        </div>

        <!-- 4. Định dạng & Mẫu phản hồi (Response Format) -->
        <div class="config-section">
          <div class="section-header-row">
            <span class="section-title-label">Định Dạng & Mẫu Phản Hồi Trả Về (Response Format)</span>
            <span class="section-desc-label">Tùy biến body và mã HTTP trả lại cho webhook sender</span>
          </div>

          <!-- Presets -->
          <div class="preset-pills-row">
            {#each formatPresets as p}
              <button
                type="button"
                class="btn-preset-pill"
                class:selected={defaultContentType === p.type && defaultStatus === p.status}
                onclick={() => applyPreset(p)}
              >
                {p.name}
              </button>
            {/each}
          </div>

          <div class="form-row-2 mt-2">
            <div class="form-group">
              <label class="form-label" for="ep-status-input">Mã Trạng Thái Phản Hồi (HTTP Status)</label>
              <select id="ep-status-input" class="form-select" bind:value={defaultStatus}>
                <option value={200}>200 OK (Thành công chuẩn)</option>
                <option value={201}>201 Created (Đã tạo thành công)</option>
                <option value={202}>202 Accepted (Đã nhận xử lý ngầm)</option>
                <option value={204}>204 No Content (Thành công không có body)</option>
                <option value={400}>400 Bad Request (Dữ liệu gửi sai)</option>
                <option value={401}>401 Unauthorized (Chưa xác thực)</option>
                <option value={404}>404 Not Found (Không tìm thấy)</option>
                <option value={429}>429 Too Many Requests (Quá tải request)</option>
                <option value={500}>500 Internal Server Error (Lỗi hệ thống)</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="ep-content-type-input">Header Content-Type</label>
              <input
                id="ep-content-type-input"
                type="text"
                class="form-input mono"
                bind:value={defaultContentType}
                placeholder="application/json"
              />
            </div>
          </div>

          <div class="form-group mt-2">
            <label class="form-label" for="ep-body-input">Nội Dung Phản Hồi (Response Body)</label>
            <textarea
              id="ep-body-input"
              rows="4"
              class="form-textarea"
              bind:value={defaultBody}
              placeholder="Nội dung HTTP response trả về cho phía gọi webhook..."
            ></textarea>
          </div>

          <!-- Custom Headers -->
          <div class="custom-headers-box">
            <div class="headers-header-row">
              <span class="form-label">Headers Phản Hồi Tùy Chỉnh (Custom Headers)</span>
              <button type="button" class="btn-add-header" onclick={addHeaderRow}>
                <Icons name="plus" size={11} />
                <span>+ Thêm Header</span>
              </button>
            </div>

            {#if customHeaders.length > 0}
              <div class="headers-list">
                {#each customHeaders as h, i}
                  <div class="header-input-row">
                    <input
                      type="text"
                      placeholder="Tên Header (VD: X-Webhook-ID)"
                      class="form-input header-k mono"
                      bind:value={h.key}
                    />
                    <input
                      type="text"
                      placeholder="Giá trị (VD: custom-value)"
                      class="form-input header-v mono"
                      bind:value={h.value}
                    />
                    <button
                      type="button"
                      class="btn-del-hdr"
                      onclick={() => removeHeaderRow(i)}
                      title="Xóa header này"
                    >
                      <Icons name="trash" size={12} color="#f87171" />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- 5. Mô phỏng Lỗi Ngẫu Nhiên & Chaos (Chaos Simulation Engine) -->
        <div class="chaos-section" class:active-chaos={enableChaos && (errorRatePercent > 0 || delayMs > 0)}>
          <div class="chaos-header-row">
            <div class="chaos-title-col">
              <div class="chaos-badge">
                <Icons name="zap" size={13} color="#f59e0b" />
                <span>Mô Phỏng Lỗi Ngẫu Nhiên & Chaos (Random Error Simulation)</span>
              </div>
              <span class="chaos-desc">
                Tự động trả về lỗi ngẫu nhiên và độ trễ mạng để kiểm tra cơ chế Retry & Circuit Breaker của bạn
              </span>
            </div>

            <label class="switch-toggle" title="Bật/Tắt mô phỏng lỗi ngẫu nhiên">
              <input type="checkbox" bind:checked={enableChaos} />
              <span class="slider-round"></span>
            </label>
          </div>

          {#if enableChaos}
            <div class="chaos-controls-grid">
              <!-- Random Failure Probability Slider -->
              <div class="slider-box">
                <div class="slider-label-row">
                  <span class="slider-title">Tỷ Lệ Bị Lỗi Ngẫu Nhiên</span>
                  <span
                    class="chaos-pill"
                    class:zero={errorRatePercent === 0}
                    class:low={errorRatePercent > 0 && errorRatePercent <= 30}
                    class:high={errorRatePercent > 30}
                  >
                    {errorRatePercent}%
                  </span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="100"
                  step="5"
                  class="chaos-range"
                  bind:value={errorRatePercent}
                />
                <span class="slider-hint">
                  {errorRatePercent === 0
                    ? 'Tắt lỗi ngẫu nhiên (100% request thành công)'
                    : `Cứ trung bình 100 request gửi đến sẽ có ${errorRatePercent} request bị trả về lỗi ngẫu nhiên`}
                </span>
              </div>

              <!-- Latency Delay Slider -->
              <div class="slider-box">
                <div class="slider-label-row">
                  <span class="slider-title">Độ Trễ Mạng Giả Lập (Delay)</span>
                  <span class="chaos-pill blue">{delayMs} ms</span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="5000"
                  step="50"
                  class="chaos-range"
                  bind:value={delayMs}
                />
                <span class="slider-hint">
                  {delayMs === 0 ? 'Không thêm độ trễ' : `Mỗi request phải đợi ${delayMs}ms mới được trả về`}
                </span>
              </div>
            </div>

            {#if errorRatePercent > 0}
              <div class="error-spec-box">
                <div class="form-group">
                  <label class="form-label" for="ep-err-status-select">Mã Lỗi HTTP Khi Bị Lỗi Ngẫu Nhiên</label>
                  <select id="ep-err-status-select" class="form-select" bind:value={errorStatus}>
                    <option value={429}>429 Too Many Requests (Mô phỏng Rate Limit)</option>
                    <option value={500}>500 Internal Server Error (Lỗi hệ thống nội bộ)</option>
                    <option value={502}>502 Bad Gateway (Lỗi cổng gateway trung gian)</option>
                    <option value={503}>503 Service Unavailable (Hệ thống tạm ngưng phục vụ)</option>
                    <option value={504}>504 Gateway Timeout (Quá thời gian chờ gateway)</option>
                    <option value={400}>400 Bad Request (Lỗi yêu cầu)</option>
                    <option value={401}>401 Unauthorized (Lỗi xác thực)</option>
                  </select>
                </div>

                <div class="form-group mt-2">
                  <label class="form-label" for="ep-err-body-input">Nội Dung Body Trả Về Khi Lỗi Ngẫu Nhiên</label>
                  <textarea
                    id="ep-err-body-input"
                    rows="3"
                    class="form-textarea"
                    bind:value={errorBody}
                    placeholder="JSON hoặc text thông báo lỗi khi request bị rơi vào tỷ lệ lỗi..."
                  ></textarea>
                </div>
              </div>
            {/if}
          {/if}
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" onclick={onClose}>Hủy Bỏ</button>
        <button class="btn-primary" onclick={handleSubmit}>
          {endpoint ? 'Lưu Thay Đổi' : 'Tạo Endpoint'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .ep-config-card {
    max-width: 720px;
  }

  .modal-title-col {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .modal-subtitle {
    font-size: 11px;
    color: var(--text-dim);
  }

  .mono-tag {
    font-family: var(--font-mono);
    color: #60a5fa;
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

  .btn-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .form-row-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .mt-2 {
    margin-top: 8px;
  }

  .mono {
    font-family: var(--font-mono);
  }

  .config-section {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title-label {
    font-size: 11.5px;
    font-weight: 700;
    color: #93c5fd;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .section-desc-label {
    font-size: 10.5px;
    color: var(--text-dim);
  }

  /* Method selector */
  .methods-selector-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
  }

  .btn-method-pill {
    padding: 4px 10px;
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-method-pill:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .btn-method-pill.active {
    background: rgba(16, 185, 129, 0.18);
    border-color: rgba(16, 185, 129, 0.4);
    color: #34d399;
  }

  /* Redirect & Forwarding cards */
  .sub-toggle-card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 10px 12px;
    transition: all 0.2s;
  }

  .sub-toggle-card.active-card {
    border-color: rgba(59, 130, 246, 0.35);
    background: rgba(59, 130, 246, 0.04);
  }

  .sub-toggle-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sub-toggle-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-main);
    cursor: pointer;
  }

  .sub-toggle-hint {
    font-size: 10.5px;
    color: var(--text-dim);
  }

  .field-hint {
    font-size: 10.5px;
    color: var(--text-dim);
    margin-top: 3px;
  }

  .preset-pills-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 2px;
  }

  .btn-preset-pill {
    padding: 4px 9px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-preset-pill:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .btn-preset-pill.selected {
    background: rgba(59, 130, 246, 0.2);
    border-color: #3b82f6;
    color: #60a5fa;
  }

  .custom-headers-box {
    margin-top: 6px;
    border-top: 1px dashed var(--border-subtle);
    padding-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .headers-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .btn-add-header {
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    color: var(--text-muted);
    padding: 2px 7px;
    font-size: 10.5px;
    cursor: pointer;
  }

  .btn-add-header:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .headers-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .header-input-row {
    display: grid;
    grid-template-columns: 1fr 1fr 28px;
    gap: 6px;
    align-items: center;
  }

  .header-k, .header-v {
    padding: 4px 8px;
    font-size: 11px;
  }

  .btn-del-hdr {
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 4px;
  }

  .btn-del-hdr:hover {
    background: rgba(239, 68, 68, 0.15);
  }

  /* Chaos Section */
  .chaos-section {
    background: rgba(245, 158, 11, 0.03);
    border: 1px solid rgba(245, 158, 11, 0.15);
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: all 0.2s;
  }

  .chaos-section.active-chaos {
    background: rgba(245, 158, 11, 0.06);
    border-color: rgba(245, 158, 11, 0.35);
    box-shadow: 0 0 16px rgba(245, 158, 11, 0.08);
  }

  .chaos-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .chaos-title-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .chaos-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 700;
    color: #fbbf24;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .chaos-desc {
    font-size: 11px;
    color: var(--text-dim);
  }

  /* Switch Toggle */
  .switch-toggle {
    position: relative;
    display: inline-block;
    width: 38px;
    height: 22px;
    flex-shrink: 0;
  }

  .switch-toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider-round {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(255, 255, 255, 0.12);
    transition: 0.2s;
    border-radius: 22px;
    border: 1px solid var(--border-medium);
  }

  .slider-round:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.2s;
    border-radius: 50%;
  }

  input:checked + .slider-round {
    background-color: #f59e0b;
  }

  input:checked + .slider-round:before {
    transform: translateX(16px);
  }

  .chaos-controls-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .slider-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .slider-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .slider-title {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .slider-hint {
    font-size: 10px;
    color: var(--text-dim);
  }

  .chaos-pill {
    font-family: var(--font-mono);
    font-size: 10.5px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 4px;
  }

  .chaos-pill.zero {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-dim);
  }

  .chaos-pill.low {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }

  .chaos-pill.high {
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
  }

  .chaos-pill.blue {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
  }

  .chaos-range {
    width: 100%;
    accent-color: #f59e0b;
    cursor: pointer;
  }

  .error-spec-box {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(245, 158, 11, 0.2);
    border-radius: 6px;
    padding: 10px 12px;
    margin-top: 2px;
  }
</style>
