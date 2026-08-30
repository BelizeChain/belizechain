# Ceiba Operations Runbook

Status: Active testnet operations
Scope: BelizeChain self-hosted runtime on Ceiba

## Host Access

- Primary: ssh wicked@100.81.45.25
- Fallback: ssh wicked@10.0.0.222

## Runtime Baseline

Dated baseline snapshot:
[CEIBA_BASELINE_2026-04-29.md](CEIBA_BASELINE_2026-04-29.md).

- Runtime: Docker Compose stack in `/opt/belizechain`
- Core container: `ceiba-node`
- Active chain spec: `/data/chain/testnet-spec.json`
- Active chain data: `/data/chain/chains/belizechain_testnet`
- Command shape: `belizechain-node --chain /data/chain/testnet-spec.json --base-path /data/chain --port 30333 --rpc-port 9944 --prometheus-port 9615 --prometheus-external --rpc-cors all --unsafe-rpc-external --rpc-methods Safe --name Ceiba-Node-1 --validator`
- P2P: 30333 (public)
- RPC: 9944 (bound to Ceiba's Tailscale address)
- Prometheus: 9615 (bound to Ceiba's Tailscale address)

Validator mode requires `--unsafe-rpc-external` instead of `--rpc-external`.
Keep `--rpc-methods Safe` and the host binding on Ceiba's Tailscale address.
When `ceiba-node` is recreated, restart `ceiba-nginx` afterward so Nginx
refreshes the node's Docker-network address. Validate `/rpc` with a JSON-RPC
POST request; a plain GET may return `405` even when RPC is healthy.

## Disposable Public Testnet Reset Plan

Use this only after choosing to abandon the current Ceiba public-testnet state.
It changes the testnet genesis authority keys and clears the local
`belizechain_testnet` database so the single Ceiba node can author from a fresh
genesis.

Do not insert the repo placeholder seeds (`validator1`, `validator2`,
`validator3`) into the current live spec. Their derived public keys do not match
the active Ceiba authority keys.

### Scope And Blast Radius

- Target host: Ceiba (`ssh wicked@100.81.45.25`).
- Target stack: `/opt/belizechain` Docker Compose.
- Target chain data: `/data/chain/chains/belizechain_testnet`.
- Target spec: `/data/chain/testnet-spec.json`.
- Blast radius: stops `ceiba-node`, abandons the current testnet genesis/state,
  changes the genesis hash, temporarily interrupts direct RPC and proxied
  `/rpc`, and requires downstream clients to reconnect to the new chain.
- Not in scope: Postgres, Redis, Pakit, Nawal, Kinich, UI data, IPFS repo data,
  or public HTTP/HTTPS proxy routing except for a short Nginx restart.

### Required Gates Before Reset

```bash
cd /opt/belizechain
grep -E '^(CHAIN|VALIDATOR|NODE_NAME|TAILSCALE_IP)=' .env
docker compose -f docker-compose.ceiba.yml --env-file .env ps
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' \
  http://100.81.45.25:9944
find /data/chain/chains/belizechain_testnet/keystore -maxdepth 1 -type f | wc -l
jq -c '.genesis.runtimeGenesis.patch | {babe,grandpa}' /data/chain/testnet-spec.json
```

Proceed only when the reset decision is explicit, the backup commands below
complete, and the new BABE/GRANDPA public keys are written into the replacement
`session.keys` before the node restarts. Do not configure both
`session.keys` and direct `babe.authorities` in the same replacement spec;
BABE initializes authorities from the session genesis config when session keys
are present.

### Backup And Rollback Anchor

```bash
cd /opt/belizechain
stamp=$(date +%Y%m%d-%H%M%S)
mkdir -p backups

cp .env "backups/.env.pre-testnet-reset-$stamp"
cp docker-compose.ceiba.yml "backups/docker-compose.ceiba.yml.pre-testnet-reset-$stamp"
cp /data/chain/testnet-spec.json "backups/testnet-spec.pre-testnet-reset-$stamp.json"
sha256sum "backups/testnet-spec.pre-testnet-reset-$stamp.json"
du -sh /data/chain/chains/belizechain_testnet
```

Keep the `stamp` value for rollback and validation notes. The pre-reset chain
database is preserved by moving it to
`/data/chain/chains/belizechain_testnet.pre-reset-$stamp` in the clear step
below; avoid a full compressed chain archive during the maintenance window.

### Generate Replacement Authority Material

These commands store the new secret phrases on Ceiba under a `0700` directory.
Do not paste the JSON files into chat, tickets, logs, or documentation.

```bash
cd /opt/belizechain
umask 077
mkdir -p "backups/authority-keys-$stamp"

docker exec ceiba-node belizechain-node key generate \
  --scheme Sr25519 --output-type json \
  > "backups/authority-keys-$stamp/babe.sr25519.json"
docker exec ceiba-node belizechain-node key generate \
  --scheme Ed25519 --output-type json \
  > "backups/authority-keys-$stamp/grandpa.ed25519.json"

BABE_SS58=$(jq -r .ss58Address "backups/authority-keys-$stamp/babe.sr25519.json")
GRANDPA_SS58=$(jq -r .ss58Address "backups/authority-keys-$stamp/grandpa.ed25519.json")
printf 'new_babe=%s\nnew_grandpa=%s\n' "$BABE_SS58" "$GRANDPA_SS58"
```

### Build Replacement Testnet Spec

```bash
cd /opt/belizechain
NEW_IMAGE=$(grep -E '^CEIBA_NODE_IMAGE=' .env | cut -d= -f2-)
docker run --rm "$NEW_IMAGE" build-spec \
  --disable-default-bootnode \
  --chain testnet-template > "backups/testnet-spec.generated-$stamp.json"

name=$(jq -r .name /data/chain/testnet-spec.json)
id=$(jq -r .id /data/chain/testnet-spec.json)
jq --arg account "$BABE_SS58" \
   --arg babe "$BABE_SS58" \
   --arg grandpa "$GRANDPA_SS58" \
   --arg name "$name" \
   --arg id "$id" '
  .name = $name |
  .id = $id |
  .bootNodes = [] |
  .genesis.runtimeGenesis.patch.session.keys = [[
    $account,
    $account,
    {babe: $babe, grandpa: $grandpa}
  ]] |
  del(.genesis.runtimeGenesis.patch.babe.authorities) |
  .genesis.runtimeGenesis.patch.grandpa = {}
' "backups/testnet-spec.generated-$stamp.json" > "/data/chain/testnet-spec.$stamp.json"

jq empty "/data/chain/testnet-spec.$stamp.json"
cp "/data/chain/testnet-spec.$stamp.json" /data/chain/testnet-spec.json
```

### Clear Testnet State And Insert Keys

```bash
cd /opt/belizechain
docker compose -f docker-compose.ceiba.yml --env-file .env stop ceiba-node
mv /data/chain/chains/belizechain_testnet "/data/chain/chains/belizechain_testnet.pre-reset-$stamp"
mkdir -p /data/chain/chains/belizechain_testnet/network
cp "/data/chain/chains/belizechain_testnet.pre-reset-$stamp/network/secret_ed25519" \
  /data/chain/chains/belizechain_testnet/network/secret_ed25519
chmod 600 /data/chain/chains/belizechain_testnet/network/secret_ed25519

mkdir -p "/data/chain/authority-keys-$stamp"
jq -r .secretPhrase "backups/authority-keys-$stamp/babe.sr25519.json" \
  > "/data/chain/authority-keys-$stamp/babe.suri"
jq -r .secretPhrase "backups/authority-keys-$stamp/grandpa.ed25519.json" \
  > "/data/chain/authority-keys-$stamp/grandpa.suri"
chmod 600 "/data/chain/authority-keys-$stamp"/*.suri

docker compose -f docker-compose.ceiba.yml --env-file .env run --rm --no-deps ceiba-node \
  key insert --base-path /data/chain --chain /data/chain/testnet-spec.json \
  --scheme Sr25519 --suri "/data/chain/authority-keys-$stamp/babe.suri" --key-type babe
docker compose -f docker-compose.ceiba.yml --env-file .env run --rm --no-deps ceiba-node \
  key insert --base-path /data/chain --chain /data/chain/testnet-spec.json \
  --scheme Ed25519 --suri "/data/chain/authority-keys-$stamp/grandpa.suri" --key-type gran

find /data/chain/chains/belizechain_testnet/keystore -maxdepth 1 -type f | wc -l

grep -q '^VALIDATOR=' .env && sed -i 's/^VALIDATOR=.*/VALIDATOR=1/' .env || printf '\nVALIDATOR=1\n' >> .env
```

### Restart And Validate

```bash
cd /opt/belizechain
docker compose -f docker-compose.ceiba.yml --env-file .env up -d ceiba-node
docker compose -f docker-compose.ceiba.yml --env-file .env restart nginx
docker compose -f docker-compose.ceiba.yml --env-file .env ps ceiba-node nginx

curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
  http://100.81.45.25:9944
curl -sS -H "Content-Type: application/json" \
  -d '{"id":2,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' \
  http://100.81.45.25:9944
curl -k -sS -H "Content-Type: application/json" \
  -d '{"id":3,"jsonrpc":"2.0","method":"system_health","params":[]}' \
  https://100.81.45.25/rpc
docker logs --tail 120 ceiba-node | grep -E "Idle|best: #|Imported|Starting consensus|Local node identity" || true
```

Successful recovery means `ceiba-node` is healthy, `/rpc` works through Nginx,
the header advances beyond `0x0`, and logs show authored/imported blocks after
the reset.

### Rollback

Use this only if the reset fails validation and the pre-reset state must be
restored.

```bash
cd /opt/belizechain
docker compose -f docker-compose.ceiba.yml --env-file .env stop ceiba-node
mv /data/chain/chains/belizechain_testnet "/data/chain/chains/belizechain_testnet.failed-reset-$stamp" 2>/dev/null || true
mv "/data/chain/chains/belizechain_testnet.pre-reset-$stamp" /data/chain/chains/belizechain_testnet
cp "backups/testnet-spec.pre-testnet-reset-$stamp.json" /data/chain/testnet-spec.json
cp "backups/.env.pre-testnet-reset-$stamp" .env
docker compose -f docker-compose.ceiba.yml --env-file .env up -d ceiba-node
docker compose -f docker-compose.ceiba.yml --env-file .env restart nginx
```

## Backup and Restore

The current Phase 2 backup/restore drill plan is
`docs/operations/CEIBA_BACKUP_RESTORE_DRILL_2026-05-02.md`. Use that drill
for chain data, Postgres, service volumes, and compose/env state. The short
commands below are only the minimal chain-data fallback.

### Backup

```bash
cd /opt/belizechain
mkdir -p backups
tar -C /data -czf backups/chain-$(date +%F-%H%M).tgz chain
sha256sum backups/chain-*.tgz | tail -n 1
```

### Restore

```bash
cd /opt/belizechain
docker compose stop ceiba-node || true
mv /data/chain /data/chain.pre-restore.$(date +%s)
mkdir -p /data/chain
tar -xzf backups/<backup-file>.tgz -C /data
docker compose up -d ceiba-node
docker compose restart nginx
```

## Incident Triage

1. Confirm the `ceiba-node` container exists and ports are bound.
2. Confirm RPC returns JSON and isSyncing state.
3. Check `docker compose ps` and `docker logs` for restart loops or panic output.
4. Validate free disk and inode space.
5. Escalate if repeated crash loops persist after restart.

## Notes

- This runbook is the source of truth for current node operations.
- Kubernetes and Azure deployment docs are legacy references unless explicitly reactivated.

## Ops Log — 2026-08-30: Two-Validator Reset And Edge Validator

### What changed
- Testnet genesis reset from dev-Alice to a two-validator authority set:
  - Validator 1 (Ceiba): BABE sr25519 `5Cg3Ez7Upm8caDfjonnMKPZ14B3H5daWM75DkYj7yEt4XSKt`, GRANDPA ed25519 `5EFaZhoAe2v2PJjYRf99VGZz8WQ3pfzyaXR8BugYaqJC9Sr1` (derived from the SUDO wallet seed).
  - Validator 2 (Oracle edge `belizechain-edge-proxy`): BABE `5HKq5zdfbmtaUo54BXnTTHEuUP1TwgL6c4Lofk1byG5FQTiB`, GRANDPA `5GAWdnhwJKSrEBkV3brd3rRSH7PPsiEKhS9kaYKcYvqrD9yc` (fresh `key generate` output; suris live on the edge host under `~/validator-keys`, 0600).
- New spec generated with `build-spec --chain testnet-template` on image
  `6c447f1-epochfix-spec105-20260502`; both validators written into
  `session.keys`; `bootNodes = []`. Pre-reset copies: `backups/testnet-spec.pre-reset-20260830-reset.json`,
  old chain data at `/data/chain/chains/belizechain_testnet.pre-reset-20260830`.

### Pitfalls hit (and rules of thumb)
- `--force-authoring` is REQUIRED for Ceiba's single-node-before-peers state:
  without it an AUTHORITY node with 0 peers sits at `best: #0` forever with no
  error. Added `${VALIDATOR:+--force-authoring}` next to `--validator` in
  `docker-compose.ceiba.yml`; backup at
  `backups/docker-compose.ceiba.yml.pre-force-authoring-20260830`.
- After any manual `mv`/`cp` of chain dirs, chown the active chain dir
  (`belizechain_testnet`) to uid 1000 (`belizechain` in-container) or the node
  crash-loops with RocksDB `PermissionDenied`.
- Keystore files must exist in the ACTIVE chain dir before node start; keystore
  is only read at startup, restart after inserting keys.
- Port 443 conflict: a stale Tailscale funnel listener (`tailscaled`) held
  0.0.0.0:443. `tailscale funnel off` errors are misleading when the serve
  config blob is empty; `sudo systemctl restart tailscaled` clears the stale
  listener.
- `/opt/belizechain/nginx/nginx.conf` had drifted to hardcoded
  `127.0.0.1`/`172.20.0.x` upstreams causing proxied 502s. Restored the repo
  version (Docker DNS names); stale copy kept at
  `backups/nginx.conf.stale-ips-20260830`. Rule: deploy nginx.conf from the
  infra repo, never hand-edit on the host.

### Nawal unhealthy fix (2026-08-30)
- Symptom: every request crashed with
  `SystemError: anyio/_backends/_asyncio.py:763: unknown opcode 235` (corrupt
  bytecode), then `ModuleNotFoundError: sniffio` after pip reinstalls.
- Fix: built image `belizechain/nawal:46c5221-fixed-anyio-20260830v2` from the
  snapshotted container (`snap-before-fix`) with `sniffio==1.3.1` installed
  into `/app/.local/lib/python3.12/site-packages`. `.env` now pins
  `NAWAL_IMAGE` to that tag. NOTE: the committed image config lost the original
  `ENTRYPOINT/CMD`; until a clean rebuild from `nawal-ai` source, the container
  runs with `--entrypoint python api_server.py` (docker run). Next
  `nawal-ai` image build should replace this stopgap and re-enable normal
  compose management.

### Validator 2 on Oracle edge (in progress)
- Host `belizechain-edge-proxy` (129.213.95.153 / 100.80.208.21, aarch64):
  8G swapfile added; rustup + clang-18/llvm-18-dev installed; repo cloned to
  `~/belizechain` at pinned commit `6c447f1dfc1307d18a5be4a246776388c430a63f`
  (same as running `6c447f1` image); `.cargo/config.toml` patched to
  llvm-18; release build running (`~/build3.log`, `LIBCLANG_PATH=/usr/lib/llvm-18/lib`).
- Compose staged at `~/validator/docker-compose.yml` (image tag
  `belizechain/ceiba-node:6c447f1-arm64` once build completes; 30333 public,
  9944/9615 bound to `100.80.208.21`; bootnode points at Ceiba peer ID
  `12D3KooWMWYcGsBhe1NGX3mf4fMkoipi9U59KxEbJp3CmDqe7cP1` — preserved across
  the reset via `--node-key`).
- Key files were relayed Ceiba → workstation → edge; direct Ceiba→edge
  Tailscale SSH hangs on check-mode auth; use the workstation relay.
- OCI security list must allow TCP 30333 from the internet (or at minimum from
  Ceiba) before the edge validator can peer.
