# Chain Lineage — BelizeChain Testnet

**Status:** Authoritative record of every testnet chain, its genesis, and why it ended.
**Last verified:** 2026-10-07

This file exists because the lineage was **not written down anywhere**, and that gap
cost real time. Four separate chains have run (three on `ceiba`, the current one on
`ceiba2`), and confusion between them produced wrong conclusions during the 2026-10-06
investigation (an operator reasonably believed the current chain's genesis was
Independence Day; it is not — that was the *previous* chain).

> **Rule:** whenever the chain is re-genesised, add a row here **in the same commit**
> as the operation. A genesis hash with no lineage entry is indistinguishable from a
> wrong hash.

## Summary

| # | Genesis hash | Born (UTC) | Died (UTC) | End reason |
|---|---|---|---|---|
| A | *not recorded* | before 2026-08-17 | 2026-08-17 | Re-generisised (see backup `chain-pre-reset-20260817`) |
| **B** | `0x631fb936…` | **2026-09-21 16:02:42** | 2026-09-23 16:02:36 | Empty BABE authority set → total deadlock at block #28,791 |
| **C** | `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef` | **2026-09-29 ~15:00** | 2026-10-04 | Runtime 109 deployed without `testnet-fast-epoch` → epoch-duration mismatch → no block imported after #70,795 |
| **D** | `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef` — **same hash as C** | **2026-10-06** (on `ceiba2`) | — | — |

⚠️ **D reuses C's genesis hash.** D was re-genesised from the *same spec* C was, and a
chain spec fully determines the genesis state — so the hash is byte-identical. The
genesis hash therefore does **not** uniquely identify a chain across a re-genesis:
C and D share it while being different chains at different blocks. Use the host
(`ceiba` vs `ceiba2`) plus the head height to disambiguate, never the hash alone.

## Chain D — current, live on `ceiba2` (2026-10-06 →)

Re-genesised from the Chain C spec onto the replacement host `ceiba2`, which is why the
genesis hash matches C. The spec embeds runtime **107** (blake2b `cd7d21125c…`,
1,420,530 B — the same blob as C), so D was **born at 107**.

**2026-10-07: D hot-upgraded 107 → 109** — the first *correct* deployment of 109.
Done with `sudo.sudo(System::set_code)`; the node was never restarted (`restarts=0`).

The blob was rebuilt with the feature the previous attempt omitted:

```
cargo build -p belizechain-runtime --release --features testnet-fast-epoch
```

- new `:code` `6c9b884cdf6633ace3efa452d93d3ebebd5112d332d76d1ae9f4a0ce5a104c9e`, 1,419,740 B
- the feature is proven present: a throwaway chain booted from the artifact reports
  `BabeApi_current_epoch.duration = 300`; the *previous* artifact (built without the
  feature) reports **14400**. Same test, opposite answers — that pair is the evidence
  that the Oct-4 blob was the Chain C killer.
- **The epoch boundary was crossed live** (epoch 4 → 5, start slot +300) with
  `Babe::Authorities = 1`, block production continuing and GRANDPA finality keeping
  pace. This is the exact transition that ended Chain C.

**Rollback blob:** `/data/upgrade-109/onchain-code-107.wasm` on `ceiba2` — the
pre-upgrade on-chain 107, verified `cd7d21125c…`. Roll back with
`upgrade-runtime.py <blob> --without-checks --execute` (`set_code` refuses a spec
*down*).

⚠️ **A fresh re-genesis from `testnet-spec.json` still boots at 107**, because the
spec was never re-embedded. That is not a defect — the spec is operator-managed and
gitignored — but it means a re-genesis silently returns the chain to 107 and needs
this same upgrade re-applied.

## Chain B — the Independence Day genesis (2026-09-21)

**Genesis deliberately set on 21 September, Belize Independence Day.** Confirmed by
the operator 2026-10-06. This is **not** recorded in
`CHAIN_RESET_AND_AUTHORING_GUARDS_2026-09-21.md`, which reads as a purely emergency
response to a corrupted database — the symbolic intent has to be stated somewhere, so
it is stated here.

- Genesis slot `298334427` → **2026-09-21 16:02:42 UTC** (`Babe::GenesisSlot`).
- Corroborated by the preserved spec `testnet-spec.old.json`, timestamped
  **Sep 21 16:02**, in `/data/regenesis-backup-20260929-115324/`.
- Self-consistent: genesis 16:02:42 on the 21st → last block #28,791 at 16:02:36 on the
  23rd is **exactly 48 h**, and 28,791 × 6 s = 48.0 h.
- Died from an empty BABE authority set — see
  `EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md`.

⚠️ Chain B's genesis hash is **not** reusable without accepting a duplicated chain
identity: re-launching from its spec produces the same genesis hash but a different
chain at block #0. Do not do this casually.

## Chain C — current, stalled (2026-09-29 → 2026-10-04)

- Genesis derived two independent ways: the Oct-1 backup (head 22,052 at
  2026-10-01 04:39:18, minus 22,052 × 6 s → 2026-09-29 ~15:54) and the regenesis run
  itself (~15:00–15:53).
- **Root cause of death (verified 2026-10-06):** `BabeEpochDuration` is a
  *compile-time* constant — `300` with the `testnet-fast-epoch` feature, `14_400`
  without (`runtime/src/lib.rs`). The runtime 109 blob was built **without** that
  feature and hot-upgraded at block #70616. The live chain had been running 300-slot
  epochs since genesis (epoch index 235 at head #70,795 → `235 × 300 = 70,500` ✓), so
  the change to 14,400 slots put producer and verifier out of step. Every block after
  #70,795 failed with `Expected epoch change to happen at …` and the chain never
  advanced again.
- **Not hardware.** Host uptime ran unbroken across the stall; no MCE/ECC errors; the
  node container recorded `restarts=0`. (Ceiba *did* have genuine RAM faults on
  2026-09-18, and that history stands on its own — but it did not cause this.)
- **Not recoverable in place.** A runtime change must be included in an imported
  block, and no block could be imported. The sudo extrinsic sat in the transaction
  pool, was packed into every candidate block, and every candidate was rejected.

## Resets and snapshots on disk

`/data/chain-backups/` on Ceiba:

| Archive | Date |
|---|---|
| `chain-pre-node-rollout-20260429-014556.tar.gz` | 2026-04-29 |
| `chain-pre-alice-sudo-reset-20260511-233903.tar.gz` | 2026-05-11 |
| `chain-pre-reset-20260817-180811.tar.gz` | 2026-08-17 |
| `chain-weekly-20260917-003335.tar.gz` | 2026-09-17 |
| `chain-pre-alice-sudo-reset-20260917-132117.tar.gz` | 2026-09-17 |

Chains **A** (pre-Aug-17) and the earlier April/May activity are not documented
beyond these archives. Treat pre-Sep-21 genesis details as **unverified**.

## Runtime lineage

```
105 ──(2026-09-20)──> 107 ──(2026-09-29)──> 108 ──(2026-10-04)──> 109
                                                        ↑
                                            deployed WITHOUT testnet-fast-epoch
                                            (the defect that killed Chain C)
```

- Genesis spec `/data/chain/testnet-spec.json` embeds runtime **107** (blake2b
  `cd7d21125cf4c1fac0f2d21c228c8feab5f0ed42089bcdf04eb779cd2b17c48d`, 1,420,530 B),
  verified 2026-10-06 by hashing `genesis.runtimeGenesis.code` and matching it to
  `/data/upgrade-108/onchain-code-107.wasm`.
- 107 is a **300-slot / fast-epoch** build, so a re-genesis from this spec is
  healthy out of the box. A re-genesis is *not* a workaround for the epoch bug — the
  bug only appeared when a wrongly-built 109 was applied.
- **Any forward runtime upgrade must be built with
  `--features testnet-fast-epoch`** and must pass the epoch-duration guard in
  `scripts/upgrade-runtime.py`, which fails the upgrade if the duration would change.

## Genesis authorities

The spec's `runtimeGenesis.patch` sets **Alice** (`5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`,
raw `d43593c7…da27d`) as BABE, GRANDPA, session, and sudo authority, with balances
pre-funded. The node keystore therefore must contain Alice's `babe` and `gran` session
keys (`/data/chain/chains/belizechain_testnet/keystore/`) — a re-genesis needs the
**keystore copied**, but never the old chain database.
