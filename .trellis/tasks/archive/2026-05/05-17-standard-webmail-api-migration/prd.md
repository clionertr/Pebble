# Standard Webmail API Migration — Final PRD

## Goal

Convert Pebble into a single-user, self-hosted, standard browser Webmail application: React SPA + Rust HTTP API + SSE, with no Tauri desktop shell behavior and no JSON-RPC as the primary frontend/backend protocol.

The work is **test-first and phase-based**. Each phase: write strict tests → run failing tests → implement → green tests → update docs → phase report to user.

---

## Feedback Loop Protocol (per-phase discipline)

每个阶段严格执行以下 7 步循环，不允许跳过或乱序。

### 步骤

| 步 | 名称 | 做什么 | 通过条件 |
|----|------|--------|----------|
| 1 | **WRITE** | 编写该阶段所有测试用例 | 测试文件存在、覆盖正常路径 + 边界 + 错误场景 |
| 2 | **VERIFY_RED** | 运行测试，确认失败 | 新测试因功能未实现而失败（红）— 证明测试有效，不是"假绿" |
| 3 | **IMPLEMENT** | 实现功能代码 | 最小化改动，api/* handler 必须委托现有 rpc/* 函数，不重写业务逻辑 |
| 4 | **VERIFY_GREEN** | 运行测试，确认全部通过 | `cargo test --all` + `npm run test` 全部绿色，0 失败 |
| 5 | **CHECK** | 运行质量门 | `cargo clippy -- -D warnings` 通过，`npm run type-check` 通过，trellis-check 跨层一致性检查通过 |
| 6 | **REPORT** | 向用户总结本阶段 | 用通俗语言说明：做了什么、有什么用、为什么 |
| 7 | **GATE** | 等待用户确认 | 用户说"继续"才能进入下一阶段；有问题则回到相应步骤修正 |

### 测试覆盖铁律

- **后端每个端点** 至少覆盖: 正常 200 响应、认证 401、参数错误 400、资源不存在 404
- **前端每个组件/页面** 至少覆盖: 正常渲染、加载态、空状态、错误态
- **集成路径** 覆盖: HTTP 请求 → handler → store 操作 → 响应 JSON → 状态码

### 阶段报告模板

每个阶段结束后向用户报告，包含以下内容：

1. **做了什么** — 具体改动清单（新增/修改/删除的文件，关键代码片段）
2. **有什么用** — 改动对用户和对系统的价值（用户能做什么了？系统哪里更好了？）
3. **为什么** — 技术决策的理由，与其他方案的权衡
4. **测试覆盖** — 本阶段测试数量、覆盖的场景类型、覆盖率变化
5. **遇到的问题** — 实现中遇到的障碍及解决方法
6. **下一步** — 下一阶段的准备条件和依赖

### 回退规则

若阶段内无法在合理时间内让测试全部变绿：
1. 记录失败的测试和错误信息
2. 回退代码到上一阶段的稳定状态
3. 分析根因
4. 调整方案后重新开始该阶段

**铁律：不允许带着失败测试进入下一阶段。**

### 质量门

| 时机 | 必须通过 |
|------|----------|
| 每步 IMPLEMENT 完成后 | `cargo test --all`、`npm run test` |
| 每阶段 REPORT 前 | `cargo clippy -- -D warnings`、`npm run type-check`、trellis-check 质量检查 |
| 每阶段 GATE 前 | 全部测试 100% 绿色 |

---

## Decisions Log (grill-me session, 2026-05-17)

Every architectural choice below has been confirmed by the project owner.

### Auth & Security
| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Shared secret (password) stored as **bcrypt hash** in `.env` (`PEBBLE_PASSWORD_HASH`) | No plaintext password anywhere; bcrypt is one-way |
| D2 | **Cookie-based session**: `Set-Cookie` with `HttpOnly; Secure; SameSite=Strict; Max-Age=604800 (7d)` | Works with EventSource (SSE can't send custom headers); SameSite=Strict prevents CSRF |
| D3 | Login page as **frontend React component** at `/login`, `POST /api/auth/login` → 302 to inbox or 401 | Consistent with existing React architecture |
| D4 | **Simple rate limiting**: track failed attempts per source IP in memory, 5 failures → lock 15 min | Prevents brute-force in production without Redis dependency |
| D5 | Session lifetime **7 days** | Balance between convenience and security |
| D6 | **No CSRF tokens** — `SameSite=Strict` cookie suffices | CSRF attack surface is minimal for single-user app |
| D7 | CORS origin from **environment variable** `ALLOWED_ORIGIN`, default `""` (same-origin only) | Flexibility for split-hosting while secure by default |
| D8 | `/webhook/gmail`, `/auth/login`, `/auth/callback` **exempt from auth middleware** | Google Pub/Sub pushes to webhook; OAuth flow needs public callback |
| D9 | Gmail webhook retains existing **secret query param** verification (`GMAIL_WEBHOOK_SECRET`) + constant-time comparison | Already implemented; just needs auth middleware exemption |

### API Design
| # | Decision | Rationale |
|---|----------|-----------|
| D10 | **Composite endpoints**: `/api/shell` (accounts+folders+counts) + per-view data endpoints | One request per page load, not N+1 fragments |
| D11 | **No `/api/v1` prefix** | Single-user self-hosted; frontend and backend always deployed together |
| D12 | `OAUTH_REDIRECT_URL` **environment variable** for callback URL | Configurable per deployment; no autodetection magic |
| D13 | **Rust integration tests** (`tests/api/`) for backend endpoints — SQLite `:memory:` + axum `into_make_service()` | Single `cargo test` runs everything; no external process needed |

### Attachments
| # | Decision | Rationale |
|---|----------|-----------|
| D14 | Upload: **traditional FormData + `fetch` POST** (`POST /api/attachments/stage`) | Simple, standard, works for typical email attachment sizes |
| D15 | Download: **preview + download** — images/PDFs preview inline, others trigger browser download via `Content-Disposition: attachment` | Good UX for common attachment types |

### Desktop Cleanup
| # | Decision | Rationale |
|---|----------|-----------|
| D16 | `src-tauri/` → **`server/`** in Phase 1 | New API code starts in the right directory immediately |
| D17 | Mailto protocol handler: **delete entirely** (`useMailtoOpen.ts`, `mailto.ts`, SSE event `app:open-mailto`, RPC `take_pending_mailto_urls`) | Browsers don't support custom protocol registration for web apps |
| D18 | Delete `useCloseToBackground.ts`, `useTrayI18n.ts`, `showMainWindow.ts`, window controls in TitleBar, `keepRunningInBackground`, `setTrayMenuLabels` | All desktop-only concepts with no web equivalent |

### Real-time
| # | Decision | Rationale |
|---|----------|-----------|
| D19 | SSE authentication via **Cookie** (carried automatically by EventSource) | EventSource API does not support custom headers; Cookie is the standard solution |
| D20 | Delete SSE event `app:open-mailto`; keep mail sync/status events; remove `mail:notification-open` until browser Web Push is implemented | Avoids preserving a desktop notification click contract with no active web trigger |

### Deployment
| # | Decision | Rationale |
|---|----------|-----------|
| D21 | Support **both Docker and bare metal** (systemd) | Docker for quick setup; bare metal for lightweight VPS |
| D22 | Frontend `dist/` served by Rust binary in production (single binary deploy) or by CDN/nginx in split deployment | Flexibility without complexity |

### Testing
| # | Decision | Rationale |
|---|----------|-----------|
| D23 | All existing tests must stay **green** throughout every phase | Prevents regression during incremental migration |
| D24 | Desktop-related tests deleted **in Phase 6** (when the code they test is removed), not before | Tests serve as safety net until the feature is actually removed |
| D25 | Frontend test mocks updated **within the same phase** as the API calls they mock | No lingering broken tests across phase boundaries |

---

## RESTful Route Mapping (Final)

### Composite Endpoints (replace former N+1 patterns)

| Endpoint | Returns | Replaces |
|----------|---------|----------|
| `GET /api/shell` | `{ accounts, folders (per-account), unreadCounts (per-account) }` | `list_accounts` + `list_folders` + `get_folder_unread_counts` (all 3 were called together on every page) |
| `GET /api/inbox?accountId=&folderId=&limit=&offset=` | `{ threads (or messages), total, hasMore }` | `list_threads` or `list_messages` (view-level data) |
| `GET /api/starred?accountId=&limit=&offset=` | `{ messages, total, hasMore }` | `list_starred_messages` |
| `GET /api/search?q=&limit=` | `{ hits, total }` | `search_messages` |
| `POST /api/search/advanced` | `{ hits, total }` | `advanced_search` |
| `GET /api/kanban?column=` | `{ cards, notes }` | `list_kanban_cards` + `list_kanban_context_notes` |
| `GET /api/snoozed` | `{ messages, total }` | `list_snoozed` |

### Standard REST Endpoints (1:1 with legacy RPC methods)

#### Accounts
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/accounts` | `list_accounts` |
| POST | `/api/accounts` | `add_account` |
| PATCH | `/api/accounts/:id` | `update_account` |
| DELETE | `/api/accounts/:id` | `delete_account` |
| POST | `/api/accounts/:id/test-connection` | `test_account_connection` |
| POST | `/api/imap/test-connection` | `test_imap_connection` |
| GET | `/api/accounts/:id/proxy` | `get_account_proxy` / `get_oauth_account_proxy` |
| GET | `/api/accounts/:id/proxy-setting` | `get_account_proxy_setting` / `get_oauth_account_proxy_setting` |
| PUT | `/api/accounts/:id/proxy` | `update_account_proxy` / `update_oauth_account_proxy` |
| PUT | `/api/accounts/:id/proxy-setting` | `update_account_proxy_setting` / `update_oauth_account_proxy_setting` |
| POST | `/api/accounts/:id/sync/start` | `start_sync` |
| POST | `/api/accounts/:id/sync/trigger` | `trigger_sync` |
| POST | `/api/accounts/:id/sync/stop` | `stop_sync` |
| POST | `/api/accounts/:id/sync/reindex` | `reindex_search` |
| GET | `/api/accounts/:id/gmail-realtime` | `get_gmail_realtime_config` |
| POST | `/api/accounts/:id/gmail-realtime/enable` | `enable_gmail_realtime` |
| POST | `/api/accounts/:id/gmail-realtime/disable` | `disable_gmail_realtime` |
| PUT | `/api/accounts/:id/gmail-realtime` | `update_gmail_realtime_config` |
| GET | `/api/accounts/:id/signature` | `get_email_signature` |
| PUT | `/api/accounts/:id/signature` | `set_email_signature` |

#### Folders
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/accounts/:id/folders` | `list_folders` |

#### Messages — Reads
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/messages/:id` | `get_message` |
| POST | `/api/messages/batch` | `get_messages_batch` |
| GET | `/api/messages/:id/html?privacyMode=` | `get_rendered_html` |
| GET | `/api/messages/:id/full?privacyMode=` | `get_message_with_html` |
| GET | `/api/threads/:id/messages` | `list_thread_messages` |

#### Messages — Mutations
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| PATCH | `/api/messages/:id/flags` | `update_message_flags` |
| POST | `/api/messages/:id/archive` | `archive_message` |
| DELETE | `/api/messages/:id` | `delete_message` |
| POST | `/api/messages/:id/restore` | `restore_message` |
| POST | `/api/messages/:id/move` | `move_to_folder` |
| DELETE | `/api/accounts/:id/trash` | `empty_trash` |
| POST | `/api/messages/batch/archive` | `batch_archive` |
| POST | `/api/messages/batch/delete` | `batch_delete` |
| POST | `/api/messages/batch/read` | `batch_mark_read` |
| POST | `/api/messages/batch/star` | `batch_star` |

#### Labels
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/labels` | `list_labels` |
| GET | `/api/messages/:id/labels` | `get_message_labels` |
| POST | `/api/messages/batch/labels` | `get_message_labels_batch` |
| POST | `/api/messages/:id/labels` | `add_message_label` |
| DELETE | `/api/messages/:id/labels/:name` | `remove_message_label` |

#### Compose & Drafts
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| POST | `/api/messages/send` | `send_email` |
| POST | `/api/drafts` | `save_draft` |
| DELETE | `/api/drafts/:id?accountId=` | `delete_draft` |

#### Attachments
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/messages/:id/attachments` | `list_attachments` |
| GET | `/api/attachments/:id` | `get_attachment_path` → returns file stream + `Content-Disposition` |
| POST | `/api/attachments/stage` | `stage_compose_attachment` → multipart/form-data upload |

#### Kanban
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| POST | `/api/kanban/cards` | `move_to_kanban` |
| DELETE | `/api/kanban/cards/:messageId` | `remove_from_kanban` |
| PUT | `/api/kanban/notes/:messageId` | `set_kanban_context_note` |
| PATCH | `/api/kanban/notes` | `merge_kanban_context_notes` |

#### Snooze
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| POST | `/api/snoozed` | `snooze_message` |
| DELETE | `/api/snoozed/:messageId` | `unsnooze_message` |

#### Rules
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/rules` | `list_rules` |
| POST | `/api/rules` | `create_rule` |
| PUT | `/api/rules/:id` | `update_rule` |
| DELETE | `/api/rules/:id` | `delete_rule` |

#### Translate
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| POST | `/api/translate` | `translate_text` |
| GET | `/api/translate/config` | `get_translate_config` |
| PUT | `/api/translate/config` | `save_translate_config` |
| POST | `/api/translate/test` | `test_translate_connection` |

#### Contacts
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/contacts?accountId=&q=&limit=` | `search_contacts` |

#### Cloud Sync (WebDAV)
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| POST | `/api/cloud-sync/webdav/test` | `test_webdav_connection` |
| POST | `/api/cloud-sync/webdav/backup` | `backup_to_webdav` |
| POST | `/api/cloud-sync/webdav/preview` | `preview_webdav_backup` |
| POST | `/api/cloud-sync/webdav/restore` | `restore_from_webdav` |

#### Diagnostics & System
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/health` | `health_check` |
| GET | `/api/logs?maxBytes=` | `read_app_log` |
| POST | `/api/diagnostics/mail-timing` | `record_mail_display_timing` |
| GET | `/api/proxy` | `get_global_proxy` |
| PUT | `/api/proxy` | `update_global_proxy` |

#### Preferences
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| PUT | `/api/preferences/realtime` | `set_realtime_preference` |
| PUT | `/api/preferences/notifications` | `set_notifications_enabled` |

#### Trusted Senders
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| POST | `/api/trusted-senders` | `trust_sender` |
| GET | `/api/trusted-senders?accountId=` | `list_trusted_senders` |
| DELETE | `/api/trusted-senders?accountId=&email=` | `remove_trusted_sender` |
| GET | `/api/trusted-senders/check?accountId=&email=` | `is_trusted_sender` |

#### Email Templates
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/templates` | `list_email_templates` |
| POST | `/api/templates` | `save_email_template` |
| DELETE | `/api/templates/:id` | `delete_email_template` |

#### Pending Ops
| Method | Endpoint | Legacy RPC |
|--------|----------|------------|
| GET | `/api/pending-ops/summary?accountId=` | `get_pending_mail_ops_summary` |
| GET | `/api/pending-ops?accountId=&limit=` | `list_pending_mail_ops` |
| POST | `/api/pending-ops/:id/cancel` | `cancel_pending_mail_op` |
| DELETE | `/api/pending-ops/:id` | `delete_pending_mail_op` |

#### Auth (new — no legacy RPC equivalent)
| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/auth/login` | Submit password → returns `Set-Cookie` + 302 to `/` |
| POST | `/api/auth/logout` | Clear cookie → 302 to `/login` |
| GET | `/api/auth/status` | Returns `{ authenticated: true/false }` for frontend routing |

### Deleted (no web equivalent)
| Legacy RPC | Disposition |
|------------|-------------|
| `set_tray_menu_labels` | Desktop tray — removed |
| `complete_oauth_flow` | Already handled by `/auth/login` + `/auth/callback` |
| `download_attachment` (saveTo param) | Merged into `GET /api/attachments/:id` |
| `take_pending_mailto_urls` | Desktop mailto protocol — removed |

---

## SSE Events (Final)

| Event | Phase | Notes |
|-------|-------|-------|
| `mail:new` | Keep | New messages arrived |
| `mail:unsnoozed` | Keep | Snoozed message restored |
| `mail:sync-complete` | Keep | Sync finished |
| `mail:sync-progress` | Keep | Sync progress update |
| `mail:error` | Keep | Sync/mail error |
| `mail:realtime-status` | Keep | Connection mode changes |
| `mail:notification-open` | Deleted | Desktop notification click path has no web trigger yet; future Web Push should define a fresh browser contract |
| `app:open-mailto` | **Deleted in Phase 6** | Desktop mailto protocol — no web equivalent |
| `mail:attachment-download-progress` | **Deleted in Phase 5** | Browser download handles progress natively |

---

## Backend Architecture

### Module Structure

```
server/                          ← renamed from src-tauri/ in Phase 1
├── src/
│   ├── main.rs                  ← axum server, router composition
│   ├── state.rs                 ← AppState (unchanged)
│   ├── auth.rs                  ← OAuth login/callback (unchanged)
│   ├── middleware/
│   │   └── mod.rs               ← NEW: auth middleware (Cookie validation)
│   ├── api/
│   │   ├── mod.rs               ← Router::new().nest("/api", api_routes())
│   │   ├── error.rs             ← ApiError enum → HTTP responses
│   │   ├── shell.rs             ← GET /api/shell (composite)
│   │   ├── auth_api.rs          ← POST /api/auth/login, logout, status
│   │   ├── accounts.rs          ← /api/accounts/*
│   │   ├── folders.rs           ← /api/accounts/:id/folders
│   │   ├── messages.rs          ← /api/messages/* (reads + mutations)
│   │   ├── threads.rs           ← /api/threads/*
│   │   ├── labels.rs            ← /api/labels, /api/messages/:id/labels
│   │   ├── compose.rs           ← /api/messages/send
│   │   ├── drafts.rs            ← /api/drafts
│   │   ├── attachments.rs       ← /api/attachments/*
│   │   ├── kanban.rs            ← /api/kanban/*
│   │   ├── snooze.rs            ← /api/snoozed
│   │   ├── rules.rs             ← /api/rules
│   │   ├── translate.rs         ← /api/translate/*
│   │   ├── contacts.rs          ← /api/contacts
│   │   ├── cloud_sync.rs        ← /api/cloud-sync/*
│   │   ├── trusted_senders.rs   ← /api/trusted-senders/*
│   │   ├── templates.rs         ← /api/templates
│   │   ├── pending_ops.rs       ← /api/pending-ops/*
│   │   ├── diagnostics.rs       ← /api/health, /api/logs, /api/diagnostics/*
│   │   ├── preferences.rs       ← /api/preferences/*
│   │   └── proxy.rs             ← /api/proxy
│   ├── rpc/                     ← KEPT as internal service layer during migration
│   │   ├── ...                  ← existing modules reused by api/ handlers
│   │   └── dispatch.rs          ← deleted in Phase 7
│   ├── events.rs                ← SSE event name constants
│   ├── gmail_realtime.rs        ← Gmail Pub/Sub (unchanged)
│   ├── snooze_watcher.rs        ← (unchanged)
│   └── ...
```

### Key Patterns

**Handler signature** (from old dispatch.rs to new api/ modules):
```rust
// OLD: dispatch.rs manually deserializes each arg from serde_json::Value
"list_messages" => {
    let folder_id: serde_json::Value = args.folder_id; // manual
    crate::rpc::messages::query::list_messages(state, folder_id, ...) .await
}

// NEW: axum extracts Path/Query/Json automatically
async fn list_messages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<Vec<MessageSummary>>, ApiError> {
    let msgs = crate::rpc::messages::query::list_messages(
        state, query.folder_id, query.folder_ids, query.limit, query.offset
    ).await?;
    Ok(Json(msgs))
}
```

**Error shape** (all endpoints return this format):
```json
{ "error": "Human-readable message" }
```
Mapped from `PebbleError`:
- `NotFound` → 404
- `Unauthorized` / `TokenExpired` → 401
- `InvalidInput` / Validation → 400
- Everything else → 500

**Auth middleware**: Applied to `/api/*` and `/events`. Exempts `/auth/*`, `/webhook/*`.

---

## Frontend Architecture

### New Files
```
src/
├── lib/
│   ├── api-client.ts      ← apiGet/apiPost/apiPatch/apiDelete (typed HTTP helpers)
│   └── sse-client.ts       ← renamed from tauri-mock.ts; SSE only (EventSource)
├── features/
│   └── auth/
│       └── LoginView.tsx   ← password input + POST /api/auth/login
```

### Deleted Files (Phase 6)
```
src/tauri-mock.ts                       ← replaced by api-client.ts + sse-client.ts
src/lib/api.ts                          ← merged into api-client.ts
src/lib/showMainWindow.ts               ← desktop concept
src/app/useCloseToBackground.ts         ← desktop concept
src/app/useTrayI18n.ts                  ← desktop concept
src/app/useMailtoOpen.ts                ← desktop concept
src/features/compose/mailto.ts          ← desktop concept
src/lib/ipc-types.ts                    ← types move to api-client.ts
```

### API Client Pattern
```typescript
// src/lib/api-client.ts
const BASE = '/api';

class ApiError extends Error {
  constructor(public status: number, public body: { error: string }) {
    super(body.error);
  }
}

async function apiGet<T>(path: string, params?: Record<string, string>): Promise<T> {
  const url = new URL(path, window.location.origin);
  if (params) Object.entries(params).forEach(([k, v]) => url.searchParams.set(k, v));
  const res = await fetch(url.toString(), { credentials: 'same-origin' });
  if (!res.ok) throw new ApiError(res.status, await res.json());
  return res.json();
}

// Exported typed functions:
export function getShell() { return apiGet<ShellData>('/shell'); }
export function getInbox(accountId: string, folderId: string, limit: number, offset: number) {
  return apiGet<InboxData>('/inbox', { accountId, folderId, limit: String(limit), offset: String(offset) });
}
// ... etc
```

---

## Phased Plan (Final)

### Phase 0: Baseline Inventory And Safety Net (~1 day)

**本阶段反馈循环：**

**Step 1 — WRITE（编写测试）：**
- 编写 `tests/api/baseline.rs`：启动 app 是否 200、SSE 连接是否建立
- 枚举测试：遍历 `dispatch.rs` 所有 RPC 方法，生成 `tests/api/rpc_inventory.rs`（每条方法一条记录）
- 前端基线测试：确保现有 20 个 vitest 测试可运行

**Step 2 — VERIFY_RED：**
- 运行 `cargo test`，确认 baseline 测试因 module 不存在而编译失败（红）
- 运行 `npm run test`，确认现有前端测试状态

**Step 3 — IMPLEMENT（实现）：**
- Create `server/src/api/mod.rs` + `server/src/api/error.rs` skeleton
- Create `server/src/middleware/mod.rs` skeleton
- Enumerate all RPC methods in dispatch.rs + frontend invoke() call sites → write to `tests/api/rpc_inventory.rs`

**Step 4 — VERIFY_GREEN：**
- `cargo test --all` → 所有测试绿色
- `npm run test` → 所有前端测试绿色

**Step 5 — CHECK：**
- `cargo clippy -- -D warnings` 通过
- `npm run type-check` 通过

**Step 6 — REPORT：** 向用户总结本阶段完成内容

**Step 7 — GATE：** 用户确认后进入 Phase 1

### Phase 1: Rename + API Foundation (~2 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests (待编写，每端点至少覆盖 200/401/400/404)：**
- `GET /api/health` → 200 `{ "status": "ok" }`
- `GET /api/health` → 401 无认证时（Phase 2 前暂不启用）
- `GET /api/nonexistent` → 404 `{ "error": "..." }`
- `POST /api/health` → 405 Method Not Allowed
- ApiError 形状契约测试：`NotFound→404`, `Unauthorized→401`, `InvalidInput→400`, `Internal→500`
- 前端 `apiGet`/`apiPost` 单元测试：JSON 解析成功、HTTP 错误抛出 ApiError、401 自动跳转 login

**Work:**
- Rename `src-tauri/` → `server/`. Update all Cargo paths, CI, docs.
- Implement `ApiError` + `From<PebbleError>`
- Mount `/api` Router alongside existing `/rpc`
- Create `src/lib/api-client.ts` with typed helpers
- Keep `/rpc` untouched

**Exit:** `/api/health` works. Error format verified. Directory renamed.

### Phase 2: Authentication (~1.5 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests (待编写，每端点至少覆盖 200/401/400/404)：**
- Unauthenticated `/api/*` → 401
- Authenticated request → 200
- SSE `/events` → 401 without cookie, 200 with cookie
- `/auth/login`, `/auth/callback`, `/webhook/gmail` → accessible without auth
- POST `/api/auth/login` with correct password → Set-Cookie + 200
- POST `/api/auth/login` with wrong password → 401
- 5 wrong attempts → rate limited (429 or 401 with delay)
- Cookie expiration test

**Work:**
- Add bcrypt password hash config loading from `.env`
- Implement auth middleware (extract cookie, verify session)
- Implement `POST /api/auth/login`, `POST /api/auth/logout`, `GET /api/auth/status`
- Simple in-memory rate limiter (IP → failure count + cooldown timestamp)
- Create `src/features/auth/LoginView.tsx`
- Frontend routing: check `GET /api/auth/status` → redirect to `/login` if needed
- Tighten CORS to `ALLOWED_ORIGIN` env var

**Exit:** All `/api/*` and `/events` require auth. Login/logout flow works end-to-end.

### Phase 3: Read APIs + Composite Endpoints (~3 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests（待编写，每端点至少覆盖 200/401/400/404）：**
- `GET /api/shell` → returns accounts + folders + unread counts structure
- `GET /api/inbox` → returns paginated threads/messages
- `GET /api/messages/:id` → returns full message or 404
- `POST /api/messages/batch` → returns array of messages
- `GET /api/messages/:id/html` → returns rendered HTML
- `GET /api/messages/:id/full` → returns combined message+html
- `GET /api/threads/:id/messages` → returns thread messages
- `GET /api/starred`, `GET /api/search`, `POST /api/search/advanced` → correct results
- `GET /api/kanban`, `GET /api/snoozed` → correct results
- All label, rule, translate, template, contact read endpoints → 200 + valid JSON
- Frontend query hooks updated to use new api-client

**Work:**
- Add read endpoints per route mapping table
- Each endpoint is thin wrapper → delegates to existing `crate::rpc::*` functions
- Migrate frontend TanStack Query hooks one-by-one to api-client
- `/rpc` kept alive as compat bridge

**Exit:** All core browsing flows load through `/api`. All read tests pass.

### Phase 4: Mutation APIs (~3 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests（待编写，每端点至少覆盖 200/401/400/404，突变端点额外覆盖 DB 状态变更）：**
- Flags: PATCH → read/star toggled, 404 for invalid id
- Archive/delete/restore/move: correct status, DB reflects change
- Batch: correct counts, partial failure scenarios
- Labels: add → label exists; remove → label gone; batch → correct map
- Kanban: move/list/remove column operations
- Snooze: timing + return-to folder
- Rules: CRUD with JSON validation
- Sync: start/stop/trigger state transitions
- Translate: translate_text, config save, connection test
- Preferences: realtime mode + notifications persistence
- Proxy: global + per-account
- Trusted senders: trust/list/remove/check
- Pending ops: summary, list, cancel, delete
- Diagnostics: log read + mail timing recording
- Frontend mutation hooks updated + optimistic update tests

**Work:**
- Add mutation endpoints
- Replace frontend `invoke()` mutation calls
- `/rpc` kept alive as compat bridge

**Exit:** Core mail actions no longer depend on `/rpc`. All mutation tests pass.

### Phase 5: Compose, Drafts, Attachments (~2 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests（待编写，上传/下载端点的特殊覆盖要求见下）：**
- `POST /api/messages/send` → email sent, attachment references correct
- `POST /api/drafts` → draft saved, updated; idempotency
- `DELETE /api/drafts/:id` → draft removed
- `POST /api/attachments/stage` (multipart) → file stored, returns attachment metadata
- `GET /api/attachments/:id` → Content-Disposition header, correct MIME, stream body
- `GET /api/messages/:id/attachments` → metadata list correct
- Image/PDF preview → inline; other types → download prompt

**Work:**
- Replace byte-array upload (`stage_compose_attachment`) with multipart/form-data
- Replace path-based download (`download_attachment`) with streaming HTTP `Content-Disposition`
- Update `AttachmentList.tsx` to render preview for images/PDFs, `<a download>` for others
- Keep drafts server-side only

**Exit:** Attachments work like standard web app.

### Phase 6: Desktop Shell Removal (~1.5 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests（待编写，重点是"不存在"的验证）：**
- Verify no `invoke(...)` calls remain in production components
- Verify no Tauri mock imports remain
- TitleBar renders without window controls
- Layout renders without tray/background hooks

**Work:**
- `TitleBar.tsx`: remove minimize/maximize/close buttons + `data-tauri-drag-region`
- Delete: `useCloseToBackground.ts`, `useTrayI18n.ts`, `showMainWindow.ts`, `useMailtoOpen.ts`, `mailto.ts`
- Delete RPC: `set_tray_menu_labels`, `take_pending_mailto_urls`
- Delete SSE: `app:open-mailto` (emit side + listener side)
- `sync.store.ts`: remove `keepRunningInBackground`
- `tauri-mock.ts` → rename to `sse-client.ts`, remove `invoke`, `getCurrentWindow`, `getVersion`, `downloadDir`
- `api.ts` → delete; all consumers migrated to `api-client.ts`
- `ipc-types.ts` → merge types into `api-client.ts`
- Delete desktop test files: `useTrayI18n.test.tsx`, `useCloseToBackground.test.tsx`, `useMailtoOpen.test.tsx`
- Update `vite.config.ts`: remove `TAURI_DEV_HOST`

**Exit:** Production frontend has no desktop shell dependency.

### Phase 7: RPC Decommission + OpenAPI (~1.5 days)

遵循反馈循环协议 (WRITE → VERIFY_RED → IMPLEMENT → VERIFY_GREEN → CHECK → REPORT → GATE)

**Tests（待编写，重点是"已删除"的验证）：**
- Negative test: no production code calls `/rpc` or `/rpc/batch`
- `/rpc` and `/rpc/batch` routes return 410 Gone (or are removed)
- OpenAPI spec serves at `/api/docs`
- Frontend build passes with zero `invoke` references

**Work:**
- Remove `/rpc` and `/rpc/batch` from `main.rs`
- Delete `server/src/rpc/dispatch.rs` (the 1500-line match)
- Add `utoipa` annotations to all `/api` handlers
- Mount Swagger UI at `/api/docs`
- Delete `replace.mjs`
- Update README, README.zh-CN, `.env.example`, `deploy/nginx.conf`, `docker-compose.yml`

**Exit:** `/api` is the sole app protocol. OpenAPI docs live. Docs updated.

---

## Documentation Update Plan

Each phase must update **at minimum** these files if their content is affected:

| Document | Phase 1 | Phase 2 | Phase 3-5 | Phase 6 | Phase 7 |
|----------|---------|---------|-----------|---------|---------|
| `README.md` | ✓ (rename paths) | ✓ (auth setup) | — | — | ✓ (API docs link) |
| `README.zh-CN.md` | ✓ | ✓ | — | — | ✓ |
| `.env.example` | — | ✓ (add PASSWORD_HASH etc.) | — | — | ✓ (final env vars) |
| `deploy/nginx.conf` | — | ✓ (auth + CORS) | — | — | ✓ (final config) |
| `docker-compose.yml` | ✓ (rename paths) | ✓ (env vars) | — | — | ✓ (final compose) |
| `Cargo.toml` | ✓ (rename) | — | — | — | ✓ (cleanup) |
| `vite.config.ts` | — | — | — | ✓ (remove Tauri) | — |
| Trellis spec files | ✓ | ✓ | per phase | ✓ | ✓ |

---

## Acceptance Criteria

* [x] All 25 architecture decisions confirmed with project owner
* [x] Strict tests written before implementation in every phase
* [x] Test failures guide implementation (not after-the-fact)
* [x] `/api` covers 100% of app behavior previously served by `/rpc`
* [x] Frontend production code contains zero `invoke(...)` calls
* [x] Attachments: upload via FormData, download via HTTP stream with Content-Disposition
* [x] Auth: all `/api/*` and `/events` require valid session cookie; exempt routes configured
* [x] `src-tauri/` → `server/`; `tauri-mock.ts` → `sse-client.ts`
* [x] Desktop shell code and Tauri naming removed
* [x] CSP header configured
* [x] All documentation updated
* [x] Each phase produces user-facing summary
* [x] `pnpm run build:frontend` passes; `cargo check` + `cargo test` pass
* [x] OpenAPI spec auto-served at `/api/docs`

## Out Of Scope

* Multi-user registration, tenants, shared mailboxes
* Public SaaS deployment model
* Replacing mail sync engine, React, Zustand, TanStack Query, Axum, SQLite, Tantivy
* UI redesign beyond desktop shell removal
* Proc-macro code generation for handlers

## Phase Workload Summary

| Phase | Focus | Est. Days |
|---|---|---|
| 0 | Baseline inventory, contract tests, module skeleton | 1 |
| 1 | `server/` rename, ApiError, `/api` foundation | 2 |
| 2 | Auth middleware, login, rate limiting | 1.5 |
| 3 | 20+ read endpoints + 7 composite endpoints; frontend read hooks | 3 |
| 4 | 40+ mutation endpoints; frontend mutation hooks | 3 |
| 5 | Multipart upload, streaming download, preview, compose/drafts | 2 |
| 6 | Desktop shell removal, file cleanup, SSE event cleanup | 1.5 |
| 7 | RPC decommission, OpenAPI + Swagger, final docs | 1.5 |
| **Total** | | **~15 days** |

## Current Status

Implementation complete. Final quality gates and documentation sync passed on 2026-05-18.
