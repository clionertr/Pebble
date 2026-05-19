<p align="center">
  <img src="src/assets/app-icon.png" alt="Pebble logo" width="120">
</p>

<h1 align="center">Pebble</h1>

<p align="center">
  一个自托管的网页邮件客户端，让收件箱更安静、更私密。
  <br>
  A self-hosted webmail client for people who want a calmer, more private inbox.
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
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20VPS%20%7C%20Self--hosted-lightgrey?style=flat-square" alt="Platform">
</p>

---

## Pebble 是什么？

简单说：**把 Gmail 搬到你自己的服务器上**。

Pebble 是一个网页邮件客户端，安装在你自己的 VPS 或 NAS 上。在浏览器里登录之后，连接你的邮箱账户（Gmail、IMAP、Outlook 都支持），就能在一个统一的界面上收发管理所有邮件。

最重要的：**所有数据都在你自己的服务器上**。邮件内容、附件、搜索索引、账户配置、规则——一律不经过第三方。

打个比方：它是你私有的 Gmail，没有广告，没有追踪，没人偷看你的收件箱。

## 快速开始

选一种适合你的方式。

### 一键 Docker 部署（推荐）

前提：你已安装 Docker 和 Docker Compose。安装脚本会拉取最新 tag 对应的 GHCR 镜像，创建 `./pebble`，写入 `.env`，启动服务，并检查 `http://127.0.0.1:9191` 是否可访问。如果当前用户不能直接连接 Docker，但免密 sudo 可用，脚本会自动改用 `sudo -n docker`。

```bash
curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh | bash
```

安装过程中可以直接回车使用默认值，也可以手动输入：

- 公网访问地址；默认会自动探测成 `http://<服务器IP>:9191`
- Pebble 登录密码；留空会自动生成 32 位随机密码
- 可选的 Google/Microsoft OAuth 凭据

把你的反向代理指向 `http://127.0.0.1:9191`。所有数据会保存在 `./pebble/data`。

非交互示例：

```bash
# 全自动：自动探测 IP，并生成 32 位登录密码
curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh | bash

# 指定域名和登录密码
curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh \
  | PEBBLE_PASSWORD='你的密码' \
    PEBBLE_PUBLIC_URL='https://mail.example.com' \
    bash
```

### 源码编译

前提：安装 **Rust**（stable 版本）、**Node.js 18+**、**pnpm 8+**。

```bash
git clone https://github.com/QingJ01/Pebble.git
cd Pebble

# 安装前端依赖
pnpm install

# 创建 .env 配置文件
cp .env.example .env
# 生成哈希：printf '%s' '你的密码' | cargo run -p pebble -- hash-password
# 编辑 .env，设置 PEBBLE_PASSWORD_HASH

# 终端 1：启动后端
cargo run -p pebble

# 终端 2：启动前端开发服务器
pnpm dev:frontend
```

打开 `http://localhost:1420`。Vite 开发服务器会自动把 API 请求转发到后端的 3000 端口。

### 生产环境部署（裸机）

```bash
# 构建后端
cargo build --release -p pebble

# 构建前端
pnpm build:frontend

# 启动后端
PEBBLE_PASSWORD_HASH='你的哈希' ./target/release/pebble
```

用 nginx 托管 `dist/` 目录（配置示例见下文）。后端默认监听 3000 端口。

## 配置指南

所有配置都通过**环境变量**设置。可以写在 `.env` 文件里，也可以直接传给二进制文件。

### 必须配置：登录密码

| 变量 | 说明 | 如何获取 |
|---|---|---|
| `PEBBLE_PASSWORD_HASH` | 登录密码的 bcrypt 哈希 | `printf '%s' '你的密码' \| pebble hash-password` |

这是唯一的必填项。不填的话，后端会拒绝启动。

### 可选：OAuth 提供商

要使用 Gmail 或 Outlook，需要配置 OAuth 凭据。

#### Gmail

1. 打开 [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. 创建一个项目，然后创建 **OAuth 2.0 客户端 ID**，类型选 **Web application**
3. 添加 `https://你的域名/auth/callback` 为已授权的重定向 URI（本地开发用 `http://localhost:3000/auth/callback`）
4. 把 Client ID 和 Client Secret 填到 `.env`：

```
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=GOCSPX-你的密钥
```

#### Outlook / Microsoft

1. 打开 [Azure 应用注册](https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/)，注册新应用
2. 重定向 URI 设为 `https://你的域名/auth/callback`
3. 应用类型选 **public/native**（不需要 client secret）。如果选了 Web 应用类型，需要提供 secret。

```
MICROSOFT_CLIENT_ID=你的-microsoft-client-id
# MICROSOFT_CLIENT_SECRET=  (public/native 应用留空即可)
```

### 可选：服务器设置

| 变量 | 默认值 | 说明 |
|---|---|---|
| `PEBBLE_HOST` | `127.0.0.1` | 监听地址。想对外提供服务设成 `0.0.0.0` |
| `PEBBLE_PORT` | `3000` | 监听端口 |
| `OAUTH_REDIRECT_URL` | `http://localhost:3000` | OAuth 回调的完整 URL。生产环境改成 `https://你的域名` |
| `ALLOWED_ORIGIN` | 空 | CORS 允许的源。前后端同源部署时空着就行。前后端分离时设为前端的 URL |

### 可选：Gmail 实时推送

Gmail 可以通过 Google Cloud Pub/Sub 向 Pebble 推送新邮件通知，无需轮询。

| 变量 | 说明 |
|---|---|
| `GMAIL_PUBSUB_TOPIC` | 完整的 Pub/Sub Topic：`projects/<项目ID>/topics/gmail-webmail-topic` |
| `GMAIL_WEBHOOK_SECRET` | 一个随机的密钥字符串，用于 webhook URL 验证 |

配置步骤：
1. 在 Google Cloud 启用 Gmail API 和 Cloud Pub/Sub API
2. 创建 Pub/Sub Topic，将 `roles/pubsub.publisher` 授予 `gmail-api-push@system.gserviceaccount.com`
3. 创建推送订阅，指向 `https://你的域名/webhook/gmail?secret=<你的密钥>`
4. 在 Pebble 中，进入 **Settings → Accounts → Enable realtime Gmail** 按账户启用

## 生产环境部署

### Nginx 反向代理

推荐方案：nginx 托管前端静态文件，反向代理 API 请求到后端。

```nginx
server {
    listen 443 ssl;
    server_name mail.你的域名.com;

    root /path/to/Pebble/dist;
    index index.html;

    # 安全头
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Referrer-Policy "no-referrer" always;
    add_header Content-Security-Policy "default-src 'self'; img-src 'self' data: https:; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'" always;

    # 前端 SPA —— 回退到 index.html 以支持客户端路由
    location / {
        try_files $uri $uri/ /index.html;
    }

    # 后端 API、SSE（实时推送）、OAuth、Gmail webhook
    location ~ ^/(api|events|auth|webhook) {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;

        # SSE 连接所需
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 3600s;
    }
}
```

### Docker Compose（生产）

一键安装脚本会从 `deploy/compose.prod.yml` 写出 compose 文件。如果你想手动维护，可以使用预构建的 GHCR 镜像：

`latest` 只会在仓库推送版本 tag（例如 `v0.0.9`）时更新。

```yaml
name: pebble

services:
  backend:
    image: ghcr.io/clionertr/pebble:latest
    volumes:
      - ./data:/app/data
    env_file:
      - .env
    environment:
      PEBBLE_HOST: 0.0.0.0
      PEBBLE_PORT: 3000
    restart: unless-stopped
    networks:
      - pebble-net

  frontend:
    image: ghcr.io/clionertr/pebble-frontend:latest
    ports:
      - "127.0.0.1:9191:80"
    depends_on:
      - backend
    restart: unless-stopped
    networks:
      - pebble-net

networks:
  pebble-net:
    driver: bridge
```

这种部署方式下，把你的公网反向代理（nginx、Caddy、1Panel OpenResty 等）指向 `http://127.0.0.1:9191` 即可。

### 数据持久化

所有数据存储在后端工作目录下的 `./data/` 目录：

| 文件 / 目录 | 内容 |
|---|---|
| `data/pebble.db` | SQLite 数据库，存储邮件、账户、规则、设置 |
| `data/pebble.key` | 凭据加密密钥（OAuth token、密码） |
| `data/index/` | Tantivy 全文搜索索引 |
| `data/attachments/` | 下载的邮件附件 |
| `data/logs/` | 应用日志 |

**请妥善保管 `data/pebble.key`。** 如果丢失，所有已连接的账户将无法解密，需要重新认证。

## 工作原理

### 架构

```
浏览器 (React SPA)
        │  HTTP REST  /api/*
        │  SSE 流     /events
        │  OAuth 流程  /auth/login  /auth/callback
        ▼
Nginx (托管前端、反向代理 API)
        │
        ▼
Rust HTTP 服务器 (Axum, 端口 3000)
        │
        ├── pebble-store    SQLite 数据库
        ├── pebble-search   Tantivy 全文索引
        ├── pebble-mail     IMAP / Gmail / Outlook 同步
        ├── pebble-crypto   凭据加密
        ├── pebble-oauth    OAuth 2.0 + PKCE
        ├── pebble-rules    规则引擎
        ├── pebble-translate 翻译
        └── pebble-privacy  HTML 清理与追踪保护
```

### 认证机制

Pebble 使用 **Cookie 会话认证**：
- 你用密码登录 → 服务器创建会话（7 天有效期）
- 会话 cookie（`pebble_session`）标记为 `HttpOnly; Secure; SameSite=Strict`
- 所有 `/api/*` 端点都需要有效会话
- 登录失败有频率限制（5 次失败 → 锁定 15 分钟，按 IP 计算）
- 单用户设计——无需注册，无需多用户管理

### 实时推送

前端通过 **Server-Sent Events**（SSE）连接 `GET /events`。服务器会实时推送新邮件通知、同步进度、稍后提醒等事件。SSE 连接使用同一个会话 cookie 认证。

### 邮件同步

Pebble 在后台同步你的邮件：
- **Gmail**：OAuth + Gmail API（基于历史的增量同步）+ 可选的 Pub/Sub 实时推送
- **IMAP**：标准 IMAP 轮询，可配置轮询间隔
- **Outlook**：OAuth + Microsoft Graph API（实验性）

## 功能一览

### 邮件
- 多账户统一收件箱
- Gmail、IMAP、Outlook（实验性）
- 线程视图 + 邮件列表视图
- 归档、删除、星标、已读标记、批量操作、恢复
- 稍后提醒（Snooze）
- 全文搜索 + 高级过滤
- 规则引擎，自动整理邮件
- 命令面板 + 键盘快捷键

### 效率工具
- **看板**：Todo → Waiting → Done 三列，管理邮件任务
- **翻译**：内置翻译能力，双语阅读模式
- **模板**：可复用的邮件模板
- **信任发件人**：按发件人控制隐私（显示图片等）
- **WebDAV 备份**：同步设置、规则和看板数据到 WebDAV 服务器

### 隐私与安全
- 所有数据存储在本地服务器
- 无遥测，无追踪
- HTML 邮件净化（移除追踪器）
- OAuth token 加密存储

## 技术栈

| 层级 | 技术 |
|---|---|
| 后端 | Rust + Axum |
| 前端 | React 19 + TypeScript |
| 状态管理 | Zustand + TanStack Query |
| 数据库 | SQLite（rusqlite） |
| 搜索 | Tantivy |
| 样式 | Tailwind CSS |
| 国际化 | i18next（英文、中文） |

## 快捷键

| 快捷键 | 操作 |
|---|---|
| `J` / `K` | 在邮件列表中上下移动 |
| `Enter` | 打开选中邮件 |
| `E` | 归档 |
| `S` | 切换星标 |
| `R` | 回复 |
| `A` | 回复全部 |
| `F` | 转发 |
| `C` | 写新邮件 |
| `/` | 聚焦搜索 |
| `Esc` | 关闭、取消、返回 |

快捷键可以在设置中查看和自定义。

## 常用命令

| 命令 | 用途 |
|---|---|
| `cargo run -p pebble` | 运行后端开发服务器 |
| `pnpm dev:frontend` | 运行前端开发服务器（代理到后端） |
| `pnpm build:frontend` | 类型检查 + 构建前端到 `dist/` |
| `cargo build --release -p pebble` | 构建发布版后端 |
| `pnpm test` | 运行前端测试 |
| `cargo test -p pebble-mail` | 运行邮件模块测试 |
| `cargo check` | 检查 Rust 代码 |

## 常见问题

### 每次请求都提示 "Authentication required"
会话过期（7 天）或后端重启了。重新登录即可。

### 部署后无法登录
检查 `.env` 中的 `PEBBLE_PASSWORD_HASH`，`$` 符号是否用 `$$` 转义了（Docker Compose 要求）。可以用 `docker exec pebble-backend env | grep PASSWORD` 查看容器内的实际值。

### 某些 API 返回 404
确认 nginx 配置中代理了 `/api/*` 路径。反向代理规则应包含：`location ~ ^/(api|events|auth|webhook)`。

### 数据库提示 "disk image is malformed"
SQLite 数据库可能因异常关闭而损坏。尝试修复：`sqlite3 data/pebble.db "PRAGMA integrity_check;"`。如果损坏，从备份恢复。

### 邮件同步不工作
查看后端日志：`docker logs pebble-backend` 或 `tail -f data/logs/`。常见原因：OAuth token 过期（在设置 → 账户中重新认证）、网络代理未配置、IMAP 凭据错误。

## 项目结构

```text
Pebble/
├── src/                    React 前端 (SPA)
│   ├── components/         通用 UI 组件
│   ├── features/           收件箱、写信、搜索、看板、设置、认证
│   ├── hooks/              React hooks 和查询工具
│   ├── lib/                API 客户端、SSE 客户端、i18n、通用工具
│   └── stores/             Zustand 状态管理
├── server/                 Rust HTTP 后端 (Axum)
│   └── src/
│       ├── main.rs         服务器入口，路由注册
│       ├── api/            REST API 处理器（80+ 端点）
│       ├── middleware/      Auth 中间件（Cookie 验证）
│       ├── session.rs      Cookie 会话 + 限流器
│       └── rpc/            内部服务层
├── crates/                 Rust 工作区
│   ├── pebble-core/        共享类型和错误
│   ├── pebble-store/       SQLite 持久化
│   ├── pebble-mail/        邮件提供商和同步
│   ├── pebble-search/      Tantivy 搜索索引
│   ├── pebble-crypto/      凭据加密
│   ├── pebble-oauth/       OAuth 2.0 和 PKCE
│   ├── pebble-rules/       规则引擎
│   ├── pebble-translate/   翻译提供商
│   └── pebble-privacy/     HTML 清理和追踪控制
├── deploy/                 Docker 和 nginx 配置
├── tests/                  前端测试
└── site/                   截图
```

## 许可证

Pebble 使用 [GNU Affero General Public License v3.0](LICENSE) 许可证。

---

<p align="center">
  由 <a href="https://github.com/QingJ01">QingJ</a> 原创构建。
  <br>
  Web 服务重架构与文档：<strong>Claude Opus 4.7</strong>。
  <br>
  友情链接：<a href="https://linux.do">LINUX DO</a>
</p>
