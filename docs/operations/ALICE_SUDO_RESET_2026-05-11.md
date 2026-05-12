# Alice-Sudo + Authority Reset (Ceiba, 2026-05-11)

**Status:** Executed and verified. Chain progressing; Sudo operational; Nawal FL
round opened on-chain.

## Context

The committed `testnet-spec.json` referenced a sudo SS58
(`5CPnByRew...`) whose SURI was generated during an earlier session and never
persisted to disk. Without that SURI we could not sign Sudo extrinsics
(`Staking.assign_fl_task` was blocked). Separately, the BABE/GRANDPA validator
mnemonics had been written to plaintext files under
`/data/chain/authority-keys-*` and exposed in chat history.

Constraint: this is a single-node testnet with no peers, no users beyond the
operator, and no production value at stake. A full genesis reset was therefore
acceptable.

## Decision

Rotate genesis to the well-known `//Alice` dev seed for sudo, validator, and
council bootstrap. This is a **testnet-only** stance. Production cutover
requires replacing `//Alice` with a real, hardware-backed seed before opening to
external peers.

## Runbook

All commands run from the operator workstation against `wicked@ceiba` over
Tailscale.

### 1. Pre-flight

```bash
ssh wicked@ceiba "docker -H ssh://wicked@ceiba ps && \
  curl -sS -H 'Content-Type: application/json' \
  -d '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_health\",\"params\":[]}' \
  http://localhost:9944"
```

Confirm the node is healthy and you know which block you're resetting from.

### 2. Backup

```bash
ssh wicked@ceiba 'sudo tar czf \
  /data/chain-backups/chain-pre-alice-sudo-reset-$(date +%Y%m%d-%H%M%S).tar.gz \
  /data/chain'
```

Verify size and readability before continuing.

### 3. Stop the node

```bash
ssh wicked@ceiba 'cd /opt/belizechain && docker compose stop ceiba-node'
```

### 4. Preserve the old spec and chain directory

```bash
ssh wicked@ceiba '
  STAMP=$(date +%Y%m%d-%H%M%S)
  sudo cp /data/chain/testnet-spec.json /data/chain/testnet-spec.pre-alice-sudo-reset-$STAMP.json
  sudo mv /data/chain/chains/belizechain_testnet \
          /data/chain/chains/belizechain_testnet.pre-alice-sudo-reset-$STAMP
'
```

### 5. Rewrite genesis to Alice

Edit `/data/chain/testnet-spec.json` so that, under `genesis.runtime.patch`:

- `sudo.key`: `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY` (//Alice)
- `session.keys[0]`: Alice's stash, Alice's controller, `{ "babe":
  "0xd43593c7…", "grandpa": "0x88dc3417…" }` (Alice sr25519 + Alice ed25519).
- Remove the `babe.authorities` array — the session pallet will derive it from
  `session.keys` at genesis.
- Add `5GrwvaEF…HGKutQY` to `balances.balances` with `1000000000000000000`
  units; keep the other two endowed accounts.
- Replace the leading entry in `governance.councilMembers`,
  `identity.initialBiometricIssuers`, `initialPassportIssuers`, and
  `initialSsnIssuers` with `5GrwvaEF…HGKutQY`.

The full diff for this rotation is in commit `28904ed` on
`feature/alice-sudo-testnet-spec` (PR #14).

### 6. Seed the keystore

Substrate's filesystem keystore expects filenames of the form
`<key_type_hex><pubkey_hex>` containing a JSON-quoted SURI.

```bash
ssh wicked@ceiba '
  KEYSTORE=/data/chain/chains/belizechain_testnet/keystore
  sudo mkdir -p "$KEYSTORE"
  # babe = 0x62616265, alice sr25519 pubkey = 0xd43593c7…
  echo -n "\"//Alice\"" | sudo tee "$KEYSTORE/62616265d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d" >/dev/null
  # gran = 0x6772616e, alice ed25519 pubkey = 0x88dc3417…
  echo -n "\"//Alice\"" | sudo tee "$KEYSTORE/6772616e88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee" >/dev/null
  sudo chown -R 1000:1000 "$KEYSTORE"
'
```

### 7. Persist the libp2p network key

```bash
ssh wicked@ceiba '
  NETDIR=/data/chain/chains/belizechain_testnet/network
  sudo mkdir -p "$NETDIR"
  sudo openssl rand 32 | sudo tee "$NETDIR/secret_ed25519" >/dev/null
  sudo chown -R 1000:1000 "$NETDIR"
'
```

The file must be **raw 32 bytes**, not hex.

Also set a deterministic `NODE_KEY` in `/opt/belizechain/.env` so the compose
flag `${NODE_KEY:+--node-key ${NODE_KEY}}` resolves to a stable node identity
across restarts:

```bash
ssh wicked@ceiba '
  NK=$(openssl rand -hex 32)
  sudo sed -i "/^NODE_KEY=/d" /opt/belizechain/.env
  echo "NODE_KEY=$NK" | sudo tee -a /opt/belizechain/.env >/dev/null
'
```

### 8. Start the node

```bash
ssh wicked@ceiba 'cd /opt/belizechain && docker compose up -d ceiba-node && \
  docker logs --tail 80 ceiba-node'
```

Watch for `Imported #1`, then steady `Idle` / `Pre-sealed block` cadence with
finalized height advancing.

### 9. Verify sudo

```bash
python scripts/assign_fl_task.py \
  --ws ws://100.81.45.25:9944 \
  --sudo-suri //Alice \
  --validator 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --task-id chain_task_$(date +%s)
```

Expected: `Staking.FLTaskAssigned` event in the block.

### 10. Restart dependents

Restart any service holding a stale WebSocket to the pre-reset chain (Nawal in
particular):

```bash
ssh wicked@ceiba 'docker restart ceiba-nawal'
curl -sS -X POST http://100.81.45.25:8000/api/v1/fl/rounds \
  -H 'Content-Type: application/json' -d '{"min_participants":1}'
```

Expected: `{"round_id":"chain_task_…","status":"pending"}` and
`blockchain_connected: true` from `/health`.

### 11. Cleanup (post-success)

```bash
ssh wicked@ceiba '
  for d in /data/chain/authority-keys-*; do
    [ -d "$d" ] || continue
    sudo find "$d" -type f -exec shred -u {} +
    sudo rm -rf "$d"
  done
'
```

Keep the timestamped pre-reset backup tarball; remove only after a multi-day
healthy window.

## Caveats

- `//Alice` is a public, well-known seed. Any party can sign sudo extrinsics on
  this chain. Acceptable only because the node is single-tenant and not
  peered. **Rotate before exposing to external validators.**
- `cargo run --bin subkey` was not used; the keystore filenames were derived
  from known Alice pubkeys. For any non-Alice rotation, derive the pubkey via
  `subkey inspect <suri> --scheme sr25519 / --scheme ed25519` first.
- The compose env-var pattern `${NODE_KEY:+--node-key ${NODE_KEY}}` silently
  drops `--node-key` when `NODE_KEY` is empty, so persistence requires a
  non-empty value in `.env`.

## References

- PR #14 — `feature/alice-sudo-testnet-spec` (committed spec)
- PR #13 — `feature/assign-fl-task-script` (merged 2026-05-12)
- Backup: `/data/chain-backups/chain-pre-alice-sudo-reset-20260511-233903.tar.gz`
- Repo memory: `/memories/repo/ceiba-ops.md`
