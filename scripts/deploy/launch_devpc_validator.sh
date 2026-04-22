#!/usr/bin/env bash
# Launch BelizeChain testnet validator — Dev PC (Bob)
# Assumes Ceiba validator is already up and reachable at CEIBA_MULTIADDR.
set -euo pipefail

BIN="${BIN:-$HOME/Projects/belizechain-belizechain/target/release/belizechain-node}"
BASE="${BASE:-/tmp/belizechain-devpc}"
NAME="${NAME:-DevPC-Validator-Bob}"
P2P_PORT="${P2P_PORT:-30333}"
RPC_PORT="${RPC_PORT:-9944}"
PROM_PORT="${PROM_PORT:-9615}"

# Ceiba validator's multiaddr for peer bootstrap.
# Set CEIBA_MULTIADDR env before running, e.g.:
#   /ip4/100.81.45.25/tcp/30333/p2p/<CEIBA_PEER_ID>
CEIBA_MULTIADDR="${CEIBA_MULTIADDR:-}"
if [[ -z "$CEIBA_MULTIADDR" ]]; then
    echo "ERROR: set CEIBA_MULTIADDR env var (see Ceiba's startup log for peer id)" >&2
    exit 1
fi

if [[ ! -x "$BIN" ]]; then
    echo "ERROR: binary not found at $BIN" >&2
    exit 1
fi

mkdir -p "$BASE"

exec "$BIN" \
    --chain local \
    --bob \
    --base-path "$BASE" \
    --port "$P2P_PORT" \
    --rpc-port "$RPC_PORT" \
    --prometheus-port "$PROM_PORT" \
    --rpc-cors all \
    --rpc-methods Safe \
    --validator \
    --name "$NAME" \
    --bootnodes "$CEIBA_MULTIADDR" \
    --telemetry-url "" \
    --log info
