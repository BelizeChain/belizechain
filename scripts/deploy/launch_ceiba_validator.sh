#!/usr/bin/env bash
# Launch BelizeChain public-testnet validator on Ceiba.
# Usage: sudo -u wicked bash launch_ceiba_validator.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHAIN_SPEC_VALIDATOR="$SCRIPT_DIR/validate_chain_spec.sh"

BIN="${BIN:-/usr/local/bin/belizechain-node}"
CHAIN_SPEC="${CHAIN_SPEC:-/etc/belizechain/testnet-chain-spec-raw.json}"
BASE="${BASE:-/data/chain}"
NAME="${NAME:-Ceiba-Validator}"
P2P_PORT="${P2P_PORT:-30333}"
RPC_PORT="${RPC_PORT:-9944}"
PROM_PORT="${PROM_PORT:-9615}"

if [[ ! -x "$BIN" ]]; then
    echo "ERROR: binary not found at $BIN" >&2
    exit 1
fi

if [[ ! -f "$CHAIN_SPEC" ]]; then
    echo "ERROR: chain spec not found at $CHAIN_SPEC" >&2
    exit 1
fi

if [[ -f "$CHAIN_SPEC_VALIDATOR" ]]; then
    bash "$CHAIN_SPEC_VALIDATOR" "$CHAIN_SPEC"
else
    if grep -Eq '"id"[[:space:]]*:[[:space:]]*"belizechain_local"|"chainType"[[:space:]]*:[[:space:]]*"Local"' "$CHAIN_SPEC"; then
        echo "ERROR: chain spec still resolves to a local chain: $CHAIN_SPEC" >&2
        exit 1
    fi

    if grep -Eq '/ip4/(127\.0\.0\.1|0\.0\.0\.0)/|/dns4/localhost/' "$CHAIN_SPEC"; then
        echo "ERROR: chain spec still contains loopback bootnodes: $CHAIN_SPEC" >&2
        exit 1
    fi
fi

# Pre-flight: ensure port is free
if ss -tlnp 2>/dev/null | grep -q ":$RPC_PORT "; then
    echo "ERROR: port $RPC_PORT already in use. Stop the current node first." >&2
    exit 1
fi

mkdir -p "$BASE"

exec "$BIN" \
    --chain "$CHAIN_SPEC" \
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
