#!/bin/bash
# BelizeChain Testnet Startup Script
# Supports local rehearsal and explicit-spec testnet launches

set -e

echo "🏛️ Starting BelizeChain Testnet Node..."
echo "========================================"
echo ""

# Check if node binary exists
if [ ! -f "./target/release/belizechain-node" ]; then
    echo "❌ BelizeChain node binary not found at ./target/release/belizechain-node"
    echo "💡 Build with: cargo build --release"
    exit 1
fi

echo "✅ BelizeChain node binary found"
echo ""

# Parse arguments
MODE="${1:-local}"
CHAIN_SPEC="${CHAIN_SPEC:-${2:-}}"

if [ "$MODE" == "dev" ] || [ "$MODE" == "local" ]; then
    echo "🔧 Starting local rehearsal node..."
    echo "   - Chain: local"
    echo "   - Temporary database (--tmp)"
    echo "   - Alice account (sudo access)"
    echo "   - RPC endpoint: http://localhost:9933"
    echo "   - WS endpoint: ws://localhost:9944"
    echo ""
    
    ./target/release/belizechain-node \
        --chain=local \
        --tmp \
        --alice \
        --rpc-cors=all \
        --rpc-methods=Unsafe \
        --rpc-external \
        --ws-external
        
elif [ "$MODE" == "validator" ]; then
    if [ -z "$CHAIN_SPEC" ] || [ ! -f "$CHAIN_SPEC" ]; then
        echo "❌ Validator mode requires CHAIN_SPEC to point to a generated JSON/raw testnet spec"
        echo "💡 Example: CHAIN_SPEC=./belizechain-testnet-raw.json $0 validator"
        exit 1
    fi

    echo "🔧 Starting validator from explicit testnet spec..."
    echo "   - Chain spec: $CHAIN_SPEC"
    echo "   - Persistent database"
    echo "   - Validator mode"
    echo "   - Bootnodes: configure separately as needed"
    echo ""
    
    ./target/release/belizechain-node \
        --chain="$CHAIN_SPEC" \
        --validator \
        --name="BelizeChain-Validator-1" \
        --rpc-cors=all \
        --rpc-methods=Safe \
        --rpc-external \
        --ws-external \
        --base-path=./chain-data
        
elif [ "$MODE" == "archive" ]; then
    if [ -z "$CHAIN_SPEC" ] || [ ! -f "$CHAIN_SPEC" ]; then
        echo "❌ Archive mode requires CHAIN_SPEC to point to a generated JSON/raw testnet spec"
        echo "💡 Example: CHAIN_SPEC=./belizechain-testnet-raw.json $0 archive"
        exit 1
    fi

    echo "🔧 Starting archive node from explicit testnet spec..."
    echo "   - Chain spec: $CHAIN_SPEC"
    echo "   - Archive mode (full history)"
    echo "   - RPC endpoint: http://localhost:9933"
    echo "   - WS endpoint: ws://localhost:9944"
    echo ""
    
    ./target/release/belizechain-node \
        --chain="$CHAIN_SPEC" \
        --pruning=archive \
        --rpc-cors=all \
        --rpc-methods=Safe \
        --rpc-external \
        --ws-external \
        --base-path=./chain-data-archive
        
else
    echo "Usage: $0 {local|validator|archive} [chain-spec.json]"
    echo ""
    echo "Modes:"
    echo "  local      - Local rehearsal chain (temporary, Alice sudo, unsafe RPC)"
    echo "  dev        - Alias of local"
    echo "  validator  - Validator mode using an explicit JSON/raw testnet spec"
    echo "  archive    - Archive mode using an explicit JSON/raw testnet spec"
    echo ""
    exit 1
fi
