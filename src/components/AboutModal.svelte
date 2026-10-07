<script lang="ts">
  import Icons from './Icons.svelte';
  import Logo from './Logo.svelte';
  import { api } from '../lib/api';

  let {
    isOpen,
    appVersion = '0.1.0',
    onClose,
  } = $props<{
    isOpen: boolean;
    appVersion: string;
    onClose: () => void;
  }>();

  let isChecking = $state(false);
  let isInstalling = $state(false);
  let checkResult = $state<{
    checked: boolean;
    hasUpdate: boolean;
    newVersion?: string;
    notes?: string;
    message?: string;
    isError?: boolean;
  }>({
    checked: false,
    hasUpdate: false,
  });

  async function handleCheckUpdates() {
    isChecking = true;
    checkResult = { checked: false, hasUpdate: false };

    try {
      const res = await api.checkForAppUpdates();
      if (res.available && res.version) {
        checkResult = {
          checked: true,
          hasUpdate: true,
          newVersion: res.version,
          notes: res.body || 'Không có mô tả chi tiết.',
        };
      } else if (res.error) {
        checkResult = {
          checked: true,
          hasUpdate: false,
          isError: true,
          message: res.error,
        };
      } else {
        checkResult = {
          checked: true,
          hasUpdate: false,
          message: 'Bạn đang sử dụng phiên bản mới nhất!',
        };
      }
    } catch (err: any) {
      checkResult = {
        checked: true,
        hasUpdate: false,
        isError: true,
        message: err?.message || 'Không thể kiểm tra cập nhật.',
      };
    } finally {
      isChecking = false;
    }
  }

  async function handleInstallUpdate() {
    isInstalling = true;
    try {
      await api.installAppUpdate();
    } catch (err: any) {
      alert('Lỗi cập nhật: ' + (err?.message || 'Không thể cài đặt bản mới'));
      isInstalling = false;
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
    <div class="modal-card about-card" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <span class="modal-title">Thông Tin & Cập Nhật Ứng Dụng</span>
        <button class="btn-close" onclick={onClose} aria-label="Đóng">
          <Icons name="x" size={16} />
        </button>
      </div>

      <div class="modal-body about-body">
        <!-- Logo & Version Header -->
        <div class="about-hero">
          <Logo size={42} />
          <h2 class="app-name">Webhook Lab</h2>
          <div class="version-pill-row">
            <span class="version-tag-pill mono">v{appVersion}</span>
            <span class="channel-pill">Kênh Ổn Định (Production)</span>
          </div>
          <p class="app-summary">
            Công cụ giả lập, gỡ lỗi và chuyển tiếp Webhook / API cục bộ không giới hạn tính năng.
          </p>
        </div>

        <!-- Update Checker Section -->
        <div class="update-section">
          <div class="update-section-header">
            <div class="update-header-title">
              <Icons name="refresh" size={14} color="#38bdf8" />
              <span>Cập Nhật Tự Động (Auto-Updater)</span>
            </div>
            <button
              class="btn-check-update"
              disabled={isChecking || isInstalling}
              onclick={handleCheckUpdates}
            >
              {#if isChecking}
                <Icons name="refresh" size={12} class="spin" />
                <span>Đang kiểm tra...</span>
              {:else}
                <Icons name="download" size={12} />
                <span>Kiểm Tra Bản Mới</span>
              {/if}
            </button>
          </div>

          <!-- Update status display -->
          {#if checkResult.checked}
            {#if checkResult.hasUpdate}
              <div class="update-available-box">
                <div class="update-alert-top">
                  <div class="update-dot-online"></div>
                  <span class="update-title-text">
                    Đã có phiên bản mới: <strong>v{checkResult.newVersion}</strong>
                  </span>
                </div>
                {#if checkResult.notes}
                  <div class="release-notes-box">
                    <span class="notes-label">Nội dung cập nhật:</span>
                    <pre class="notes-content">{checkResult.notes}</pre>
                  </div>
                {/if}
                <button
                  class="btn-install-now"
                  disabled={isInstalling}
                  onclick={handleInstallUpdate}
                >
                  {#if isInstalling}
                    <Icons name="refresh" size={13} class="spin" />
                    <span>Đang tải & cài đặt bản cập nhật...</span>
                  {:else}
                    <Icons name="download" size={13} />
                    <span>Cập Nhật & Khởi Động Lại Ngay</span>
                  {/if}
                </button>
              </div>
            {:else if checkResult.isError}
              <div class="update-status-box error">
                <Icons name="alert-circle" size={13} color="#f87171" />
                <span>{checkResult.message}</span>
              </div>
            {:else}
              <div class="update-status-box success">
                <Icons name="check" size={13} color="#34d399" />
                <span>{checkResult.message}</span>
              </div>
            {/if}
          {:else}
            <div class="update-hint-row">
              <span class="hint-text">
                Ứng dụng tự động kiểm tra bản phát hành mới từ GitHub khi khởi động.
              </span>
            </div>
          {/if}
        </div>

        <!-- Technical Security & Key Info -->
        <div class="tech-info-card">
          <div class="tech-row">
            <span class="tech-label">Chữ Ký Bảo Mật:</span>
            <span class="tech-val mono">Ed25519 Minisign (Đã cấu hình)</span>
          </div>
          <div class="tech-row">
            <span class="tech-label">Nguồn Phát Hành:</span>
            <span class="tech-val mono">minhduc2001/webhook-lab</span>
          </div>
          <div class="tech-row">
            <span class="tech-label">Kiến Trúc Máy Chủ:</span>
            <span class="tech-val">Tauri v2 + Rust Axum + SQLite</span>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" onclick={onClose}>Đóng</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .about-card {
    max-width: 460px;
    width: 90%;
    border: 1px solid rgba(56, 189, 248, 0.3);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3px;
    border-radius: 4px;
  }

  .btn-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .about-body {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 20px;
  }

  .about-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 6px;
    padding-bottom: 6px;
  }

  .app-name {
    font-size: 17px;
    font-weight: 800;
    letter-spacing: -0.02em;
    color: #f8fafc;
    margin: 4px 0 0;
  }

  .version-pill-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .version-tag-pill {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 7px;
    border-radius: 10px;
    background: rgba(56, 189, 248, 0.15);
    border: 1px solid rgba(56, 189, 248, 0.35);
    color: #38bdf8;
  }

  .channel-pill {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-dim);
  }

  .app-summary {
    font-size: 11px;
    color: var(--text-dim);
    max-width: 380px;
    line-height: 1.4;
    margin: 2px 0 0;
  }

  /* Update Section */
  .update-section {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .update-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .update-header-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    font-weight: 700;
    color: #e2e8f0;
  }

  .btn-check-update {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    color: #38bdf8;
    padding: 3px 8px;
    border-radius: 5px;
    font-size: 10.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-check-update:hover {
    background: rgba(56, 189, 248, 0.22);
  }

  .update-hint-row {
    font-size: 10px;
    color: var(--text-dim);
  }

  .update-status-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-radius: 5px;
    font-size: 11px;
  }

  .update-status-box.success {
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid rgba(16, 185, 129, 0.3);
    color: #34d399;
  }

  .update-status-box.error {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
  }

  .update-available-box {
    background: rgba(16, 185, 129, 0.08);
    border: 1px solid rgba(16, 185, 129, 0.35);
    border-radius: 6px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .update-alert-top {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .update-dot-online {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #10b981;
    box-shadow: 0 0 6px #10b981;
  }

  .update-title-text {
    font-size: 11.5px;
    color: #e2e8f0;
  }

  .update-title-text strong {
    color: #34d399;
  }

  .release-notes-box {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .notes-label {
    font-size: 10px;
    color: var(--text-dim);
    font-weight: 600;
  }

  .notes-content {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    background: rgba(0, 0, 0, 0.25);
    padding: 5px 8px;
    border-radius: 4px;
    white-space: pre-wrap;
    max-height: 80px;
    overflow-y: auto;
  }

  .btn-install-now {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background: linear-gradient(135deg, #10b981, #059669);
    border: none;
    border-radius: 5px;
    color: #ffffff;
    padding: 5px 12px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-install-now:hover {
    background: linear-gradient(135deg, #059669, #047857);
  }

  /* Technical Info */
  .tech-info-card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .tech-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 10.5px;
  }

  .tech-label {
    color: var(--text-dim);
  }

  .tech-val {
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
