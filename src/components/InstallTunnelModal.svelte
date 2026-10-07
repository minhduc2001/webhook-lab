<script lang="ts">
  import Icons from './Icons.svelte';

  let {
    isOpen,
    onClose,
    onConfirm,
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
  }>();
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
      class="modal-card install-card"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <div class="title-with-badge">
          <div class="icon-bubble">
            <Icons name="globe" size={18} color="#38bdf8" />
          </div>
          <div class="title-col">
            <h3 class="modal-title">Xác Nhận Cài Đặt Cloudflare Quick Tunnel</h3>
            <span class="modal-subtitle">Tạo URL HTTPS công khai để nhận webhook từ Internet về máy tính</span>
          </div>
        </div>
        <button class="btn-close" onclick={onClose} aria-label="Đóng">
          <Icons name="x" size={16} />
        </button>
      </div>

      <div class="modal-body">
        <div class="info-banner">
          <Icons name="zap" size={15} color="#f59e0b" />
          <span>
            Máy tính của bạn chưa có công cụ <strong>Cloudflare Tunnel (cloudflared)</strong>. Bạn có muốn Webhook Lab tự động tải về và cài đặt không?
          </span>
        </div>

        <div class="specs-grid">
          <div class="spec-item">
            <span class="spec-label">Công cụ:</span>
            <span class="spec-val">Cloudflare Tunnel (<code>cloudflared</code>)</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Nguồn tải:</span>
            <span class="spec-val">Chính hãng từ Cloudflare GitHub Releases</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Dung lượng:</span>
            <span class="spec-val">~35 MB (Tải 1 lần duy nhất)</span>
          </div>
          <div class="spec-item">
            <span class="spec-label">Tài khoản:</span>
            <span class="spec-val text-green">Miễn phí 100%, không cần đăng ký tài khoản</span>
          </div>
        </div>

        <div class="features-list">
          <div class="feature-row">
            <Icons name="check" size={14} color="#10b981" />
            <span>Tạo URL HTTPS công khai (VD: <code>https://xxxx.trycloudflare.com</code>) trong 2 giây.</span>
          </div>
          <div class="feature-row">
            <Icons name="check" size={14} color="#10b981" />
            <span>Nhận webhook từ Stripe, ZaloPay, MoMo, GitHub, PayOS về thẳng <code>localhost:4567</code>.</span>
          </div>
          <div class="feature-row">
            <Icons name="check" size={14} color="#10b981" />
            <span>Không cần mở port router (No Port Forwarding), an toàn và bảo mật cao.</span>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" onclick={onClose}>
          Hủy Bỏ
        </button>
        <button
          class="btn-primary btn-confirm-install"
          onclick={() => {
            onClose();
            onConfirm();
          }}
        >
          <Icons name="download" size={14} color="#ffffff" />
          <span>Đồng Ý & Tự Động Tải Về</span>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .install-card {
    max-width: 580px;
    border: 1px solid rgba(56, 189, 248, 0.25);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6), 0 0 24px rgba(56, 189, 248, 0.1);
  }

  .title-with-badge {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon-bubble {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .title-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .modal-subtitle {
    font-size: 11px;
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

  .btn-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .info-banner {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.22);
    border-radius: 8px;
    padding: 10px 12px;
    font-size: 12.5px;
    color: var(--text-main);
    line-height: 1.45;
  }

  .specs-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-medium);
    border-radius: 8px;
    padding: 12px 14px;
  }

  .spec-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .spec-label {
    font-size: 11px;
    color: var(--text-dim);
  }

  .spec-val {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-main);
  }

  .spec-val code {
    font-family: var(--font-mono);
    color: #60a5fa;
  }

  .text-green {
    color: #34d399 !important;
  }

  .features-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-left: 2px;
  }

  .feature-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .feature-row code {
    font-family: var(--font-mono);
    color: #93c5fd;
  }

  .btn-confirm-install {
    background: linear-gradient(135deg, #0284c7, #2563eb);
    border: none;
    box-shadow: 0 2px 10px rgba(37, 99, 235, 0.35);
  }

  .btn-confirm-install:hover {
    background: linear-gradient(135deg, #0369a1, #1d4ed8);
  }
</style>
