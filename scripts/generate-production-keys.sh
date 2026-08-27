#!/usr/bin/env bash
# ==============================================================================
# BelizeChain Production Genesis Key & Validator Generator
# ==============================================================================
# Generates sovereign Root Sudo key, Treasury, and 4 multi-node validator
# session keypairs (BABE sr25519 + GRANDPA ed25519) using belizechain-node.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
NODE_BIN="${ROOT_DIR}/target/debug/belizechain-node"

if [[ ! -x "${NODE_BIN}" ]]; then
    # Check release build
    NODE_BIN="${ROOT_DIR}/target/release/belizechain-node"
    if [[ ! -x "${NODE_BIN}" ]]; then
        echo "❌ Node binary not found at target/debug/belizechain-node. Run 'cargo build -p belizechain-node' first." >&2
        exit 1
    fi
fi

OUTPUT_DIR="${ROOT_DIR}/generated_keys"
mkdir -p "${OUTPUT_DIR}"
chmod 700 "${OUTPUT_DIR}"

KEYS_JSON="${OUTPUT_DIR}/mainnet-keys-inventory.json"
CHAINSPEC_PATCH="${OUTPUT_DIR}/genesis-authorities.json"

echo "🔐 Generating Sovereign BelizeChain Production Keys..."

# Function to generate an account
generate_account() {
    local name="$1"
    local raw_json
    raw_json=$("${NODE_BIN}" key generate --output-type json --scheme sr25519)
    
    local phrase
    local seed
    local pubkey
    local ss58
    
    phrase=$(echo "${raw_json}" | jq -r '.secretPhrase')
    seed=$(echo "${raw_json}" | jq -r '.secretSeed')
    pubkey=$(echo "${raw_json}" | jq -r '.publicKey')
    ss58=$(echo "${raw_json}" | jq -r '.ss58Address')
    
    # Generate ed25519 for GRANDPA from same phrase
    local gran_json
    gran_json=$("${NODE_BIN}" key inspect --scheme ed25519 --output-type json "${phrase}")
    local gran_pubkey
    local gran_ss58
    gran_pubkey=$(echo "${gran_json}" | jq -r '.publicKey')
    gran_ss58=$(echo "${gran_json}" | jq -r '.ss58Address')

    jq -n \
        --arg name "${name}" \
        --arg phrase "${phrase}" \
        --arg seed "${seed}" \
        --arg sr_pub "${pubkey}" \
        --arg sr_ss58 "${ss58}" \
        --arg ed_pub "${gran_pubkey}" \
        --arg ed_ss58 "${gran_ss58}" \
        '{
            name: $name,
            secretPhrase: $phrase,
            secretSeed: $seed,
            sr25519: {
                publicKey: $sr_pub,
                ss58Address: $sr_ss58
            },
            ed25519: {
                publicKey: $ed_pub,
                ss58Address: $ed_ss58
            }
        }'
}

echo "  -> Generating Founder / Root Sudo Key..."
ROOT_KEY=$(generate_account "FounderRootSudo")

echo "  -> Generating Treasury Sovereign Reserve..."
TREASURY_KEY=$(generate_account "BelizeTreasury")

echo "  -> Generating Validator 1 (Ceiba Authority)..."
VAL1=$(generate_account "Validator-1-Ceiba")

echo "  -> Generating Validator 2 (Maya Authority)..."
VAL2=$(generate_account "Validator-2-Maya")

echo "  -> Generating Validator 3 (BarrierReef Authority)..."
VAL3=$(generate_account "Validator-3-BarrierReef")

echo "  -> Generating Validator 4 (Cayo Authority)..."
VAL4=$(generate_account "Validator-4-Cayo")

# Combine into master inventory
MASTER_INVENTORY=$(jq -n \
    --argjson root "${ROOT_KEY}" \
    --argjson treasury "${TREASURY_KEY}" \
    --argjson val1 "${VAL1}" \
    --argjson val2 "${VAL2}" \
    --argjson val3 "${VAL3}" \
    --argjson val4 "${VAL4}" \
    '{
        generatedAt: (now | todate),
        network: "belizechain-mainnet",
        rootSudo: $root,
        treasury: $treasury,
        validators: [
            $val1,
            $val2,
            $val3,
            $val4
        ]
    }')

echo "${MASTER_INVENTORY}" > "${KEYS_JSON}"
chmod 600 "${KEYS_JSON}"

# Ensure output directory is ignored in git
if ! grep -q "generated_keys" "${ROOT_DIR}/.gitignore"; then
    echo "generated_keys/" >> "${ROOT_DIR}/.gitignore"
fi

echo "✅ Keys generated successfully!"
echo "📁 Stored securely at: ${KEYS_JSON}"
echo ""
echo "=============================================================================="
echo "👑 Sudo Root Account SS58: $(echo "${ROOT_KEY}" | jq -r '.sr25519.ss58Address')"
echo "🏛️  Treasury Account SS58:  $(echo "${TREASURY_KEY}" | jq -r '.sr25519.ss58Address')"
echo "🛡️  Validator 1 (Ceiba):   $(echo "${VAL1}" | jq -r '.sr25519.ss58Address')"
echo "🛡️  Validator 2 (Maya):    $(echo "${VAL2}" | jq -r '.sr25519.ss58Address')"
echo "🛡️  Validator 3 (Reef):    $(echo "${VAL3}" | jq -r '.sr25519.ss58Address')"
echo "🛡️  Validator 4 (Cayo):    $(echo "${VAL4}" | jq -r '.sr25519.ss58Address')"
echo "=============================================================================="
