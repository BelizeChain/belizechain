#!/bin/bash
# Compile-check every custom BelizeChain pallet (19 pallets, runtime indexes 20-38).

echo "🔍 Checking all 19 BelizeChain custom pallets..."

PACKAGES=(
    "pallet-belize-belizex"
    "pallet-belize-bns"
    "pallet-belize-community"
    "pallet-belize-compliance"
    "pallet-belize-consensus"
    "pallet-belize-economy"
    "pallet-belize-governance"
    "pallet-belize-identity"
    "pallet-belize-interoperability"
    "pallet-belize-justice"
    "pallet-belize-landledger"
    "pallet-belize-mesh"
    "pallet-belize-moderation"
    "pallet-belize-oracle"
    "pallet-belize-payroll"
    "pallet-belize-quantum"
    "pallet-belize-staking"
    "pallet-storage-proof"
    "pallet-belize-whistleblower"
)

for package in "${PACKAGES[@]}"; do
    echo "Checking $package..."
    
    # Try to compile
    if cargo check -p "$package" 2>&1 | grep -q "error:"; then
        echo "  ❌ Has errors - needs manual fix"
    else
        echo "  ✅ Compiles successfully"
    fi
done

echo ""
echo "Summary: Check which pallets need fixes above"
