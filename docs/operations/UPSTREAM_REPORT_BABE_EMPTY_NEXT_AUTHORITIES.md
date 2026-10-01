# Upstream report (ready to file): `pallet_babe::enact_epoch_change` accepts an empty *next* authority set

**Status:** prepared, not yet filed. Written to be pasted into a polkadot-sdk issue.
**Prepared:** 2026-09-30
**Affected:** `pallet-babe` (verified against `46.0.0`)

**Still applicable upstream — re-verified 2026-09-30 against five pinned revisions.**
The asymmetry is present in every one of them, including the newest stable branch:

| Revision | Commit | `enact_epoch_change` | guard on current set | guard on next set |
|---|---|---|---|---|
| `master` | `d2e08992098a` | 623 | **yes** (632) | **no** |
| `stable2609` | `3c87d294a1a7` | 623 | **yes** (632) | **no** |
| `stable2606` | `672a7bb6b81c` | 623 | **yes** (632) | **no** |
| `stable2603` | `0b045b7d9dfb` | 622 | **yes** (631) | **no** |
| `stable2512` | `54f11b102d8b` | 617 | **yes** (626) | **no** |
| *(BelizeChain)* | `2e4dd0bc2236` | 622 | **yes** (631) | **no** |

The most recent commit to touch `substrate/frame/babe/src/lib.rs` is `e3865cf72`
(2026-04-02, `ValidateUnsigned` deprecation) and is unrelated. `pallet-session`'s
rotation path is byte-identical to ours apart from an unrelated benchmark helper and a
`set_keys` consumer-count check. Quoted line numbers below give both, as
`<master> / <our rev>`.

## Environment

| Item | Value |
|---|---|
| Repo | `paritytech/polkadot-sdk` |
| Rev verified | `2e4dd0bc22366a5af820492528869a493b5a5208` |
| Also checked | `master`, `stable2609`, `stable2606`, `stable2603`, `stable2512` (see table above) |
| Crate | `pallet-babe 46.0.0` |
| File | `substrate/frame/babe/src/lib.rs` |
| Function | `enact_epoch_change` (line 623 master / 622 our rev) |

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

The asymmetry looks unintentional: the guard covers the set that is *already live*,
while the set that will become live is left unchecked.

## The code

Guard on the current set — line **632** (master) / 631 (our rev):

```rust
if authorities.is_empty() {
        log::warn!(target: LOG_TARGET, "Ignoring empty epoch change.");
        return;
}
```

Unconditional uses of the next set — lines **694**, **707** and **710** (master) /
693, 706 and 709 (our rev):

```rust
NextAuthorities::<T>::put(&next_authorities);          // 694
...
let next_epoch = NextEpochDescriptor {
        authorities: next_authorities.into_inner(),     // 707
        randomness: next_randomness,
};
Self::deposit_consensus(ConsensusLog::NextEpochData(next_epoch));   // 710
```

There is no `next_authorities.is_empty()` check anywhere in the function — confirmed
by re-reading all 101 lines of it (623-723) on both `master` and `stable2609`.

### The documented contract does not forbid it

The doc comment on the function (lines 616-622 on master) says:

> DANGEROUS: Enact an epoch change. Should be done on every block where
> `should_epoch_change` has returned `true`, and the caller is the only caller of
> this function.
> ...
> This doesn't do anything if `authorities` is empty.

That sentence covers the **current** set only. Nothing in the function or its
documentation says an empty *next* set is acceptable. And the one in-tree caller that
supplies both sets itself, `SameAuthoritiesForever` (line 103), clones the current set
into the next — so it structurally cannot reach this. The only way in is
`ExternalTrigger`, which is what every chain wiring `pallet-session` uses.

It is also worth noting that `do_try_state` (added by #11216, 2026-03-12) does bound
both authority lists *upwards* (`decode_len() <= MaxAuthorities`) but never asserts
that either is non-empty — so `try-runtime` will not catch this either.

## Why the empty set is reachable through documented pallet behaviour

`pallet_session::rotate_session` builds `QueuedKeys` by *filtering out* every
validator whose session keys cannot be loaded
(`substrate/frame/session/src/lib.rs`, lines 838-847 on master, 845-848 on our rev):

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

## Related prior art (checked — this is not a duplicate)

Searched before preparing this, and re-searched on 2026-09-30 across all of GitHub (not
just this repo) for `enact_epoch_change`, `NextAuthorities`, `empty epoch change`,
`zero authorities babe`, `babe no authorities`, `chain halt babe authorities`,
`pallet-session empty validators`, plus the archived `paritytech/substrate` tracker and
open PRs. Exactly one adjacent issue exists, and it is a **different path**:

| Issue | State | Why it is not this |
|---|---|---|
| [#5064](https://github.com/paritytech/polkadot-sdk/issues/5064) — *Chain halts after runtime upgrade \| Getting `Ignoring empty epoch change.`* | closed, `COMPLETED`, 0 comments, 2024-07-18 → 2024-07-20 | Theirs is the **guarded current-set** branch: a corrupt `OneSessionHandler` produced an empty *current* set, which logged and returned — and the chain then died from a *different* error (`Import failed: Expected epoch change to happen at …, s286882962`). This report is the **unguarded next-set** branch, which produces no log line at all. |

No PR mentioning `enact_epoch_change` was found, and no issue matched "empty next
authorities" or "enact_epoch_change empty authorities".

Worth noting about #5064: it was closed `COMPLETED` two days after opening with **no
comments**, which reads more like an abandonment than a fix — so it is worth linking
rather than ignoring.

## Suggested fix

Mirror the existing guard onto the next set, so an empty next set can never be
written to storage or announced in a digest.

The minimal form treats it like the empty-current case — refuse the change:

```rust
if next_authorities.is_empty() {
        log::error!(
                target: LOG_TARGET,
                "Refusing to announce an epoch with no authorities."
        );
        return;
}

if authorities.is_empty() {
        log::warn!(target: LOG_TARGET, "Ignoring empty epoch change.");
        return;
}
```

One trade-off is worth flagging, since it is the maintainer's call and not ours: an
early `return` is consistent with the existing guard, but it leaves `EpochIndex`
un-advanced while `CurrentSlot` keeps moving. The alternative is to carry the live set
forward — assign `next_authorities = authorities.clone()` before the writes — which
keeps the epoch advancing and the chain producing blocks. Either way the unclaimable
epoch is never published, which is the part that matters.

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
