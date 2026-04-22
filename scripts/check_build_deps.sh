#!/bin/bash
# Pre-flight check for BelizeChain build dependencies

echo "🔍 Checking BelizeChain Build Dependencies..."
echo

ERRORS=0

# Check Rust
echo "✓ Checking Rust toolchain..."
if ! rustc --version &>/dev/null; then
    echo "  ❌ rustc not found"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ rustc $(rustc --version | cut -d' ' -f2)"
fi

# Check WASM target
echo "✓ Checking WASM target..."
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "  ❌ wasm32-unknown-unknown target not installed"
    echo "     Run: rustup target add wasm32-unknown-unknown"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ wasm32-unknown-unknown installed"
fi

# Check protoc
echo "✓ Checking protobuf compiler..."
if ! command -v protoc &>/dev/null; then
    echo "  ❌ protoc not found"
    echo "     Run: sudo apt-get install protobuf-compiler"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ protoc $(protoc --version | cut -d' ' -f2)"
fi

# Check clang
echo "✓ Checking clang..."
if ! command -v clang &>/dev/null; then
    echo "  ❌ clang not found"
    echo "     Run: sudo apt-get install clang"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ clang $(clang --version | head -n1 | grep -oP '\d+\.\d+\.\d+')"
fi

# Check cmake
echo "✓ Checking cmake..."
if ! command -v cmake &>/dev/null; then
    echo "  ❌ cmake not found"
    echo "     Run: sudo apt-get install cmake"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ cmake $(cmake --version | head -n1 | cut -d' ' -f3)"
fi

# Check pkg-config
echo "✓ Checking pkg-config..."
if ! command -v pkg-config &>/dev/null; then
    echo "  ❌ pkg-config not found"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ pkg-config found"
fi

# Check libssl-dev
echo "✓ Checking libssl-dev..."
if ! pkg-config --exists openssl; then
    echo "  ❌ libssl-dev not found"
    echo "     Run: sudo apt-get install libssl-dev"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ libssl-dev $(pkg-config --modversion openssl)"
fi

# Check disk space
echo "✓ Checking disk space..."
DISK_PATH="/mnt/workspace-storage"
if [ ! -d "$DISK_PATH" ]; then
    DISK_PATH="."
fi

AVAILABLE=$(df -BG "$DISK_PATH" 2>/dev/null | tail -1 | awk '{print $4}' | sed 's/G//')
if [[ "$AVAILABLE" =~ ^[0-9]+$ ]]; then
    if [ "$AVAILABLE" -lt 15 ]; then
        echo "  ⚠️  Low disk space on $DISK_PATH: ${AVAILABLE}GB available (need 15GB+)"
        ERRORS=$((ERRORS + 1))
    else
        echo "  ✅ ${AVAILABLE}GB available on $DISK_PATH"
    fi
else
    echo "  ⚠️  Unable to determine available disk space on $DISK_PATH"
    ERRORS=$((ERRORS + 1))
fi

echo
if [ $ERRORS -eq 0 ]; then
    echo "✅ All dependencies satisfied! Ready to build."
    exit 0
else
    echo "❌ $ERRORS issue(s) found. Please fix before building."
    exit 1
fi
