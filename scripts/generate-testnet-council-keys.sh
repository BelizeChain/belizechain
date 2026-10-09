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
# Output:
#   default      -> generated_keys/testnet-council-keys.json   (12 district seats)
#   --technical  -> generated_keys/testnet-technical-keys.json (7 technical seats)
#
# Plus a Rust snippet to paste into node/src/chain_spec.rs.
#
# Usage:
#   generate-testnet-council-keys.sh [--technical] [--force]
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
ROLE="districts"
for arg in "$@"; do
    case "$arg" in
        --force)     FORCE=1 ;;
        --technical) ROLE="technical" ;;
        *) echo "unknown argument: $arg" >&2; exit 2 ;;
    esac
done

OUTPUT_DIR="${ROOT_DIR}/generated_keys"

# Seats are grouped by what each member represents: districts group the
# democratic GovernanceCouncil, functions group the technical body.
if [[ "${ROLE}" == "technical" ]]; then
    CONST_NAME="TESTNET_TECHNICAL_COUNCIL"
    OUT_JSON="${OUTPUT_DIR}/testnet-technical-keys.json"
    MODEL="7 seats across protocol / security / operations"
    # 7 is the smallest size at which all three thresholds stay distinct:
    # majority needs 4, supermajority 5, three-quarters 6.
    SEAT_GROUPS=(Protocol:3 Security:2 Operations:2)
else
    CONST_NAME="TESTNET_COUNCIL_BOOTSTRAP"
    OUT_JSON="${OUTPUT_DIR}/testnet-council-keys.json"
    MODEL="12 seats = 2 per district x 6 districts (BelizeDistrict)"
    SEAT_GROUPS=(Belize:2 Cayo:2 OrangeWalk:2 Corozal:2 StannCreek:2 Toledo:2)
fi

if [[ -f "${OUT_JSON}" && "${FORCE}" != "1" ]]; then
    echo "ERROR: ${OUT_JSON} already exists." >&2
    echo "       Refusing to overwrite existing key material. Re-run with --force" >&2
    echo "       ONLY if you intend to invalidate the current testnet council." >&2
    exit 1
fi

mkdir -p "${OUTPUT_DIR}"
chmod 700 "${OUTPUT_DIR}"

TOTAL_SEATS=0
for entry in "${SEAT_GROUPS[@]}"; do
    TOTAL_SEATS=$(( TOTAL_SEATS + ${entry##*:} ))
done

echo "=== BelizeChain testnet council bootstrap (${ROLE}) ==="
echo "groups           : ${SEAT_GROUPS[*]}"
echo "total seats      : ${TOTAL_SEATS}"
echo

SEEN_FILE="$(mktemp)"
trap 'rm -f "${SEEN_FILE}"' EXIT

GROUPS_JSON="{}"
SEAT_INDEX=0

for entry in "${SEAT_GROUPS[@]}"; do
    group="${entry%%:*}"
    count="${entry##*:}"
    group_json="[]"
    for ((seat = 1; seat <= count; seat++)); do
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

        label="$(printf '%s-%d' "${group}" "${seat}")"
        group_json="$(jq -n --argjson acc "${group_json}" \
            --arg label "${label}" --arg ss58 "${ss58}" --arg pk "${pubkey}" \
            --arg phrase "${phrase}" --arg seed "${seed}" \
            '$acc + [{ seatLabel: $label, ss58Address: $ss58, publicKey: $pk,
                       secretPhrase: $phrase, secretSeed: $seed }]')"

        printf '  %-16s %s\n' "${label}" "${ss58}"
    done
    GROUPS_JSON="$(jq -n --argjson all "${GROUPS_JSON}" \
        --arg d "${group}" --argjson seats "${group_json}" \
        '$all + { ($d): $seats }')"
done

jq -n \
    --arg generatedAt "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg role "${ROLE}" \
    --arg warning "TESTNET ONLY - generated on a build machine, secret phrases stored in plaintext. mainnet_genesis() refuses to build while these are in place." \
    --arg model "${MODEL}" \
    --argjson seats "${GROUPS_JSON}" \
    '{
        generatedAt: $generatedAt,
        network: "belizechain-testnet",
        purpose: "testnet-council-bootstrap",
        role: $role,
        warning: $warning,
        model: $model,
        seats: $seats
    }' > "${OUT_JSON}"
chmod 600 "${OUT_JSON}"

echo
echo "written          : ${OUT_JSON#"${ROOT_DIR}/"} (mode 600)"

# Emit the const array for node/src/chain_spec.rs.
echo
echo "--- paste into node/src/chain_spec.rs (${CONST_NAME}) ---"
echo "pub(crate) const ${CONST_NAME}: [&str; ${SEAT_INDEX}] = ["
jq -r '.seats | to_entries[] | .value[] | "    \"\(.ss58Address)\",  // \(.seatLabel)"' "${OUT_JSON}"
echo "];"
echo
echo "Done. ${SEAT_INDEX} seats generated."
