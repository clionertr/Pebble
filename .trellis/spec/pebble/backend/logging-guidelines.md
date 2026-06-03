# Logging Guidelines

> How logging is done in this project.

---

## Overview

Pebble 后端使用 `tracing` 记录结构化日志。日志的目标是帮助定位同步、OAuth、
推送、搜索和数据库问题，同时避免泄露邮件正文、令牌、Cookie、OAuth code、
代理密码等敏感信息。

---

## Log Levels

- `debug`：开发期诊断信息，例如可选的邮件渲染耗时、内部状态细节。默认生产路径不依赖 debug 日志排障。
- `info`：低频生命周期事件，例如服务启动、后台任务启动、同步完成摘要。
- `warn`：可恢复但需要关注的问题，例如推送设备失效、索引写入失败后降级、外部服务临时失败。
- `error`：请求或后台任务失败且需要人工介入的问题。API 内部错误转换为安全文案时，应在日志记录原始错误。

---

## Structured Logging

优先使用 `tracing` 字段而不是把所有内容拼进字符串：

```rust
tracing::warn!(device_id = %device.id, "Web Push send failed: {error}");
```

常用字段：

- `account_id`、`message_id`、`thread_id`：定位数据对象。
- `device_id`：定位推送设备。
- `provider`：区分 Gmail、Outlook、IMAP。
- `route` 或 `operation`：后台任务或 API 操作名。

字段值应是 ID、枚举、计数、耗时等低敏信息，不放正文、收件人列表、访问令牌。

---

## What to Log

- 外部网络调用失败：OAuth、IMAP、SMTP、Web Push、翻译、WebDAV。
- 关键后台任务异常：同步、Gmail watch renewal、snooze watcher、pending mail ops、重建索引。
- 关键降级路径：删除无效推送设备失败、账户回滚失败、索引状态写入失败。
- 安全相关拒绝：认证失败、限流触发、请求体过大、查询过长。注意不要记录密码或原始 token。

---

## What NOT to Log

不得记录：

- OAuth access token、refresh token、authorization code、state。
- Cookie、session id、VAPID private key、代理密码、SMTP/IMAP 密码。
- 邮件正文、附件内容、完整收件人/抄送列表。
- `.env` 路径中的具体密钥值。

如果排障必须关联用户数据，优先记录内部 ID 和计数。例如记录 `message_id`
和 `attachment_count`，不要记录附件文件名列表和正文片段。
