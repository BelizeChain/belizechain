# BelizeChain Node

The sovereign blockchain node implementation for Belize, built on Substrate framework with Polkadot SDK stable2512.

## 📦 Overview

This package implements the BelizeChain node binary, which runs the blockchain network and exposes:
- P2P networking for block propagation and consensus
- RPC endpoints for client applications (Maya Wallet, Blue Hole Portal)
- Consensus mechanism (Aura + GRANDPA)
- Runtime integration with all 16 custom pallets

## 🏗️ Architecture

### Core Components

| File | Purpose |
|------|---------|
| `main.rs` | Node entry point and CLI handling |
| `cli.rs` | Command-line interface definition |
| `service.rs` | Node service construction and network backends |
| `rpc.rs` | RPC method registration (System, TransactionPayment) |
| `chain_spec.rs` | Genesis block configuration for different networks |
| `chain_spec_configs.rs` | Network preset configurations (dev/testnet/mainnet) |
| `validator_config.rs` | Production validator settings and bootnode addresses |
| `build.rs` | Build script for version injection |

## 🚀 Usage

### Development Mode
```bash
# Fast local testing with Alice as validator
./target/release/belizechain-node --dev --tmp

# Persist data to disk
./target/release/belizechain-node --dev
```

### Local Testnet (Multi-Node)
```bash
# Node 1 (Alice)
./target/release/belizechain-node \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --rpc-port 9944 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --validator

# Node 2 (Bob)
./target/release/belizechain-node \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --rpc-port 9945 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
  --validator
```

### Mainnet Node
```bash
# Full node (non-validator)
./target/release/belizechain-node \
  --base-path /var/lib/belizechain \
  --chain belize \
  --port 30333 \
  --rpc-port 9944 \
  --prometheus-port 9615

# Validator node (requires staking)
./target/release/belizechain-node \
  --base-path /var/lib/belizechain \
  --chain belize \
  --validator \
  --name "MyValidatorNode" \
  --port 30333 \
  --rpc-port 9944 \
  --prometheus-port 9615
```

## 🌐 Chain Specifications

### Built-in Chain IDs

| Chain ID | Description | Sudo | Validators | Block Time |
|----------|-------------|------|------------|------------|
| `dev` | Development (Alice) | ✅ | 1 (Alice) | 3s |
| `local` | Local testnet | ✅ | 2 (Alice, Bob) | 6s |
| `belize` | Mainnet | ❌ | 30+ | 6s |

### Custom Chain Spec
```bash
# Export existing chain spec to JSON
./target/release/belizechain-node build-spec --chain dev > custom-spec.json

# Edit custom-spec.json as needed, then build raw spec
./target/release/belizechain-node build-spec --chain custom-spec.json --raw > custom-raw.json

# Run node with custom spec
./target/release/belizechain-node --chain custom-raw.json
```

## 🔌 RPC Endpoints

### Default Ports
- **RPC WebSocket**: `ws://127.0.0.1:9944`
- **RPC HTTP**: `http://127.0.0.1:9944`
- **Prometheus Metrics**: `http://127.0.0.1:9615/metrics`

### Available RPC Methods
- **System**: `system_*` - Node information and health
- **Chain**: `chain_*` - Block and state queries
- **Author**: `author_*` - Transaction submission
- **Payment**: `payment_*` - Fee estimation
- **State**: `state_*` - Runtime storage queries

## 🏛️ Genesis Configuration

### Testnet Genesis (Development/Local)
- **Initial Balance**: 1,000,000 DALLA per account
- **Authorities**: Alice (dev) or Alice + Bob (local)
- **Identity Issuers**: Alice acts as SSN/Passport/Biometric issuer
- **Council Members**: 3 initial members with rank 1
- **Operation Fee**: 10 DALLA for identity operations
- **Rate Limits**: 500 SSN, 200 Passports, 100 Biometrics per day

### Mainnet Genesis (Production)
- **Initial Supply**: 100,000,000 DALLA (100M total supply)
- **Treasury Allocation**: 500M DALLA (multi-sig controlled)
- **Staking Rewards Pool**: 300M DALLA (10-year emission)
- **Development Fund**: 100M DALLA
- **Validators**: 30 initial validators (configured via secure key management)
- **Sudo**: Disabled (governance-only control)
- **Operation Fee**: 50 DALLA for identity operations
- **Stricter Rate Limits**: 200 SSN, 100 Passports, 50 Biometrics per day

## ⚙️ Validator Configuration

### Network Presets

| Environment | Min Peers | Max Peers | mDNS | Telemetry | Block Time |
|-------------|-----------|-----------|------|-----------|------------|
| Development | 0 | 25 | ✅ | ❌ | 3s |
| Testnet | 5 | 50 | ❌ | ✅ | 6s |
| Mainnet | 10 | 100 | ❌ | ✅ | 6s |
| Bootnode | 0 | 500 | ❌ | ❌ | 6s |

### Mainnet Bootnodes
```
/dns4/bootnode1.belizechain.org/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp (Belize City)
/dns4/bootnode2.belizechain.org/tcp/30333/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD (San Pedro)
/dns4/bootnode3.belizechain.org/tcp/30333/p2p/12D3KooWLmrYDLoNTyTYtRdDyZLWDe1paxzxTw5RgjmHLfzW96SX (Belmopan)
```

## 🔧 Build Instructions

### Development Build (Fast)
```bash
cargo build -p belizechain-node
```

### Release Build (Optimized)
```bash
cargo build --release -p belizechain-node
# Binary: ./target/release/belizechain-node (~150MB)
```

### Check Compilation
```bash
cargo check -p belizechain-node
```

## 📊 Telemetry

Mainnet and testnet nodes report to:
- **Endpoint**: `wss://telemetry.belizechain.org:443/submit/`
- **Dashboard**: `https://telemetry.belizechain.org`

Disable telemetry with `--no-telemetry` flag.

## 🔐 Security Considerations

### Production Deployment Checklist
- [ ] **NEVER** use development keys (Alice/Bob) in production
- [ ] Configure validator keys via secure key management system
- [ ] Disable `--dev` and `--unsafe-*` flags
- [ ] Enable firewall rules (allow P2P on 30333, restrict RPC to localhost)
- [ ] Run behind reverse proxy for RPC access
- [ ] Enable Prometheus monitoring
- [ ] Configure automated backups of chain data
- [ ] Set up alerting for node downtime
- [ ] Use systemd service for auto-restart
- [ ] Rotate session keys regularly

### Key Management
```bash
# Generate session keys
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' \
  http://localhost:9944

# Insert keys (development only - use secure storage in production)
./target/release/belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain belize \
  --scheme Sr25519 \
  --suri "your-secret-phrase" \
  --key-type aura
```

## 📚 Related Documentation

- **Runtime**: `../runtime/README.md` - Pallet integration and runtime logic
- **Pallets**: `../pallets/*/README.md` - Individual pallet documentation
- **Developer Guide**: `../../docs/developer-guides/DEVELOPMENT_GUIDE.md`
- **Deployment Guide**: `../../docs/deployment/NODE_DEPLOYMENT.md`

## 🐛 Troubleshooting

### Common Issues

**Problem**: `Wasm runtime not available`
```bash
# Solution: Rebuild with WASM runtime included
cargo build --release -p belizechain-runtime
cargo build --release -p belizechain-node
```

**Problem**: `Error creating runtime`
```bash
# Solution: Clear state and restart
./target/release/belizechain-node purge-chain --dev -y
./target/release/belizechain-node --dev
```

**Problem**: Peer connection issues
```bash
# Solution: Check firewall and enable mDNS for local networks
./target/release/belizechain-node --dev --discover-local
```

## 📜 License

MIT License - Copyright (c) 2024-2026 BelizeChain Core Team
