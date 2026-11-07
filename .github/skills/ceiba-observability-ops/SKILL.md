---
name: ceiba-observability-ops
description: Diagnose the live BelizeChain Ceiba stack through Prometheus, Grafana, Alertmanager, container logs, and JSON-RPC. Use for health checks, alert triage, epoch and authority verification, backup checks, and preflight before high-impact host actions.
---

# Ceiba Observability And Ops

Use this skill for read-only diagnosis of the live Ceiba deployment, then for clearly
scoped remediation. Live versions, heights, endpoints, and container counts are volatile:
read `docs/ECOSYSTEM_STATE.md` and `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md` in the
same session before quoting any of them.

## Read First

1. `docs/ECOSYSTEM_STATE.md` — current chain, runtime, host, and endpoint facts.
2. `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md` — host access, runtime baseline, incident
   triage, backup/restore, and the newest ops log.
3. `infra/docker-compose.ceiba.yml` — service names, images, ports, and health checks.
4. The workflow that produced the current state, if the question is about a rollout.

## Access

- Host alias: `ssh wicked@ceiba`. The LAN fallback and Tailscale address are recorded in
  the runbook; do not use a decommissioned address.
- Compose stack: `/opt/belizechain`; core container: `ceiba-node`; reverse proxy: `nginx`.
- Ports: 30333 P2P, 9944 RPC (HTTP + WS), 9615 Prometheus metrics. Legacy 9933 is not
  exposed.
- Read-only commands first:

```bash
ssh wicked@ceiba 'cd /opt/belizechain && docker compose ps'
ssh wicked@ceiba 'docker logs ceiba-node --tail 200'
docker -H ssh://wicked@ceiba ps
```

## Triage Ladder

1. **Container and port state** — `docker compose ps`; is `ceiba-node` up, restarting, or
   exited? Are 9944 and 9615 bound?
2. **RPC liveness** — POST JSON to `/rpc`; a plain GET may return 405 while RPC is healthy.

```bash
curl -sS -H 'Content-Type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
  http://<host>:9944
```

3. **Chain progress** — `system_syncState`, `chain_getHeader`, and
   `state_getRuntimeVersion`; compare head movement across two samples.
4. **Epoch and authority set** — `BabeApi_current_epoch` via
   `scripts/verify_epoch_config.py <rpc> 300`. An empty authority set is unrecoverable and
   is the failure that stalled the chain before; duration must stay 300 on testnet.
5. **Logs** — search for `panic`, `Expected epoch change`, `Idle`, and restart loops.
   Healthy block flow shows `Prepared` / `Pre-sealed` / `Imported`.
6. **Host resources** — `df -h`, `df -i`, `free -h`, `docker stats --no-stream`. Disk or
   inode exhaustion presents as a node that is up but not importing.
7. **Functional smoke test** — `sudo systemctl start ceiba-live-actions.service`, then read
   `/data/log/live-actions/live-actions-YYYYMMDD.log`; expect `FAIL = 0`. This is what
   catches a re-genesis silently dropping the Nawal operator registration.

## Observability Stack

Services in the Ceiba compose file include `prometheus`, `alertmanager`, `grafana`,
`postgres-exporter`, and `nginx`. Check them from the host network:

```bash
ssh wicked@ceiba 'cd /opt/belizechain && docker compose ps prometheus alertmanager grafana nginx'
ssh wicked@ceiba 'curl -sS http://localhost:9090/api/v1/targets'
ssh wicked@ceiba 'curl -sS http://localhost:9090/api/v1/alerts'
ssh wicked@ceiba 'curl -sS http://localhost:9093/api/v2/alerts'
```

- A target that is `down` is a monitoring defect, not a chain defect; report it separately.
- Alertmanager routes to Telegram. Never print the bot token, chat ID, webhook URL, or any
  value from an env file. Say "configured" or "not configured" instead.
- Do not silence an alert without recording the reason, the duration, and the follow-up.
- If Grafana is reachable but Prometheus is not, fix the data source before interpreting
  dashboards.

## High-Impact Actions

These require explicit approval, a stated blast radius, and a rollback path:

- `docker compose down/up`, container recreation, or host reboot.
- Moving or deleting `/data/chain`, `/data/chain/chains/belizechain_testnet`, or backups.
- Any runtime upgrade.
- `sudo`, `systemctl`, firewall, or Tailscale changes.
- Re-genesis or chain-spec edits.

Before any of them: take or verify a backup, record the current head and spec version,
and write the rollback command. For a runtime upgrade use
`scripts/upgrade-runtime.py` with an explicit `--rpc` (no default), run it without
`--execute` first, build with `--features testnet-fast-epoch`, and confirm the epoch
duration did not change. A spec-down rollback needs `--without-checks` and does not roll
back storage migrations.

## Incident Classification

State one of:

- **Chain stalled** — container up, head not advancing, or epoch/authority error in logs.
- **Node down** — container exited or port unbound.
- **Service degraded** — chain healthy but a sibling (nawal, pakit, kinich, indexer,
  relayer) fails its health check.
- **Observability blind** — metrics, alerts, or dashboards unavailable while the chain is
  healthy.
- **Host pressure** — disk, inode, memory, or CPU exhaustion.

Do not restart the node to "see if it helps" before capturing logs and the current head;
that destroys the evidence needed for the post-mortem.

## Evidence Format

For each check, report:

```text
time (UTC) | command | key output | interpretation
```

Mark anything you did not run as **not verified**. Do not restate a live value from a
document if you can query it, and do not present a historical runbook snapshot as current.
