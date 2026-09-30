# Empty BABE Authority Set — 6-Day Stall and Re-Genesis (Ceiba, 2026-09-29)

**Status:** Resolved. Chain re-genesised and verified; a session rotation has been
observed on the live chain. This is the **authoritative record for the current
chain** and supersedes the chain identity established in
[CHAIN_RESET_AND_AUTHORING_GUARDS_2026-09-21.md](CHAIN_RESET_AND_AUTHORING_GUARDS_2026-09-21.md).

**Read this first:** the previous chain (`0x631fb936…`) is **dead and remained
dead**. It is not a rollback target and its blocks will never resume. Current
chain: genesis `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef`.

## 1. Summary

Block production stopped at **#28791 on 2026-09-23 16:02:36** and never resumed.
It was found on **2026-09-29** during a routine check-in — roughly six days later,
because the only signal was a zero-block log that nothing alerted on.

The chain did not crash, and nothing was corrupt. **BABE had announced a next
epoch containing zero authorities.** Past the end of the current epoch no slot
was ever claimable again, so no block could be produced, so the epoch could never
advance. It was a total deadlock with no in-place recovery path.

## 2. What the evidence said

All read from the live node, read-only.

| Observation | Value |
|---|---|
| Last block imported | `#28791` at `2026-09-23 16:02:36` |
| Finalized head | `#28789` (frozen) |
| `Babe::EpochIndex` | `1` |
| `Babe::GenesisSlot` | `298334427` |
| `Babe::CurrentSlot` | `298363226` (frozen at the boundary) |
| Epoch 1 window | slots `[298348827, 298363227)` |

Blocks 28790 and 28791 were authored in slots `298363225` and `298363226` — the
**final two slots of epoch 1**. They differ by exactly one slot, which
self-validates the slot decode.

The stall guard fired correctly and repeatedly from `16:13:21` onward, every five
minutes, and told the operator exactly where to look:

```
STALL DETECTED: no block imported for 100 slots (~600s). best #28791, finalized #28789.
Local authoring capability: keys present … Block production has stopped even though
this node holds the keys — collect the node logs and check the BABE slot-claim path
(epoch/slot state) before restarting.
```

Restarting could not help, and did not: the nightly restart at `04:34` on
2026-09-29 came up clean (`authoring preflight … matched local keys 1`), started
the BABE worker, and produced nothing.

### The digest at the point of failure

Block **14393** is the epoch-change block (`EpochStart` block `14393`; its
pre-runtime digest carries `slot 298348827`, exactly epoch 1's start). Its BABE
consensus digest decodes as:

| chain | block | digest bytes | authority count |
|---|---|---|---|
| this (dead) chain | 14393 | 34 | **0** |
| current chain | 1 | 74 | 1 |
| current chain | 301 | 74 | 1 |

The arithmetic is exact: `1 variant + 1 count + 0×40 authorities + 32 randomness
= 34` bytes, against `1 + 1 + 1×40 + 32 = 74` for a single authority. **The epoch
change announced zero authorities.**

> Decoding note: that payload is a SCALE **compact length-prefixed `Vec<u8>`**.
> The first byte is a length prefix, not the variant (`0x88` = mode 0; `0x29` =
> mode 1, two-byte). Run it through a compact decoder before reading the variant,
> or the variant appears to be the nonsense value `0x29`.

### Why the client would not author

Re-running the node with `babe=debug` showed the claim path being entered every
six seconds, forever, and never succeeding:

```
DEBUG babe: Attempting to claim slot 298448879
DEBUG babe: No block for 85652 slots. Applying exponential lenience, total proposing duration: 516000ms
```

`Claimed slot` never appeared — 85,000+ consecutive failures. Lenience was not the
constraint: a 516 s proposing window was offered every slot. The claim itself was
returning `None`.

## 3. Root cause

`BelizeSessionManager::new_session` sourced the next validator set from
`pallet_belize_staking::Validators`. On this chain that map contained one entry,
`0xce2f6ecf…b902d054` — the **Nawal AI signer**, registered by
`scripts/register-nawal-signer.js` — and that account **had never registered
session keys**:

| Storage | Value |
|---|---|
| `Staking::Validators` | `0xce2f6ecf…b902d054` |
| `Session::NextKeys[0xce2f6ecf…]` | **absent** |
| `Session::Validators` | `0xd43593c7…` (dev Alice) — keys present |
| `Session::QueuedKeys` | **0 entries (empty)** |

`pallet_session` builds its queued set by dropping every validator whose key
entry is absent — silently:

```rust
let queued_amalgamated = next_validators.into_iter()
    .filter_map(|a| { let next_keys = T::NextKeys::get(&a)?; ... })   // no log, no event
    .collect::<Vec<_>>();
```

The set was therefore filtered down to **nothing**, and `on_new_session` handed
BABE `authorities = [Alice]`, `next_authorities = []`. BABE then announced the
zero-authority `NextEpochData` digest above.

**Why it looked healthy for a full day.** Epoch 1's authorities came from the
*previous* queued set (Alice, from genesis). Only the *newly computed* queue was
empty, and it was not consumed until the next boundary. There had been exactly
**one** rotation in the chain's life, at block 14393 — the one that emptied it.

### The deadlock is structural

Runtime config is `EpochChangeTrigger = ExternalTrigger` with
`ShouldEndSession = Babe`, and `should_epoch_change` is
`CurrentSlot − current_epoch_start() ≥ EpochDuration`:

- `CurrentSlot` is only written inside `pallet_babe::initialize()`, which runs
  **during block execution**;
- the client only authors inside `[start_slot, end_slot)`;
- the runtime only advances the epoch at `≥ end_slot`.

With a next epoch holding no authorities, `[end_slot, next_epoch_with_authorities)`
is **unreachable**. No restart, resync, or additional peer can cross it — the
client rebuilds its epoch tree from the same on-chain digest.

### SDK asymmetry worth reporting upstream

`pallet_babe::enact_epoch_change` guards an empty **current** set but not an empty
**next** one:

```rust
if authorities.is_empty() {
    log::warn!(target: LOG_TARGET, "Ignoring empty epoch change.");
    return;                                   // current set: safely ignored, recoverable
}
…
NextAuthorities::<T>::put(&next_authorities);  // no emptiness check
Self::deposit_consensus(ConsensusLog::NextEpochData(next_epoch));  // announces zero authorities
```

A chain can therefore be bricked by state that the runtime is willing to
announce. Our fix is at the runtime layer — never construct an empty queued set —
which is the correct place given the SDK behaviour.

## 4. Why there was no rollback

Every available archive post-dated the damage:

- `/data/backups/chain/` held daily snapshots from Sep 23 → Sep 29. The earliest,
  Sep 23 04:42, was taken **after** block 14393 (~Sep 22 16:02), so it already
  contained the zero-authority digest.
- The `/data/chain-backups/*` tarballs predate the Sep 21 re-genesis entirely and
  are a different chain.

This is the second time a "we have backups" assumption failed on this host. **A
backup whose age is not checked against the failure time is not a rollback
asset.** Worth adding to the backup job: record the block range covered, not just
the timestamp.

## 5. The fix

Committed to `belizechain` (`5b75a33`, `4974022`, `509830c`, `f6dc065`, `aeb3ada`).

**Runtime — `BelizeSessionManager::new_session` now filters to validators holding
session keys**, falls back to the current authority set when that filter empties
the list, and logs an error naming excluded accounts. This makes an empty queued
authority set unreachable regardless of how the state arose. **This is the root
fix.**

**Pallet — a `SessionKeyRegistry` provider trait**, used to warn (not block) when
a joining validator holds no session keys. Both validator entry paths
(`join_validators` and `force_join_validator`; the latter was a root path an early
pass missed) report the condition.

**Benchmark safety.** The lookup is relaxed only in `runtime-benchmarks` builds,
whose setup force-joins derived accounts that cannot exist in genesis. The lookup
is always performed first so measured weights still include the storage read.

**Tests.** `session_rotation_never_queues_an_empty_authority_set` rotates a real
session containing a keyless validator and asserts non-empty queued keys and
non-empty `NextAuthorities`. It was confirmed to **fail without the fix**:

```
rotation queued no session keys — BABE would have no authorities
```

Suite: pallet 60 · pallet + benchmarks 68 · runtime 11 (both epoch modes) ·
`cargo fmt --check` · `clippy -D warnings`.

> ### Session-key precondition relaxed (2026-09-29, after this incident)
> An initial version of the fix made the session-key check a hard `ensure!` on
> both join paths. That **rejected the PoUW-only validator path**: the A1 Nawal
> signer is a legitimate PoUW participant that joins for rewards and never
> authors, and `scripts/register-nawal-signer.js` registers it through
> `staking.joinValidators` without session keys — so the call failed with
> `SessionKeysNotRegistered`.
>
> Resolved by **relaxing the check to a `log::warn!`** on both paths and deleting
> the now-unreachable `SessionKeysNotRegistered` error (it was the last variant, so
> no other variant index shifted). The guard was always stricter than the failure
> required: what actually prevents the empty-authority deadlock is
> `BelizeSessionManager::new_session`, which is unchanged and still covered by
> `session_rotation_never_queues_an_empty_authority_set`.
>
> Behaviour now: a keyless account **may** join and earn PoUW rewards, receives a
> warning at join time, and is excluded from the BABE authority set at the next
> rotation with an error-level log naming the count. The A1 restore procedure in
> `CHAIN_RESET_AND_AUTHORING_GUARDS_2026-09-21.md` §A1 runs again.
>
> **Not yet deployed.** The live chain still runs the stricter build; picking this
> up requires a runtime upgrade (or another re-genesis).

## 6. Recovery

Re-genesis, chosen deliberately over an in-place client patch (which would have
left the chain on a `--wasm-runtime-overrides` runtime indefinitely and would
still have re-stuck at the next boundary).

1. Rebuilt the runtime and node with the fix. The shipped default still carries
   `BabeEpochDuration = 14_400`; a **separate opt-in `testnet-fast-epoch` feature**
   sets 300 slots (~30 min) so a rotation can actually be observed. Mainnet
   consensus timing is untouched by the testnet build.
2. Regenerated the spec with `scripts/generate-testnet-spec.sh --verify` — boot
   check reported `OK: … is born at spec 107`. New runtime blob 1,420,530 bytes,
   blake2b `cd7d2112…`.
3. Built the image on Ceiba (`Dockerfile.runtime` copies a pre-built binary; there
   is no toolchain on the host) and verified the `sha256` of the shipped binary
   matched the local build.
4. Backed up `.env`, the old spec, and **the stalled database**
   (`/data/regenesis-backup-20260929-115324/`, 105 MB) — kept as forensic
   evidence, since it is the only remaining copy of the zero-authority digest.
5. Wiped **only** `chains/belizechain_testnet/db` and `network/`. The **keystore
   was preserved** and still matches the genesis authority, so the new chain
   authors from block #1 without touching keys.
6. Opened the `CEIBA_OPERATIONS_RUNBOOK.md` Ops Log entry for the new chain.

**Procedure is scripted:** `scripts/regenesis-ceiba-emptyauth.sh` (dry run unless
`--assume-yes`; refuses to run if the keystore is missing, and re-asserts it
immediately before the wipe).

## 7. Verification

Two independent chains, because the failure only manifests at a boundary and
"it has not died yet" is not evidence.

**Ceiba (live):**

```
*** EPOCH ROTATION OBSERVED ***   epoch 0 -> 1   block #302
new epoch slots [298449841, 298450141)  duration 300  authorities: 1
```

Authoring continued through the boundary (`#303`→`#308` at 6 s), finality
advanced, and the guard counts stayed at **0** `STALL DETECTED`, **0** `Ignoring
empty epoch change`, **0** `Epoch(s) skipped`.

**Throwaway chain (same spec, independent base path):** `👶 New epoch 1 launching
at block slot 298449823`, authorities 1, no stall-guard fires.

The decisive comparison is the digest table in §2: the exact measurement that
diagnosed the outage now reports **1** authority instead of **0**.

Tooling added: `scripts/verify_epoch_config.py` (asserts a populated authority set
and the expected duration — worth running on every boot) and
`scripts/watch_epoch_rotation.py` (blocks until an epoch advances, fails fast on a
stall).

## 8. Side effects found during recovery

- **`ceiba-nawal` went unhealthy and did not recover on its own.** Its own report
  was accurate — `blockchain_connected: false`,
  `required_dependency_failures: ["blockchain_unavailable"]` — and the network was
  fine (`tcp connect OK`, HTTP 405 from the container). But it had logged *nothing*
  but health checks for six days: **it latches the failure and never retries its
  RPC connection.** A restart cleared it immediately. Any chain outage will
  silently strand this service until someone restarts it.
  **Fixed — see §10 item 2.**
- **`Dockerfile.runtime` had lost its `chmod +x`.** The image built cleanly and
  every start failed with `exec: "belizechain-node": executable file not found in
  $PATH` — a permissions problem wearing a "missing file" costume, because `COPY`
  preserves the source mode and a binary shipped by `scp` arrives without the
  execute bit. Restored, and verified by running `--version` inside the image.
- **`ceiba-explorer` needed nothing.** It is `jacogr/polkadot-js-apps` —
  Polkadot.js Apps, a static client-side UI. It holds no chain state and connects
  from the browser, so there was nothing to resync.

## 9. Follow-ups

1. **Decide the `join_validators` guard question** (§5 open item) — it currently
   blocks PoUW-only validator registration, including the A1 Nawal signer.
   **Resolved and deployed — see §10 item 1.**
2. **Make dependent services retry.** Nawal should reconnect to the chain with
   backoff instead of latching a failed startup state. Highest-value item here.
   **Resolved — see §10 item 2.**
3. **Alert on zero block production.** The stall guard logs a loud `ERROR`, and
   nothing consumes it. A six-day outage should page someone on the first
   occurrence, not be found by chance.
4. **Record block ranges in chain backups** so "age vs. failure time" is checkable.
5. **Report the `enact_epoch_change` asymmetry upstream** (§3).
6. Consider a pre-deploy check that runs `generate-testnet-spec.sh --verify`
   automatically — the gap flagged in the 2026-09-21 doc is still open.

## 10. Follow-up completion (2026-09-29, same day)

### 1. Session-key precondition relaxed — shipped as runtime **108**

The `ensure!` added in §5 was stricter than the failure required, and it broke the
PoUW-only path: `scripts/register-nawal-signer.js` joins through
`staking.joinValidators` **without** session keys, so registration failed with
`SessionKeysNotRegistered`. What actually prevents the empty-authority deadlock is
`BelizeSessionManager::new_session`, which was already correct and is untouched.

Change: the check becomes a `log::warn!` on both join paths, and the
`SessionKeysNotRegistered` error variant is deleted (it was the **last** variant,
so no other error index moved). A keyless validator now registers, warns at join
time, and is excluded from the authority set at the next rotation with an
`ERROR`-level log naming the count.

Deployed by **hot runtime upgrade, no node restart**:

```
sudo.sudo(System::set_code(1421085 bytes))   extrinsic 0xe47e3b10…8448f
included       : SUCCESS
spec_version   : 107 -> 108
on-chain :code : 1421085 bytes  blake2b 8ad7ea9d18a02e7b5150cf68e455a68c06f84fa6a18c6e4db7c53befcaa45c0d
```

Verified independently after the upgrade: `Babe::Authorities` and
`Babe::NextAuthorities` both hold 1 authority, `Session::QueuedKeys` is non-empty,
blocks continue at exactly 6 s, `epoch_length` is still 300, and the raw metadata
no longer contains `SessionKeysNotRegistered` (its neighbour
`DomainContributionCapExceeded` is still present, so the absence is meaningful).
`ceiba-node` restarted **0** times across the upgrade.

Rollback artifact: the previous runtime is saved as
`/data/upgrade-108/onchain-code-107.wasm` (1,420,530 bytes, blake2b `cd7d2112…`),
verified byte-identical to what the chain was running before the change.

Tooling added: `scripts/upgrade-runtime.py` — dry run by default, refuses to sign
unless the local key matches the on-chain sudo key, and re-reads
`spec_version` plus `:code` afterwards to confirm the upgrade actually applied.

> **Note on SS58.** The chain sets its own `SS58Prefix` (**1981**), so the same
> account renders as `r1Wm6WgK…` on-chain and `5GrwvaEF…` under the default
> prefix 42. The first dry run compared *address strings* and reported a false
> "not the sudo key" failure; it now compares raw public keys. Verify accounts by
> their 32 bytes, not their rendered address.

### 2. Nawal reconnects without inbound traffic

The real mechanism was narrower than §8 first implied. Log evidence contradicts
the "latched a failed startup state" reading: **every** startup connected
successfully, including a restart on 2026-09-24 *inside* the outage — there is no
`Failed to connect` or `degraded mode` line anywhere in the container's life.
Health responses form only seven contiguous runs, ending in **7,785 consecutive
503s** cleared by a manual restart.

What actually happened is two separate facts, and the second is the important one:

1. **A frozen chain answers RPC fine.** While the node was stalled, health returned
   **12,588 × 200**; `System.Number` still reads on a node that produces no blocks.
   Health is therefore no evidence of block production.
2. **The socket later dropped, and nothing drove reconnection.** The connector's
   `is_connected` flag stayed stale-`True`, the liveness probe went false, every
   chain-backed endpoint answered 503 — so no request ever reached
   `_ensure_connected()`, which is only called from those same endpoints. Recovery
   depended on the very traffic the failure suppressed.

Two fixes (`nawal-ai`): a failed initial connect no longer discards the connector
(it was set to `None`, leaving nothing to retry with), and a connection supervisor
drives the existing reconnect path on a timer, decoupling recovery from request
traffic. `connect()` now builds its client in a worker thread so an unreachable
node cannot block the event loop while the supervisor polls.

Proven against the live node with a real outage and a real reconnect: the
connector is retained, the probe correctly reports the dead socket, and the
supervisor restores the connection in **4.0 s with zero API traffic**.

