#!/bin/bash
# BelizeChain Complete Test Suite Runner
# Runs Rust tests, an explicit public-testnet smoke test, and Python integration tests.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TESTNET_CHAIN_SPEC_PATH="${TESTNET_CHAIN_SPEC_PATH:-${CHAIN_SPEC_PATH:-}}"
CHAIN_SPEC_VALIDATOR="$ROOT_DIR/scripts/deploy/validate_chain_spec.sh"
SMOKE_NODE_PID=""
DEV_NODE_PID=""

cleanup_node() {
    local node_pid="$1"

    if [ -n "$node_pid" ] && ps -p "$node_pid" > /dev/null 2>&1; then
        kill "$node_pid" 2>/dev/null || true
        wait "$node_pid" 2>/dev/null || true
    fi
}

cleanup() {
    cleanup_node "$SMOKE_NODE_PID"
    cleanup_node "$DEV_NODE_PID"
}

wait_for_rpc() {
    local port="$1"
    local method="$2"
    local expected="$3"
    local response=""

    for _ in $(seq 1 15); do
        response=$(curl -sS -H "Content-Type: application/json" \
            -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":[]}" \
            "http://127.0.0.1:$port" 2>/dev/null || true)

        if echo "$response" | grep -q "$expected"; then
            return 0
        fi

        sleep 1
    done

    return 1
}

trap cleanup EXIT

# Prefer project virtualenv for Python tools
PYTHON_BIN="$(command -v python3)"
if [ -x ".venv/bin/python" ]; then
    PYTHON_BIN=".venv/bin/python"
fi
PIP_CMD="$PYTHON_BIN -m pip"

echo "🧪 BelizeChain Complete Test Suite"
echo "===================================="
echo ""

RUST_TESTS_PASSED=0
PYTHON_TESTS_PASSED=0

echo "📦 Running Rust Unit Tests..."
echo "------------------------------"
if cargo test --workspace --release; then
    echo "✅ Rust unit tests passed"
    RUST_TESTS_PASSED=1
else
    echo "❌ Rust unit tests failed"
fi
echo ""

echo "🔨 Building BelizeChain node..."
echo "--------------------------------"
echo "Building release node binary (this may take several minutes)..."
cargo build --release -p belizechain-node
echo "✅ Node binary ready"
echo ""

if [ -n "$TESTNET_CHAIN_SPEC_PATH" ]; then
    echo "🛰️ Running explicit-spec smoke test..."
    if [ ! -f "$CHAIN_SPEC_VALIDATOR" ]; then
        echo "❌ Chain spec validation script not found: $CHAIN_SPEC_VALIDATOR"
        exit 1
    fi

    bash "$CHAIN_SPEC_VALIDATOR" "$TESTNET_CHAIN_SPEC_PATH"

    ./target/release/belizechain-node \
        --chain "$TESTNET_CHAIN_SPEC_PATH" \
        --tmp \
        --port 30334 \
        --rpc-port 9935 \
        --prometheus-port 9616 \
        --rpc-methods Safe \
        > /tmp/belizechain-testnet-smoke.log 2>&1 &
    SMOKE_NODE_PID=$!

    if ! ps -p "$SMOKE_NODE_PID" > /dev/null 2>&1; then
        echo "❌ Explicit-spec smoke node failed to start. Check /tmp/belizechain-testnet-smoke.log"
        exit 1
    fi

    if ! wait_for_rpc 9935 system_chain 'BelizeChain'; then
        echo "❌ Explicit-spec smoke test did not expose a BelizeChain RPC endpoint"
        echo "   Check /tmp/belizechain-testnet-smoke.log"
        exit 1
    fi

    if ! wait_for_rpc 9935 system_chainType 'Live'; then
        echo "❌ Explicit-spec smoke test did not boot a Live chain"
        echo "   Check /tmp/belizechain-testnet-smoke.log"
        exit 1
    fi

    echo "✅ Explicit-spec smoke test passed"
    cleanup_node "$SMOKE_NODE_PID"
    SMOKE_NODE_PID=""
    echo ""
fi

echo "🚀 Starting local integration node..."
echo "-------------------------------------"
./target/release/belizechain-node --dev --tmp --alice > /tmp/belizechain-node.log 2>&1 &
DEV_NODE_PID=$!
echo "✅ Node started (PID: $DEV_NODE_PID)"
echo ""

echo "⏳ Waiting for node to be ready..."
if ! wait_for_rpc 9933 system_chain 'Development'; then
    echo "❌ Local integration node failed to expose RPC. Check /tmp/belizechain-node.log"
    exit 1
fi
echo "✅ Node is responding"
echo ""

echo "🐍 Running Python Integration Tests..."
echo "---------------------------------------"

export PYTHONPATH="$(pwd):${PYTHONPATH:-}"

if ! $PYTHON_BIN -m pytest --version > /dev/null 2>&1; then
    echo "Installing Python test dependencies..."
    $PIP_CMD install -r tests/requirements.txt
fi

if $PYTHON_BIN -m pytest tests/ -v --tb=short; then
    echo "✅ Python integration tests passed"
    PYTHON_TESTS_PASSED=1
else
    echo "❌ Python integration tests failed"
fi
echo ""

echo "🧹 Cleaning up..."
cleanup_node "$DEV_NODE_PID"
DEV_NODE_PID=""
echo "✅ Node stopped"
echo ""

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

if [ $RUST_TESTS_PASSED -eq 1 ] && [ $PYTHON_TESTS_PASSED -eq 1 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "💥 Some tests failed"
    exit 1
fi
