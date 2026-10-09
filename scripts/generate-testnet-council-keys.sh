#!/usr/bin/env bash
# ==============================================================================
# BelizeChain Testnet Council Bootstrap Key Generator
# ==============================================================================
#
# ⚠️  TESTNET ONLY. These keys are generated on a build machine and their secret
#     phrases are written to disk in plaintext. They MUST NEVER be reused for
#     mainnet, and `mainnet_genesis()` refuses to build while they are still in
#     place (see `assert_council_is_production_grade` in node/src/chain_spec.rs).
#
# Why this exists
# ---------------
# The GovernanceCouncil collective (the origin behind `GovernanceCouncilMajority`
# and therefore treasury burn / reserves / minter authorization) had **no members
# on any chain**. With an empty collective the proportion branch of
# `EnsureProportionMoreThan<_, 1, 2>` can never be satisfied, so every
# "council-majority" path silently degraded to sudo. Documentation claimed
# "4 of 7 seats" for a mechanism that could not execute at all.
#
# The model is 12 seats: 2 per Belize district x 6 districts, matching
# `GovernanceCouncilSize` (12) and the district list already modelled as
# `BelizeDistrict` in pallets/mesh/src/types.rs.
#
# Output: generated_keys/testnet-council-keys.json (mode 600), plus a Rust
# snippet to paste into node/src/chain_spec.rs.
#
# Usage:
#   generate-testnet-council-keys.sh [--force]
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

NODE_BIN="${ROOT_DIR}/target/release/belizechain-node"
if [[ ! -x "${NODE_BIN}" ]]; then
    NODE_BIN="${ROOT_DIR}/target/debug/belizechain-node"
fi
if [[ ! -x "${NODE_BIN}" ]]; then
    echo "ERROR: node binary not found. Build it first:" >&2
    echo "         cargo build --release -p belizechain-node" >&2
    exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "ERROR: jq is required." >&2
    exit 1
fi

FORCE=0
[[ "${1:-}" == "--force" ]] && FORCE=1

OUTPUT_DIR="${ROOT_DIR}/generated_keys"
OUT_JSON="${OUTPUT_DIR}/testnet-council-keys.json"

if [[ -f "${OUT_JSON}" && "${FORCE}" != "1" ]]; then
    echo "ERROR: ${OUT_JSON} already exists." >&2
    echo "       Refusing to overwrite existing key material. Re-run with --force" >&2
    echo "       ONLY if you intend to invalidate the current testnet council." >&2
    exit 1
fi

mkdir -p "${OUTPUT_DIR}"
chmod 700 "${OUTPUT_DIR}"

# 2 seats per district x 6 districts = 12. Order matches `BelizeDistrict`.
DISTRICTS=(Belize Cayo OrangeWalk Corozal StannCreek Toledo)
SEATS_PER_DISTRICT=2

echo "=== BelizeChain testnet council bootstrap ==="
echo "districts        : ${#DISTRICTS[@]}"
echo "seats per district: ${SEATS_PER_DISTRICT}"
echo "total seats      : $(( ${#DISTRICTS[@]} * SEATS_PER_DISTRICT ))"
echo

SEEN_FILE="$(mktemp)"
trap 'rm -f "${SEEN_FILE}"' EXIT

DISTRICTS_JSON="{}"
SEAT_INDEX=0

for district in "${DISTRICTS[@]}"; do
    district_json="[]"
    for ((seat = 1; seat <= SEATS_PER_DISTRICT; seat++)); do
        SEAT_INDEX=$(( SEAT_INDEX + 1 ))

        raw="$("${NODE_BIN}" key generate --scheme sr25519 --output-type json)"
        ss58="$(echo "${raw}" | jq -r '.ss58Address')"
        pubkey="$(echo "${raw}" | jq -r '.publicKey')"
        phrase="$(echo "${raw}" | jq -r '.secretPhrase')"
        seed="$(echo "${raw}" | jq -r '.secretSeed')"

        # Duplicate keys would make the collective's genesis assert panic at
        # build time; catch it here with a clearer message.
        if grep -qx "${ss58}" "${SEEN_FILE}"; then
            echo "ERROR: duplicate key generated (${ss58}); re-run." >&2
            exit 1
        fi
        echo "${ss58}" >> "${SEEN_FILE}"

        label="$(printf '%s-%d' "${district}" "${seat}")"
        district_json="$(jq -n --argjson acc "${district_json}" \
            --arg label "${label}" --arg ss58 "${ss58}" --arg pk "${pubkey}" \
            --arg phrase "${phrase}" --arg seed "${seed}" \
            '$acc + [{ seatLabel: $label, ss58Address: $ss58, publicKey: $pk,
                       secretPhrase: $phrase, secretSeed: $seed }]')"

        printf '  %-16s %s\n' "${label}" "${ss58}"
    done
    DISTRICTS_JSON="$(jq -n --argjson all "${DISTRICTS_JSON}" \
        --arg d "${district}" --argjson seats "${district_json}" \
        '$all + { ($d): $seats }')"
done

jq -n \
    --arg generatedAt "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg purpose "testnet-council-bootstrap" \
    --arg warning "TESTNET ONLY - generated on a build machine, secret phrases stored in plaintext. mainnet_genesis() refuses to build while these are in place." \
    --argjson districts "${DISTRICTS_JSON}" \
    '{
        generatedAt: $generatedAt,
        network: "belizechain-testnet",
        purpose: $purpose,
        warning: $warning,
        model: "12 seats = 2 per district x 6 districts (BelizeDistrict)",
        districts: $districts
    }' > "${OUT_JSON}"
chmod 600 "${OUT_JSON}"

echo
echo "written          : generated_keys/testnet-council-keys.json (mode 600)"

# Emit the const array for node/src/chain_spec.rs.
echo
echo "--- paste into node/src/chain_spec.rs (TESTNET_COUNCIL_BOOTSTRAP) ---"
echo "const TESTNET_COUNCIL_BOOTSTRAP: [&str; ${SEAT_INDEX}] = ["
jq -r '.districts | to_entries[] | .value[] | "    \"\(.ss58Address)\",  // \(.seatLabel)"' "${OUT_JSON}"
echo "];"
echo
echo "Done. ${SEAT_INDEX} seats generated across ${#DISTRICTS[@]} districts."
