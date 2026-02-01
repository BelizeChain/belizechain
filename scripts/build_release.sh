#!/bin/bash
# BelizeChain Production Build Script
# Creates optimized release build for testnet/mainnet deployment

set -e

echo "🔨 BelizeChain Production Build"
echo "================================"
echo ""

# Check Rust version
echo "📋 Checking build environment..."
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust not found. Install from: https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "✅ Rust $RUST_VERSION"

# Check for required toolchain
if ! rustup show | grep -q "stable"; then
    echo "⚠️ Installing Rust stable toolchain..."
    rustup install stable
fi

# Ensure wasm target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Installing wasm32 target..."
    rustup target add wasm32-unknown-unknown
fi
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean
echo "✅ Clean complete"
echo ""

# Build release
echo "🔨 Building release binary (this may take 8-10 minutes)..."
echo "   - Optimizations: release"
echo "   - WASM runtime: included"
echo "   - Target: x86_64-unknown-linux-gnu"
echo ""

time cargo build --release

echo ""
echo "✅ Build complete!"
echo ""

# Verify binary
if [ -f "./target/release/belizechain-node" ]; then
    BINARY_SIZE=$(du -h ./target/release/belizechain-node | cut -f1)
    echo "📦 Binary Information:"
    echo "   - Path: ./target/release/belizechain-node"
    echo "   - Size: $BINARY_SIZE"
    echo ""
    
    # Show version
    echo "📌 Node Version:"
    ./target/release/belizechain-node --version
    echo ""
    
    # Run quick test
    echo "🧪 Running quick validation..."
    if timeout 10 ./target/release/belizechain-node --dev --tmp --no-telemetry 2>&1 | grep -q "Running in --dev mode"; then
        echo "✅ Binary validation passed"
    else
        echo "⚠️ Binary may have issues (check manually)"
    fi
    
else
    echo "❌ Build failed - binary not found"
    exit 1
fi

echo ""
echo "🎉 Production build ready for deployment!"
echo ""
echo "Next steps:"
echo "  1. Test: ./scripts/start_testnet.sh dev"
echo "  2. Run tests: ./scripts/run_all_tests.sh"
echo "  3. Deploy: ./scripts/deploy/deploy_testnet.sh"
