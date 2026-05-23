# Browser Push Notifications

## Goal

允许用户在当前设备上开启浏览器通知。开启后，即使 Pebble 页面关闭，只要后端检测到新邮件到达，也能通过浏览器 Web Push 触发系统通知。第一版重点服务桌面浏览器，兼顾自托管部署的安全、隐私和可维护性。

## Requirements

- 使用 Web Push + Service Worker 实现页面关闭后的浏览器通知；生产环境要求 HTTPS 或浏览器认可的安全上下文。
- 通知设置按当前设备生效，不做全局“所有设备一起开关”。
- 新设备默认关闭通知；只有用户点击通知开关时才请求浏览器权限。
- 旧版本地 `notificationsEnabled=true` 偏好升级后允许自动尝试恢复：如果浏览器权限已经是 `granted`，自动重新登记订阅；否则保持关闭并等待用户手动点击启用。
- Manual only 手动同步模式下禁止开启通知，用户必须先选择 Realtime、Balanced 或 Battery saver。
- 第一版通知覆盖所有已连接邮箱账号。
- 第一版默认只通知规则处理后最终仍在收件箱的未读新邮件；后续预留按账号、文件夹或标签配置通知范围。
- 如果新邮件被自动规则立即归档或移动出收件箱，不发送通知。
- Pebble 页面处于前台时不弹普通新邮件系统通知；后台标签页、后台窗口或页面关闭时才弹。首次启用摘要和测试通知可在前台弹。
- 普通单封邮件通知标题显示主题，正文显示发件人和收件账号。
- 验证码邮件使用内置启发式识别，标题显示“验证码 + 主题”，正文显示发件人和收件账号。
- 验证码邮件绕过合并窗口，识别后立即通知。
- 普通邮件使用 5 秒合并窗口；单封普通邮件也允许最多等待 5 秒再通知。
- 多封普通邮件合并通知标题显示数量，正文最多列出 3 个最近发件人。
- 首次开启通知时，如果已有收件箱未读邮件，只补发一条未读摘要，不逐封补发。
- 通知已开启后新增邮箱账号，初次同步发现旧未读邮件时只发摘要，不逐封通知。
- 单封通知点击后打开 Pebble 并定位到对应邮件，同时自动标为已读。
- 合并通知点击后打开收件箱，不选中特定邮件，也不自动标记任何邮件为已读。
- 点击通知时如果 Pebble 已打开且正在写邮件，必须遵守现有草稿/离开确认保护，不直接丢弃写信内容。
- 第一版不提供通知操作按钮，例如“标为已读”“归档”。
- 显式退出登录时自动关闭当前设备通知并取消当前设备订阅。
- 登录会话自然过期后，当前设备通知暂停；服务端重启导致会话失效时也暂停。
- 重新登录后，如果当前设备此前开启过通知且浏览器权限仍允许，自动重新登记订阅并恢复通知。
- 提供简单通知设备列表：显示已登记设备、状态、最后活跃时间，允许远程移除。
- 设备名称自动生成，并允许用户改名。
- 会话过期后暂停的设备保留在列表中并标记“已暂停/需重新登录”。
- 第一版提供“发送测试通知”按钮，用于验证当前设备权限和推送链路。
- VAPID 密钥支持环境变量配置；如果未配置，Pebble 自动生成并保存。
- 推送离线保留时间：普通邮件最多 24 小时，验证码邮件最多 15 分钟。
- 接受 Web Push 通过浏览器厂商推送服务中转：内容加密，但推送时间、订阅端点等元数据可能经过浏览器厂商服务。

## Acceptance Criteria

- [ ] 在支持 Web Push 的桌面浏览器中，用户可在 Settings → General 中为当前设备开启通知，浏览器权限请求只在点击开关时出现。
- [ ] Manual only 模式下启用通知被阻止，并给出清晰提示。
- [ ] 当前设备成功开启后，服务端保存 Push Subscription 和设备记录；设备列表可看到该设备并支持改名、移除。
- [ ] 页面关闭后，符合条件的新收件箱未读邮件会触发浏览器通知。
- [ ] Pebble 页面处于前台时，普通新邮件不弹系统通知，但页面数据仍刷新。
- [ ] 普通新邮件在 5 秒窗口内按规则合并；验证码邮件立即通知。
- [ ] 单封通知点击后打开对应邮件并标为已读；合并通知点击后打开收件箱。
- [ ] 首次开启通知和新增账号初次同步的历史未读只发送摘要，不逐封通知。
- [ ] 退出登录会取消当前设备订阅；会话过期或服务端重启后设备显示暂停，重新登录后可自动恢复。
- [ ] 测试通知按钮能向当前设备发送测试通知并反馈成功/失败。
- [ ] VAPID 环境变量可用；未配置时自动生成并持久化。
- [ ] 前端类型检查、后端编译、相关单元/组件测试通过。

## Definition of Done

- Tests added or updated for frontend notification settings, Service Worker click behavior contracts where feasible, backend subscription storage, notification filtering, batching, OTP extraction, and API validation.
- Frontend build/typecheck passes via `pnpm build:frontend`.
- Frontend tests pass for touched behavior via `pnpm test` or targeted Vitest commands.
- Rust tests/compile checks pass for touched crates via `cargo test` or targeted package tests where runtime cost requires narrowing.
- User-visible strings are localized in English and Chinese.
- Deployment/security notes are updated if Web Push requires new environment variables, HTTPS assumptions, or reverse-proxy/static asset handling.

## Technical Approach

- Add a Service Worker and frontend registration layer for Push Subscription lifecycle.
- Add REST API endpoints for VAPID public key, current-device subscription upsert/delete, notification device list/update/delete, and test notification.
- Persist notification devices/subscriptions in SQLite with fields for endpoint identity, keys, generated device name, custom name, status, session binding/expiry, last active time, and created/updated times.
- Reuse the existing `mail:new` backend path as the source of newly stored messages, but move notification eligibility to after rule processing so final folder membership is used.
- Use the existing `StoredMessage.notify` flag to avoid notifying for historical sync/refresh events; add explicit summary paths for first enable and new-account initial unread summaries.
- Add OTP extraction as a small deterministic helper using keyword-gated patterns against subject/snippet/body text.
- Add backend batching for ordinary notifications with a 5 second account-independent buffer per active device; OTP notifications bypass batching.
- Include notification click payload data for `messageId` or inbox summary destination. Add frontend deep-link handling so `/?messageId=<id>` opens the message and marks it read after authentication.

## Decision (ADR-lite)

**Context**: A browser tab cannot receive SSE after it is closed. Supporting notifications while the page is closed requires a browser-supported background delivery mechanism.

**Decision**: Use standard Web Push with Service Worker, VAPID keys, backend subscription storage, and device-scoped user settings.

**Consequences**: This provides page-closed notifications in supported desktop browsers, but requires HTTPS/security context, introduces VAPID key management, and routes encrypted push payloads through browser vendor push services. iOS Safari and mobile behavior are not first-class in this MVP.

## Research References

- [`research/web-push-implementation.md`](research/web-push-implementation.md) — Web Push requires a Service Worker, user-granted notification permission, encrypted push payloads, and VAPID sender identity; use Rust `web-push` crate instead of hand-rolling protocol details.

## Out of Scope

- iOS Safari / installed PWA first-class support.
- Per-account, per-folder, or per-label notification rule UI in the first version.
- Notification action buttons such as archive, mark read, snooze, or reply.
- Full privacy-level selector in the first version; detailed notifications are the only first-version content mode.
- Native desktop app or OS-level background daemon.
- Guaranteed hard real-time delivery; delivery remains subject to mail provider push/poll behavior, network, browser, OS notification policy, and device sleep.

## Technical Notes

- Existing frontend SSE client: `src/lib/sse-client.ts`.
- Existing new mail event listener/data refresh: `src/components/StatusBar.tsx`.
- Existing notification preference placeholder: `src/stores/sync.store.ts`, `src/features/settings/GeneralTab.tsx`, `src/app/useRealtimePreferenceSync.ts`.
- Existing backend new mail emission: `server/src/rpc/indexing.rs`.
- Existing `StoredMessage.notify` flag: `crates/pebble-mail/src/sync.rs`, `crates/pebble-mail/src/gmail_sync.rs`, `crates/pebble-mail/src/outlook_sync.rs`.
- Existing rule engine is applied in `server/src/rpc/indexing.rs` after `mail:new` is currently emitted; notification eligibility must account for rule-processed final folders.
- Existing single-user session model: `server/src/session.rs`, `src/features/auth/AuthContext.tsx`.
- Existing deep-open helper: `useUIStore.openMessageInInbox(messageId)` in `src/stores/ui.store.ts`.
- Existing sync preference modes: `src/stores/sync.store.ts` and `server/src/rpc/sync_cmd.rs`.
