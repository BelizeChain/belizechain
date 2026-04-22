# Ceiba Operations Runbook

Status: Active testnet operations
Scope: BelizeChain self-hosted runtime on Ceiba

## Host Access

- Primary: ssh wicked@100.81.45.25
- Fallback: ssh wicked@10.0.0.222

## Runtime Baseline

- Process: belizechain-node --dev --base-path /data/chain --port 30333 --rpc-port 9944 --prometheus-port 9615 --prometheus-external --rpc-cors all --rpc-external --rpc-methods Safe --name Ceiba-Node-1
- P2P: 30333
- RPC: 9944 (bound to 100.81.45.25)
- Prometheus: 9615

## Health Checks

Run these on Ceiba:

```bash
pgrep -a -f belizechain-node
ss -ltnp | grep -E "30333|9944|9615"
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
  http://100.81.45.25:9944
```

Expected JSON fields:
- result.isSyncing
- result.peers
- result.shouldHavePeers

## Restart Procedure

If managed by systemd:

```bash
sudo systemctl restart belizechain
sudo systemctl status belizechain --no-pager
journalctl -u belizechain -n 100 --no-pager
```

If managed as a manual process:

```bash
pkill -f "belizechain-node --dev --base-path /data/chain" || true
nohup belizechain-node --dev --base-path /data/chain --port 30333 --rpc-port 9944 --prometheus-port 9615 --prometheus-external --rpc-cors all --rpc-external --rpc-methods Safe --name Ceiba-Node-1 >/data/logs/belizechain-node.log 2>&1 &
```

## Backup and Restore

### Backup

```bash
tar -czf /data/backups/chain-$(date +%F-%H%M).tgz /data/chain
sha256sum /data/backups/chain-*.tgz | tail -n 1
```

### Restore

```bash
pkill -f belizechain-node || true
mv /data/chain /data/chain.pre-restore.$(date +%s)
mkdir -p /data/chain
tar -xzf /data/backups/<backup-file>.tgz -C /
```

## Incident Triage

1. Confirm process exists and ports are bound.
2. Confirm RPC returns JSON and isSyncing state.
3. Check journal or nohup logs for panic/restart loops.
4. Validate free disk and inode space.
5. Escalate if repeated crash loops persist after restart.

## Notes

- This runbook is the source of truth for current node operations.
- Kubernetes and Azure deployment docs are legacy references unless explicitly reactivated.
