FROM rust:slim-bookworm AS builder
WORKDIR /usr/src/app

# Install dependencies required for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Build the pebble release binary
RUN cargo build --release -p pebble

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (sqlite3 and libssl for Axum/Reqwest)
RUN apt-get update && \
    apt-get install -y ca-certificates sqlite3 libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/pebble /usr/local/bin/pebble

# Volume for database, tantivy index, and keys
VOLUME ["/app/data"]

EXPOSE 3000

CMD ["pebble"]
