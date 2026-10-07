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
            <Icons name="globe" size={17} color="#38bdf8" />
          </div>
          <h3 class="modal-title">Cài Đặt Cloudflare Tunnel</h3>
        </div>
        <button class="btn-close" onclick={onClose} aria-label="Đóng">
          <Icons name="x" size={16} />
        </button>
      </div>

      <div class="modal-body">
        <p class="install-desc">
          Máy tính của bạn chưa có công cụ <strong>cloudflared</strong>. 
          Webhook Lab sẽ tự động tải phiên bản chính hãng từ Cloudflare (~35MB) để tạo URL HTTPS công khai ra Internet.
        </p>

        <div class="mini-spec-row">
          <div class="spec-tag">
            <Icons name="check" size={12} color="#10b981" />
            <span>Miễn phí 100%</span>
          </div>
          <div class="spec-tag">
            <Icons name="check" size={12} color="#10b981" />
            <span>Không cần tài khoản</span>
          </div>
          <div class="spec-tag">
            <Icons name="check" size={12} color="#10b981" />
            <span>HTTPS an toàn</span>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" onclick={onClose}>
          Hủy
        </button>
        <button
          class="btn-primary"
          onclick={() => {
            onClose();
            onConfirm();
          }}
        >
          <Icons name="download" size={13} color="#ffffff" />
          <span>Tải & Cài Đặt</span>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .install-card {
    max-width: 440px;
    border: 1px solid rgba(56, 189, 248, 0.3);
  }

  .title-with-badge {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .icon-bubble {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
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

  .install-desc {
    font-size: 11.5px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .install-desc strong {
    color: #e2e8f0;
  }

  .mini-spec-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
  }

  .spec-tag {
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(16, 185, 129, 0.08);
    border: 1px solid rgba(16, 185, 129, 0.25);
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 10.5px;
    color: #34d399;
  }
</style>
