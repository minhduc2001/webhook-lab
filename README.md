# Webhook Lab

**Webhook Lab** is an ultra-lightweight, local-first webhook development and debugging desktop application.

Built specifically for backend developers, it combines the inspection capabilities of **webhook.site** / **RequestBin**, a programmable **mock server**, a **request replay engine**, and **zero-account Cloudflare Quick Tunnels** into a blazing-fast desktop application.

---

## ⚡ Technology Stack

- **Desktop Framework**: [Tauri v2](https://v2.tauri.app/) (No Electron, tiny memory footprint, native Webview)
- **Frontend**: [Svelte 5](https://svelte.dev/) with Runes (`$state`, `$derived`, `$effect`) + TypeScript + Vite
- **Styling**: Tailored Vanilla CSS design system (Dark mode, glassmorphism, responsive 3-pane layout, typography by *JetBrains Mono* and *Plus Jakarta Sans*)
- **Core / HTTP Server**: Rust + [Axum](https://github.com/tokio-rs/axum) + Tokio (high throughput, ultra-low idle CPU and memory)
- **Database / Persistence**: [rusqlite](https://github.com/rusqlite/rusqlite) (bundled SQLite with WAL mode, zero configuration)
- **Replay Client**: [reqwest](https://github.com/seanmonstar/reqwest) with rustls
- **Public Tunnel**: Integrated Cloudflare Quick Tunnel (`cloudflared`) with 1-click zero-account public HTTPS URLs

---

## ✨ Features

### 1. Webhook Endpoints
- Create and manage isolated webhook endpoints (e.g. `/wh/stripe-checkout`, `/wh/github-actions`, `/wh/orders`).
- Customize default HTTP response status (200, 201, 204, 400, 429, 500), content-type, headers, and body.

### 2. Real-Time Request Inspector
- Incoming requests stream directly into the UI with zero polling latency via native Tauri event emission.
- Inspect complete HTTP Method, Path, Query Parameters, Headers, and Raw/Parsed JSON Body with byte size indicators.
- Overview card displaying client IP (including proxy/tunnel IP resolution) and timing.

### 3. Conditional Response Rules Engine
- Match incoming requests against flexible criteria:
  - **Headers**: Check exact value, substring match, regex, or existence (e.g., `stripe-signature` verification failures, `X-GitHub-Event`).
  - **JSON Body**: Inspect nested JSON keys (e.g., `type == "payment_intent.succeeded"`).
  - **Path / Query / Method**: Route subpaths and query flags to custom responses.
- Define custom return status codes, headers, response bodies, and rule-specific latency delays.

### 4. Chaos & Simulation Engine
- **Latency Delay**: Simulate network latency from 0 to 5,000ms.
- **Error Simulation**: Set chaos failure rates (0 - 100%) to test how upstream providers and your retry logic handle 429 (Rate Limit) and 500 (Internal Error) responses.

### 5. Webhook Replay Tool
- Re-send captured requests directly to your local backend server (`http://localhost:3000/webhook`, `http://127.0.0.1:8000/hooks`, etc.).
- Modify headers or payload on the fly.
- Inspect the live replay response code, response headers, latency, and response body.

### 6. Zero-Account Public HTTPS Tunnel
- 1-click **"Expose to Internet"** button powered by Cloudflare Quick Tunnel.
- Instantly allocates a public HTTPS URL (e.g., `https://abc-123.trycloudflare.com/wh/<slug>`).
- **No account, credit card, or registration required.**
- 1-click copy directly into Stripe, GitHub, Shopify, or Clerk developer webhooks.

### 7. Export & Integration
- 1-click **Copy as cURL** command.
- Generate **JavaScript (Fetch)** and **Python** request code snippets.
- Built-in **"Send Test Webhook"** trigger with Stripe, GitHub, and custom presets.

---

## 🚀 Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (v1.75+)
- Optional: `cloudflared` for 1-click public tunnels (auto-detected if installed in PATH)

### Running in Development Mode

```bash
# 1. Install frontend dependencies
npm install

# 2. Run Tauri Desktop App in dev mode
npx tauri dev
```

Or run the web interface in standard browser preview:
```bash
npm run dev
```

### Building Production App

```bash
npx tauri build
```

The resulting native installer / executable will be in `src-tauri/target/release/`.

---

## 📁 Project Architecture

```
webhook-lab/
├── src/                         # Svelte 5 Frontend
│   ├── components/
│   │   ├── Header.svelte        # Top bar with Cloudflare tunnel toggle & test triggers
│   │   ├── EndpointsSidebar.svelte # Endpoint list, URL slug copy, and filter
│   │   ├── RequestsList.svelte  # Live incoming request stream with filters
│   │   ├── RequestDetail.svelte # Request inspector, payload formatter, cURL export
│   │   ├── EndpointModal.svelte # Endpoint settings, default response & chaos sliders
│   │   ├── RulesModal.svelte    # Conditional response mock rules engine
│   │   ├── ReplayModal.svelte   # Interactive webhook replay tool
│   │   └── Icons.svelte         # Ultra-light SVG vector icons
│   ├── lib/
│   │   ├── api.ts               # Unified Tauri invoke client with mock fallback
│   │   ├── utils.ts             # Time, cURL generator, status colors, formatting
│   │   └── mockData.ts          # Realistic initial test fixtures
│   ├── types/
│   │   └── index.ts             # TypeScript definitions
│   ├── app.css                  # Tailored dark-mode design system
│   ├── main.ts                  # Svelte 5 mount
│   └── App.svelte               # Root component coordinating state & layout
├── src-tauri/                   # Rust Backend
│   ├── src/
│   │   ├── commands.rs          # Tauri command handlers
│   │   ├── db.rs                # SQLite persistence (rusqlite WAL mode)
│   │   ├── models.rs            # Rust data structures (Serde)
│   │   ├── replay.rs            # HTTP replay engine (reqwest)
│   │   ├── server.rs            # Axum HTTP server with rule evaluation engine
│   │   ├── tunnel.rs            # Cloudflare Quick Tunnel child process manager
│   │   ├── lib.rs               # Tauri v2 setup & lifecycle
│   │   └── main.rs              # Application entrypoint
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri v2 app configuration
└── package.json                 # Frontend scripts & dependencies
```

---

## 🔒 Security & Local-First Philosophy
- All webhook requests, payloads, and rules are stored strictly on your local machine in an embedded SQLite database (`webhook_lab.db`).
- No telemetry, no external accounts, no cloud database.
