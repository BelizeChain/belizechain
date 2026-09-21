#!/usr/bin/env bash
# Fail closed if a supplied public-testnet spec still resolves to a local chain
# or is keyed with well-known Substrate dev accounts.
#
# Usage: validate_chain_spec.sh <chain-spec-path> [--allow-dev-accounts]
#
# `--allow-dev-accounts` is for local devnets and CI smoke artifacts only. Any
# spec used to launch or upgrade a shared chain must be validated WITHOUT it: a
# spec whose sudo, session or endowed accounts are publicly-derivable dev
# accounts hands control of the chain to anyone who can read this repository.
#
# Dev accounts are detected in both forms a chain spec can carry:
#   * SS58 addresses (plain `build-spec` output)
#   * 32-byte public-key hex (raw `build-spec --raw` output)

set -euo pipefail

CHAIN_SPEC_PATH="${1:-}"
shift || true

ALLOW_DEV_ACCOUNTS=false
for arg in "$@"; do
    case "$arg" in
        --allow-dev-accounts) ALLOW_DEV_ACCOUNTS=true ;;
        *)
            echo "ERROR: unknown argument: $arg" >&2
            exit 1
            ;;
    esac
done

if [[ -z "$CHAIN_SPEC_PATH" ]]; then
    echo "ERROR: usage: validate_chain_spec.sh <chain-spec-path> [--allow-dev-accounts]" >&2
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

if [[ "$ALLOW_DEV_ACCOUNTS" == false ]]; then
    # `//Alice` … `//Ferdie`, plus Alice's ed25519 session key.
    DEV_ACCOUNTS_SS58=(
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY" # //Alice (sr25519)
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty" # //Bob (sr25519)
        "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y" # //Charlie (sr25519)
        "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy" # //Dave (sr25519)
        "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw" # //Eve (sr25519)
        "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL" # //Ferdie (sr25519)
        "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu" # //Alice (ed25519)
    )
    DEV_ACCOUNTS_HEX=(
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d" # //Alice (sr25519)
        "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48" # //Bob (sr25519)
        "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22" # //Charlie (sr25519)
        "306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20" # //Dave (sr25519)
        "e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e" # //Eve (sr25519)
        "1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c" # //Ferdie (sr25519)
        "88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee" # //Alice (ed25519)
    )
    DEV_MNEMONIC="bottom drive obey lake curtain smoke basket hold race lonely fit walk"

    for account in "${DEV_ACCOUNTS_SS58[@]}"; do
        if grep -qF "$account" "$CHAIN_SPEC_PATH"; then
            echo "ERROR: chain spec contains the well-known dev account $account: $CHAIN_SPEC_PATH" >&2
            echo "       Regenerate it with operator keys, or pass --allow-dev-accounts for a local devnet." >&2
            exit 1
        fi
    done

    for key in "${DEV_ACCOUNTS_HEX[@]}"; do
        if grep -qiF "$key" "$CHAIN_SPEC_PATH"; then
            echo "ERROR: chain spec contains the well-known dev public key 0x$key: $CHAIN_SPEC_PATH" >&2
            echo "       Regenerate it with operator keys, or pass --allow-dev-accounts for a local devnet." >&2
            exit 1
        fi
    done

    if grep -qiF "$DEV_MNEMONIC" "$CHAIN_SPEC_PATH"; then
        echo "ERROR: chain spec contains the standard dev mnemonic: $CHAIN_SPEC_PATH" >&2
        exit 1
    fi
fi

echo "OK: chain spec passes non-local, non-dev-account preflight checks"