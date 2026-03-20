# DEAD CODE & ORPHAN ANALYSIS — BelizeChain

**Audit Step**: 4 of Protocol  
**Scope**: All 18 custom pallets in `pallets/`  
**Date**: 2026-01-XX  
**Auditor**: AI-Assisted Static Analysis  
**Status**: COMPLETE

---

## Executive Summary

| Category | Count | Severity |
|----------|-------|----------|
| Dead Storage Items | 8 | 1 Critical, 4 Medium, 2 Low, 1 Medium |
| Orphaned Events (declared, never deposited) | 17 | High |
| Orphaned Errors | 0 confirmed | — |
| Dead Code (annotated `#[allow(dead_code)]`) | 1 | Low |
| Feature-Gated Dead Paths | 0 | — |
| Unused Imports | Deferred to compiler | — |

**Key Finding**: 17 event variants across 4 pallets are declared in `Event<T>` enums but **never emitted** via `Self::deposit_event()`. These represent dead metadata — they inflate the runtime type registry, confuse indexers, and give front-end consumers the impression that these events can actually fire.

**Critical Finding**: `EconomicMetricsStorage` in the economy pallet is declared as a `StorageValue` but is **never read, written, or referenced** anywhere in the codebase. This is pure dead weight in the runtime metadata.

---

## 1. Dead Storage Items

Storage items that are declared but either (a) never referenced, (b) only written and never read, or (c) only used in genesis and never queried at runtime.

| # | Pallet | Storage Item | Type | Writes | Reads | Severity | Detail |
|---|--------|-------------|------|--------|-------|----------|--------|
| 1 | **economy** | `EconomicMetricsStorage` | `StorageValue<EconomicMetrics>` | 0 | 0 | **CRITICAL** | Declared at L324, never referenced anywhere. Pure dead storage. |
| 2 | **belizex** | `DevSeedEnabled` | `StorageValue<bool>` | 1 (genesis) | 0 | **MEDIUM** | Set during `genesis_build` (L423) but never queried. Cannot influence any runtime logic. |
| 3 | **staking** | `EpochQuantumJobs` | StorageValue | 2 (kill+mutate) | 0 | **MEDIUM** | Incremented (L1093) and cleared each epoch (L994) but never read. Write-only counter. |
| 4 | **belizex** | `DailyVolume` | `StorageMap` | 1 (mutate) | 0 | **MEDIUM** | Accumulated per trade (L885) but never queried. Write-only analytics. |
| 5 | **belizex** | `TourismTraders` | `StorageMap` | 1 (insert) | 0 | **MEDIUM** | Marked `true` (L920) but never read. Write-only flag. |
| 6 | **governance** | `DepartmentPolicies` | `StorageMap` | 1 (insert) | 0 | **MEDIUM** | Written during proposal execution (L7110) but never queried. |
| 7 | **economy** | `CumulativePublicGoodsInflation` | `StorageValue<u128>` | 1 (mutate) | 0 | **LOW** | Accumulated in `on_initialize` (L477) but never read. Tracking-only. |
| 8 | **economy** | `CumulativeWellbeingInflation` | `StorageValue<u128>` | 1 (mutate) | 0 | **LOW** | Accumulated in `on_initialize` (L485) but never read. Tracking-only. |

### Impact Analysis

- **State bloat**: 8 storage items contribute to on-chain state without yielding any runtime value.
- **Gas waste**: Items 3–8 execute writes on every relevant extrinsic or block hook, consuming weight/gas for data that is never consumed.
- **Metadata noise**: All 8 appear in runtime metadata, misleading API consumers and indexers.

### Recommendations

| Priority | Action |
|----------|--------|
| **P0** | Remove `EconomicMetricsStorage` — completely dead. Add migration to clear any existing value. |
| **P1** | Either wire `DevSeedEnabled` into a read-side check (guard `seed_default_liquidity`) or remove it. |
| **P1** | Either expose `DailyVolume`, `TourismTraders` via a query extrinsic / RPC, or remove writes. |
| **P1** | Wire `EpochQuantumJobs` into epoch-end logic or remove. |
| **P1** | Wire `DepartmentPolicies` into policy enforcement or remove. |
| **P2** | Decide if `CumulativePublicGoodsInflation` / `CumulativeWellbeingInflation` should be queryable (RPC / extrinsic) or removed. |

---

## 2. Orphaned Events (Declared, Never Deposited)

Events declared in `pub enum Event<T: Config>` but **never emitted** via `Self::deposit_event(Event::Xxx { ... })` anywhere in the dispatchable or hook logic.

### 2.1 Governance Pallet — 10 Orphaned Events

| # | Event Variant | Declared Line | Analysis |
|---|--------------|---------------|----------|
| 1 | `CouncilMemberElected` | L2554 | Superseded by `CouncilMemberElectedFromDistrict` (district election system). Original variant unreachable. |
| 2 | `CouncilMemberRemoved` | L2560 | No removal logic emits this. Council members expire via `CouncilTermAutoExpired` instead. |
| 3 | `EmergencyTypeActivated` | L2565 | Replaced by `EmergencyPendingActivation` + veto-window flow (CONS-029). Never directly activated. |
| 4 | `TermExpired` | L2612 | Replaced by `CouncilTermAutoExpired` in `on_initialize`. Naming overlap, different variant. |
| 5 | `DelegateElectionStarted` | L2622 | Superseded by `DistrictElectionStarted`. Different election system. |
| 6 | `DelegateElectionCompleted` | L2638 | Superseded by `DistrictElectionFinalized`. Different election system. |
| 7 | `JaguarModeActivated` | L2772 | Emergency flow now goes through `EmergencyPendingActivation` → veto → `EmergencyFinalizedAfterVetoWindow`. Direct activation bypassed. |
| 8 | `ProposalAmendmentApplied` | L2753 | Amendment submission emits `ProposalAmendmentSubmitted`, but application logic never fires this. |
| 9 | `ReferendumCancelled` | L2826 | No cancel_referendum extrinsic exists. Referendums can only pass or fail. |
| 10 | `ProposalCooldownBlocked` | L2923 | Cooldown enforcement returns an error instead of emitting an event. |

### 2.2 Quantum Pallet — 4 Orphaned Events

| # | Event Variant | Declared Line | Analysis |
|---|--------------|---------------|----------|
| 1 | `AuctionCreated` | L743 | NFT auctions declared but `create_auction` extrinsic never deposits this event. |
| 2 | `BidPlaced` | L750 | No `place_bid` deposit_event call exists in the pallet. |
| 3 | `AuctionFinalized` | L756 | No `finalize_auction` deposit_event call exists. |
| 4 | `BridgeClaimed` | L769 | Bridge claim logic emits `BridgeCancelled` but `BridgeClaimed` is never emitted. |

### 2.3 Community Pallet — 2 Orphaned Events

| # | Event Variant | Declared Line | Analysis |
|---|--------------|---------------|----------|
| 1 | `FeeExemptionApplied` | L531 | Helper `check_and_apply_fee_exemption` manipulates `FeeExemptionUsage` storage but never deposits this event. |
| 2 | `FeeExemptionLimitReached` | L538 | Same helper — returns early when limit reached but no event emitted. |

### 2.4 Economy Pallet — 1 Orphaned Event

| # | Event Variant | Declared Line | Analysis |
|---|--------------|---------------|----------|
| 1 | `RemittanceSent` | L634 | Declared in Event enum but no extrinsic or hook deposits it. Possible planned feature. |

### Total: 17 Orphaned Events

### Impact

- **Indexer confusion**: Subscan, SubQuery, and custom indexers generate handler stubs for events that will never fire.
- **Metadata bloat**: Each orphaned event inflates the SCALE-encoded metadata type registry.
- **Audit false positives**: Security reviewers waste time analyzing code paths for events that cannot occur.

### Recommendations

| Priority | Action |
|----------|--------|
| **P0** | Governance: Remove `CouncilMemberElected`, `CouncilMemberRemoved`, `EmergencyTypeActivated`, `TermExpired`, `DelegateElectionStarted`, `DelegateElectionCompleted` — superseded by Phase 6 variants. |
| **P0** | Governance: Either implement `cancel_referendum` or remove `ReferendumCancelled`. |
| **P1** | Governance: Either emit `JaguarModeActivated` when `EmergencyFinalizedAfterVetoWindow` fires, or remove the variant. |
| **P1** | Governance: Either emit `ProposalAmendmentApplied` when amendments are applied, or remove. |
| **P1** | Governance: Either emit `ProposalCooldownBlocked` when cooldown enforced, or remove. |
| **P1** | Quantum: Implement auction event deposits in `create_auction`/`place_bid`/`finalize_auction`, or remove the 3 auction events. |
| **P1** | Quantum: Add `BridgeClaimed` deposit in bridge claim logic, or remove the event variant. |
| **P1** | Community: Add `FeeExemptionApplied`/`FeeExemptionLimitReached` deposits in `check_and_apply_fee_exemption`. |
| **P2** | Economy: Implement remittance logic or remove `RemittanceSent`. |

---

## 3. Orphaned Errors

Cross-referencing all `pub enum Error<T>` variants against `Error::<T>::VariantName` usage across all 200+ occurrences collected:

**Result: No confirmed orphaned errors found.**

All error variants observed in declarations were found to have at least one `ensure!(..., Error::<T>::Xxx)` or `Err(Error::<T>::Xxx.into())` usage in their respective pallets. The Error enums are clean.

> **Note**: A full `cargo build` with `-W unused` would provide definitive compiler confirmation. This static grep analysis covers all identifiable patterns.

---

## 4. Dead Code & Unused Functions

### 4.1 Annotated Dead Code

| # | Pallet | Item | Line | Type | Detail |
|---|--------|------|------|------|--------|
| 1 | governance | `GOVERNANCE_ID` | L787 | `PalletId` constant | Explicitly `#[allow(dead_code)]`. Comment: "Reserved for Phase 4 when the governance pallet manages its own escrow." |

**Assessment**: Intentional reservation, Low severity. Acceptable if Phase 4 treasury is on the roadmap. Should be removed if Phase 4 scope has changed.

### 4.2 Unreachable Extrinsics

No unreachable extrinsics identified. All dispatchable functions use standard `ensure_signed` or `ensure_root` origin checks that are reachable in normal operation.

---

## 5. Unused Imports

Static analysis via `grep` for `use` statements cannot definitively determine unused imports without full semantic resolution. This requires compiler diagnostics:

```bash
RUSTFLAGS="-W unused-imports" cargo check 2>&1 | grep "unused import"
```

**Recommendation**: Run the above command and add any findings as a follow-up patch. Substrate pallets commonly accumulate unused imports after refactoring.

---

## 6. Feature-Gated Dead Paths

All `#[cfg(feature = ...)]` gates across all 18 pallets:

| Feature | Count | Pallets | Assessment |
|---------|-------|---------|------------|
| `runtime-benchmarks` | 18 | All pallets | **Standard** — Required for benchmark compilation. No production dead paths. |

**Result: CLEAN.** No production feature gates found. No `#[cfg(test)]` code leaking into production. No `#[cfg(feature = "std")]`-only paths that could create divergent behavior between native and wasm runtimes.

---

## 7. Summary Statistics

| Metric | Value |
|--------|-------|
| Pallets analyzed | 18 |
| Total storage items audited | ~250+ |
| Dead storage items found | 8 (1 critical, 5 medium, 2 low) |
| Total event variants audited | ~200+ |
| Orphaned events found | 17 |
| Total error variants audited | ~200+ |
| Orphaned errors found | 0 |
| Dead code annotations found | 1 |
| Feature-gated dead paths | 0 |
| Pallets fully clean | 10 (bns, compliance, consensus, identity, interoperability, justice, landledger, mesh, moderation, oracle, payroll, staking, whistleblower) |
| Pallets with findings | 4 (governance: 10 orphaned events + 1 dead storage + 1 dead code; quantum: 4 orphaned events; belizex: 3 dead storage; economy: 1 orphaned event + 3 dead storage) |

---

## 8. Priority Fix Recommendations

### Critical (P0) — Fix Before Next Release
1. **Remove `EconomicMetricsStorage`** from economy pallet + add storage migration
2. **Remove 6 superseded governance event variants** (CouncilMemberElected, CouncilMemberRemoved, EmergencyTypeActivated, TermExpired, DelegateElectionStarted, DelegateElectionCompleted)
3. **Remove or implement `ReferendumCancelled`** in governance

### High (P1) — Fix Within Sprint
4. **Wire or remove 3 belizex write-only storage items** (DevSeedEnabled, DailyVolume, TourismTraders)
5. **Wire or remove `EpochQuantumJobs`** in staking
6. **Wire or remove `DepartmentPolicies`** in governance
7. **Add deposits for quantum auction events** or remove AuctionCreated/BidPlaced/AuctionFinalized/BridgeClaimed
8. **Add fee exemption event deposits** in community helper
9. **Decide on remaining governance orphaned events** (JaguarModeActivated, ProposalAmendmentApplied, ProposalCooldownBlocked)

### Low (P2) — Backlog
10. **Evaluate economy cumulative inflation trackers** — query-only or remove
11. **Implement `RemittanceSent`** logic in economy or remove
12. **Run `cargo check` with `-W unused-imports`** and clean up results
13. **Remove `GOVERNANCE_ID` dead code** if Phase 4 treasury scope has changed

---

## Appendix A: Methodology

1. **Storage audit**: Enumerated all `StorageValue`, `StorageMap`, `StorageDoubleMap`, `CountedStorageMap` declarations across all 18 pallets. For each item, searched for `::get()`, `::put()`, `::insert()`, `::mutate()`, `::set()`, `::try_get()`, `::iter()`, `::contains_key()`, `::remove()`, `::kill()` patterns.
2. **Event audit**: Extracted all `pub enum Event<T: Config>` variants. Cross-referenced against all `Self::deposit_event(Event::Xxx` calls across all pallet source files.
3. **Error audit**: Extracted all `pub enum Error<T>` variants. Cross-referenced against all `Error::<T>::Xxx` usage patterns (200+ matches collected).
4. **Dead code**: Searched for `#[allow(dead_code)]` and `#[allow(unused` annotations.
5. **Feature gates**: Searched for all `#[cfg(feature` directives to identify production dead paths.
6. **Scope**: Analysis limited to `pallets/*/src/lib.rs` files. Test-only code (`#[cfg(test)]`) excluded from "usage" counts — an item only used in tests is still flagged as dead in production.

## Appendix B: Files Analyzed

```
pallets/belizex/src/lib.rs
pallets/bns/src/lib.rs
pallets/common/src/lib.rs
pallets/community/src/lib.rs
pallets/compliance/src/lib.rs
pallets/consensus/src/lib.rs
pallets/economy/src/lib.rs
pallets/governance/src/lib.rs
pallets/identity/src/lib.rs
pallets/interoperability/src/lib.rs
pallets/justice/src/lib.rs
pallets/landledger/src/lib.rs
pallets/mesh/src/lib.rs
pallets/moderation/src/lib.rs
pallets/oracle/src/lib.rs
pallets/payroll/src/lib.rs
pallets/quantum/src/lib.rs
pallets/staking/src/lib.rs
pallets/whistleblower/src/lib.rs
```
