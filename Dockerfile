# BelizeChain Node Dockerfile
FROM paritytech/ci-linux:production as builder

WORKDIR /belizechain

# Copy workspace files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY belizechain/ ./belizechain/

# Build the blockchain node
RUN cargo build --release --package belizechain-node

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /belizechain/target/release/belizechain-node /usr/local/bin/belizechain-node

# Create data directory
RUN mkdir -p /data && \
    useradd -m -u 1000 -U -s /bin/sh -d /data belizechain && \
    chown -R belizechain:belizechain /data

USER belizechain

EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:9933/health || exit 1

ENTRYPOINT ["/usr/local/bin/belizechain-node"]
