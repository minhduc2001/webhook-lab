<script lang="ts">
  import type { ApiForwarder } from '../types';
  import { copyToClipboard } from '../lib/utils';
  import Icons from './Icons.svelte';

  let {
    isOpen,
    forwarders = [],
    onClose,
    onSaveForwarder,
    onDeleteForwarder,
    onStartForwarder,
    onStopForwarder,
  } = $props<{
    isOpen: boolean;
    forwarders: ApiForwarder[];
    onClose: () => void;
    onSaveForwarder: (forwarder: Partial<ApiForwarder>) => Promise<void>;
    onDeleteForwarder: (id: string) => Promise<void>;
    onStartForwarder: (id: string, host: string, port: number) => Promise<void>;
    onStopForwarder: (id: string) => Promise<void>;
  }>();

  // Form state
  let showForm = $state(false);
  let editingId = $state<string | null>(null);
  let formName = $state('');
  let formHost = $state('127.0.0.1');
  let formPort = $state(3000);
  let formError = $state<string | null>(null);
  let isSubmitting = $state(false);

  // Copy status tracker by forwarder id
  let copiedId = $state<string | null>(null);

  const portPresets = [
    { label: ':3000 Node/Next', port: 3000 },
    { label: ':8080 Spring/Java', port: 8080 },
    { label: ':8000 FastAPI/Django', port: 8000 },
    { label: ':5000 Flask/.NET', port: 5000 },
    { label: ':5173 Vite Dev', port: 5173 },
  ];

  let runningCount = $derived(forwarders.filter((f: ApiForwarder) => f.is_running).length);

  function resetForm() {
    formName = '';
    formHost = '127.0.0.1';
    formPort = 3000;
    formError = null;
    editingId = null;
    showForm = false;
  }

  function handleOpenCreate() {
    resetForm();
    showForm = true;
    formName = `API Cổng :${formPort}`;
  }

  function handleOpenEdit(fwd: ApiForwarder) {
    editingId = fwd.id;
    formName = fwd.name;
    formHost = fwd.host || '127.0.0.1';
    formPort = fwd.port || 3000;
    formError = null;
    showForm = true;
  }

  async function handleFormSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!formPort || formPort <= 0 || formPort > 65535) {
      formError = 'Cổng port phải nằm trong khoảng từ 1 đến 65535';
      return;
    }

    isSubmitting = true;
    formError = null;

    try {
      await onSaveForwarder({
        id: editingId || undefined,
        name: formName.trim() || `API Cổng :${formPort}`,
        host: formHost.trim() || '127.0.0.1',
        port: Number(formPort),
      });
      resetForm();
    } catch (err: any) {
      formError = err?.message || 'Không thể lưu forwarder';
    } finally {
      isSubmitting = false;
    }
  }

  async function handleCopy(id: string, url: string) {
    const ok = await copyToClipboard(url);
    if (ok) {
      copiedId = id;
      setTimeout(() => {
        if (copiedId === id) copiedId = null;
      }, 2000);
    }
  }

  async function handleDelete(fwd: ApiForwarder) {
    if (confirm(`Bạn có chắc muốn xóa forwarder "${fwd.name}" (cổng ${fwd.port})?`)) {
      await onDeleteForwarder(fwd.id);
      if (editingId === fwd.id) {
        resetForm();
      }
    }
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
    <div class="modal-card multi-forwarder-card" onclick={(e) => e.stopPropagation()}>
      <!-- Header -->
      <div class="modal-header">
        <div class="title-with-badge">
          <div class="icon-bubble">
            <Icons name="globe" size={16} color="#38bdf8" />
          </div>
          <div class="title-row">
            <h3 class="modal-title">Forward API (Cloudflare)</h3>
            <span class="count-tag" class:active={runningCount > 0}>
              {runningCount} / {forwarders.length} Online
            </span>
          </div>
        </div>

        <div class="header-actions">
          {#if !showForm}
            <button class="btn-add-fwd" onclick={handleOpenCreate}>
              <Icons name="plus" size={12} />
              <span>Thêm Cổng</span>
            </button>
          {/if}
          <button class="btn-close" onclick={onClose} aria-label="Đóng">
            <Icons name="x" size={16} />
          </button>
        </div>
      </div>

      <div class="modal-body scrollable-body">
        <!-- Add / Edit Form Card -->
        {#if showForm}
          <form class="fwd-form-card" onsubmit={handleFormSubmit}>
            <div class="form-card-header">
              <span class="form-card-title">
                <Icons name={editingId ? 'edit' : 'plus'} size={14} color="#38bdf8" />
                <span>{editingId ? 'Chỉnh Sửa Cổng Chuyển Tiếp' : 'Thêm Cổng Chuyển Tiếp Mới'}</span>
              </span>
              <button type="button" class="btn-form-cancel-sm" onclick={resetForm}>
                Hủy
              </button>
            </div>

            {#if formError}
              <div class="error-banner">
                <Icons name="alert-circle" size={13} color="#f87171" />
                <span>{formError}</span>
              </div>
            {/if}

            <div class="form-row-2">
              <div class="form-group flex-2">
                <label class="form-label" for="fwd-name-input">Tên dịch vụ</label>
                <input
                  id="fwd-name-input"
                  type="text"
                  class="form-input"
                  placeholder="vd: Backend API, Auth Service"
                  bind:value={formName}
                  required
                />
              </div>

              <div class="form-group flex-1">
                <label class="form-label" for="fwd-host-input">Host</label>
                <input
                  id="fwd-host-input"
                  type="text"
                  class="form-input mono"
                  placeholder="127.0.0.1"
                  bind:value={formHost}
                  required
                />
              </div>

              <div class="form-group w-port">
                <label class="form-label" for="fwd-port-input">Port</label>
                <input
                  id="fwd-port-input"
                  type="number"
                  min="1"
                  max="65535"
                  class="form-input mono"
                  placeholder="3000"
                  bind:value={formPort}
                  required
                />
              </div>
            </div>

            <!-- Quick Port Presets -->
            <div class="presets-row">
              <span class="presets-label">Gợi ý:</span>
              {#each portPresets as preset}
                <button
                  type="button"
                  class="btn-port-chip"
                  class:active={formPort === preset.port}
                  onclick={() => {
                    formPort = preset.port;
                    if (!formName || formName.startsWith('API Cổng :')) {
                      formName = `API Cổng :${preset.port}`;
                    }
                  }}
                >
                  {preset.label}
                </button>
              {/each}
            </div>

            <div class="form-actions-row">
              <button type="button" class="btn-secondary btn-sm" onclick={resetForm}>
                Hủy
              </button>
              <button type="submit" class="btn-primary btn-sm" disabled={isSubmitting}>
                {#if isSubmitting}
                  <Icons name="refresh" size={12} class="spin" />
                  <span>Đang lưu...</span>
                {:else}
                  <Icons name="check" size={12} />
                  <span>{editingId ? 'Cập Nhật' : 'Lưu'}</span>
                {/if}
              </button>
            </div>
          </form>
        {/if}

        <!-- Forwarders List -->
        {#if forwarders.length === 0 && !showForm}
          <div class="empty-fwd-state">
            <div class="empty-icon-circle">
              <Icons name="globe" size={28} color="#64748b" />
            </div>
            <h4>Chưa Có Cổng Forward Nào</h4>
            <p>
              Tạo cổng chuyển tiếp để mở các dịch vụ API chạy nội bộ trên máy bạn (localhost) ra ngoài Internet
              với URL HTTPS công khai hoàn toàn miễn phí.
            </p>
            <button class="btn-primary btn-create-first" onclick={handleOpenCreate}>
              <Icons name="plus" size={14} />
              <span>Tạo Cổng Chuyển Tiếp Đầu Tiên</span>
            </button>
          </div>
        {:else}
          <div class="forwarders-list">
            {#each forwarders as fwd (fwd.id)}
              <div
                class="fwd-item-card"
                class:is-active={fwd.is_running}
                class:is-has-error={!!fwd.error}
              >
                <!-- Card Header -->
                <div class="fwd-item-top">
                  <div class="fwd-info-col">
                    <div class="fwd-name-row">
                      <span class="fwd-name">{fwd.name}</span>
                      <span class="fwd-target-tag mono">
                        http://{fwd.host || '127.0.0.1'}:{fwd.port}
                      </span>
                    </div>

                    <div class="fwd-status-row">
                      {#if fwd.is_running && fwd.public_url}
                        <div class="online-indicator">
                          <span class="pulse-dot"></span>
                          <span class="online-text">Online</span>
                        </div>
                      {:else if fwd.is_running}
                        <div class="starting-indicator">
                          <Icons name="refresh" size={11} class="spin" color="#38bdf8" />
                          <span>Đang kết nối...</span>
                        </div>
                      {:else if fwd.error}
                        <div class="error-indicator">
                          <Icons name="alert-circle" size={11} color="#f87171" />
                          <span>Lỗi</span>
                        </div>
                      {:else}
                        <div class="idle-indicator">
                          <span class="idle-dot"></span>
                          <span>Offline</span>
                        </div>
                      {/if}
                    </div>
                  </div>

                  <!-- Item Actions -->
                  <div class="fwd-actions-col">
                    {#if fwd.is_running}
                      <button
                        class="btn-stop-tunnel"
                        disabled={fwd.is_loading}
                        onclick={() => onStopForwarder(fwd.id)}
                        title="Dừng chuyển tiếp cổng này"
                      >
                        <Icons name="stop" size={11} color="#f87171" />
                        <span>Dừng</span>
                      </button>
                    {:else}
                      <button
                        class="btn-start-tunnel"
                        disabled={fwd.is_loading}
                        onclick={() => onStartForwarder(fwd.id, fwd.host, fwd.port)}
                        title="Bật Cloudflare Tunnel cho cổng này"
                      >
                        {#if fwd.is_loading}
                          <Icons name="refresh" size={11} class="spin" />
                          <span>Đang bật...</span>
                        {:else}
                          <Icons name="play" size={11} color="#ffffff" />
                          <span>Bật</span>
                        {/if}
                      </button>
                      <button
                        class="btn-icon-subtle"
                        onclick={() => handleOpenEdit(fwd)}
                        title="Chỉnh sửa cấu hình"
                      >
                        <Icons name="edit" size={13} color="#94a3b8" />
                      </button>
                      <button
                        class="btn-icon-subtle danger"
                        onclick={() => handleDelete(fwd)}
                        title="Xóa cổng này"
                      >
                        <Icons name="trash" size={13} color="#94a3b8" />
                      </button>
                    {/if}
                  </div>
                </div>

                <!-- Error Display -->
                {#if fwd.error}
                  <div class="fwd-error-box">
                    <Icons name="alert-circle" size={13} color="#f87171" />
                    <span>{fwd.error}</span>
                  </div>
                {/if}

                <!-- Active Public URL Box -->
                {#if fwd.is_running && fwd.public_url}
                  <div class="fwd-url-box">
                    <div class="fwd-url-control">
                      <input
                        type="text"
                        readonly
                        class="fwd-url-input mono"
                        value={fwd.public_url}
                      />
                      <button
                        class="btn-copy-url"
                        class:copied={copiedId === fwd.id}
                        onclick={() => handleCopy(fwd.id, fwd.public_url!)}
                        title="Sao chép URL"
                      >
                        <Icons
                          name={copiedId === fwd.id ? 'check' : 'copy'}
                          size={12}
                          color={copiedId === fwd.id ? '#34d399' : '#ffffff'}
                        />
                        <span>{copiedId === fwd.id ? 'Đã Chép' : 'Sao Chép'}</span>
                      </button>
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button class="btn-secondary" onclick={onClose}>Đóng</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .multi-forwarder-card {
    max-width: 680px;
    width: 95%;
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    border: 1px solid rgba(56, 189, 248, 0.3);
  }

  .title-with-badge {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .icon-bubble {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .count-tag {
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
  }

  .count-tag.active {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.4);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-add-fwd {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(56, 189, 248, 0.15);
    border: 1px solid rgba(56, 189, 248, 0.35);
    color: #38bdf8;
    padding: 3px 9px;
    border-radius: 5px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-add-fwd:hover {
    background: rgba(56, 189, 248, 0.25);
    border-color: #38bdf8;
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

  .scrollable-body {
    overflow-y: auto;
    padding: 14px 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Form Card */
  .fwd-form-card {
    background: rgba(15, 23, 42, 0.6);
    border: 1px solid rgba(56, 189, 248, 0.35);
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .form-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .form-card-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 700;
    color: #e2e8f0;
  }

  .btn-form-cancel-sm {
    background: transparent;
    border: none;
    color: var(--text-dim);
    font-size: 10.5px;
    cursor: pointer;
  }

  .btn-form-cancel-sm:hover {
    color: var(--text-main);
  }

  .form-row-2 {
    display: flex;
    gap: 10px;
  }

  .flex-1 {
    flex: 1;
  }

  .flex-2 {
    flex: 2;
  }

  .w-port {
    width: 120px;
  }

  .mono {
    font-family: var(--font-mono);
  }

  .presets-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
  }

  .presets-label {
    font-size: 10.5px;
    color: var(--text-dim);
  }

  .btn-port-chip {
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 10.5px;
    font-family: var(--font-mono);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.12s;
  }

  .btn-port-chip:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .btn-port-chip.active {
    background: rgba(56, 189, 248, 0.15);
    border-color: #38bdf8;
    color: #38bdf8;
    font-weight: 600;
  }

  .form-actions-row {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .btn-sm {
    padding: 4px 10px;
    font-size: 11px;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 5px;
    font-size: 11px;
    color: #f87171;
  }

  /* Empty State */
  .empty-fwd-state {
    padding: 30px 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    background: rgba(0, 0, 0, 0.15);
    border: 1px dashed var(--border-subtle);
    border-radius: 8px;
    gap: 8px;
  }

  .empty-icon-circle {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.04);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 4px;
  }

  .empty-fwd-state h4 {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-main);
  }

  .empty-fwd-state p {
    font-size: 11px;
    color: var(--text-dim);
    max-width: 440px;
    line-height: 1.5;
  }

  .btn-create-first {
    margin-top: 6px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 14px;
    font-size: 11.5px;
  }

  /* Forwarder Items List */
  .forwarders-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .fwd-item-card {
    background: rgba(30, 41, 59, 0.35);
    border: 1px solid var(--border-subtle);
    border-radius: 7px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: all 0.15s ease;
  }

  .fwd-item-card:hover {
    border-color: var(--border-medium);
    background: rgba(30, 41, 59, 0.5);
  }

  .fwd-item-card.is-active {
    border-color: rgba(16, 185, 129, 0.4);
    background: rgba(16, 185, 129, 0.04);
  }

  .fwd-item-card.is-has-error {
    border-color: rgba(239, 68, 68, 0.35);
  }

  .fwd-item-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .fwd-info-col {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .fwd-name-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .fwd-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-main);
  }

  .fwd-target-tag {
    font-size: 10.5px;
    color: #93c5fd;
    background: rgba(59, 130, 246, 0.12);
    padding: 1px 6px;
    border-radius: 4px;
    border: 1px solid rgba(59, 130, 246, 0.25);
  }

  .fwd-status-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .online-indicator {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .pulse-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #10b981;
    box-shadow: 0 0 6px #10b981;
  }

  .online-text {
    font-size: 10.5px;
    font-weight: 600;
    color: #34d399;
  }

  .starting-indicator {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10.5px;
    color: #38bdf8;
  }

  .error-indicator {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10.5px;
    color: #f87171;
  }

  .idle-indicator {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10.5px;
    color: var(--text-dim);
  }

  .idle-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #64748b;
  }

  .fwd-actions-col {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-start-tunnel {
    display: flex;
    align-items: center;
    gap: 5px;
    background: linear-gradient(135deg, #0284c7, #2563eb);
    border: none;
    border-radius: 5px;
    color: #ffffff;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-start-tunnel:hover {
    background: linear-gradient(135deg, #0369a1, #1d4ed8);
  }

  .btn-stop-tunnel {
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid rgba(239, 68, 68, 0.35);
    border-radius: 5px;
    color: #f87171;
    padding: 3px 9px;
    font-size: 10.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-stop-tunnel:hover {
    background: rgba(239, 68, 68, 0.25);
  }

  .btn-icon-subtle {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 4px 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .btn-icon-subtle:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--border-medium);
  }

  .btn-icon-subtle.danger:hover {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.4);
  }

  .fwd-error-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: 4px;
    font-size: 10.5px;
    color: #f87171;
  }

  .fwd-url-box {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(56, 189, 248, 0.25);
    border-radius: 6px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .fwd-url-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .fwd-url-input {
    flex: 1;
    background: var(--bg-input);
    border: 1px solid rgba(56, 189, 248, 0.3);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 11px;
    color: #38bdf8;
    outline: none;
  }

  .btn-copy-url {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #0284c7;
    border: none;
    border-radius: 4px;
    color: #ffffff;
    padding: 4px 9px;
    font-size: 10.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn-copy-url:hover {
    background: #0369a1;
  }

  .btn-copy-url.copied {
    background: #065f46;
  }
</style>
