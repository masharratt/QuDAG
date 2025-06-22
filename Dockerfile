# Multi-stage build for QuDAG node
# Stage 1: Build environment
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    g++ \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set up working directory
WORKDIR /qudag

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY core/ ./core/
COPY cli-standalone/ ./cli-standalone/
COPY qudag/ ./qudag/
COPY benchmarks/ ./benchmarks/
COPY tools/ ./tools/

# Build release binary with all features
RUN cargo build --release --bin qudag --features "cli full"

# Stage 2: Runtime environment
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash qudag

# Copy binary from builder
COPY --from=builder /qudag/target/release/qudag /usr/local/bin/qudag

# Create data directories
RUN mkdir -p /data /config /keys && \
    chown -R qudag:qudag /data /config /keys

# Switch to non-root user
USER qudag

# Set environment variables
ENV QUDAG_DATA_DIR=/data
ENV QUDAG_CONFIG_DIR=/config
ENV QUDAG_KEY_DIR=/keys
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose ports
# P2P port
EXPOSE 4001
# RPC port
EXPOSE 8080
# Metrics port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD qudag status || exit 1

# Volume mounts
VOLUME ["/data", "/config", "/keys"]

# Default command
ENTRYPOINT ["qudag"]
CMD ["start", "--config", "/config/node.toml"]