# Rust 后端内置前端静态文件

## Goal

让 Pebble 从“后端容器 + 前端 nginx 容器”的双容器部署，演进为“一个 Rust 后端容器同时提供 API 和前端静态页面”。目标是降低部署认知负担：用户只需要启动一个 Pebble 服务，再按需接宿主机 nginx 或 Cloudflare Tunnel。

## What I already know

* 用户希望开启 Trellis 工作流，主题是“Rust 后端内置前端静态文件”。
* 用户希望删除多余部署文件，并同步更新文档、CI 等外围设施。
* 当前部署存在双层 nginx / 前后端双容器，容易混淆。
* 当前后端 `server/src/main.rs` 只提供 API、SSE、OAuth、Gmail webhook 和 API docs，没有托管前端静态资源。
* 当前 `server/src/middleware/mod.rs` 对非 `/api/*` 路径默认豁免鉴权；因此首页和 JS/CSS 静态文件不会被登录中间件拦截。
* 当前 `tower-http` 已存在，但未启用 `fs` feature；`ServeDir`/`ServeFile` 是自然候选。
* 当前 Docker 发布 CI 会构建并发布两个镜像：`ghcr.io/clionertr/pebble` 和 `ghcr.io/clionertr/pebble-frontend`。
* 当前部署相关文件包括 `deploy/frontend.Dockerfile`、`deploy/backend.Dockerfile`、`deploy/nginx.conf`、`deploy/nginx-public.example.conf`、`deploy/compose.prod.yml`、根目录 `docker-compose.yml`、`deploy/docker-compose.yml`、`deploy/install.sh` 等。

## Assumptions (temporary)

* 主目标是 Docker 部署轮椅化：一个容器提供完整 Pebble Web UI + API。
* 仍允许宿主机 nginx 或 Cloudflare Tunnel 作为外部入口，但它们只做公网入口/TLS，不再负责区分前端和 API。
* 不打算在生产里用 `npm run dev` 或 `vite preview` 跑前端。
* 初始实现选择“运行时读取 dist 目录”：前端构建产物内置到 Docker 镜像中，Rust 后端从镜像内目录托管静态文件；本轮不把 dist 编译进单个 Rust 二进制。

## Open Questions

* 无。用户已确认进入实现阶段，并要求实现计划包含合适的验证步骤。

## Requirements (evolving)

* Rust 后端需要能托管前端 `dist/` 静态文件。
* Rust 后端需要接管原 nginx 的基础安全响应头：`X-Content-Type-Options`、`X-Frame-Options`、`Referrer-Policy`、`Content-Security-Policy`；默认不加 HSTS。
* 静态资源缓存策略：`index.html` / SPA fallback 使用 `Cache-Control: no-cache`；`/assets/*` 等带 hash 资源使用 `Cache-Control: public, max-age=31536000, immutable`；`/pebble-sw.js` 使用 `Cache-Control: no-cache`，避免 Service Worker 升级卡旧版本。
* 登录 cookie 继续保持 `Secure=true`、`HttpOnly=true`、`SameSite=Strict`，不为 HTTP 域名放宽。
* 当 `dist/index.html` 不存在时，后端仍应继续启动，保留 API-only 能力；访问前端页面时返回清晰提示，而不是整个进程退出。
* 本轮 MVP 只要求 Docker 单镜像包含并托管前端 `dist/`；如果体验足够好，可以长期不做“dist 编译进 Rust 二进制”的方案。
* SPA 路由刷新时需要回退到 `index.html`。
* 静态文件与 SPA fallback 只处理 GET/HEAD；非 GET/HEAD 未匹配路由时不应返回 `index.html`。
* API/SSE/OAuth/webhook 路由必须继续优先生效，不能被静态 fallback 吃掉。
* Docker 生产部署应从两个服务简化为一个 Pebble 服务。
* 单容器部署的宿主机入口继续沿用 `127.0.0.1:9191`，映射到容器内 `3000`，保证旧反代和 Cloudflare Tunnel 指向尽量不变。
* 保留并改造 `deploy/compose.prod.yml` 作为一键安装/生产部署模板。
* 保留并改造根目录 `docker-compose.yml` 作为开发者本地 `docker compose up -d --build` 入口。
* 删除重复的 `deploy/docker-compose.yml`，避免同一用途出现两份 compose。
* 彻底删除前端镜像路线：删除 `deploy/frontend.Dockerfile`、`deploy/nginx.conf`、`deploy/nginx-public.example.conf`，CI 不再发布 `pebble-frontend`，文档不再讲前端容器。
* 不需要写旧双容器迁移说明；当前只有维护者本人使用。
* README / 中文 README / 部署文档 / 架构文档 / 集成指南中涉及部署边界的内容需要同步。
* README 需要保留一个很短的 Cloudflare Tunnel 小节：Public Hostname 的 service 指向 `http://127.0.0.1:9191`。
* CI / 发布流程需要从双镜像改为单镜像，并确保前端构建产物进入后端镜像。
* 删除裸二进制 GitHub Release 路线；Docker 单镜像成为唯一主推分发产物。
* GitHub Actions 处理边界：删除 `.github/workflows/release.yml`；保留 `ci.yml`；改造 `docker.yml` 为只构建/发布单个 `pebble` 镜像。

## Acceptance Criteria (evolving)

* [ ] `cargo run -p pebble` 在存在前端 `dist/` 时可以访问 `/` 并返回前端页面。
* [ ] `dist/index.html` 缺失时后端仍能启动，API 路由可用，前端入口返回清楚的缺失提示。
* [ ] 刷新 `/inbox`、`/settings` 等 SPA 路径时返回 `index.html`。
* [ ] 非 GET/HEAD 的未知路径不会 fallback 到 `index.html`。
* [ ] `index.html`/SPA fallback、`/assets/*`、`/pebble-sw.js` 的 `Cache-Control` 符合约定。
* [ ] `/api/*`、`/events`、`/auth/*`、`/webhook/*` 仍走原后端逻辑。
* [ ] Docker 生产 Compose 只需要一个 Pebble 应用服务。
* [ ] 根目录 `docker-compose.yml` 可用于本地单容器构建运行。
* [ ] `deploy/docker-compose.yml` 已删除或不再作为入口出现。
* [ ] 默认生产端口保持 `127.0.0.1:9191`，旧部署反代基本无需改动。
* [ ] GHCR 发布流程不再发布 `pebble-frontend` 镜像。
* [ ] 裸二进制 GitHub Release workflow 已删除，README 不再主推下载裸二进制部署。
* [ ] `.github/workflows/ci.yml` 保留质量门禁，`.github/workflows/docker.yml` 只发布单个 `pebble` 镜像。
* [ ] 仓库中不存在默认路径会引导用户继续使用前端 nginx 容器。
* [ ] 文档不再推荐内层 nginx / 前端 nginx 容器。
* [ ] 删除 nginx 后，前端页面/API 响应仍带基础安全响应头，且登录 cookie 安全属性不退化。
* [ ] README 有简短 Cloudflare Tunnel 指向 `http://127.0.0.1:9191` 的说明。
* [ ] 质量门禁通过：Rust fmt/clippy/test，前端 lint/test/build，Docker 构建配置可验证。

## Definition of Done

* Tests added/updated where appropriate.
* Lint / typecheck / CI green.
* Docs updated if deployment behavior changes.
* Rollout/rollback considered: 当前只有维护者本人使用，不要求单独旧部署迁移说明；文档直接呈现新推荐路径。
* 删除文件经过清单确认，避免误删仍有价值的裸机/开发辅助文件。

## Out of Scope (explicit)

* 暂不承诺重做登录系统、OAuth 流程或邮件同步逻辑。
* 暂不把 Cloudflare Tunnel 做成内置服务；它只是外部入口文档/示例。
* 暂不以 npm/Vite dev server 作为生产前端服务。
* 本轮不保留 GitHub Release 裸二进制分发路线；如未来需要裸机完整部署，再单独设计二进制 + dist 打包方式。

## Research References

* [`research/static-serving-research.md`](research/static-serving-research.md) — `tower-http::ServeDir`/`ServeFile` 是 Rust/Axum 托管前端静态文件的自然方案；推荐先做运行时读取 `dist/`。



## Technical Approach

本轮采用“Docker 单镜像内含前端 dist，Rust 后端运行时托管 dist”的方案：

* Docker 构建阶段先安装前端依赖并执行 `pnpm build:frontend`，再编译 Rust 后端，最终运行镜像只包含 `pebble` 二进制、`dist/` 和运行时依赖。
* Rust 后端使用 `tower-http` 静态文件服务能力托管 `dist/`，并确保 API/SSE/OAuth/webhook 路由优先于 SPA fallback。
* Rust 后端统一注入基础安全响应头与静态资源缓存头，替代原 nginx 配置职责。
* 部署层保留一个生产 compose 和一个根目录开发 compose，删除前端 nginx、前端镜像、重复 compose、裸二进制 release workflow。
* 文档只呈现新的单容器部署模型，并保留简短 Cloudflare Tunnel 指向说明。

## Implementation Plan

1. **后端静态服务与响应头**
   * 启用 `tower-http` 静态文件服务所需 feature。
   * 在后端路由末尾接入 `dist/` 静态服务和 SPA fallback。
   * 增加基础安全响应头。
   * 增加缓存头规则：`index.html`/fallback/no-cache、`pebble-sw.js`/no-cache、`assets/*`/immutable。
   * 验证：新增或更新 Rust 测试覆盖首页、SPA fallback、缺失 dist、非 GET/HEAD、API 优先级、安全头、缓存头。

2. **Docker 单镜像构建**
   * 改造 `deploy/backend.Dockerfile`，在构建阶段生成前端 `dist/` 并复制进最终运行镜像。
   * 删除前端 Dockerfile 和 nginx 配置。
   * 验证：运行 `docker compose config`，条件允许时运行 `docker compose build` 或至少验证 Dockerfile/compose 引用不再指向已删除文件。

3. **Compose 与安装脚本**
   * `deploy/compose.prod.yml` 改为单服务，默认 `127.0.0.1:9191:3000`。
   * 根目录 `docker-compose.yml` 改为单服务本地构建入口。
   * 删除 `deploy/docker-compose.yml`。
   * `deploy/install.sh` 去掉前端镜像变量和前端服务日志引用。
   * 验证：`docker compose -f deploy/compose.prod.yml config` 和根目录 `docker compose config` 通过。

4. **GitHub Actions 与发布路线**
   * 删除 `.github/workflows/release.yml`。
   * 改造 `.github/workflows/docker.yml`，只构建/发布 `ghcr.io/clionertr/pebble` 单镜像。
   * 保留 `.github/workflows/ci.yml`。
   * 验证：静态检查 workflow 中不再出现 `pebble-frontend`、`frontend.Dockerfile`、`release.yml` 入口。

5. **文档同步**
   * 更新 `README.md`、`README.zh-CN.md`、`docs/architecture.md`、`docs/integration-guide.md` 等涉及部署边界的内容。
   * 移除前端 nginx/双容器/裸二进制主推说明。
   * 添加简短 Cloudflare Tunnel 指向 `http://127.0.0.1:9191` 的说明。
   * 验证：全文搜索确认文档不再推荐前端 nginx 容器或 `pebble-frontend` 镜像。

6. **最终质量门禁**
   * `pnpm lint`
   * `pnpm format:check`
   * `pnpm test`
   * `pnpm build:frontend`
   * `cargo fmt --check`
   * `cargo clippy --workspace --all-targets -- -D warnings`
   * `cargo test --workspace --all-targets`
   * `docker compose config`
   * `docker compose -f deploy/compose.prod.yml config`
   * 本机部署验证：`http://127.0.0.1:9191` 可访问
   * 反代验证：用户已配置 `mail.closev.com -> 127.0.0.1:9191`，实现完成后通过该域名检查页面与安全头
   * `grep` 检查删除路线残留：`pebble-frontend`、`frontend.Dockerfile`、`nginx.conf`、`nginx-public.example.conf`、`release.yml`。

## Decision Log

### 2026-06-21: 静态文件内置层级

**Decision**: 本轮 MVP 选择“前端 dist 内置到 Docker 镜像，Rust 后端运行时读取并托管”，不把 dist 编译进 Rust 二进制。  
**Reason**: 这能先消灭前端 nginx 和双容器，同时避免过早引入二进制嵌入、跨平台 release 构建链复杂度。  
**Consequence**: Docker 部署会成为完整单容器；裸二进制如果没有随包携带 dist，可能只适合 API/开发调试，需在文档中讲清楚。

### 2026-06-21: 生产入口端口

**Decision**: 单容器生产部署继续保留宿主机 `127.0.0.1:9191` 作为默认入口，映射到容器内后端 `3000`。  
**Reason**: 旧版一键部署、宿主机反代和 Cloudflare Tunnel 已经围绕 `9191` 建立，保留门牌号能降低迁移成本。  
**Consequence**: 文档需说明“外部入口默认 9191，容器内部服务端口 3000”。

### 2026-06-21: 缺失 dist 时的行为

**Decision**: 找不到前端 `dist/index.html` 时，后端继续启动为 API-only 模式；访问前端入口时返回清晰提示。  
**Reason**: 本地后端调试和裸二进制运行不应被前端构建产物硬性阻断；Docker 镜像构建阶段会保证生产镜像包含 dist。  
**Consequence**: 实现需避免静态 fallback 吃掉 API 路由，并在文档中说明源码运行前端需要先 `pnpm build:frontend`。

### 2026-06-21: 前端 nginx/前端镜像路线

**Decision**: 本任务彻底删除前端镜像路线，不保留 legacy 默认路径；删除前端 nginx 相关配置，CI 不再发布 `pebble-frontend`。  
**Reason**: 仓库目前主要由维护者本人使用，保留旧路线会继续制造“到底用哪个”的混乱。  
**Consequence**: 文档直接呈现新单容器部署，不单独写旧双容器迁移说明。

### 2026-06-21: Compose 文件边界

**Decision**: 保留 `deploy/compose.prod.yml` 作为一键安装/生产模板；保留根目录 `docker-compose.yml` 作为开发者本地构建入口；删除重复的 `deploy/docker-compose.yml`。  
**Reason**: 两个入口分别服务“普通部署”和“本地开发”，而 `deploy/docker-compose.yml` 与根目录 compose 职责重复、路径更绕。  
**Consequence**: 文档和脚本只引用这两个入口，避免三份 compose 并存。

### 2026-06-21: Cloudflare Tunnel 文档

**Decision**: README 中保留简短 Cloudflare Tunnel 小节，说明 Public Hostname 的 service 指向 `http://127.0.0.1:9191`。  
**Reason**: 单容器后路径分流都由 Pebble 自己处理，Tunnel 只需要转发整个站点；这是“不开公网端口”的轮椅部署路线。  
**Consequence**: 不新增复杂 cloudflared 配置文件，先以文档提示为主。

### 2026-06-21: 裸二进制 Release 路线

**Decision**: 删除/停止 GitHub Release 裸二进制发布路线，Docker 单镜像成为主推完整分发方式。  
**Reason**: 本轮不把前端 dist 编译进 Rust 二进制，继续发布裸二进制容易制造“下载即可完整 Web UI”的误解。  
**Consequence**: 需要删除或停用 `.github/workflows/release.yml`，并从 README 中移除裸二进制主推部署说明；如未来需要裸机完整部署，再设计带 dist 的压缩包。

### 2026-06-21: GitHub Actions 边界

**Decision**: 删除 `.github/workflows/release.yml`；保留 `.github/workflows/ci.yml`；改造 `.github/workflows/docker.yml`，只构建/发布单个 `pebble` Docker 镜像。  
**Reason**: CI 仍是质量门禁，Docker 镜像仍是主分发物；只有裸二进制 Release 和前端镜像发布路线需要移除。  
**Consequence**: Docker workflow 中所有 `pebble-frontend` tag、build、manifest 逻辑需要删除。

### 2026-06-21: 安全响应头和 Cookie

**Decision**: Rust 后端接管原 nginx 的基础安全响应头：`X-Content-Type-Options: nosniff`、`X-Frame-Options: DENY`、`Referrer-Policy: no-referrer`、原 CSP 策略；默认不加 HSTS。登录 cookie 继续保持 `Secure=true`、`HttpOnly=true`、`SameSite=Strict`。  
**Reason**: 删除 nginx 后不能让浏览器安全边界倒退；cookie 的 `Secure` 会要求生产通过 HTTPS 或 localhost 访问，这是合理约束。  
**Consequence**: HTTP 域名访问可能登录异常，这是预期行为；文档需引导生产通过 HTTPS/Cloudflare Tunnel/反代访问。

### 2026-06-21: 静态资源缓存策略

**Decision**: 对静态资源做基础缓存区分：`index.html` 和 SPA fallback 使用 `Cache-Control: no-cache`；`/assets/*` 等带 hash 构建产物使用 `Cache-Control: public, max-age=31536000, immutable`。  
**Reason**: SPA 入口像“菜单”，应方便更新；hashed assets 像“带版本号的包裹”，可长期缓存提高性能。  
**Consequence**: 实现需能按路径设置响应头，避免所有资源同一缓存策略。

### 2026-06-21: Service Worker 缓存策略

**Decision**: `/pebble-sw.js` 使用 `Cache-Control: no-cache`，不跟 hashed assets 一样长期缓存。  
**Reason**: Service Worker 是浏览器里的“门卫规则表”，长期缓存会导致推送通知/离线相关逻辑卡旧版本。  
**Consequence**: 静态资源 header 逻辑需单独识别 `/pebble-sw.js`。

### 2026-06-21: SPA fallback 方法边界

**Decision**: 静态文件和 SPA fallback 只处理 GET/HEAD 请求；非 GET/HEAD 的未知路径不返回 `index.html`。  
**Reason**: 页面访问才应回退到 SPA 入口；POST/DELETE 等动作类请求如果没有匹配 API，应明确失败，避免把错误伪装成网页。  
**Consequence**: 实现静态服务时需要保留方法判断或测试覆盖。

## Technical Notes

* 可能改动：`server/src/main.rs`、`server/Cargo.toml`、`deploy/backend.Dockerfile`、`deploy/compose.prod.yml`、`.github/workflows/docker.yml`、`deploy/install.sh`、README 和 docs。
* 可能删除：`deploy/frontend.Dockerfile`、`deploy/nginx.conf`、`deploy/nginx-public.example.conf`，以及重复/过时 compose 文件（待确认）。
* `tower-http` 静态文件服务需要 `fs` feature。
