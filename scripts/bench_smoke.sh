#!/bin/bash
set -euo pipefail

PALLETS=(
  pallet_belize_economy
  pallet_belize_identity
  pallet_belize_governance
  pallet_belize_compliance
  pallet_belize_staking
  pallet_belize_oracle
  pallet_belize_community
  pallet_belize_payroll
  pallet_belize_interoperability
  pallet_belize_belizex
  pallet_belize_landledger
  pallet_belize_consensus
  pallet_belize_quantum
  pallet_belize_bns
  pallet_belize_mesh
  pallet_belize_justice
  pallet_belize_whistleblower
  pallet_belize_moderation
)

PASS=0
FAIL=0
FAILED_PALLETS=""

for p in "${PALLETS[@]}"; do
  echo "=== Benchmarking: $p ==="
  if ./target/release/belizechain-node benchmark pallet \
    --chain dev \
    --pallet "$p" \
    --extrinsic "*" \
    --steps 2 \
    --repeat 1 \
    --no-storage-info \
    --no-median-slopes \
    --no-min-squares 2>&1 | tail -5; then
    echo ">>> PASS: $p"
    PASS=$((PASS + 1))
  else
    echo ">>> FAIL: $p"
    FAIL=$((FAIL + 1))
    FAILED_PALLETS="$FAILED_PALLETS $p"
  fi
  echo ""
done

echo "=============================="
echo "RESULTS: $PASS passed, $FAIL failed"
if [[ -n "$FAILED_PALLETS" ]]; then
  echo "FAILED:$FAILED_PALLETS"
fi

if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
