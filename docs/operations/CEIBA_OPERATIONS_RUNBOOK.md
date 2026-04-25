# Ceiba Operations Runbook

Status: Active testnet operations
Scope: BelizeChain self-hosted runtime on Ceiba

## Host Access

- Primary: ssh wicked@100.81.45.25
- Fallback: ssh wicked@10.0.0.222

## Runtime Baseline

- Runtime: Docker container `ceiba-node`
- Compose working directory: /opt/belizechain
- Command: belizechain-node --dev --base-path /data/chain --port 30333 --rpc-port 9944 --prometheus-port 9615 --prometheus-external --rpc-cors all --rpc-external --rpc-methods Safe --name Ceiba-Node-1
- P2P: 30333
- RPC: 9944 (bound to 100.81.45.25)
- Prometheus: 9615

## Health Checks

Run these on Ceiba:

```bash
cd /opt/belizechain
docker compose ps
docker ps --filter name=ceiba-node
docker logs --tail 100 ceiba-node
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

Preferred path on Ceiba:

```bash
cd /opt/belizechain
docker compose up -d ceiba-node
docker compose ps ceiba-node
docker logs --tail 100 ceiba-node
```

If the systemd wrapper is present and healthy:

```bash
sudo systemctl restart belizechain-compose
sudo systemctl status belizechain-compose --no-pager
cd /opt/belizechain && docker compose ps
docker logs --tail 100 ceiba-node
```

## Backup and Restore

### Backup

```bash
tar -czf /data/backups/chain-$(date +%F-%H%M).tgz /data/chain
sha256sum /data/backups/chain-*.tgz | tail -n 1
```

### Restore

```bash
cd /opt/belizechain
docker compose stop ceiba-node || true
mv /data/chain /data/chain.pre-restore.$(date +%s)
mkdir -p /data/chain
tar -xzf /data/backups/<backup-file>.tgz -C /
docker compose up -d ceiba-node
```

## Clean Reset For Ceiba Dev Chain

Use this when the goal is to discard the current Ceiba development chain and start from a fresh local identity.

Current assumptions:
- Ceiba is running a disposable `--dev` chain, not a production or long-lived testnet.
- The active chain data lives under `/data/chain/chains/belizechain_dev`.
- The libp2p node identity is stored at `/data/chain/chains/belizechain_dev/network/secret_ed25519`.

### Reset Scope Options

- Reset chain data only: keep the current libp2p identity, discard blocks and state.
- Reset chain data and libp2p identity: also delete the stored node key so Substrate generates a new peer identity on next boot.
- Session keys: only relevant if Ceiba is later promoted to validator duties outside the disposable dev-chain flow.

### Pre-Reset Snapshot

```bash
mkdir -p /data/backups
tar -C /data/chain/chains -czf /data/backups/ceiba-dev-pre-reset-$(date +%F-%H%M).tgz belizechain_dev belizechain_local
sha256sum /data/backups/ceiba-dev-pre-reset-*.tgz | tail -n 1
```

### Stop The Runtime

```bash
cd /opt/belizechain
docker compose stop ceiba-node
docker compose ps ceiba-node
```

### Option A: Keep Current Peer Identity

```bash
rm -rf /data/chain/chains/belizechain_dev
rm -rf /data/chain/chains/belizechain_local
mkdir -p /data/chain/chains
docker compose up -d ceiba-node
docker logs --tail 100 ceiba-node
```

### Option B: Rotate Peer Identity Too

```bash
mkdir -p /data/backups/node-keys
cp /data/chain/chains/belizechain_dev/network/secret_ed25519 /data/backups/node-keys/ceiba-dev-secret_ed25519.$(date +%s) 2>/dev/null || true
cp /data/chain/chains/belizechain_local/network/secret_ed25519 /data/backups/node-keys/ceiba-local-secret_ed25519.$(date +%s) 2>/dev/null || true

rm -rf /data/chain/chains/belizechain_dev
rm -rf /data/chain/chains/belizechain_local
mkdir -p /data/chain/chains

docker compose up -d ceiba-node
docker logs --tail 100 ceiba-node
```

If the node key is absent on startup, Substrate will generate a fresh local peer identity and log a new `Local node identity` value.

### Post-Reset Validation

```bash
docker logs --tail 100 ceiba-node | grep -E "Local node identity|Running JSON-RPC server|Prometheus exporter started" || true
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_chain","params":[]}' \
  http://100.81.45.25:9944
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
  http://100.81.45.25:9944
```

Expected outcome after reset:
- `system_chain` returns `BelizeChain Development`.
- `system_health.isSyncing` is `false`.
- Logs show a new genesis initialization.

### Validator/Session-Key Follow-Up

Do not rotate session keys for the disposable dev chain unless Ceiba is being repurposed as a real validator target.

If Ceiba is later promoted to validator duties:
1. Generate fresh session keys with `author_rotateKeys`.
2. Submit `session.setKeys` on-chain from the controller/stash flow.
3. Back up the new session keys separately from the libp2p node key.

## Incident Triage

1. Confirm the `ceiba-node` container exists and ports are bound.
2. Confirm RPC returns JSON and isSyncing state.
3. Check `docker compose ps` and `docker logs` for restart loops or panic output.
4. Validate free disk and inode space.
5. Escalate if repeated crash loops persist after restart.

## Notes

- This runbook is the source of truth for current node operations.
- Kubernetes and Azure deployment docs are legacy references unless explicitly reactivated.
