# Build Stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    proto-compiler \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY . .

# Touch main.rs to ensure rebuild
RUN touch src/main.rs && cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/axum-leptos-htmx-wc /app/server

# Copy any necessary assets (e.g. static files, config)
# COPY --from=builder /app/assets /app/assets
# COPY --from=builder /app/.env /app/.env
# (We expect .env and assets to be mounted or managed via config)

# Expose port
EXPOSE 3000

# Run the binary
CMD ["/app/server"]
