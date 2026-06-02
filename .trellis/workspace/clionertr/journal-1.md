# Journal - clionertr (Part 1)

> AI development session journal
> Started: 2026-05-04

---



## Session 1: 修复 1Panel 部署下的 502 错误

**Date**: 2026-05-04
**Task**: 修复 1Panel 部署下的 502 错误
**Package**: pebble
**Branch**: `master`

### Summary

调查并解决了 1Panel 环境下 Docker Compose 启动后出现的 502 Bad Gateway 错误。根本原因是后端服务硬编码监听 127.0.0.1。已修改代码支持 PEBBLE_HOST 环境变量，并更新了 1Panel 的 .env 配置为 0.0.0.0。同时完善了后端配置规范文档。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `d910242` | (see git log) |
| `7ec7c7b` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 2: Mobile Web Adaptation

**Date**: 2026-05-04
**Task**: Mobile Web Adaptation
**Package**: pebble
**Branch**: `master`

### Summary

Adapted the React app and landing page for mobile devices using responsive design and stack-based navigation.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `c06dc63` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 3: Fix Build Error

**Date**: 2026-05-05
**Task**: Fix Build Error
**Package**: pebble
**Branch**: `master`

### Summary

Fixed a syntax error in SettingsView.tsx that was causing the build to fail.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `c6a9b0b` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 4: Optimize Docker Build Caching

**Date**: 2026-05-06
**Task**: Optimize Docker Build Caching
**Package**: pebble
**Branch**: `001-improve-experience`

### Summary

Implemented cargo-chef and BuildKit cache mounts in Dockerfiles. Updated GitHub Actions for multi-arch builds, GHA caching, and GHCR.io pushing. Updated backend quality specs.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `78b512d` | (see git log) |
| `2e17041` | (see git log) |
| `e890d9f` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 5: Gmail Pub/Sub realtime sync

**Date**: 2026-05-14
**Task**: Gmail Pub/Sub realtime sync
**Package**: pebble
**Branch**: `001-improve-experience`

### Summary

Implemented per-account Gmail API Pub/Sub realtime sync with configurable fallback polling, fixed OAuth add-account login to use /auth/login, updated tests/docs, and recorded retired endpoint call guidance.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `bf7621c` | (see git log) |
| `82acafb` | (see git log) |
| `b8af4b9` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 6: Pebble 性能与UX全面优化 — P0/P1/P2 15个任务全部完成

**Date**: 2026-05-16
**Task**: Pebble 性能与UX全面优化 — P0/P1/P2 15个任务全部完成
**Package**: pebble
**Branch**: `001-improve-experience`

### Summary

完成 05-15-improve-ux-mail-latency-performance 全部 15 个子任务:
P0: 异步阻塞修复, 推送绕过断路器, SSE重连+丢弃监控, 闪屏移除, A11y修复
P1: 缓存精准失效, 数据库复合索引, Tantivy双存消除, Zustand拆分, IMAP连接池
P2: RPC超时+并发, 线程虚拟化, N+1 JOIN优化, 搜索框移除, 延迟持久化

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `5f9a8c9` | (see git log) |
| `25fc972` | (see git log) |
| `08ef8c0` | (see git log) |
| `1dbf154` | (see git log) |
| `3d968e2` | (see git log) |
| `7753af4` | (see git log) |
| `98aa840` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 7: Standard Webmail API Migration — 从桌面 JSON-RPC 到 REST Webmail

**Date**: 2026-05-17 to 2026-05-18
**Task**: 05-17-standard-webmail-api-migration
**Package**: pebble
**Branch**: `001-improve-experience`

### Summary

将 Pebble 从 Tauri 桌面 JSON-RPC 架构彻底改造为标准单用户自托管 Webmail：React SPA + Rust REST API + SSE + Cookie 认证。26 commits，覆盖 PRD 全部 8 个阶段（Phase 0-7），80+ REST 端点，25 个架构决策全部兑现。

### Main Changes

**Phase 0-2: 基础设施**
- `server/` 目录重命名（原 `src-tauri/`）
- ApiError 错误类型 + From<PebbleError> 自动映射
- Axum 路由挂载 `/api` 与 `/events`
- Cookie 认证中间件 + bcrypt 密码验证 + 频率限制
- LoginView + AuthContext 前端认证门控

**Phase 3-4: 核心 API**
- `/api/shell` 复合端点（accounts + folders + unread counts）
- 20+ 读取端点 + 40+ 突变端点
- 前端 TanStack Query hooks 全部迁移至 REST api-client
- 零 `invoke(` 调用残留

**Phase 5-7: 附件/桌面移除/RPC 退役**
- 附件上传从字节数组改为 multipart/form-data
- 附件下载改为 HTTP 流 + Content-Disposition
- 桌面 shell 代码完全移除（TitleBar 窗口控件、托盘/后台钩子、mailto 协议）
- `/rpc` 和 `/rpc/batch` 路由退役
- OpenAPI 3.0 文档 + `/api/docs` 端点
- 库存/拒绝测试（api_rpc_inventory.rs）作为迁移安全网

**Bug Fix & 打磨**
- CSP 安全头 + nginx 部署配置
- SSE 重连 + 消息丢弃监控
- API 请求减少（精准缓存失效 + SSE 感知轮询）
- 路由参数修正（`{param}` → `:param`）
- Docker 多架构构建缓存
- 双语 README 同步

### Git Commits

(26 total)

| Hash | Message |
|------|---------|
| `80d23e1` | docs: sync webmail deployment guidance |
| `c8cec9b` | chore: remove desktop packaging path |
| `5d4d5c4` | refactor(frontend): use REST and SSE clients |
| `d081c9b` | fix(server): harden webmail API contract |
| `2d80949` | fix: limit focus/blur sync + Docker cache-busting |
| `a7c1159` | fix: reduce excessive frontend API requests |
| `37f6f1d` | docs: deploy/docker-compose.yml + .env.example |
| `87fe402` | docs: comprehensive bilingual README |
| `10b2046` | fix: parse comma-separated folderIds |
| `cd8a469` | fix: replace {param} with :param in Axum routes |
| `d82e4b4` | chore: OpenAPI docs + docker-compose |
| `f23d16b` | fix: missing REST endpoints (kanban mutations, etc.) |
| `8de195e` | feat: invoke() removal + /rpc decommission |
| `6fbf05c` | fix: /api/proxy GET/PUT endpoints |
| `9af1941` | feat: CSP + security headers + deploy config |
| `c13fe7f` | feat: 15 backend REST endpoints |
| `8e8cbc4` | feat: migrate settings/tools API to REST |
| `516eeb4` | feat: migrate compose/attachments/drafts/contacts API |
| `ff596dc` | feat: migrate core mail API from invoke() to REST |
| `90affbd` | feat: LoginView + AuthContext frontend auth gating |
| `464dd81` | feat: Phase 7 OpenAPI docs at /api/docs |
| `f34348f` | refactor: Phase 6 remove desktop shell (Tauri) code |
| `8b221b1` | feat: Phase 5 compose, drafts, and attachments |
| `d9afef9` | feat: Phase 4 mutation APIs across 40+ endpoints |
| `74313c2` | feat: Phase 3 read APIs + /api/shell composite |
| `f2c8da0` | refactor: Phase 0-2 Webmail API migration foundation |

### Testing

- [OK] `pnpm test` — 67 frontend vitest files
- [OK] `cargo test --all` — 291 crate unit tests + 7 server integration tests + inventory/rejection gate
- [OK] `cargo clippy --all-targets -- -D warnings`
- [OK] `pnpm run build:frontend`

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 8: 同步上游 Webmail 安全补丁

**Date**: 2026-05-19
**Task**: 同步上游 Webmail 安全补丁
**Branch**: `master`

### Summary

选择性同步上游共享 crates 和少量前端修复：隐私渲染纯文本 fallback、搜索 stored snippet 与后台重建、WebDAV 备份校验、ProviderPush 熔断保护、Sidebar 草稿确认和 Compose 无障碍属性；保留 Webmail REST/SSE、移动端和多账号能力。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `7f209bc` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 9: Browser Push Notifications implementation + source-run DX improvements

**Date**: 2026-05-23
**Task**: Browser Push Notifications implementation + source-run DX improvements
**Branch**: `master`

### Summary

Implemented Web Push browser notifications (backend push service, notification device storage, REST API, Service Worker, frontend subscription lifecycle, settings UI), ran full quality gate (cargo fmt/clippy/test, pnpm test/build), fixed migration test version assertions, updated README with dev-vs-production sections and LockBusy troubleshooting, added auto .env loading and actionable startup errors, created systemd service template. User confirmed end-to-end browser test passed.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `86c7888` | (see git log) |
| `e9803c3` | (see git log) |
| `d00359c` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 10: Webmail shell metadata cache

**Date**: 2026-05-30
**Task**: Webmail shell metadata cache
**Branch**: `master`

### Summary

扩展 /api/shell 返回账号、文件夹、未读数和 Gmail realtime 配置；前端用 shell hydrate React Query 元数据缓存，减少 accounts/folders/gmail-realtime N+1 请求；网络恢复时 wake sync 并刷新必要缓存；修复周期性 poll 完成误触发 shell/inbox 重拉的问题，改为由 mail:new、pending ops、网络恢复等实际变化事件驱动。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `8dac36f` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 11: Fix forwarded mail recipient display

**Date**: 2026-06-01
**Task**: Fix forwarded mail recipient display
**Branch**: `master`

### Summary

邮件详情页优先展示 message.to_list，修复转发邮件把接收账户误当收件人的问题。补充回归测试和 spec 契约文档。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `ffbdb92` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 12: 修复受信任发件人 DELETE 路由与全部邮箱聚合 + 补全 Snooze API 路由

**Date**: 2026-06-01
**Task**: 修复受信任发件人 DELETE 路由与全部邮箱聚合 + 补全 Snooze API 路由
**Branch**: `master`

### Summary

后端 trusted-senders 和 snooze API 均缺失写操作路由，前端按钮静默失败。补上 trusted-senders DELETE/全量查询、snooze POST/DELETE，修复 PrivacyTab 全部邮箱视图，补 Rust API 集成测试与规格文档。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `cbd4c04` | (see git log) |
| `8a0ab6e` | (see git log) |
| `1c7593b` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 13: 第1阶段安全与稳定性止血实现

**Date**: 2026-06-02
**Task**: 第1阶段安全与稳定性止血实现
**Branch**: `master`

### Summary

实现第1阶段全部11个整改项：前端依赖漏洞修复、OAuth XSS防护(escape_html)、附件上传25MB上限、查询limit clamp 500、搜索长度限制500字符、消除12处HTTP可达.unwrap()、关键let _改warn!、版本统一为0.0.10/clionertr/Pebble、.dockerignore补齐、nginx real_ip收紧、ApiError脱敏。质量门全部通过。更新error-handling.md和quality-guidelines.md规范。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `5ebbe89` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete
