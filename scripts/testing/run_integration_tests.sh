#!/bin/bash
# BelizeChain Integration Test Runner
# Run this script to execute integration tests for testnet readiness

set -e

echo "🧪 BelizeChain Integration Test Runner"
echo "======================================="
echo ""

# Check if node is running
if ! curl -s -o /dev/null -w "%{http_code}" http://localhost:9933 2>/dev/null | grep -q "200\|405"; then
    echo "❌ BelizeChain node not running at localhost:9933"
    echo "💡 Start node with: ./target/release/belizechain-node --chain=testnet --tmp --alice"
    exit 1
fi
echo "✅ BelizeChain node running"
echo ""

# Set Python path
export PYTHONPATH=$(pwd):$PYTHONPATH

# Run tests based on argument
if [ "$1" == "blockchain" ]; then
    echo "🔬 Running Blockchain Core Tests (16 pallets)..."
    python3 -m pytest tests/blockchain/ -v --tb=short
elif [ "$1" == "cross-pallet" ]; then
    echo "🔬 Running Cross-Pallet Integration Tests..."
    python3 -m pytest tests/cross_pallet/ -v --tb=short
elif [ "$1" == "governance" ]; then
    echo "🔬 Running Governance System Tests..."
    python3 -m pytest tests/governance/ -v --tb=short
elif [ "$1" == "economic" ]; then
    echo "🔬 Running Economic System Tests (DALLA/bBZD)..."
    python3 -m pytest tests/economic/ -v --tb=short
elif [ "$1" == "belizex" ]; then
    echo "🔬 Running BelizeX DEX Tests..."
    python3 -m pytest tests/belizex/ -v --tb=short
elif [ "$1" == "e2e" ]; then
    echo "🔬 Running E2E Scenario Tests..."
    python3 -m pytest tests/e2e/ -v --tb=short -s
elif [ "$1" == "all" ]; then
    echo "🔬 Running All Integration Tests..."
    python3 -m pytest tests/ -v --tb=short
else
    echo "Usage: $0 {blockchain|cross-pallet|governance|economic|belizex|e2e|all}"
    echo ""
    echo "Options:"
    echo "  blockchain    - Run blockchain core pallet tests (16 pallets)"
    echo "  cross-pallet  - Run cross-pallet integration tests"
    echo "  governance    - Run governance system tests (proposals, voting, treasury, council, emergency)"
    echo "  economic      - Run economic system tests (DALLA/bBZD)"
    echo "  belizex       - Run BelizeX DEX tests"
    echo "  e2e           - Run end-to-end scenario tests"
    echo "  all           - Run complete test suite"
    echo "  ipfs       - Run IPFS integration tests only"
    echo "  pakit      - Run all Pakit storage tests"
    echo "  kinich     - Run Kinich quantum tests"
    echo "  nawal      - Run Nawal federated learning tests"
    echo "  e2e        - Run end-to-end scenario tests"
    echo "  all        - Run all integration tests"
    exit 1
fi
