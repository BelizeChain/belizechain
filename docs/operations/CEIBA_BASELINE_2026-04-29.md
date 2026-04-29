# Ceiba Baseline - 2026-04-29

Status: Active Phase 2 stabilization baseline
Host: `ceiba` at `100.81.45.25` over Tailscale
Compose path: `/opt/belizechain`

## Chain State

- Chain: `BelizeChain Testnet`
- Chain spec: `/data/chain/testnet-spec.json`
- Chain data: `/data/chain/chains/belizechain_testnet`
- Validator mode: enabled with `VALIDATOR=1`
- RPC: `100.81.45.25:9944`, proxied at `https://100.81.45.25/rpc`
- P2P: `0.0.0.0:30333`
- Metrics: `100.81.45.25:9615`
- BABE authority: `5HGrvAhz7oghrAM6m8x6EjscUsnrdwU6FR2p9wwnNtneU2Qo`
- GRANDPA authority: `5Ebravkqw9JLLGnyB6Qae6xXaFzhJYpzdavGnbKZmKzXvJCF`
- Bootnodes: empty for the current single-node testnet baseline

Last verified during Phase 1 baseline check: best head advanced past block
`0x67a`, finalized head advanced past block `0x678`, direct RPC worked, and
proxied JSON-RPC POST worked.

## Live Images

| Service | Container | Image | Status |
|---|---|---|---|
| Core node | `ceiba-node` | `belizechain/ceiba-node:73c36db` | live validator |
| Pakit | `ceiba-pakit` | `belizechain/pakit-storage:86b01dd` | healthy |
| Nawal | `ceiba-nawal` | `belizechain/nawal:4f2646b` | healthy, idle FL state |
| Kinich | `ceiba-kinich` | `belizechain/kinich-quantum:5220c8a-pgfix1` | healthy, no submitted jobs yet |
| UI | `ceiba-ui` | `belizechain/blue-hole-portal:c3ecbeb` | live through Nginx |
| Nginx | `ceiba-nginx` | `nginx:1.27-alpine` | public HTTP/HTTPS proxy |
| Prometheus | `ceiba-prometheus` | `prom/prometheus:v2.47.0` | Tailscale only |
| Grafana | `ceiba-grafana` | `grafana/grafana:10.1.5` | Tailscale only |
| Postgres | `ceiba-postgres` | `postgres:15-alpine` | Tailscale only |
| Redis | `ceiba-redis` | `redis:7-alpine` | Tailscale only |
| IPFS | `ceiba-ipfs` | `ipfs/kubo:v0.23.0` | API and gateway Tailscale only |

## Published Ports

| Port | Bind | Purpose |
|---|---|---|
| 30333 | `0.0.0.0` | Substrate P2P |
| 80 | `0.0.0.0` | HTTP redirect / ACME |
| 443 | `0.0.0.0` | HTTPS UI/API/RPC proxy |
| 9944 | `100.81.45.25` | Direct JSON-RPC / WebSocket RPC |
| 9615 | `100.81.45.25` | Node Prometheus metrics |
| 5432 | `100.81.45.25` | Postgres |
| 6379 | `100.81.45.25` | Redis |
| 5001 | `100.81.45.25` | IPFS API |
| 8082 | `100.81.45.25` | IPFS gateway |
| 9090 | `100.81.45.25` | Prometheus UI/API |
| 3003 | `100.81.45.25` | Grafana |

## Verified Routes

- `GET /` -> `200`
- `GET /health` -> `200`
- `POST /rpc` -> JSON-RPC success
- `GET /rpc` -> may return `405`; use POST for validation
- `GET /api/pakit/health` -> `200`
- `GET /api/nawal/health` -> `200`
- `GET /api/kinich/health` -> `200`
- `GET /api/pakit/metrics` -> `404` by design

## Prometheus Targets

- `ceiba-node`: up
- `kinich-quantum`: up
- `pakit-storage`: up
- `prometheus`: up

Nawal currently exposes JSON at `/api/v1/fl/metrics`, not Prometheus text at `/metrics`; add an exporter or native Prometheus endpoint before scraping it.

## Rollback Anchors From Testnet Reset

- `/opt/belizechain/backups/.env.pre-testnet-reset-20260429-193956`
- `/opt/belizechain/backups/docker-compose.ceiba.yml.pre-testnet-reset-20260429-193956`
- `/opt/belizechain/backups/testnet-spec.pre-testnet-reset-20260429-193956.json`
- `/data/chain/chains/belizechain_testnet.pre-reset-20260429-193956`
- `/opt/belizechain/backups/docker-compose.ceiba.yml.pre-validator-rpc-flag-20260429-154235`

Authority secret material is currently stored on Ceiba under `/opt/belizechain/backups/authority-keys-20260429-193956`. Move this into a deliberate secrets backup policy before treating the testnet as durable.

## Phase 1 Follow-Ups

- Commit and push matching core and infra source changes.
- Re-run GitHub Actions and fix failing gates before new feature work.
- Add a restore drill that validates chain data, Postgres, Redis, Pakit DAG data, and service route recovery.
- Add Nawal Prometheus-format metrics.
- Capture future deployments with immutable image tags and this baseline format.
