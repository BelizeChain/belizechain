# BelizeChain Quantum Pallet — Security Audit Report

**Date**: 2026-03-15  
**Auditor**: AI Security Audit (line-by-line)  
**Scope**: `pallets/quantum/src/` — lib.rs, weights.rs, benchmarking.rs, mock.rs, tests.rs  
**Total Lines Analyzed**: ~4,976  
**Pallet Version**: Phase 2.3 (NFT marketplace + bridge + multi-validator verification)

---

## Executive Summary

| Severity | Count |
|----------|-------|
| **CRITICAL** | 4 |
| **WARNING** | 11 |
| **INFO** | 9 |

**Overall Pallet Score: 62/100**

The quantum pallet has solid foundational architecture with proper use of `BoundedVec`, saturating arithmetic, and `ensure!` guards. However, it contains **4 critical vulnerabilities** primarily around access control gaps, economic exploits in the marketplace, and a missing bridge security model. These must be fixed before mainnet.

---

## Per-File Scores

| File | Lines | Score | Summary |
|------|-------|-------|---------|
| `lib.rs` | ~2,200 | 58/100 | Core logic — critical AC gaps, economic exploits |
| `weights.rs` | ~120 | 72/100 | Hand-estimated, not benchmarked — weight undercount risk |
| `benchmarking.rs` | ~440 | 82/100 | Good coverage, minor helper issues |
| `mock.rs` | ~155 | 90/100 | Well-structured mock runtime |
| `tests.rs` | ~1,500 | 75/100 | Good coverage, missing adversarial edge cases |

---

## CRITICAL Findings

### C-1: `record_quantum_result` — No Access Control on Executor (lib.rs L1023-1096)

**Severity**: CRITICAL  
**Category**: Access Control  

```rust
pub fn record_quantum_result(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    ...
) -> DispatchResult {
    let executor = ensure_signed(origin)?;
    // ...
    // NO CHECK: Any signed account can record a result for any Running job
    QuantumJobs::<T>::try_mutate(&job_id, |maybe_job| {
        let job = maybe_job.as_mut().ok_or(Error::<T>::JobNotFound)?;
        // ...
        job.executor = Some(executor.clone()); // Attacker becomes executor
```

**Impact**: Any account can call `record_quantum_result` for any Running/Completed job, injecting arbitrary `result_data_hash` and becoming the `executor`. When the result is later verified (via root or multi-validator consensus), payment (`dalla_cost`) is transferred to the attacker-as-executor. This is a **fund theft vector**.

**Recommendation**: Require the caller to be either:
- An approved executor/validator registered on-chain, OR
- The job submitter's designated executor, OR
- Implement a stake-based executor registration system.

```rust
// Add check: executor must be registered or match job's assigned executor
ensure!(
    Self::is_registered_executor(&executor) || 
    job.executor.as_ref() == Some(&executor),
    Error::<T>::NotAuthorized
);
```

---

### C-2: `update_job_status` — Submitter Can Self-Approve Completion (lib.rs L950-1020)

**Severity**: CRITICAL  
**Category**: Access Control / Economic  

```rust
ensure!(
    caller == job.submitter || 
    job.executor.as_ref() == Some(&caller),
    Error::<T>::NotAuthorized
);
```

The submitter can transition their own job from `Running → Completed`. Combined with C-1 (anyone can become executor), an attacker flow exists:

1. Submit job (reserves funds)
2. Transition `Pending → Running` (submitter allowed)
3. Record result as self (C-1 — becomes executor)
4. Transition `Running → Completed` (submitter allowed)
5. Request verification → collude with validators → collect own reserved funds

**Impact**: Self-referential payment loop allowing economic extraction. The submitter effectively pays themselves.

**Recommendation**: Separate privilege: only the executor or root should transition `Running → Completed`. The submitter should only be able to cancel.

---

### C-3: `buy_nft` — Royalty Payment Skipped Silently (lib.rs L1435-1445)

**Severity**: CRITICAL  
**Category**: Economic / Logic Error  

```rust
// Pay royalty to original minter if not seller
if listing.original_minter != listing.seller {
    T::Currency::transfer(&buyer, &listing.original_minter, royalty, ExistenceRequirement::KeepAlive)?;
}
```

When `original_minter == seller`, the royalty (5% of sale price) is **not** transferred to anyone — it disappears from the accounting but the buyer is only charged `seller_amount + marketplace_fee`. The buyer pays 95% instead of 100%.

**Impact**: Every time the original minter sells their NFT, the buyer gets a 5% discount at the expense of the total sale price integrity. The seller receives 93% and treasury receives 2%, but 5% vanishes. This is economically incorrect — when `original_minter == seller`, the seller should receive `seller_amount + royalty` (98%).

**Recommendation**:
```rust
if listing.original_minter != listing.seller {
    T::Currency::transfer(&buyer, &listing.original_minter, royalty, ExistenceRequirement::KeepAlive)?;
} else {
    // Original minter is selling — add royalty back to seller amount
    T::Currency::transfer(&buyer, &listing.seller, royalty, ExistenceRequirement::KeepAlive)?;
}
```

---

### C-4: Bridge — No Relayer Authorization Model (lib.rs L1515-1620)

**Severity**: CRITICAL  
**Category**: Security Architecture  

The bridge functions (`bridge_to_ethereum`, `bridge_to_parachain`) lock NFTs and emit events, but there is:
- **No authorized relayer set** — anyone can observe events
- **No claim verification** — `BridgeClaimed` event and `claim_tx_hash` are referenced but never set by any extrinsic
- **No timeout mechanism** — locked NFTs can be stuck forever if the bridge never completes
- **No signature verification** — the destination chain claim is never verified on BelizeChain

**Impact**: NFTs can be permanently locked with no reclaim mechanism beyond `cancel_bridge`. If a relayer is compromised or non-existent, there's no way to verify claims actually happened on the destination chain. The bridge is effectively a one-way lock.

**Recommendation**: 
1. Add `complete_bridge(origin, nft_id, claim_tx_hash)` extrinsic callable only by authorized relayers
2. Add bridge timeout (e.g., 7200 blocks) after which `cancel_bridge` auto-executes
3. Implement light-client or oracle-based claim verification

---

## WARNING Findings

### W-1: `submit_verification` — Vote Count Overflow Without Saturation (lib.rs L1830-1832)

**Severity**: WARNING  
**Category**: Arithmetic  

```rust
match vote {
    VerificationVote::Approve => request.approvals += 1,
    VerificationVote::Reject => request.rejections += 1,
    VerificationVote::Abstain => {},
}
```

`approvals` and `rejections` are `u8` fields. With `BoundedVec<ValidatorVerification, ConstU32<10>>`, max 10 votes are possible, so overflow at 255 is unreachable in practice. However, raw `+= 1` without `.saturating_add(1)` violates the project's no-panic arithmetic rule.

**Recommendation**: Use `request.approvals = request.approvals.saturating_add(1)`.

---

### W-2: Hand-Estimated Weights — Undercount Risk (weights.rs entire file)

**Severity**: WARNING  
**Category**: Weight / DoS  

All 14 weight functions are hand-estimated, not generated by `frame-benchmarking`. Comments even state:
```
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
```

**Specific concerns**:
- `submit_verification()`: 50M ref_time but includes iterating verifications list (O(N), N≤10), decoding accounts, updating reputations for all validators. This is likely **underweighted** by 2-3x.
- `buy_nft()`: 45M ref_time but performs 2-3 `Currency::transfer` calls plus storage mutations. Each transfer involves multiple DB reads/writes.
- `submit_quantum_job()`: Claims 4 reads, 5 writes, but `try_mutate` on `JobsByAccount` involves read+write, and `AccountStats` is another read+write. Actual: 6 reads, 6 writes.

**Impact**: Underweighted extrinsics allow cheaper-than-expected block stuffing.

**Recommendation**: Run `frame-benchmarking` before mainnet. As interim, multiply all computational weights by 2x safety factor.

---

### W-3: `update_job_status` — Cancellation From Completed State Leaks Funds (lib.rs L994-998)

**Severity**: WARNING  
**Category**: Economic / State Machine  

```rust
(_, JobStatus::Cancelled) => {
    if job.status != JobStatus::Completed {
        let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
    }
},
```

The wildcard `_` match allows transition from **any** status to `Cancelled`, including from `Failed` (which already unreserved funds) and `Cancelled` itself (idempotent but wasteful). If a job is cancelled after failing, `unreserve` is called on funds that were already unreserved — this is safe (returns 0), but the state machine allows illogical transitions like `Cancelled → Cancelled`.

**Impact**: Low direct impact due to `unreserve` safety, but the overly-permissive wildcard weakens state machine integrity. A `Completed → Cancelled` transition skips the refund but resets the status, potentially confusing downstream consumers.

**Recommendation**: Explicitly enumerate valid cancellation sources:
```rust
(JobStatus::Pending | JobStatus::Running, JobStatus::Cancelled) => {
    let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
},
```

---

### W-4: `mint_achievement_nft` — No Duplicate Mint Guard (lib.rs L1207-1310)

**Severity**: WARNING  
**Category**: Logic / Economic  

There is no check preventing the same account from minting multiple NFTs for the same `job_id` and `achievement_type`. Each mint charges `NFTMintingFee` (burned), but creates unlimited NFTs from a single quantum job.

**Impact**: Dilutes NFT rarity and undermines the achievement system. An attacker can mint 1000 "ShorAlgorithm" NFTs from a single job if they have sufficient DALLA for fees.

**Recommendation**: Add storage check:
```rust
ensure!(
    !Self::has_achievement_for_job(owner, &job_id, &achievement_type),
    Error::<T>::AchievementAlreadyMinted
);
```

---

### W-5: `get_active_job_count` — O(N) Full Iteration (lib.rs L1895-1905)

**Severity**: WARNING  
**Category**: DoS / Performance  

```rust
pub fn get_active_job_count(account: &T::AccountId) -> u32 {
    let jobs = JobsByAccount::<T>::get(account);
    jobs.iter()
        .filter(|job_id| {
            QuantumJobs::<T>::get(job_id)  // DB read per job
                .map(|job| job.status == JobStatus::Pending || job.status == JobStatus::Running)
                .unwrap_or(false)
        })
        .count() as u32
}
```

Iterates up to 100 jobs and performs a DB read for each. This is called from **no extrinsic** (it's a public helper), but if added to a query or extrinsic path, it becomes a DoS vector.

**Additionally**: `MaxActiveJobs` (100) is checked via `JobsByAccount` push limit, but completed/failed jobs are never removed from `JobsByAccount`, so the limit actually constrains *total* jobs (not active ones).

**Impact**: Users are permanently locked out after 100 total jobs (not 100 active).

**Recommendation**: Remove completed/failed jobs from `JobsByAccount` when status changes, or maintain a separate `ActiveJobCount` counter.

---

### W-6: `has_achievement` — O(N) Full Storage Iteration (lib.rs L1908-1912)

**Severity**: WARNING  
**Category**: DoS / Performance  

```rust
pub fn has_achievement(account: &T::AccountId, achievement: AchievementType) -> bool {
    QuantumAchievements::<T>::iter()
        .any(|(_, nft)| nft.owner == *account && nft.achievement_type == achievement)
}
```

Iterates **ALL** NFTs in storage. If used in an extrinsic path, this is an unbounded O(N) DoS vector.

**Impact**: Currently only used as a public helper. If any future extrinsic calls this, it creates unbounded computation.

**Recommendation**: Add indexed storage `AchievementsByAccount<T>` mapping `(AccountId, AchievementType) → bool`.

---

### W-7: `bridge_to_ethereum` — Weak Address Validation (lib.rs L1548)

**Severity**: WARNING  
**Category**: Input Validation  

```rust
ensure!(recipient.len() == 20 || recipient.len() == 42, Error::<T>::InvalidRecipientAddress);
```

Accepts either 20 raw bytes or 42 bytes (hex-encoded with "0x" prefix), but:
- Does not validate hex encoding for 42-byte case
- Does not check for all-zero addresses (burn address)
- `bridge_to_parachain` has **no** recipient validation at all

**Impact**: Invalid or burn addresses lock NFTs permanently.

**Recommendation**: For 42-byte: validate hex prefix "0x". For parachain: validate address format. Reject all-zero recipients.

---

### W-8: NFT Counter Overflow (lib.rs L1272-1276)

**Severity**: WARNING  
**Category**: Arithmetic  

```rust
let nft_id = NFTCounter::<T>::mutate(|counter| {
    let id = *counter;
    *counter = counter.saturating_add(1);
    id
});
```

Uses `saturating_add` so no panic, but at `u64::MAX`, the counter saturates and all subsequent mints get the same ID, overwriting previous NFTs.

**Impact**: At u64::MAX (18.4 quintillion), NFT ID collisions occur. Practically unreachable but architecturally wrong.

**Recommendation**: Add `ensure!(*counter < u64::MAX, Error::<T>::ArithmeticOverflow)` before mutation. Same applies to `JobCounter`, `ListingCounter`, `BridgeCounter`.

---

### W-9: `ListingCounter` Never Decremented on Purchase/Delist (lib.rs L1395, L1477)

**Severity**: WARNING  
**Category**: State Consistency  

`ListingCounter` is incremented in `list_nft` but never decremented in `buy_nft` or `delist_nft`. It becomes a monotonically increasing counter that does not reflect actual listings.

**Impact**: Any off-chain service or UI relying on `ListingCounter` as "active listings count" will display incorrect data. Low on-chain impact.

**Recommendation**: Add `ListingCounter::<T>::mutate(|c| *c = c.saturating_sub(1))` in both `buy_nft` and `delist_nft`.

---

### W-10: `submit_verification` — Events Emitted After Storage But Before Return (lib.rs L1870-1875)

**Severity**: WARNING  
**Category**: Consistency  

The `VerificationConsensusReached` event is emitted inside the consensus-reached block, then `VerificationRequests::insert` happens afterward, then `VerificationSubmitted` event. If the event deposit fails (it shouldn't in Substrate, but for logical ordering):
- Consensus event is emitted before the request is persisted
- Two separate events may have inconsistent ordering relative to storage state

**Impact**: Low — Substrate event deposits don't fail. But event ordering inconsistency could confuse indexers.

**Recommendation**: Move `VerificationRequests::insert` before the consensus event emission.

---

### W-11: `verify_quantum_result` — Ignores `repatriate_reserved` Return (lib.rs L1152-1158)

**Severity**: WARNING  
**Category**: Error Handling  

```rust
let _ = T::Currency::repatriate_reserved(
    &job.submitter,
    executor,
    job.dalla_cost,
    frame_support::traits::BalanceStatus::Free,
);
```

`repatriate_reserved` can fail if the submitter's reserved balance is less than `dalla_cost` (e.g., after a slash). The `let _ =` silently discards the error, meaning verification succeeds but executor is never paid.

**Impact**: After a slash event, verified jobs silently fail to pay executors. The job shows as "Verified" but no funds are transferred.

**Recommendation**: At minimum, emit a warning event. Ideally, propagate the error or check the returned deficit.

---

## INFO Findings

### I-1: `QuantumBackend::from_str` Catches All Unknown Inputs as `Other` (lib.rs L225)

**Severity**: INFO  
**Category**: Input Handling  

The `from_str` method returns `Some(QuantumBackend::Other)` for all unrecognized strings, never returning `None`. This means invalid input is silently accepted. Low risk as this is a helper function, not used in extrinsic paths.

---

### I-2: `recorded_at` in `QuantumResult` Uses `u32` (lib.rs L423)

**Severity**: INFO  
**Category**: Type Safety  

`recorded_at` is `u32` while `BlockNumberFor<T>` is the standard Substrate block number type. The `saturated_into::<u32>()` conversion is documented and safe for BelizeChain's u32 blocks, but limits portability.

---

### I-3: `generate_metadata_uri` Uses Placeholder IPFS URI (lib.rs L2035-2060)

**Severity**: INFO  
**Category**: Production Readiness  

Generates `ipfs://QmBelizeChain/{rarity}/{category}/{nft_id}.json` — a placeholder, not a real IPFS CID. Noted as TODO for production.

---

### I-4: No `on_initialize` / `on_finalize` Hooks (lib.rs)

**Severity**: INFO  
**Category**: Architecture  

The pallet has no block hooks. This means:
- Expired listings are never auto-cleaned
- Verification deadlines are only checked on explicit `submit_verification` calls
- Bridge timeouts are never enforced

**Recommendation**: Consider an `on_initialize` hook to clean expired listings and enforce bridge timeouts.

---

### I-5: `AuctionListing` Struct Defined But Never Used (lib.rs L485-508)

**Severity**: INFO  
**Category**: Dead Code  

`AuctionListing`, `NFTAuctions`, `AuctionCreated`, `BidPlaced`, `AuctionFinalized` events, and `AuctionNotFound`/`AuctionEnded`/`AuctionStillActive`/`BidTooLow` errors are all defined but no auction extrinsics exist. This is dead code.

---

### I-6: `format!` Macro in `no_std` Context (lib.rs L60)

**Severity**: INFO  
**Category**: Compatibility  

```rust
extern crate alloc;
use alloc::format;
```

Using `alloc::format` is correct for `no_std`, but `generate_metadata_uri` allocates a `String` on every NFT mint, which is heap-intensive. Consider pre-computed URI components.

---

### I-7: `circuit_depth` Parameter Accepted But Never Validated (lib.rs L881)

**Severity**: INFO  
**Category**: Input Validation  

`circuit_depth` is stored in the `QuantumJob` struct but has no validation bounds (unlike `num_qubits` 1-100 and `num_shots` 1-1M). Any `u32` value is accepted.

---

### I-8: `JobsByAccount` Stores All Jobs, Not Just Active (lib.rs L535-541)

**Severity**: INFO  
**Category**: Storage Bloat  

`JobsByAccount` bounded to 100 entries is never pruned. All jobs (Pending, Running, Completed, Failed, Cancelled) accumulate. This limits users to 100 lifetime jobs.

---

### I-9: `validator` Field in `QuantumResult` Is `BoundedVec<u8, ConstU32<32>>` Not `AccountId` (lib.rs L421)

**Severity**: INFO  
**Category**: Type Safety  

The validator is stored as raw bytes via `executor.encode().try_into().unwrap_or_default()`. If the `AccountId` encoding exceeds 32 bytes, it silently truncates to an empty vec. Using the actual `AccountId` type would be safer, though this requires making `QuantumResult` generic over `T`.

---

## Test Coverage Assessment

| Area | Tests | Coverage | Missing |
|------|-------|----------|---------|
| Job submission | 8 | ✅ Good | Edge case: max concurrent jobs per account |
| Job status transitions | 5 | ✅ Good | Missing: double-cancel, already-completed cancel |
| Result recording | 4 | ✅ Good | Missing: unauthorized executor test |
| Root verification | 3 | ✅ Good | — |
| NFT minting | 2 | ⚠️ Partial | Missing: duplicate mint, unauthorized mint |
| NFT transfer | 2 | ⚠️ Partial | Missing: transfer to self, locked NFT |
| Marketplace | 4 | ⚠️ Partial | Missing: expired listing buy, price=0 listing |
| Bridge | 3 | ⚠️ Partial | Missing: bridge timeout, double bridge |
| Multi-validator | 2 | ⚠️ Partial | Missing: deadline expiry, abstain-only consensus |
| Cancel bridge | 2 | ✅ Good | — |

**Test Score: 75/100** — Good happy-path coverage, weak adversarial/edge-case testing.

---

## Determinism Assessment

| Check | Status |
|-------|--------|
| No floating-point | ✅ Pass |
| No randomness | ✅ Pass |
| No external I/O | ✅ Pass |
| No timestamp dependency | ✅ Pass |
| Deterministic error paths | ✅ Pass |
| Block number used correctly | ✅ Pass |

---

## Recommendations Priority Matrix

| Priority | Finding | Effort |
|----------|---------|--------|
| 🔴 P0 | C-1: Add executor access control | Medium |
| 🔴 P0 | C-2: Restrict status transitions by role | Medium |
| 🔴 P0 | C-3: Fix royalty accounting in `buy_nft` | Low |
| 🔴 P0 | C-4: Design bridge completion + timeout | High |
| 🟡 P1 | W-1: Saturating arithmetic on vote counts | Trivial |
| 🟡 P1 | W-2: Run frame-benchmarking for weights | Medium |
| 🟡 P1 | W-3: Restrict cancellation state transitions | Low |
| 🟡 P1 | W-4: Duplicate mint guard | Low |
| 🟡 P1 | W-5: Fix `JobsByAccount` pruning | Medium |
| 🟡 P1 | W-11: Handle `repatriate_reserved` errors | Low |
| 🟢 P2 | W-6: Index achievements by account | Medium |
| 🟢 P2 | W-7: Strengthen bridge address validation | Low |
| 🟢 P2 | W-8: Counter overflow guard | Trivial |
| 🟢 P2 | W-9: Decrement ListingCounter | Trivial |

---

## Conclusion

The quantum pallet implements a complex system spanning job management, NFT lifecycle, marketplace economics, cross-chain bridging, and multi-validator consensus. The architecture is generally sound with proper use of Substrate primitives (`BoundedVec`, `ensure!`, `saturating_*`).

**Critical issues C-1 and C-2** represent the most urgent security concerns: the open executor registration and self-approval flow create a fund theft vector that must be closed before any tokens have real value. **C-3** is a straightforward accounting bug. **C-4** requires architectural design work.

The pallet is **not mainnet-ready** in its current state. After fixing the 4 critical issues and running proper benchmarks, a re-audit is recommended.
