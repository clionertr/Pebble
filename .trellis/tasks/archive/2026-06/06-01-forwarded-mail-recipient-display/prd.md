# Fix forwarded mail recipient display

## Goal

修复邮件详情页收件人展示错误：当邮件是从其他地址转发到当前接收账户时，详情页应展示邮件头中的原始收件人（`message.to_list`），而不是固定展示接收账户邮箱。这样用户能直观看到“这封信原本寄给谁”。

## What I already know

* 用户反馈：转发而来的邮件不会显示原本的收件人，而是显示接收账户作为收件人。
* `src/components/MessageDetail.tsx` 当前通过 `useAccountsQuery()` 找到 `message.account_id` 对应账号，并在发件人旁展示 `to <receivingAccount.email>`。
* 后端/类型契约已经在 `Message` / `MessageSummary` 中携带 `to_list`、`cc_list`、`bcc_list`；IMAP/Gmail/Outlook 同步路径也会尽量从邮件头/Provider API 解析收件人列表。
* 线程气泡 `ThreadMessageBubble` 已使用 `message.to_list` 展示收件人，说明详情页现有逻辑与其他展示点不一致。

## Assumptions

* 本任务只修正前端展示层，不改变邮件同步、数据库 schema 或 REST API。
* 当 `to_list` 为空时，为兼容历史/异常邮件，可回退展示接收账户；但只要邮件数据里有 `to_list`，就必须优先展示原始收件人。
* 多个收件人应以逗号连接；有 display name 时展示 `Name <email>`，没有 display name 时展示邮箱地址。

## Requirements

* 邮件详情页发件人旁的 `to ...` 文案优先来源于 `message.to_list`。
* 转发到当前账户但原始 `To` 是其他地址的邮件，详情页不能再展示当前接收账户作为收件人。
* `to_list` 为空时保留安全 fallback，避免老数据/坏数据详情页完全没有收件人提示。
* 补充前端回归测试，覆盖“`to_list` 与接收账户不同”场景。

## Acceptance Criteria

* [ ] `MessageDetail` 渲染含 `to_list=[Original <original@example.com>]`、接收账号为 `receiver@example.com` 的邮件时，页面显示 `Original <original@example.com>`。
* [ ] 同一场景下页面不再显示 `receiver@example.com` 作为 `to` 收件人。
* [ ] 现有隐私模式、翻译、选中文本操作测试不回退。
* [ ] 相关 lint/typecheck/test 通过，或记录无法通过的原因。

## Definition of Done

* Tests added/updated（前端单元测试）。
* Lint/typecheck/test 按项目可用脚本验证。
* Trellis 任务记录、质量检查和必要的知识沉淀完成。
* 不混入当前工作区已有的无关未提交改动。

## Out of Scope

* 不重写 Gmail/Outlook/IMAP 收件人解析逻辑。
* 不修改数据库迁移、消息 API 响应字段或历史数据修复脚本。
* 不新增完整邮件头展开 UI（如 Cc/Bcc 全量展示）。

## Technical Approach

* 在 `MessageDetail` 内新增/抽出轻量格式化逻辑：把 `EmailAddress[]` 格式化为 `Name <address>` 或 `address`。
* 计算展示收件人：`message.to_list` 非空时使用格式化后的值；否则 fallback 到 `receivingAccount.email`。
* 将渲染条件从 `receivingAccount` 调整为计算出的展示字符串。
* 新增/扩展 `MessageDetail` 测试，模拟 `message.to_list` 与账户邮箱不同，断言 UI 优先显示原始收件人。

## Decision (ADR-lite)

**Context**: `account_id` 表示邮件存在哪个接收账户里，`to_list` 才是邮件头/Provider 返回的收件人语义。详情页把二者混用，会让自动转发或别名收信场景看起来像“邮件原本就是发给接收账户”。

**Decision**: 展示层优先使用 `message.to_list`，仅在没有收件人数据时回退到账户邮箱。

**Consequences**: 变更范围小、风险低；依赖现有同步链路已经正确填充 `to_list`。如果某些 Provider 对转发场景本身不给原始 `To`，那属于后续同步解析问题，不在本任务内扩大范围。

## Technical Notes

* 相关文件：`src/components/MessageDetail.tsx`、`tests/components/MessageDetail.privacy.test.tsx`（或新建组件测试）。
* 参考契约：`src/lib/api-types.ts` 中 `Message.to_list` 与 `EmailAddress`；`crates/pebble-core/src/types.rs` 中 `Message` / `MessageSummary`。
* 现有工作区已有无关未提交改动：`.codex/config.toml`、`.trellis/.template-hashes.json`、`.trellis/.version`、`src/hooks/useMessageLoader.ts`、`src/lib/api-client.ts`、`src/lib/api.ts`、`tests/hooks/useMessageLoader.test.tsx`、`tests/lib/api.privacyMode.test.ts`，本任务不触碰这些文件。
