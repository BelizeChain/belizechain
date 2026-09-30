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
#   scripts/generate-testnet-spec.sh --check              # read-only drift report
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
CHECK=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --verify) VERIFY=1; shift ;;
        --check)  CHECK=1; shift ;;
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
# --check is a report, not a build step: it must work on a machine with no
# build artefacts, so a missing wasm is reported rather than fatal.
if [[ "${CHECK}" != "1" ]]; then
    [[ -f "${WASM}" ]] || fail "runtime wasm not found: ${WASM}
Build one first: cargo build --release -p belizechain-runtime"
fi

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

# ── Drift report. Read-only: it answers "would a fresh chain boot at the version
# the sources declare?" without touching the spec.
#
# This is the check that was missing when the live spec sat at 105 while the repo
# was at 107. Note that a spec legitimately lags right after a live hot upgrade -
# the chain upgrades forward, the embedded genesis runtime does not. That case
# reports the same numbers and the same remedy; the point is that it can no
# longer go unnoticed.
if [[ "${CHECK}" = "1" ]]; then
    python3 - "${OUT_SPEC}" "${WASM}" "${DECLARED:-unknown}" <<'PY'
import hashlib, json, struct, subprocess, sys, tempfile

spec_path, wasm_path, declared = sys.argv[1], sys.argv[2], sys.argv[3]


def blake(data: bytes) -> str:
    return hashlib.blake2b(data, digest_size=32).hexdigest()


def leb(buf, off):
    result = shift = 0
    while True:
        byte = buf[off]
        off += 1
        result |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return result, off
        shift += 7


def compact(buf, off):
    first = buf[off]
    mode = first & 3
    if mode == 0:
        return first >> 2, off + 1
    if mode == 1:
        return ((first | (buf[off + 1] << 8)) >> 2), off + 2
    return (struct.unpack("<I", buf[off:off + 4])[0] >> 2), off + 4


def cstr(buf, off):
    size, off = compact(buf, off)
    return buf[off:off + size].decode(), off + size


def runtime_version(wasm: bytes):
    """Read spec_version from the `runtime_version` custom section."""
    off = 8
    while off < len(wasm):
        section_id = wasm[off]
        off += 1
        size, off = leb(wasm, off)
        body = off
        if section_id == 0:
            name_len, name_off = leb(wasm, off)
            if wasm[name_off:name_off + name_len].decode(errors="replace") == "runtime_version":
                _, cursor = cstr(wasm, name_off + name_len)   # spec_name
                _, cursor = cstr(wasm, cursor)                # impl_name
                _, spec_version, _ = struct.unpack("<III", wasm[cursor:cursor + 12])
                return spec_version
        off = body + size
    return None


def embedded_runtime(spec_file, scratch):
    """Decompress the substrate blob embedded under genesis.runtimeGenesis.code."""
    spec = json.load(open(spec_file))
    raw = spec.get("genesis", {}).get("runtimeGenesis", {}).get("code", "") or ""
    if not raw.startswith("0x"):
        return None, None, "no embedded runtime (raw or empty spec)"
    blob = bytes.fromhex(raw[2:])
    # Substrate's compact-compressed wasm = 8-byte header + zstd frame.
    with open(f"{scratch}.zst", "wb") as handle:
        handle.write(blob[8:] if blob[:4] == bytes.fromhex("52bc5376") else blob)
    result = subprocess.run(
        ["zstd", "-d", "-f", "-q", "-o", scratch, f"{scratch}.zst"],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        return blob, None, result.stderr.strip()[:120] or "zstd failed"
    with open(scratch, "rb") as handle:
        return blob, handle.read(), None


with tempfile.NamedTemporaryFile(suffix=".wasm", delete=False) as tmp:
    scratch = tmp.name

blob, wasm, error = embedded_runtime(spec_path, scratch)
if blob is None:
    print(f"check        : {error}")
    sys.exit(0)

print(f"embedded     : {len(blob)} bytes  blake2b {blake(blob)}")
if error:
    print(f"check        : could not decompress the embedded runtime ({error})")
    sys.exit(0)

embedded_spec = runtime_version(wasm)
print(f"             : spec_version {embedded_spec} (from the embedded runtime)")

current = False
try:
    with open(wasm_path, "rb") as handle:
        built = handle.read()
    print(f"built wasm   : {len(built)} bytes  blake2b {blake(built)}")
    current = built == blob
except OSError:
    print(f"built wasm   : not found at {wasm_path} - cannot compare hashes")

if current:
    print()
    print(f"IN SYNC: the spec embeds the current build (spec {embedded_spec}).")
    sys.exit(0)

print()
print("DRIFT: the spec does NOT embed the current build.")
print(f"  spec embeds       : spec {embedded_spec}")
print(f"  sources declare   : spec {declared}")
print("  A fresh chain would be BORN at the embedded version and would need")
print("  an upgrade to catch up - which is how the 2026-09-21 spec-105/107")
print("  drift shipped unnoticed.")
if str(embedded_spec) == str(declared):
    print()
    print("  The versions agree even though the bytes differ, so this is likely a")
    print("  rebuild of the same version rather than a behaviour change.")
print()
print("  Remedy: re-embed and prove it boots")
print("      scripts/generate-testnet-spec.sh --verify")
print("  (Regenerate the spec only if a fresh chain at this version is what you")
print("   want; a re-genesis would then boot at whatever the spec embeds.)")
sys.exit(1)
PY
    exit $?
fi

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
