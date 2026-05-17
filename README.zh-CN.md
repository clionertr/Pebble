<p align="center">
  <img src="src/assets/app-icon.png" alt="Pebble logo" width="120">
</p>

<h1 align="center">Pebble</h1>

<p align="center">
  一个本地优先的邮件客户端，让收件箱更安静、更私密。现已支持作为自托管 Web 服务运行。
</p>

<p align="center">
  <a href="README.md">English</a>
  ·
  <a href="https://github.com/QingJ01/Pebble/releases">发布版本</a>
  ·
  <a href="LICENSE">许可证</a>
</p>

<p align="center">
  <a href="https://github.com/QingJ01/Pebble/releases"><img src="https://img.shields.io/github/v/release/QingJ01/Pebble?style=flat-square&color=d4714e" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square" alt="License"></a>
  <a href="https://github.com/QingJ01/Pebble/actions"><img src="https://img.shields.io/github/actions/workflow/status/QingJ01/Pebble/ci.yml?style=flat-square&label=build" alt="Build"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20VPS%20%7C%20Self--hosted-lightgrey?style=flat-square" alt="Platform">
</p>

## 项目简介

Pebble 是一个使用 Rust 和 React 构建的自托管邮件客户端。它已经从 Tauri 桌面应用重构为**Web 服务**：Rust 后端作为独立的 HTTP 服务器运行，React 前端作为标准 Web 应用提供服务，并通过 HTTP 连接到后端。

所有的邮件数据、搜索索引、附件、规则和应用设置都保留在你的服务器端。

Pebble 的设计目标很直接：

- 邮箱应该清晰、快速、安静。
- 邮件工作流应该本地优先，而不是被云端仪表盘绑住。
- 隐私控制应该明确可见，并且可以按单封邮件临时放宽。
- 搜索、稍后提醒、规则和看板应该协同工作，而不是散落在不同工具里。

Pebble 目前支持 Gmail、IMAP，以及实验性的 Outlook 账户。

## 架构说明

当前分支将原有的 Tauri 桌面外壳替换为客户端-服务器（C/S）架构：

```
浏览器 (React SPA)
        │  HTTP fetch  /rpc/batch
        │  SSE stream  /events
        │  OAuth 流程   /auth/login  /auth/callback
        ▼
Rust HTTP 服务器  (Axum, 端口 3000)
        │
        ├── pebble-store    SQLite 数据库
        ├── pebble-search   Tantivy 全文搜索索引
        ├── pebble-mail     IMAP / Gmail / Outlook 同步
        ├── pebble-crypto   凭据加密
        ├── pebble-oauth    OAuth 2.0 + PKCE
        ├── pebble-rules    规则引擎
        ├── pebble-translate 翻译提供商
        └── pebble-privacy  HTML 清理与追踪保护
```

### 与上游（Upstream）的主要区别

| 上游 (Tauri 桌面端) | 当前分支 (Web 服务) |
| --- | --- |
| Tauri IPC (`invoke`) | 基于 HTTP 的 JSON-RPC，路径为 `POST /rpc/batch` |
| Tauri 事件系统 | Server-Sent Events (SSE)，路径为 `GET /events` |
| 桌面端 OAuth 重定向 | HTTP OAuth 流程，路径为 `/auth/login` 和 `/auth/callback` |
| 应用数据在操作系统用户目录 | 本地 `./data/` 目录（适合 VPS 部署） |
| 平台原生的系统密钥链 | 基于文件的密钥 `./data/pebble.key` |

## 主要特性

### 本地优先与隐私

- 使用本地 SQLite 数据库存储邮件、文件夹、标签、规则和设置。
- 使用本地 Tantivy 全文索引提供快速搜索。
- 附件保存在磁盘的 `./data/attachments/` 目录下。
- OAuth token 和账号凭据使用服务器本地密钥文件加密。
- 不包含遥测。
- 网络请求只发生在你启用的功能中：邮件同步、翻译、可选的 WebDAV 设置备份。

### 邮件处理

- 多账户聚合收件箱。
- 支持 Gmail、IMAP 和实验性的 Outlook。
- 支持线程视图和普通邮件列表视图。
- 支持归档、删除、星标、标记已读、批量操作和恢复。
- 支持邮件稍后提醒（Snooze）。
- 支持全文搜索和高级过滤。
- 支持规则引擎，自动整理邮件。

### 效率工具

- 看板视图，包含 Todo、Waiting、Done 三列。
- 命令面板和键盘优先导航。
- 内置翻译能力，支持双语阅读。
- 深色和浅色主题。
- 内置英文和中文界面。
- 可选的 WebDAV 备份，用于同步设置、规则、看板卡片和看板备注。

## 截图

<table>
  <tr>
    <td><img src="site/screenshots/inbox.png" alt="收件箱"><br><b>收件箱</b></td>
    <td><img src="site/screenshots/kanban.png" alt="看板"><br><b>看板</b></td>
  </tr>
  <tr>
    <td><img src="site/screenshots/dark.png" alt="深色模式"><br><b>深色模式</b></td>
    <td><img src="site/screenshots/settings.png" alt="设置"><br><b>设置</b></td>
  </tr>
</table>

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 后端服务器 | Rust + Axum |
| 传输协议 | JSON-RPC over HTTP, SSE 用于推送事件 |
| 前端 | React 19、TypeScript |
| 状态管理 | Zustand、TanStack Query |
| 数据库 | SQLite / rusqlite |
| 搜索 | Tantivy |
| 样式 | Tailwind CSS |
| 国际化 | i18next |

## 开始使用

### 环境要求

- Rust stable
- Node.js 18 或更新版本
- pnpm 8 或更新版本

### 开发环境

```bash
git clone https://github.com/QingJ01/Pebble.git
cd Pebble

pnpm install
cp .env.example .env
# 在 .env 中填写你的 OAuth 凭据
```

启动后端服务器（终端 1）：

```bash
cargo run -p pebble
```

启动前端开发服务器（终端 2）：

```bash
pnpm dev:frontend
```

在浏览器中打开 `http://localhost:1420`。Vite 开发服务器会自动将 `/rpc`、`/events`、`/auth` 和 `/webhook` 的请求代理到 3000 端口的后端。

### 生产环境部署

构建前端：

```bash
pnpm build:frontend
```

静态文件会输出到 `dist/` 目录。你可以使用任何 Web 服务器（nginx、caddy 等）来托管这些文件，并将 `/rpc`、`/events`、`/auth` 和 `/webhook` 路径代理到 Rust 后端。

构建并运行后端：

```bash
cargo build --release -p pebble
./target/release/pebble
```

数据存放在工作目录下的 `./data/` 目录中。请让后端在一个持久化的目录下运行，并妥善保管 `./data/pebble.key` —— 如果丢失该文件，你将失去对已存凭据的访问权限。

Nginx 配置示例（假设后端在 3000 端口，前端从 `dist/` 托管）：

```nginx
server {
    listen 443 ssl;
    server_name mail.example.com;

    root /path/to/Pebble/dist;
    index index.html;

    # 前端 SPA —— 回退到 index.html 以支持客户端路由
    location / {
        try_files $uri $uri/ /index.html;
    }

    # 后端 API、SSE、OAuth 和 Gmail Pub/Sub webhook
    location ~ ^/(rpc|events|auth|webhook) {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;

        # SSE 连接所需的配置
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 3600s;
    }
}
```

## OAuth 配置

Pebble 可以通过 OAuth 连接 Gmail 和 Outlook。IMAP 账户使用应用内配置的 IMAP/SMTP 凭据。

复制 `.env.example` 为 `.env`，然后填写你需要的提供商配置。这些环境变量必须**在编译时**设置，以便它们能够嵌入到发布的二进制文件中。

| 变量 | 说明 |
| --- | --- |
| `GOOGLE_CLIENT_ID` | Google OAuth 客户端 ID。请使用 Web 应用客户端（Web application），并添加 `http://localhost:3000/auth/callback` 为已授权的重定向 URI。 |
| `GOOGLE_CLIENT_SECRET` | 必填（Web 应用客户端需要）。 |
| `MICROSOFT_CLIENT_ID` | Microsoft public/native app 客户端 ID。 |
| `MICROSOFT_CLIENT_SECRET` | 可选。public/native Microsoft 应用通常应留空。 |

> **注意**：由于 OAuth 回调现在由 HTTP 服务器在 `/auth/callback` 处理，你必须在你的 Google/Microsoft 应用设置中配置 `http://<your-host>/auth/callback`（本地开发为 `http://localhost:3000/auth/callback`）作为已授权的重定向 URI。

## Gmail 实时推送

Gmail 账户可以选择使用 Gmail API `watch`，通过 Google Cloud Pub/Sub 接收变更通知。该功能在 **Settings -> Accounts -> Enable realtime Gmail** 中按账号启用；普通 Gmail OAuth 登录和 IMAP 账号不依赖这项配置。

运行时环境变量：

| 变量 | 说明 |
| --- | --- |
| `GMAIL_PUBSUB_TOPIC` | 完整 Pub/Sub Topic 名称，例如 `projects/<project-id>/topics/gmail-webmail-topic`。 |
| `GMAIL_WEBHOOK_SECRET` | Pub/Sub 推送 URL query string 中需要携带的共享密钥。不要提交到代码仓库。 |

Google Cloud 设置：

1. 启用 Gmail API 和 Cloud Pub/Sub API。
2. 创建一个 Pub/Sub Topic。
3. 将该 Topic 的 `roles/pubsub.publisher` 权限授予 `gmail-api-push@system.gserviceaccount.com`。
4. 创建 Push Subscription，推送地址设为 `https://<your-host>/webhook/gmail?secret=<your-secret>`。
5. 在反向代理中将 `/webhook/gmail` 转发到 Pebble 后端。

Pebble 会在启动时以及之后每 12 小时检查已启用的 Gmail watch，并续期缺少过期时间或 24 小时内过期的 watch。当前 MVP 不包含 Pub/Sub OIDC JWT 校验；现在使用 URL secret，若要生产强化，应后续添加 authenticated push 校验。

## API 参考

后端暴露了三组服务端点：

### `POST /rpc`

单个 JSON-RPC 调用。请求体：`{ "method": "<command>", "params": { ... } }`。直接返回结果，或在失败时返回 `{ "error": "<message>" }`。

### `POST /rpc/batch`

按顺序处理的 JSON-RPC 调用数组。请求体：`[{ "method": "...", "params": {...} }, ...]`。返回对应结果的数组。

### `GET /events`

Server-Sent Events 流。前端连接到此端点接收有关新邮件、同步状态、稍后提醒唤醒和其他实时更新的推送通知。每个事件都有一个命名的 type 和一个 JSON payload。

### `GET /auth/login?provider=<google|microsoft>`

发起 OAuth PKCE 流程。将浏览器重定向到提供商的授权页面。

### `GET /auth/callback`

OAuth 重定向目标。用授权码交换 token 并创建账户。成功后重定向到 `/`。

### `POST /webhook/gmail?secret=<secret>`

Cloud Pub/Sub 的 Gmail 通知推送端点。合法请求会被立即确认；Pebble 会将 Gmail `emailAddress` 映射到已启用推送的 Gmail 账号，并异步触发现有 Gmail 同步流水线。

## 常用命令

| 命令 | 用途 |
| --- | --- |
| `cargo run -p pebble` | 运行后端 HTTP 服务器。 |
| `pnpm dev:frontend` | 运行 Vite 前端开发服务器（并代理到后端）。 |
| `pnpm test` | 使用 Vitest 运行前端测试。 |
| `pnpm build:frontend` | 类型检查并构建前端到 `dist/`。 |
| `cargo build --release -p pebble` | 构建用于发布的后端二进制文件。 |
| `cargo test -p pebble-mail` | 运行邮件模块测试。 |
| `cargo check` | 检查 Rust 工作区。 |

## 项目结构

```text
Pebble/
|-- src/                    React 前端 (SPA)
|   |-- components/         通用 UI 组件
|   |-- features/           收件箱、写信、搜索、看板、设置等功能
|   |-- hooks/              React hooks 和查询工具
|   |-- lib/                HTTP API 客户端、SSE 客户端、i18n、通用工具
|   |-- stores/             Zustand 状态管理
|   `-- sse-client.ts       SSE 事件监听 (EventSource)
|-- server/                 Rust HTTP 后端 (Axum)
|   `-- src/
|       |-- main.rs         服务器入口，路由注册
|       |-- auth.rs         OAuth 登录和回调处理
|       |-- state.rs        共享应用状态
|       |-- session.rs      Cookie 会话 + 限流器
|       |-- middleware/      Auth 中间件（Cookie 验证）
|       |-- api/            REST API 处理器（80+ 端点）
|       |-- realtime/       后台同步工作线程
|       |-- snooze_watcher.rs 稍后提醒定时后台任务
|       `-- rpc/            旧版 JSON-RPC 处理器（已弃用）
|-- crates/                 Rust 工作区
|   |-- pebble-core/        共享类型和错误定义
|   |-- pebble-store/       SQLite 持久化
|   |-- pebble-mail/        邮件提供商和同步逻辑
|   |-- pebble-search/      Tantivy 搜索索引
|   |-- pebble-crypto/      凭据加密
|   |-- pebble-oauth/       OAuth 2.0 和 PKCE
|   |-- pebble-rules/       规则引擎
|   |-- pebble-translate/   翻译提供商
|   `-- pebble-privacy/     HTML 清理和追踪保护
|-- tests/                  前端测试
`-- site/                   静态项目站点和截图
```

## 快捷键

| 快捷键 | 操作 |
| --- | --- |
| `J` / `K` | 在邮件列表中上下移动 |
| `Enter` | 打开选中的邮件 |
| `E` | 归档 |
| `S` | 切换星标 |
| `R` | 回复 |
| `A` | 回复全部 |
| `F` | 转发 |
| `C` | 写新邮件 |
| `/` | 聚焦搜索 |
| `Esc` | 关闭、取消或返回 |

快捷键可以在设置中查看和自定义。

## 当前状态

Pebble 正在持续开发中。它可以用于日常测试，但邮件客户端会处理敏感数据，不同邮件服务商的行为也存在差异。测试新版本时，请为重要邮件保留备份，并在服务商网页端核对关键操作。

## 参与贡献

欢迎提交 issue 和 pull request。

代码改动请尽量保持聚焦；涉及行为变化时，请补充相应测试。提交前建议运行相关检查：

```bash
pnpm test
pnpm build:frontend
cargo check
```

## 许可证

Pebble 使用 [GNU Affero General Public License v3.0](LICENSE) 许可证。

---

<p align="center">
  由 <a href="https://github.com/QingJ01">QingJ</a> 构建。
  <br>
  友情链接：<a href="https://linux.do">LINUX DO</a>
</p>
