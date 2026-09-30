# Upstream report (ready to file): `pallet_babe::enact_epoch_change` accepts an empty *next* authority set

**Status:** prepared, not yet filed. Written to be pasted into a polkadot-sdk issue.
**Prepared:** 2026-09-30
**Affected:** `pallet-babe` (verified against `46.0.0`)

## Environment

| Item | Value |
|---|---|
| Repo | `paritytech/polkadot-sdk` |
| Rev | `2e4dd0bc22366a5af820492528869a493b5a5208` |
| Crate | `pallet-babe 46.0.0` |
| File | `substrate/frame/babe/src/lib.rs` |
| Function | `enact_epoch_change` (line 622) |

## Summary

`enact_epoch_change` refuses to apply an epoch change whose **current** authority set
is empty, but applies one whose **next** authority set is empty. The second case
writes an empty set to `NextAuthorities` and announces it in a
`ConsensusLog::NextEpochData` digest. Once that block is final, the chain has
published "the next epoch has no authorities" and no slot in that epoch is
claimable — so no block can be produced, so the epoch can never advance, and no
further epoch change can ever be enacted. **The chain cannot recover, including
across a restart**, because every client rebuilds its epoch tree from the same
on-chain digest. Recovery requires a re-genesis or a `--wasm-runtime-overrides`
patch.

The asymmetry looks unintentional: the guard exists, is commented, and covers the
set that is *already live* while leaving the set that will become live unguarded.

## The code

Guard on the current set — line 631:

```rust
if authorities.is_empty() {
        log::warn!(target: LOG_TARGET, "Ignoring empty epoch change.");
        return;
}
```

Unconditional writes of the next set — lines 693, 706 and 709:

```rust
NextAuthorities::<T>::put(&next_authorities);          // 693
...
let next_epoch = NextEpochDescriptor {
        authorities: next_authorities.into_inner(),     // 706
        randomness: next_randomness,
};
Self::deposit_consensus(ConsensusLog::NextEpochData(next_epoch));   // 709
```

There is no `next_authorities.is_empty()` check anywhere between the guard at 631
and the write at 693.

## Why the empty set is reachable without operator error

`pallet_session::rotate_session` builds `QueuedKeys` by *filtering out* every
validator whose session keys cannot be loaded
(`substrate/frame/session/src/lib.rs`, ~line 848):

```rust
let queued_amalgamated =
        next_validators
                .into_iter()
                .filter_map(|a| {
                        let k = Self::load_keys(&a).or_else(|| {
                                log!(warn, "failed to load session key for {:?}, skipping for next session, maybe you need to set session keys for them?", a);
                                None
                        })?;
                        check_next_changed(&k);
                        Some((a, k))
                })
                .collect::<Vec<_>>();
```

To its credit this path **does** warn, once per skipped validator — so the
condition is not entirely silent, and a runtime that watches its logs has a
chance to catch it a full epoch before the boundary. (This is worth stating
plainly: it weakens the "no warning at all" framing, and the report is stronger
for saying so.)

But a warning is not a guard. `filter_map` returns an empty `Vec` when every
validator is keyless, that empty slice is passed to
`on_new_session(changed, &session_keys, &queued_amalgamated)` → `enact_epoch_change`,
and the change is applied. A runtime can therefore hand BABE a next authority set
that is faithfully derived ("no validator can author") yet fatal when applied.

## Observed impact (live, not theoretical)

On 2026-09-23 a Substrate-based chain (BelizeChain testnet) with a single
validator in `Staking::Validators` that had **no session keys** stopped producing
blocks for **six days**:

| Evidence | Value |
|---|---|
| Last block | `#28791`, 2026-09-23 16:02:36 UTC |
| Epoch-change block | `#14393` |
| `Consensus` digest on that block | 34 bytes — `NextEpochData` with compact authority count `0x00` (**zero**) |
| `Babe::EpochIndex` | `1` (never advanced) |
| `Babe::CurrentSlot` | frozen for the entire outage |
| `Session::QueuedKeys` | **0 entries** |
| `Session::Validators` | 1 (a healthy, keyed validator) |
| Node logs | `Attempting to claim slot <n>` every 6 s, never `Claimed slot` |

The node stayed `Up`, healthy and RPC-responsive throughout; `system_health`
answered normally and the head simply stopped moving. Critically, **the halt is
not self-reporting**: the client keeps asking to claim slots that can never be
claimed, and nothing escalates. The one warning that does exist fires at the
boundary, hours or days before anyone notices the chain has stopped, and is not
distinguished from routine noise by any severity or metric.

For contrast, before the change, the same digest carried **1** authority
(74 bytes). The digest length is the tell: `34` bytes = zero authorities.

## Reproduction sketch

1. A runtime whose `SessionManager::new_session` can return validators that hold
   no session keys (e.g. any runtime that surfaces a staking/registry set directly).
2. Put one keyless account in that set at an epoch boundary.
3. `QueuedKeys` becomes empty; `enact_epoch_change(authorities=[A], next=[])`.
4. Observe: the epoch-change block carries a `NextEpochData` digest with zero
   authorities; no slot in the following epoch is claimable; the chain halts
   permanently.

Step 4 happens with no `pallet-babe` log line, no panic, and no error. The only
signal is the per-validator `warn` from `pallet_session` at step 3.

## Suggested fix

Mirror the existing guard onto the next set. The conservative form is to treat an
empty next set the same way as an empty current set — ignore the change rather than
publish an unclaimable epoch:

```rust
if next_authorities.is_empty() {
        log::error!(
                target: LOG_TARGET,
                "Refusing to announce an epoch with no authorities; \
                 retaining the current authority set."
        );
        return;
}

if authorities.is_empty() {
        log::warn!(target: LOG_TARGET, "Ignoring empty epoch change.");
        return;
}
```

An alternative is to keep announcing the epoch but fall back to the current
authorities for `NextAuthorities`, which preserves liveness while a runtime bug is
fixed. Either way, the important property is that a chain cannot be driven into a
permanently unclaimable epoch by a session-boundary edge case.

If the current behaviour is deliberate — i.e. "an empty next set is the runtime's
explicit instruction to halt" — then it is worth documenting in
`enact_epoch_change`, because as written it is indistinguishable from an oversight
and the failure it produces is unrecoverable, silent, and takes days to diagnose.

## Runtime-side mitigation already applied (for reference)

The affected chain now filters keyless accounts in its own
`SessionManager::new_session`, so an empty queued set is unreachable there. That
works, but it means every runtime must independently know about a failure mode that
`pallet-babe` could refuse on its own behalf — the defensive check belongs upstream.
