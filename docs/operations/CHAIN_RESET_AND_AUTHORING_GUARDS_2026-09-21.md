# Chain Reset, Authoring Guards, and Born-at-107 Genesis (Ceiba, 2026-09-21)

**Status:** Executed and verified. The Ceiba testnet runs a fresh chain that is
**spec 107 from genesis**, with a validator that can no longer stall silently.
This is a **testnet-only** operation under
[TESTNET_ONLY_RULE_2026-09-18.md](TESTNET_ONLY_RULE_2026-09-18.md).

## 1. What happened

A node image redeploy (`a32223f-spec107-20260921`) failed with:

```
Error: Service(Client(Backend("Invalid argument: Column families not opened: col12, col11, … col0")))
```

Reverting the image tag did **not** restore service — the previous binary then
failed identically. The live database had been damaged.

Restoring the nightly backup brought the chain *up* but **not authoring**: it sat
at block 39013 producing zero blocks for hours with no error at any log level.

### Root cause of the crash: not determined

Evidence gathered, all verified:

| Observation | Implication |
|---|---|
| Both DBs expose the same CF set (`col0`–`col12`) | The "column families" message is misleading; the CF set is not the difference |
| Live DB: `MANIFEST-000512`, 622 WAL files, 2.2 GB | Differs from the backup's `MANIFEST-001464`, 1 WAL, 1.3 GB |
| Old **and** new binaries both open the backup fine | The binaries are not the cause |
| New binary authors a fresh chain perfectly | Binary and key material are fine |
| Keystore files identical and readable (`wicked:wicked`, uid 1000) | Keystore was never the problem |

The DB itself is the only remaining variable. **Treat as unresolved**; the
corrupted DB was deliberately deleted during cleanup, so a re-run of this
investigation is no longer possible.

### Why the restored chain could not author

Reproduced deliberately by booting a copy of the restored DB **with a known-good
keystore**: identical silent stall. On-chain state was healthy throughout —
authorities 1 (weight 1), `epochIndex` 3, `EpochStart` = (15469, 29869) with the
head at 39013, i.e. **inside** the 14400-block epoch at block 29869+.

The failure is in the client's slot-claim path, which is **silent by
construction**:

```rust
// sc_consensus_slots: on_slot()
let claim = self.claim_slot(&slot_info.chain_head, slot, &aux_data).await?;   // <- silent exit
```

`claim_slot` returns `None` and `on_slot` returns with no further statement. At
`info` level the operator sees only idle lines; the `Attempting to claim slot N`
message is `debug`. **This is the defect class that cost hours.**

Also relevant: `slot_lenience_exponential` is **hard-capped** at `2^7` slots
(128 slots ≈ 8.6 min at 6 s). A chain stalled longer than that can never bridge
the gap by lenience alone — it will not self-heal. That is SDK behaviour, not a
BelizeChain patch target; the fix is to make the silent case *loud*.

## 2. What was fixed

### 2a. Silent authoring failures are now impossible (`node/src/authoring_guard.rs`)

Two guards, both observational — neither changes consensus behaviour:

**GUARD-1 — startup preflight.** Cross-checks the head's on-chain BABE and
GRANDPA authority sets against the keys actually present in the keystore. A
validator holding no matching BABE key gets an `ERROR` naming the problem and the
consequence. Verified live:

```
authoring preflight: validator at #0 — BABE authorities on-chain 1 / matched local keys 1;
GRANDPA authorities on-chain 1 / matched local keys 1
```

**GUARD-2 — stall watchdog.** If this validator holds matching keys yet no block
has been imported for `STALL_ALERT_SLOTS` (100 ≈ 10 min), it escalates to `ERROR`
with a diagnostic snapshot (best/finalized, on-chain authorities, matched local
keys), repeating at most every 5 minutes, and logs recovery when production
resumes.

Both use log target `authoring-guard`; filter with
`RUST_LOG=authoring-guard=info` (the preflight/warning path) or `=debug`.

### 2b. "107 everywhere" — genesis is born at spec 107

`testnet-spec.json` embedded the **spec-105** runtime (`3,579,880` hex chars), so
every fresh chain booted at 105 and needed a second upgrade pass. The spec now
embeds the spec-107 runtime:

| | Before | After |
|---|---|---|
| Embedded runtime | 1,789,939 bytes, blake2b `1b5a8b58…e1b42` | **1,417,379 bytes, blake2b `1aa7d55d…c0547`** |
| Fresh chain boots at | spec 105 | **spec 107** |

Verified on an isolated chain before touching production: booted at
`specVersion: 107` with no upgrade applied, authoring from block #3.

> The compressed blob size (`1,417,379`) matching the on-chain `:code` length is
> expected — that is the same compact-compressed artifact the upgrade used.

### 2c. Backups can no longer be unbootable (`infra/deploy/backup-chain.sh`)

The nightly job tarred a **running** RocksDB. Combined with the restore failing to
author, that is an unacceptable rollback asset. The script now:

1. Stops `ceiba-node` before copying (default `STOP_NODE=1`; documented opt-out).
2. Copies and compresses with no lock/WAL exclusions.
3. Restarts the node and **verifies block production resumes** within 180 s,
   logging `OK: node authoring resumed (block N -> M)` — so an unrestorable
   archive is noticed the same night, not at the next restore.
4. Restarts the node from an `EXIT` trap, so a failed backup cannot leave the
   chain down.

**Rollback lesson (record this):** reverting an image tag does **not** undo a
database change. A binary swap's rollback asset is a DB snapshot taken
immediately *before* the swap — nothing else works.

## 3. Final state (all verified live)

| Property | Value |
|---|---|
| Image | `belizechain/ceiba-node:authoring-guard-spec107genesis-20260921` |
| `.env` | `CEIBA_NODE_IMAGE` points at the above; backup at `.env.bak-pre-guard-*` |
| Spec at genesis | **107** (no upgrade applied) |
| Genesis hash | `0x631fb9369f5ddd3e6c2649fd1f425b7a61a55b38c41d626bd8a4cd0d70df7399` |
| Authoring | every 6 s, confirmed |
| NEMO registry | seeded, `isAuthority: true` |
| A1 Nawal signer | recreated — see below |
| Keys | unchanged; no key regeneration at any point |

### A1 registration recreated

Values were read from the **preserved pre-reset chain state** rather than
reconstructed from memory:

| Field | Value |
|---|---|
| Signer | `r1WdCQ9cUTAdtF2foeLY9CmzXLazwkcQqfLCGTEe37NX94NJL` (= `5Gj3p3X5…`) |
| Identity | id 1, `Nawal AI Validator`, owner = signer |
| KYC | L2 — valid **SSN + Passport** attestations (L2 needs exactly those two; biometrics would be L3) |
| Staking | stake 1,000 DALLA, capacity 100, location `Belize City`, complianceScore 100 |
| `validatorCount` | 1 |

Reproduced by `scripts/register-nawal-signer.js` (idempotent; derives the signer
from `NAWAL_SEED` and refuses if it does not match `SIGNER_ADDRESS`; never logs
the key). Run:

```bash
NAWAL_SEED="$(ssh wicked@ceiba 'sudo grep -m1 "^NAWAL_KEYPAIR_URI=" /opt/belizechain/.env | cut -d= -f2-')" \
  NODE_PATH=/path/to/polkadot-js node scripts/register-nawal-signer.js
```

The signer key lives in `/opt/belizechain/.env` as `NAWAL_KEYPAIR_URI` (a
12-word mnemonic). No new key was generated.

**Note:** the pre-reset chain also carried identity id 1 named `Government`
(Alice), which shifted the Nawal identity to id 2. A fresh chain has no
`Government` identity, so the Nawal identity is id 1. Functional behaviour is
identical; recreate the `Government` identity explicitly if it is wanted.

### Federated-learning round (PoUW evidence restored)

`scripts/run_fl_round.py` (nawal-ai) trains a sovereign `NawalTransformer` from
scratch on the Belize corpus with the real Belize tokenizer, then submits the
proof. Measured run:

| Metric | Value |
|---|---|
| Parameters | 53,316,480 — sized to the tokenizer's real 12,137-token vocabulary |
| Steps / tokens | 25 steps, 5,969 tokens |
| Loss | 12.3124 → 10.0127 |
| Duration | 57 s |
| Model hash | `0x6ffe48883ad630841219a540bedd8af5aa98a33b5f554f974802b708fa1f927e` |

On-chain result, all read back from the chain:

| Evidence | Value |
|---|---|
| `ActiveFLTask` | taskId 1, modelHash = the hash above (assigned via sudo with `scripts/assign-fl-task.js`) |
| `ModelSubmissions[signer]` | present — validator `0xce2f6ecf…b902d054`, delta = the model hash, `submitted_at` 222 |
| `Validators[signer]` | `lastFlContribution` 222, `totalContributions` 1, scores computed by the pallet |
| `EpochRewards[0]` | **0.46 DALLA** minted; signer balance rose by exactly 460000000000 |

**CORRECTION to an earlier reading.** `Staking.ModelSubmissions` does not persist
across an epoch: `distribute_rewards` calls `ModelSubmissions::clear(...)` as it
advances the epoch. The empty `ModelSubmissions` seen in the pre-reset backup at
block 39013 was therefore **normal life-cycle, not corruption** — the A1 round had
simply been rewarded already. This round reproduced that sequence exactly
(submission visible, then cleared on the epoch advance), and the durable evidence
that survives is `Validators[signer]` plus `EpochRewards`.

Also note `assign_fl_task` clears `ModelSubmissions` too, so assigning a new task
invalidates the previous one's submissions.

### Not restored

The A1 round's *specific* numbers are gone regardless — `ModelSubmissions` was
already empty before the reset (see the correction above). What is restored is the
evidence that the mechanism works end to end on this chain. The original A1 driver
script no longer exists, so `scripts/run_fl_round.py` is a reconstruction, not a
re-run of the same code; the model is also 53.3M parameters rather than 84.3M,
because it is sized to the tokenizer's real vocabulary.

## 4. Cleanup performed

Removed from Ceiba (14 GB): all `/data/rt-*` sandbox copies, the corrupted and
stalled DBs, and the historic orphan chain directories
(`.corrupted-cf-*`, `.failed-authority-*`, `.pre-alice-sudo-reset-*`,
`.pre-reset*`).

`/data` went from 65 % to 62 % used; 170 GB free. Retained: the live chain
directory, its pre-final-reset predecessor, and the `/data/backups/chain`
archives (339 MB, 4 archives, 7-day retention).

## 5. Follow-ups

1. ~~Re-run one FL round to restore the on-chain PoUW evidence.~~ **DONE** — see above.
2. Consider whether the `Government` identity should exist on the fresh chain.
3. `chain-specs/chain-spec-production.json` carries **no** embedded code
   (`code bytes: none`) and `generated_specs/chainspec-mainnet.json` still embeds
   a **9,335,011-byte** runtime — both need review before mainnet work. Not
   touched here; mainnet scope was explicitly out of this task.
4. The next nightly backup run should be checked for the new
   `OK: node authoring resumed` line, confirming the consistency fix works.
5. ~~Add a tracked generation path for the testnet spec.~~ **DONE** —
   `scripts/generate-testnet-spec.sh` embeds the freshly built runtime, warns when
   the wasm is older than the sources, and with `--verify` boots a throwaway chain
   to confirm the spec comes up at the version the sources declare.

   Both directions are verified: it reports `OK: … is born at spec 107` with the
   current runtime, and fails with `FAILED: the spec boots at spec 105 but the
   sources declare 107` when handed the old spec-105 wasm. That is precisely the
   drift that shipped unnoticed, because `testnet-spec.json` is gitignored and
   nothing else ever looked at it.

   Remaining gap: nothing runs this automatically. A release step or a pre-deploy
   check that runs `generate-testnet-spec.sh --verify` would make the drift
   impossible rather than merely detectable.
6. `RocksDB` inspection tooling (`ldb`, `sst_dump`) is absent from both the node
   image and the Ceiba host. A damaged DB therefore cannot be examined on the
   machine where it broke. Worth adding to the image: the 2026-09-21 corruption
   was undiagnosable partly for this reason.
7. The `.corrupted-cf-*` directory removed in cleanup showed this CF-corruption
   class also occurred on **2026-08-26**, roughly four weeks earlier. Two
   occurrences points at a recurring trigger rather than a one-off, and the
   evidence from both is now gone. Worth treating as a live risk.
