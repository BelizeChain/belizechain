#!/usr/bin/env bash
#
# Build a deployable runtime blob for a stated target and PROVE its BABE epoch
# configuration before it can be handed to anyone.
#
# Why this exists
# ---------------
# `BabeEpochDuration` is a *compile-time* constant:
#
#     300    with `--features testnet-fast-epoch`
#     14_400 without it
#
# Deploying a blob built without the feature onto a fast-epoch chain changes the
# live epoch length and permanently halts block import. That is exactly how a
# previous chain died. Version monotonicity does NOT protect you: `set_code`
# rejects a *stale* blob but happily accepts one with the *wrong features*, and
# the failure only surfaces one epoch boundary later.
#
# This script removes the footgun in two ways:
#   1. It is the only supported way to produce a deployable blob, and it applies
#      the feature set for the declared target rather than trusting a hand-typed
#      `cargo build` line.
#   2. It boots the resulting blob in a throwaway chain and reads the live epoch
#      duration back, so a mis-built blob fails here instead of on a real chain.
#
# Usage:
#   build-runtime-blob.sh <testnet|mainnet> [--skip-verify]
#
# Exit codes:
#   0  blob built and (unless skipped) epoch configuration verified
#   1  build failed, or the blob's epoch duration does not match the target

set -euo pipefail

TARGET="${1:-}"
SKIP_VERIFY=0
[[ "${2:-}" == "--skip-verify" ]] && SKIP_VERIFY=1

if [[ "$TARGET" != "testnet" && "$TARGET" != "mainnet" ]]; then
    echo "usage: $0 <testnet|mainnet> [--skip-verify]" >&2
    exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Expected epoch duration is the whole point of declaring a target.
if [[ "$TARGET" == "testnet" ]]; then
    FEATURES=(--features testnet-fast-epoch)
    EXPECTED_EPOCH=300
else
    FEATURES=()
    EXPECTED_EPOCH=14400
fi

echo "=== BelizeChain runtime blob build ==="
echo "target           : $TARGET"
echo "features         : ${FEATURES[*]:-<none>}"
echo "expected epoch   : $EXPECTED_EPOCH slots"
echo

WASM_PATH="target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm"
NODE_BIN="target/release/belizechain-node"

echo "--- building runtime ---"
cargo build --release -p belizechain-runtime "${FEATURES[@]}"

if [[ ! -f "$WASM_PATH" ]]; then
    echo "ERROR: runtime WASM not found at $WASM_PATH" >&2
    exit 1
fi

if command -v b2sum &>/dev/null; then
    WASM_HASH="$(b2sum -l 256 "$WASM_PATH" | awk '{print $1}')"
else
    WASM_HASH="$(openssl dgst -blake2b256 "$WASM_PATH" | awk '{print $NF}')"
fi
WASM_SIZE="$(stat -c%s "$WASM_PATH")"

echo
echo "wasm             : $WASM_PATH"
echo "size             : $WASM_SIZE bytes"
echo "blake2b-256      : $WASM_HASH"

if [[ "$SKIP_VERIFY" == "1" ]]; then
    echo
    echo "WARNING: --skip-verify set. The epoch configuration of this blob has NOT"
    echo "         been proven. Do not deploy it to a live chain without running"
    echo "         this script again with verification enabled."
    exit 0
fi

# ---------------------------------------------------------------------------
# Prove the epoch configuration by booting the blob in a throwaway chain.
# ---------------------------------------------------------------------------
if [[ ! -x "$NODE_BIN" ]]; then
    echo
    echo "ERROR: $NODE_BIN not found, so the blob cannot be verified." >&2
    echo "       Build the node first:" >&2
    echo "         cargo build --release -p belizechain-node" >&2
    echo "       Then re-run this script. Verification is not optional: an" >&2
    echo "       unverified blob is exactly how a chain gets halted." >&2
    exit 1
fi

TMP_DIR="$(mktemp -d)"
RPC_PORT=19944
NODE_PID=""

cleanup() {
    if [[ -n "$NODE_PID" ]] && kill -0 "$NODE_PID" 2>/dev/null; then
        kill "$NODE_PID" 2>/dev/null || true
        wait "$NODE_PID" 2>/dev/null || true
    fi
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo
echo "--- verifying epoch configuration (throwaway chain) ---"

# Embed the freshly built blob into a spec so the chain is *born* with it.
if [[ -f testnet-spec.json ]]; then
    python3 scripts/set-spec-code.py testnet-spec.json "$WASM_PATH" -o "$TMP_DIR/spec.json"
else
    "$NODE_BIN" build-spec --chain dev --disable-default-bootnode > "$TMP_DIR/spec.json"
    python3 scripts/set-spec-code.py "$TMP_DIR/spec.json" "$WASM_PATH" -o "$TMP_DIR/spec.json"
fi

"$NODE_BIN" --chain "$TMP_DIR/spec.json" --tmp \
    --rpc-port "$RPC_PORT" --no-telemetry --no-prometheus \
    >"$TMP_DIR/node.log" 2>&1 &
NODE_PID=$!

# Wait for RPC to answer, then verify.
VERIFIED=0
for _ in $(seq 1 40); do
    if ! kill -0 "$NODE_PID" 2>/dev/null; then
        echo "ERROR: node exited during boot; see log below." >&2
        tail -30 "$TMP_DIR/node.log" >&2
        exit 1
    fi
    if python3 scripts/verify_epoch_config.py "http://127.0.0.1:$RPC_PORT" "$EXPECTED_EPOCH" \
        >"$TMP_DIR/verify.log" 2>&1; then
        VERIFIED=1
        break
    fi
    sleep 3
done

if [[ "$VERIFIED" != "1" ]]; then
    echo "ERROR: epoch configuration did NOT verify for target '$TARGET'." >&2
    echo >&2
    cat "$TMP_DIR/verify.log" >&2 || true
    echo >&2
    echo "The blob at $WASM_PATH is NOT deployable. Most likely cause: it was" >&2
    echo "built without the correct feature set for '$TARGET'." >&2
    exit 1
fi

cat "$TMP_DIR/verify.log"

echo
echo "PASS  $TARGET blob verified: $WASM_SIZE bytes, blake2b-256 $WASM_HASH"
echo
echo "Deploy with:"
echo "  python3 scripts/upgrade-runtime.py --rpc <ws-url> --wasm $WASM_PATH"
