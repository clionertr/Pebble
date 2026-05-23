<p align="center">
  <img src="src/assets/app-icon.png" alt="Pebble logo" width="120">
</p>

<h1 align="center">Pebble</h1>

<p align="center">
  A self-hosted webmail client for people who want a calmer, more private inbox.
  <br>
  一个自托管的网页邮件客户端，让收件箱更安静、更私密。
</p>

<p align="center">
  <a href="README.zh-CN.md">简体中文</a>
  ·
  <a href="https://github.com/clionertr/Pebble/releases">Releases</a>
  ·
  <a href="LICENSE">License</a>
</p>

<p align="center">
  <a href="https://github.com/clionertr/Pebble/releases"><img src="https://img.shields.io/github/v/release/clionertr/Pebble?style=flat-square&color=d4714e" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20VPS%20%7C%20Self--hosted-lightgrey?style=flat-square" alt="Platform">
</p>

---

## What is Pebble?

Pebble turns your server into a private webmail portal. You log in through a browser, connect your email accounts (Gmail, IMAP, Outlook), and read/send/manage all your mail from one place. All data — messages, attachments, search index, settings — stays on **your** server.

Think of it as a self-hosted Gmail. No third party sees your inbox. No ads. No tracking.

**Pebble 是什么？** 它把你的服务器变成一个私人网页邮箱。在浏览器里登录，连接你的邮箱账户（支持 Gmail、IMAP、Outlook），在一个地方收发管理所有邮件。所有数据——邮件、附件、搜索索引、设置——都存在你自己的服务器上。

## Quick Start

Pick the method that fits you.

> This webmail fork is maintained at `https://github.com/clionertr/Pebble.git`. `https://github.com/QingJ01/Pebble.git` is the original upstream project; do not clone it when you want this webmail version.

### One-command Docker deploy (recommended)

You need Docker and Docker Compose installed. The installer pulls the latest tagged GHCR images, creates `./pebble`, writes `.env`, starts the services, and checks `http://127.0.0.1:9191`. If Docker needs elevated privileges and passwordless sudo is available, the installer will use `sudo -n docker` automatically.

```bash
curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh | bash
```

During setup you can accept or enter:

- your public URL; the default is auto-detected as `http://<server-ip>:9191`
- your Pebble login password; leave it blank to generate a 32-character password
- optional Google/Microsoft OAuth credentials

Point your reverse proxy to `http://127.0.0.1:9191`. All Pebble data is stored in `./pebble/data`.

Non-interactive examples:

```bash
# Fully automatic: detects IP and generates a 32-character login password
curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh | bash

# Use a domain and a provided password instead of generated defaults
curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh \
  | PEBBLE_PASSWORD='your-secret-password' \
    PEBBLE_PUBLIC_URL='https://mail.example.com' \
    bash
```

### Development from Source

You need: **Rust** (stable), **Node.js 18+**, **pnpm 8+**.

```bash
git clone https://github.com/clionertr/Pebble.git
cd Pebble

# Install frontend dependencies
pnpm install

# Copy and edit environment config
cp .env.example .env
# Generate a hash with: printf '%s' 'your-password' | cargo run -p pebble -- hash-password
# Set PEBBLE_PASSWORD_HASH in .env

# Terminal 1: Start the backend
cargo run -p pebble

# Terminal 2: Start the frontend dev server
pnpm dev:frontend
```

Open `http://localhost:1420`. The dev server proxies API calls to the backend at port 3000.

Important dev rule: run only one backend process against the same `./data` directory. If a release binary, `cargo run`, or a systemd service is already running, the search index will be locked and the next backend start will fail.

### Production from Source

For a VPS that runs from source, use a process manager such as systemd. The mental model is:

1. stop the old Pebble backend
2. pull/build the new code
3. start exactly one backend again

```bash
# One-time setup
git clone https://github.com/clionertr/Pebble.git /opt/pebble
cd /opt/pebble
pnpm install --frozen-lockfile
cp .env.example .env
printf '%s' 'your-password' | cargo run -p pebble -- hash-password
# Edit .env and set PEBBLE_PASSWORD_HASH to the generated hash.
# Direct source runs use single $ characters, for example '$2b$12$...'.
```

Build and restart after code changes:

```bash
# Update code if this server tracks git
git pull --ff-only

# Build while the old service keeps serving traffic
pnpm install --frozen-lockfile
pnpm run build:frontend
cargo build --release -p pebble

# Restart once. systemd stops the old backend before starting the new one.
sudo systemctl restart pebble
```

Serve `dist/` with nginx (example config below). The backend listens on port 3000 by default.

Example systemd unit. A ready-to-edit copy is also available at `deploy/pebble.service.example`:

```ini
[Unit]
Description=Pebble webmail backend
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/pebble
EnvironmentFile=/opt/pebble/.env
ExecStart=/opt/pebble/target/release/pebble
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Install it with:

```bash
sudo cp deploy/pebble.service.example /etc/systemd/system/pebble.service
sudo systemctl daemon-reload
sudo systemctl enable --now pebble
```

For quick manual testing without systemd, stop any existing Pebble process first, then run `./target/release/pebble` from the repository root. The binary now reads `.env` from the current working directory.

## Configuration Guide

All configuration goes into **environment variables**. You can set them in a `.env` file, pass them directly when running the binary, or use Docker Compose's `env_file`. Direct source runs read `.env` from the current working directory without an extra `source .env` step.

### Required: Password

| Variable | What it is | How to get it |
|---|---|---|
| `PEBBLE_PASSWORD_HASH` | Your login password, bcrypt-hashed | `printf '%s' 'your-password' \| pebble hash-password` |

This is the only required variable. Without it, the backend refuses to start.

### Optional: OAuth Providers

If you want Gmail or Outlook support, you need OAuth credentials.

#### Gmail

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Create a project, then create an **OAuth 2.0 Client ID** of type **Web application**
3. Add `https://your-domain.com/auth/callback` as an authorized redirect URI (use `http://localhost:3000/auth/callback` for local dev)
4. Copy the Client ID and Client Secret to your `.env`:

```
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=GOCSPX-your-secret
```

#### Outlook / Microsoft

1. Go to [Azure App Registrations](https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/), register a new app
2. Set redirect URI to `https://your-domain.com/auth/callback`
3. The client type should be **public/native** (no client secret needed). If you registered as a web app, provide the secret.

```
MICROSOFT_CLIENT_ID=your-microsoft-client-id
# MICROSOFT_CLIENT_SECRET=  (leave empty for public/native apps)
```

### Optional: Server Settings

| Variable | Default | What it does |
|---|---|---|
| `PEBBLE_HOST` | `127.0.0.1` | IP address the server listens on. Set to `0.0.0.0` to accept external connections. |
| `PEBBLE_PORT` | `3000` | TCP port. |
| `OAUTH_REDIRECT_URL` | `http://localhost:3000` | Full URL where `/auth/callback` is reachable. Set to `https://your-domain.com` in production. |
| `ALLOWED_ORIGIN` | (empty) | CORS allowed origin. Leave empty for same-origin. Set to your frontend URL if hosting frontend and backend on different origins. |
| `PEBBLE_VAPID_PRIVATE_KEY` | auto-generated | Optional base64url VAPID private key for browser Web Push. If omitted, Pebble generates and stores one in its local data. |
| `PEBBLE_VAPID_PUBLIC_KEY` | derived | Optional VAPID public key. If set, it must match `PEBBLE_VAPID_PRIVATE_KEY`; otherwise the server refuses to start. |

### Optional: Gmail Real-time Push

Gmail can push new-email notifications to Pebble via Google Cloud Pub/Sub, instead of polling.

| Variable | What it is |
|---|---|
| `GMAIL_PUBSUB_TOPIC` | Full Pub/Sub topic: `projects/<project-id>/topics/gmail-webmail-topic` |
| `GMAIL_WEBHOOK_SECRET` | A random secret string for the webhook URL |

Setup steps:
1. Enable Gmail API and Cloud Pub/Sub API in Google Cloud
2. Create a Pub/Sub topic, grant `roles/pubsub.publisher` to `gmail-api-push@system.gserviceaccount.com`
3. Create a push subscription pointing at `https://your-domain.com/webhook/gmail?secret=<your-secret>`
4. In Pebble, go to **Settings → Accounts → Enable realtime Gmail** per account

## Production Deployment

### Nginx Reverse Proxy

The recommended setup: nginx serves the frontend static files and proxies API calls to the backend.

```nginx
server {
    listen 443 ssl;
    server_name mail.your-domain.com;

    root /path/to/Pebble/dist;
    index index.html;

    # Security headers
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Referrer-Policy "no-referrer" always;
    add_header Content-Security-Policy "default-src 'self'; img-src 'self' data: https:; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'" always;

    # Frontend SPA — fall back to index.html for client-side routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Backend API, SSE (real-time events), OAuth, and Gmail webhook
    location ~ ^/(api|events|auth|webhook) {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;

        # Required for Server-Sent Events (real-time updates)
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 3600s;
    }
}
```

### Docker Compose (Production)

The one-command installer writes a compose file from `deploy/compose.prod.yml`. If you want to maintain it manually, use the prebuilt GHCR images:

`latest` is updated only when this repository pushes a version tag such as `v0.0.9`.

```yaml
name: pebble

services:
  backend:
    image: ghcr.io/clionertr/pebble:latest
    volumes:
      - ./data:/app/data
    env_file:
      - .env
    environment:
      PEBBLE_HOST: 0.0.0.0
      PEBBLE_PORT: 3000
    restart: unless-stopped
    networks:
      - pebble-net

  frontend:
    image: ghcr.io/clionertr/pebble-frontend:latest
    ports:
      - "127.0.0.1:9191:80"
    depends_on:
      - backend
    restart: unless-stopped
    networks:
      - pebble-net

networks:
  pebble-net:
    driver: bridge
```

With this setup, point your public reverse proxy (nginx, Caddy, 1Panel OpenResty, etc.) to `http://127.0.0.1:9191`.

### Data Persistence

All data lives in the `./data/` directory relative to where the backend runs:

| File / Directory | Contains |
|---|---|
| `data/pebble.db` | SQLite database with all messages, accounts, rules, settings |
| `data/pebble.key` | Encryption key for stored credentials (OAuth tokens, passwords) |
| `data/index/` | Tantivy full-text search index |
| `data/attachments/` | Downloaded email attachments |
| `data/logs/` | Application logs |

**Keep `data/pebble.key` safe.** If you lose it, you lose access to all connected accounts and need to re-authenticate.

## How It Works

### Architecture

```
Browser (React SPA)
        │  HTTP REST  /api/*
        │  SSE stream /events
        │  OAuth flow /auth/login  /auth/callback
        ▼
Nginx (serves frontend, proxies API)
        │
        ▼
Rust HTTP Server (Axum, port 3000)
        │
        ├── pebble-store    SQLite database
        ├── pebble-search   Tantivy full-text index
        ├── pebble-mail     IMAP / Gmail / Outlook sync
        ├── pebble-crypto   Credential encryption
        ├── pebble-oauth    OAuth 2.0 + PKCE
        ├── pebble-rules    Rules engine
        ├── pebble-translate Translation
        └── pebble-privacy  HTML sanitizing & tracker protection
```

### Authentication

Pebble uses **cookie-based session auth**:
- You log in with your password → server creates a session (7-day TTL)
- Session cookie (`pebble_session`) is `HttpOnly; Secure; SameSite=Strict`
- All `/api/*` endpoints require a valid session
- Failed logins are rate-limited (5 attempts → 15-minute lock per IP)
- No registration, no multi-user — it's single-user by design

### Real-time Updates

The frontend connects to `GET /events` via **Server-Sent Events** (SSE). The server pushes notifications for new mail, sync progress, and snooze wakeups. The SSE connection uses the same session cookie for auth.

Browser push notifications use **Web Push + Service Worker** so notifications can arrive after the Pebble tab is closed. Production browsers require HTTPS or another secure context; localhost works for development.

### Email Sync

Pebble syncs with your providers in the background:
- **Gmail**: OAuth + Gmail API (history-based sync) + optional Pub/Sub push
- **IMAP**: Standard IMAP polling with configurable intervals
- **Outlook**: OAuth + Microsoft Graph API (experimental)

## Features

### Mail
- Unified inbox across multiple accounts
- Gmail, IMAP, and experimental Outlook
- Thread view and message list view
- Archive, delete, star, mark read, batch actions, restore
- Snooze messages (bring them back later)
- Full-text search with advanced filters
- Rules engine for automatic mail organization
- Command palette and keyboard shortcuts

### Productivity
- **Kanban board**: Todo → Waiting → Done columns for email tasks
- **Translation**: Built-in translation providers, bilingual reading mode
- **Templates**: Reusable email templates
- **Trusted Senders**: Per-sender privacy controls (show images, etc.)
- **WebDAV backup**: Sync settings, rules, and Kanban data to a WebDAV server

### Privacy & Security
- All data stored locally on your server
- No telemetry, no tracking
- HTML email sanitization (removes trackers)
- OAuth tokens encrypted at rest

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust + Axum |
| Frontend | React 19 + TypeScript |
| State | Zustand + TanStack Query |
| Database | SQLite (rusqlite) |
| Search | Tantivy |
| Styling | Tailwind CSS |
| i18n | i18next (English, Chinese) |

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `J` / `K` | Move through messages |
| `Enter` | Open selected message |
| `E` | Archive |
| `S` | Toggle star |
| `R` | Reply |
| `A` | Reply all |
| `F` | Forward |
| `C` | Compose |
| `/` | Focus search |
| `Esc` | Close, cancel, go back |

Shortcuts can be customized in Settings.

## Useful Commands

| Command | Purpose |
|---|---|
| `cargo run -p pebble` | Run backend dev server |
| `pnpm dev:frontend` | Run frontend dev server (proxies to backend) |
| `pnpm build:frontend` | Type-check and build frontend to `dist/` |
| `cargo build --release -p pebble` | Build release backend binary |
| `pnpm test` | Run frontend tests (Vitest) |
| `cargo fmt --check` | Check Rust formatting |
| `cargo clippy --all-targets -- -D warnings` | Run Rust lint checks |
| `cargo test --all` | Run all Rust tests |
| `sudo systemctl restart pebble` | Restart a source-deployed backend managed by systemd |

## Troubleshooting

### "Authentication required" on every request
Your session expired (7-day TTL) or the backend restarted. Log in again.

### Can't log in after deployment
Check that `PEBBLE_PASSWORD_HASH` in `.env` has `$$` escaping (not `$`) when used with Docker Compose. Test with: `docker exec pebble-backend env | grep PASSWORD`.

For direct source runs, use normal single `$` characters, usually quoted: `PEBBLE_PASSWORD_HASH='$2b$12$...'`. The backend reads `.env` automatically from the directory where you start it.

### `Failed to acquire index lock` or `LockBusy`

Pebble's full-text search index lives in `data/index/`. Tantivy allows only one writer, so this error almost always means another Pebble backend is still running with the same `./data` directory.

Check and stop the old process:

```bash
sudo systemctl status pebble
sudo systemctl stop pebble
pgrep -af pebble
```

Then start only one backend again: either `sudo systemctl start pebble`, `cargo run -p pebble`, or `./target/release/pebble`, not several at the same time.

If `pgrep -af pebble` shows no running process but the lock remains, reboot the server first. Only remove stale lock files under `data/index/` after confirming no Pebble process is running and after backing up `data/`.

### Routes returning 404
Make sure the nginx config proxies `/api/*` to the backend. The proxy rule should be: `location ~ ^/(api|events|auth|webhook)`.

### Database "disk image is malformed"  
The SQLite database may have been corrupted by an unclean shutdown. Try: `sqlite3 data/pebble.db "PRAGMA integrity_check;"`. If corrupted, restore from backup.

### Email sync not working
Check the backend logs: `docker logs pebble-backend` or `tail -f data/logs/`. Common issues: OAuth token expired (re-authenticate in Settings → Accounts), network proxy not configured, IMAP credentials wrong.

## Project Structure

```text
Pebble/
├── src/                    React frontend (SPA)
│   ├── components/         Shared UI components
│   ├── features/           Inbox, compose, search, Kanban, settings, auth
│   ├── hooks/              React hooks and query helpers
│   ├── lib/                API client, SSE client, i18n, utilities
│   └── stores/             Zustand stores
├── server/                 Rust HTTP backend (Axum)
│   └── src/
│       ├── main.rs         Server entry point, route registration
│       ├── api/            REST API handlers (80+ endpoints)
│       ├── middleware/      Auth middleware (cookie validation)
│       ├── session.rs      Cookie sessions + rate limiter
│       └── rpc/            Internal service layer
├── crates/                 Rust workspace crates
│   ├── pebble-core/        Shared types and errors
│   ├── pebble-store/       SQLite persistence
│   ├── pebble-mail/        Mail providers and sync
│   ├── pebble-search/      Tantivy search index
│   ├── pebble-crypto/      Credential encryption
│   ├── pebble-oauth/       OAuth 2.0 and PKCE
│   ├── pebble-rules/       Rules engine
│   ├── pebble-translate/   Translation providers
│   └── pebble-privacy/     HTML sanitizing and tracker controls
├── deploy/                 Docker and nginx configs
├── tests/                  Frontend tests
└── site/                   Screenshots
```

## License

Pebble is licensed under [GNU Affero General Public License v3.0](LICENSE).

---

<p align="center">
  Originally built by <a href="https://github.com/QingJ01">QingJ</a>.
  <br>
  Web service re-architecture and documentation by <strong>Claude Opus 4.7</strong>.
  <br>
  Friend link: <a href="https://linux.do">LINUX DO</a>
</p>
