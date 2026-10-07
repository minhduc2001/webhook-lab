<script lang="ts">
  let { size = 26, showText = false, class: className = '' } = $props<{
    size?: number;
    showText?: boolean;
    class?: string;
  }>();
</script>

<div class="logo-wrap {className}" style="--logo-size: {size}px;">
  <svg
    width={size}
    height={size}
    viewBox="0 0 40 40"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="logo-svg"
  >
    <!-- 2D Base Squircle -->
    <rect
      x="2"
      y="2"
      width="36"
      height="36"
      rx="9"
      fill="#0f172a"
      stroke="url(#logo_border)"
      stroke-width="1.8"
    />

    <!-- Subtle 2D Grid Pattern in Background -->
    <path
      d="M10 2v36M20 2v36M30 2v36M2 10h36M2 20h36M2 30h36"
      stroke="#1e293b"
      stroke-width="0.8"
      stroke-opacity="0.6"
    />

    <!-- Webhook Connector Flow Lines (2D Vector) -->
    <!-- Incoming Event Arc: Cyan -->
    <path
      d="M11 20H19C21.2 20 23 18.2 23 16V13"
      stroke="#38bdf8"
      stroke-width="2.6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- Outgoing Dispatch Arc: Violet / Emerald -->
    <path
      d="M19 20C21.2 20 23 21.8 23 24V27H29"
      stroke="#818cf8"
      stroke-width="2.6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- Left Incoming Node Dot -->
    <circle cx="10" cy="20" r="3" fill="#38bdf8" />
    <circle cx="10" cy="20" r="1.2" fill="#ffffff" />

    <!-- Top Branch Node (Filter / Inspect) -->
    <circle cx="23" cy="12" r="2.8" fill="#34d399" />

    <!-- Right Target Node (Replay / Deliver) -->
    <circle cx="30" cy="27" r="3.2" fill="#818cf8" />
    <circle cx="30" cy="27" r="1.4" fill="#ffffff" />

    <!-- Center Pulse Core -->
    <circle cx="19" cy="20" r="2.5" fill="#f59e0b" />

    <!-- Gradients Definition -->
    <defs>
      <linearGradient id="logo_border" x1="2" y1="2" x2="38" y2="38" gradientUnits="userSpaceOnUse">
        <stop stop-color="#38bdf8" />
        <stop offset="0.5" stop-color="#818cf8" />
        <stop offset="1" stop-color="#34d399" />
      </linearGradient>
    </defs>
  </svg>

  {#if showText}
    <div class="logo-text-block">
      <span class="logo-brand-name">Webhook<span class="highlight-lab">Lab</span></span>
      <span class="logo-sub">Dev Studio</span>
    </div>
  {/if}
</div>

<style>
  .logo-wrap {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .logo-svg {
    display: block;
    filter: drop-shadow(0 2px 6px rgba(56, 189, 248, 0.25));
    transition: transform 0.2s ease;
  }

  .logo-wrap:hover .logo-svg {
    transform: scale(1.05);
  }

  .logo-text-block {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
  }

  .logo-brand-name {
    font-size: 13.5px;
    font-weight: 800;
    letter-spacing: -0.02em;
    color: #f8fafc;
  }

  .highlight-lab {
    background: linear-gradient(135deg, #38bdf8, #818cf8);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    margin-left: 2px;
  }

  .logo-sub {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #64748b;
  }
</style>
