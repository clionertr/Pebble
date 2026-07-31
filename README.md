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
  <a href="LICENSE">License</a>
</p>

<p align="center">
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

You need Docker and Docker Compose installed. The installer pulls the latest tagged GHCR image, creates `./pebble`, writes `.env`, starts the single Pebble container, and checks `http://127.0.0.1:9191`. If Docker needs elevated privileges and passwordless sudo is available, the installer will use `sudo -n docker` automatically.

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

> Prefer integrity verification over `curl | bash`? Download the script, check its SHA-256 against the published checksum, then execute:
>
> ```bash
> curl -fsSLo install.sh https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh
> curl -fsSLo install.sh.sha256 https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh.sha256
> sha256sum -c install.sh.sha256   # fails loudly on mismatch
> bash install.sh
> ```

### Development from Source

You need: **Rust** (stable), **Node.js 22+**, **pnpm 11+**.

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
If you access Vite through a reverse proxy or remote dev domain, set comma-separated hostnames in `PEBBLE_VITE_ALLOWED_HOSTS`, for example `PEBBLE_VITE_ALLOWED_HOSTS=pebble.example.com,dev.example.com`.

Important dev rule: run only one backend process against the same `./data` directory. If another `cargo run` or Docker container is already using the data directory, the search index will be locked and the next backend start will fail.

## Configuration Guide

All configuration goes into **environment variables**. You can set them in a `.env` file, pass them directly when running from source, or use Docker Compose's `env_file`. Direct source runs read `.env` from the current working directory without an extra `source .env` step.

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

### Single-container Docker (recommended)

Pebble now serves both the React SPA and the Webmail API from the Rust backend. The Docker image contains the frontend `dist/` files, so production deployment only needs one container.

The one-command installer writes a compose file from `deploy/compose.prod.yml`. If you want to maintain it manually, use the prebuilt GHCR image:

`latest` is updated only when this repository pushes a version tag such as `v0.0.12`.

```yaml
name: pebble

services:
  pebble:
    image: ghcr.io/clionertr/pebble:latest
    env_file:
      - .env
    environment:
      PEBBLE_HOST: 0.0.0.0
      PEBBLE_PORT: 3000
    ports:
      - "127.0.0.1:9191:3000"
    volumes:
      - ./data:/app/data
    restart: unless-stopped
```

Point your public reverse proxy (nginx, Caddy, 1Panel OpenResty, etc.) to `http://127.0.0.1:9191`. Pebble itself handles frontend files, `/api/*`, `/events`, `/auth/*`, and `/webhook/*`; your reverse proxy only needs to forward the whole site.

### Cloudflare Tunnel

For Cloudflare Tunnel, create a Public Hostname and set the service to:

```text
http://127.0.0.1:9191
```

No extra path rules are required.

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

### Releasing a new version (one-click)

Releasing is one button in GitHub Actions. The **Release** workflow (`.github/workflows/release.yml`) runs `deploy/release.sh`, which:

1. computes the version — leave the input blank to auto-bump the patch (`0.0.12 → 0.0.13`);
2. aborts if the latest `ci.yml` run on `master` is not green;
3. bumps `package.json` and `server/Cargo.toml`, and moves the `[Unreleased]` notes under a new `## [X.Y.Z] - <date>` heading;
4. commits `chore(release): vX.Y.Z`, tags `vX.Y.Z`, and pushes both;
5. triggers `docker.yml`, which builds and pushes the multi-arch image (and updates `latest` for non-prerelease versions);
6. creates a GitHub Release whose notes are the changelog section just written.

Steps: **Actions → Release → Run workflow** → optionally type the version → **Run workflow**.

> One-time setup: the tag push must use a personal access token, not `GITHUB_TOKEN` — GitHub does not re-trigger workflows from `GITHUB_TOKEN`-created events, so `docker.yml` would never run otherwise. Create a PAT with `repo` access (or fine-grained `contents: read & write`) and add it as the `RELEASE_PAT` secret under **Settings → Secrets and variables → Actions**.

The same logic runs locally: `./deploy/release.sh` (full release), `./deploy/release.sh --dry-run v0.0.13` (preview the file diffs without touching the repo), or `./deploy/release.sh --self-check`.

## How It Works

### Architecture

```
Browser (React SPA)
        │  Frontend static files
        │  HTTP REST  /api/*
        │  SSE stream /events
        │  OAuth flow /auth/login  /auth/callback
        ▼
Rust HTTP Server (Axum, port 3000)
        │  serves frontend static files and Webmail API
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

For a deeper developer view, see [`docs/architecture.md`](docs/architecture.md) and [`docs/integration-guide.md`](docs/integration-guide.md).

### Authentication

Pebble uses **cookie-based session auth**:
- You log in with your password → server creates a session (7-day TTL)
- Session cookie (`pebble_session`) is `HttpOnly; Secure; SameSite=Strict`
- All `/api/*` endpoints require a valid session
- Failed logins are rate-limited (5 attempts → 15-minute lock per IP)
- No registration, no multi-user — it's single-user by design

### Real-time Updates

The frontend connects to `GET /events` via **Server-Sent Events** (SSE). The server pushes notifications for new mail, sync progress, and snooze wakeups. The SSE connection uses the same session cookie for auth.

On startup, the web app fetches `GET /api/shell` once for account metadata, folders, unread counts, and Gmail real-time configuration. Message and thread lists stay paginated through `/api/inbox` and `/api/threads`; Pebble does not load all historical mail into the startup snapshot.

Routine sync poll completions update the status bar only. The frontend refreshes shell/list caches on actual change signals such as `mail:new`, pending remote-write changes, network recovery, or one-shot sync completion.

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
| `pnpm test` | Run frontend tests (Vitest) |
| `cargo fmt --check` | Check Rust formatting |
| `cargo clippy --all-targets -- -D warnings` | Run Rust lint checks |
| `cargo test --workspace --all-targets` | Run all Rust tests |

## Troubleshooting

### "Authentication required" on every request
Your session expired (7-day TTL) or the backend restarted. Log in again.

### Can't log in after deployment
Check that `PEBBLE_PASSWORD_HASH` in `.env` has `$$` escaping (not `$`) when used with Docker Compose. Test with: `docker exec pebble env | grep PASSWORD`.

For direct source runs, use normal single `$` characters, usually quoted: `PEBBLE_PASSWORD_HASH='$2b$12$...'`. The backend reads `.env` automatically from the directory where you start it.

### `Failed to acquire index lock` or `LockBusy`

Pebble's full-text search index lives in `data/index/`. Tantivy allows only one writer, so this error almost always means another Pebble backend is still running with the same `./data` directory.

Check and stop the old process:

```bash
docker ps --filter name=pebble
pgrep -af pebble
```

Then start only one backend again: either Docker or `cargo run -p pebble`, not several at the same time.

If `pgrep -af pebble` shows no running process but the lock remains, reboot the server first. Only remove stale lock files under `data/index/` after confirming no Pebble process is running and after backing up `data/`.

### Routes returning 404
Make sure your reverse proxy forwards the whole site to `http://127.0.0.1:9191`. Pebble handles frontend routes and API routes itself.

### Database "disk image is malformed"  
The SQLite database may have been corrupted by an unclean shutdown. Try: `sqlite3 data/pebble.db "PRAGMA integrity_check;"`. If corrupted, restore from backup.

### Email sync not working
Check the backend logs: `docker logs pebble` or `tail -f data/logs/`. Common issues: OAuth token expired (re-authenticate in Settings → Accounts), network proxy not configured, IMAP credentials wrong.

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
├── deploy/                 Docker deployment files
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
