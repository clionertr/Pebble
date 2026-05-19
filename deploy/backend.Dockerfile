# Use cargo-chef for dependency caching
FROM lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm AS chef
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
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
FROM debian:bookworm-slim

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
