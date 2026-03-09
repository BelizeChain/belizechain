# 🌐 BelizeChain Testnet Deployment Guide

**Version**: 1.0.0  
**Date**: November 4, 2025  
**Status**: Production Ready  

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Network Architecture](#network-architecture)
4. [Building the Node](#building-the-node)
5. [Genesis Configuration](#genesis-configuration)
6. [Boot Node Setup](#boot-node-setup)
7. [Validator Node Setup](#validator-node-setup)
8. [Full Node Setup](#full-node-setup)
9. [Monitoring & Maintenance](#monitoring--maintenance)
10. [Troubleshooting](#troubleshooting)

---

## Overview

The BelizeChain Testnet is a production-ready test environment for:
- Smart contract deployment and testing
- Validator onboarding and training
- Application integration testing
- Load testing and performance optimization
- Community governance testing

**Network Specifications**:
- **Consensus**: BABE (Block Production) + Grandpa (Finality)
- **Block Time**: 6 seconds
- **Finality**: ~30 seconds (5 blocks)
- **Validators**: 21 initial validators (expandable to 100)
- **Session Length**: 600 blocks (~1 hour)
- **Era Length**: 14,400 blocks (~24 hours)

---

## Prerequisites

### Hardware Requirements

#### Boot Node (Archive Node)
- **CPU**: 8+ cores
- **RAM**: 32 GB
- **Storage**: 1 TB NVMe SSD (archive mode)
- **Network**: 100 Mbps, static IP required
- **OS**: Ubuntu 22.04 LTS or Debian 11+

#### Validator Node
- **CPU**: 4+ cores (8 recommended)
- **RAM**: 16 GB (32 GB recommended)
- **Storage**: 500 GB NVMe SSD
- **Network**: 50 Mbps, static IP preferred
- **OS**: Ubuntu 22.04 LTS or Debian 11+

#### Full Node (RPC)
- **CPU**: 4+ cores
- **RAM**: 16 GB
- **Storage**: 250 GB SSD
- **Network**: 50 Mbps
- **OS**: Ubuntu 22.04 LTS or Debian 11+

### Software Requirements

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y \
    build-essential \
    git \
    clang \
    curl \
    libssl-dev \
    llvm \
    libudev-dev \
    pkg-config \
    protobuf-compiler

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Configure Rust
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown
```

---

## Network Architecture

### Topology

```
┌─────────────────────────────────────────────────────────────┐
│                    BelizeChain Testnet                      │
└─────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │  Boot Node 1 │ (Archive, Static IP)
                    │  (Belize)    │
                    └───────┬──────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   ┌────▼─────┐       ┌────▼─────┐       ┌────▼─────┐
   │ Boot     │       │ Boot     │       │ Boot     │
   │ Node 2   │       │ Node 3   │       │ Node 4   │
   │ (US)     │       │ (EU)     │       │ (APAC)   │
   └────┬─────┘       └────┬─────┘       └────┬─────┘
        │                   │                   │
   ┌────┴────┬──────────────┴──────┬───────────┴────┐
   │         │                     │                 │
┌──▼──┐   ┌──▼──┐              ┌──▼──┐          ┌──▼──┐
│Val 1│   │Val 2│     ...      │Val  │          │Full │
│     │   │     │              │ 21  │          │Node │
└─────┘   └─────┘              └─────┘          └─────┘

Validator Pool (21 initial)      RPC Nodes (Public)
```

### Network Roles

1. **Boot Nodes**: Entry points, peer discovery, archive mode
2. **Validators**: Block production, finalization, staking
3. **Full Nodes**: RPC endpoints, chain synchronization
4. **Light Clients**: Mobile/browser, minimal resources

---

## Building the Node

### Method 1: Build from Source (Recommended)

```bash
# Clone repository
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain

# Checkout testnet branch
git checkout testnet

# Build release binary (takes 20-40 minutes)
cargo build --release

# Verify build
./target/release/belizechain-node --version

# Install globally (optional)
sudo cp target/release/belizechain-node /usr/local/bin/
```

### Method 2: Docker

```bash
# Pull image
docker pull belizechain/node:testnet

# Run container
docker run -d \
    --name belizechain-node \
    -p 9944:9944 \
    -p 9615:9615 \
    -p 30333:30333 \
    -v /data/belizechain:/data \
    belizechain/node:testnet \
    --chain testnet \
    --base-path /data
```

### Method 3: Pre-built Binary (Fast)

```bash
# Download latest release
wget https://github.com/BelizeChain/belizechain/releases/download/v1.0.0-testnet/belizechain-node-linux-amd64

# Make executable
chmod +x belizechain-node-linux-amd64
sudo mv belizechain-node-linux-amd64 /usr/local/bin/belizechain-node

# Verify
belizechain-node --version
```

---

## Genesis Configuration

### Create Custom Chain Spec

```bash
# Generate base chain spec
belizechain-node build-spec --chain testnet > belizechain-testnet-plain.json

# Edit genesis (see below)
nano belizechain-testnet-plain.json

# Convert to raw format
belizechain-node build-spec \
    --chain belizechain-testnet-plain.json \
    --raw > belizechain-testnet-raw.json
```

### Genesis Configuration Example

```json
{
  "name": "BelizeChain Testnet",
  "id": "belizechain_testnet",
  "chainType": "Live",
  "bootNodes": [
    "/dns/boot1.testnet.belizechain.org/tcp/30333/p2p/12D3KooW...",
    "/dns/boot2.testnet.belizechain.org/tcp/30333/p2p/12D3KooW...",
    "/dns/boot3.testnet.belizechain.org/tcp/30333/p2p/12D3KooW...",
    "/dns/boot4.testnet.belizechain.org/tcp/30333/p2p/12D3KooW..."
  ],
  "telemetryEndpoints": [
    ["wss://telemetry.belizechain.org/submit/", 0]
  ],
  "protocolId": "bzc-testnet",
  "properties": {
    "tokenSymbol": "tDALLA",
    "tokenDecimals": 12,
    "ss58Format": 42
  },
  "genesis": {
    "runtime": {
      "system": {},
      "balances": {
        "balances": [
          ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1000000000000000000],
          ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 500000000000000000]
        ]
      },
      "babe": {
        "authorities": [
          "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
          "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
        ]
      },
      "grandpa": {
        "authorities": [
          ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1],
          ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 1]
        ]
      },
      "sudo": {
        "key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
      },
      "belizeEconomy": {
        "treasuryAccount": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "initialSupply": 1000000000000000000,
        "inflationRate": 5
      }
    }
  }
}
```

---

## Boot Node Setup

Boot nodes are critical infrastructure for network health.

### 1. Generate Node Key

```bash
# Generate libp2p key
belizechain-node key generate-node-key --file /data/belizechain/node-key

# Get peer ID
belizechain-node key inspect-node-key --file /data/belizechain/node-key
# Output: 12D3KooWYourPeerIdHere...
```

### 2. Systemd Service

Create `/etc/systemd/system/belizechain-boot.service`:

```ini
[Unit]
Description=BelizeChain Boot Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=belizechain
Group=belizechain
WorkingDirectory=/home/belizechain
ExecStart=/usr/local/bin/belizechain-node \
    --base-path /data/belizechain \
    --chain /home/belizechain/belizechain-testnet-raw.json \
    --name "BelizeChain Boot Node 1" \
    --node-key-file /data/belizechain/node-key \
    --port 30333 \
    --rpc-port 9944 \
    --prometheus-port 9615 \
    --rpc-cors all \
    --rpc-methods Safe \
    --rpc-external \
    --ws-external \
    --pruning archive \
    --telemetry-url "wss://telemetry.belizechain.org/submit/ 0"

Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

### 3. Start Boot Node

```bash
# Create user
sudo useradd -r -s /bin/bash -m belizechain

# Set permissions
sudo mkdir -p /data/belizechain
sudo chown -R belizechain:belizechain /data/belizechain

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable belizechain-boot
sudo systemctl start belizechain-boot

# Check status
sudo systemctl status belizechain-boot
sudo journalctl -u belizechain-boot -f
```

### 4. Configure Firewall

```bash
# Allow P2P
sudo ufw allow 30333/tcp

# Allow RPC (if public)
sudo ufw allow 9944/tcp

# Allow Prometheus metrics
sudo ufw allow 9615/tcp from 10.0.0.0/8

# Enable firewall
sudo ufw enable
```

---

## Validator Node Setup

### 1. Generate Session Keys

```bash
# Method 1: RPC call (node must be running)
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' \
     http://localhost:9944

# Output: "0x1234...abcd" (your session keys)

# Method 2: Command line
belizechain-node key generate --scheme Sr25519 --output-type Json
```

### 2. Create Validator Account

```bash
# Generate stash account (holds funds)
belizechain-node key generate --scheme Sr25519

# Generate controller account (manages stash)
belizechain-node key generate --scheme Sr25519

# Store mnemonics securely (hardware wallet recommended)
```

### 3. Systemd Service

Create `/etc/systemd/system/belizechain-validator.service`:

```ini
[Unit]
Description=BelizeChain Validator Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=belizechain
Group=belizechain
WorkingDirectory=/home/belizechain
ExecStart=/usr/local/bin/belizechain-node \
    --base-path /data/belizechain \
    --chain /home/belizechain/belizechain-testnet-raw.json \
    --name "Validator-YourName" \
    --validator \
    --port 30333 \
    --rpc-port 9944 \
    --prometheus-port 9615 \
    --rpc-methods Safe \
    --no-telemetry \
    --pruning 1000 \
    --bootnodes /dns/boot1.testnet.belizechain.org/tcp/30333/p2p/12D3KooW...

Restart=always
RestartSec=10
LimitNOFILE=65536

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/data/belizechain

[Install]
WantedBy=multi-user.target
```

### 4. Set Session Keys (via UI or CLI)

Using Polkadot.js Apps:

1. Navigate to https://polkadot.js.org/apps/
2. Connect to your validator node (ws://localhost:9944)
3. Go to **Developer → Extrinsics**
4. Select **session → setKeys**
5. Paste your session keys (from step 1)
6. Set proof to `0x00`
7. Submit transaction

Using CLI:

```bash
# This requires polkadot.js CLI tool
npm install -g @polkadot/api-cli

polkadot-js-api \
    --ws ws://localhost:9944 \
    tx.session.setKeys \
    '["0x1234...abcd", "0x00"]' \
    --seed "your mnemonic phrase"
```

### 5. Bond Funds & Validate

1. **Bond DALLA**: Minimum 10,000 DALLA
2. **Set controller**: Separate account for management
3. **Set session keys**: From step 1
4. **Start validating**: Call `staking.validate()`

---

## Full Node Setup (RPC)

Public RPC nodes for dApp developers.

### Systemd Service

Create `/etc/systemd/system/belizechain-rpc.service`:

```ini
[Unit]
Description=BelizeChain RPC Node
After=network.target

[Service]
Type=simple
User=belizechain
ExecStart=/usr/local/bin/belizechain-node \
    --base-path /data/belizechain \
    --chain /home/belizechain/belizechain-testnet-raw.json \
    --name "RPC-Public" \
    --rpc-port 9944 \
    --ws-port 9945 \
    --rpc-cors all \
    --rpc-methods Safe \
    --rpc-external \
    --ws-external \
    --ws-max-connections 1000 \
    --pruning 256 \
    --bootnodes /dns/boot1.testnet.belizechain.org/tcp/30333/p2p/12D3KooW...

Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

### Nginx Reverse Proxy (SSL)

```nginx
upstream belizechain_rpc {
    server 127.0.0.1:9944;
}

upstream belizechain_ws {
    server 127.0.0.1:9945;
}

server {
    listen 443 ssl http2;
    server_name rpc.testnet.belizechain.org;

    ssl_certificate /etc/letsencrypt/live/rpc.testnet.belizechain.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.testnet.belizechain.org/privkey.pem;

    # RPC HTTP
    location / {
        proxy_pass http://belizechain_rpc;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_http_version 1.1;
    }

    # WebSocket
    location /ws {
        proxy_pass http://belizechain_ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }
}
```

---

## Monitoring & Maintenance

### Health Checks

```bash
# Check sync status
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
     http://localhost:9944

# Check peers
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers"}' \
     http://localhost:9944

# Check block height
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader"}' \
     http://localhost:9944
```

### Log Rotation

Create `/etc/logrotate.d/belizechain`:

```
/var/log/belizechain/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    copytruncate
}
```

### Database Pruning

```bash
# Stop node
sudo systemctl stop belizechain-validator

# Prune state (keeps last 256 blocks)
belizechain-node purge-chain \
    --base-path /data/belizechain \
    --chain belizechain-testnet-raw.json \
    --keep-blocks 256

# Restart
sudo systemctl start belizechain-validator
```

---

## Troubleshooting

### Node Won't Start

```bash
# Check logs
sudo journalctl -u belizechain-validator -n 100

# Common issues:
# 1. Port already in use
sudo lsof -i :30333

# 2. Permission denied
sudo chown -R belizechain:belizechain /data/belizechain

# 3. Missing chain spec
ls -la /home/belizechain/*.json
```

### Sync Issues

```bash
# Clear state and resync
sudo systemctl stop belizechain-validator
rm -rf /data/belizechain/chains/*/db/full
sudo systemctl start belizechain-validator

# Use warp sync (faster)
# Add to service: --sync warp
```

### Validator Not Producing Blocks

1. **Check session keys**: Ensure `setKeys` transaction succeeded
2. **Check stake**: Must have minimum 10,000 DALLA bonded
3. **Check validator set**: May need to wait for next era
4. **Check peers**: Must be connected to other validators

```bash
# Verify session keys
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_hasSessionKeys", "params":["0x..."]}' \
     http://localhost:9944
```

---

## Next Steps

- ✅ **Validator Registration**: [VALIDATOR_GUIDE.md](VALIDATOR_GUIDE.md)
- ✅ **Monitoring Setup**: [MONITORING_GUIDE.md](MONITORING_GUIDE.md)
- ✅ **Disaster Recovery**: [DISASTER_RECOVERY.md](DISASTER_RECOVERY.md)
- ✅ **Security Best Practices**: [SECURITY.md](SECURITY.md)

---

## Support

- **Discord**: https://discord.gg/belizechain
- **Forum**: https://forum.belizechain.org
- **Email**: validators@belizechain.org
- **Emergency**: emergency@belizechain.org

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
