# Rust 后端托管前端静态文件调研

## 资料来源

- Context7 `/tower-rs/tower-http`：`ServeDir` / `ServeFile` 可用于托管目录或单文件，支持 SPA fallback、MIME 类型、Range、预压缩文件等；需要启用 `tower-http` 的 `fs` feature。
- 本仓库现状：`server/Cargo.toml` 已使用 `tower-http = 0.6.8`，当前 features 为 `cors`、`compression-full`，尚未启用 `fs`。

## 代码约束

- 当前 `server/src/main.rs` 只注册 API/SSE/OAuth/webhook/docs 路由，没有静态文件路由。
- `server/src/middleware/mod.rs` 的鉴权规则：非 `/api/*` 默认豁免，`/events` 不豁免。因此首页、JS/CSS 静态资源不会被登录中间件挡住。
- 当前 Docker 是双镜像：`deploy/backend.Dockerfile` 构建后端，`deploy/frontend.Dockerfile` 构建前端并用 nginx 托管。
- 当前生产 Compose 有 backend/frontend 两个服务；Docker 发布 CI 同时发布 `pebble` 和 `pebble-frontend` 两个 GHCR 镜像。

## 可行方案

### 方案 A：后端运行时读取 `/app/dist`

- Docker 构建阶段先 `pnpm build:frontend` 生成 `dist/`，再复制到后端运行镜像 `/app/dist`。
- Rust 使用 `ServeDir::new("dist")` 或环境变量指定静态目录。
- 本地源码运行也可先 `pnpm build:frontend`，再 `cargo run -p pebble`。

优点：实现简单，二进制较小，静态资源不塞进 ELF。  
缺点：裸二进制发布时必须随包携带 `dist/`，否则只能提供 API。

### 方案 B：编译期把 `dist/` 嵌进 Rust 二进制

- 使用 `rust-embed` / `include_dir` 之类把前端资源编译进二进制。
- 发布一个二进制即可运行完整 Web UI。

优点：真正“一文件应用”。  
缺点：增加依赖和构建脚本复杂度；每次前端变更都导致 Rust 二进制体积和编译产物变化；CI 多平台二进制发布需要先构建前端再 cargo build。

## 初步建议

推荐先做方案 A：Docker 单镜像内含 `/app/dist`，Rust 后端用 `tower-http::services::ServeDir` 托管静态文件，同时删掉前端 nginx 镜像。这样最小化新增依赖和风险，也能达成“一个容器、无内层 nginx”的部署目标。
