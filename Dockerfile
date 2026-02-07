# ─────────────────────────────────────────────────────────
# BelizeChain Node — Multi-stage Production Dockerfile
# ─────────────────────────────────────────────────────────

# Stage 1: Build the node binary
FROM paritytech/ci-linux:production AS builder

WORKDIR /build

# Copy entire workspace (filtered by .dockerignore)
COPY . .

# Build optimised release binary
RUN cargo build --release --package belizechain-node && \
    # Strip debug symbols to shrink binary (~50 %)
    strip /build/target/release/belizechain-node

# Optional: export a raw chain spec for testnet inside the image
RUN /build/target/release/belizechain-node build-spec \
    --chain local --raw --disable-default-bootnode \
    > /build/testnet-chain-spec-raw.json


# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

LABEL maintainer="BelizeChain Core Team <dev@belizechain.org>" \
      org.opencontainers.image.source="https://github.com/BelizeChain/belizechain" \
      org.opencontainers.image.description="BelizeChain sovereign blockchain node"

# Runtime dependencies only
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary and chain spec
COPY --from=builder /build/target/release/belizechain-node /usr/local/bin/belizechain-node
COPY --from=builder /build/testnet-chain-spec-raw.json      /etc/belizechain/testnet-chain-spec-raw.json

# Non-root user
RUN useradd -m -u 1000 -U -s /bin/sh -d /data belizechain && \
    mkdir -p /data /etc/belizechain && \
    chown -R belizechain:belizechain /data /etc/belizechain

USER belizechain

# P2P | RPC (legacy) | RPC (unified) | Prometheus
EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -sf -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
        http://localhost:9944 || exit 1

ENTRYPOINT ["belizechain-node"]
