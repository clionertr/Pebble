# 质量指南

## 安全编码必跑门禁

### 前端
```bash
pnpm audit --audit-level moderate   # 不得有 moderate 及以上漏洞
pnpm test
pnpm run build:frontend
pnpm exec tsc --noEmit
```

### 后端
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

### 部署
```bash
bash -n deploy/install.sh
bash -n deploy/build.sh
```

---

## 安全加固模式

以下模式在第 1 阶段"安全与稳定性止血"中确立，新增代码和 CR 时必须遵守。

### HTML 输出必须转义

**适用场景**：后端直接返回 HTML（如 OAuth 错误页）。

**规则**：所有来自用户输入、外部服务、错误信息的动态内容，在插入 HTML 前必须经 `escape_html()` 转义。

```rust
// 正确（server/src/auth.rs）
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#x27;")
}

Html(format!("<h1>Error</h1><p>{}</p>", escape_html(&message)))
```

### 请求体大小必须限制

**适用场景**：所有文件上传端点。

**规则**：使用 `DefaultBodyLimit` 层设置上限，并在 handler 内做字节级二次检查。

```rust
// server/src/api/attachments.rs
const MAX_ATTACHMENT_SIZE: usize = 25 * 1024 * 1024;

.post(stage_handler)
.layer(DefaultBodyLimit::max(MAX_ATTACHMENT_SIZE))

// handler 内二次兜底
if data.len() > MAX_ATTACHMENT_SIZE {
    return Err(ApiError::bad_request("..."));
}
```

### 分页/查询参数必须有上限

**适用场景**：所有列表查询、搜索端点。

**规则**：定义 `MAX_PAGE_LIMIT` 常量，对所有来自请求的 limit/offset 参数做 `.min(MAX_PAGE_LIMIT)` clamp。

```rust
// server/src/api/messages.rs, threads.rs
const MAX_PAGE_LIMIT: usize = 500;
let limit = query.limit.unwrap_or(50).min(MAX_PAGE_LIMIT) as u32;
```

搜索查询额外限制字符串长度：

```rust
const MAX_SEARCH_QUERY_LEN: usize = 500;
if query.q.len() > MAX_SEARCH_QUERY_LEN {
    return Err(ApiError::bad_request("Search query too long"));
}
```

### 禁止请求可达 `.unwrap()`（见 error-handling.md）

### 禁止静默吞没关键错误（见 error-handling.md）

---

## Docker 构建约定

### Rust 后端
- 使用 `cargo-chef` 分离依赖编译和业务代码编译，保持 Docker layer 可缓存。
- 使用 BuildKit cache mount 缓存 `/usr/local/cargo/registry` 和 `target/`。
- 运行时镜像使用 `debian:bookworm-slim` 等精简基础镜像，只复制最终 `pebble` 二进制。

### 前端
- 使用 `pnpm install --frozen-lockfile` 保证可复现安装。
- Docker 构建阶段使用 pnpm store cache mount。
- 生产镜像只包含 `dist/` 静态文件和 nginx 配置。

---

## CI/CD

- `master` 分支：运行 Webmail 质量门，不触发 Docker 镜像发布。
- `vMAJOR.MINOR.PATCH` tag：发布 Docker 镜像，生成版本号、major/minor、`latest` 和 `sha-*` 标签。
- Docker 镜像构建按镜像和 CPU 架构拆分 BuildKit/GitHub Actions cache scope；优先用原生架构 runner，避免 QEMU 拖慢构建。
- 不构建 Windows/macOS 桌面包；Pebble 当前发布物是后端与前端 Docker 镜像。

---

## nginx 安全配置

**真实 IP 信任范围**：不得使用 `0.0.0.0/0`，应限定为 Docker 私有网络 CIDR。

```nginx
# deploy/nginx.conf — 正确配置
set_real_ip_from 172.16.0.0/12;
set_real_ip_from 10.0.0.0/8;
set_real_ip_from 192.168.0.0/16;
set_real_ip_from 127.0.0.1;
```

如有外部反向代理，在注释中说明如何添加其 IP/CIDR。

---

## .dockerignore 维护清单

新增以下目录时，同步更新 `.dockerignore`：
- `.agent/`, `.claude/`, `.codex/`, `.gemini/`, `.antigravitycli/` — AI/agent 本地配置
- `.trellis/` — Trellis 工作流数据（已在排除列表）
- `data/`, `server/data/` — 运行时数据库和密钥
- `.env` — 本地环境变量

**原则**：所有开发环境配置、运行时数据、密钥文件都不应进入 Docker build context。
