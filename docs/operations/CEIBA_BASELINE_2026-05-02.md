# Ceiba Baseline - 2026-05-02

Status: Active Phase 2 post-recovery stabilization baseline
Host: `ceiba` at `100.81.45.25` over Tailscale
Compose path: `/opt/belizechain`
Capture time: 2026-05-02 22:22-22:23 UTC

## Chain State

- Chain spec: `/data/chain/testnet-spec.json`
- Chain data: `/data/chain/chains/belizechain_testnet`
- RPC: `100.81.45.25:9944`, proxied at `https://100.81.45.25/rpc`
- P2P: `0.0.0.0:30333`
- Metrics: `100.81.45.25:9615`
- Current node image: `belizechain/ceiba-node:6c447f1-epochfix-spec105-20260502`
- `system_health`: `{"peers":0,"isSyncing":false,"shouldHavePeers":false}`

Progression during this read-only baseline window:

| Time | Best Head | Finalized Head |
|---|---:|---:|
| 2026-05-02T22:22:35Z | `0x763` | `0x760` |
| 2026-05-02T22:23:38Z | `0x76d` | `0x76b` |

Result: block production and finality advanced after the epoch/session recovery.

## Live Images

| Service | Container | Image | Status |
|---|---|---|---|
| Core node | `ceiba-node` | `belizechain/ceiba-node:6c447f1-epochfix-spec105-20260502` | healthy, running 3 hours |
| Pakit | `ceiba-pakit` | `belizechain/pakit-storage:86b01dd` | healthy, running 3 days |
| Nawal | `ceiba-nawal` | `belizechain/nawal:46c5221-metrics-20260502` | healthy, running 3 hours |
| Kinich | `ceiba-kinich` | `belizechain/kinich-quantum:5220c8a-pgfix1` | healthy, running 3 days |
| UI | `ceiba-ui` | `belizechain/blue-hole-portal:c3ecbeb` | live through Nginx |
| Nginx | `ceiba-nginx` | `nginx:1.27-alpine` | public HTTP/HTTPS proxy |
| Prometheus | `ceiba-prometheus` | `prom/prometheus:v2.47.0` | Tailscale only |
| Grafana | `ceiba-grafana` | `grafana/grafana:10.1.5` | Tailscale only |
| Postgres | `ceiba-postgres` | `postgres:15-alpine` | healthy |
| Redis | `ceiba-redis` | `redis:7-alpine` | healthy |
| IPFS | `ceiba-ipfs` | `ipfs/kubo:v0.23.0` | healthy |

## Prometheus Targets

All active targets were up:

| Job | Target |
|---|---|
| `ceiba-node` | `http://ceiba-node:9615/metrics` |
| `kinich-quantum` | `http://kinich:8888/metrics` |
| `nawal-ai` | `http://nawal:8080/metrics` |
| `pakit-storage` | `http://pakit:8001/metrics` |
| `prometheus` | `http://localhost:9090/metrics` |

Nawal `/metrics` is now Prometheus text and reported:

- `nawal_runtime_ready 1.0`
- `nawal_runtime_dependency_failures 0.0`
- `nawal_blockchain_connected 1.0`
- `nawal_fl_rounds_total 1.0`
- `nawal_fl_active_rounds 0.0`

## Activation Endpoints

All probes below used the Ceiba HTTPS proxy at `https://100.81.45.25` with certificate verification disabled for the Tailscale IP.

| Endpoint | Result |
|---|---|
| `GET /api/nawal/health` | healthy; blockchain connected; identity verifier ready |
| `GET /api/nawal/readyz` | ready |
| `GET /api/nawal/api/v1/status` | blockchain enabled and connected to `ws://ceiba-node:9944`; `total_rounds=1` |
| `GET /api/nawal/api/v1/fl/rounds/chain_task_1` | `pending`, `participants=0`, `submissions_received=0` |
| `GET /api/nawal/api/v1/fl/metrics` | `total_rounds=1`, blockchain connected |
| `GET /api/kinich/health` | healthy; node ready; blockchain connected |
| `GET /api/kinich/readyz` | ready; blockchain connected |
| `GET /api/kinich/api/v1/status` | persistence connected to Postgres; `active_jobs=2`, `total_jobs=2` |
| `GET /api/kinich/api/v1/jobs/job_dde529a38c824ed8ba729b3958bfed5a` | completed; qiskit counts `00:8`, `11:8` |
| `GET /api/pakit/health` | healthy; DAG backend ready; IPFS disabled by configuration |
| `GET /api/pakit/api/v1/metadata/a9c24b48541ca24b700f674e1babc2303044a70039a310173a9c8d0290c36011` | metadata present for `pakit-phase2-smoke.txt`, owner `5CPnByRewLBGFt4tCg5HruzJLTQEFj1xRDdFfPeaA2xVPeaa` |

## GEM Contracts

Current Ceiba GEM deployment record from the `gem` repo `.env.testnet`, verified on 2026-05-02 with `contracts.contractInfoOf`:

| Contract | Address |
|---|---|
| DALLA Token | `r1SAvDb2f5iFbafWL87rE1jP6QV3qCV5xtK8b1QXuqKpGknj5` |
| BeliNFT | `r1Wywor1ittVCyZeYaA9hweBwuDiXLC2UdLNQm4oUB1ub8qyN` |
| Simple DAO | `r1VnpeWtfLmtZ2W2UJhYXSLoHhwo7tAY48RZyVirRu5ucLi7i` |
| Faucet | `r1TDXUdxgeLC5BAkFQeZnZNSAX67FwRAaavmG19TzPtc2Szcg` |

Ceiba deployment note: `/opt/belizechain/docker-compose.ceiba.yml` and `/opt/belizechain/.env` did not contain the new `NEXT_PUBLIC_*_CONTRACT` UI env wiring at this baseline. The latest `infra` repo source has that wiring, but the live Ceiba compose file has not yet been synchronized. The public `ui` service is still `belizechain/blue-hole-portal:c3ecbeb`; Maya Wallet is source-wired in the `ui` repo but not separately exposed on Ceiba.

## Follow-Ups

- Complete the post-push CI sweep for the fresh core fix commit `260a6b0` and the UI security rerun.
- Run and document the Phase 2 backup/restore drill for chain data, Postgres, service volumes, and compose/env state.
- Decide whether to keep only Blue Hole public for stabilization or add a separate Maya Wallet deployment path.
- Track Dependabot/security hygiene separately from the activation baseline; current SDK-blocked RustSec entries remain queued for the next Polkadot SDK upgrade.