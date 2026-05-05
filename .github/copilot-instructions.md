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
- Ports: 30333 (P2P), 9944 (RPC, unified HTTP+WS), 9615 (Prometheus). Legacy 9933 is not exposed by current SDK.

## Production Deployment (LIVE)
- **Primary Host**: `ceiba` (Ubuntu 24.04 LTS)
- **Runtime**: Docker Compose at `/opt/belizechain` (not `--dev`, not raw systemd)
- **Live Image**: `belizechain/ceiba-node:6c447f1-epochfix-spec105-20260502`
- **Primary Access**: `ssh wicked@ceiba` or `ssh wicked@100.81.45.25` (Tailscale; may require browser auth)
- **LAN Fallback**: `ssh wicked@10.0.0.222` (wired; run `ssh-keyscan 10.0.0.222 >> ~/.ssh/known_hosts` first if unseen)
- **Node Args**: `--chain /data/chain/testnet-spec.json --base-path /data/chain --rpc-port 9944 --prometheus-port 9615`
- **Chain Data**: `/data/chain/chains/belizechain_testnet`
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
- Phase 2 COMPLETE (2026-05-02): Pakit, Nawal, Kinich, GEM contracts, and Blue Hole Portal UI deployed and verified on Ceiba; chain recovered from epoch-rotation stall and progressing
- Phase 3 IN PROGRESS: Productionize observability, backup/restore drills, security hygiene queue (Dependabot backlog), and host-level automation
- Authoritative live ops docs: `docs/operations/CEIBA_BASELINE_2026-05-02.md`, `CEIBA_OPERATIONS_RUNBOOK.md`, `CEIBA_BACKUP_RESTORE_DRILL_2026-05-02.md`, and `docs/deployment/PHASE2_CEIBA_SERVICES_PLAN.md`

## Dev Commands
```bash
cargo build --release                    # Build node
cargo test                               # Run all tests
docker build -t belizechain-node .       # Build Docker image
ssh wicked@ceiba                         # Access Ceiba node host (Tailscale)
docker -H ssh://wicked@ceiba ps          # Inspect live containers
docker -H ssh://wicked@ceiba logs ceiba-node --tail 200
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
