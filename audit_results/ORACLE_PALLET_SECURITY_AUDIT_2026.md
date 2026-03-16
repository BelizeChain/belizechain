# BelizeChain Oracle Pallet — Security Audit Report

**Auditor**: AI Security Auditor (Rust/Substrate specialist)
**Date**: 2026-03-15
**Pallet**: `pallet-belize-oracle`
**Files reviewed**: `lib.rs`, `types.rs`, `weights.rs`, `mock.rs`, `tests.rs`, `benchmarking.rs`, `integration_tests.rs`
**Lines reviewed**: ~2,500 (all source files end-to-end)
**Runtime configuration**: `runtime/src/lib.rs` lines 930–943

---

## Executive Summary

The oracle pallet is a **multi-responsibility data oracle** providing price feeds, KYC identity verification, sanctions compliance, merchant verification, land registry, IoT device data ingestion, behavior flagging, and operator rewards.  Previous audit rounds have addressed several issues (O-2, O-4, DOS-009, AR-7, H-29).  This line-by-line review identifies **5 CRITICAL**, **9 WARNING**, and **8 INFO** findings.  The pallet scores **62/100** overall — functional but carrying meaningful attack surface, particularly around oracle manipulation, reward gaming, and cross-pallet trust assumptions.

---

## Per-File Scores

| File | Lines | Score | Notes |
|------|-------|-------|-------|
| [lib.rs](pallets/oracle/src/lib.rs) | ~1,700 | 58/100 | Core logic — most findings here |
| [types.rs](pallets/oracle/src/types.rs) | ~600 | 82/100 | Well-structured, minor overflow risk |
| [weights.rs](pallets/oracle/src/weights.rs) | ~110 | 55/100 | Hardcoded estimates, no benchmarks used |
| [mock.rs](pallets/oracle/src/mock.rs) | ~140 | 70/100 | Testing config weakens invariants |
| [tests.rs](pallets/oracle/src/tests.rs) | ~1,000 | 65/100 | Decent coverage but critical gaps |
| [benchmarking.rs](pallets/oracle/src/benchmarking.rs) | ~30 | 30/100 | Only 2/16 extrinsics benchmarked |
| [integration_tests.rs](pallets/oracle/src/integration_tests.rs) | ~180 | 40/100 | Non-compiling, stale API references |

**Overall pallet score: 62/100**

---

## CRITICAL Findings

### C-1: Operator Count Uses Unbounded `iter().count()` — DoS Vector
**File**: [lib.rs](pallets/oracle/src/lib.rs#L523)
**Lines**: 523–524
```rust
let count = OracleOperators::<T>::iter().count() as u32;
ensure!(count < T::MaxOperators::get(), Error::<T>::TooManyOperators);
```
**Impact**: `OracleOperators` is a `StorageMap`. Calling `iter().count()` reads **all** keys from the trie. With `MaxOperators = 10` this is tolerable today, but the pattern is fundamentally unsafe — if the constant is raised, this becomes a PoV/weight bomb.  The weight declaration accounts for only 1 read + 2 writes.
**Recommendation**: Add a dedicated `StorageValue<u32>` counter (e.g., `OperatorCount`) incremented/decremented on add/remove.  Weight then stays O(1).

---

### C-2: `verify_identity` Finalizes With First Caller's `provider`, `biometric_verified`, `address_verified` — Oracle Sybil Vector
**File**: [lib.rs](pallets/oracle/src/lib.rs#L731-L758)
**Lines**: 731–758
```rust
if finalize {
    let identity_info = IdentityInfo {
        ...
        provider,                 // ← from THIS call's arguments
        biometric_verified,       // ← from THIS call's arguments
        address_verified,         // ← from THIS call's arguments
    };
    IdentityVerifications::<T>::insert(&account, identity_info);
```
**Impact**: Only `kyc_level` and `id_hash` are subject to oracle plurality consensus.  The finalizing oracle unilaterally sets `provider`, `biometric_verified`, and `address_verified`.  A single malicious oracle reaching the quorum threshold last can inject `biometric_verified = false` or a bogus `provider` string, downgrading the effective verification quality stored on-chain.  Cross-pallet consumers (identity, compliance) may rely on these fields for compliance decisions.
**Recommendation**: Either (a) include `provider`/`biometric_verified`/`address_verified` in the consensus tuple, or (b) require all oracles to submit identical metadata (reject if different), or (c) use the first submission's metadata and ignore subsequent overrides.

---

### C-3: `aggregate_price_feed` Called Inside `submit_price` — No Variance Reject Stops Storage Write
**File**: [lib.rs](pallets/oracle/src/lib.rs#L582-L592)
**Lines**: 582–592, 1353–1404
```rust
PriceSubmissions::<T>::insert(pair, &operator, (price, current_block));
Self::aggregate_price_feed(pair)?;   // aggregates and stores
```
The aggregation computes variance but **never rejects** high-variance feeds.  The `HighVariance` error exists in the error enum but is **never returned**.  An operator can submit a price 1000× off from peers.  If `MinConsensusOperators = 3` and one operator submits a wildly deviant price, it is included in the median calculation and will shift the result.
**Impact**: With only 3 operators required in production, a single compromised oracle operator can skew the median by ~33%.  This directly affects DeFi pricing via `get_exchange_rate()` which is consumed by economy, payroll, belizex, and land-ledger pallets.
**Recommendation**: Either (a) reject submissions that deviate >N% from the current running median, or (b) exclude outliers before median computation (e.g., use trimmed median), or (c) implement the variance threshold check and return `HighVariance` when exceeded.

---

### C-4: Reward Calculation Yields Zero for Most Operators — Economic Griefing
**File**: [lib.rs](pallets/oracle/src/lib.rs#L1647-L1680)
**Lines**: 1647–1680
```rust
let avg_domain_multiplier = total_domain_submissions / (total_submissions * 100);
let volume_factor = total_submissions / 1000;
```
**Impact**: Integer division truncation means:
- Any operator with < 1,000 total submissions gets `volume_factor = 0` → reward = 0.
- `avg_domain_multiplier` divides by `total_submissions * 100`.  For `total_submissions = 500` with all AgriTech (multiplier 150), result: `500*150 / (500*100) = 75000/50000 = 1`.  For general submissions (multiplier 100): `500*100 / (500*100) = 1`.  The domain multiplier effectively collapses to 1 for all practical use, negating its purpose.
- Operators who submit fewer than 1,000 data points are economically incentivized to not participate, creating a dead zone.

**Recommendation**: Scale numerators up by 1e6 before division, or use basis-point arithmetic throughout. Ensure operators can earn non-zero rewards proportionally to their actual contributions.

---

### C-5: `MaxDataStaleness = 100 blocks` (~10 Minutes) in Production — Critically Short for Off-Chain Data
**File**: [runtime/src/lib.rs](runtime/src/lib.rs#L936)
**Line**: 936
```rust
type MaxDataStaleness = ConstU32<100>;
```
**Impact**: 100 blocks × 6 seconds = **10 minutes**.  Oracle operators must submit fresh price data every 10 minutes, or `get_exchange_rate()` returns `None`, breaking:
- Economy pallet bBZD issuance
- Payroll BZD↔DALLA conversions
- BelizeX DEX price guardrails
- Land-ledger valuations

If any outage or slow epoch occurs, all oracle-dependent pallets silently fail.  There is no on-chain alert mechanism, and downstream pallets use `Option::unwrap_or()` patterns that may use stale defaults.
**Recommendation**: Increase to at least 300 blocks (30 minutes), ideally 1,800 blocks (3 hours) for a production sovereign chain with limited validator-operators.  Add an `on_initialize` hook that emits warnings when data approaches staleness.

---

## WARNING Findings

### W-1: `add_sanctioned_entity` Overwrites Existing Sanctions Without Check
**File**: [lib.rs](pallets/oracle/src/lib.rs#L647-L668)
**Lines**: 647–668
```rust
SanctionedEntities::<T>::insert(&account, sanction_info);
```
**Impact**: An `AlreadySanctioned` error exists but is **never checked**.  A caller can overwrite a permanent OFAC sanction with a temporary `Other` source sanction that expires soon, effectively circumventing enforcement.  While `add_sanctioned_entity` requires `OracleAdminOrigin`, a compromised council member could exploit this.
**Recommendation**: Add `ensure!(!SanctionedEntities::<T>::contains_key(&account), Error::<T>::AlreadySanctioned)` or explicitly merge/upgrade sanctions rather than replacing them.

---

### W-2: `verify_merchant` Single-Oracle Attestation — No Plurality Required
**File**: [lib.rs](pallets/oracle/src/lib.rs#L596-L636)
**Lines**: 596–636
**Impact**: Unlike `verify_identity` (which requires `MinOracleAgreement` oracles), merchant verification is single-oracle.  A rogue oracle operator can verify arbitrary accounts as tourism merchants, entitling them to 3-8% cashback.  With 10 operators and only 1 needed, the attack surface is wide.
**Recommendation**: Apply the same multi-oracle consensus pattern used for KYC to merchant verification, or at minimum require 2-of-N agreement.

---

### W-3: `verify_merchant` Silently Overwrites Existing Verification
**File**: [lib.rs](pallets/oracle/src/lib.rs#L626)
**Line**: 626
```rust
MerchantCategories::<T>::insert(&merchant, merchant_info);
```
**Impact**: No check for `MerchantAlreadyVerified` (error exists but unused).  An operator can overwrite an `Accommodation` (8%) with `Other` (3%) or vice versa, or reset the expiry clock.
**Recommendation**: Check for existing verification and require explicit re-verification flow.

---

### W-4: `submit_iot_data` Requires No Oracle Operator Status — Any Device Owner Can Submit
**File**: [lib.rs](pallets/oracle/src/lib.rs#L864-L870)
**Lines**: 864–870
```rust
let operator = ensure_signed(origin)?;
let mut device = IoTDevices::<T>::get(device_id).ok_or(Error::<T>::DeviceNotFound)?;
ensure!(device.owner == operator, Error::<T>::NotDeviceOwner);
```
**Impact**: IoT data submission only checks device ownership, **not** oracle operator status.  Any account can register a device and submit data, inflating `OracleOperatorStats` and eventually claiming treasury rewards via `claim_oracle_rewards`.  This is a direct treasury drain vector.
**Recommendation**: Either (a) require `OracleOperators::<T>::get(&operator)` check for IoT data submissions, or (b) separate IoT contributor stats from oracle operator reward eligibility.

---

### W-5: Weight Declarations Are Hardcoded Estimates — Only 2/16 Extrinsics Benchmarked
**File**: [weights.rs](pallets/oracle/src/weights.rs), [benchmarking.rs](pallets/oracle/src/benchmarking.rs)
**Impact**: Only `add_operator` and `remove_operator` have benchmarks.  The other 14 extrinsics use hand-estimated weights.  `submit_price` calls `aggregate_price_feed` which iterates `OracleOperators` (up to `MaxOperators`), but the weight only claims 3 reads.  `verify_identity` does multiple storage reads/writes (dispute flag, pending submissions, leading vote, identity verifications) but its weight claims 1 read + 1 write.
**Recommendation**: Implement benchmarks for all extrinsics, especially `submit_price`, `verify_identity`, `submit_behavior_flag`, and `submit_iot_data`.

---

### W-6: `register_iot_device` Permits Unlimited Registration per Account — Spam Vector
**File**: [lib.rs](pallets/oracle/src/lib.rs#L833-L880)
**Lines**: 833–880
**Impact**: Any signed account can register unlimited IoT devices.  Combined with W-4 (no oracle operator check for IoT data), an attacker can:
1. Register thousands of devices
2. Submit data to each
3. Accumulate operator stats
4. Claim rewards from treasury

The only cost is transaction fees.  No deposit or stake requirement exists.
**Recommendation**: Add per-account device registration limits, or require a bond/deposit per device, or gate registration behind oracle operator status.

---

### W-7: `update_operator_stats` Average Quality Score Vulnerable to Overflow
**File**: [lib.rs](pallets/oracle/src/lib.rs#L1607-L1612)
**Lines**: 1607–1612
```rust
stats.avg_quality_score = ((old_avg * (stats.total_submissions.saturating_sub(1)) as u32 + new_score)
    / stats.total_submissions as u32) as u16;
```
**Impact**: `old_avg` is `u16` (max 65535).  `total_submissions` is `u64`.  The multiplication `old_avg * (total_submissions - 1) as u32` will overflow `u32` when `total_submissions` > ~65,535.  After overflow the average becomes meaningless, potentially allowing an attacker with many low-quality submissions to "reset" their quality score.
**Recommendation**: Perform calculation in `u64`:
```rust
let numerator = (old_avg as u64) * ((stats.total_submissions.saturating_sub(1)) as u64) + (new_score as u64);
stats.avg_quality_score = (numerator / stats.total_submissions).min(1000) as u16;
```

---

### W-8: `is_sanctioned` Does Not Auto-Remove Expired Sanctions
**File**: [lib.rs](pallets/oracle/src/lib.rs#L1452-L1468)
**Lines**: 1452–1468
**Impact**: Expired sanctions remain in storage forever.  While `is_sanctioned()` correctly returns `false` for expired entries, the storage is never cleaned.  Over time this grows unboundedly.  An `on_initialize` or lazy-cleanup pattern is needed if the chain runs for years.
**Recommendation**: Either remove expired sanctions in `is_sanctioned()` (lazy cleanup), or implement an `on_initialize` sweep hook.

---

### W-9: `behavior_flag` Votes Are Not Typed — Operator Can Vote Different Flags for Same Account
**File**: [lib.rs](pallets/oracle/src/lib.rs#L1209-L1218)
**Lines**: 1209–1218
```rust
ensure!(
    !PendingBehaviorFlags::<T>::contains_key(&account, &oracle),
    Error::<T>::AlreadyVotedBehaviorFlag
);
PendingBehaviorFlags::<T>::insert(&account, &oracle, flag_type_u8);
```
**Impact**: Once an operator votes (say flag_type=0), they cannot change their vote.  But if 3 operators are needed and 2 vote for type 0 and 1 votes for type 1, neither reaches consensus.  There is no dispute mechanism (unlike KYC).  Pending votes accumulate permanently until consensus or admin clear.
**Recommendation**: Either allow vote changes (overwrite), or add a timeout/expiry for pending behavior flag votes.

---

## INFO Findings

### I-1: `integration_tests.rs` Does Not Compile
**File**: [integration_tests.rs](pallets/oracle/src/integration_tests.rs)
**Impact**: Uses non-existent APIs (`get_exchange_rate()` without pair argument, `add_sanction()`, `has_encumbrances()`, `is_merchant_verified()`, wrong `verify_identity()` signatures).  This file provides zero test coverage.
**Recommendation**: Either fix to match current APIs or remove.  Currently dead code.

---

### I-2: Mock Config Uses `MinConsensusOperators = 1` and `MinOracleAgreement = 1` — Tests Do Not Reflect Production
**File**: [mock.rs](pallets/oracle/src/mock.rs#L88-L90)
**Lines**: 88–90
**Impact**: Production uses 3 for both.  Tests pass with single-oracle attestation but wouldn't test multi-oracle consensus paths properly.  The `verify_identity_with_quorum` helper partially addresses this but many tests use `MinOracleAgreement = 1`.
**Recommendation**: Set mock `MinOracleAgreement = 2` (matching the genesis operator count of 2) to exercise the consensus path in all tests.

---

### I-3: `BLOCKS_PER_YEAR` Constant Not Used Consistently
**File**: [lib.rs](pallets/oracle/src/lib.rs#L24)
**Line**: 24
```rust
const BLOCKS_PER_YEAR: u32 = 5_256_000;
```
**Impact**: Used for merchant and identity expiry, but `BehaviorCooldownBlocks` is configured separately via a runtime constant.  No validation that `BehaviorCooldownBlocks < BLOCKS_PER_YEAR`.  Minor consistency concern only.

---

### I-4: `IoTDataSubmission` Built But Never Stored
**File**: [lib.rs](pallets/oracle/src/lib.rs#L912-L929)
**Lines**: 912–929
```rust
let _submission = IoTDataSubmission { ... };
```
**Impact**: The IoT data submission record is constructed but prefixed with `_`, indicating it is intentionally unused.  The actual data `payload` parameter is consumed but never persisted.  Only device stats and operator stats are updated.  Data integrity verification is impossible after submission since the `data_hash` is not stored anywhere.
**Recommendation**: Store at least the `data_hash` in device or operator state so cross-referencing is possible.

---

### I-5: No `on_initialize` or `on_finalize` Hooks
**File**: [lib.rs](pallets/oracle/src/lib.rs)
**Impact**: The pallet has no lifecycle hooks.  There is no automated:
- Expiry cleanup for merchants, identities, or sanctions
- Staleness alerting for price feeds
- Behavior flag cooldown auto-clearing
All cleanup is reactive/lazy, which is acceptable but should be documented as a design choice.

---

### I-6: `get_exchange_rate` Emits Events During Read Operations
**File**: [lib.rs](pallets/oracle/src/lib.rs#L1484-L1510)
**Lines**: 1484–1510
**Impact**: `get_exchange_rate()` is a public helper that can be called from other pallets' extrinsics.  It emits `ExchangeRateStale` events.  If multiple pallets call it in the same block, duplicate stale events are emitted.  Events in read-helper functions are unusual and can cause confusion in event indexers.
**Recommendation**: Move staleness event emission to an `on_initialize` hook or a dedicated check extrinsic.

---

### I-7: `PropertyId` is `[u8; 32]` With No Validation
**File**: [types.rs](pallets/oracle/src/types.rs#L295)
**Line**: 295
**Impact**: A `PropertyId` of all zeros is valid.  No namespace/format enforcement exists.  Two oracles could use different encoding schemes for the same physical property, leading to duplicate registrations.
**Recommendation**: Consider a structured format (district_code ++ parcel_number ++ hash) with validation.

---

### I-8: `DataQualityMetrics::calculate_score()` Has Maximum of 1000 But No Minimum Enforcement
**File**: [types.rs](pallets/oracle/src/types.rs#L519-L527)
**Lines**: 519–527
```rust
pub fn calculate_score(&self) -> u16 {
    let weighted_sum = (self.accuracy as u16 * 30) + ...
    weighted_sum / 10
}
```
**Impact**: With all-zero inputs the score is 0.  The `submit_iot_data` extrinsic now validates `accuracy <= 100` etc. (after a prior audit fix), but a score of 0 is accepted and reduces device reputation.  This is likely intentional behavior but could be exploited by a device owner to tank their own reputation score (unclear motive).

---

## Cross-Pallet Trust Analysis

| Consumer Pallet | Oracle API Used | Trust Risk |
|----------------|----------------|------------|
| Economy | `get_exchange_rate()` | **HIGH** — returns `None` after 10 min staleness; downstream handling unclear |
| Identity | `get_kyc_level()`, `meets_kyc_requirement()` | **MEDIUM** — C-2 metadata injection risk |
| Compliance | `is_sanctioned()` | **MEDIUM** — W-1 sanction overwrite risk |
| BelizeX | `get_exchange_rate()` with `MaxOracleDeviationBps=500` | **HIGH** — C-3 price manipulation feeds into DEX |
| Payroll | Oracle provider for salary conversion | **HIGH** — stale rates (C-5) break payroll |
| LandLedger | `verify_land_owner()`, `get_land_ownership()` | **LOW** — static data, one-time write |
| Staking | `OracleVerifier` | **LOW** — verification check only |
| Governance | `has_active_behavior_flag()` | **MEDIUM** — W-9 stuck votes could prevent flagging |

---

## Remediation Priority Matrix

| ID | Severity | Effort | Priority | Description |
|----|----------|--------|----------|-------------|
| C-1 | CRITICAL | Low | P0 | Add operator counter StorageValue |
| C-2 | CRITICAL | Medium | P0 | Include metadata in KYC consensus tuple |
| C-3 | CRITICAL | Medium | P0 | Implement price deviation rejection |
| C-4 | CRITICAL | Medium | P1 | Fix reward calculation integer math |
| C-5 | CRITICAL | Low | P0 | Increase MaxDataStaleness in runtime |
| W-1 | WARNING | Low | P1 | Check existing sanction before overwrite |
| W-2 | WARNING | Medium | P1 | Add multi-oracle merchant verification |
| W-3 | WARNING | Low | P2 | Check existing merchant before overwrite |
| W-4 | WARNING | Medium | P0 | Gate IoT data behind operator status |
| W-5 | WARNING | High | P1 | Implement all benchmarks |
| W-6 | WARNING | Low | P1 | Add device registration limit/bond |
| W-7 | WARNING | Low | P1 | Fix u32 overflow in avg quality calc |
| W-8 | WARNING | Low | P2 | Add lazy cleanup for expired sanctions |
| W-9 | WARNING | Low | P2 | Add timeout for pending behavior votes |

---

## Test Coverage Assessment

| Area | Test Coverage | Gap |
|------|-------------|-----|
| Operator add/remove | ✅ Good | — |
| Price feed submission | ✅ Good | No high-variance test |
| Price aggregation | ✅ Good | No outlier rejection test |
| Staleness detection | ✅ Good | — |
| Merchant verification | ✅ Basic | No expiry re-verification test |
| Sanctions add/remove/expiry | ✅ Good | No overwrite test |
| KYC verification (plurality) | ✅ Good | No conflicting metadata test (C-2) |
| KYC dispute resolution | ❌ Missing | No test for dispute flow |
| Land registry | ✅ Good | — |
| IoT device registration | ✅ Good | No spam/limit test |
| IoT data submission | ✅ Basic | No owner-faking test for stats |
| Oracle rewards claim | ✅ Basic | No edge-case reward amount test |
| Behavior flag voting | ❌ Missing | No tests at all |
| Behavior flag clearing | ❌ Missing | No tests at all |
| Exchange rate manual set | ✅ Basic | — |
| Cross-pallet integration | ❌ Broken | integration_tests.rs doesn't compile |

**Estimated test coverage: ~55%** of critical paths.

---

## Summary

The BelizeChain oracle pallet is architecturally sound with good separation of concerns and a well-implemented KYC plurality mechanism (Phase 3A).  However, it carries **critical risks** in price feed manipulation (C-3), identity metadata injection (C-2), treasury drain via IoT data gaming (W-4 + W-6), and reward calculation arithmetic (C-4).  The 10-minute staleness window (C-5) is operationally dangerous for a sovereign chain.

**Immediate action required**: C-1, C-3, C-5, and W-4 should be addressed before any mainnet launch or DeFi activation.
