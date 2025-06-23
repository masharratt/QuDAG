# Build Stage: Compile QuDAG binaries
FROM rust:1.83 AS builder
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copy entire workspace
COPY . .

# Build both qudag CLI and exchange server in release mode
RUN cargo build --release --bin qudag && \
    cargo build --release --bin qudag-exchange-server

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash qudag

# Copy both binaries from builder
COPY --from=builder /app/target/release/qudag /usr/local/bin/qudag
COPY --from=builder /app/target/release/qudag-exchange-server /usr/local/bin/qudag-exchange-server

# Copy configuration files
COPY configs/*.toml /app/configs/

# Copy startup script
COPY startup.sh /usr/local/bin/startup.sh
RUN chmod +x /usr/local/bin/startup.sh

# Create data directories
RUN mkdir -p /data/qudag && chown -R qudag:qudag /data

# Switch to non-root user
USER qudag

# Set working directory
WORKDIR /data/qudag

# Environment variables for QuDAG configuration
ENV RUST_LOG=info
ENV QUDAG_DATA_DIR=/data/qudag
ENV QUDAG_P2P_PORT=4001
ENV QUDAG_RPC_PORT=8080
ENV QUDAG_METRICS_PORT=9090

# Expose ports
# P2P networking
EXPOSE 4001/tcp
EXPOSE 4001/udp
# RPC API
EXPOSE 8080/tcp
# Metrics
EXPOSE 9090/tcp

# Health check - works for both node types
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:${QUDAG_RPC_PORT:-8080}/health || \
        curl -f http://localhost:${QUDAG_RPC_PORT:-8080}/api/health || \
        exit 1

# Entry point - uses startup script that selects binary based on NODE_TYPE
ENTRYPOINT ["/usr/local/bin/startup.sh"]
CMD ["--config", "/data/qudag/config.toml"]