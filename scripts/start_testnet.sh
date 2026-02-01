#!/bin/bash
# BelizeChain Testnet Node Startup Script
# Starts a local testnet node for development and testing

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
MODE="${1:-dev}"

if [ "$MODE" == "dev" ]; then
    echo "🔧 Starting testnet node in development mode..."
    echo "   - Chain: testnet"
    echo "   - Temporary database (--tmp)"
    echo "   - Alice account (sudo access)"
    echo "   - RPC endpoint: http://localhost:9933"
    echo "   - WS endpoint: ws://localhost:9944"
    echo ""
    
    ./target/release/belizechain-node \
        --chain=testnet \
        --tmp \
        --alice \
        --rpc-cors=all \
        --rpc-methods=Unsafe \
        --rpc-external \
        --ws-external
        
elif [ "$MODE" == "validator" ]; then
    echo "🔧 Starting testnet node in validator mode..."
    echo "   - Chain: testnet"
    echo "   - Persistent database"
    echo "   - Validator mode"
    echo "   - Bootnodes: <to be configured>"
    echo ""
    
    ./target/release/belizechain-node \
        --chain=testnet \
        --validator \
        --name="BelizeChain-Validator-1" \
        --rpc-cors=all \
        --rpc-external \
        --ws-external \
        --base-path=./chain-data
        
elif [ "$MODE" == "archive" ]; then
    echo "🔧 Starting testnet node in archive mode..."
    echo "   - Chain: testnet"
    echo "   - Archive mode (full history)"
    echo "   - RPC endpoint: http://localhost:9933"
    echo "   - WS endpoint: ws://localhost:9944"
    echo ""
    
    ./target/release/belizechain-node \
        --chain=testnet \
        --pruning=archive \
        --rpc-cors=all \
        --rpc-methods=Safe \
        --rpc-external \
        --ws-external \
        --base-path=./chain-data-archive
        
else
    echo "Usage: $0 {dev|validator|archive}"
    echo ""
    echo "Modes:"
    echo "  dev        - Development mode (temporary, Alice sudo, unsafe RPC)"
    echo "  validator  - Validator mode (persistent, production)"
    echo "  archive    - Archive mode (full history, for explorers)"
    echo ""
    exit 1
fi
