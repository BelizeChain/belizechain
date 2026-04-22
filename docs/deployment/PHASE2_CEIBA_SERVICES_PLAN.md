# Phase 2 Ceiba Services Plan

Status: In progress
Goal: Bring sibling services online on Ceiba with deterministic rollback steps.

## Target Services

- ui (Maya Wallet + Blue Hole Portal)
- pakit-storage
- nawal-ai
- kinich-quantum
- gem

## Deployment Order

1. Shared dependencies: postgres, redis, reverse proxy, base networks/volumes
2. pakit-storage
3. nawal-ai
4. kinich-quantum
5. ui
6. gem integration endpoints

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

- All five services pass health checks for 24 hours.
- Belizechain node RPC remains stable.
- Prometheus targets healthy for node + sibling services.
- Backup snapshot and restore drill documented.
