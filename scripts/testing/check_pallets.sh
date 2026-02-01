#!/bin/bash
# Fix saturating operations in all BelizeChain pallets

echo "🔧 Fixing saturating operations in all pallets..."

PALLETS=(
    "belizex"
    "community"
    "compliance"
    "consensus"
    "governance"
    "interoperability"
    "landledger"
    "oracle"
    "payroll"
    "quantum"
    "staking"
)

for pallet in "${PALLETS[@]}"; do
    echo "Checking pallet-belize-$pallet..."
    
    # Try to compile
    if cargo check -p pallet-belize-$pallet 2>&1 | grep -q "error:"; then
        echo "  ❌ Has errors - needs manual fix"
    else
        echo "  ✅ Compiles successfully"
    fi
done

echo ""
echo "Summary: Check which pallets need fixes above"
