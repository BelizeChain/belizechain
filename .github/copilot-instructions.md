# BelizeChain — Core Blockchain Node

## Project Identity
- **Repo**: `BelizeChain/belizechain`
- **Role**: Layer 1 sovereign blockchain node (Substrate-based)
- **Language**: Rust
- **Branch**: `belizechain` (default)

## Architecture
- Core workspace: `node/`, `runtime/`, `pallets/`, `scripts/`, `tests/`, `docs/`
- 18 Belize-specific pallets in `pallets/` plus shared `pallets/common`: belizex, bns, community, compliance, consensus, economy, governance, identity, interoperability, justice, landledger, mesh, moderation, oracle, payroll, quantum, staking, whistleblower
- Runtime topology source of truth: `Cargo.toml` workspace members plus `runtime/src/lib.rs` `construct_runtime!`
- Substrate runtime in `runtime/src/lib.rs`
- Node binary in `node/src/`
- Multi-stage Docker build: `paritytech/ci-linux:production` → `debian:bookworm-slim`
- Ports: 30333 (P2P), 9933 (RPC HTTP), 9944 (RPC WebSocket), 9615 (Prometheus)

## Production Deployment (LIVE)
- **Primary Host**: `ceiba` (Ubuntu 24.04 LTS)
- **Primary Access**: `ssh wicked@100.81.45.25` (Tailscale)
- **LAN Fallback**: `ssh wicked@10.0.0.222` (wired segment)
- **Node Process**: `belizechain-node --dev --base-path /data/chain --port 30333 --rpc-port 9944 --prometheus-port 9615`
- **Binding Note**: RPC is exposed on Ceiba's Tailscale address (not localhost)
- **Runtime Ports**: `30333` (P2P), `9944` (RPC), `9615` (Prometheus)

## GitHub Secrets (on this repo)
`POSTGRES_HOST`, `POSTGRES_PASSWORD`, `POSTGRES_USER`, `REDIS_HOST`, `REDIS_PASSWORD`, `REDIS_PORT`, `VM_HOST`, `VM_SSH_KEY`, `VM_USER`

## Sibling Repos
| Repo | Role | Runtime Target |
|------|------|----------------|
| `BelizeChain/ui` | Maya Wallet + Blue Hole Portal (Next.js) | Ceiba/self-hosted plan |
| `BelizeChain/infra` | Compose + host infrastructure manifests | Ceiba/self-hosted plan |
| `BelizeChain/kinich-quantum` | Hybrid quantum-classical compute | Ceiba/self-hosted plan |
| `BelizeChain/nawal-ai` | Federated learning + privacy ML | Ceiba/self-hosted plan |
| `BelizeChain/gem` | Smart contracts (ink! 4.0) | Ceiba/self-hosted plan |
| `BelizeChain/pakit-storage` | DAG-based decentralized storage | Ceiba/self-hosted plan |

## Current Task Context
- Phase 1 COMPLETE: Ceiba host hardening + BelizeChain node running via Tailscale
- Phase 2 TODO: Containerize and deploy sibling services (ui, pakit, nawal, kinich, gem) on Ceiba
- Phase 3 TODO: Productionize observability + backup/restore + host-level automation

## Dev Commands
```bash
cargo build --release                    # Build node
cargo test                               # Run all tests
docker build -t belizechain-node .       # Build Docker image
ssh wicked@100.81.45.25                  # Access Ceiba node host
curl -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://100.81.45.25:9944
```

## Rules
- Follow `.github/instructions/belizechain_instructions.instructions.md` for Rust/runtime/review work
- Use the `BelizeChain` custom agent for multi-repo, review, or ops-heavy tasks
- Use the `BelizeChain Review` prompt for findings-first PR or audit review workflows
- No floating-point in consensus logic
- All state transitions must be deterministic
- Use `Result<T, Error>` — no `unwrap()` in production
- Treat `Cargo.toml`, `runtime/src/lib.rs`, and active operations docs as authoritative when prose docs disagree
- For docs and sibling rollout state, prefer `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md`, `docs/deployment/PHASE2_CEIBA_SERVICES_PLAN.md`, and `gh` repo metadata over older summaries
- Mark historical or generated architecture reports as snapshots when they no longer match the live branch
- High-impact ops commands should include preflight checks, blast radius, and rollback context before execution
- Keep deployment guidance aligned with current Ceiba self-hosted architecture
