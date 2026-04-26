#!/usr/bin/env bash
# Fail closed if a supplied public-testnet spec still resolves to a local chain.

set -euo pipefail

CHAIN_SPEC_PATH="${1:-}"

if [[ -z "$CHAIN_SPEC_PATH" ]]; then
    echo "ERROR: usage: validate_chain_spec.sh <chain-spec-path>" >&2
    exit 1
fi

if [[ ! -f "$CHAIN_SPEC_PATH" ]]; then
    echo "ERROR: chain spec not found: $CHAIN_SPEC_PATH" >&2
    exit 1
fi

if grep -Eq '"id"[[:space:]]*:[[:space:]]*"belizechain_local"' "$CHAIN_SPEC_PATH"; then
    echo "ERROR: chain spec id is belizechain_local: $CHAIN_SPEC_PATH" >&2
    exit 1
fi

if grep -Eq '"chainType"[[:space:]]*:[[:space:]]*"Local"' "$CHAIN_SPEC_PATH"; then
    echo "ERROR: chain spec still declares chainType Local: $CHAIN_SPEC_PATH" >&2
    exit 1
fi

if grep -Eq '/ip4/(127\.0\.0\.1|0\.0\.0\.0)/|/dns4/localhost/' "$CHAIN_SPEC_PATH"; then
    echo "ERROR: chain spec still contains loopback bootnodes: $CHAIN_SPEC_PATH" >&2
    exit 1
fi

echo "OK: chain spec passes non-local preflight checks"