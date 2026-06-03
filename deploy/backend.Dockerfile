# Use cargo-chef for dependency caching
# Pinned to multi-arch index digest of `lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm`.
# Update digest by re-fetching: `docker manifest inspect lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm`.
FROM lukemathwalker/cargo-chef@sha256:4a51277f4e3e8e4643dd6384f6f6b2b3c8de9f074299cd0c19a80f3c29e8dd15 AS chef
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        perl \
        make \
        perl-modules && \
    rm -rf /var/lib/apt/lists/*

# Stage 1: Plan the build
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies and the application
FROM chef AS builder
ARG TARGETARCH
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer is cached as long as recipe.json doesn't change.
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=pebble-target-${TARGETARCH},target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Cache-busting arg: change to force rebuild from this point
ARG CACHEBUST=0

# Copy the rest of the source code
COPY . .

# Build the application
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=pebble-target-${TARGETARCH},target=/app/target \
    echo "Build: ${CACHEBUST}" && \
    cargo build --release -p pebble && \
    cp target/release/pebble /app/pebble-bin

# Stage 3: Runtime
# Pinned to multi-arch index digest of `debian:bookworm-slim`.
FROM debian@sha256:0104b334637a5f19aa9c983a91b54c89887c0984081f2068983107a6f6c21eeb

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates sqlite3 libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/pebble-bin /usr/local/bin/pebble

# Volume for data
VOLUME ["/app/data"]

EXPOSE 3000

CMD ["pebble"]
