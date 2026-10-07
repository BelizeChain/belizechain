# BelizeChain — Core Blockchain Node

## Project Identity
- **Repo**: `BelizeChain/belizechain`
- **Role**: Layer 1 sovereign blockchain node (Substrate-based)
- **Language**: Rust
- **Branch**: `belizechain` (default)

## Architecture
- Core workspace: `node/`, `runtime/`, `pallets/`, `scripts/`, `tests/`, `docs/`
- 19 Belize-specific pallets in `pallets/` plus shared `pallets/common`: belizex, bns, community, compliance, consensus, economy, governance, identity, interoperability, justice, landledger, mesh, moderation, oracle, payroll, quantum, staking, storage-proof, whistleblower
- Runtime topology source of truth: `Cargo.toml` workspace members plus `runtime/src/lib.rs` `construct_runtime!`
- Substrate runtime in `runtime/src/lib.rs`
- Node binary in `node/src/`
- Multi-stage Docker build: `paritytech/ci-linux:production` → `debian:bookworm-slim`
- Ports: 30333 (P2P), 9944 (RPC, unified HTTP+WS), 9615 (Prometheus). Legacy 9933 is not exposed by current SDK.

## Production Deployment (LIVE)
- **Primary Host**: `ceiba2` (Ubuntu 26.04 LTS, `100.119.97.38` over Tailscale) — the stack moved here 2026-10-06. The old `ceiba` box still exists but runs the **dead** Chain C; do not point new work at it.
- **Runtime**: Docker Compose at `/opt/belizechain` (not `--dev`, not raw systemd)
- **Live Chain**: **Chain D ("Jade")** — see `docs/operations/CHAIN_LINEAGE.md`
- **Live Genesis**: `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef` — **identical to Chain C's**, because D was re-genesised from C's spec. The hash alone does **not** identify the chain; use host + head height.
- **Live Epoch**: `BabeEpochDuration = 300` slots (~30 min). This is the **`testnet-fast-epoch` build feature**, used so a session rotation can be observed in minutes; the default build keeps 14,400 slots (~24 h) for mainnet
- **Live Runtime**: `spec_version = 109`. Hot `sudo.sudo(System::set_code(…))` upgrade **2026-10-07 on Chain D**, from 107 — no node restart. The genesis runtime embedded in `testnet-spec.json` is still **107**, so a re-genesis would boot at 107 and upgrade forward. Rollback blob: `/data/upgrade-109/onchain-code-107.wasm` on `ceiba2`
- ⚠️ **Any forward runtime upgrade MUST be built with `--features testnet-fast-epoch`.** A blob built without it silently changes the epoch length from 300 to 14,400 and permanently halts block import — that is exactly how Chain C died on 2026-10-04. `scripts/upgrade-runtime.py` refuses to proceed if the duration would change; always run its dry run first.
- ⚠️ `scripts/upgrade-runtime.py` still defaults `--rpc` to the decommissioned `ws://100.81.45.25:9944`. Pass `--rpc ws://100.119.97.38:9944` explicitly.
- **Primary Access**: `ssh wicked@ceiba2` or `ssh wicked@100.119.97.38` (Tailscale)
- **LAN Fallback**: `ssh wicked@10.0.0.229` (wired)
- **Node Args**: `--chain /data/chain/testnet-spec.json --base-path /data/chain --rpc-port 9944 --prometheus-port 9615`
- **Chain Data**: `/data/chain/chains/belizechain_testnet`
- **Binding Note**: RPC is exposed on the host's Tailscale address (not localhost)
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
- Phase 2 COMPLETE (2026-05-02; Maya Wallet added 2026-09-21): Pakit, Nawal, Kinich, GEM contracts, Blue Hole Portal UI, and Maya Wallet (nginx `/wallet/`, HTTP 200) deployed and verified on Ceiba; chain recovered from epoch-rotation stall and progressing
- **2026-09-29: chain re-genesised** after a 6-day stall caused by a zero-authority BABE epoch. See `docs/operations/EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md` — it is the authoritative record for the current chain, and it supersedes the 2026-09-21 reset in part.
- Phase 3 IN PROGRESS: alerting live (Alertmanager → Telegram armed 2026-10-02); nightly postgres/chain/pakit backups live; backup/restore drill complete 2026-09-17 (non-destructive — live-restore drill awaits a maintenance window); Dependabot backlog cleared except flwr-blocked nawal `cryptography` ×2. Remaining: live-restore drill, host-console stability items (memtest/BIOS/kernel), multi-node testnet expansion
- Authoritative live ops docs: `docs/operations/EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md` (current chain), `CEIBA_OPERATIONS_RUNBOOK.md` (node ops), `CEIBA_BASELINE_2026-05-02.md` (superseded snapshot), and `docs/deployment/PHASE2_CEIBA_SERVICES_PLAN.md`

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
