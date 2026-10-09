#!/bin/bash
# BelizeChain Production Build Script
# Creates optimized release build for testnet/mainnet deployment
#
# Usage: build_release.sh <testnet|mainnet>
#
# The target is REQUIRED and not defaulted: `BabeEpochDuration` is a compile-time
# constant (300 with `testnet-fast-epoch`, 14_400 without), so a build with the
# wrong feature silently changes the epoch length and halts block import. Ask for
# the target explicitly rather than guessing. For a runtime blob you intend to
# deploy, prefer `scripts/build-runtime-blob.sh`, which also proves the blob's
# epoch configuration by booting it.

set -e

TARGET="${1:-}"
if [[ "$TARGET" != "testnet" && "$TARGET" != "mainnet" ]]; then
    echo "usage: $0 <testnet|mainnet>" >&2
    echo "" >&2
    echo "The target selects the BABE epoch configuration and cannot be defaulted:" >&2
    echo "  testnet -> --features testnet-fast-epoch (epoch = 300 slots)" >&2
    echo "  mainnet -> default features              (epoch = 14400 slots)" >&2
    exit 1
fi

if [[ "$TARGET" == "testnet" ]]; then
    BUILD_FEATURES=(--features testnet-fast-epoch)
    EXPECTED_EPOCH=300
else
    BUILD_FEATURES=()
    EXPECTED_EPOCH=14400
fi

echo "🔨 BelizeChain Production Build"
echo "================================"
echo "   target: $TARGET (epoch $EXPECTED_EPOCH slots)"
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
echo "   - Features: ${BUILD_FEATURES[*]:-<none>}"
echo "   - Target: x86_64-unknown-linux-gnu"
echo ""

# The runtime must be built with the target's feature set. Pass them at the
# workspace level so both the node (embedded genesis wasm) and the runtime agree.
time cargo build --release "${BUILD_FEATURES[@]}"

# Prove the runtime blob's epoch configuration before anything can deploy it.
echo ""
echo "🔎 Verifying runtime epoch configuration..."
if ! ./scripts/build-runtime-blob.sh "$TARGET"; then
    echo "❌ Runtime blob failed epoch verification — NOT deployable." >&2
    exit 1
fi

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
