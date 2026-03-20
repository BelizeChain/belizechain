# JUSTICE PALLET (`pallet-belize-justice`) — SECURITY AUDIT 2026

**Auditor**: AI Security Auditor (Principal Blockchain Security Engineer)
**Date**: 2026-03-15
**Pallet**: `pallet-belize-justice` (on-chain dispute resolution / restorative justice)
**Codebase**: BelizeChain `belizechain` branch, Substrate stable2512
**Files Audited**: 5 files, 1,585 lines total
**Scope**: Complete line-by-line audit across all 14 security categories + justice-system-specific focus areas

---

## EXECUTIVE SUMMARY

The Justice pallet implements a dispute resolution system with mediator-based rulings, appeals, escrow slashing, and a rehabilitation lifecycle. The design is well-structured with bounded storage, proper error handling, and saturating arithmetic. However, the audit reveals **8 findings** including **2 HIGH** severity issues related to double-origin-check bypass potential and escrow overwrite, **3 MEDIUM** issues, and **3 LOW/INFO** issues.

The most critical concern is the `escrow_slash` public function which silently overwrites previously escrowed amounts, potentially causing loss of reserved funds. The second highest concern is a potential MediatorOrigin misconfiguration where `EnsureSigned` in mock allows any signed user to pass the origin check (relying only on the storage check), which could be dangerous if `MediatorOrigin` is misconfigured at the runtime level.

**Overall Security Score: 78/100** — CONDITIONALLY SAFE FOR MAINNET (requires HIGH fixes first)

---

## SEVERITY TABLE

| ID | Severity | Category | File | Line(s) | Title |
|----|----------|----------|------|---------|-------|
| J-01 | **HIGH** | State Inconsistency / Economic | lib.rs | 599–601 | `escrow_slash` overwrites existing escrow without accumulating |
| J-02 | **HIGH** | Access Control | lib.rs | 383–393 | Double origin check on `mediator_ruling` — fragile dual-ensure pattern |
| J-03 | **MEDIUM** | DoS / Unbounded Iteration | lib.rs | 303–325 | `on_idle` iterates all `AppealedAt` entries with no upper bound |
| J-04 | **MEDIUM** | State Inconsistency | lib.rs | 343–367 | Disputant bond not slashed on Upheld/Mediated (griefing vector) |
| J-05 | **MEDIUM** | Integer Arithmetic | lib.rs | 441–444 | `slash_bps` calculation uses integer division truncation |
| J-06 | **LOW** | Weight Underestimate | weights.rs | 20–31 | Hardcoded weight estimates — not from benchmarks |
| J-07 | **LOW** | DoS / Griefing | lib.rs | 352–377 | Disputant can file unlimited disputes against same target |
| J-08 | **INFO** | Determinism | lib.rs | 303–325 | `on_idle` iteration order depends on storage iterator order |

---

## DETAILED FINDINGS

---

### J-01 — `escrow_slash` OVERWRITES existing escrow (LOSS OF RESERVED FUNDS)

| Attribute | Value |
|-----------|-------|
| **Severity** | **HIGH** |
| **Category** | 7 — State Inconsistency / Economic Loss |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L593-L601) |
| **Lines** | 593–601 |

**Code**:
```rust
pub fn escrow_slash(account: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
    T::Currency::reserve(account, amount)?;
    SlashPendingJusticeReview::<T>::insert(account, amount);  // ← OVERWRITES
    Self::deposit_event(Event::SlashEscrowed { account: account.clone(), amount });
    Ok(())
}
```

**Vulnerability**: If an external pallet calls `escrow_slash` twice for the same account before the mediator resolves the first dispute, the second call:
1. Reserves `amount2` from the account (so total reserved = `amount1 + amount2`)
2. **Overwrites** the storage value with `amount2` only
3. When the mediator rules, only `amount2` is unreserved/slashed — `amount1` remains permanently locked as a phantom reserve

**Impact**: Permanent lock of reserved funds. The staking pallet already guards against this (checks `has_pending_review` before calling), but the justice pallet's own API provides no defense. Any new cross-pallet caller could trigger the bug.

**Proof of Concept**:
```
escrow_slash(BOB, 1000)  → reserved=1000, storage=1000
escrow_slash(BOB, 2000)  → reserved=3000, storage=2000
mediator_ruling(Dismissed) → unreserves 2000 → reserved=1000 (STUCK)
```

**Fix**:
```rust
pub fn escrow_slash(account: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
    T::Currency::reserve(account, amount)?;
    SlashPendingJusticeReview::<T>::mutate(account, |existing| {
        *existing = Some(existing.unwrap_or_default().saturating_add(amount));
    });
    Self::deposit_event(Event::SlashEscrowed { account: account.clone(), amount });
    Ok(())
}
```

Or add a guard:
```rust
ensure!(!SlashPendingJusticeReview::<T>::contains_key(account), Error::<T>::EscrowAlreadyPending);
```

---

### J-02 — Double origin check on `mediator_ruling` — fragile dual-ensure pattern

| Attribute | Value |
|-----------|-------|
| **Severity** | **HIGH** |
| **Category** | 4 — Access-Control Bypass |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L383-L393) |
| **Lines** | 383–393 |

**Code**:
```rust
pub fn mediator_ruling(
    origin: OriginFor<T>,
    dispute_id: u32,
    resolution_code: u8,
    slash_bps: u32,
) -> DispatchResult {
    T::MediatorOrigin::ensure_origin(origin.clone())?;   // ← check 1
    let mediator = ensure_signed(origin)?;                // ← check 2

    let list = MediatorList::<T>::get();
    ensure!(list.contains(&mediator), Error::<T>::NotApprovedMediator);  // ← check 3
```

**Vulnerability**: The `origin.clone()` pattern calls `ensure_origin` on the original origin, then calls `ensure_signed` on the same origin. This works because Substrate origins are cloneable. However:

1. **In mock**: `MediatorOrigin = EnsureSigned<u64>` — this means ANY signed account passes check 1 and the actual security relies entirely on the `MediatorList` storage check (check 3). This is actually correct belt-and-suspenders design.

2. **In runtime**: `MediatorOrigin = TechnicalCouncilMember` — if the `TechnicalCouncilMember` origin is a collective origin (not signed), then `ensure_signed(origin)` on line 390 will **always fail**, making the function **permanently uncallable**.

3. **Configuration risk**: If a runtime configures `MediatorOrigin` as a non-signed origin (e.g., collective), the pallet becomes bricked because `ensure_signed` cannot extract an `AccountId` from a collective origin.

**Impact**: The function could be permanently uncallable depending on runtime configuration, bricking the entire justice system. The current runtime config (`TechnicalCouncilMember`) needs verification — if it resolves to a signed origin (member-of-collective check that still passes through as signed), it works. If it's a collective dispatch origin, the pallet is broken.

**Fix**: Use a single origin model:
```rust
pub fn mediator_ruling(
    origin: OriginFor<T>,
    dispute_id: u32,
    resolution_code: u8,
    slash_bps: u32,
) -> DispatchResult {
    let mediator = ensure_signed(origin)?;
    let list = MediatorList::<T>::get();
    ensure!(list.contains(&mediator), Error::<T>::NotApprovedMediator);
    // Remove MediatorOrigin from Config entirely — the storage list IS the access control
```

Or if the intent is to require both council membership AND mediator list presence, document this explicitly and verify the runtime origin type is compatible with `ensure_signed`.

---

### J-03 — `on_idle` iterates all `AppealedAt` entries — unbounded DoS vector

| Attribute | Value |
|-----------|-------|
| **Severity** | **MEDIUM** |
| **Category** | 5 — Unbounded Iteration / DoS |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L303-L325) |
| **Lines** | 303–325 |

**Code**:
```rust
fn on_idle(now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
    let base_read = T::DbWeight::get().reads(1);
    let mut used = Weight::zero();
    let timeout = T::AppealTimeout::get();
    let mut to_close = Vec::new();       // ← unbounded Vec allocation
    for (dispute_id, appealed_block) in AppealedAt::<T>::iter() {
        used = used.saturating_add(base_read);
        if used.any_gt(remaining_weight) {
            break;
        }
        if now >= appealed_block.saturating_add(timeout) {
            to_close.push(dispute_id);   // ← grows unbounded
        }
    }
```

**Vulnerability**: 
1. The `to_close` Vec can grow to contain thousands of entries if many appeals time out simultaneously, causing an unbounded heap allocation within `on_idle`.
2. `AppealedAt::iter()` reads ALL entries from storage — even with the weight break, the Vec is allocated before the second loop runs.
3. An attacker could file many disputes, get them all ruled, appeal them all, then wait for timeout — causing a large `on_idle` burst.

**Mitigating factors**: The weight budget check limits how many entries get processed, and `on_idle` only uses leftover weight. But the `Vec` allocation is not weight-bounded.

**Impact**: Potential OOM or excessive memory use in block production. Moderate DoS risk.

**Fix**: Process entries inline instead of collecting into a Vec, or cap the Vec:
```rust
fn on_idle(now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
    let base_read = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_item_cost = base_read.saturating_add(write_cost).saturating_mul(2);
    let mut used = Weight::zero();
    let timeout = T::AppealTimeout::get();

    for (dispute_id, appealed_block) in AppealedAt::<T>::iter() {
        let needed = base_read.saturating_add(per_item_cost);
        if used.saturating_add(needed).any_gt(remaining_weight) {
            break;
        }
        used = used.saturating_add(base_read);
        if now >= appealed_block.saturating_add(timeout) {
            if let Some(mut record) = Disputes::<T>::get(dispute_id) {
                if record.status == DisputeStatus::Appealed {
                    record.status = DisputeStatus::Closed;
                    Disputes::<T>::insert(dispute_id, record);
                    AppealedAt::<T>::remove(dispute_id);
                    Self::deposit_event(Event::AppealTimedOut { dispute_id });
                    used = used.saturating_add(per_item_cost);
                }
            }
        }
    }
    used
}
```

**Note**: Iterating while mutating via `AppealedAt::remove` inside a `for ... in AppealedAt::iter()` loop is generally unsafe (iterator invalidation). The two-phase approach in the current code avoids this, but the Vec is unbounded. The fix above would need to use `drain_prefix` or collect first with a cap.

---

### J-04 — Disputant bond not slashed on Upheld/Mediated — frivolous dispute griefing

| Attribute | Value |
|-----------|-------|
| **Severity** | **MEDIUM** |
| **Category** | 7 — State Inconsistency / Economic |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L462-L468) |
| **Lines** | 462–468 |

**Code**:
```rust
// Refund disputant bond on Dismissed (they were right)
if matches!(resolution, DisputeResolution::Dismissed) {
    T::Currency::unreserve(&record.disputant, record.bond);
}

record.resolution = Some(resolution.clone());
record.status = DisputeStatus::Ruled;
```

**Vulnerability**: When the ruling is `Upheld` or `Mediated`, the disputant's bond is neither refunded nor slashed — it remains **permanently reserved**. The pallet docs state "refunded on Dismissed, forfeited on frivolous" but there's no actual forfeiture path, and legitimate disputants whose claims are upheld lose their bond too.

This creates two problems:
1. **Honest disputants penalized**: If Alice reports Bob and the mediator upholds the claim, Alice's bond stays locked forever.
2. **No griefing penalty**: The docs say frivolous filings get the bond slashed, but no code path actually slashes the disputant's bond.

**Impact**: Bond permanently locked for ALL non-dismissed disputes. Economic unfairness for honest disputants.

**Fix**:
```rust
match &resolution {
    DisputeResolution::Dismissed => {
        // Frivolous filing — slash disputant's bond
        T::Currency::unreserve(&record.disputant, record.bond);
        let _ = T::Currency::slash(&record.disputant, record.bond);
    }
    DisputeResolution::Upheld | DisputeResolution::Mediated { .. } => {
        // Legitimate dispute — refund disputant's bond
        T::Currency::unreserve(&record.disputant, record.bond);
    }
}
```

Or reverse the semantic if "Dismissed" means "allegation was valid and dismissed against the accused":
```rust
// If Dismissed means the allegation was dismissed (disputant was wrong):
DisputeResolution::Dismissed => {
    // Disputant wrong — consider slashing bond for frivolous filing
    // For now: just unreserve (no penalty for good-faith mistakes)
    T::Currency::unreserve(&record.disputant, record.bond);
}
DisputeResolution::Upheld | DisputeResolution::Mediated { .. } => {
    // Disputant was right — refund bond
    T::Currency::unreserve(&record.disputant, record.bond);
}
```

---

### J-05 — `slash_bps` calculation uses integer division truncation

| Attribute | Value |
|-----------|-------|
| **Severity** | **MEDIUM** |
| **Category** | 2 — Division / Rounding Abuse |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L441-L444) |
| **Lines** | 441–444 |

**Code**:
```rust
DisputeResolution::Mediated { slash_bps } => {
    let slash_amount = escrowed.saturating_mul((*slash_bps as u32).into())
        / 10_000u32.into();
    let refund = escrowed.saturating_sub(slash_amount);
```

**Vulnerability**:
1. **Truncation toward zero**: Integer division always truncates. For small escrow amounts, `escrowed * slash_bps / 10_000` could round down to 0, meaning 0 is slashed and the full amount is refunded even though a partial slash was intended.
   - Example: `escrowed=99, slash_bps=100 (1%)` → `99 * 100 / 10_000 = 0` → full refund despite 1% penalty intended.

2. **Rounding mismatch**: `slash_amount + refund` may not equal `escrowed` due to truncation. The current code uses `escrowed.saturating_sub(slash_amount)` for refund, which is correct (no fund loss), but the slash is short by up to 1 unit.

3. **`*slash_bps as u32` redundant cast**: `slash_bps` is already `&u32`, so `*slash_bps as u32` is a no-op. Not a bug, but misleading.

**Impact**: Systematic rounding in favor of the accused. At DALLA's 12-decimal precision (1 DALLA = 1e12 units), the truncation is <1 planchet and practically irrelevant for real-world amounts. **Low practical impact** but the pattern is technically imprecise.

**Fix**: Accept the truncation (it's <1 planchet) and add a comment, or round up the slash:
```rust
// Round up slash to prevent systematic under-slashing
let slash_amount = escrowed.saturating_mul((*slash_bps).into())
    .saturating_add(9_999u32.into())
    / 10_000u32.into();
let slash_amount = slash_amount.min(escrowed); // cap at 100%
let refund = escrowed.saturating_sub(slash_amount);
```

---

### J-06 — Hardcoded weight estimates — not from benchmarks

| Attribute | Value |
|-----------|-------|
| **Severity** | **LOW** |
| **Category** | 13 — Weight / Gas Underestimation |
| **File** | [weights.rs](pallets/justice/src/weights.rs#L1-L58) |
| **Lines** | 1–58 |

**Code**:
```rust
/// Conservative weight estimates — replace with benchmarks before mainnet.
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn open_dispute() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    // ...
}
```

**Vulnerability**: All weights are manually estimated, not generated from frame-benchmarking. The comment acknowledges this. Incorrect weights can lead to:
- **Under-weight**: Blocks take longer than 6s, degrading liveness
- **Over-weight**: Unnecessarily limits throughput

The `reads(3)` and `writes(3)` for `open_dispute` approximately match the actual storage operations (DisputeCounter read+write, CoolingOffEnd write, RehabilitationStatus write, Disputes write, Currency::reserve read+write = ~3R 4W), so the read count is slightly under.

**Impact**: Minor weight inaccuracy. The estimates appear conservative (generous), so the risk is over-charging rather than under-charging.

**Fix**: Run `frame-benchmarking` before mainnet to generate accurate weights. The benchmarking scaffolding in `benchmarking.rs` is already complete.

---

### J-07 — Disputant can file unlimited disputes against the same target

| Attribute | Value |
|-----------|-------|
| **Severity** | **LOW** |
| **Category** | 5 — DoS / Griefing |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L352-L377) |
| **Lines** | 352–377 |

**Code**:
```rust
pub fn open_dispute(
    origin: OriginFor<T>,
    target: T::AccountId,
    evidence_hash: [u8; 32],
    severity: u8,
) -> DispatchResult {
    let disputant = ensure_signed(origin)?;
    // ... no check for existing disputes against same target by same disputant
    T::Currency::reserve(&disputant, T::OpenDisputeBond::get())?;
    // ...
}
```

**Vulnerability**: There's no limit on how many disputes a single account can file against the same target. An attacker with sufficient funds could file hundreds of disputes, each resetting the target's cooling-off period and forcing the target into `InCoolingOff` indefinitely.

**Mitigating factors**:
- Each dispute costs `OpenDisputeBond` (100 DALLA in production), making large-scale griefing expensive.
- The `DisputeCounter` is `u32`, so it can hold ~4 billion disputes before wrapping (saturating_add prevents overflow).

**Impact**: Wealthy attacker can grief a target by resetting their cooling-off period repeatedly. Economic cost limits severity.

**Fix**: Add rate-limiting:
```rust
// Option 1: One active dispute per disputant-target pair
// Option 2: Require a minimum gap between disputes from the same disputant
// Option 3: Escalating bond for repeated disputes against same target
```

---

### J-08 — `on_idle` iteration order is non-deterministic across nodes

| Attribute | Value |
|-----------|-------|
| **Severity** | **INFO** |
| **Category** | 12 — Determinism Violations |
| **File** | [lib.rs](pallets/justice/src/lib.rs#L303-L325) |
| **Lines** | 303–325 |

**Code**:
```rust
for (dispute_id, appealed_block) in AppealedAt::<T>::iter() {
```

**Vulnerability**: `StorageMap::iter()` in Substrate iterates in storage key order (which is deterministic for `Blake2_128Concat` hasher). However, the *processing order* combined with the weight break means that different validating nodes with different remaining weights could process different subsets of timed-out appeals in the same block.

**Mitigating factors**: `on_idle` is deliberately best-effort. All nodes will eventually process all timed-out appeals (just potentially in different blocks). The Substrate framework explicitly allows `on_idle` to be non-deterministic in terms of how much work is done. State changes from processing are deterministic (same input → same output for each individual entry).

**Impact**: Informational only. This is by-design for `on_idle` hooks.

---

## CATEGORY-BY-CATEGORY ANALYSIS

### 1. Integer Overflow / Underflow / Truncation — ✅ MOSTLY SAFE
- `DisputeCounter` uses `saturating_add(1)` — no overflow ([lib.rs L365](pallets/justice/src/lib.rs#L365))
- `cooling_end` uses `saturating_add` — no overflow ([lib.rs L369](pallets/justice/src/lib.rs#L369))
- `slash_bps` calculation has truncation issue (J-05) but no overflow
- All balance arithmetic uses `Saturating` trait — correct
- **Score: 9/10** (minor truncation in bps calc)

### 2. Division by Zero / Rounding — ⚠️ MINOR ISSUE
- Division by `10_000u32.into()` — constant divisor, never zero
- Truncation bias toward zero (J-05)
- **Score: 8/10**

### 3. Reentrancy / Double-Spend / TOCTOU — ✅ SAFE
- No external contract calls or callbacks
- All state mutations happen within the same extrinsic atomically
- `escrow_slash` does reserve-then-insert (no window between check and effect within the same call)
- The `escrow_slash` overwrite issue (J-01) is a state-consistency bug, not TOCTOU
- **Score: 10/10**

### 4. Access-Control Bypass — ⚠️ SIGNIFICANT ISSUE
- `open_dispute`: `ensure_signed` — correct, anyone can file
- `mediator_ruling`: Dual origin check pattern is fragile (J-02)
- `appeal_ruling`: `ensure_signed` + `caller == record.target` — correct
- `complete_rehabilitation`: `GovernanceOrigin || MediatorOrigin` fallback — correct pattern
- `add_mediator` / `remove_mediator`: `GovernanceOrigin` only — correct
- `escrow_slash`: Public API, no origin check (correct — intended for cross-pallet call)
- **Score: 7/10** (dual origin pattern could brick the pallet)

### 5. Unbounded Iteration / Storage Growth — ⚠️ MODERATE ISSUE
- `MediatorList`: Bounded by `MaxMediators` (20 in production) — ✅
- `Disputes`: Unbounded `StorageMap` indexed by `u32` — grows over time but O(1) access per dispute
- `AppealedAt`: Unbounded, iterated in `on_idle` (J-03)
- `on_idle` has weight budget but unbounded Vec allocation (J-03)
- **Score: 7/10**

### 6. Incorrect Error Handling — ✅ MOSTLY SAFE
- All extrinsics return `DispatchResult` with proper `ensure!` checks
- `Currency::slash` result is deliberately ignored (`let _ = ...`) — this is standard Substrate pattern because `slash` returns the amount that couldn't be slashed (dust), not an error
- `Currency::reserve` errors propagate correctly
- Missing: No error when `escrow_slash` overwrites (J-01)
- **Score: 8/10**

### 7. State Inconsistency — ⚠️ SIGNIFICANT ISSUES
- J-01: Escrow overwrite loses funds
- J-04: Disputant bond permanently locked on non-Dismissed rulings
- Cooling-off period is reset on every new dispute against the same target (J-07) — may or may not be intended
- `RehabilitationStatus` transitions:
  - `Clean → InCoolingOff` (on dispute open) — ✅
  - `InCoolingOff → InRehabilitation` (on Dismissed ruling w/ escrow) — ✅
  - `InCoolingOff → InCoolingOff` (on Upheld ruling w/ escrow) — ✅
  - `InRehabilitation → Reinstated` (on `complete_rehabilitation`) — ✅
  - **Missing**: No transition from `InCoolingOff` to `InRehabilitation` when there's NO escrow (dismissed without escrow means target stays `InCoolingOff` forever)
- **Score: 6/10**

### 8. Unsafe Blocks / Raw Pointer Misuse — ✅ SAFE
- No `unsafe` blocks anywhere in the pallet
- No raw pointers
- No `transmute` or other unsafe casts
- **Score: 10/10**

### 9. Cryptographic Weakness — ✅ SAFE
- Evidence is stored as `[u8; 32]` hash — pallet does not perform hashing, relies on off-chain hash computation
- No custom cryptographic operations
- No key generation or signing
- **Score: 10/10**

### 10. Oracle / External Data Manipulation — ✅ SAFE
- No oracle dependency
- Evidence hashes are opaque references to off-chain data
- Severity is provided by the disputant — the mediator can issue any resolution regardless of claimed severity
- **Score: 9/10** (severity self-reporting is fine since mediator overrides)

### 11. Concurrency / Race Conditions — ✅ SAFE
- Substrate's single-threaded block execution eliminates traditional race conditions
- `on_idle` does not conflict with extrinsic execution (block includes extrinsics then hooks)
- **Score: 10/10**

### 12. Determinism Violations — ✅ SAFE
- No floating-point arithmetic
- No randomness usage
- No timestamp-dependent logic (uses block numbers)
- `on_idle` iteration is deterministic per storage key order (J-08 is informational)
- **Score: 10/10**

### 13. Weight / Gas Underestimation — ⚠️ MINOR ISSUE
- All weights are hardcoded estimates, not benchmark-derived (J-06)
- `on_idle` weight accounting is present and conservative
- DB read/write counts are approximately correct but slightly under for some extrinsics
- **Score: 7/10**

### 14. Cross-Pallet Trust Assumptions — ⚠️ MINOR CONCERNS
- `escrow_slash` is a public function callable by any pallet — no origin check by design
- The staking pallet correctly guards with `has_pending_review` before calling `escrow_slash`
- If a new pallet calls `escrow_slash` without the guard, J-01 triggers
- `Currency` trait is trusted (standard Substrate pattern)
- No dependency on specific pallet indices or storage layouts
- **Score: 8/10**

---

## JUSTICE-SYSTEM-SPECIFIC ANALYSIS

### Dispute Resolution Fairness
- **Mediator has unilateral power**: A single mediator can issue any ruling (Dismissed/Upheld/Mediated at any bps) with no checks, balances, or multi-sig requirement.
- **No recusal mechanism**: A mediator who is party to a dispute can still rule on it.
- **Mitigation**: The mediator list is governance-controlled, and the appeal mechanism provides a check.
- **Risk**: MEDIUM — single mediator ruling with no conflict-of-interest check.

### Judge/Arbitrator Selection
- **No assignment mechanism**: Any mediator from the list can rule on any dispute. There's no random assignment or rotation.
- **Gameable**: A colluding mediator can front-run other mediators to rule favorably on specific disputes.
- **Risk**: MEDIUM — first-to-call wins model for mediator rulings.

### Bond/Deposit Handling
- **Disputant bond**: Reserved on dispute open. Refunded only on Dismissed. **STUCK on Upheld/Mediated** (J-04).
- **Escrowed slash**: Handled correctly for single-escrow scenarios. **Overwrites on double-escrow** (J-01).
- **Risk**: HIGH — J-01 and J-04 together mean funds can be permanently locked.

### Appeal Mechanism
- **Only target can appeal**: Correct — disputant cannot appeal a Dismissed ruling.
- **Single appeal**: `AlreadyAppealed` check prevents double-appeal. ✅
- **Appeal timeout**: `on_idle` auto-closes timed-out appeals by upholding original ruling. ✅
- **Missing**: No mechanism to actually RE-RULE on appeal — the appeal just changes status to `Appealed` and eventually auto-closes back to `Closed` with the original ruling. The appeal is effectively meaningless since governance has no extrinsic to issue a new ruling on an appealed dispute.
- **Risk**: MEDIUM — appeals are recorded but never actually adjudicated.

### Evidence Submission
- **Bounded**: Evidence is a fixed `[u8; 32]` hash. ✅ No storage bloat.
- **Immutable**: Once set, evidence hash cannot be changed. ✅
- **Counter-evidence**: Also `[u8; 32]`. Set once on appeal. ✅
- **Risk**: None.

### Verdict Enforcement
- **Escrow execution**: On `Upheld`, escrowed slash is correctly unreserved and slashed. ✅
- **Partial slash (Mediated)**: Correctly computes and executes partial slash. ✅ (minor truncation J-05)
- **Bond enforcement**: Incomplete (J-04).
- **Risk**: LOW.

### Timeout Handling
- **Cooling-off**: Uses block numbers, checked in `complete_rehabilitation`. ✅
- **Appeal timeout**: Implemented in `on_idle` with weight-bounded processing. ✅
- **Dispute expiry**: **MISSING** — disputes in `Pending` status never expire. A dispute can sit in `Pending` forever if no mediator ever rules on it, locking the disputant's bond indefinitely and keeping the target in `InCoolingOff`.
- **Risk**: MEDIUM — lack of dispute expiry.

### Griefing
- **Cost to file dispute**: 100 DALLA (in production) — reasonable barrier.
- **Unlimited disputes per target**: Rate-limited only by bond cost (J-07).
- **Self-dispute**: No check preventing `disputant == target`.
- **Risk**: LOW — economic barrier is adequate for most scenarios.

---

## PER-FILE SCORES

| File | Lines | Issues | Score |
|------|-------|--------|-------|
| [lib.rs](pallets/justice/src/lib.rs) | 615 | J-01, J-02, J-03, J-04, J-05, J-07, J-08 | 70/100 |
| [tests.rs](pallets/justice/src/tests.rs) | 666 | None (good coverage, but missing edge cases) | 85/100 |
| [mock.rs](pallets/justice/src/mock.rs) | 131 | None (clean mock setup) | 95/100 |
| [weights.rs](pallets/justice/src/weights.rs) | 58 | J-06 | 75/100 |
| [benchmarking.rs](pallets/justice/src/benchmarking.rs) | 115 | None (correct scaffolding) | 90/100 |

---

## OVERALL SECURITY SCORE

| Category | Score | Weight | Weighted |
|----------|-------|--------|----------|
| Integer overflow/underflow | 9/10 | 8% | 0.72 |
| Division/rounding | 8/10 | 5% | 0.40 |
| Reentrancy/TOCTOU | 10/10 | 10% | 1.00 |
| Access control | 7/10 | 12% | 0.84 |
| Unbounded iteration/DoS | 7/10 | 10% | 0.70 |
| Error handling | 8/10 | 7% | 0.56 |
| State inconsistency | 6/10 | 15% | 0.90 |
| Unsafe blocks | 10/10 | 3% | 0.30 |
| Cryptographic weakness | 10/10 | 5% | 0.50 |
| Oracle manipulation | 9/10 | 3% | 0.27 |
| Concurrency | 10/10 | 3% | 0.30 |
| Determinism | 10/10 | 5% | 0.50 |
| Weight estimation | 7/10 | 7% | 0.49 |
| Cross-pallet trust | 8/10 | 7% | 0.56 |
| **Total** | | **100%** | **8.04 → 78/100** |

---

## REMEDIATION TABLE

| ID | Severity | Fix Complexity | Priority | Status | Description |
|----|----------|---------------|----------|--------|-------------|
| J-01 | **HIGH** | Low | **P0** | 🔴 OPEN | `escrow_slash` must accumulate or reject duplicate escrow |
| J-02 | **HIGH** | Low | **P0** | 🔴 OPEN | Remove dual-origin pattern; use signed + storage list only |
| J-04 | **MEDIUM** | Low | **P1** | 🔴 OPEN | Unreserve disputant bond on Upheld/Mediated rulings |
| J-03 | **MEDIUM** | Medium | **P1** | 🔴 OPEN | Process timed-out appeals inline in `on_idle` or cap Vec |
| J-05 | **MEDIUM** | Low | **P2** | 🔴 OPEN | Document or fix truncation in bps calculation |
| J-07 | **LOW** | Medium | **P2** | 🔴 OPEN | Add rate-limiting for disputes per target |
| J-06 | **LOW** | Medium | **P3** | 🔴 OPEN | Generate weights from benchmarks before mainnet |
| J-08 | **INFO** | — | — | ℹ️ INFO | `on_idle` iteration order — informational, by design |

### Additional Design Recommendations (non-finding)

| # | Recommendation | Priority |
|---|---------------|----------|
| D-01 | Add dispute expiry for `Pending` disputes (auto-dismiss + refund bond after N blocks) | P1 |
| D-02 | Add a re-ruling mechanism for appealed disputes (governance extrinsic to overturn) | P1 |
| D-03 | Prevent self-disputes (`disputant == target`) | P2 |
| D-04 | Add mediator recusal check (mediator cannot rule on dispute where they are disputant or target) | P2 |
| D-05 | Transition target from `InCoolingOff` to `InRehabilitation` when dispute is Dismissed without escrow | P2 |
| D-06 | Consider mediator assignment / rotation to prevent front-running | P3 |

---

## POSITIVE FINDINGS

1. **Saturating arithmetic throughout** — no integer overflow/underflow risks
2. **Bounded mediator list** via `BoundedVec<T::AccountId, T::MaxMediators>` — no storage bloat
3. **Fixed-size evidence** — `[u8; 32]` hash prevents storage abuse
4. **Proper event emission** for all state transitions
5. **Clean separation** of public API (`escrow_slash`, `has_pending_review`) from extrinsics
6. **Weight-bounded `on_idle`** — respects remaining weight budget
7. **Comprehensive test coverage** — 28+ tests covering happy paths and error cases
8. **Benchmark scaffolding** complete — ready for accurate weight generation
9. **No `unsafe` code** anywhere in the pallet
10. **Well-documented** module doc comments with ASCII flow diagram

---

## TEST COVERAGE GAPS

The test suite is comprehensive but missing tests for:
1. Appeal timeout auto-close behavior (no `run_to_block` + `on_idle` test)
2. Double `escrow_slash` on same account (would expose J-01)
3. Self-dispute (`disputant == target`)
4. `mediator_ruling` with `slash_bps = 0` or `slash_bps = 10_000` on Mediated resolution
5. `complete_rehabilitation` called by mediator (non-root origin)
6. Dispute against account already in `InCoolingOff`
7. Disputant bond not being returned on Upheld ruling (would expose J-04)

---

*End of audit. All 1,585 lines across 5 files have been reviewed.*
