# Storage Commitment Registry — Design

**Status:** IMPLEMENTED (uncommitted) 2026-10-12. See §15 for the implementation
record, including two defects found while building it.
**Author:** audit session 2026-10-12
**Supersedes:** the uncommitted "PoUW item C (anchor gate + minted `ProofReward`)" and the
domain-separation change to `derive_merkle_root`. Both were reverted before this was built.
**Related:** `POUW_REWARD_VERIFICATION_GAP.md`, `pallets/storage-proof`

---

## 1. Why this exists

`pallet_storage_proof` verifies Merkle proofs correctly. Two defects made paying for those
proofs impossible:

1. **Reads 0 and 1 of `submit_storage_proof` were circular.** `content_id` is an extrinsic
   parameter *and* the proof comes from the same caller, so the pallet could only establish
   that the caller built a self-consistent tree. Any account could fabricate a leaf plus
   siblings, compute the root off-chain, pass it as `content_id`, and be "verified" for
   arbitrary bytes.
2. **Nothing funds the storage.** With no payer, any reward must be *minted* — and minting
   against a caller-declared root is a free-money exploit.

Attempting to fix it by anchoring the root in LandLedger failed, and the reason is worth
recording: **pakit's content roots do not correspond to anything LandLedger commits.**
Pakit derives `content_id = derive_merkle_root(leaf = BLAKE2(block content), siblings = parent
hashes)` (`pakit-storage/core/merkle_storage_proofs.py`, `build_proof_from_blocks`) and
submits it via `storageProof.submit_storage_proof`
(`pakit-storage/blockchain/storage_proof_connector.py`, `call_function="submit_storage_proof"`).
LandLedger anchors `blake2_256(property_id ++ title_number)` — a different value space
entirely. Wiring the anchor to LandLedger would have rejected **every real pakit proof**.

No pallet commits Pakit DAG roots; that is the missing piece. This document designs it.

## 2. The model

**Storage is paid for by whoever wants the data stored.** A client locks their own funds
against a content root; a provider proves storage and is paid from that escrow.

This resolves the anchor problem without any cryptographic argument. The anchor becomes
*"a client funded this"* — a fact the chain can check cheaply, and one that the submitter
cannot manufacture for profit. See §5.

**Nothing is ever minted.** `ProofReward`, `MaxSupply` and `deposit_creating` are deleted.

**One provider per deal.** The first account to submit an accepted proof is recorded as the
deal's provider and is the only account thereafter paid from that escrow. This keeps the
tranche arithmetic unambiguous and stops the client's funds being raced away from a provider
that has already committed to retention.

## 3. Data structures

```rust
pub struct StorageCommitment<AccountId, Balance, BlockNumber> {
    /// Who funded the escrow. The only account that can be refunded or challenge.
    pub client: AccountId,
    /// Escrow originally reserved. `total - released` is what remains owed.
    pub total: Balance,
    /// Already paid out to the provider.
    pub released: Balance,
    /// Paid per accepted proof: `total / expected_proofs`. Stored because an
    /// exact `u32 → Balance` conversion is not available generically.
    pub tranche: Balance,
    /// Proofs the term expects: `duration / proof_interval`.
    pub expected_proofs: u32,
    /// Proofs paid so far. The commitment closes once this reaches
    /// `expected_proofs`.
    pub proofs_paid: u32,
    /// Set by the first accepted proof and fixed thereafter.
    pub provider: Option<AccountId>,
    /// Minimum spacing between paid proofs.
    pub proof_interval: BlockNumber,
    /// Block of the last paid proof (the commit block before the first).
    pub last_proof_at: BlockNumber,
    /// Term end. Inclusive for proving, exclusive for refunds (§4).
    pub expires_at: BlockNumber,
    /// Set while a client challenge is open; cleared by an accepted proof.
    pub challenged_at: Option<BlockNumber>,
}
```

Maps to the content id, so the content id is **not** duplicated as a field — a small
deviation from the draft, which wasted 32 bytes per row.

Storage (all new, all empty at upgrade — see §10):

```rust
#[pallet::storage] pub type Commitments<T> =
    StorageMap<_, Blake2_128Concat, ContentId, StorageCommitment<...>>;

/// Per-client cap, so one account cannot wedge unbounded state.
#[pallet::storage] pub type ClientCommitmentCount<T> =
    StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;
```

`StorageProofs` (the existing proof record) is unchanged.

## 4. Extrinsics

Call indices 0 (`submit_storage_proof`) and 1 (`revoke_proof`) are live and must not move.
New calls are appended:

| Index | Call | Origin | Effect |
|---|---|---|---|
| 2 | `commit_storage(content_id, escrow, proof_interval, duration)` | signed | Reserves `escrow` from the client; creates the commitment. **The only writer of anchors.** |
| 3 | `refund_expired(content_id)` | signed | After the term, unreserves `total - released` back to `client`. Permissionless — the destination is fixed, so griefing is pointless. |
| 4 | `challenge_provider(content_id)` | signed | **Client only.** Opens a challenge; the provider must answer with an accepted proof inside `ChallengeWindow`. |
| 5 | `claim_challenge_failed(content_id)` | signed | **Client only, earned exit.** After an unanswered challenge, reclaims `total - released`. |
| 6 | `release_provider(content_id)` | signed | **Client only, discretionary exit.** Ends the deal at any time and reclaims `total - released`. |

`submit_storage_proof` (index 0) gains a commitment precondition and a payout step,
in this order:

```
require a commitment exists                    → Error::NoCommitment
require now <= expires_at                      → Error::CommitmentExpired
if no challenge open, require the interval     → Error::ProofTooSoon
claim the provider slot, or require it         → Error::NotDealProvider
pay one tranche from the client's escrow        (repatriate_reserved)
reserve StorageDeposit — only if no record yet
write the proof record, clear any challenge
settle (remove) if proofs_paid == expected_proofs, else persist
```

Two boundary rules matter and are easy to get wrong:

- **The term is inclusive for proving.** `expected_proofs × proof_interval` can land
  exactly on `expires_at`, so `submit_storage_proof` accepts `now <= expires_at` while
  `refund_expired` requires `now > expires_at`. The two can never both fire, and the
  final tranche is not lost to an off-by-one.
- **`challenge_provider` also allows `now <= expires_at`**, and `claim_challenge_failed`
  requires `now >= challenged_at + ChallengeWindow` — so a challenge raised near expiry
  can still be claimed after the term ends.

Call index 1 (`revoke_proof`) is unchanged.

## 5. Safety argument

The claim: **participating cannot increase an account's total balance, except by receiving
a legitimate client's escrow.**

Every balance movement in this design is one of three, and all three preserve total issuance:

| Operation | Movement | Primitive |
|---|---|---|
| Commit | client free → client reserved | `Currency::reserve` |
| Release | client reserved → provider free | `Currency::repatriate_reserved` |
| Refund | client reserved → client free | `Currency::unreserve` |

There is **no `deposit_creating` and no `slash`**, so `total_issuance` is invariant.

*Self-dealing.* An actor controlling both the client and provider accounts moves its own
money from its own reserved balance to its own free balance. Net result: `-(fees)`. It is
unprofitable for any positive fee, and no amount of correctly-formatted evidence improves it.

*Consequence for the anchor requirement.* Because faking the anchor yields nothing, **the
anchor does not need to be unfakeable.** It only needs to be *cheap to check* and *honest
about who is paying*. This is precisely why the domain-separation change of §11 is not
needed under this design: there is no mint to farm, so there is no collision worth closing.

*Consequence for verification quality.* An incorrectly accepted proof moves the *client's own*
escrow to a provider. The party harmed is the party that funded it, never the chain or
unrelated accounts. Verification quality is therefore a client-protection concern, not a
consensus-safety one — a materially weaker and more honest requirement than the minted model
demanded.

## 6. Retention: what was done, and what is still not proven

A Merkle path proves the provider **could produce a valid path over the committed content
at proof time**. It does **not** prove the provider still holds the bytes, and a provider
can fetch the block, compute the proof, and discard it.

Full retention proofs (Filecoin/Arweave class) need chunk-indexed challenges with a
deadline tight enough that a network fetch cannot beat it, or erasure coding with
spot-checks. That is a research-grade subsystem and it is **not** implemented.

### What was implemented instead: makes non-retention detectable and costly

The chain cannot police retention, but the party whose money is at stake can. So the
client gets a challenge:

1. `challenge_provider` — the client (and only the client) flags the deal.
2. The provider must answer with an **accepted proof** inside `ChallengeWindow`. Answering
   is just a normal proof submission, and it also earns that proof's tranche.
3. If no proof arrives, `claim_challenge_failed` returns the unspent escrow to the client
   and closes the deal.

The challenge path is deliberately **earned, not chosen**: the chain verifies that the window
elapsed with no accepted proof, so a client cannot rug a provider that is still answering
challenges.

**`release_provider` also exists** (call 6) — a discretionary early exit, added on explicit
product decision. It recovers the unspent escrow at any time, whether or not a provider has
claimed the deal.

> ⚠️ **This creates a real asymmetry, stated plainly.** A provider keeps everything already
> paid for accepted proofs, but has **no claim on the remaining tranches and no recourse** if
> the client walks away. A provider's expected future income is therefore *not* guaranteed.
> That is a deliberate trade — a client must not be forced to fund storage it no longer needs
> — but it means the escrow schedule binds only one side. If provider-side certainty is wanted
> later, the options are a notice period, pro-rata settlement for work already scheduled, or
> bonding.

### Honest limits of this remedy

- **Detection, not proof.** The provider may genuinely have the data; the client simply
  learns whether it can produce a proof on demand. A false alarm costs the provider the
  deal's remaining escrow, which is why the window is generous.
- **A long window weakens it.** If `ChallengeWindow` exceeds the time needed to fetch the
  data, a provider that discarded it can still answer. The constant is set to ~10 minutes
  at 6s blocks as a placeholder; a real value needs a retrieval-time budget.
- **No penalty beyond forfeiting escrow.** Nothing is bonded, so there is no slash. The
  provider's cost of non-retention is exactly the escrow it fails to collect.
- **Retention across the whole term is still unproven.** Passing a challenge at block *t*
  says nothing about block *t+1*.

This does not close §6's gap. It buys the property that the client's funds move only on
evidence the client itself can demand.

## 7. Config additions

Removed: `ProofReward`, `MaxSupply` (both only existed for the minted model).

Added:

```rust
#[pallet::constant] type MinEscrow: Get<BalanceOf<Self>>;              // floor per commitment
#[pallet::constant] type MinProofInterval: Get<BlockNumberFor<Self>>;  // global spam floor
#[pallet::constant] type MaxCommitmentDuration: Get<BlockNumberFor<Self>>;
#[pallet::constant] type MaxOpenCommitments: Get<u32>;                 // per client
#[pallet::constant] type ChallengeWindow: Get<BlockNumberFor<Self>>;   // §6
```

Unchanged: `Currency: ReservableCurrency`, `StorageDeposit`, `RevocationOrigin`, `WeightInfo`.

**No `ContentAnchor` trait was introduced.** The draft kept one and re-implemented it
against `Commitments`; since the registry lives in the same pallet, the trait was a layer
of indirection with nothing behind it, so `submit_storage_proof` reads `Commitments`
directly. This also removes all LandLedger coupling and the mock anchor.

`StorageDeposit` becomes partly redundant once the commitment itself carries economic weight.
It is **kept unchanged** to minimise churn and preserve the existing anti-state-growth
behaviour; that redundancy is acknowledged rather than silently removed.

## 8. Events and errors

Events: `StorageCommitted { content_id, client, escrow, expires_at }`,
`CommitmentRefunded { content_id, client, amount }`,
`ProviderClaimed { content_id, provider }`,
`TrancheReleased { content_id, provider, amount, released_total }`,
`CommitmentPaidOut { content_id, provider, released }`,
`CommitmentReleased { content_id, client, provider, refunded }`,
`ProviderChallenged { content_id, deadline }`,
`ChallengeAnswered { content_id, provider }`,
`ChallengeDefaulted { content_id, provider, refunded }`.
Existing `StorageProofAccepted` / `StorageProofRevoked` unchanged.

Errors added: `NoCommitment`, `CommitmentExpired`, `CommitmentNotExpired`,
`CommitmentExists`, `ProofTooSoon`, `NotDealProvider`, `EscrowBelowMinimum`,
`InvalidDuration`, `IntervalBelowMinimum`, `TooManyCommitments`, `NotCommitmentClient`,
`NoProvider`, `ChallengeAlreadyOpen`, `NoOpenChallenge`, `ChallengeStillOpen`,
`EscrowExhausted`, `EscrowUnavailable`, `InsufficientEscrow`.

**`ContentAlreadyProven` was removed** — see §15.1 for why it was a live bug under
repeated proofs.

## 9. Weights and benchmarking

`scripts/bench_smoke.sh` benchmarks `pallet_storage_proof` against `--chain dev` on every PR,
and the workflow comment explicitly rejects mock-only coverage. A fresh dev genesis has no
commitments, so `submit_storage_proof` would fail its own precondition.

**Resolution: the benchmark seeds a real commitment by calling `commit_storage` from a funded
account.** No benchmark-only bypass exists — no forgery hook, no feature-gated writer — so the
benchmark exercises exactly the path a real client takes. Because the anchor is a funded
commitment rather than a hash, there is nothing to fake: the benchmark simply funds one.

All seven extrinsics are benchmarked. `refund_expired` advances past the term; the challenge
benchmarks claim a provider slot first (via a real `submit_storage_proof`), then advance
`ChallengeWindow`.

**`weights.rs` is regenerated.** The five hand-written interim estimates and their
`NOT BENCHMARKED` banner are gone, replaced by measured values, so `weights.rs` now carries
the hostname/CPU/DATE header every other pallet's generated file has.

The interim figures were conservative, and the earlier "nothing is under-charged" assurance was
**correct**. Measured against the benchmarked values:

| Extrinsic | Interim | Measured | Error |
|---|---|---|---|
| `submit_storage_proof` | 45.6M ps | 45.9M ps | exact within 1 % |
| `revoke_proof` | 44.1M ps | 24.5M ps | over-charged 1.8x |
| `commit_storage` | 68.0M ps | 28.8M ps | over-charged 2.4x |
| `refund_expired` | 52.0M ps | 29.8M ps | over-charged 1.7x |
| `challenge_provider` | 48.0M ps | 14.6M ps | over-charged 3.3x |
| `claim_challenge_failed` | 54.0M ps | 29.4M ps | over-charged 1.8x |
| `release_provider` | 54.0M ps | 29.4M ps | over-charged 1.8x |

`submit_storage_proof`'s reads/writes did legitimately go 1/1 → 2/2: the commitment requirement
added a `Commitments` read, so the storage component genuinely grew. That part was a real
correction to a stale weight.

⚠️ **Measurement provenance matters here.** An intermediate regeneration on 2026-10-13
(`d9403e8`) was taken while the benchmark host was on battery power, where it throttles to
roughly a third of its clocks. Those figures came out **1.4-1.8x inflated**, and one of them —
`submit_storage_proof` at 82.7M ps — went live in runtime **spec 115**. It is corrected to the
45.9M ps figure above in spec 116. Every number in the table is from the AC-powered run, and
`scripts/bench_weights.sh` now refuses to benchmark off AC power for exactly this reason.

## 10. Rollout

- **No storage migration.** Every new item is additive and starts empty; no existing item
  changes shape or key. `STORAGE_VERSION` may stay at `0`. No `on_runtime_upgrade` hook.
- **`transaction_version` should bump** `1 → 2` (`runtime/src/lib.rs:458`). Adding call
  indices to a live pallet is additive and safe, but the version advertises that the
  extrinsics set changed, per Substrate convention.
- **`spec_version` bumps** `114 → 115` (`runtime/src/lib.rs:455`).
- Deploy via the established path: build the runtime **with `--features testnet-fast-epoch`**,
  then hot `sudo.sudo(System::set_code)`. A blob built without that feature silently changes
  `BabeEpochDuration` from 300 to 14,400 and permanently halts block import — this is how
  Chain C died on 2026-10-04. Run `scripts/upgrade-runtime.py`'s dry run first and pass
  `--rpc ws://100.119.97.38:9944` explicitly (its default endpoint is a decommissioned host).

**Pre-upgrade check.** Confirm whether any rows already exist in `StorageProofs`. If real
pakit proofs have been accepted on the live chain, those records predate the commitment
requirement and `revoke_proof` must keep working for them — which it does, since `revoke_proof`
is unchanged and reads only `StorageProofs`. Record the count in the upgrade note either way.

**Recorded 2026-10-13 (`StorageProofs` rows = 0).** Checked over RPC immediately before the
upgrade: `state_getKeysPaged` on the `StorageProof::StorageProofs` prefix returned no keys, so
there are no pre-commitment-acceptance rows and the back-compatibility question is moot in
practice. The chain was at `spec_version` 114, `transaction_version` 1.

**Deployed 2026-10-13 (`spec_version 115`).** Hot `sudo.sudo(System::set_code)` from 114; no
node restart. Blob 1,489,480 B, blake2b-256
`0ba718768be4dafea9088d14d5f4c652d712a741f3268cc0e4b14e99111945e3`, built with
`--features testnet-fast-epoch`. The blob was booted in a throwaway chain before submission and
`BabeApi_current_epoch` read directly (duration 300), and again after: the 318 → 319 epoch
boundary crossed with `duration 300` and 2 authorities, so the upgrade did not reproduce the
empty-authority or wrong-epoch-duration stalls. `Commitments` and `ClientCommitmentCount` are
present in live metadata. Rollback blob (the 114 `:code`) is preserved on the host at
`/data/upgrade-115/onchain-code-114.wasm`.

Note this upgrade also carried regenerated `staking` and `storage-proof` weights (see
`pallets/*/src/weights.rs`); weights compile into the runtime, so they ride the same blob.

## 11. What this superseded

All of the following was **reverted before implementation**, so the diff that ships contains
only the registry:

- **The LandLedger anchor gate** in `pallets/storage-proof/src/lib.rs` plus its mock, tests,
  runtime wiring and benchmark hook. It could never have worked: pakit's content roots are
  DAG roots, and LandLedger commits land-title hashes — disjoint value spaces, so every real
  pakit proof would have been rejected.
- **The minted `ProofReward` / `MaxSupply` path** — never reintroduced.
- **The `derive_merkle_root` domain separator (`MERKLE_NODE_DOMAIN`)** — not needed. It was to
  close a collision between a single-step fold over a caller-chosen 64-byte input and
  LandLedger's 64-byte `blake2_256(property_id ++ title_number)` preimage. That collision was
  empirically confirmed (`untagged fold == anchored hash: True`), but with no mint there is
  nothing to farm, and pakit must mirror any format change byte-for-byte.

`derive_merkle_root` is therefore **unchanged**, and so is the byte-exact parity asserted
between `pallets/storage-proof` and `pakit-storage/core/merkle_storage_proofs.py`.

## 12. Cross-repo impact

**None.** `commit_storage` is a *client* action — it belongs in a wallet or UI, not in the
pakit server. Pakit keeps submitting proofs exactly as today; the only new failure mode it can
observe is `NoCommitment` for content nobody funded, which is correct behaviour.

`derive_merkle_root` is unchanged, so the byte-exact parity asserted between
`pallets/storage-proof` and `pakit-storage/core/merkle_storage_proofs.py` continues to hold.

## 13. Non-goals

- **Durable-retention proofs** (§6). Requires period challenges or erasure coding; not designed here.
- **Multiple providers per deal / replication factor.** `DAG_REPLICATION_FACTOR=3` in pakit is
  not modelled; one deal currently pays one provider.
- **A storage marketplace** (provider bidding, pricing, reputation). Out of scope.
- **Slashing a non-retaining provider beyond withholding payment.** No stake is bonded here.
- **Replacing `StorageDeposit`** (§7).

### Resolved — product decisions taken

1. **Tranche count** — *option 1, as built*: `expected_proofs = duration / proof_interval`,
   `proof_interval` caller-supplied but floored by `MinProofInterval`, and the **final** proof
   sweeps `total - released` so integer division cannot strand escrow. `tranche` is stored
   because a generic `u32 → Balance` conversion is not available.
2. **Expiry semantics** — **persist.** A commitment is *not* closed when its escrow is paid
   out; it holds the content id until `expires_at`, then `refund_expired` releases it. The term
   is inclusive for proving (`now <= expires_at`) and exclusive for refunding
   (`now > expires_at`), so the two can never both fire.
3. **`MinEscrow` / `MinProofInterval`** — *"average numbers, adjust later"*: implemented as
   clearly-flagged `⚠️ POLICY PLACEHOLDER` runtime constants (10 DALLA; 100 blocks; ~1 year
   term; 16 open commitments; 100-block challenge window). They are a starting point, not a
   settled economic policy, and are labelled as such in the code.
4. **Provider replacement** — **yes.** Implemented as `release_provider` (call 6), alongside
   the earned challenge exit. See §6 for the asymmetry this creates.

### Still open

5. **`weights.rs` regeneration** (§9). Estimates ship until then.
6. **`ChallengeWindow` vs retrieval time** (§6) — the constant needs a real budget before it
   means anything.
7. **Pre-upgrade inspection** (§10) — confirm whether live `StorageProofs` rows exist.
8. **Provider-side certainty** (§6) — whether a notice period or bonding is wanted, given
   `release_provider` is unilateral.

## 15. Implementation record

Built 2026-10-12, once the product decisions above were given.

| Check | Result |
|---|---|
| `cargo test -p pallet-storage-proof` | 27 passed, 0 failed |
| `… --features runtime-benchmarks` | 34 passed, 0 failed (27 unit + 7 benchmarks) |
| `cargo clippy -p pallet-storage-proof --all-targets --features runtime-benchmarks -- -D warnings` | clean |
| `cargo clippy -p belizechain-runtime -- -D warnings` | clean |
| `cargo test -p pallet-belize-staking` (item D) | 61 passed, 0 failed |

### 15.1 Three real defects found while building this

All three were latent in the draft and would have shipped as bugs.

**(a) `ContentAlreadyProven` would have capped every deal at one tranche.** The existing pallet
refused a second proof for the same content id. Under a tranche model only proof #1 would ever
pay, the other `expected_proofs - 1` tranches would be unpayable, and the deal could never
complete. Repeated proofs over the term *are* the mechanism, so the duplicate guard was
removed; the spacing rule (`ProofTooSoon`) plus `proofs_paid == expected_proofs` bounds payment
instead.

**(b) The storage deposit was re-reserved on every proof.** `StorageProofs` is an upsert, so
reserving on each submission would lock N deposits against a single record while `revoke_proof`
can return only one — silently confiscating the rest. The deposit is now taken only when no
record exists for the content.

**(c) The expiry boundary lost the final tranche.** With `expected_proofs × proof_interval`
landing exactly on `expires_at`, a strict `now < expires_at` rejected the last, largest payment.
Fixed by making the term inclusive for proving and exclusive for refunding.

### 15.2 Two invariants worth keeping

- **`EscrowExhausted` is unreachable by construction.** After the last paid proof, the next
  permitted proof could only fall at `commit + (expected + 1) × interval`, which is strictly
  past `expires_at` — so `CommitmentExpired` always fires first. A commitment can never be
  asked to pay a tranche it does not hold. The guard is kept as an invariant, not a live path.
- **The final payout is always non-zero.** `total = expected × tranche + r` with `r < expected`,
  so the last payout is `tranche + r ≥ tranche > 0`.

### 15.3 A constraint worth knowing

The provider must be an **existing account**: `repatriate_reserved` returns the full value as
leftover when the beneficiary account does not exist, which surfaced as `EscrowUnavailable`.
Benign in production, because `ensure_signed` implies the submitter's account exists (it paid a
fee and holds a nonce), but it means a payout can never *create* a provider account. Found by
test, not by reading — the first version of the test used a genesis-less account and failed.

### 15.4 Pre-existing issue found while verifying — **not fixed**

`cargo clippy -p belizechain-runtime --all-targets --features runtime-benchmarks -- -D warnings`
fails on **21 pre-existing unused-parameter warnings** in `runtime/src/lib.rs` (lines ~2128–2532,
inside the `providers` module). They appear only with `runtime-benchmarks` because that module's
KYC/sanctions helpers deliberately short-circuit for benchmarks, leaving their arguments unused
(hence `account`, `required`, `who`, `level`, `now`).

Untouched by this work — `git diff runtime/src/lib.rs` has exactly two hunks (the `VERSION`
block and the storage-proof `Config`), neither in that region. CI is unaffected because the
benchmark job runs `cargo build --features runtime-benchmarks` without `-D warnings`.

**Relevance:** the obvious fix is `_`-prefixing the parameters, but it would silently hide the
fact that the benchmark path ignores its inputs. Worth a deliberate decision rather than a
blanket lint fix. Flagged, not swept.
