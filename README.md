<p align="center">
  <img src="src/assets/app-icon.png" alt="Pebble logo" width="120">
</p>

<h1 align="center">Pebble</h1>

<p align="center">
  A local-first email client for people who want a calmer, more private inbox. Now runs as a self-hosted web service.
</p>

<p align="center">
  <a href="README.zh-CN.md">简体中文</a>
  ·
  <a href="https://github.com/QingJ01/Pebble/releases">Releases</a>
  ·
  <a href="LICENSE">License</a>
</p>

<p align="center">
  <a href="https://github.com/QingJ01/Pebble/releases"><img src="https://img.shields.io/github/v/release/QingJ01/Pebble?style=flat-square&color=d4714e" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square" alt="License"></a>
  <a href="https://github.com/QingJ01/Pebble/actions"><img src="https://img.shields.io/github/actions/workflow/status/QingJ01/Pebble/ci.yml?style=flat-square&label=build" alt="Build"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20VPS%20%7C%20Self--hosted-lightgrey?style=flat-square" alt="Platform">
</p>

## Overview

Pebble is a self-hosted email client built with Rust and React. It has been re-architected from a Tauri desktop application into a **web service**: the Rust backend runs as a standalone HTTP server, and the React frontend is served as a standard web application that connects to it over HTTP.

All mail data, the search index, attachments, rules, and application settings remain on your server.

The app is designed around a few practical ideas:

- Your mailbox should stay readable, fast, and quiet.
- Email workflows should be local-first instead of cloud-dashboard-first.
- Privacy controls should be explicit, visible, and easy to override per message.
- Search, snooze, rules, and a Kanban board should work together instead of living in separate tools.

Pebble currently supports Gmail, IMAP, and experimental Outlook accounts.

## Architecture

This fork replaces the original Tauri desktop shell with a client–server architecture:

```
Browser (React SPA)
        │  HTTP fetch  /rpc/batch
        │  SSE stream  /events
        │  OAuth flow  /auth/login  /auth/callback
        ▼
Rust HTTP Server  (Axum, port 3000)
        │
        ├── pebble-store    SQLite database
        ├── pebble-search   Tantivy full-text index
        ├── pebble-mail     IMAP / Gmail / Outlook sync
        ├── pebble-crypto   Credential encryption
        ├── pebble-oauth    OAuth 2.0 + PKCE
        ├── pebble-rules    Rules engine
        ├── pebble-translate Translation providers
        └── pebble-privacy  HTML sanitizing & tracker controls
```

### Key changes from upstream

| Upstream (Tauri desktop) | This fork (web service) |
| --- | --- |
| Tauri IPC (`invoke`) | HTTP JSON-RPC via `POST /rpc/batch` |
| Tauri event system | Server-Sent Events (SSE) via `GET /events` |
| Desktop OAuth redirect | HTTP OAuth flow at `/auth/login` and `/auth/callback` |
| App data in OS user dir | Local `./data/` directory (VPS-friendly) |
| Platform-native keyring | File-based key at `./data/pebble.key` |

## Highlights

### Local-first privacy

- Local SQLite database for messages, folders, labels, rules, and settings.
- Local Tantivy full-text index for fast search.
- Attachments are stored on disk under the `./data/attachments/` directory.
- OAuth tokens and credentials are encrypted with a per-server key file.
- No telemetry.
- Network requests are limited to features you configure: mail sync, translation, and optional WebDAV settings backup.

### Mail workflow

- Unified inbox across multiple accounts.
- Gmail, IMAP, and experimental Outlook support.
- Threaded and message-list views.
- Archive, delete, star, mark read, batch actions, and restore flows.
- Snooze messages and bring them back later.
- Full-text search and advanced filters.
- Rules engine for automatic organization.

### Productivity tools

- Kanban board with Todo, Waiting, and Done columns.
- Command palette and keyboard-first navigation.
- Built-in translation providers with bilingual reading.
- Dark and light themes.
- English and Chinese UI.
- Optional WebDAV backup for settings, rules, Kanban cards, and Kanban notes.

## Screenshots

<table>
  <tr>
    <td><img src="site/screenshots/inbox.png" alt="Inbox"><br><b>Inbox</b></td>
    <td><img src="site/screenshots/kanban.png" alt="Kanban board"><br><b>Kanban</b></td>
  </tr>
  <tr>
    <td><img src="site/screenshots/dark.png" alt="Dark mode"><br><b>Dark Mode</b></td>
    <td><img src="site/screenshots/settings.png" alt="Settings"><br><b>Settings</b></td>
  </tr>
</table>

## Tech Stack

| Layer | Technology |
| --- | --- |
| Backend server | Rust + Axum |
| Transport | JSON-RPC over HTTP, SSE for push events |
| Frontend | React 19, TypeScript |
| State | Zustand, TanStack Query |
| Database | SQLite via rusqlite |
| Search | Tantivy |
| Styling | Tailwind CSS |
| Localization | i18next |

## Getting Started

### Prerequisites

- Rust stable
- Node.js 18 or newer
- pnpm 8 or newer

### Development Setup

```bash
git clone https://github.com/QingJ01/Pebble.git
cd Pebble

pnpm install
cp .env.example .env
# Fill in your OAuth credentials in .env
```

Start the backend server (terminal 1):

```bash
cargo run -p pebble
```

Start the frontend dev server (terminal 2):

```bash
pnpm dev:frontend
```

Open `http://localhost:1420` in your browser. The Vite dev server automatically proxies `/rpc`, `/events`, `/auth`, and `/webhook` to the backend at port 3000.

### Production Deployment

Build the frontend:

```bash
pnpm build:frontend
```

The static files are written to `dist/`. Serve them with any web server (nginx, caddy, etc.) and proxy the `/rpc`, `/events`, `/auth`, and `/webhook` paths to the Rust backend.

Build and run the backend:

```bash
cargo build --release -p pebble
./target/release/pebble
```

Data is stored in `./data/` relative to the working directory. Point the backend at a persistent directory and keep `./data/pebble.key` safe — losing it means losing access to stored credentials.

Example nginx snippet (assuming backend on port 3000, frontend served from `dist/`):

```nginx
server {
    listen 443 ssl;
    server_name mail.example.com;

    root /path/to/Pebble/dist;
    index index.html;

    # Frontend SPA — fall back to index.html for client-side routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Backend API, SSE, OAuth, and Gmail Pub/Sub webhook
    location ~ ^/(rpc|events|auth|webhook) {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;

        # Required for SSE connections
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 3600s;
    }
}
```

## OAuth Configuration

Pebble can connect to Gmail and Outlook through OAuth. IMAP accounts use the IMAP/SMTP credentials configured in the app.

Copy `.env.example` to `.env`, then fill the provider values you need. The environment variables must be set **at compile time** so they are embedded into the release binary.

| Variable | Description |
| --- | --- |
| `GOOGLE_CLIENT_ID` | Google OAuth client ID. Use a Web application client and add `http://localhost:3000/auth/callback` as an authorized redirect URI. |
| `GOOGLE_CLIENT_SECRET` | Required for web application clients. |
| `MICROSOFT_CLIENT_ID` | Microsoft public/native app client ID. |
| `MICROSOFT_CLIENT_SECRET` | Optional. Leave empty for public/native Microsoft apps. |

> **Note**: Because the OAuth callback is now handled by the HTTP server at `/auth/callback`, you must configure `http://<your-host>/auth/callback` (or `http://localhost:3000/auth/callback` for local dev) as the authorized redirect URI in your Google/Microsoft app settings.

## Gmail Realtime Push

Gmail accounts can optionally use Gmail API `watch` notifications delivered through Google Cloud Pub/Sub. This is enabled per account from **Settings -> Accounts -> Enable realtime Gmail**; normal Gmail OAuth login and IMAP accounts do not depend on this configuration.

Runtime environment variables:

| Variable | Description |
| --- | --- |
| `GMAIL_PUBSUB_TOPIC` | Fully qualified Pub/Sub topic, for example `projects/<project-id>/topics/gmail-webmail-topic`. |
| `GMAIL_WEBHOOK_SECRET` | Shared secret required on the Pub/Sub push URL query string. Do not commit this value. |

Google Cloud setup:

1. Enable Gmail API and Cloud Pub/Sub API.
2. Create a Pub/Sub topic.
3. Grant `roles/pubsub.publisher` on that topic to `gmail-api-push@system.gserviceaccount.com`.
4. Create a push subscription pointing at `https://<your-host>/webhook/gmail?secret=<your-secret>`.
5. Expose `/webhook/gmail` through your reverse proxy to the Pebble backend.

Pebble renews enabled Gmail watches on startup and every 12 hours, renewing any watch that is missing an expiration or expires within 24 hours. Pub/Sub OIDC JWT validation is not part of this MVP; use the URL secret now and add authenticated push verification before treating the endpoint as production-hardened.

## API Reference

The backend exposes three endpoint groups:

### `POST /rpc`

Single JSON-RPC call. Request body: `{ "method": "<command>", "params": { ... } }`. Returns the result directly or `{ "error": "<message>" }` on failure.

### `POST /rpc/batch`

Array of JSON-RPC calls processed in order. Request body: `[{ "method": "...", "params": {...} }, ...]`. Returns a matching array of results.

### `GET /events`

Server-Sent Events stream. The frontend connects here to receive push notifications for new mail, sync status, snooze wakeups, and other real-time updates. Each event has a named type and a JSON payload.

### `GET /auth/login?provider=<google|microsoft>`

Initiates the OAuth PKCE flow. Redirects the browser to the provider's authorization page.

### `GET /auth/callback`

OAuth redirect target. Exchanges the authorization code for tokens and creates the account. Redirects to `/` on success.

### `POST /webhook/gmail?secret=<secret>`

Cloud Pub/Sub push endpoint for Gmail notifications. Valid requests are acknowledged immediately; Pebble maps the Gmail `emailAddress` to push-enabled Gmail accounts and triggers the existing Gmail sync pipeline asynchronously.

## Useful Scripts

| Command | Purpose |
| --- | --- |
| `cargo run -p pebble` | Run the backend HTTP server. |
| `pnpm dev:frontend` | Run the Vite frontend dev server (proxies to backend). |
| `pnpm test` | Run frontend tests with Vitest. |
| `pnpm build:frontend` | Type-check and build the frontend to `dist/`. |
| `cargo build --release -p pebble` | Build the release backend binary. |
| `cargo test -p pebble-mail` | Run the mail crate tests. |
| `cargo check` | Check the Rust workspace. |

## Project Structure

```text
Pebble/
|-- src/                    React frontend (SPA)
|   |-- components/         Shared UI components
|   |-- features/           Inbox, compose, search, Kanban, settings
|   |-- hooks/              React hooks and query helpers
|   |-- lib/                HTTP API client, i18n, utilities
|   |-- stores/             Zustand stores
|   `-- tauri-mock.ts       HTTP/SSE bridge (replaces Tauri IPC)
|-- src-tauri/              Rust HTTP backend (Axum)
|   `-- src/
|       |-- main.rs         Server entry point, route registration
|       |-- auth.rs         OAuth login & callback handlers
|       |-- state.rs        Shared application state
|       |-- realtime/       Background sync workers
|       |-- snooze_watcher.rs  Snooze timer background task
|       `-- rpc/            JSON-RPC command handlers
|-- crates/                 Rust workspace crates
|   |-- pebble-core/        Shared types and errors
|   |-- pebble-store/       SQLite persistence
|   |-- pebble-mail/        Mail providers and sync
|   |-- pebble-search/      Tantivy search index
|   |-- pebble-crypto/      Credential encryption
|   |-- pebble-oauth/       OAuth 2.0 and PKCE
|   |-- pebble-rules/       Rules engine
|   |-- pebble-translate/   Translation providers
|   `-- pebble-privacy/     HTML sanitizing and tracker controls
|-- tests/                  Frontend tests
`-- site/                   Static project site and screenshots
```

## Keyboard Shortcuts

| Shortcut | Action |
| --- | --- |
| `J` / `K` | Move through messages |
| `Enter` | Open the selected message |
| `E` | Archive |
| `S` | Toggle star |
| `R` | Reply |
| `A` | Reply all |
| `F` | Forward |
| `C` | Compose |
| `/` | Focus search |
| `Esc` | Close, cancel, or go back |

Shortcuts can be reviewed and customized in Settings.

## Status

Pebble is under active development. It is usable for day-to-day testing, but mail clients handle sensitive data and provider behavior varies. Keep backups of important mail, and verify account actions against your provider when testing new builds.

## Contributing

Issues and pull requests are welcome.

For code changes, please keep patches focused and include tests for behavior changes when practical. Before submitting, run the relevant checks:

```bash
pnpm test
pnpm build:frontend
cargo check
```

## License

Pebble is licensed under the [GNU Affero General Public License v3.0](LICENSE).

---

<p align="center">
  Built by <a href="https://github.com/QingJ01">QingJ</a>.
  <br>
  Friend link: <a href="https://linux.do">LINUX DO</a>
</p>
