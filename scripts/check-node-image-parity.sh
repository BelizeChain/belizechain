#!/usr/bin/env bash
# ==============================================================================
# Node image parity guard
# ==============================================================================
# Fails if a deployed node image was built BEFORE the newest change to the
# client surface (`node/`), and warns if it predates the newest runtime spec
# bump.
#
# Why this exists
# ---------------
# A runtime upgrade ships as `sudo.sudo(System::set_code)` and does NOT need a
# new node image. That is correct and deliberate. But the JSON-RPC *servers* and
# the host functions live in the node binary, so a client change does need one,
# and nothing linked the two facts. Ceiba therefore ran `76dea3f` (built
# 2026-10-19, native spec 116) against a spec-124 chain for two days, and every
# `belizechain_*` RPC returned `-32601 Method not found` while every ordinary
# health check reported success.
#
# It was invisible because `state_getRuntimeVersion` reports the *chain's* WASM
# blob, not the binary's embedded runtime - the obvious check is structurally
# incapable of showing this drift. It is also not the first time: the spec
# 105 -> 107 upgrade left the same follow-up filed and unowned in
# `docs/operations/RUNTIME_UPGRADE_105_TO_107_2026-09-20.md`.
#
# What is actually checked
# ------------------------
# Since polkadot-sdk stable2609 the executor is WASM-only (`sc-executor` exports
# no `NativeExecutor`, and `--execution` is a deprecated no-op), so the binary's
# embedded runtime is never executed on a live chain. The embedded spec is
# therefore NOT the risk, and comparing it to the chain's spec would be a false
# signal:
#
#   * HARD FAIL - the image must descend from the newest commit touching `node/`.
#     That is the client surface: RPC servers, host functions, service wiring.
#     A binary older than it is missing exactly the code that was added.
#   * WARN - the image should descend from the newest `runtime/src/lib.rs`
#     commit. A stale embedded runtime does not break a live chain, but it
#     misreports its spec under `--dev` and would re-genesis at the wrong one.
#
# The risk this prevents from becoming real: a runtime that needs a host
# function, inherent or digest the old binary does not provide does not fail
# with a tidy `-32601`. It fails to instantiate, which is a consensus halt.
#
# Usage
# -----
#   ./scripts/check-node-image-parity.sh --tag 01d51bb-spec124-20261027
#   ./scripts/check-node-image-parity.sh --sha 01d51bb
#   ./scripts/check-node-image-parity.sh --remote ceiba      # read the live pin
#
# REQUIRES FULL GIT HISTORY. Under a shallow clone `git merge-base` cannot
# answer, so this fails loudly rather than reporting a false pass.
# ==============================================================================

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

IMAGE_REF=""
REMOTE_HOST=""
REMOTE_ENV="/opt/belizechain/.env"
PROVE_WASM=""

usage() {
    cat <<'USAGE'
Usage:
  ./scripts/check-node-image-parity.sh --tag 01d51bb-spec124-20261027
  ./scripts/check-node-image-parity.sh --sha 01d51bb
  ./scripts/check-node-image-parity.sh --remote ceiba      # read the live pin

Options:
  --wasm <blob.wasm>   also prove the image can instantiate this runtime, by
                       checking every host function it imports is registered in
                       the image's binary. Uncompressed module, not the
                       .compact.compressed.wasm envelope.
USAGE
    exit "${1:-0}"
}

# Extracts the first `spec_version: N` from Rust source on stdin. Reads the
# whole stream deliberately: an early exit (grep -m1, head -1) SIGPIPEs the
# upstream `git show`, and under `pipefail` that turns a successful read into a
# failure.
spec_version_of() {
    awk '
        found == "" && /spec_version:/ {
            line = $0
            sub(/.*spec_version:[[:space:]]*/, "", line)
            sub(/[^0-9].*/, "", line)
            found = line
        }
        END { print (found == "" ? "?" : found) }
    '
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --tag)    IMAGE_REF="${2:?--tag needs a value}"; shift 2 ;;
        --sha)    IMAGE_REF="${2:?--sha needs a value}"; shift 2 ;;
        --remote) REMOTE_HOST="${2:?--remote needs a host}"; shift 2 ;;
        --wasm)   PROVE_WASM="${2:?--wasm needs a value}"; shift 2 ;;
        -h|--help) usage 0 ;;
        *) echo "unknown argument: $1" >&2; usage 1 ;;
    esac
done

if [[ -f "$(git rev-parse --git-dir)/shallow" ]]; then
    echo "❌ shallow clone detected - this check needs full history." >&2
    echo "   Fetch it with: git fetch --unshallow" >&2
    exit 1
fi

# ---- Resolve the image reference -------------------------------------------

if [[ -z "$IMAGE_REF" && -n "$REMOTE_HOST" ]]; then
    IMAGE_REF="$(ssh -o BatchMode=yes "$REMOTE_HOST" \
        "grep -m1 '^CEIBA_NODE_IMAGE=' '$REMOTE_ENV'" </dev/null 2>/dev/null \
        | cut -d= -f2- | tr -d '"'"'"' \r')" || true
    if [[ -z "$IMAGE_REF" ]]; then
        echo "❌ could not read CEIBA_NODE_IMAGE from ${REMOTE_HOST}:${REMOTE_ENV}" >&2
        exit 1
    fi
    echo "Read live pin from ${REMOTE_HOST}: ${IMAGE_REF}"
fi

if [[ -z "$IMAGE_REF" ]]; then
    usage 1
fi

# Tags are `<sha>-<free text>`, e.g. `01d51bb-spec124-20261027`. Accept a bare
# tag name, a fully-qualified `repo:<tag>`, or a raw sha.
IMAGE_TAG="${IMAGE_REF##*:}"
IMAGE_SHA="$(printf '%s' "$IMAGE_TAG" | grep -oE '^[0-9a-f]{7,40}' || true)"

if [[ -z "$IMAGE_SHA" ]]; then
    echo "❌ cannot find a commit sha at the start of '${IMAGE_TAG}'." >&2
    echo "   Image tags must begin with the short sha they were built from." >&2
    exit 1
fi

# Fail closed: an unknown sha means the question cannot be answered, and a
# silent pass here is the failure mode this script exists to prevent.
if ! IMAGE_COMMIT="$(git rev-parse --verify --quiet "${IMAGE_SHA}^{commit}")"; then
    echo "❌ commit ${IMAGE_SHA} (from tag '${IMAGE_TAG}') is not in this clone." >&2
    echo "   Fetch it before trusting this image, or pass --sha explicitly." >&2
    exit 1
fi

IMAGE_SHORT="$(git rev-parse --short "$IMAGE_COMMIT")"
EMBEDDED_SPEC="$(git show "${IMAGE_COMMIT}:runtime/src/lib.rs" 2>/dev/null \
    | spec_version_of)" || EMBEDDED_SPEC="?"

echo "Image   : ${IMAGE_TAG}"
echo "Commit  : ${IMAGE_SHORT}  ($(git log -1 --format=%cs "$IMAGE_COMMIT"))"
echo "Embedded: spec ${EMBEDDED_SPEC}"

FAIL=0

# ---- HARD: the image must include the newest client surface ----------------

NODE_COMMIT="$(git log -1 --format=%h -- node/ 2>/dev/null || true)"
if [[ -z "$NODE_COMMIT" ]]; then
    echo "❌ no history for node/ - cannot verify the client surface." >&2
    exit 1
fi
NODE_DESC="$(git log -1 --format='%h %cs %s' -- node/ | cut -c1-90)"

if git merge-base --is-ancestor "$NODE_COMMIT" "$IMAGE_COMMIT" 2>/dev/null; then
    echo "✅ client surface: image contains node/${NODE_COMMIT}"
else
    echo "❌ client surface: newest node/ change is ${NODE_DESC}"
    echo "   ...but the image was built before it, so it lacks that code."
    FAIL=1
fi

# ---- WARN: the embedded runtime should not be stale ------------------------

RUNTIME_COMMIT="$(git log -1 --format=%h -- runtime/src/lib.rs 2>/dev/null || true)"
CURRENT_SPEC="$(spec_version_of < runtime/src/lib.rs)"

if [[ -n "$RUNTIME_COMMIT" ]] \
    && git merge-base --is-ancestor "$RUNTIME_COMMIT" "$IMAGE_COMMIT" 2>/dev/null; then
    echo "✅ embedded runtime: image contains runtime/${RUNTIME_COMMIT} (spec ${CURRENT_SPEC})"
else
    echo "⚠️  embedded runtime: image embeds spec ${EMBEDDED_SPEC}, tree is at ${CURRENT_SPEC}"
    echo "   Newest runtime commit is ${RUNTIME_COMMIT}, which the image predates."
    echo "   Harmless on a live chain - execution is WASM-only - but the image"
    echo "   misreports its spec under --dev and would re-genesis at the old one."
fi

# ---- Prove the image can actually instantiate that runtime -----------------
#
# Ancestry says the source is in the image. It does not say the image can RUN a
# given blob: a runtime that imports a host function the client does not
# register fails at WASM instantiation and stops producing blocks, which no
# amount of ancestry checking can see. That check needs both artifacts in one
# place, so the blob is shipped to wherever the image lives.

if [[ -n "$PROVE_WASM" ]]; then
    if [[ ! -f "$PROVE_WASM" ]]; then
        echo "❌ --wasm ${PROVE_WASM}: no such file" >&2
        exit 1
    fi
    CHECKER="${ROOT_DIR}/scripts/check-runtime-host-functions.py"

    if [[ -n "$REMOTE_HOST" ]]; then
        remote_dir="/tmp/bz-hostfn-$$"
        if ! ssh -o BatchMode=yes "$REMOTE_HOST" "mkdir -p '$remote_dir'" </dev/null; then
            echo "❌ could not create ${remote_dir} on ${REMOTE_HOST}" >&2
            exit 1
        fi
        # shellcheck disable=SC2064  # expand remote_dir now, at trap definition
        trap "ssh -o BatchMode=yes '$REMOTE_HOST' 'rm -rf $remote_dir' </dev/null >/dev/null 2>&1 || true" EXIT
        scp -q -o BatchMode=yes "$CHECKER" "$PROVE_WASM" "${REMOTE_HOST}:${remote_dir}/" </dev/null
        if ssh -o BatchMode=yes "$REMOTE_HOST" \
            "cd '$remote_dir' && python3 $(basename "$CHECKER") \
                --wasm $(basename "$PROVE_WASM") --image '$IMAGE_REF'" </dev/null; then
            echo "✅ instantiation: the image registers every host function the runtime imports"
        else
            echo "❌ instantiation NOT proven - see the reason above" >&2
            FAIL=1
        fi
    else
        if python3 "$CHECKER" --wasm "$PROVE_WASM" --image "$IMAGE_REF"; then
            echo "✅ instantiation: the image registers every host function the runtime imports"
        else
            echo "❌ instantiation NOT proven - see the reason above" >&2
            FAIL=1
        fi
    fi
else
    echo "⏭️  instantiation: not checked (pass --wasm <blob.wasm> to prove it)"
fi

# ---- Verdict ---------------------------------------------------------------

if [[ "$FAIL" -ne 0 ]]; then
    echo "" >&2
    echo "❌ This image predates the client code it is meant to ship." >&2
    echo "   Rebuild it on the host with the testnet epoch feature, then verify" >&2
    echo "   the result BEFORE moving the pin:" >&2
    echo "" >&2
    echo "     docker build --build-arg CARGO_FEATURES=testnet-fast-epoch \\" >&2
    echo "       -t belizechain/ceiba-node:<sha>-<label> ." >&2
    echo "" >&2
    echo "   A blob built without that arg embeds a 14,400-slot epoch and halts" >&2
    echo "   block import - that is how Chain C died on 2026-10-04." >&2
    exit 1
fi

echo "✅ Node image is at parity with the client surface."
