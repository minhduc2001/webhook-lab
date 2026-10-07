<script lang="ts">
  import type { TunnelStatus, ServerStatus, ApiForwarder } from '../types';
  import { copyToClipboard } from '../lib/utils';
  import Icons from './Icons.svelte';
  import Logo from './Logo.svelte';

  let {
    tunnelStatus,
    serverStatus,
    forwarders = [],
    appVersion = '0.1.0',
    onToggleTunnel,
    onOpenForwarderModal,
    onOpenAboutModal,
    onSendTest,
    onClearRequests,
    isTunnelLoading = false,
  } = $props<{
    tunnelStatus: TunnelStatus;
    serverStatus: ServerStatus;
    forwarders?: ApiForwarder[];
    appVersion?: string;
    onToggleTunnel: () => void;
    onOpenForwarderModal: () => void;
    onOpenAboutModal?: () => void;
    onSendTest: (preset: string) => void;
    onClearRequests: () => void;
    isTunnelLoading?: boolean;
  }>();

  let copied = $state(false);
  let showPresets = $state(false);
  let runningCount = $derived(forwarders.filter((f: ApiForwarder) => f.is_running).length);

  async function handleCopyUrl() {
    if (!tunnelStatus.public_url) return;
    const ok = await copyToClipboard(tunnelStatus.public_url);
    if (ok) {
      copied = true;
      setTimeout(() => (copied = false), 2000);
    }
  }
</script>

<header class="top-bar">
  <div class="brand">
    <Logo size={23} />
    <span class="brand-title">Webhook Lab</span>
    <button
      class="brand-version-badge"
      onclick={onOpenAboutModal}
      title="Phiên bản v{appVersion} - Bấm để kiểm tra cập nhật"
    >
      v{appVersion}
    </button>
  </div>

  <div class="top-bar-center">
    <!-- Tunnel Pill -->
    <div class="tunnel-pill" class:online={tunnelStatus.is_running}>
      <span class="status-dot" class:active={tunnelStatus.is_running}></span>
      
      {#if tunnelStatus.is_running && tunnelStatus.public_url}
        <span class="tunnel-url" title={tunnelStatus.public_url}>
          {tunnelStatus.public_url}
        </span>
        <button
          class="btn-icon"
          onclick={handleCopyUrl}
          title={copied ? 'Đã sao chép!' : 'Sao chép URL'}
        >
          <Icons name={copied ? 'check' : 'copy'} size={12} color={copied ? '#34d399' : '#94a3b8'} />
        </button>
        <button
          class="btn-text-danger"
          onclick={onToggleTunnel}
          disabled={isTunnelLoading}
          title="Dừng Cloudflare Tunnel"
        >
          Dừng
        </button>
      {:else if isTunnelLoading || tunnelStatus.is_installing}
        <span class="tunnel-loading" title={tunnelStatus.status_message || ''}>
          <Icons name="refresh" size={12} class="spin" />
          <span>{tunnelStatus.status_message || 'Đang kết nối tunnel...'}</span>
        </span>
      {:else}
        {#if tunnelStatus.error}
          <span class="tunnel-error-text" title={tunnelStatus.error}>
            <Icons name="alert-circle" size={11} color="#f87171" />
            <span>Lỗi khởi động</span>
          </span>
        {:else}
          <span class="tunnel-idle-text">Chỉ Local</span>
        {/if}
        <button
          class="btn-tunnel-start"
          onclick={onToggleTunnel}
          disabled={isTunnelLoading || !!tunnelStatus.is_installing}
          title="Khởi động Cloudflare Quick Tunnel mở ra Internet"
        >
          <Icons name="globe" size={12} color="#38bdf8" />
          <span>Mở Internet</span>
        </button>
      {/if}
    </div>

    <!-- Server Port Badge -->
    <div class="port-badge" title="Cổng Server HTTP Cục Bộ">
      <span class="port-dot"></span>
      <span>:{serverStatus.port}</span>
    </div>

    <!-- API Forwarder Tool Button (Multi-forwarder) -->
    <button
      class="btn-forwarder-nav"
      class:active={runningCount > 0}
      onclick={onOpenForwarderModal}
      title="Public bất kỳ local port nào ra Internet"
    >
      <Icons name="globe" size={12} color={runningCount > 0 ? '#34d399' : '#94a3b8'} />
      <span>Forward API</span>
      {#if runningCount > 0}
        <span class="active-port-pill">{runningCount} Bật</span>
      {/if}
    </button>
  </div>

  <div class="top-bar-actions">
    <!-- Send Test Webhook Dropdown -->
    <div class="dropdown-wrapper">
      <button
        class="btn-action-test"
        onclick={() => (showPresets = !showPresets)}
        title="Gửi thử request webhook giả lập để kiểm tra"
      >
        <Icons name="play" size={13} color="#60a5fa" />
        <span>Bắn Thử</span>
      </button>

      {#if showPresets}
        <div class="dropdown-menu">
          <div class="dropdown-header">Mẫu Webhook Giả Lập</div>
          <button
            class="dropdown-item"
            onclick={() => {
              onSendTest('stripe');
              showPresets = false;
            }}
          >
            <span class="preset-name">Thanh toán Stripe</span>
            <span class="preset-desc">payment_intent.succeeded</span>
          </button>
          <button
            class="dropdown-item"
            onclick={() => {
              onSendTest('github');
              showPresets = false;
            }}
          >
            <span class="preset-name">Sự kiện GitHub Push</span>
            <span class="preset-desc">refs/heads/main</span>
          </button>
          <button
            class="dropdown-item"
            onclick={() => {
              onSendTest('custom');
              showPresets = false;
            }}
          >
            <span class="preset-name">JSON Tùy Chỉnh</span>
            <span class="preset-desc">timestamp & payload</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Clear Requests -->
    <button
      class="btn-action-clear"
      onclick={onClearRequests}
      title="Xóa danh sách các request đã ghi nhận"
    >
      <Icons name="trash" size={14} color="#94a3b8" />
    </button>
  </div>
</header>

<style>
  .brand-version-badge {
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    color: #38bdf8;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-mono);
    padding: 1px 5px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .brand-version-badge:hover {
    background: rgba(56, 189, 248, 0.25);
    border-color: #38bdf8;
  }

  .tunnel-idle-text {
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .tunnel-error-text {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: #f87171;
  }

  .btn-tunnel-start {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    border-radius: 12px;
    color: #38bdf8;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-tunnel-start:hover {
    background: rgba(56, 189, 248, 0.22);
    border-color: #38bdf8;
  }

  .btn-text-danger {
    background: transparent;
    border: none;
    color: #f87171;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    transition: background-color 0.15s;
  }

  .btn-text-danger:hover {
    background: rgba(239, 68, 68, 0.15);
  }

  .btn-icon {
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3px;
    border-radius: 4px;
  }

  .btn-icon:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .port-badge {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-subtle);
    padding: 2px 7px;
    border-radius: 10px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .port-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #10b981;
  }

  .dropdown-wrapper {
    position: relative;
  }

  .btn-action-test {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(59, 130, 246, 0.12);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-action-test:hover {
    background: rgba(59, 130, 246, 0.22);
    border-color: #60a5fa;
  }

  .btn-action-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    padding: 4px 7px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-action-clear:hover {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.3);
    color: #f87171;
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    width: 200px;
    background: var(--bg-card);
    border: 1px solid var(--border-medium);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    z-index: 50;
    padding: 4px;
    display: flex;
    flex-direction: column;
  }

  .dropdown-header {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 6px 8px 4px;
  }

  .dropdown-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 7px 9px;
    border-radius: 5px;
    background: transparent;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    text-align: left;
    transition: background-color 0.12s;
  }

  .dropdown-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .preset-name {
    font-size: 12px;
    font-weight: 600;
  }

  .preset-desc {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-dim);
  }

  .tunnel-loading {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: #38bdf8;
  }

  .btn-forwarder-nav {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-forwarder-nav:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
    border-color: var(--border-medium);
  }

  .btn-forwarder-nav.active {
    background: rgba(16, 185, 129, 0.12);
    border-color: rgba(16, 185, 129, 0.4);
    color: #34d399;
  }

  .active-port-pill {
    font-family: var(--font-mono);
    font-size: 9.5px;
    background: rgba(16, 185, 129, 0.25);
    padding: 1px 4px;
    border-radius: 3px;
    color: #34d399;
  }
</style>
