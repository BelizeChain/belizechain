#!/usr/bin/env bash
# ==============================================================================
# BelizeChain Testnet ChainSpec Builder
# ==============================================================================
# Rewrites the runtime embedded in a non-raw testnet chain spec so a fresh chain
# is BORN at the runtime's current spec version.
#
# Why this exists: a non-raw spec carries the genesis runtime under
# `genesis.runtimeGenesis.code`. Whatever WASM sits there is what a fresh chain
# starts with. On 2026-09-21 the live spec was found pinning the spec-105 runtime
# while the repo was at 107, so every fresh chain booted one version behind and
# needed a second upgrade pass to catch up. The spec is gitignored (operator
# managed, carries live keys), so nothing in CI could notice.
#
# This script closes that gap: it embeds the current runtime, checks the wasm is
# not stale relative to the sources, and — with --verify — actually boots a
# throwaway chain to confirm the spec comes up at the expected version.
#
# Usage:
#   scripts/generate-testnet-spec.sh                      # in-place testnet-spec.json
#   scripts/generate-testnet-spec.sh --verify             # also boot-check it
#   scripts/generate-testnet-spec.sh --base foo.json --out bar.json
#   WASM=path/to/runtime.wasm scripts/generate-testnet-spec.sh
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

NODE_BIN="${ROOT_DIR}/target/release/belizechain-node"
DEFAULT_WASM="${ROOT_DIR}/target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm"
BASE_SPEC="${ROOT_DIR}/testnet-spec.json"
OUT_SPEC=""
VERIFY=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --verify) VERIFY=1; shift ;;
        --base)   BASE_SPEC="$2"; shift 2 ;;
        --out)    OUT_SPEC="$2"; shift 2 ;;
        -h|--help) sed -n '2,28p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

WASM="${WASM:-${DEFAULT_WASM}}"
[[ -n "${OUT_SPEC}" ]] || OUT_SPEC="${BASE_SPEC}"

fail() { echo "ERROR: $*" >&2; exit 1; }

[[ -f "${BASE_SPEC}" ]] || fail "base spec not found: ${BASE_SPEC}"
[[ -f "${WASM}" ]] || fail "runtime wasm not found: ${WASM}
Build one first: cargo build --release -p belizechain-runtime"

# ── Staleness: a wasm older than the sources it claims to represent is the
# exact failure this script exists to prevent.
NEWEST_SRC=$(find "${ROOT_DIR}/runtime/src" "${ROOT_DIR}/pallets" -name '*.rs' -newer "${WASM}" -print -quit 2>/dev/null || true)
if [[ -n "${NEWEST_SRC}" ]]; then
    echo "WARNING: ${NEWEST_SRC#"${ROOT_DIR}/"} is newer than the runtime wasm." >&2
    echo "         The spec would embed a stale runtime. Rebuild before trusting this." >&2
fi

# Spec version the sources declare. The boot check below proves the wasm agrees.
DECLARED=$(grep -oE 'spec_version:[[:space:]]*[0-9]+' "${ROOT_DIR}/runtime/src/lib.rs" | head -1 | grep -oE '[0-9]+' || echo "")
echo "runtime wasm : ${WASM#"${ROOT_DIR}/"}"
echo "declared spec: ${DECLARED:-unknown} (runtime/src/lib.rs)"

echo "embedding runtime..."
python3 "${SCRIPT_DIR}/set-spec-code.py" "${BASE_SPEC}" "${WASM}" -o "${OUT_SPEC}"

EMBEDDED=$(python3 - "${OUT_SPEC}" <<'PY'
import hashlib, json, sys
spec = json.load(open(sys.argv[1]))
code = bytes.fromhex(spec["genesis"]["runtimeGenesis"]["code"][2:])
print(f"{len(code)} {hashlib.blake2b(code, digest_size=32).hexdigest()}")
PY
)
echo "embedded     : ${EMBEDDED}"

if [[ "${VERIFY}" != "1" ]]; then
    echo
    echo "Run with --verify to boot a throwaway chain and confirm the spec comes up"
    echo "at spec ${DECLARED:-?}. That check is what catches a stale embed."
    exit 0
fi

# ── Boot check: prove the spec's genesis runtime is the version we expect.
[[ -x "${NODE_BIN}" ]] || fail "node binary not found: ${NODE_BIN}
Build one first: cargo build --release -p belizechain-node"

RPC_PORT="${VERIFY_RPC_PORT:-19944}"
TMP_BASE="$(mktemp -d)"
NODE_LOG="$(mktemp)"

cleanup() {
    [[ -n "${NODE_PID:-}" ]] && kill "${NODE_PID}" 2>/dev/null || true
    wait "${NODE_PID}" 2>/dev/null || true
    rm -rf "${TMP_BASE}" "${NODE_LOG}"
}
trap cleanup EXIT

echo
echo "booting a throwaway chain from ${OUT_SPEC#"${ROOT_DIR}/"} (rpc ${RPC_PORT})..."
"${NODE_BIN}" --chain "${OUT_SPEC}" --base-path "${TMP_BASE}" \
    --rpc-port "${RPC_PORT}" --no-mdns --no-telemetry --port 0 >"${NODE_LOG}" 2>&1 &
NODE_PID=$!

ACTUAL=""
for _ in $(seq 1 30); do
    sleep 2
    ACTUAL=$(curl -s -m 5 -H 'Content-Type: application/json' \
        -d '{"id":1,"jsonrpc":"2.0","method":"state_getRuntimeVersion","params":[]}' \
        "http://127.0.0.1:${RPC_PORT}" \
        | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["specVersion"])' 2>/dev/null || true)
    [[ -n "${ACTUAL}" ]] && break
done

if [[ -z "${ACTUAL}" ]]; then
    echo "FAILED: node never answered on port ${RPC_PORT}. Last log lines:" >&2
    tail -20 "${NODE_LOG}" >&2
    exit 1
fi

echo "chain reports: spec ${ACTUAL}"

if [[ -n "${DECLARED}" && "${ACTUAL}" != "${DECLARED}" ]]; then
    echo "FAILED: the spec boots at spec ${ACTUAL} but the sources declare ${DECLARED}." >&2
    exit 1
fi

echo "OK: ${OUT_SPEC#"${ROOT_DIR}/"} is born at spec ${ACTUAL}"
