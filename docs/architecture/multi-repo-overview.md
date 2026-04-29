# Multi-Repository Architecture Overview

**BelizeChain's Modular Component Structure**

BelizeChain uses a multi-repository architecture for independent development, deployment, and scaling of specialized components.

---

## Repository Structure

| Repository | Purpose | Language | Size | Status |
|------------|---------|----------|------|--------|
| **belizechain/belizechain** | Core blockchain (18 Belize-specific pallets + runtime) | Rust | ~500 MB | Active on Ceiba |
| **belizechain/kinich-quantum** | Quantum computing integration | Python | ~50 MB | Live on Ceiba; activation workflows pending |
| **belizechain/nawal-ai** | Federated learning AI | Python | ~120 MB | Live on Ceiba; FL workload and Prometheus metrics pending |
| **belizechain/pakit-storage** | DAG storage system | Python | ~80 MB | Live on Ceiba; DAG activation workflows pending |
| **belizechain/gem** | ink! smart contracts platform | Rust | ~30 MB | Contracts repo active; Ceiba address wiring pending |
| **belizechain/ui** | Maya Wallet + Blue Hole Portal | TypeScript | ~200 MB | Blue Hole Portal live on Ceiba |
| **belizechain/infra** | Infrastructure as Code | YAML/HCL | ~10 MB | Active Ceiba compose source |

---

## Component Integration Map

```
┌─────────────────────────────────────────────────────────┐
│                    BelizeChain Core                      │
│  Substrate Runtime + 18 Belize Pallets (Rust)           │
│  Ceiba RPC: http://100.81.45.25:9944                     │
└──────┬────────┬────────┬────────┬────────┬─────────────┘
       │        │        │        │        │
       ↓        ↓        ↓        ↓        ↓
┌──────────┐ ┌─────┐ ┌──────┐ ┌─────┐ ┌─────────┐
│  Nawal   │ │Kinich│ │Pakit │ │ GEM │ │   UI    │
│ AI (FL)  │ │Quantum│ │ DAG  │ │ink! │ │Wallets  │
│:8080     │ │:8888 │ │:8001 │ │contracts│ │:3000 │
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

### Message Queue / Event Bus (Planned)
- **RabbitMQ**: Event-driven communication
- **Redis Pub/Sub**: Real-time updates
- **Status**: Planned, but not yet the operational source of truth for Ceiba

### Meshtastic LoRa Mesh (Off-Grid)
- **Protocol**: Meshtastic (LoRa 915 MHz + BLE)
- **Phone → Radio**: Bluetooth Low Energy (BLE) to Meshtastic hardware
- **Radio → Mesh**: LoRa 915 MHz ISM band, up to 7 hops, 1-25 km per hop
- **Mesh → Blockchain**: Gateway nodes bridge mesh transactions to RPC
- **Hardware**: T-Beam, Heltec V3, RAK WisBlock, Station G2
- **Use Cases**: Off-grid payments (rural, cayes, jungle), emergency alerts (NEMO), validator consensus fallback
- **Managed By**: `pallet-belize-mesh` (on-chain), Maya Wallet BLE integration (UI repo)

---

## Deployment Topologies

### Development (Local)

If sibling repos are cloned adjacent to this repo, a local layout may look like this:

```bash
# Terminal 1: Blockchain
./target/release/belizechain-node --dev --tmp

# Terminal 2: Nawal AI
cd ../nawal-ai && python -m nawal.orchestrator server

# Terminal 3: Kinich Quantum  
cd ../kinich-quantum && python -m kinich.core.quantum_node

# Terminal 4: Pakit Storage
cd ../pakit-storage && python -m pakit.node

# Terminal 5: UI Portals
cd ../ui && npm run dev:all
```

### Production (Ceiba Self-Hosted)
```bash
# Core node and sibling services are operated via Docker Compose on Ceiba.
cd /opt/belizechain
docker compose ps
docker logs --tail 100 ceiba-node

# Dated baseline snapshot:
# docs/operations/CEIBA_BASELINE_2026-04-29.md
```

### Host Allocation (Ceiba)
- **Blockchain Node**: live single-validator testnet process on Ceiba
- **Nawal AI**: live CPU federated-learning API behind `/api/nawal`
- **Kinich Quantum**: live quantum API behind `/api/kinich`
- **Pakit Storage**: live DAG storage API behind `/api/pakit`
- **UI**: Blue Hole Portal live behind Nginx; Maya Wallet public exposure requires a separate routing decision
- **GEM**: contract deployment and address wiring pending

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
- **Kinich**: self-hosted service with configurable backend providers
- **UI**: Hosted in Belize on Ceiba/self-hosted stack

### Foreign Dependencies (Acceptable)
- **Development Tools**: GitHub, npm registry, crates.io
- **Quantum Backends**: configurable providers (compute only, no on-chain data storage)
- **Telemetry**: Optional Prometheus/Grafana (can be self-hosted)

---

## Compatibility Policy

- The core runtime currently targets Polkadot SDK `stable2603`.
- Exact sibling compatibility must be verified from each repo's active branch, PRs, workflow runs, or releases before rollout.
- Do not treat this document as the source of truth for exact sibling version numbers.
- Runtime upgrades may require connector updates in Nawal, Kinich, Pakit, UI, or GEM integration paths.

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
# Run the core workspace tests plus the relevant test suites in each touched sibling repo.

# 4. Submit PR to respective repository
gh pr create --title "Add XYZ pallet"
```

### Release Process
1. **Core Blockchain**: Tag release → Docker build → Deploy to testnet → Mainnet (1 week)
2. **Python Components**: PyPI publish → Update configs → Rolling deployment (1 day)
3. **UI**: npm build → Ceiba reverse proxy deployment (1 hour)
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
- UI: self-hosted logs and Prometheus-compatible metrics

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
