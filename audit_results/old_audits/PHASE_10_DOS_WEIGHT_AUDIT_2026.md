# Phase 10: DoS & Weight Accuracy Audit — BelizeChain

**Date:** 2026-07-15
**Auditor:** AI Security Audit (Phase 10 of 11)
**Scope:** All 18 custom pallets, runtime weight/fee configuration, hook implementations
**Methodology:** Static analysis of weight files, hook implementations, iteration patterns, and runtime configuration

---

## Executive Summary

All 17 pallet weight files are **hand-estimated** — none are auto-generated from `frame-benchmarking`. While estimates appear conservative, they are unverified against actual execution costs. Forty-five (45) extrinsics use hardcoded `Weight::from_parts(...)` annotations bypassing the `WeightInfo` trait entirely. Hook implementations (`on_initialize`, `on_idle`) are generally well-bounded with defense-in-depth `.take()` guards, though several `iter_prefix().count()` calls in user-callable extrinsics still perform full storage scans. Runtime `BlockWeights` and fee configuration reflect prior DOS fix applications (DOS-006, DOS-007).

| Severity | Count |
|----------|-------|
| CRITICAL | 1     |
| HIGH     | 3     |
| MEDIUM   | 5     |
| LOW      | 3     |
| INFO     | 4     |

---

## DOS-W-001 — All Weight Files Are Hand-Estimated (HIGH)

**Severity:** HIGH
**Status:** Open
**Affected:** All 17 pallet weight files

Every pallet's `weights.rs` header states:
```
THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
For production, run `frame-benchmarking` to generate accurate weights.
```

**Files:**
- [pallets/belizex/src/weights.rs](../pallets/belizex/src/weights.rs)
- [pallets/bns/src/weights.rs](../pallets/bns/src/weights.rs)
- [pallets/community/src/weights.rs](../pallets/community/src/weights.rs)
- [pallets/compliance/src/weights.rs](../pallets/compliance/src/weights.rs)
- [pallets/consensus/src/weights.rs](../pallets/consensus/src/weights.rs)
- [pallets/governance/src/weights.rs](../pallets/governance/src/weights.rs)
- [pallets/identity/src/weights.rs](../pallets/identity/src/weights.rs)
- [pallets/interoperability/src/weights.rs](../pallets/interoperability/src/weights.rs)
- [pallets/justice/src/weights.rs](../pallets/justice/src/weights.rs)
- [pallets/landledger/src/weights.rs](../pallets/landledger/src/weights.rs)
- [pallets/mesh/src/weights.rs](../pallets/mesh/src/weights.rs)
- [pallets/moderation/src/weights.rs](../pallets/moderation/src/weights.rs)
- [pallets/oracle/src/weights.rs](../pallets/oracle/src/weights.rs)
- [pallets/payroll/src/weights.rs](../pallets/payroll/src/weights.rs)
- [pallets/quantum/src/weights.rs](../pallets/quantum/src/weights.rs)
- [pallets/staking/src/weights.rs](../pallets/staking/src/weights.rs)
- [pallets/whistleblower/src/weights.rs](../pallets/whistleblower/src/weights.rs)

**Risk:** If estimates are too low, blocks can be over-filled, causing validator slowdowns. If too high, legitimate throughput is artificially limited.

**Recommendation:** Run `frame-benchmarking` for all 18 pallets on reference hardware and regenerate weight files using `benchmarking-cli`.

---

## DOS-W-002 — 45 Hardcoded Weight Annotations Bypass WeightInfo (MEDIUM)

**Severity:** MEDIUM
**Status:** Open

Three pallets use inline `Weight::from_parts(...)` in `#[pallet::weight(...)]` annotations instead of routing through `T::WeightInfo::*()`. This means even if benchmarks are later generated, these extrinsics will not pick up the results.

### governance/src/lib.rs — 38 instances

Lines: 3857, 3889, 4013, 4074, 4169, 4247, 4340, 4375, 4426, 4458, 4525, 4652, 4728, 4867, 4954, 5028, 5119, 5203, 5388, 5460, 5490, 5559, 5650, 5712, 5777, 5861, 5922, 6013, 6095, 6170, 6232, 6297, 6348, 6400, 6552, 6603, 6647, 6724, 6754

Example at line 3857:
```rust
#[pallet::weight(Weight::from_parts(25_000_000, 1024)
    .saturating_add(T::DbWeight::get().reads(2))
    .saturating_add(T::DbWeight::get().writes(1)))]
```

### oracle/src/lib.rs — 2 instances

Lines: 1187, 1258

### payroll/src/lib.rs — 5 instances

Lines: 656, 843, 1141, 1186, 1236

**Recommendation:** Replace all 45 instances with `T::WeightInfo::function_name()` calls and add corresponding functions to each pallet's `WeightInfo` trait + weight file.

---

## DOS-W-003 — Unbounded `iter_prefix().count()` in User-Callable Extrinsics (HIGH)

**Severity:** HIGH
**Status:** Partially Mitigated

### governance/src/lib.rs line 5079 — `register_candidate`
```rust
let candidate_count = ElectionCandidates::<T>::iter_prefix(election.id).count() as u32;
```
Full storage scan to count candidates before enforcing `MaxCandidatesPerElection`. As the candidate count approaches the limit, every call to `register_candidate` reads all existing candidates. An attacker who fills the map to `MaxCandidatesPerElection - 1` forces every subsequent registration attempt to iterate the entire set.

**Mitigation:** Maintain a separate `ElectionCandidateCount<T>` `StorageMap<ElectionId, u32>` counter, incrementing on insert and decrementing on removal.

### community/src/lib.rs line 1611 — `attest_community_work`
```rust
let vote_count = PendingAttestations::<T>::iter_prefix(&key).count() as u32;
```
Counts pending attestations via full iteration. Cost grows linearly with attestors.

### community/src/lib.rs line 1768 — `calculate_green_index` (helper)
```rust
let completed_count = CompletedEducation::<T>::iter_prefix(account).count() as u32;
```

### community/src/lib.rs line 1785 — `calculate_green_index` (helper)
```rust
let contributions = GreenContributions::<T>::iter_prefix(account);
```
Unbounded iteration over an account's green contributions.

**Recommendation:** For all four cases, maintain `CounterFor*` storage items that are updated on insert/remove, or use `CountedStorageMap`.

---

## DOS-W-004 — Unbounded `Employees::iter_prefix` in `batch_payment` (MEDIUM)

**Severity:** MEDIUM
**Status:** Partially Mitigated (PR-2 FIX applied)

**File:** `pallets/payroll/src/lib.rs` lines 990–1004

```rust
for (_emp_account, emp) in Employees::<T>::iter_prefix(&employer) {
    if emp.active {
        total_gross = total_gross.saturating_add(emp.salary);
        count = count.saturating_add(1);
    }
}
ensure!(count <= T::MaxEmployees::get(), Error::<T>::MaxEmployeesReached);
```

The iteration occurs *before* the bound check. If an employer has accumulated more employees than `MaxEmployees` (possible if the bound was added after data existed), the entire set is still iterated before the rejection.

Weight annotation at line 982:
```rust
#[pallet::weight(T::WeightInfo::batch_payment(T::MaxEmployees::get()))]
```
The weight charges for `MaxEmployees` iterations, which is correct for the happy path but the iteration cost is incurred even on rejection.

**Recommendation:** Move the count check to a separate pre-validation step using a `EmployeeCount<T>` counter, or iterate with `.take(MaxEmployees + 1)` and fail early.

---

## DOS-W-005 — on_initialize Hooks Analysis (INFO)

**Severity:** INFO
**Status:** Verified Bounded

Five pallets implement `on_initialize`:

| Pallet | File:Line | Behavior | Bounded? |
|--------|-----------|----------|----------|
| economy | lib.rs:416 | bBZD invariant check + annual inflation | Yes — constant-time |
| governance | lib.rs:2373 | Council term expiry (drain bounded to 10) + quarterly audit (take 20) | Yes — `MAX_CHECKS_PER_QUARTER=20` |
| payroll | lib.rs:585 | Returns `Weight::zero()` (work in on_idle) | N/A — AR-10 fix |
| belizex | lib.rs:417 | Returns `Weight::zero()` (work in on_idle) | N/A — AR-10 fix |
| interoperability | lib.rs:722 | Returns `Weight::zero()` (work in on_idle) | N/A — AR-10 fix |

**governance on_initialize detail (line 2413):**
```rust
AccountVoteParticipation::<T>::iter()
    .take(MAX_CHECKS_PER_QUARTER)
```
Bounded to 20 accounts per block — acceptable.

**Assessment:** All `on_initialize` implementations are properly bounded or return zero weight. No DoS vectors identified in hooks.

---

## DOS-W-006 — on_idle Hooks Analysis (LOW)

**Severity:** LOW
**Status:** Verified Weight-Gated

Four pallets implement `on_idle`:

| Pallet | File:Line | Behavior | Weight-Gated? |
|--------|-----------|----------|---------------|
| payroll | lib.rs:595 | Process due payments | Yes — `MaxSchedulesPerBlock`, `remaining_weight` check |
| belizex | lib.rs:421 | One-time dev seed | Yes — requires 50M ref_time minimum |
| interoperability | lib.rs:729 | Finalize expired bridges | Yes — bounded to 20, `remaining_weight` check |
| compliance | lib.rs:456 | Prune stale SAR reports | Yes — bounded to 5 accounts (DOS-017 FIX) |

**Assessment:** All `on_idle` implementations check `remaining_weight` before proceeding and have explicit iteration bounds. The payroll on_idle at line 595 includes DOS-016 FIX for per-scan read cost accounting.

---

## DOS-W-007 — No on_finalize Hooks (INFO)

**Severity:** INFO
**Status:** Verified

No pallets implement `on_finalize`. This is ideal — `on_finalize` runs unconditionally and cannot be cut short by remaining weight. No issues.

---

## DOS-W-008 — `Proposals::iter().take(50)` in `claim_reward` (LOW)

**Severity:** LOW
**Status:** Partially Mitigated

**File:** `pallets/governance/src/lib.rs` line 5587

```rust
let has_proposed = Proposals::<T>::iter()
    .take(50)
    .any(|(_, prop)| prop.proposer == claimer);
```

Bounded to 50 proposals via `.take(50)`, but each iteration decodes a full `Proposal` struct from storage. With 50 proposals, this is approximately 50 DB reads per claim. The hardcoded read cost should match:

Weight annotation at line 5559:
```rust
#[pallet::weight(Weight::from_parts(50_000_000, 2048)
    .saturating_add(T::DbWeight::get().reads(3))
    .saturating_add(T::DbWeight::get().writes(2)))]
```
**Issue:** Annotation charges for 3 reads but the body may perform up to 50+ reads. **Under-weighted.**

**Recommendation:** Either index proposals by proposer (`ProposalsByProposer<T>`) for O(1) lookup, or charge for worst-case 50 reads in the weight annotation.

---

## DOS-W-009 — `DelegationReceivers::get()` Iteration in `calculate_voting_power` (LOW)

**Severity:** LOW
**Status:** Open

**File:** `pallets/governance/src/lib.rs` line 7323

```rust
let delegators = DelegationReceivers::<T>::get(account);
for delegator in delegators.iter() {
    if Self::is_delegation_active(delegator) {
        power = power.saturating_add(1);
    }
}
```

`DelegationReceivers` returns a `BoundedVec` — the iteration is bounded by the vector's max length. However, each `is_delegation_active` call may perform a storage read. The calling extrinsic must account for `O(MaxDelegators)` reads.

**Assessment:** Bounded by `BoundedVec` max length. Verify that calling extrinsics charge for the full iteration cost.

---

## DOS-W-010 — `CouncilMembers::iter().take()` in Helper Functions (INFO)

**Severity:** INFO
**Status:** Verified Bounded

**File:** `pallets/governance/src/lib.rs` lines 6789, 6851

```rust
CouncilMembers::<T>::iter()
    .take(max_council)
    .map(|(_, member)| member.voting_weight)
    .sum()
```

Bounded by `MaxCandidatesPerElection`. Comment at line 6783 (DOS-018) documents the iteration cost requirement. Callers must include the iteration cost in their extrinsic weight.

---

## DOS-W-011 — Runtime BlockWeights Configuration (INFO)

**Severity:** INFO
**Status:** Prior Fixes Verified

**File:** `runtime/src/lib.rs`

| Parameter | Value | Assessment |
|-----------|-------|------------|
| `max_block` ref_time | 2,000,000,000,000 (2s) | Standard for 6s blocks |
| `max_block` proof_size | 5,242,880 (5 MiB) | DOS-006 FIX applied |
| `NORMAL_DISPATCH_RATIO` | 75% | Standard |
| `BlockLength::max` | 5 MiB | Standard |

Fee configuration:
| Parameter | Value | Assessment |
|-----------|-------|------------|
| `TargetBlockFullness` | 25% | Standard |
| `AdjustmentVariable` | 75/1,000,000 | Moderate adjustment speed |
| `MinimumMultiplier` | 1/10 | Fee floor exists (DOS-007 FIX) |
| `MaximumMultiplier` | 10/1 | Fee ceiling prevents overflow |
| `WeightToFee` | IdentityFee | 1:1 mapping, may need production tuning |
| `LengthToFee` | IdentityFee | 1:1 mapping, may need production tuning |

**Assessment:** BlockWeights and fee parameters are within Substrate norms. DOS-006 and DOS-007 fixes verified. `IdentityFee` for both weight-to-fee and length-to-fee is simple but should be calibrated against actual token economics for mainnet.

---

## DOS-W-012 — Top-Level Shared Weight Trait (MEDIUM)

**Severity:** MEDIUM
**Status:** Open

**File:** `pallets/weights.rs` (top-level)

Provides generic weight defaults used by pallets that haven't defined more specific weights:
- `create_transaction`: 50M ref_time, 2048 proof_size
- `update_config`: 25M ref_time, 1024 proof_size
- `emergency_action`: 100M ref_time, 3072 proof_size
- `simple_action`: 10M ref_time, 512 proof_size
- `complex_action`: 75M ref_time, 2560 proof_size

**Risk:** These generic defaults may undercharge or overcharge specific extrinsics that use them. The proof_size values (512–3072 bytes) are relatively small and may undercount for extrinsics that read large storage items.

**Recommendation:** Audit which extrinsics use the top-level `WeightInfo` trait defaults vs. pallet-specific `WeightInfo` implementations. Replace generic weights with pallet-specific ones.

---

## DOS-W-013 — Parameterized Weight Functions Verified (CRITICAL — Prior Fix Verification)

**Severity:** CRITICAL (Residual after prior fixes)
**Status:** Prior Fixes Verified — Underlying Issue Remains

Several weight functions are parameterized by validator/employee count, which is correct for O(N) operations. However, since all weights are hand-estimated, the **per-iteration cost constants are unverified**:

### staking/src/weights.rs — `distribute_rewards`
```rust
fn distribute_rewards(v: u32) -> Weight {
    Weight::from_parts(50_000_000, 3072)
        .saturating_add(Weight::from_parts(5_000_000, 512).saturating_mul(v as u64))
        .saturating_add(T::DbWeight::get().reads(2 + v as u64))
        .saturating_add(T::DbWeight::get().writes(1 + v as u64))
}
```
Assumes 5M ref_time per validator. DOS-002 FIX bounds `v` to `MaxValidators=100`.

### consensus/src/weights.rs — `finalize_consensus_round`
```rust
fn finalize_consensus_round(v: u32) -> Weight {
    Weight::from_parts(100_000_000, 4096)
        .saturating_add(Weight::from_parts(10_000_000, 256).saturating_mul(v as u64))
        .saturating_add(T::DbWeight::get().reads(3 + v as u64))
        .saturating_add(T::DbWeight::get().writes(2 + v as u64))
}
```
Assumes 10M ref_time per validator. DOS-014 FIX applied.

### payroll/src/weights.rs — `batch_payment`
```rust
fn batch_payment(n: u32) -> Weight {
    Weight::from_parts(50_000_000, 2048)
        .saturating_add(Weight::from_parts(15_000_000, 512).saturating_mul(n as u64))
        .saturating_add(T::DbWeight::get().reads(2 + n as u64 * 2))
        .saturating_add(T::DbWeight::get().writes(n as u64 * 2))
}
```
Assumes 15M ref_time per employee. DOS-013 FIX applied.

**Residual Risk:** The per-iteration constants (5M, 10M, 15M ref_time) are guesses. If actual cost is 2–3x higher, blocks containing max-size operations will exceed the 2-second target. This is the **highest-impact residual risk** in the weight system.

**Recommendation:** Benchmark these three functions specifically on reference hardware. These are the most sensitive weights in the system because they scale with participant count and directly affect block execution time.

---

## DOS-W-014 — economy Pallet Has No weights.rs (MEDIUM)

**Severity:** MEDIUM
**Status:** Open

The economy pallet does not have a `weights.rs` file. Its extrinsics either use the top-level generic weight trait or have hardcoded annotations.

**Recommendation:** Create a dedicated `pallets/economy/src/weights.rs` with appropriate weight estimates for all economy extrinsics.

---

## DOS-W-015 — `Weight::zero()` in on_initialize Returns (MEDIUM)

**Severity:** MEDIUM — Context-dependent
**Status:** Partially Mitigated

Three pallets return `Weight::zero()` from `on_initialize` (payroll, belizex, interoperability) as part of AR-10 refactoring that moved work to `on_idle`. The `on_initialize` bodies actually perform some reads (e.g., checking feature flags) before returning zero, which means the block execution budget doesn't account for those reads.

**Assessment:** The actual work performed before returning zero is minimal (1–2 storage reads), so the impact is negligible. However, if `on_initialize` logic grows, this under-accounting could become meaningful.

**Recommendation:** Return a small non-zero weight that accounts for the conditional checks performed before the early return.

---

## Summary of Prior DOS Fixes Verified

| Fix ID | Description | Pallet | Verified |
|--------|-------------|--------|----------|
| DOS-002 | Bound distribute_rewards by MaxValidators | staking | ✅ |
| DOS-003 | Bound slash weight | staking | ✅ |
| DOS-006 | BlockWeights proof_size = 5 MiB | runtime | ✅ |
| DOS-007 | MinimumMultiplier fee floor | runtime | ✅ |
| DOS-013 | Parameterized batch_payment weight | payroll | ✅ |
| DOS-014 | Parameterized finalize_consensus_round | consensus | ✅ |
| DOS-015 | Bounded iteration in on_idle | interoperability | ✅ |
| DOS-016 | Per-scan read cost in payroll on_idle | payroll | ✅ |
| DOS-017 | Bounded SAR pruning to 5/block | compliance | ✅ |
| DOS-018 | Documented council iteration cost | governance | ✅ |
| AR-10  | Move work from on_initialize to on_idle | payroll, belizex, interop | ✅ |
| PR-2   | MaxEmployees bound in batch_payment | payroll | ✅ |

---

## Remediation Priority

| Priority | Finding | Action |
|----------|---------|--------|
| 1 | DOS-W-001 | Run frame-benchmarking for all 18 pallets |
| 2 | DOS-W-013 | Benchmark parameterized weight functions on reference hardware |
| 3 | DOS-W-003 | Replace iter_prefix().count() with CountedStorageMap |
| 4 | DOS-W-002 | Replace 45 hardcoded weights with WeightInfo calls |
| 5 | DOS-W-008 | Fix under-weighted claim_reward read costs |
| 6 | DOS-W-014 | Create economy/src/weights.rs |
| 7 | DOS-W-012 | Audit generic weight trait usage |
| 8 | DOS-W-004 | Add early count check in batch_payment |
| 9 | DOS-W-015 | Return non-zero weight from on_initialize |

---

*End of Phase 10 Audit*
