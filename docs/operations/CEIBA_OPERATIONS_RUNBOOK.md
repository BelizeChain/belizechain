# Ceiba Operations Runbook

Status: Active testnet operations
Scope: BelizeChain self-hosted runtime

## Host Access

The live stack moved to the replacement host on 2026-10-06 (provisioned as `ceiba2`,
**renamed `ceiba` on 2026-10-07** once the original box was retired). The original
Ceiba — which ran the dead Chain C — has been **decommissioned and wiped**; it is no
longer on the network and is not a fallback.

- **Primary: `ssh wicked@ceiba`** (`100.119.97.38` over Tailscale)
- LAN: `ssh wicked@10.0.0.19` (ethernet) — WiFi is disabled; `10.0.0.229` is stale

⚠️ The original host's addresses (`100.81.45.25` / `10.0.0.222` / WAN `174.161.1.222`)
are **dead** — nothing answers there.

## Runtime Baseline

Current chain identity: **Chain D ("Jade") on `ceiba`** — see
[CHAIN_LINEAGE.md](CHAIN_LINEAGE.md) for the full lineage, including why D reuses
Chain C's genesis hash. Chain C's stall is recorded in
[EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md](EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md).
Dated baseline snapshots:
[CEIBA_BASELINE_2026-05-02.md](CEIBA_BASELINE_2026-05-02.md) (superseded),
[CEIBA_BASELINE_2026-04-29.md](CEIBA_BASELINE_2026-04-29.md).

- Runtime: Docker Compose stack in `/opt/belizechain`
- Core container: `ceiba-node`
- Genesis: `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef`
  — **shared with Chain C**; the hash alone does not identify the chain
- On-chain runtime: `spec_version = 109` (hot-upgraded 2026-10-07 from 107 — see
  the 109 ops-log entry at the end of this file)
- Epoch: `BabeEpochDuration = 300` slots (~30 min) via the `testnet-fast-epoch`
  build feature. The default build keeps 14,400 slots (~24 h), which is the
  mainnet value; 300 is a testnet convenience for observing rotations.
  **Any forward runtime upgrade must be built with `--features testnet-fast-epoch`** —
  deploying a blob built without it silently changes the epoch length and halts
  block import. That is exactly how Chain C died.
- Active chain spec: `/data/chain/testnet-spec.json`
  (operator-managed on the host; **not tracked in git** since 2026-09-20 because it
  carries the live sudo/session keys — see RULE 4 in [TESTNET_ONLY_RULE_2026-09-18.md](TESTNET_ONLY_RULE_2026-09-18.md))
  — it still embeds runtime **107**, so a re-genesis boots at 107 and must be
  upgraded forward
- Active chain data: `/data/chain/chains/belizechain_testnet`
- Command shape: `belizechain-node --chain /data/chain/testnet-spec.json --base-path /data/chain --port 30333 --rpc-port 9944 --prometheus-port 9615 --prometheus-external --rpc-cors all --unsafe-rpc-external --rpc-methods Safe --name Ceiba-Node-1 --validator --force-authoring`
  (`--force-authoring` is REQUIRED for Ceiba's single-node-before-peers state;
  the libp2p `--node-key`/`--node-key-file` is supplied from the host environment
  and is deliberately not recorded here)
- P2P: 30333 (public)
- RPC: 9944 (bound to the host's Tailscale address)
- Prometheus: 9615 (bound to the host's Tailscale address)

Health checks worth running after any restart or re-genesis:

`scripts/verify_epoch_config.py <rpc-url> <expected-duration>` — asserts the
on-chain BABE authority set is **non-empty**. An empty set is unrecoverable and
is what stalled the chain on 2026-09-23; it is worth checking on every boot
rather than discovering days later from a silent `Idle` log.

`sudo systemctl start ceiba-live-actions.service` — the functional smoke test:
Pakit upload/download/delete round-trip, Nawal chain connectivity **and validator
registration**, Kinich backend + jobs. Writes
`/data/log/live-actions/live-actions-YYYYMMDD.log`; expect `FAIL = 0`. This is the
check that catches a re-genesis silently dropping the Nawal operator registration —
see "Post-Reset: Re-register The Nawal Operator" below.

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

- Target host: `ceiba` (`ssh wicked@ceiba`, or `100.119.97.38` over Tailscale).
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
  http://100.119.97.38:9944
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
# The built-in template is dev-seeded, so it needs the explicit allowance; the
# generated spec is patched with operator keys further down.
docker run --rm -e BELIZECHAIN_ALLOW_DEV_SEEDS=1 "$NEW_IMAGE" build-spec \
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
  http://100.119.97.38:9944
curl -sS -H "Content-Type: application/json" \
  -d '{"id":2,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' \
  http://100.119.97.38:9944
curl -k -sS -H "Content-Type: application/json" \
  -d '{"id":3,"jsonrpc":"2.0","method":"system_health","params":[]}' \
  https://100.119.97.38/rpc
docker logs --tail 120 ceiba-node | grep -E "Idle|best: #|Imported|Starting consensus|Local node identity" || true
```

Successful recovery means `ceiba-node` is healthy, `/rpc` works through Nginx,
the header advances beyond `0x0`, and logs show authored/imported blocks after
the reset.

### Post-Reset: Re-register The Nawal Operator

**Required after every re-genesis — and easy to forget.** Identity and staking state
do not survive a genesis reset, so the Nawal AI operator silently drops out of
`Staking::Validators`. The chain keeps producing blocks, so nothing looks broken —
but `deploy/ceiba-live-actions.sh` starts failing with
`Nawal operator is not registered as a validator on chain`.

```bash
cd <repo>
export NODE_PATH="$PWD/ui/node_modules"      # this repo has no @polkadot deps; ui does
export RPC_ENDPOINT=ws://100.119.97.38:9944
export NAWAL_SEED="$(ssh wicked@ceiba 'docker exec ceiba-nawal printenv NAWAL_KEYPAIR_URI')"
node scripts/register-nawal-signer.js
```

The signer is the **keyless A1 account** (`ce2f6ecf…b902d054`) — it holds no session
keys by design and is PoUW-only. `BelizeSessionManager::new_session` filters it out of
the queued authority set, and that filter is exactly what keeps BABE at one authority
instead of announcing an empty next epoch. A log line naming it as excluded is
**expected, not an error**.

Verify after registering — `ValidatorCount` should be ≥ 1, the session set should
still be Alice alone, and BABE authorities must **not** be empty:

```bash
docker exec ceiba-nawal python -c "
from substrateinterface import SubstrateInterface
s = SubstrateInterface(url='ws://ceiba-node:9944')
print('validator count :', s.query('Staking','ValidatorCount').value)
print('session set     :', s.query('Session','Validators').value)
print('BABE authorities:', s.query('Babe','Authorities').value)
"
```

Then re-run the smoke test — expect `FAIL = 0`:
`sudo systemctl start ceiba-live-actions.service`

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

## Ops Log — 2026-09-03: Two-Validator Consensus Convergence & System Recovery

### What was resolved
- **Ceiba Docker Daemon Recovery**: On Sept 1, rapid crash-loops in `ceiba-nawal` caused `dockerd` to hang on its unix socket. Killed the hung `dockerd` process, restarted systemd socket and docker service.
- **Nawal Container Entrypoint**: Added explicit `entrypoint: ["python", "api_server.py"]` to `nawal` in `/opt/belizechain/docker-compose.ceiba.yml` so it executes directly without shell wrapper failure (`cannot open python: No such file`). Nginx now boots cleanly without 502/host-not-found errors for upstream `nawal:8080`.
- **Consensus & Finality Deadlock Resolution**:
  - During Ceiba's partition (Sept 1–3), Edge had authored solo via `--force-authoring` up to #8566, while Ceiba had local unfinalized blocks up to #8260.
  - Finality had stalled at #8077, causing BABE's finality lag backoff to trigger (lag > 480 blocks).
  - Synchronized the canonical ledger DB from Edge (`Edge-Validator-2`) to Ceiba (`Ceiba-Validator-01`), matching both nodes at block #8566 and synchronizing GRANDPA voter set round counters.
  - Upon coordinated restart, Ceiba and Edge achieved immediate 2-of-2 quorum in GRANDPA, finalized the chain up to head, reduced finality lag to <3 blocks, and resumed live block production (~6s intervals) with alternating VRF slot authorship.
- **Service Verification**:
  - `ceiba-node`: Healthy, peering (1 peer), authoring blocks.
  - `edge-node`: Healthy, peering (1 peer), authoring blocks.
  - Reverse proxy `/rpc` (Nginx): Functional and responding to JSON-RPC requests.
  - Web stack: UI (`/`), `/health`, `/api/nawal/health`, `/api/kinich/health`, and `/api/pakit/health` all returning HTTP 200.

## Ops Log — 2026-09-29: Empty-Authority Epoch Stall And Re-Genesis

Full write-up: [EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md](EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md).

### What happened
- Block production stopped at **#28791 on 2026-09-23 16:02:36** and did not resume
  for ~6 days. Nothing crashed; the log simply went silent (`💤 Idle`).
- Cause: a validator in `Staking::Validators` (`0xce2f6ecf…b902d054`, the Nawal AI
  signer) had **no session keys**. `pallet_session` silently dropped it while
  building the queued set, leaving it **empty**; BABE then announced a
  `NextEpochData` digest containing **zero authorities**. Past the end of that
  epoch no slot was claimable, so no block could be produced, so the epoch could
  never advance. **Unrecoverable in place**, and no backup predated the damage.
- Restarting did not help and could not — the client rebuilds its epoch tree from
  the same on-chain digest.

### What was done
- Fixed the runtime: `BelizeSessionManager::new_session` now filters to validators
  holding session keys and falls back to the current authority set, making an
  empty queued set unreachable. Validator entry paths gained a session-key check.
- Re-genesis via `scripts/regenesis-ceiba-emptyauth.sh`: new spec
  (`generate-testnet-spec.sh --verify` → born at spec 107), new image, wiped only
  `chains/belizechain_testnet/db` and `network/`. **Keystore preserved.**
- Verified at the exact failure point: the epoch-change digest now carries
  **1 authority (74 bytes)** where the dead chain had **0 (34 bytes)**, and a live
  session rotation was observed (epoch 0 → 1 at block #302) with the chain
  continuing to author through it.

### Current chain
- Image `belizechain/ceiba-node:509830c-emptyauth-20260929`
- Genesis `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef`
- Epoch 300 slots (`testnet-fast-epoch`)
- Forensic backup of the stalled database:
  `/data/regenesis-backup-20260929-115324/`

### Pitfalls hit (and rules of thumb)
- **`ceiba-nawal` used to latch a dropped chain connection** — fixed later the
  same day; see the runtime-108 entry below.
- **A `Dockerfile.runtime` start failure can be a permissions problem wearing a
  "missing file" costume.** `scp` drops the execute bit, `COPY` preserves the
  source mode, and the result is `exec: "belizechain-node": executable file not
  found in $PATH` from a cleanly built image. Verify with
  `docker run --rm --entrypoint /bin/sh <image> -c '<bin> --version'`.
- **Two operational lessons worth remembering:** a restart cannot fix state that
  the consensus client reads from the authoritative chain, and a backup is only a
  rollback asset if its capture time predates the failure. Check the failure time
  against the archive before promising a rollback.
- `ceiba-explorer` is Polkadot.js Apps (a client-side UI), not an indexer — a
  re-genesis requires nothing of it.
- **Accounts have two representations.** The chain sets its own `SS58Prefix`
  (**1981**), so one account renders as `r1Wm6WgK…` on-chain and `5GrwvaEF…`
  under the default prefix 42. Compare accounts by their 32 raw bytes, never by
  the rendered address.

## Ops Log — 2026-09-29 (later): Runtime 108 hot upgrade

Relaxes the session-key precondition added earlier the same day, which was
stricter than the failure required and blocked a legitimate path.

### Why
`scripts/register-nawal-signer.js` registers the A1 PoUW signer through
`staking.joinValidators` **without** session keys — it earns rewards and never
authors. The hard `ensure!` made that call fail with `SessionKeysNotRegistered`.
The actual protection against the empty-authority deadlock is
`BelizeSessionManager::new_session`, which was correct and is untouched.

### Change
The check is now a `log::warn!` on both join paths, and the
`SessionKeysNotRegistered` error variant is deleted. It was the **last** variant,
so no other error index moved, and there is no storage migration.

### Deployed
```
sudo.sudo(System::set_code(1421085 bytes))   extrinsic 0xe47e3b10…8448f
spec_version   : 107 -> 108     (no node restart; RestartCount stayed 0)
on-chain :code : 1421085 bytes  blake2b 8ad7ea9d18a02e7b5150cf68e455a68c06f84fa6a18c6e4db7c53befcaa45c0d
```

Afterwards: `Babe::Authorities` and `NextAuthorities` each hold 1 authority,
`Session::QueuedKeys` is non-empty, `epoch_length` is still 300, and the raw
metadata no longer mentions `SessionKeysNotRegistered` (its neighbour
`DomainContributionCapExceeded` is still present, so the absence is meaningful).

**Rollback:** `/data/upgrade-108/onchain-code-107.wasm` (1,420,530 bytes, blake2b
`cd7d2112…`) is the runtime the chain was running immediately before the change,
verified byte-identical at capture time. Re-deploy with
`scripts/upgrade-runtime.py <file> --execute`.

**Tooling:** `scripts/upgrade-runtime.py` — dry run by default, refuses to sign
unless the local key matches the on-chain sudo key, and re-verifies `spec_version`
and `:code` after submission.

**Note:** the genesis runtime inside `testnet-spec.json` is still **107**, so a
future re-genesis would boot at 107 and upgrade forward. Re-run
`scripts/set-spec-code.py` if that matters.

`scripts/generate-testnet-spec.sh --check` now reports that drift without
modifying anything (read-only; exits non-zero on drift).

### A1 signer re-registered (2026-09-30) — proof the guard is safe

`staking.joinValidators` for the A1 Nawal signer previously failed with
`SessionKeysNotRegistered`; it now succeeds. This deliberately recreates the
precondition that emptied the authority set on 2026-09-23 (the signer is
`0xce2f6ecf…b902d054`, keyless by design). The next epoch boundary
(**53 → 54**, slot `298465741`) crossed with it registered:

| Evidence | Value |
|---|---|
| `Babe::Authorities` after the boundary | **1** (was **0** in the incident) |
| Runtime filter | `BelizeSessionManager: 1 staking validator(s) have no session keys and were excluded` |
| `STALL DETECTED` / empty-epoch / skipped | **0 / 0 / 0** |
| Blocks after the boundary | 63 and climbing |

Re-run it any time with:

```bash
  RPC_ENDPOINT=ws://100.119.97.38:9944 \
    NAWAL_SEED="$(ssh wicked@ceiba 'docker exec ceiba-nawal printenv NAWAL_KEYPAIR_URI')" \
    NODE_PATH=/home/wicked/Projects/Belizechain/ui/node_modules \
    node scripts/register-nawal-signer.js
  ```
  
  It is idempotent, self-funds from the issuer, and never logs the key.
  `RPC_ENDPOINT` is **required** — the script exits 2 without it, because a hardcoded
  fallback silently targets whichever host was live when the script was written.

  **Re-done on Jade (Chain D) 2026-10-07** — see "Post-Reset: Re-register The Nawal
  Operator" above. The registration does **not** survive a re-genesis, so this is a
  standing post-reset step, not a one-off. On Jade the epoch boundary **31 → 32**
  crossed with the signer registered and `Babe::Authorities` stayed **1**.

### Monitoring now actually reports a stall

- **Alertmanager IS deployed and pages Telegram.** Added 2026-10-02
  (`ceiba-alertmanager`, Tailscale `:9093`); Prometheus has an `alerting:` block
  pointing at `alertmanager:9093`, and the rendered config routes to Telegram
  (`TELEGRAM_BOT_TOKEN` / `TELEGRAM_CHAT_ID` in `/opt/belizechain/.env`). **Delivery
  was verified end-to-end on 2026-10-02** — a test alert reached the operator's phone.
  Re-confirmed 2026-10-07: container up, 1 active Alertmanager in Prometheus,
  `telegram_configs` present, **0 unsubstituted placeholders**, default route →
  telegram. ⚠️ *An earlier version of this section claimed "there is no Alertmanager…
  notifies nobody" — that was true before 2026-10-02 and is now wrong.*
- **Reading the logs:** Alertmanager logs a successful notify at **DEBUG only**.
  Absence of "Notify" lines is **not** evidence of failure — only `ERROR` or
  `Notify attempt failed` lines are. Silence in the log means nothing either way.
- What *also* reports, independently of Alertmanager: `deploy/health-check.sh` (every
  5 min via `ceiba-health-log.timer`) reads the chain head twice, 15 s apart, and fails
  if it has not advanced. A halted chain shows up in `/data/log/health` within minutes.
  `ceiba-live-actions.timer` (4×/day) adds a functional smoke test. Both propagate a
  failed status to systemd, so `systemctl --failed` shows them.
- **A responsive node is not an authoring node.** On 2026-09-23 the chain was
  frozen for six days while `system_health` answered normally and the container
  stayed `Up`; health checks passed the whole time. Only block progress is
  evidence of block production.

### Chain archives now record what they contain

`deploy/backup-chain.sh` writes `<archive>.manifest.json` next to each archive with
`head_block`, `finalized_block`, `spec_version`, `genesis_hash` and `taken_utc`.
Before restoring, compare an archive's `head_block` against the incident's block:
an archive whose head is **above** the incident **cannot** roll it back, however
recent its mtime looks. (On 2026-09-29 the newest archive was taken six hours after
the chain froze.)

## Ops Log — 2026-10-04: Runtime 109 hot upgrade (CONS-006)

> 🔴 **THIS UPGRADE KILLED CHAIN C.** It looked successful for ~179 blocks and is
> preserved here as the failure record, not as a procedure to copy. The build note
> below is the **defect**: it instructs building "no feature flags", which produces
> `BabeEpochDuration = 14_400` instead of `300`. See the 2026-10-07 entry for the
> correct build.

Ships CONS-006 (merged 2026-10-02): `AIAuthorityOrigin` moves from any single
TechnicalCouncil member to `TechnicalCouncilSuperMajority` — a >2/3 council
motion, or Root. No storage migration; affects `validate_ai_model`,
`start_consensus_round`, `finalize_consensus_round`.

### Deployed
```
sudo.sudo(System::set_code(1420195 bytes))   extrinsic 0x06d6f422b4301a2f80c104e53d78afbda90742f06ee538dd4f3a680bbe6792a3
spec_version   : 108 -> 109     (no node restart; container uptime predates the upgrade)
on-chain :code : 1420195 bytes  blake2b 00a1c38a7bc9463fe07f0e6880a1625f8e15951ee88c6fe4ae3e7c839287375c
```

Applied in **block #70616**, which carries `System.CodeUpdated` +
`Sudo.Sudid { sudo_result: Ok }` from the sudo key. Verified on fresh connections:
`specVersion = 109`, the on-chain `:code` is byte-identical to the submitted blob.

⚠️ **The "authoring stayed healthy" claim was true only briefly.** Block import
continued to #70,795, then stopped permanently at the next epoch boundary
(#70,800) — the blob carried a 14,400-slot epoch while the chain had been running
300, so producer and verifier went out of step and every subsequent block failed
with `Expected epoch change to happen at …`. The chain was unrecoverable in place
and was re-genesised as Chain D. The `00a1c38a…` blob is confirmed to report
`duration = 14400` when booted; the 2026-10-07 blob reports `300`.

**Rollback:** `/data/upgrade-109/onchain-code-108.wasm` (1,421,085 bytes, blake2b
`8ad7ea9d…`) — the runtime the chain was running immediately before the change,
captured from `:code` and verified byte-identical at capture time. Re-deploy with
`scripts/upgrade-runtime.py <file> --execute`.

**Build note (SUPERSEDED — this is the bug):** "rebuild the runtime before
extracting the deploy blob (`cargo build --release -p belizechain-runtime`, no
feature flags)". The `no feature flags` instruction is what changed the epoch
length. `system.set_code` rejects a blob whose `spec_version` does not strictly
increase, so a *stale* blob is caught — but a blob with the wrong **features** and
a higher version is accepted and then halts the chain. Version monotonicity does
not protect against a wrong build.

## Ops Log — 2026-10-07: Runtime 109 hot upgrade on Chain D (correct build)

The first *correct* deployment of 109. Same code change as the 2026-10-04 attempt
(CONS-006); the difference is the build.

### Build
```
cargo build -p belizechain-runtime --release --features testnet-fast-epoch
```
The feature is **required**. Proven present by booting a throwaway chain from the
artifact and reading `BabeApi_current_epoch`:
```
new blob      duration = 300     <- correct
Oct-4 blob    duration = 14400   <- the Chain C killer
```
Same test, opposite answers. `scripts/upgrade-runtime.py` performs this check
automatically and **refuses to proceed if the duration would change**, which is the
guard that would have caught the 2026-10-04 mistake.

### Deployed
```
sudo.sudo(System::set_code(1419740 bytes))
spec_version   : 107 -> 109     (no node restart; restarts=0 throughout)
on-chain :code : 1419740 bytes  blake2b 6c9b884cdf6633ace3efa452d93d3ebebd5112d332d76d1ae9f4a0ce5a104c9e
```

### Verified
- `specVersion = 109`, on-chain `:code` byte-identical to the submitted blob.
- **Epoch boundary crossed live** — epoch 4 → 5, start slot `298555045 → 298555345`
  (exactly +300). `Babe::Authorities = 1` across the transition.
- Block production continued (head 1461 → 1516+) and GRANDPA finality kept pace
  (finalized ~2 blocks behind head).
- No errors, panics, stall guards, or `Expected epoch change` lines in the node log.

This is the same transition that ended Chain C, and it succeeded — which is the
evidence that the epoch-duration defect is fixed, not merely avoided.

**Rollback:** `/data/upgrade-109/onchain-code-107.wasm` on `ceiba` (1,420,530 bytes,
blake2b `cd7d21125cf4c1fac0f2d21c228c8feab5f0ed42089bcdf04eb779cd2b17c48d`). Spec
goes *down* on rollback, so it needs the `--without-checks` path:
`scripts/upgrade-runtime.py <file> --without-checks --execute`.

✅ **FIXED 2026-10-07 — `scripts/upgrade-runtime.py` no longer carries a hardcoded
default.** `--rpc` is now **required** (or `CEIBA_RPC_URL`), and the script exits 2
naming the live endpoint instead of silently targeting whichever host was live when it
was written. The same sweep removed the stale literal from
`regenesis-ceiba-emptyauth.sh`, the `scripts/test/*` helpers, and
`register-nawal-signer.js` / `assign-fl-task.js`. Still state the endpoint explicitly:
`--rpc ws://100.119.97.38:9944`.

The genesis runtime inside `testnet-spec.json` is still **107**; a future
re-genesis would boot at 107 and upgrade forward (107 → 108 → 109 so far).

