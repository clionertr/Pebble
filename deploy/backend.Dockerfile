# 固定到 `node:22-alpine` 的多架构索引 digest。
# 更新 digest 时重新执行：`docker manifest inspect node:22-alpine`。
FROM node@sha256:968df39aedcea65eeb078fb336ed7191baf48f972b4479711397108be0966920 AS frontend-builder
WORKDIR /app
ARG TARGETARCH

# 使用 corepack 安装 pnpm。
RUN corepack enable && corepack prepare pnpm@11.1.1 --activate

# 先复制包描述文件，提升依赖安装缓存命中率。
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# 用 BuildKit cache mount 缓存 pnpm store。
RUN --mount=type=cache,id=pnpm-${TARGETARCH},target=/pnpm/store,sharing=locked \
    PNPM_HOME="/pnpm" pnpm install --frozen-lockfile

# 缓存击穿参数：修改后从这里强制重建。
ARG CACHEBUST=0

# 复制源码并构建前端静态资源。
COPY . .
RUN echo "Frontend build: ${CACHEBUST}" && pnpm run build:frontend

# 使用 cargo-chef 缓存 Rust 依赖。
# 固定到 `lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm` 的多架构索引 digest。
# 更新 digest 时重新执行：`docker manifest inspect lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm`。
FROM lukemathwalker/cargo-chef@sha256:4a51277f4e3e8e4643dd6384f6f6b2b3c8de9f074299cd0c19a80f3c29e8dd15 AS chef
WORKDIR /app

# 安装 Rust 构建依赖。
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        perl \
        make \
        perl-modules && \
    rm -rf /var/lib/apt/lists/*

# 阶段 1：生成 cargo-chef 构建计划。
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 阶段 2：构建依赖和应用。
FROM chef AS builder
ARG TARGETARCH
COPY --from=planner /app/recipe.json recipe.json

# 构建依赖；只要 recipe.json 不变，这一层就可复用。
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=pebble-target-${TARGETARCH},target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# 缓存击穿参数：修改后从这里强制重建。
ARG CACHEBUST=0

# 复制剩余源码。
COPY . .

# 构建后端应用。
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=pebble-target-${TARGETARCH},target=/app/target \
    echo "Build: ${CACHEBUST}" && \
    cargo build --release -p pebble && \
    cp target/release/pebble /app/pebble-bin

# 阶段 3：运行时镜像。
# 固定到 `debian:bookworm-slim` 的多架构索引 digest。
FROM debian@sha256:0104b334637a5f19aa9c983a91b54c89887c0984081f2068983107a6f6c21eeb

# 安装运行时依赖。
RUN apt-get update && \
    apt-get install -y ca-certificates sqlite3 libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制已编译的二进制。
COPY --from=builder /app/pebble-bin /usr/local/bin/pebble

# 复制由 Rust 后端托管的前端构建产物。
COPY --from=frontend-builder /app/dist /app/dist

# 数据卷。
VOLUME ["/app/data"]

EXPOSE 3000

CMD ["pebble"]
