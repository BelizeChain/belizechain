#!/usr/bin/env bash
# Launch BelizeChain testnet validator — Ceiba (Alice)
# Usage: sudo -u wicked bash launch_ceiba_validator.sh
set -euo pipefail

BIN="${BIN:-/usr/local/bin/belizechain-node}"
BASE="${BASE:-/data/chain}"
NAME="${NAME:-Ceiba-Validator-Alice}"
P2P_PORT="${P2P_PORT:-30333}"
RPC_PORT="${RPC_PORT:-9944}"
PROM_PORT="${PROM_PORT:-9615}"

if [[ ! -x "$BIN" ]]; then
    echo "ERROR: binary not found at $BIN" >&2
    exit 1
fi

# Pre-flight: ensure port is free
if ss -tlnp 2>/dev/null | grep -q ":$RPC_PORT "; then
    echo "ERROR: port $RPC_PORT already in use. Stop the --dev node first." >&2
    exit 1
fi

mkdir -p "$BASE"

exec "$BIN" \
    --chain local \
    --alice \
    --base-path "$BASE" \
    --port "$P2P_PORT" \
    --rpc-port "$RPC_PORT" \
    --prometheus-port "$PROM_PORT" \
    --prometheus-external \
    --rpc-cors all \
    --rpc-external \
    --rpc-methods Safe \
    --validator \
    --name "$NAME" \
    --node-key-file "$BASE/node-key" \
    --telemetry-url "" \
    --log info
