#!/usr/bin/env bash
# CRYPTO-002: Runtime WASM integrity verification.
# Computes the blake2b-256 hash of the compiled runtime blob and compares
# it against a committed baseline. Run this in CI after every runtime build
# to detect unauthorised or accidental modifications.
#
# Usage:
#   ./scripts/verify-wasm-hash.sh            # compare against baseline
#   ./scripts/verify-wasm-hash.sh --update   # write a new baseline
#
# Exit codes:
#   0  match (or successful baseline update)
#   1  mismatch or missing file

set -euo pipefail

WASM_PATH="target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm"
BASELINE_PATH="scripts/wasm-hash.baseline"

# b2sum is part of coreutils; fall back to openssl if not available.
if command -v b2sum &>/dev/null; then
    compute_hash() { b2sum -l 256 "$1" | awk '{print $1}'; }
elif command -v openssl &>/dev/null; then
    compute_hash() { openssl dgst -blake2b256 "$1" | awk '{print $NF}'; }
else
    echo "ERROR: neither b2sum nor openssl found. Cannot compute BLAKE2b-256 hash." >&2
    exit 1
fi

if [[ ! -f "$WASM_PATH" ]]; then
    echo "ERROR: Runtime WASM not found at $WASM_PATH" >&2
    echo "       Run 'cargo build --release' first." >&2
    exit 1
fi

CURRENT_HASH=$(compute_hash "$WASM_PATH")

if [[ "${1:-}" == "--update" ]]; then
    echo "$CURRENT_HASH" > "$BASELINE_PATH"
    echo "CRYPTO-002: Baseline updated → $CURRENT_HASH"
    exit 0
fi

if [[ ! -f "$BASELINE_PATH" ]]; then
    echo "ERROR: No baseline found at $BASELINE_PATH" >&2
    echo "       Run '$0 --update' after a known-good build to create one." >&2
    exit 1
fi

EXPECTED_HASH=$(cat "$BASELINE_PATH" | tr -d '[:space:]')

if [[ "$CURRENT_HASH" == "$EXPECTED_HASH" ]]; then
    echo "CRYPTO-002: WASM hash OK — $CURRENT_HASH"
    exit 0
else
    echo "CRYPTO-002: WASM HASH MISMATCH" >&2
    echo "  expected : $EXPECTED_HASH" >&2
    echo "  actual   : $CURRENT_HASH" >&2
    echo "  file     : $WASM_PATH" >&2
    echo "" >&2
    echo "The runtime blob does not match the committed baseline." >&2
    echo "Re-build from a clean tree and re-run, or update the baseline if this change is intentional." >&2
    exit 1
fi
