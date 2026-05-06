# Directory Structure

> How backend code is organized in this project.

---

## Overview

Pebble uses a workspace-based Rust architecture. The main application lives in `src-tauri` (now a standalone HTTP server), and specialized logic is moved into separate crates under `crates/`.

- **Main Server**: `src-tauri/src` handles HTTP routing, JSON-RPC dispatch, and application state.
- **Business Logic**: Most core business logic resides in specialized crates under `crates/`.
- **API Endpoints**: Defined in `src-tauri/src/main.rs` (routes) and `src-tauri/src/rpc/` (JSON-RPC handlers).

---

## Directory Layout

```
Pebble/
├── src-tauri/              # Main Rust HTTP backend (Axum)
│   └── src/
│       ├── main.rs         # Entry point, route registration
│       ├── auth.rs         # OAuth login & callback handlers
│       ├── state.rs        # Shared application state
│       ├── realtime/       # Background sync workers
│       ├── snooze_watcher.rs # Snooze timer background task
│       └── rpc/            # JSON-RPC command handlers
└── crates/                 # Rust workspace crates
    ├── pebble-core/        # Shared types and errors
    ├── pebble-store/       # SQLite persistence
    ├── pebble-mail/        # Mail providers and sync
    ├── pebble-search/      # Tantivy search index
    ├── pebble-crypto/      # Credential encryption
    ├── pebble-oauth/       # OAuth 2.0 and PKCE
    ├── pebble-rules/       # Rules engine
    ├── pebble-translate/   # Translation providers
    └── pebble-privacy/     # HTML sanitizing and tracker controls
```

---

## Module Organization

- **Internal Modules**: Keep server-specific logic in `src-tauri/src`.
- **Reusable Logic**: Move any logic that could be reused or needs strict isolation into a new crate in `crates/`.
- **RPC Handlers**: Place new command handlers in `src-tauri/src/rpc/` and register them in the main RPC dispatcher.

---

## Naming Conventions

- **Files/Folders**: Use `snake_case` for all Rust source files and directories.
- **Crates**: Use `pebble-<name>` for crates under `crates/`.

---

## Examples

- **JSON-RPC**: See `src-tauri/src/rpc/` for command implementation patterns.
- **Domain Crate**: See `crates/pebble-mail/` for a large, feature-complete domain module.
