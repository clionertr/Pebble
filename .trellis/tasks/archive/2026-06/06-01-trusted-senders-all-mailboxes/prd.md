# 修复受信任发件人删除接口和全部邮箱展示

## Goal

修复 `/api/trusted-senders?accountId=...&email=...` 删除受信任发件人时报 “无此方法 / Method Not Allowed” 的问题，并让设置页“受信任的发件人”在“全部邮箱”场景下显示所有账号的信任记录及其所属邮箱。

## What I already know

* 前端 `removeTrustedSender(accountId, email)` 已经发送 `DELETE /api/trusted-senders?accountId=...&email=...`。
* 后端 `/api/trusted-senders` 当前只注册了 `GET` 和 `POST`，没有挂载 `DELETE`，因此删除会命中 405。
* 后端服务层和 store 已经有 `remove_trusted_sender(account_id, email)` 能力，只缺 API handler/route 连接。
* 设置页 `PrivacyTab` 在 `activeAccountId === null` 时会清空受信任发件人；这正对应“全部邮箱”模式，因此看不到跨账号记录。
* `TrustedSender` 已包含 `account_id`，前端可用 `useAccountsQuery()` 的账号列表把它映射成对应邮箱。

## Assumptions

* “全部邮箱”在前端状态里表现为 `activeAccountId === null`。
* `GET /api/trusted-senders` 不带 `accountId` 时可作为“列出全部账号受信任发件人”的 API 语义。
* 删除仍然必须按 `accountId + email` 精确删除；即使多个账号都信任同一个发件人，也只删除用户点击的那一条。

## Requirements

* 后端 `/api/trusted-senders` 支持 `DELETE`，读取 `accountId` 和 `email` 查询参数并删除对应记录。
* 后端 `GET /api/trusted-senders?accountId=<id>` 保持列出单账号记录。
* 后端 `GET /api/trusted-senders` 在缺省 `accountId` 时返回所有账号记录。
* 前端 `listTrustedSenders` 支持 `accountId: string | null`，为 `null` 时省略查询参数。
* 设置页在“全部邮箱”下加载全部信任记录，并显示每条记录对应的账号邮箱。
* 设置页删除按钮使用该条记录自己的 `account_id`，不是全局 active account。

## Acceptance Criteria

* [ ] `DELETE /api/trusted-senders?accountId=...&email=...` 不再返回 405。
* [ ] 单账号设置页仍只显示该账号的受信任发件人。
* [ ] “全部邮箱”设置页显示所有账号的受信任发件人。
* [ ] “全部邮箱”列表中每条受信任发件人能看到对应邮箱。
* [ ] 删除同一发件人在某个账号下的信任记录，不会误删其他账号的同名发件人记录。
* [ ] 相关前端和后端回归测试通过。

## Definition of Done

* 后端 API 路由、handler、store 查询能力完成。
* 前端 API wrapper 与设置页 UI 完成。
* 测试覆盖删除路由、全部账号列表、全部邮箱展示和按记录账号删除。
* 运行针对性测试，必要时说明未运行的全量门禁。

## Out of Scope

* 不改变隐私渲染策略本身。
* 不新增新的信任等级。
* 不调整账号切换和全部邮箱的全局状态模型。

## Technical Notes

* 相关规格：`.trellis/spec/pebble/backend/webmail-api-contracts.md` 已规定取消信任必须按 `accountId + email` 删除。
* 相关数据流：`trusted_senders` 表 → `pebble-store` → `server/src/rpc/trusted_senders.rs` → `server/src/api/resources.rs` → `src/lib/api-client.ts` / `src/lib/api.ts` → `src/features/settings/PrivacyTab.tsx`。
* 跨层边界要保证 `account_id` 不丢失，因为全部邮箱列表需要用它定位所属账号。
