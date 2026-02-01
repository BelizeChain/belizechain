# Multi-Repository Architecture Overview

**BelizeChain's Modular Component Structure**

BelizeChain uses a multi-repository architecture for independent development, deployment, and scaling of specialized components.

---

## Repository Structure

| Repository | Purpose | Language | Size | Status |
|------------|---------|----------|------|--------|
| **belizechain/belizechain** | Core blockchain (15 pallets, runtime) | Rust | ~500 MB | Production |
| **belizechain/kinich-quantum** | Quantum computing integration | Python | ~50 MB | Production |
| **belizechain/nawal-ai** | Federated learning AI | Python | ~120 MB | Production |
| **belizechain/pakit-storage** | DAG storage system | Python | ~80 MB | Production |
| **belizechain/gem** | ink! smart contracts platform | Rust | ~30 MB | Production |
| **belizechain/ui** | Maya Wallet + Blue Hole Portal | TypeScript | ~200 MB | Production |
| **belizechain/infra** | Infrastructure as Code | YAML/HCL | ~10 MB | Production |

---

## Component Integration Map

```
┌─────────────────────────────────────────────────────────┐
│                    BelizeChain Core                      │
│  Substrate Runtime + 15 Pallets (Rust)                  │
│  ws://localhost:9944                                     │
└──────┬────────┬────────┬────────┬────────┬─────────────┘
       │        │        │        │        │
       ↓        ↓        ↓        ↓        ↓
┌──────────┐ ┌─────┐ ┌──────┐ ┌─────┐ ┌─────────┐
│  Nawal   │ │Kinich│ │Pakit │ │ GEM │ │   UI    │
│ AI (FL)  │ │Quantum│ │ DAG  │ │ink! │ │Wallets  │
│:8889     │ │:8888 │ │:8890 │ │:3000│ │:3001-06 │
└──────────┘ └─────┘ └──────┘ └─────┘ └─────────┘
```

---

## Communication Patterns

### RPC/WebSocket (Blockchain ↔ All)
- **Protocol**: Polkadot.js RPC
- **Port**: 9944 (WS), 9933 (HTTP)
- **Authentication**: None (public RPC)
- **Used By**: All components query blockchain state

### HTTP REST (Component ↔ Component)
- **Nawal → Blockchain**: Submit PoUW rewards (POST /staking/report_training)
- **Kinich → Blockchain**: Submit PQW proofs (POST /consensus/submit_quantum_work)
- **Pakit → Blockchain**: Register storage proofs (POST /landledger/register_document_proof)
- **UI → All**: Query endpoints for dashboard data

### Message Queue (Future)
- **RabbitMQ**: Event-driven communication
- **Redis Pub/Sub**: Real-time updates
- **Status**: Planned for Phase 2 (Q2 2026)

---

## Deployment Topologies

### Development (Local)
```bash
# Terminal 1: Blockchain
./target/release/belizechain-node --dev --tmp

# Terminal 2: Nawal AI
cd nawal && python -m nawal.orchestrator server

# Terminal 3: Kinich Quantum  
cd kinich && python -m kinich.core.quantum_node

# Terminal 4: Pakit Storage
cd pakit && python -m pakit.node

# Terminal 5: UI Portals
cd ui && npm run dev:all

# All-in-one script
./scripts/start_dev.sh
```

### Production (Kubernetes)
```yaml
# Namespace: belizechain-prod
apiVersion: apps/v1
kind: Deployment
metadata:
  name: belizechain-core
spec:
  replicas: 3  # 3 validator nodes
  containers:
  - name: node
    image: belizechain/node:stable2512
    ports: [9944, 9933, 30333]
---
# Separate deployments for each component
# - nawal-ai (replicas: 2)
# - kinich-quantum (replicas: 1)  
# - pakit-storage (replicas: 5)
# - gem-faucet (replicas: 1)
# - ui-maya-wallet (replicas: 3)
# - ui-blue-hole-portal (replicas: 2)
```

### Cloud (Azure)
- **Blockchain Nodes**: 3x Standard_D8s_v5 (8 vCPU, 32 GB RAM)
- **Nawal AI**: 2x Standard_NC6s_v3 (6 vCPU, 112 GB, V100 GPU)
- **Kinich Quantum**: Azure Quantum workspace (pay-per-use)
- **Pakit Storage**: 5x Standard_E8s_v5 (8 vCPU, 64 GB, 4TB NVMe)
- **UI**: Azure Static Web Apps (CDN-backed)

---

## Dependency Flow

```
UI ────→ Blockchain ←──── Nawal
         ↓                 ↓
         Pakit ←────── Kinich
         ↓
         GEM
```

**Critical Path**:
1. Blockchain MUST be running first (foundation)
2. Pakit can start independently (storage layer)
3. Nawal/Kinich require blockchain (submit rewards/proofs)
4. UI requires blockchain + Pakit (queries + content resolution)
5. GEM requires blockchain (smart contract execution)

---

## Data Sovereignty

### Belizean-Only Infrastructure
- **Blockchain**: 100% Belize nodes (Belmopan, Belize City, San Pedro)
- **Pakit**: Sovereign DAG (no IPFS/Arweave gateways)
- **Nawal**: On-premise training (no cloud ML APIs)
- **Kinich**: Azure Quantum (West US) - acceptable for compute-only
- **UI**: Hosted in Belize (static assets on Pakit DAG)

### Foreign Dependencies (Acceptable)
- **Development Tools**: GitHub, npm registry, crates.io
- **Quantum Backends**: Azure Quantum, IBM Quantum (compute only, no data storage)
- **Telemetry**: Optional Prometheus/Grafana (can be self-hosted)

---

## Version Compatibility Matrix

| Core Runtime | Nawal | Kinich | Pakit | GEM | UI |
|--------------|-------|--------|-------|-----|-----|
| stable2512 | 2.1.0 | 1.8.0 | 3.0.0 | 1.2.0 | 2.5.0 |
| stable2509 | 2.0.x | 1.7.x | 2.9.x | 1.1.x | 2.4.x |
| stable2406 | 1.9.x | 1.6.x | 2.8.x | 1.0.x | 2.3.x |

**Breaking Changes**:
- Runtime upgrade requires Nawal/Kinich connector updates
- Pakit API v3.0 breaks compatibility with v2.x clients
- GEM contracts compiled with ink! 4.0 not backward compatible

---

## Development Workflow

### Feature Development
```bash
# 1. Create feature branch in respective repo
git checkout -b feature/new-pallet

# 2. Develop with mock integrations
export PAKIT_MOCK=true
export KINICH_MOCK=true
cargo test --workspace

# 3. Integration testing
./scripts/integration_test.sh --all-components

# 4. Submit PR to respective repository
gh pr create --title "Add XYZ pallet"
```

### Release Process
1. **Core Blockchain**: Tag release → Docker build → Deploy to testnet → Mainnet (1 week)
2. **Python Components**: PyPI publish → Update configs → Rolling deployment (1 day)
3. **UI**: npm build → Azure Static Web Apps → CDN propagation (1 hour)
4. **GEM Contracts**: Compile → Upload to faucet → Documentation update (immediate)

---

## Monitoring & Observability

### Prometheus Metrics
```yaml
# Each component exports metrics on :9090/metrics
- belizechain_block_height
- nawal_training_sessions_total
- kinich_quantum_jobs_queued
- pakit_dag_blocks_stored
- gem_contract_calls_total
- ui_active_users
```

### Centralized Logging
```
Loki Stack:
- Blockchain: /var/log/belizechain/*.log
- Nawal: /var/log/nawal/*.log (JSON format)
- Kinich: /var/log/kinich/*.log (JSON format)
- Pakit: /var/log/pakit/*.log (JSON format)
- UI: Azure App Insights

Query: {component="nawal"} |= "ERROR"
```

---

## Disaster Recovery

### Backup Strategy
- **Blockchain**: State snapshot every 1000 blocks (compressed ~2 GB)
- **Pakit**: Full DAG backup daily (rsync to 3 locations)
- **Nawal**: Model checkpoints every 100 epochs (~500 MB)
- **Kinich**: Quantum job queue state (Redis RDB)
- **GEM**: Compiled contracts + metadata (GitHub releases)

### Recovery Time Objectives
- **Blockchain**: 4 hours (restore from snapshot + replay 1000 blocks)
- **Pakit**: 2 hours (rsync 100 GB at 50 MB/s)
- **Nawal**: 30 minutes (restore checkpoint)
- **Kinich**: 10 minutes (Redis restore)
- **UI**: 5 minutes (redeploy from CDN)

---

## Related Documentation

- [Kinich Integration](./kinich-integration.md)
- [Nawal Integration](./nawal-integration.md)
- [Pakit Integration](./pakit-integration.md)
- [GEM Integration](./gem-integration.md)
- [UI Integration](./ui-integration.md)
- [INTEGRATION_ARCHITECTURE.md](../../INTEGRATION_ARCHITECTURE.md)
