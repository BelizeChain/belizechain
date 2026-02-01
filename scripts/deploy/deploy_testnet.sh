#!/bin/bash
# BelizeChain Testnet Deployment Script
# Deploys BelizeChain node to testnet environment

set -e

echo "🚀 BelizeChain Testnet Deployment"
echo "==================================="
echo ""

# Check prerequisites
echo "📋 Pre-deployment checks..."

# 1. Check if binary exists
if [ ! -f "./target/release/belizechain-node" ]; then
    echo "❌ Node binary not found"
    echo "💡 Build with: ./scripts/build_release.sh"
    exit 1
fi
echo "✅ Node binary found"

# 2. Validate environment
if [ ! -f "./scripts/deploy/validate_env.sh" ]; then
    echo "❌ Environment validation script not found"
    exit 1
fi

echo "🔍 Validating deployment environment..."
bash ./scripts/deploy/validate_env.sh
echo ""

# 3. Run tests
echo "🧪 Running pre-deployment tests..."
if ! ./scripts/run_all_tests.sh; then
    echo "❌ Tests failed - deployment aborted"
    exit 1
fi
echo ""

# 4. Create deployment package
echo "📦 Creating deployment package..."
DEPLOY_DIR="./deploy-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$DEPLOY_DIR"

# Copy binary
cp ./target/release/belizechain-node "$DEPLOY_DIR/"
echo "✅ Binary copied"

# Copy chain spec
if [ -f "./node/res/testnet-spec.json" ]; then
    cp ./node/res/testnet-spec.json "$DEPLOY_DIR/"
    echo "✅ Chain spec copied"
fi

# Create systemd service file
cat > "$DEPLOY_DIR/belizechain-testnet.service" << 'EOF'
[Unit]
Description=BelizeChain Testnet Node
After=network.target

[Service]
Type=simple
User=belizechain
Group=belizechain
WorkingDirectory=/opt/belizechain
ExecStart=/opt/belizechain/belizechain-node \
    --chain=testnet \
    --validator \
    --name="Testnet-Validator-1" \
    --base-path=/var/lib/belizechain \
    --rpc-cors=all \
    --rpc-external \
    --ws-external \
    --prometheus-external
Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
echo "✅ Systemd service file created"

# Create deployment instructions
cat > "$DEPLOY_DIR/DEPLOY.md" << 'EOF'
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
sudo cp testnet-spec.json /opt/belizechain/ # if exists
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
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933
```

## Monitoring
- Prometheus metrics: http://<node-ip>:9615/metrics
- Logs: `sudo journalctl -u belizechain-testnet -f`

## Maintenance
- Update: Replace binary and `sudo systemctl restart belizechain-testnet`
- Stop: `sudo systemctl stop belizechain-testnet`
- Logs: `sudo journalctl -u belizechain-testnet --since "1 hour ago"`

## Backup
```bash
sudo systemctl stop belizechain-testnet
sudo tar czf belizechain-backup-$(date +%Y%m%d).tar.gz /var/lib/belizechain
sudo systemctl start belizechain-testnet
```
EOF
echo "✅ Deployment instructions created"

# Create archive
echo "📦 Creating deployment archive..."
tar czf "$DEPLOY_DIR.tar.gz" "$DEPLOY_DIR"
echo "✅ Archive created: $DEPLOY_DIR.tar.gz"
echo ""

# Cleanup temp directory
rm -rf "$DEPLOY_DIR"

echo "🎉 Deployment package ready!"
echo ""
echo "📦 Package: $DEPLOY_DIR.tar.gz"
echo ""
echo "Next steps:"
echo "  1. Transfer package to server: scp $DEPLOY_DIR.tar.gz user@server:/tmp/"
echo "  2. Extract: tar xzf /tmp/$DEPLOY_DIR.tar.gz"
echo "  3. Follow instructions in DEPLOY.md"
