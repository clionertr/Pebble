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
