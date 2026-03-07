# BelizeChain — Core Blockchain Node

## Project Identity
- **Repo**: `BelizeChain/belizechain`
- **Role**: Layer 1 sovereign blockchain node (Substrate-based)
- **Language**: Rust
- **Branch**: `belizechain` (default)

## Architecture
- 16 custom pallets in `pallets/` (belizex, bns, community, compliance, consensus, economy, governance, identity, interoperability, justice, landledger, mesh, moderation, oracle, payroll, quantum, randomness, staking, whistleblower)
- Substrate runtime in `runtime/src/lib.rs`
- Node binary in `node/src/`
- Multi-stage Docker build: `paritytech/ci-linux:production` → `debian:bookworm-slim`
- Ports: 30333 (P2P), 9933 (RPC HTTP), 9944 (RPC WebSocket), 9615 (Prometheus)

## Azure Deployment (LIVE)
- **ACR**: `belizechainacr.azurecr.io` (Basic SKU, admin-enabled)
- **AKS**: `belizechain-aks` (Free tier, 1x Standard_D2s_v3, K8s v1.33.6)
- **Resource Group**: `BelizeChain` in `centralus`
- **Subscription**: `77e6d0a2-78d2-4568-9f5a-34bd62357c40`
- **Tenant**: `belizechain.org` (`8e20a9b6-5590-46ec-a8aa-1c71aa0a7e13`)
- **Service Principal**: `belizechain-github-actions` (Contributor on RG)
- **CI/CD**: `.github/workflows/deploy.yml` — test → coverage → benchmark → build (Docker+ACR) → deploy (AKS kubectl)

## GitHub Secrets (on this repo)
`ACR_USERNAME`, `ACR_PASSWORD`, `AZURE_CREDENTIALS`, `AZURE_RESOURCE_GROUP`, `AKS_CLUSTER_NAME`, `POSTGRES_HOST`, `POSTGRES_PASSWORD`, `POSTGRES_USER`, `REDIS_HOST`, `REDIS_PASSWORD`, `REDIS_PORT`, `VM_HOST`, `VM_SSH_KEY`, `VM_USER`

## Sibling Repos (same AKS cluster)
| Repo | Role | Image |
|------|------|-------|
| `BelizeChain/ui` | Maya Wallet + Blue Hole Portal (Next.js) | `belizechainacr.azurecr.io/ui` |
| `BelizeChain/infra` | GitOps: Helm, ArgoCD, Grafana, K8s manifests | — |
| `BelizeChain/kinich-quantum` | Hybrid quantum-classical compute | `belizechainacr.azurecr.io/kinich` |
| `BelizeChain/nawal-ai` | Federated learning + privacy ML | `belizechainacr.azurecr.io/nawal` |
| `BelizeChain/gem` | Smart contracts (ink! 4.0) | `belizechainacr.azurecr.io/gem` |
| `BelizeChain/pakit-storage` | DAG-based decentralized storage | `belizechainacr.azurecr.io/pakit` |

## Current Task Context
- Phase 1 COMPLETE: ACR + AKS + SP + secrets + deploy workflow migrated to AKS
- Phase 2 TODO: Containerize & deploy sibling services (ui, pakit, nawal, kinich, gem)
- Phase 3 TODO: Deploy infra repo Helm charts, ArgoCD, Grafana to AKS

## Dev Commands
```bash
cargo build --release                    # Build node
cargo test                               # Run all tests
docker build -t belizechain-node .       # Build Docker image
az acr login --name belizechainacr       # Login to ACR
kubectl get pods -n belizechain          # Check AKS pods
```

## Rules
- Follow `belizechain_instructions.instructions.md` for all Rust/consensus code
- No floating-point in consensus logic
- All state transitions must be deterministic
- Use `Result<T, Error>` — no `unwrap()` in production
- AKS cost ceiling: Free tier only, 1 node Standard_D2s_v3 (~$75/mo total)
