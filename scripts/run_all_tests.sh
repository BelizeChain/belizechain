#!/bin/bash
# BelizeChain Complete Test Suite Runner
# Runs all tests: Rust unit tests + Python integration tests

set -e

echo "🧪 BelizeChain Complete Test Suite"
echo "===================================="
echo ""

# Track results
RUST_TESTS_PASSED=0
PYTHON_TESTS_PASSED=0

# 1. Rust Unit Tests
echo "📦 Running Rust Unit Tests..."
echo "------------------------------"
if cargo test --workspace --release; then
    echo "✅ Rust unit tests passed"
    RUST_TESTS_PASSED=1
else
    echo "❌ Rust unit tests failed"
fi
echo ""

# 2. Build node for integration tests
echo "🔨 Building BelizeChain node..."
echo "--------------------------------"
if [ ! -f "./target/release/belizechain-node" ]; then
    echo "Building node (this may take several minutes)..."
    cargo build --release
fi
echo "✅ Node binary ready"
echo ""

# 3. Start testnet node in background
echo "🚀 Starting testnet node..."
echo "----------------------------"
./target/release/belizechain-node --chain=testnet --tmp --alice > /tmp/belizechain-node.log 2>&1 &
NODE_PID=$!
echo "✅ Node started (PID: $NODE_PID)"
echo ""

# Wait for node to be ready
echo "⏳ Waiting for node to be ready..."
sleep 5

# Check if node is running
if ! ps -p $NODE_PID > /dev/null; then
    echo "❌ Node failed to start. Check logs: /tmp/belizechain-node.log"
    exit 1
fi

if curl -s -o /dev/null -w "%{http_code}" http://localhost:9933 2>/dev/null | grep -q "200\|405"; then
    echo "✅ Node is responding"
else
    echo "⚠️ Node may not be fully ready yet"
fi
echo ""

# 4. Run Python Integration Tests
echo "🐍 Running Python Integration Tests..."
echo "---------------------------------------"

# Set Python path
export PYTHONPATH=$(pwd):$PYTHONPATH

# Check if pytest is available
if ! command -v pytest &> /dev/null; then
    echo "Installing pytest..."
    pip install -r tests/requirements.txt
fi

# Run all integration tests
if python3 -m pytest tests/ -v --tb=short; then
    echo "✅ Python integration tests passed"
    PYTHON_TESTS_PASSED=1
else
    echo "❌ Python integration tests failed"
fi
echo ""

# Cleanup: Stop node
echo "🧹 Cleaning up..."
kill $NODE_PID 2>/dev/null || true
wait $NODE_PID 2>/dev/null || true
echo "✅ Node stopped"
echo ""

# Summary
echo "📊 Test Summary"
echo "==============="
if [ $RUST_TESTS_PASSED -eq 1 ]; then
    echo "✅ Rust Unit Tests: PASSED"
else
    echo "❌ Rust Unit Tests: FAILED"
fi

if [ $PYTHON_TESTS_PASSED -eq 1 ]; then
    echo "✅ Python Integration Tests: PASSED"
else
    echo "❌ Python Integration Tests: FAILED"
fi
echo ""

# Exit with error if any tests failed
if [ $RUST_TESTS_PASSED -eq 1 ] && [ $PYTHON_TESTS_PASSED -eq 1 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "💥 Some tests failed"
    exit 1
fi
