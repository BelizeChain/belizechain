#!/usr/bin/env bash
# ==============================================================================
# BelizeChain Production Mainnet ChainSpec Builder
# ==============================================================================
# Injects sovereign Root Sudo key, Treasury, and 4 multi-node validator
# session keys from mainnet-keys-inventory.json and builds canonical raw chainspec.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
NODE_BIN="${ROOT_DIR}/target/debug/belizechain-node"
KEYS_JSON="${ROOT_DIR}/generated_keys/mainnet-keys-inventory.json"
OUTPUT_DIR="${ROOT_DIR}/generated_specs"

if [[ ! -f "${KEYS_JSON}" ]]; then
    echo "❌ Keys inventory not found at ${KEYS_JSON}. Run './scripts/generate-production-keys.sh' first." >&2
    exit 1
fi

mkdir -p "${OUTPUT_DIR}"

SPEC_HUMAN="${OUTPUT_DIR}/chainspec-mainnet.json"
SPEC_RAW="${OUTPUT_DIR}/chainspec-mainnet-raw.json"

echo "📜 Step 1: Exporting baseline template chainspec from runtime WASM..."
"${NODE_BIN}" build-spec --chain staging --disable-default-bootnode > "${SPEC_HUMAN}.tmp"

echo "🔧 Step 2: Injecting sovereign Root Sudo, Treasury, and 4-Validator Authorities..."

ROOT_SS58=$(jq -r '.rootSudo.sr25519.ss58Address' "${KEYS_JSON}")
TREASURY_SS58=$(jq -r '.treasury.sr25519.ss58Address' "${KEYS_JSON}")

VAL1_SS58=$(jq -r '.validators[0].sr25519.ss58Address' "${KEYS_JSON}")
VAL1_BABE=$(jq -r '.validators[0].sr25519.ss58Address' "${KEYS_JSON}")
VAL1_GRAN=$(jq -r '.validators[0].ed25519.ss58Address' "${KEYS_JSON}")

VAL2_SS58=$(jq -r '.validators[1].sr25519.ss58Address' "${KEYS_JSON}")
VAL2_BABE=$(jq -r '.validators[1].sr25519.ss58Address' "${KEYS_JSON}")
VAL2_GRAN=$(jq -r '.validators[1].ed25519.ss58Address' "${KEYS_JSON}")

VAL3_SS58=$(jq -r '.validators[2].sr25519.ss58Address' "${KEYS_JSON}")
VAL3_BABE=$(jq -r '.validators[2].sr25519.ss58Address' "${KEYS_JSON}")
VAL3_GRAN=$(jq -r '.validators[2].ed25519.ss58Address' "${KEYS_JSON}")

VAL4_SS58=$(jq -r '.validators[3].sr25519.ss58Address' "${KEYS_JSON}")
VAL4_BABE=$(jq -r '.validators[3].sr25519.ss58Address' "${KEYS_JSON}")
VAL4_GRAN=$(jq -r '.validators[3].ed25519.ss58Address' "${KEYS_JSON}")

# 100M DALLA with 12 decimals = 100_000_000 * 10^12 = 100_000_000_000_000_000_000
# Treasury: 60M DALLA
# Root Founder: 20M DALLA
# Validators (each 2.5M DALLA): 10M DALLA
# Faucet/Community: 10M DALLA
jq \
    --arg root "${ROOT_SS58}" \
    --arg treasury "${TREASURY_SS58}" \
    --arg v1 "${VAL1_SS58}" --arg v1_babe "${VAL1_BABE}" --arg v1_gran "${VAL1_GRAN}" \
    --arg v2 "${VAL2_SS58}" --arg v2_babe "${VAL2_BABE}" --arg v2_gran "${VAL2_GRAN}" \
    --arg v3 "${VAL3_SS58}" --arg v3_babe "${VAL3_BABE}" --arg v3_gran "${VAL3_GRAN}" \
    --arg v4 "${VAL4_SS58}" --arg v4_babe "${VAL4_BABE}" --arg v4_gran "${VAL4_GRAN}" \
    '
    .name = "BelizeChain Mainnet" |
    .id = "belizechain_mainnet" |
    .chainType = "Live" |
    .genesis.runtimeGenesis.patch.sudo.key = $root |
    .genesis.runtimeGenesis.patch.balances.balances = [
        [$treasury, 60000000000000000000],
        [$root, 20000000000000000000],
        [$v1, 2500000000000000000],
        [$v2, 2500000000000000000],
        [$v3, 2500000000000000000],
        [$v4, 2500000000000000000]
    ] |
    .genesis.runtimeGenesis.patch.session.keys = [
        [$v1, $v1, {babe: $v1_babe, grandpa: $v1_gran}],
        [$v2, $v2, {babe: $v2_babe, grandpa: $v2_gran}],
        [$v3, $v3, {babe: $v3_babe, grandpa: $v3_gran}],
        [$v4, $v4, {babe: $v4_babe, grandpa: $v4_gran}]
    ] |
    .genesis.runtimeGenesis.patch.identity.initialSsnIssuers = [$root] |
    .genesis.runtimeGenesis.patch.identity.initialPassportIssuers = [$root] |
    .genesis.runtimeGenesis.patch.identity.initialBiometricIssuers = [$root] |
    .genesis.runtimeGenesis.patch.governance.councilMembers = [
        [$root, 1, 0],
        [$treasury, 1, 0]
    ]
    ' "${SPEC_HUMAN}.tmp" > "${SPEC_HUMAN}"

rm -f "${SPEC_HUMAN}.tmp"

echo "⚙️  Step 3: Compiling canonical raw genesis spec (chainspec-mainnet-raw.json)..."
"${NODE_BIN}" build-spec --chain "${SPEC_HUMAN}" --raw --disable-default-bootnode > "${SPEC_RAW}"

echo "🔒 Step 4: Computing Cryptographic Checksums..."
SHA_HUMAN=$(sha256sum "${SPEC_HUMAN}" | awk '{print $1}')
SHA_RAW=$(sha256sum "${SPEC_RAW}" | awk '{print $1}')

echo ""
echo "=============================================================================="
echo "🎉 BelizeChain Production Genesis Spec Created Successfully!"
echo "=============================================================================="
echo "📄 Human-Readable Spec: ${SPEC_HUMAN} (SHA256: ${SHA_HUMAN})"
echo "⚡ Raw Binary Spec:     ${SPEC_RAW} (SHA256: ${SHA_RAW})"
echo "👑 Sovereign Root Sudo: ${ROOT_SS58}"
echo "🏛️  Treasury Balance:    60,000,000 DALLA"
echo "💼 Founder Reserve:     20,000,000 DALLA"
echo "🛡️  4 Authorities:      Ceiba, Maya, Reef, Cayo (10,000,000 DALLA Total Stake)"
echo "=============================================================================="
