# 🏛️ BelizeChain Complete Development Guide

**Sovereign Blockchain Infrastructure for Belize**

This is the master instruction file for AI coding agents working on BelizeChain. For component-specific details, refer to individual `.github-instructions.md` files in each module.

---

## 📋 Quick Navigation

| Component | Location | Instructions | Primary Language |
|-----------|----------|--------------|------------------|
| **Blockchain** | `belizechain/` | `.github/copilot-instructions.md` | Rust |
| **Federated AI** | `nawal/` | `.github-instructions.md` | Python |
| **Quantum** | `kinich/` | `.github-instructions.md` | Python |
| **Storage** | `pakit/` | `.github-instructions.md` | Python |
| **UI Portals** | `ui/` | `.github-instructions.md` | TypeScript |

---

## 🌍 BelizeChain Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      BELIZECHAIN ECOSYSTEM                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   LAYER 1: BLOCKCHAIN CORE                      │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐               │
│  │  Economy   │  │ Governance │  │ Compliance │               │
│  │  Staking   │  │  Identity  │  │   Oracle   │  13 Pallets  │
│  │  BelizeX   │  │ LandLedger │  │ Payroll    │               │
│  │Interop│Quantum│Consensus│Community│...      │               │
│  └────────────┘  └────────────┘  └────────────┘               │
│                                                                 │
│  Runtime: Substrate + Polkadot SDK stable2509                  │
│  Consensus: Proof of Useful Work (PoUW)                        │
│  Tokens: DALLA (native), bBZD (fiat-backed stablecoin, 1:1 BZD)  │
└─────────────────────────────────────────────────────────────────┘
                              │
                 ┌────────────┼────────────┐
                 ▼            ▼            ▼
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│  LAYER 2: NAWAL AI  │ │ LAYER 3: KINICH     │ │ LAYER 4: PAKIT      │
│  Federated Learning │ │ Quantum Computing   │ │ Storage System      │
│                     │ │                     │ │                     │
│  • DP-SGD Privacy   │ │  • configured quantum backend    │ │  • Quantum Compress │
│  • Genome Evolution │ │  • Multi-Backend    │ │  • Deduplication    │
│  • PoUW Integration │ │  • PQW Rewards      │ │  • IPFS/Arweave     │
│  • Multilingual     │ │  • QKD Security     │ │  • Blockchain Proofs│
└─────────────────────┘ └─────────────────────┘ └─────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    LAYER 5: USER INTERFACES                     │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  │
│  │ Maya │  │ Blue │  │Winik │  │ Pek  │  │Gúbida│  │Kijka │  │
│  │Wallet│  │ Hole │  │  Gov │  │ Biz  │  │Valid.│  │Explor│  │
│  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘  │
│  Citizens  Govt     Districts  Tourism  Validators  Public     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 💰 Critical: Currency & Economics

### **bBZD Stablecoin - BZD Peg**
**🚨 IMPORTANT**: bBZD is pegged **1:1 to the Belize Dollar (BZD)**, NOT USD!

**Rationale**: Belize is a Commonwealth nation with historical ties to the UK. There is NO association with the USA.

```rust
// ✅ CORRECT: Display and documentation
"bBZD is pegged 1:1 with BZD (Belize Dollar)"
"1 bBZD = $1 BZD (central bank governed)"

// ❌ WRONG: Never reference USD peg
"1 bBZD = $1 USD"  // INCORRECT!
```

### Currency Formatting
```typescript
// TypeScript/JavaScript
formatCurrency(1000, "bBZD")  // "BZ$1,000" (Belize Dollar)
formatCurrency(500, "DALLA")  // "500 DALLA"

// Python
f"{amount} bBZD (BZ${amount} BZD)"

// Rust
format!("{} bBZD (pegged to BZD)", amount)
```

### Tokenomics
- **DALLA**: Native token, 12 decimals, inflation-based
- **bBZD**: BZD-pegged stablecoin, 6 decimals, governance-maintained peg
- **Mahogany**: Smallest unit (1 DALLA = 1,000,000 Mahogany)

---

## 🏗️ Component Interaction Map

### How Components Communicate

```
┌────────────────────────────────────────────────────────────────┐
│                    BLOCKCHAIN (Rust)                           │
│                                                                │
│  Staking Pallet ──┬──> Nawal PoUW Rewards                     │
│                   └──> Kinich PQW Rewards                      │
│                                                                │
│  Economy Config ───────> bBZD/BZD Peg Target (primary)        │
│                                                                │
│  LandLedger Pallet ───> Pakit Document Storage (IPFS CIDs)    │
│                                                                │
│  Compliance Pallet ───> Nawal KYC Verification Required       │
└────────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  Nawal (Python) │  │ Kinich (Python) │  │ Pakit (Python)  │
│                 │  │                 │  │                 │
│  Reports to:    │  │  Reports to:    │  │  Stores in:     │
│  • Staking      │  │  • Consensus    │  │  • IPFS         │
│  (training)     │  │  (quantum work) │  │  • Arweave      │
│                 │  │                 │  │  • Local        │
│  Uses:          │  │  Uses:          │  │                 │
│  • Kinich       │  │  • configured quantum backend│  │  Proofs in:     │
│  (quantum ML)   │  │  • IBM Quantum  │  │  • LandLedger   │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │                    │                    │
         └────────────────────┴────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   UI (TypeScript)│
                    │                  │
                    │  Connects via:   │
                    │  • Polkadot.js   │
                    │  • WebSocket     │
                    │  (ws://9944)     │
                    └──────────────────┘
```

---

## 🔐 Security & Compliance Principles

### 1. Data Sovereignty
- All Belizean citizen data MUST stay in Belize or trusted Commonwealth jurisdictions
- Kinich quantum jobs on sensitive data: force local execution only
- Pakit storage: prefer IPFS nodes in Belize, Arweave for archival

### 2. Privacy Requirements
- **Nawal**: Differential Privacy (DP-SGD) with ε=3.0 for national datasets
- **Kinich**: Quantum Key Distribution (QKD) for secure communications
- **UI**: Never store private keys in localStorage (use Polkadot extension)

### 3. KYC/AML Compliance
- All financial operations require KYC verification via Compliance pallet
- Thresholds: 
  - Level 0 (None): 1,000 DALLA/transaction
  - Level 1 (Basic): 10,000 DALLA/transaction
  - Level 2 (Enhanced): 100,000 DALLA/transaction
  - Level 3 (Full): Unlimited

### 4. Multi-Signature Requirements
- Government treasury: 4-of-7 multi-sig
- Large land transfers (>$500k GBP): 2-of-3 multi-sig
- Emergency governance: 5-of-9 multi-sig

---

## 🛠️ Development Environment Setup

### Prerequisites
```bash
# System Requirements
- Rust: stable toolchain (rustc 1.77+)
- Python: 3.13+
- Node.js: 18+
- Docker: 20.10+
- Kubernetes: 1.28+ (production)
```

### Complete Setup
```bash
Clone the repository:

```bash
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain
```

# 2. Blockchain (Rust)
cd belizechain/
cargo build --release  # ~8 minutes
./target/release/belizechain-node --dev --tmp

# 3. Nawal AI (Python)
cd ../nawal/
python3.13 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python -m nawal.orchestrator server --config config.dev.yaml

# 4. Kinich Quantum (Python)
cd ../kinich/
pip install -e .
# Configure configured quantum backend credentials
az login
export AZURE_QUANTUM_WORKSPACE="<workspace-id>"

# 5. Pakit Storage (Python)
cd ../pakit/
pip install -e .
ipfs daemon  # Start IPFS

# 6. UI (TypeScript)
cd ../ui/
npm install
npm run dev:all  # Start all portals (Maya Wallet + Blue Hole Portal)
```

---

## 🚀 Common Development Workflows

### Workflow 1: Add New Blockchain Feature
```bash
# 1. Create pallet
cd belizechain/pallets/
cargo generate --git https://github.com/substrate-developer-hub/substrate-node-template pallet my-pallet

# 2. Implement logic in lib.rs
# See: .github/copilot-instructions.md for patterns

# 3. Add to runtime
# Edit: belizechain/runtime/src/lib.rs
# Add pallet to construct_runtime! macro

# 4. Build and test
cargo build --release
cargo test -p pallet-my-pallet

# 5. Update UI
cd ../../ui/maya-wallet/
# Add new feature to UI
```

### Workflow 2: Train Nawal AI Model
```bash
# 1. Prepare dataset (CSV, JSON, or text)
cd nawal/data/

# 2. Start federated learning server
python -m nawal.orchestrator server

# 3. Start validator clients (multiple terminals)
python -m nawal.client.train --validator-key keys/alice.json
python -m nawal.client.train --validator-key keys/bob.json
python -m nawal.client.train --validator-key keys/charlie.json

# 4. Monitor training
curl http://localhost:9090/metrics  # Prometheus metrics

# 5. Submit PoUW to blockchain
python -m nawal.blockchain.pouw_reporter submit
```

### Workflow 3: Execute Quantum Job
```bash
# 1. Create quantum circuit
cd kinich/examples/
python qaoa_maxcut.py --graph-size 10

# 2. Submit to Kinich
python -m kinich.core.quantum_node submit \
  --circuit circuit.qasm \
  --backend azure_quantum \
  --shots 1024

# 3. Wait for results
python -m kinich.core.job_scheduler status --job-id <id>

# 4. Submit PQW to blockchain
python -m kinich.blockchain.pqw_reporter submit --job-id <id>
```

### Workflow 4: Store Large File in Pakit
```bash
# 1. Compress and deduplicate
cd pakit/
pakit store /path/to/large-file.dat --backend ipfs

# 2. Get CID
# Output: "QmX... (CID)"

# 3. Register proof on-chain
python -m pakit.backends.blockchain_proofs register \
  --cid QmX... \
  --keypair /path/to/key.json

# 4. Verify storage
pakit verify QmX...
```

---

## 🧪 Testing Strategy

### Blockchain Tests
```bash
# Unit tests (individual pallets)
cargo test -p pallet-economy
cargo test -p pallet-staking

# Integration tests
cargo test --workspace

# Runtime tests
cd belizechain/runtime/
cargo test
```

### Python Component Tests
```bash
# Nawal tests
cd nawal/
pytest tests/ -v --cov=nawal

# Kinich tests
cd kinich/
pytest tests/ -v

# Pakit tests
cd pakit/
pytest tests/ -v
```

### UI Tests
```bash
# Unit tests
cd ui/
npm run test

# E2E tests
npm run test:e2e

# Component tests
npm run test:component
```

---

## 📊 Monitoring & Observability

### Blockchain Metrics
```bash
# Prometheus endpoint
curl http://localhost:9615/metrics

# Key metrics:
# - substrate_block_height
# - substrate_finality_grandpa_round
# - substrate_network_peers
# - substrate_transactions_total
```

### Nawal Metrics
```bash
# Training metrics
curl http://localhost:9090/metrics

# Key metrics:
# - nawal_training_accuracy
# - nawal_model_loss
# - nawal_participants_active
# - nawal_privacy_epsilon
```

### Kinich Metrics
```bash
# Quantum job metrics
curl http://localhost:9091/metrics

# Key metrics:
# - kinich_jobs_total
# - kinich_job_duration_seconds
# - kinich_backend_availability
# - kinich_cost_usd_total
```

---

## 🐛 Common Issues Across All Components

### Issue 1: "Connection Refused" to Blockchain
```bash
# ❌ Problem: UI/Python can't connect to ws://localhost:9944

# ✅ Solution: Start blockchain node
cd belizechain/
./target/release/belizechain-node --dev --tmp --ws-external

# Verify it's running
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9944
```

### Issue 2: Out of Memory During Compilation
```bash
# ❌ Problem: cargo build killed (OOM)

# ✅ Solution: Reduce parallel jobs
cargo build --release -j 2  # Use only 2 cores

# Or: Increase swap space
sudo fallocate -l 8G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### Issue 3: Python Package Conflicts
```bash
# ❌ Problem: Incompatible package versions

# ✅ Solution: Use separate virtual environments
cd nawal/ && python -m venv .venv-nawal
cd kinich/ && python -m venv .venv-kinich
cd pakit/ && python -m venv .venv-pakit

# Activate specific environment when working
source .venv-nawal/bin/activate  # For Nawal
```

### Issue 4: TypeScript Type Errors
```bash
# ❌ Problem: "Property X does not exist on type Y"

# ✅ Solution: Update Polkadot types
cd ui/
npm install @polkadot/api@latest @polkadot/types@latest

# Regenerate types
npm run generate:types
```

---

## 📚 Documentation Structure

```
docs/
├── getting-started/          # Beginner guides
│   ├── what-is-belizechain.md
│   ├── your-first-transaction.md
│   ├── glossary.md
│   └── README.md
├── developer-guides/         # Technical guides
│   ├── pallet-development.md
│   ├── nawal-training.md
│   ├── kinich-quantum-jobs.md
│   └── ui-integration.md
├── technical-reference/      # API docs
│   ├── runtime-api.md
│   ├── rpc-methods.md
│   └── pallet-reference.md
├── tutorials/               # Step-by-step
│   ├── accept-payments.md
│   ├── create-proposal.md
│   └── run-validator.md
├── deployment/              # Production guides
│   ├── ceiba-operations.md
│   ├── monitoring.md
│   └── backup-recovery.md
└── archive/                # Historical docs
```

---

## 🎯 Project Status & Roadmap

### Phase 1: Foundation (✅ COMPLETE - October 2025)
- [x] 13 blockchain pallets
- [x] Polkadot SDK stable2509 migration
- [x] Zero-warning build
- [x] Nawal federated learning core
- [x] Kinich quantum orchestration
- [x] Pakit storage system
- [x] 6 UI portals (basic functionality)

### Phase 2: Integration (🚧 IN PROGRESS)
- [x] PoUW blockchain integration (Nawal)
- [x] PQW blockchain integration (Kinich)
- [ ] Pakit <-> LandLedger integration
- [ ] Full multilingual support (UI)
- [ ] Byzantine detection (Nawal)
- [ ] Quantum error mitigation (Kinich)

### Phase 3: Production Hardening (⏳ Q1 2026)
- [ ] Ceiba self-hosted deployment hardening
- [ ] High-availability clustering
- [ ] Automated monitoring & alerting
- [ ] Disaster recovery procedures
- [ ] Security audit (third-party)
- [ ] Load testing (1000+ TPS)

### Phase 4: Public Launch (⏳ Q2 2026)
- [ ] Mainnet genesis
- [ ] Validator onboarding (50+ validators)
- [ ] Public UI deployment
- [ ] Mobile apps (iOS/Android)
- [ ] National awareness campaign
- [ ] Government integration complete

---

## 🌟 Key Success Metrics

```yaml
Blockchain Performance:
  Block Time: 6 seconds
  Finality: ~30 blocks (~3 minutes)
  TPS Target: 1,000+ transactions/second
  Validator Count: 50+ active validators
  
AI Performance:
  Training Participants: 150+ validators
  Model Accuracy: 85%+ (task-dependent)
  Privacy Budget: ε=3.0 (differential privacy)
  Convergence Time: 100-200 rounds
  
Quantum Performance:
  Job Success Rate: 95%+ (with error mitigation)
  Average Queue Time: <30 minutes (configured quantum backend)
  Fidelity: 98%+ (after error mitigation)
  Cost per Job: <$5 USD (converted to GBP internally)
  
Storage Efficiency:
  Compression Ratio: 70-90% (text files)
  Deduplication Savings: 20-60% (typical)
  Retrieval Time: <2 seconds (hot tier)
  
User Adoption:
  Active Wallets: 50,000+ (10% of population)
  Daily Transactions: 10,000+
  Government Adoption: 100% (all ministries)
  Validator Network: Geographically distributed
```

---

## 🔗 External Resources

- **Official Website**: https://belizechain.org
- **Block Explorer**: https://explorer.belizechain.org (pending)
- **GitHub**: https://github.com/BelizeChain/belizechain
- **Documentation**: https://docs.belizechain.org
- **Polkadot SDK**: https://github.com/paritytech/polkadot-sdk
- **Substrate Docs**: https://docs.substrate.io
- **Quantum backend documentation**: use selected provider docs

---

## ⚖️ Legal & Compliance

- **License**: MIT License
- **Jurisdiction**: Belize
- **Regulatory Body**: Financial Services Commission (FSC)
- **Data Protection**: Aligned with GDPR principles
- **KYC/AML**: Compliant with FATF recommendations
- **Emergency Powers**: Government override capability (multi-sig)

---

## 🤝 Contributing Guidelines

See `CONTRIBUTING.md` for detailed contribution guidelines.

**Quick Checklist**:
- ✅ All blockchain code must compile with zero warnings
- ✅ Python code must pass `pytest` and `mypy` checks
- ✅ TypeScript must pass `eslint` and type checks
- ✅ Always display bBZD as BZD-pegged (never USD!)
- ✅ Test with actual blockchain node (not mocks)
- ✅ Update relevant `.github-instructions.md` if adding features

---

## 📞 Support & Contact

- **Technical Issues**: Open GitHub issue with `[BUG]` prefix
- **Feature Requests**: Open GitHub issue with `[FEATURE]` prefix
- **Security Issues**: Email security@belizechain.org (pending)
- **General Questions**: Discord server (link TBD)

---

**Last Updated**: October 23, 2025  
**Version**: 0.1.0 (Pre-Mainnet)  
**Maintained By**: BelizeChain Core Team

---

When working on BelizeChain, remember:
1. 🇧🇿 **Sovereignty First**: Belize's data stays in Belize
2. 💵 **BZD Peg**: bBZD pegged to Belize Dollar (central bank governed)
3. 🔒 **Privacy Preserved**: Differential privacy, QKD, encryption
4. ⚖️ **Compliance Built-in**: KYC/AML in every financial operation
5. 🌍 **Multilingual**: Support all Belizean languages
6. 🏛️ **Government-Grade**: Security, reliability, transparency
