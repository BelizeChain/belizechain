#!/bin/bash
# BelizeChain Testnet Deployment Script
# Packages a node release for public testnet deployment.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BINARY_PATH="$ROOT_DIR/target/release/belizechain-node"
CHAIN_SPEC_VALIDATOR="$ROOT_DIR/scripts/deploy/validate_chain_spec.sh"
CHAIN_SPEC_PATH="${CHAIN_SPEC_PATH:-}"
RESERVED_NODES_PATH="${RESERVED_NODES_PATH:-$ROOT_DIR/scripts/deploy/reserved-nodes.generated.txt}"

validate_reserved_nodes() {
    local file_path="$1"
    local valid_entries

    if grep -Eq 'Placeholder|192\.0\.2\.' "$file_path"; then
        return 1
    fi

    valid_entries="$(grep -Ec '^[[:space:]]*/(ip4|dns4|ip6|dns6)/' "$file_path" || true)"
    if [ "$valid_entries" -eq 0 ]; then
        return 1
    fi
}

echo "🚀 BelizeChain Testnet Deployment"
echo "==================================="
echo ""

echo "📋 Pre-deployment checks..."

if [ ! -f "$BINARY_PATH" ]; then
    echo "ℹ️ Release node binary not found yet; the test run will build it before packaging"
else
    echo "✅ Existing node binary found (will be rebuilt before packaging)"
fi

if [ ! -f "$CHAIN_SPEC_VALIDATOR" ]; then
    echo "❌ Chain spec validation script not found: $CHAIN_SPEC_VALIDATOR"
    exit 1
fi

if [ -z "$CHAIN_SPEC_PATH" ]; then
    echo "❌ CHAIN_SPEC_PATH is not set"
    echo "💡 Generate a dedicated testnet spec first:"
    echo "   ./target/release/belizechain-node build-spec --disable-default-bootnode --chain testnet-template > belizechain-testnet-plain.json"
    echo "   ./target/release/belizechain-node build-spec --disable-default-bootnode --chain belizechain-testnet-plain.json --raw > belizechain-testnet-raw.json"
    echo "   CHAIN_SPEC_PATH=./belizechain-testnet-raw.json ./scripts/deploy/deploy_testnet.sh"
    exit 1
fi

if [ ! -f "$CHAIN_SPEC_PATH" ]; then
    echo "❌ CHAIN_SPEC_PATH does not exist: $CHAIN_SPEC_PATH"
    exit 1
fi

echo "🔍 Validating chain spec..."
bash "$CHAIN_SPEC_VALIDATOR" "$CHAIN_SPEC_PATH"
echo "✅ Chain spec found: $CHAIN_SPEC_PATH"
echo ""

echo "ℹ️ Node-only testnet packaging does not require the Phase 2 service-stack .env preflight."
echo ""

echo "🧪 Running pre-deployment tests..."
if ! TESTNET_CHAIN_SPEC_PATH="$CHAIN_SPEC_PATH" "$ROOT_DIR/scripts/run_all_tests.sh"; then
    echo "❌ Tests failed - deployment aborted"
    exit 1
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Fresh release node binary was not produced by the test run"
    exit 1
fi
echo ""

echo "📦 Creating deployment package..."
DEPLOY_DIR="$ROOT_DIR/deploy-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$DEPLOY_DIR"

cp "$BINARY_PATH" "$DEPLOY_DIR/"
echo "✅ Binary copied"

cp "$CHAIN_SPEC_PATH" "$DEPLOY_DIR/testnet-chain-spec-raw.json"
echo "✅ Chain spec copied"

USE_RESERVED_ONLY=0
if [ -n "$RESERVED_NODES_PATH" ] && [ -f "$RESERVED_NODES_PATH" ]; then
    if validate_reserved_nodes "$RESERVED_NODES_PATH"; then
        cp "$RESERVED_NODES_PATH" "$DEPLOY_DIR/reserved-nodes.txt"
        echo "✅ Reserved nodes config copied"
        USE_RESERVED_ONLY=1
    else
        echo "⚠️ Reserved nodes inventory is missing, incomplete, or still contains placeholders; packaging without --reserved-only"
    fi
else
    echo "ℹ️ No operator-generated reserved nodes file provided; packaging without --reserved-only"
fi

if [ "$USE_RESERVED_ONLY" -eq 1 ]; then
    RESERVED_NODE_ARGS='    --reserved-nodes-file=/opt/belizechain/reserved-nodes.txt \
    --reserved-only \
'
    RESERVED_NODE_COPY_STEP='sudo cp reserved-nodes.txt /opt/belizechain/'
    RESERVED_NODE_NOTE='- Reserved-node allowlist is enabled in this package.'
else
    RESERVED_NODE_ARGS=''
    RESERVED_NODE_COPY_STEP='# reserved-nodes.txt intentionally omitted until operators publish reserved-nodes.generated.txt'
    RESERVED_NODE_NOTE='- Reserved-node allowlist is disabled in this package because no operator-generated validator inventory has been provided yet.'
fi

cat > "$DEPLOY_DIR/belizechain-testnet.service" << EOF
[Unit]
Description=BelizeChain Public Testnet Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=belizechain
Group=belizechain
WorkingDirectory=/opt/belizechain
ExecStartPre=/usr/bin/test -f /opt/belizechain/testnet-chain-spec-raw.json
ExecStartPre=/bin/sh -c '! grep -Eq '"'"'"id"[[:space:]]*:[[:space:]]*"belizechain_local"'"'"' /opt/belizechain/testnet-chain-spec-raw.json'
ExecStartPre=/bin/sh -c '! grep -Eq '"'"'"chainType"[[:space:]]*:[[:space:]]*"Local"'"'"' /opt/belizechain/testnet-chain-spec-raw.json'
ExecStartPre=/bin/sh -c '! grep -Eq '"'"'/ip4/(127\\.0\\.0\\.1|0\\.0\\.0\\.0)/|/dns4/localhost/'"'"' /opt/belizechain/testnet-chain-spec-raw.json'
ExecStart=/opt/belizechain/belizechain-node \
    --chain=/opt/belizechain/testnet-chain-spec-raw.json \
    --validator \
    --name="Testnet-Validator-1" \
    --base-path=/var/lib/belizechain \
    --port=30333 \
    --rpc-port=9933 \
    --prometheus-port=9615 \
${RESERVED_NODE_ARGS}    --no-mdns \
    --in-peers=5 \
    --out-peers=50 \
    --rpc-cors=all \
    --rpc-external \
    --ws-external \
    --rpc-methods=Safe \
    --prometheus-external
Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
echo "✅ Systemd service file created"

cat > "$DEPLOY_DIR/DEPLOY.md" << EOF
# BelizeChain Testnet Deployment Instructions

## Prerequisites
- Ubuntu 22.04 LTS or newer
- 4+ CPU cores
- 8+ GB RAM
- 100+ GB SSD storage
- Public IP address
- Open ports: 30333 (P2P), 9933 (RPC), 9944 (WS), 9615 (Prometheus)

## Installation Steps

### 1. Create user
```bash
sudo useradd -m -s /bin/bash belizechain
sudo mkdir -p /opt/belizechain /var/lib/belizechain
sudo chown belizechain:belizechain /opt/belizechain /var/lib/belizechain
```

### 2. Copy binary and config
```bash
sudo cp belizechain-node /opt/belizechain/
sudo cp testnet-chain-spec-raw.json /opt/belizechain/
$RESERVED_NODE_COPY_STEP
sudo chmod +x /opt/belizechain/belizechain-node
```

### 3. Install systemd service
```bash
sudo cp belizechain-testnet.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable belizechain-testnet
sudo systemctl start belizechain-testnet
```

### 4. Verify installation
```bash
sudo systemctl status belizechain-testnet
sudo journalctl -u belizechain-testnet -f
```

### 5. Check node health
```bash
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  http://localhost:9933
```

## Notes
$RESERVED_NODE_NOTE
- The packaged service validates that the supplied raw chain spec is not local and does not contain loopback bootnodes before startup.

## Monitoring
- Prometheus metrics: http://<node-ip>:9615/metrics
- Logs: sudo journalctl -u belizechain-testnet -f

## Maintenance
- Update: Replace binary and sudo systemctl restart belizechain-testnet
- Stop: sudo systemctl stop belizechain-testnet
- Logs: sudo journalctl -u belizechain-testnet --since "1 hour ago"

## Backup
```bash
sudo systemctl stop belizechain-testnet
sudo tar czf belizechain-backup-$(date +%Y%m%d).tar.gz /var/lib/belizechain
sudo systemctl start belizechain-testnet
```
EOF
echo "✅ Deployment instructions created"

echo "📦 Creating deployment archive..."
tar czf "$DEPLOY_DIR.tar.gz" -C "$ROOT_DIR" "$(basename "$DEPLOY_DIR")"
echo "✅ Archive created: $DEPLOY_DIR.tar.gz"
echo ""

rm -rf "$DEPLOY_DIR"

echo "🎉 Deployment package ready!"
echo ""
echo "📦 Package: $DEPLOY_DIR.tar.gz"
echo ""
echo "Next steps:"
echo "  1. Transfer package to server: scp $DEPLOY_DIR.tar.gz user@server:/tmp/"
echo "  2. Extract: tar xzf /tmp/$(basename "$DEPLOY_DIR").tar.gz"
echo "  3. Follow instructions in DEPLOY.md"
