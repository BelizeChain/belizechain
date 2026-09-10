# Phase 2 Ceiba Services Plan

Status: Live on Ceiba; stabilization and activation in progress *(flagged 2026-09-10 docs audit: "live on Ceiba" here is the ops-host runbook view — the full Ceiba stack including Pakit is not running locally, and the Nawal sovereign-signer gate is still open; see `infra/docs/NEXT3_ROADMAP_LANDED_2026-09-10.md`)*
Goal: Keep the live sibling services reproducible, observable, and ready for
real workflows with deterministic rollback steps.

## Target Services

- ui (Maya Wallet + Blue Hole Portal)
- pakit-storage
- nawal-ai
- kinich-quantum
- gem

## Current Live Baseline

The Ceiba stack is running the Phase 2 service set through
`/opt/belizechain/docker-compose.ceiba.yml`. The dated operational snapshot is
[CEIBA_BASELINE_2026-05-02.md](../operations/CEIBA_BASELINE_2026-05-02.md).

Live containers verified on 2026-05-02:

| Service | Container | Live state |
|---|---|---|
| Core node | `ceiba-node` | recovered after epoch/session fix; producing and finalizing testnet blocks |
| Pakit | `ceiba-pakit` | healthy, DAG backend ready |
| Nawal | `ceiba-nawal` | healthy, blockchain-connected, Prometheus `/metrics` live, `chain_task_1` pending |
| Kinich | `ceiba-kinich` | healthy and blockchain-connected, qiskit activation job completed |
| UI | `ceiba-ui` | Blue Hole Portal live behind Nginx |
| Observability | `ceiba-prometheus`, `ceiba-grafana` | live on Tailscale-bound ports |

GEM remains a contract deployment and address-wiring workstream rather than a
long-running Ceiba service. The primary GEM contracts were deployed and verified
on the active Ceiba testnet on 2026-05-02:

| Contract | Address |
|---|---|
| DALLA Token | `r1SAvDb2f5iFbafWL87rE1jP6QV3qCV5xtK8b1QXuqKpGknj5` |
| BeliNFT | `r1Wywor1ittVCyZeYaA9hweBwuDiXLC2UdLNQm4oUB1ub8qyN` |
| Simple DAO | `r1VnpeWtfLmtZ2W2UJhYXSLoHhwo7tAY48RZyVirRu5ucLi7i` |
| Faucet | `r1TDXUdxgeLC5BAkFQeZnZNSAX67FwRAaavmG19TzPtc2Szcg` |

Current frontend deployment decision:
- Keep the single public Ceiba `ui` slot on Blue Hole Portal for the stabilization window.
- Maya Wallet is source-wired to GEM contract env through the `ui` repo, but it is not separately exposed on Ceiba.
- The latest `infra` source has `NEXT_PUBLIC_*_CONTRACT` defaults for the `ui` service. The live `/opt/belizechain/docker-compose.ceiba.yml` captured on 2026-05-02 does not yet include those env keys, so syncing live compose/env remains a separate deployment step.
- Expose Maya only after the post-push CI sweep and backup/restore drill are complete, using an explicit infra route/image contract and rollback plan.

## Completed Rollout Order

1. Shared dependencies: postgres, redis, reverse proxy, base networks/volumes
2. pakit-storage
3. nawal-ai
4. kinich-quantum
5. ui
6. gem integration endpoints

## Stabilization Order

1. Lock core and infra source to the live Ceiba baseline.
2. Make failing CI gates green across core, infra, Nawal, GEM, and UI.
3. Add missing Nawal Prometheus-format metrics. Completed on 2026-05-02.
4. Run real activation workflows for Pakit, Nawal, and Kinich. Completed on 2026-05-02.
5. Deploy and record GEM contract addresses for the testnet. Completed on 2026-05-02.
6. Finish backup/restore drill validation before exposing additional frontend surfaces.
7. Prepare multi-node testnet expansion after the single-node baseline is stable.

## Preflight

- Confirm host directories under /data for each service.
- Confirm secrets present in environment files and not committed.
- Confirm compose file validates before start.
- Confirm existing belizechain-node remains healthy during rollout.

## Rollout Pattern Per Service

1. Pull/build image with immutable tag.
2. Start service only.
3. Run service health endpoint check.
4. Run dependency check against node RPC at 100.81.45.25:9944.
5. Add to reverse proxy and re-test externally.
6. Capture baseline metrics after 15 minutes.

## Rollback Pattern

1. Stop failing service.
2. Restore prior image tag.
3. Restart and re-run health checks.
4. Record failure root cause before next attempt.

## Exit Criteria

- All live services pass health checks for 24 hours after the baseline commit.
- BelizeChain node RPC remains stable and finalized height advances.
- Prometheus targets stay healthy for node, Kinich, Pakit, Nawal, and Prometheus.
- Nawal has either a Prometheus exporter or native `/metrics` endpoint. Completed with native `/metrics`.
- Pakit, Nawal, and Kinich each complete one real activation workflow on Ceiba.
- GEM testnet contracts are deployed, recorded, and wired into UI/GEM env files.
- Backup snapshot and restore drill are documented and validated.
