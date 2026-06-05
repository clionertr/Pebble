# 配置指南

## Environment Variables

后端服务通过环境变量配置。生产和 Docker 部署必须显式提供登录密码哈希。

| Variable | Description | Default |
|----------|-------------|---------|
| `PEBBLE_PASSWORD_HASH` | 单用户登录密码的 bcrypt 哈希。缺失时后端拒绝启动。 | 必填 |
| `PEBBLE_HOST` | The host address the backend binds to. | `127.0.0.1` |
| `PEBBLE_PORT` | The port the backend listens on. | `3000` |
| `ALLOWED_ORIGIN` | 跨域部署时允许的前端 Origin；同源部署保持为空。 | 空 |
| `OAUTH_REDIRECT_URL` | OAuth 回调根地址，最终回调为 `/auth/callback`。 | `http://localhost:3000` |
| `GMAIL_PUBSUB_TOPIC` | Gmail Pub/Sub topic，用于 Gmail 实时推送。 | 可选 |
| `GMAIL_WEBHOOK_SECRET` | Gmail webhook query secret。 | 可选 |
| `PEBBLE_VAPID_PRIVATE_KEY` | 浏览器 Web Push 使用的 base64url VAPID 私钥；缺省时自动生成并保存。 | 自动生成 |
| `PEBBLE_VAPID_PUBLIC_KEY` | 可选 VAPID 公钥；若设置必须和私钥匹配，否则启动失败。 | 从私钥推导 |
| `PEBBLE_VITE_ALLOWED_HOSTS` | Vite 开发服务器额外允许的 Host，逗号分隔；仅用于反向代理或远程开发域名访问本地 Vite。 | 空 |

### Binding Strategy

- **本地开发**：保持默认 `127.0.0.1:3000`，通过 Vite dev server 代理访问。
- **远程开发域名**：如果通过反向代理访问 Vite dev server，使用 `PEBBLE_VITE_ALLOWED_HOSTS=dev.example.com,pebble.example.com` 显式放行 Host，不要把个人域名硬编码进 `vite.config.ts`。
- **源码直接运行**：后端启动时会读取当前工作目录的 `.env`，但已经存在的进程环境变量优先；systemd 部署必须设置正确的 `WorkingDirectory` 和 `EnvironmentFile`。
- **Docker 部署**：Compose 必须设置 `PEBBLE_HOST=0.0.0.0`，否则 frontend 容器无法通过容器网络访问 backend。
- **公网部署**：推荐只暴露前端 nginx 端口，后端只在容器网络或本机回环地址内可达。
- **浏览器通知**：Web Push 生产环境需要 HTTPS 或浏览器认可的安全上下文；localhost 仅用于开发例外。
- **单进程数据目录**：同一个 `data/` 目录只能有一个 Pebble 后端进程；`data/index/` 的 Tantivy writer lock 出现 `LockBusy` 时，先停止旧进程，不要并行运行 `cargo run`、release binary 和 systemd 服务。

## Examples

### Docker Compose (.env)
```env
PEBBLE_PASSWORD_HASH=$$2b$$12$$...
OAUTH_REDIRECT_URL=https://mail.example.com
ALLOWED_ORIGIN=
```

## Scenario: 邮件内联样式与生产 CSP

### 1. Scope / Trigger
- Trigger: 前端 nginx 的 `Content-Security-Policy` 会直接影响 `ShadowDomEmail` 渲染出的邮件 HTML。
- 范围：`deploy/nginx.conf`、`deploy/nginx-public.example.conf`、README 中的 nginx 示例、`src/components/ShadowDomEmail.tsx` 的 Shadow DOM 样式加载，以及 `crates/pebble-privacy/src/sanitizer.rs` 的 CSS 白名单。

### 2. Signatures
- nginx 响应头：
  ```nginx
  add_header Content-Security-Policy "default-src 'self'; img-src 'self' data: https:; script-src 'self'; style-src 'self'; style-src-elem 'self'; style-src-attr 'unsafe-inline'; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'" always;
  ```
- sanitizer 入口：`PrivacyGuard::render_safe_html(raw_html, mode)`。

### 3. Contracts
- `style-src 'self'` 和 `style-src-elem 'self'` 继续限制外部 CSS 与 `<style>` 元素。
- `ShadowDomEmail` 的 Shadow DOM 壳样式必须通过同源静态文件加载，例如 `/shadow-dom-email.css`；不得在运行时创建内联 `<style>` 元素。
- `style-src-attr 'unsafe-inline'` 只允许元素上的 `style=""` 属性，用于邮件正文和 blocked image placeholder 的安全样式。
- 允许 `style-src-attr 'unsafe-inline'` 的前提是 sanitizer 对 `style` 属性执行 CSS 属性和值白名单过滤。
- 禁止把 nginx 配置退回到全局 `style-src 'self' 'unsafe-inline'`。

### 4. Validation & Error Matrix
- CSP 缺少 `style-src-attr 'unsafe-inline'` -> Docker/nginx 生产环境会拦截邮件正文或 blocked image placeholder 的 `style=""`。
- `ShadowDomEmail` 运行时创建内联 `<style>` -> `style-src-elem 'self'` 会阻止该样式，邮件壳样式失效。
- CSP 使用全局 `style-src 'self' 'unsafe-inline'` -> `<style>` 元素也被放宽，超出邮件正文样式需求。
- sanitizer 放行 `url()`、`data:`、`javascript:`、`@import` 或 CSS 反斜杠转义 -> 邮件内联样式可能变成外部请求或脚本绕过载体。
- sanitizer 放行 `position`、`z-index` 等覆盖布局属性 -> 邮件内容可能伪装系统界面或遮挡应用 UI。

### 5. Good/Base/Bad Cases
- Good: 生产 CSP 使用 `style-src-attr 'unsafe-inline'`，邮件中 `color`、`width` 等白名单样式生效，危险 CSS 值被 sanitizer 删除。
- Base: blocked image placeholder 生成 `style="width:600px;height:200px"`，浏览器允许应用该属性；Shadow DOM 壳样式通过 `/shadow-dom-email.css` 加载，外部图片仍被隐私模式阻止。
- Bad: 只保留 `style-src 'self'`，dev 正常但生产邮件样式被浏览器 CSP 拦截。

### 6. Tests Required
- `cargo test -p pebble-privacy --all-targets`，断言安全 CSS 保留、`url()` / `data:` / `javascript:` / `@import` / 反斜杠转义 / `position` / `z-index` 被过滤。
- `cargo clippy --workspace --all-targets -- -D warnings`。
- `cargo test --workspace --all-targets`。
- 修改 nginx CSP 后搜索 README、deploy 示例和最佳实践文档，确认没有残留旧策略。
- `pnpm build:frontend`，断言 `public/shadow-dom-email.css` 被复制到 `dist/shadow-dom-email.css`。

### 7. Wrong vs Correct

#### Wrong
```typescript
const style = document.createElement("style");
style.textContent = "...";
shadow.appendChild(style);
```

#### Correct
```typescript
const stylesheet = document.createElement("link");
stylesheet.rel = "stylesheet";
stylesheet.href = "/shadow-dom-email.css";
shadow.appendChild(stylesheet);
```

## Scenario: 一键 Docker 部署配置链路

### 1. Scope / Trigger
- Trigger: Pebble 支持从 GitHub 拉取 `deploy/install.sh` 后直接部署，部署链路横跨 GitHub Actions、GHCR 镜像、Docker Compose、`.env` 和后端启动参数。
- 范围：`.github/workflows/docker.yml`、`deploy/install.sh`、`deploy/compose.prod.yml`、`deploy/backend.Dockerfile`、`deploy/frontend.Dockerfile`、`server/src/main.rs`。

### 2. Signatures
- 安装命令：`curl -fsSL https://raw.githubusercontent.com/clionertr/Pebble/master/deploy/install.sh | bash`。
- 密码哈希命令：`pebble hash-password [password]`，不传参数时从 stdin 读取。
- 生产 compose：`deploy/compose.prod.yml`，默认镜像 `ghcr.io/clionertr/pebble:latest` 和 `ghcr.io/clionertr/pebble-frontend:latest`。
- 入口端口：前端容器默认绑定 `127.0.0.1:9191:80`。
- Docker 镜像 workflow 只由 SemVer tag（`vMAJOR.MINOR.PATCH` 或 `vMAJOR.MINOR.PATCH-prerelease`）触发；每次构建始终发布 `latest` 和 `sha-*` tag；`MAJOR.MINOR` 只在正式版本 tag 构建成功后更新，预发布 tag 只发布完整版本号。

### 3. Contracts
- `deploy/install.sh` 默认在当前目录创建 `./pebble`，其中包含 `compose.yml`、`.env` 和 `data/`。
- `.env` 必须写入 `PEBBLE_PASSWORD_HASH`、`OAUTH_REDIRECT_URL`、`PEBBLE_BACKEND_IMAGE`、`PEBBLE_FRONTEND_IMAGE`、`PEBBLE_HTTP_BIND`。
- Docker Compose 中后端必须设置 `PEBBLE_HOST=0.0.0.0`，前端 nginx 通过容器网络访问 `backend:3000`。
- 同源 Docker 部署中 `ALLOWED_ORIGIN` 必须保持为空。
- bcrypt hash 写入 Docker Compose `.env` 时，`$` 必须写成 `$$`；容器运行时会得到单 `$`。
- GHCR 镜像应公开可拉取；安装脚本不得要求普通用户先 `docker login ghcr.io`。
- 如果当前用户无法连接 Docker daemon，但免密 sudo 可用，安装脚本必须自动使用 `sudo -n docker`。
- `PEBBLE_PUBLIC_URL` 未设置时，安装脚本默认探测公网 IP 并生成 `http://<ip>:<port>`。
- 登录密码留空或非交互且未设置 `PEBBLE_PASSWORD` 时，安装脚本默认生成 32 位随机密码并在结束时打印一次。

### 4. Validation & Error Matrix
- 缺少 `docker` -> 安装脚本报错并退出，不继续写半截部署。
- 缺少 `docker compose` -> 安装脚本报错并退出。
- GHCR 镜像拉取失败 -> 提示检查 GitHub Packages 是否设为 Public。
- `PEBBLE_PUBLIC_URL` 不以 `http://` 或 `https://` 开头 -> 安装脚本报错或重新提示。
- 健康检查 `http://127.0.0.1:9191` 超时 -> 打印 `docker compose ps` 和 backend/frontend 最近日志。
- `pebble hash-password` 收到空密码 -> 返回错误，不输出 hash。
- 源码运行时 `.env` 缺失 `PEBBLE_PASSWORD_HASH` 或值不是 bcrypt hash -> 启动失败并打印生成命令。
- `data/index/` 返回 `LockBusy` -> 启动失败并提示停止旧后端或处理 stale lock。

### 5. Good/Base/Bad Cases
- Good: 用户运行安装脚本，输入 `https://mail.closev.com` 或接受自动探测的 IP URL；登录密码可以手动输入，也可以留空生成 32 位随机密码。脚本生成 `.env`、拉取 `latest` 镜像、启动服务，并确认 `127.0.0.1:9191` 可访问。
- Base: 用户暂不配置 Google/Microsoft OAuth，`.env` 中 OAuth 字段为空，服务仍能启动；后续可编辑 `.env` 后重启。
- Bad: 将生产 compose 改回本地 `build:` 或暴露后端 `3000` 到公网，会破坏“一键部署拉镜像”和后端只在内部网络可达的契约。

### 6. Tests Required
- `bash -n deploy/install.sh`。
- `docker compose --project-directory <tmp> --env-file <tmp>/.env -f <tmp>/compose.yml config`，断言前端绑定 `127.0.0.1:9191`。
- `printf '%s' 'password' | cargo run -p pebble -- hash-password` 输出 bcrypt 格式。
- `cargo clippy --workspace --all-targets -- -D warnings`。
- `cargo test --workspace --all-targets`。
- `pnpm test` 和 `pnpm build:frontend`。

### 7. Wrong vs Correct

#### Wrong
```yaml
services:
  backend:
    ports:
      - "3000:3000"
  frontend:
    build:
      context: .
```

#### Correct
```yaml
services:
  backend:
    image: ghcr.io/clionertr/pebble:latest
    environment:
      PEBBLE_HOST: 0.0.0.0
  frontend:
    image: ghcr.io/clionertr/pebble-frontend:latest
    ports:
      - "127.0.0.1:9191:80"
```
