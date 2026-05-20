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

### Binding Strategy

- **本地开发**：保持默认 `127.0.0.1:3000`，通过 Vite dev server 代理访问。
- **Docker 部署**：Compose 必须设置 `PEBBLE_HOST=0.0.0.0`，否则 frontend 容器无法通过容器网络访问 backend。
- **公网部署**：推荐只暴露前端 nginx 端口，后端只在容器网络或本机回环地址内可达。

## Examples

### Docker Compose (.env)
```env
PEBBLE_PASSWORD_HASH=$$2b$$12$$...
OAUTH_REDIRECT_URL=https://mail.example.com
ALLOWED_ORIGIN=
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
