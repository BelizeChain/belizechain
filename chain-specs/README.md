# `chain-specs/` — what each file is for

Three specs, three jobs. **They are not interchangeable.**

| File | Purpose | Safe to boot the live chain from? |
|---|---|---|
| `../testnet-spec.json` (repo root, gitignored) | The genesis that **already happened**; Ceiba boots from a byte-identical copy at `/data/chain/testnet-spec.json` | ✅ Yes — it *is* the current chain |
| `genesis-live-0xb2664568.json` | The same bytes, **tracked in git**, so the live chain's origin cannot be lost with the host | ✅ Yes (identical) — never use it to start a *different* chain |
| `genesis-hardened-2026-11-07.json` | A genesis for a **future** chain, with every privileged slot moved off well-known dev keys | ❌ No — it produces a different chain |
| `chain-spec-production.json` | Mainnet **placeholder**, not a working spec (see below) | ❌ No — it does not parse |

## The live chain's origin, measured 2026-11-08

```
spec genesis hash : 0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef
live  genesis hash: 0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef
spec md5          : 31171e1020cc772d47ace036bb6be1fa
```

Both hashes were read in the same session: the live one from `chain_getBlockHash(0)`, the
spec one by booting `belizechain-node --chain testnet-spec.json --base-path <scratch>`.
They match, so this file **is** the origin of the running chain — it is not a stale copy.

## This spec cannot be edited, and that is the whole point

`genesis-live-0xb2664568.json` still pins four publicly-known dev keys across nine
privileged slots. Measured with `scripts/harden-genesis-spec.js --check-only`:

```
✗ 4 publicly-known dev key(s) still present:
    5GrwvaEF… → //Alice (sr25519)     sudo.key, session.keys[0].babe,
                                      identity.*Issuers (×3), councilMembers[0]
    5FHneW46… → //Bob (sr25519)       councilMembers[1]
    5FLSigC9… → //Charlie (sr25519)   councilMembers[2]
    5FA9nQDV… → //Alice (ed25519)     session.keys[0].grandpa
```

Blocker #13 rotated all of these **in running storage**; the genesis still describes the
day-one values. Removing them from the file changes the genesis hash under the same chain
id, and the node then refuses the chain data it has been authoring for months. The result
is not a hardened chain — it is a node that will not start.

So the operational file stays as it is, and it is now tracked here for one reason: **the
genesis was the one piece of the chain no backup contained.** `deploy/backup-chain.sh`
archived `/data/chain/chains/belizechain_testnet` and nothing else, so every archive taken
before 2026-11-08 could restore a database that no spec on earth still matched. That gap is
closed: the script now archives the spec beside the database, and
`verify-chain-backup.sh` prefers the copy inside the archive.

**Consequence to accept:** getting belizechain keys into the *boot* spec requires a
re-genesis, which is a new chain — different genesis hash, no history, no contracts. That
is a deliberate decision, not a cleanup step. Until one is made, a re-genesis from this
file restores `//Alice` as sudo.

## `genesis-hardened-2026-11-07.json`

Produced by:

```bash
NODE_PATH=../ui/node_modules node scripts/harden-genesis-spec.js \
  --sudo      0x32967ce9efd49154ebea4afd4b599563a7dae718a2da237f10f2abed2f3fb017 \
  --validator 0x6e451283e1e4dc9ab5bc8b624c7f743c11b626cbc59dca2cbdf7723332cc981e \
  --grandpa   0xfdf2056326eb4a755279d81f2449c13441e9d207f7c86c70517fafa27a7daced \
  --out chain-specs/genesis-hardened-2026-11-07.json
```

What it changes relative to the operational spec:

| Slot | Operational spec | Hardened spec |
|---|---|---|
| `sudo.key` | `//Alice` (`5GrwvaEF…`) | `5DD2xP514R5ZprsrPYaDaSYaCh1Pm2DBh1PdeX2sz2oYQ3yW` |
| `session.keys` | 1 authority, `//Alice` BABE + `//Alice` grandpa | 1 authority, `5EZHeG8L…` BABE + `5HofsFi7j…` grandpa |
| `identity.*Issuers` (×3) | `//Alice` | the root account |
| `governance.councilMembers` | `//Alice`, `//Bob`, `//Charlie` | 1 seat, root account |
| `balances` | 1,000,000 DALLA to `//Alice` | moved to the root account |

Every `//Alice`/`//Bob`/`//Charlie`/`//Dave`/`//Eve`/`//Ferdie` seed (plus `//stash`
variants) is published in Substrate's documentation. Any of them in a privileged slot is a
chain anyone can administer.

The filename carries a date because the keys will rotate again. A spec older than the
current key rotation should be treated as stale, not reused.

## `chain-spec-production.json` is a placeholder

Measured 2026-11-08:

```bash
./belizechain-node build-spec --chain chain-specs/chain-spec-production.json
# Exit 1 — multiaddr parsing error: invalid IPv4 address syntax
```

It carries literal `NODE1_IP` / `NODE1_PEER_ID` placeholders and a legacy `genesis.runtime`
block naming pallets that no longer exist. It has never booted anything. A real mainnet spec
has to be generated from the runtime with mainnet keys, which do not exist yet.

## Verify before trusting a spec

```bash
# Fails (exit 1) and names every offending slot while a dev key remains.
NODE_PATH=../ui/node_modules node scripts/harden-genesis-spec.js \
  --check-only --spec chain-specs/genesis-hardened-2026-11-07.json

# Confirm at the raw genesis-storage layer, not just in the patch JSON.
./belizechain-node build-spec --chain chain-specs/genesis-hardened-2026-11-07.json --raw > /tmp/raw.json
```
