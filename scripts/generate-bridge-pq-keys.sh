#!/usr/bin/env bash
#
# Generate ML-DSA-87 keypairs for bridge validators, in the exact byte layout
# `pallet-belize-interoperability` verifies against.
#
# ## Why OpenSSL and not the chain's own crate
#
# The pallet verifies with `fips204` 0.4.6 (`ml_dsa_87`, context
# `belizechain-bridge-v1`, PK_LEN 2592, SIG_LEN 4627). OpenSSL 3.5+ implements
# the same FIPS 204 final standard, and the two interoperate byte-for-byte:
# measured 2026-11-09 by generating a key here, signing, and verifying the
# signature through `fips204` — accepted for the right context string, rejected
# for a wrong one and for a tampered message. OpenSSL is used because it is
# already present, is audited, and does not require adding a dependency to the
# node build just to mint a key.
#
# ## What "raw" means
#
# `openssl pkey -pubout -outform DER` emits SPKI: a 22-byte header followed by
# the raw FIPS 204 public key. The pallet stores and verifies the raw 2592 bytes
# only, so the header is stripped here rather than left for every caller to get
# wrong. The signature OpenSSL produces is already raw (4627 bytes, no ASN.1
# wrapper), so it is usable as-is.
#
# ## Usage
#
#   ./scripts/generate-bridge-pq-keys.sh <out-dir> <name> [<name> ...]
#
# Writes, per name, into <out-dir>:
#   <name>.pem   private key, mode 0600 — never commit, never copy off-host
#   <name>.pub   raw 2592-byte public key, hex-encoded, one line
#
# Refuses to overwrite an existing key: a silently regenerated key invalidates
# every signature already anchored to the old one.
set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "usage: $0 <out-dir> <name> [<name> ...]" >&2
    exit 2
fi

out_dir="$1"
shift

if ! openssl list -signature-algorithms 2>/dev/null | grep -qi 'ML-DSA-87'; then
    echo "error: this OpenSSL has no ML-DSA-87 support." >&2
    echo "       OpenSSL 3.5+ is required; found: $(openssl version)" >&2
    exit 1
fi

mkdir -p "$out_dir"
chmod 700 "$out_dir"

for name in "$@"; do
    pem="$out_dir/$name.pem"
    pub="$out_dir/$name.pub"

    if [[ -e "$pem" || -e "$pub" ]]; then
        echo "error: refusing to overwrite $pem / $pub" >&2
        echo "       delete it deliberately if that is really intended." >&2
        exit 1
    fi

    # umask, not a post-hoc chmod: the private key must never exist readable.
    ( umask 077; openssl genpkey -algorithm ML-DSA-87 -out "$pem" )

    # SPKI DER is a fixed 22-byte header + the raw key. Verify that assumption
    # instead of trusting it, so a future OpenSSL change fails loudly here
    # rather than producing a key the pallet rejects at verification time.
    der_len=$(openssl pkey -in "$pem" -pubout -outform DER 2>/dev/null | wc -c)
    if [[ "$der_len" -ne 2614 ]]; then
        echo "error: $name SPKI DER is $der_len bytes, expected 2614 (22 + 2592)." >&2
        echo "       the raw-key extraction below would be wrong; aborting." >&2
        exit 1
    fi

    openssl pkey -in "$pem" -pubout -outform DER 2>/dev/null | tail -c 2592 | xxd -p -c 2592 > "$pub"

    bytes=$(($(wc -c < "$pub") / 2))
    if [[ "$bytes" -ne 2592 ]]; then
        echo "error: $name public key is $bytes bytes, expected 2592." >&2
        exit 1
    fi

    echo "generated $name  (private: $pem, public: $pub)"
done

echo
echo "Public keys are hex in *.pub. Register them with:"
echo "  node scripts/rotate-bridge-pq-keys.js --rpc <ws> \\"
echo "    --pq-keys $(printf '%s,' "$out_dir"/*.pub | sed 's/,$//')"
