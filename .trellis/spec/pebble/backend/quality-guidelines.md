# 质量指南

## Docker 构建约定

### Rust 后端
- 使用 `cargo-chef` 分离依赖编译和业务代码编译，保持 Docker layer 可缓存。
- 使用 BuildKit cache mount 缓存 `/usr/local/cargo/registry` 和 `target/`。
- 运行时镜像使用 `debian:bookworm-slim` 等精简基础镜像，只复制最终 `pebble` 二进制。

### 前端
- 使用 `pnpm install --frozen-lockfile` 保证可复现安装。
- Docker 构建阶段使用 pnpm store cache mount。
- 生产镜像只包含 `dist/` 静态文件和 nginx 配置。

## CI/CD

- `master` 分支：运行 Webmail 质量门，不触发 Docker 镜像发布。
- `vMAJOR.MINOR.PATCH` tag：发布 Docker 镜像，生成版本号、major/minor、`latest` 和 `sha-*` 标签。
- Docker 镜像构建按镜像和 CPU 架构拆分 BuildKit/GitHub Actions cache scope；优先用原生架构 runner，避免 QEMU 拖慢构建。
- 不构建 Windows/macOS 桌面包；Pebble 当前发布物是后端与前端 Docker 镜像。

## 必跑门禁

```bash
pnpm test
pnpm run build:frontend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```
