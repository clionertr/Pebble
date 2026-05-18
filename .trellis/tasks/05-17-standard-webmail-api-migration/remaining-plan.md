# Remaining Work Plan (grill-me session 2026-05-18)

## Context

Original 7 phases completed: backend REST API, auth middleware, composite endpoints, mutation APIs, compose/drafts/attachments, desktop shell removal, OpenAPI docs.

**What's NOT done yet:**
- Frontend LoginView + auth gating (Phase 2 frontend part)
- Frontend hooks migration from `invoke()` to `api-client.ts` (Phase 3-4 frontend part)
- `/rpc` route deletion + `dispatch.rs` deletion (Phase 7 cleanup)
- CSP headers + deploy config updates (Phase 7 docs)
- README nginx example sync

## Decisions (grill-me confirmed)

| # | Decision | Rationale |
|---|----------|-----------|
| G1 | **Batch migration** by module: core mail → compose/attachments → settings/tools → cleanup | Risk isolation; each batch independently verifiable |
| G2 | **Two APIs coexist** during migration: `/rpc` and `/api` both mounted until all `invoke()` calls gone | Allows incremental migration without breaking app |
| G3 | LoginView startup: **loading spinner** until `/api/auth/status` returns | No flicker; clean UX |
| G4 | Auth state: **React Context** (not Zustand store) | More React-idiomatic for auth gating |
| G5 | CSP `img-src`: **`'self' data: https:`** (compromise) | Allows email images while blocking HTTP plaintext leaks |
| G6 | CSP location: **nginx layer** (`add_header`) | Deployment-configurable; no rebuild needed |
| G7 | Security headers: X-Content-Type-Options, X-Frame-Options, Referrer-Policy — **yes**; HSTS — **no** | HSTS breaks self-hosted setups without HTTPS |
| G8 | `/rpc` deletion: **after ALL frontend migrations complete** | Single cleanup step; no partial dispatch deletion |
| G9 | IP whitelist in nginx-public: **keep** as defense-in-depth | Extra layer beyond cookie auth |

## Execution Order

```
Step 1: LoginView + AuthContext (prerequisite)
  ├── Create src/features/auth/AuthContext.tsx (loading|authenticated|unauthenticated)
  ├── Create src/features/auth/LoginView.tsx (password input → POST /api/auth/login)
  ├── Update Layout.tsx: auth gate (loading→spinner, unauthenticated→LoginView)
  ├── Add "login" to ActiveView type
  └── Gate: pnpm build:frontend + cargo check + all tests green

Step 2a: Core mail hooks migration (~35 methods)
  ├── Shell, health, accounts list
  ├── Messages: list, get, batch, flags, archive, delete, restore, move, empty trash, batch ops
  ├── Folders, threads, search, labels, snooze
  └── Gate: pnpm build:frontend + cargo check + all tests green

Step 2b: Compose + attachments hooks migration (~15 methods)
  ├── Send, drafts, attachments stage/download/list
  ├── Templates, signatures, contacts
  └── Gate: pnpm build:frontend + cargo check + all tests green

Step 2c: Settings + tools hooks migration (~39 methods)
  ├── Rules, kanban, translate, WebDAV, trusted senders
  ├── Diagnostics, preferences, sync, proxy, OAuth, Gmail realtime, pending ops
  ├── AboutTab.tsx version check
  └── Gate: pnpm build:frontend + cargo check + all tests green

Step 3: /rpc deletion + cleanup
  ├── Delete invoke() function + queue logic from sse-client.ts
  ├── Delete server/src/rpc/dispatch.rs
  ├── Remove /rpc and /rpc/batch routes from main.rs
  ├── Clean rpc/ submodules only referenced by dispatch
  └── Gate: cargo test --all + pnpm build:frontend + all 214+ tests green

Step 4: Deploy config + CSP + docs
  ├── deploy/nginx.conf: add /api proxy + CSP + security headers
  ├── deploy/nginx-public.example.conf: same updates
  ├── README.md + README.zh-CN.md: sync nginx examples
  └── Gate: manual review of nginx config syntax
```

## Key Files to Modify (per step)

### Step 1
- `src/features/auth/AuthContext.tsx` — NEW
- `src/features/auth/LoginView.tsx` — NEW
- `src/app/Layout.tsx` — auth gate wrapper
- `src/stores/ui.store.ts` — add "login" to ActiveView

### Step 2a-2c
- `src/lib/api-client.ts` — add ~80 typed REST functions
- `src/lib/api.ts` — replace invoke() calls with api-client calls
- `src/lib/signatures.ts` — replace invoke() calls
- `src/lib/templates.ts` — replace invoke() calls
- `src/features/settings/AboutTab.tsx` — replace invoke() call

### Step 3
- `src/lib/sse-client.ts` — remove invoke(), keep listen()
- `server/src/rpc/dispatch.rs` — DELETE
- `server/src/main.rs` — remove /rpc routes
- `server/src/rpc/mod.rs` — remove dispatch module declaration

### Step 4
- `deploy/nginx.conf` — add /api + CSP + security headers
- `deploy/nginx-public.example.conf` — same
- `README.md` — sync nginx example
- `README.zh-CN.md` — sync nginx example

## Constraints

- All existing tests (214+) must stay green throughout
- Two APIs coexist until Step 3
- Each step's gate: `pnpm build:frontend` + `cargo check` + `cargo test --all` + `pnpm test` all green
- LoginView MUST be done first (auth gate required before API calls work in browser)
