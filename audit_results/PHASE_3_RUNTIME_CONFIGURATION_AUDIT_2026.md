# Phase 3: Runtime Configuration Audit

**Scope**: `runtime/src/lib.rs` (2,153 lines), `runtime/src/migrations.rs` (241 lines), `runtime/Cargo.toml`  
**Auditor**: AI Full-Spectrum Audit Engine  
**Date**: 2026-07-11  
**Runtime Version**: `spec_version: 103`, `transaction_version: 1`  
**Toolchain**: Rust 1.90.0, Substrate FRAME v42  

---

## Executive Summary

The runtime configuration is architecturally sound with well-structured multi-house governance, proper BABE/GRANDPA/Session integration, conditional Sudo exclusion, PoUW weight decoupling, and comprehensive cross-pallet provider wiring. However, the audit identified **27 findings** including **4 Critical**, **5 High**, **9 Medium**, **5 Low**, and **4 Informational** issues, predominantly centered on systemic decimal denomination errors, missing pallet indices, and security-sensitive defaults in the smart contract platform configuration.

### Severity Breakdown

| Severity | Count |
|----------|-------|
| CRITICAL | 4 |
| HIGH | 5 |
| MEDIUM | 9 |
| LOW | 5 |
| INFO | 4 |
| **TOTAL** | **27** |

---

## Section 1: Parameter Values Audit

### RT-C-1: CRITICAL — Systemic Decimal Denomination Error (9-Decimal vs 12-Decimal)

**Location**: Multiple `parameter_types!` blocks  
**Lines**: 558–559, 562, 800, 885, 995  

**Finding**: The DALLA token is defined as 12 decimals (`DOLLARS = 1_000_000_000_000 = 10^12`) at line 115. However, multiple constants are denominated in 9-decimal math (10^9 base units per token) instead of 12-decimal, producing values 1,000× lower than intended:

| Constant | Value | Comment Claims | Actual (12 dec) | Error |
|----------|-------|----------------|------------------|-------|
| `StakeUnitSize` | `100_000_000_000` (10^11) | "100 DALLA = 100 × 10^9" | **0.1 DALLA** | 1,000× low |
| `TravelRuleThreshold` (economy) | `100_000_000_000` (10^11) | "100 DALLA" | **0.1 DALLA** | 1,000× low |
| `TravelRuleThreshold` (compliance) | `100_000_000_000` (10^11) | "100 DALLA" | **0.1 DALLA** | 1,000× low |
| `RegistrationDeposit` (landledger) | `100_000_000_000` (10^11) | "100 DALLA" | **0.1 DALLA** | 1,000× low |
| `LargeHolderStakeThreshold` | `50_000_000_000_000` (5×10^13) | "50,000 DALLA = 50,000 × 10^9" | **50 DALLA** | 1,000× low |

**Root Cause**: The `StakeUnitSize` comment on line 558 explicitly states "9 decimal places" — the constant author was using 9-decimal math throughout. The `DOLLARS` constant at line 115 proves 12 decimals are canonical.

**Impact**:
- **FATF Travel Rule**: Threshold is 0.1 DALLA instead of 100 DALLA — potentially all transfers trigger compliance reporting (massive false-positive rate) or, if the comparison direction is "amount > threshold", nearly all transfers pass (FATF violation).
- **Quadratic Voting**: 1 vote unit costs 0.1 DALLA instead of 100 DALLA — whales accumulate 1,000× more vote units, completely defeating the quadratic voting fairness mechanism.
- **Large Holder Detection**: Accounts with >50 DALLA are flagged as "large holders" instead of >50,000 DALLA — nearly all active accounts would be penalized.
- **Land Registration**: Only 0.1 DALLA deposit instead of 100 DALLA — trivially cheap property spam.

**Fix**: Replace all affected values with `DOLLARS`-based expressions:
```rust
pub const StakeUnitSize: Balance = 100 * DOLLARS;                      // 100 DALLA
pub const LargeHolderStakeThreshold: Balance = 50_000 * DOLLARS;       // 50,000 DALLA
// In both economy and compliance Config impls:
type TravelRuleThreshold = ConstU128<{ 100 * DOLLARS }>;               // 100 DALLA
// In landledger Config:
type RegistrationDeposit = ConstU128<{ 100 * DOLLARS }>;               // 100 DALLA
```

### RT-C-2: CRITICAL — Smart Contract Deposit Comment Errors Mask Potential Economic Issues

**Location**: `parameter_types!` for `pallet_contracts`, lines 473–474  

**Finding**: The contract storage deposit constants have incorrect comments:
- `DepositPerItem = 1_000_000_000` — comment says "1 DALLA per storage item" but actually = 0.001 DALLA (same as existential deposit). With 12 decimals: `1_000_000_000 / 1_000_000_000_000 = 0.001`.
- `DepositPerByte = 100_000` — comment says "0.0001 DALLA per byte" but actually = 0.0000001 DALLA (10^-7 DALLA).
- `DefaultDepositLimit = 1_000_000_000_000` — comment says "1000 DALLA max deposit" but actually = 1 DALLA.

**Impact**: If the intended deposit was truly 1 DALLA/item, storage spam is 1,000× cheaper than designed. A malicious contract deployer can fill storage at 1/1000th the intended cost. If the low values are intentional for accessibility, the comments mislead auditors.

**Fix**: Either correct the values to match comments or correct the comments to match values. Given the theme of 9-vs-12 decimal confusion, the values are likely intended to be 1,000× higher:
```rust
pub const DepositPerItem: Balance = 1 * DOLLARS;           // 1 DALLA per storage item
pub const DepositPerByte: Balance = DOLLARS / 10_000;       // 0.0001 DALLA per byte
pub const DefaultDepositLimit: Balance = 1_000 * DOLLARS;   // 1,000 DALLA max deposit
```

### RT-C-3: CRITICAL — construct_runtime! Uses Implicit Pallet Indices

**Location**: `construct_runtime!` block, lines 1160–1204  

**Finding**: The `construct_runtime!` macro does not assign explicit pallet indices. Pallets receive auto-assigned indices based on their declaration order. This is dangerous for a blockchain with live state because:

1. **Adding or removing a pallet changes all subsequent indices**, invalidating existing encoded call data, storage prefixes, and events.
2. The `#[cfg(feature = "dev")] Sudo: pallet_sudo` conditional inclusion means that **Sudo's presence or absence shifts the index of every pallet below it**, breaking cross-compilation compatibility between dev and production builds.
3. Any runtime upgrade that reorders pallets will corrupt storage.

**Current ordering** (implicit indices, assuming `dev` feature disabled):
```
System=0, Timestamp=1, Babe=2, Grandpa=3, Balances=4, TransactionPayment=5,
TechnicalCouncil=6, GovernanceCouncil=7, Session=8, Authorship=9, Historical=10,
Offences=11, Contracts=12, Economy=13, Identity=14, Governance=15, Compliance=16,
Staking=17, Oracle=18, Payroll=19, Interoperability=20, BelizeX=21, LandLedger=22,
Consensus=23, Quantum=24, Community=25, Bns=26, Mesh=27, BelizeJustice=28,
BelizeWhistleblower=29, BelizeModeration=30
```

With `dev` feature: Sudo=6 is inserted, shifting TechnicalCouncil to 7, etc.

**Impact**: **State corruption** if pallets are reordered in any upgrade. Dev/production builds produce incompatible storage. Extrinsics encoded against one build will fail on the other.

**Fix**: Assign explicit indices to every pallet:
```rust
construct_runtime!(
    pub struct Runtime {
        System: frame_system = 0,
        Timestamp: pallet_timestamp = 1,
        Babe: pallet_babe = 2,
        Grandpa: pallet_grandpa = 3,
        Balances: pallet_balances = 4,
        TransactionPayment: pallet_transaction_payment = 5,
        #[cfg(feature = "dev")]
        Sudo: pallet_sudo = 99,  // High index, never conflicts
        TechnicalCouncil: pallet_collective::<Instance1> = 10,
        GovernanceCouncil: pallet_collective::<Instance2> = 11,
        Session: pallet_session = 12,
        // ... etc with gaps for future pallets
    }
);
```

### RT-C-4: CRITICAL — `force_from` in inject_pouw_weights() Silently Truncates Authority Set

**Location**: `inject_pouw_weights()`, lines 349–399  

**Finding**: `WeakBoundedVec::force_from()` converts an unbounded `Vec` into a bounded collection, **silently dropping entries** beyond `MaxAuthorities` (100). While logging was added (CONS-022), the truncation still occurs — validators beyond position 100 are silently removed from the BABE authority set without any on-chain event, slashing, or error.

**Impact**: If more than 100 validators join via `pallet_session`, the ones beyond index 100 in `iter_keys()` ordering (which is non-deterministic for `StorageMap`) are silently excluded from block production. They remain staked but cannot author blocks — a loss of funds.

**Additionally**: `BelizeSessionManager::new_session()` also uses `.take(100)` (line 307), which truncates using `iter_keys()` ordering. `StorageMap::iter_keys()` iterates in **hash order** (Blake2_128Concat key hash), which means the truncation is non-deterministic — which 100 validators are selected changes unpredictably.

**Fix**: 
1. Sort validators by stake (descending) before truncating to ensure the top-100 by economic weight are always selected.
2. Emit an event when validators are excluded.
3. Consider rejecting `join_validators` when the set is full instead of silent truncation.

---

## Section 2: Origin Mapping Audit

### RT-H-1: HIGH — `AIAuthorityOrigin = TechnicalCouncilMember` — Single Member Controls AI Consensus

**Location**: `pallet_belize_consensus::Config`, line 1003  

**Finding**: `AIAuthorityOrigin` requires only a single TechnicalCouncil member (`EnsureMember`). This controls AI round management, quality score attestation, and PoUW computation validation. A single compromised council member can manipulate consensus rewards.

**Impact**: A single TechnicalCouncil member can manipulate AI authority operations. With ≤12 members, this is 1-of-12 threshold for consensus-affecting operations.

**Recommendation**: Upgrade to `TechnicalCouncilMajority` (>1/2) for AI authority operations.

### RT-H-2: HIGH — `OracleAdminOrigin = TechnicalCouncilMember` — Single Member Controls Oracle

**Location**: `pallet_belize_oracle::Config`, line 901  

**Finding**: Oracle admin operations (add/remove operators, configure staleness, set parameters) require only a single TechnicalCouncil member. The oracle feeds data used by Economy (merchant verification), Compliance (KYC), Identity, BelizeX (exchange rates), LandLedger, Payroll, and Consensus pallets.

**Impact**: A single compromised council member can manipulate oracle operator sets, affecting every dependent pallet. This is the most impactful single-member origin in the runtime.

**Recommendation**: Upgrade to `TechnicalCouncilMajority`.

### RT-H-3: HIGH — `ReviewerOrigin = TechnicalCouncilMember` and `NawalOracleOrigin = TechnicalCouncilMember` — Single Member Can Rule on Whistleblower Reports and Moderate Content

**Location**: `pallet_belize_whistleblower::Config` (line 1131), `pallet_belize_moderation::Config` (line 1150)  

**Finding**: A single TechnicalCouncil member can:
- Dismiss whistleblower reports (slashing the reporter's bond)
- Verify reports (paying out 2,000–20,000 DALLA rewards from the pool)
- Submit Nawal AI risk scores that auto-queue content for moderation

**Impact**: Conflict of interest — a council member who is the subject of a whistleblower report can dismiss it. A single member can drain the whistleblower pool by approving fabricated reports.

**Recommendation**: Require `TechnicalCouncilMajority` for whistleblower review; keep `TechnicalCouncilMember` for Nawal score submissions (automated oracle role).

### RT-H-4: HIGH — `SetMembersOrigin = EnsureRoot` for Both Councils Persists Beyond Bootstrap

**Location**: `pallet_collective::Config<Instance1>` and `<Instance2>`, lines 680, 700  

**Finding**: `SetMembersOrigin`, `DisapproveOrigin`, and `KillOrigin` for both TechnicalCouncil and GovernanceCouncil remain `EnsureRoot`. The comments state this is "ONLY for bootstrapping initial members via Sudo" with plans for removal, but no on-chain enforcement or timeline exists. When Sudo is removed (dev feature disabled), `EnsureRoot` becomes unreachable **unless a runtime upgrade extrinsic is used**, effectively locking council membership forever.

**Impact**: If Sudo is removed without first setting up a governance pathway for council membership changes, the councils become immutable. No members can be added or removed.

**Fix**: Implement a governance-controlled origin for `SetMembersOrigin` before mainnet (e.g., require both councils to agree for membership changes, or use a dedicated membership pallet).

### RT-H-5: HIGH — `SurveyorOrigin = TechnicalCouncilMember` — Single Member Can Survey Land

**Location**: `pallet_belize_landledger::Config`, line 990  

**Finding**: A single TechnicalCouncil member can approve land surveys, which are prerequisites for property registration and transfer. Combined with the 0.1 DALLA registration deposit (RT-C-1), a single member can enable mass property fraud.

**Recommendation**: Upgrade to `TechnicalCouncilMajority` for surveyor attestation.

---

## Section 3: Feature Gates & Sudo Audit

### RT-M-1: MEDIUM — Contracts Debug/Events Leak in `dev` Builds

**Location**: `impl_runtime_apis!`, Contracts API (lines 2055–2107)  

**Finding**: The `call()` and `instantiate()` runtime APIs expose `DebugInfo::UnsafeDebug` and `CollectEvents::UnsafeCollect` when the `dev` feature is enabled. If a testnet or staging deployment accidentally enables `dev`, contract execution internals (memory layout, gas traces, internal state) are exposed via the RPC.

**Impact**: Information disclosure in non-production deployments that use `dev` feature.

**Recommendation**: This is intentional but should be documented. Consider a separate `debug-contracts` feature flag independent of `dev`.

### RT-M-2: MEDIUM — `pallet_sudo` in `try-runtime` Feature List Without `dev` Guard

**Location**: `runtime/Cargo.toml`, `try-runtime` feature list (line ~192)  

**Finding**: The `try-runtime` feature propagates to `pallet-sudo/try-runtime` unconditionally. While the pallet is only compiled into `construct_runtime!` under `#[cfg(feature = "dev")]`, the dependency is always present in `Cargo.toml`, increasing binary size and dependency surface.

**Impact**: Minor — no security impact but increases attack surface of the dependency tree.

### RT-M-3: MEDIUM — No `#[cfg(feature = "dev")]` Guard on `pallet_sudo` in `std` Feature

**Location**: `runtime/Cargo.toml`, `std` feature list (line ~108)  

**Finding**: `pallet-sudo/std` is unconditionally included in the `std` feature list. Same concern as RT-M-2 — the dependency is always compiled even for production builds.

---

## Section 4: impl Config Blocks Audit

### RT-M-4: MEDIUM — `pallet_babe::WeightInfo = ()` — Zero-Weight Equivocation Reporting

**Location**: `pallet_babe::Config`, line 173  

**Finding**: Despite the comment "S5-5 FIX: use calibrated weights instead of placeholder ()", `WeightInfo` is still `()` (zero weight). This means BABE equivocation reports are free, enabling spam of equivocation proofs. Same issue for GRANDPA (line 185).

**Impact**: An attacker can spam the network with fabricated equivocation proofs at zero weight cost, consuming block space and forcing the offences handler to process every report.

**Fix**: Use `pallet_babe::weights::SubstrateWeight<Runtime>` and `pallet_grandpa::weights::SubstrateWeight<Runtime>`.

### RT-M-5: MEDIUM — `pallet_contracts::Randomness = RandomnessFromOneEpochAgo` — Predictable Contract Randomness

**Location**: `pallet_contracts::Config`, line 488  

**Finding**: Smart contracts receive randomness from one epoch ago (24 hours stale). This is known to be predictable — BABE VRF outputs from the previous epoch are public. Any on-chain game, lottery, or random-selection contract will be vulnerable to prediction attacks.

**Impact**: Any ink! contract using `random()` host function will use predictable randomness.

**Recommendation**: Document this limitation prominently for contract developers. Consider offering a commit-reveal randomness source or VRF-based per-block randomness via a dedicated pallet.

### RT-M-6: MEDIUM — `pallet_contracts::CallFilter = Everything` — No Extrinsic Filtering for Contracts

**Location**: `pallet_contracts::Config`, line 490  

**Finding**: `CallFilter = frame_support::traits::Everything` allows contracts to call **any** runtime extrinsic via `call_runtime`. This means a malicious contract can:
- Call governance extrinsics
- Call staking extrinsics
- Call compliance/identity extrinsics
- Potentially call oracle admin functions if the contract owner is a TechnicalCouncil member

**Impact**: Contracts can bypass the intended origin restrictions by calling extrinsics on behalf of their deployer. This is a permission escalation vector if `call_runtime` is enabled.

**Fix**: Implement a restrictive call filter that only allows safe calls:
```rust
type CallFilter = frame_support::traits::Contains<RuntimeCall>;
// Only allow Balances and TransactionPayment calls from contracts
```

### RT-M-7: MEDIUM — `BelizeSlashHandler::on_offence` Returns `Weight::zero()` — Underweight Slashing

**Location**: `BelizeSlashHandler`, line 260  

**Finding**: The offence handler performs storage reads and writes (calling `slash_validator` which modifies the staking storage map) but returns `Weight::zero()`, claiming no computation occurred. This undercharges the block weight budget.

**Impact**: Slashing operations consume real computational resources but are not metered, potentially causing blocks to exceed their weight limit.

**Fix**: Return an estimate of the actual weight consumed: `T::DbWeight::get().reads_writes(offenders.len() as u64, offenders.len() as u64)`.

### RT-M-8: MEDIUM — `pallet_session::Config::Currency` Set Without Fee Documentation

**Location**: Line 421  

**Finding**: `type Currency = Balances` is set with `type KeyDeposit = ConstU128<0>` (zero deposit). While intentional for a permissioned chain, this means session key registration is free. Combined with unlimited key rotations, an attacker could spam `set_keys` calls to bloat session key storage.

**Recommendation**: Document or add rate limiting for session key changes.

### RT-M-9: MEDIUM — `MaxConsumers = 16` May Be Insufficient

**Location**: `frame_system::Config`, line 152  

**Finding**: `MaxConsumers` limits how many pallets can add provider/consumer references to an account. With 18+ custom pallets plus Balances, Session, Contracts, and 2 Collectives, the theoretical maximum consumers exceeds 16. If an account interacts with >16 pallets simultaneously, the 17th will fail with a `TooManyConsumers` error.

**Impact**: Edge case — most accounts won't interact with all pallets. But validators, council members, and power users could hit this limit.

**Recommendation**: Increase to `ConstU32<32>` or audit all pallets' consumer/provider usage.

---

## Section 5: construct_runtime! Ordering Audit

### RT-L-1: LOW — Missing Pallets from Benchmark Definitions

**Location**: `define_benchmarks!` block, lines 2016–2033  

**Finding**: The following pallets are registered in `construct_runtime!` but missing from `define_benchmarks!`:
- `pallet_session` — session key operations
- `pallet_collective` (both instances) — council voting operations
- `pallet_offences` — equivocation reporting
- `pallet_contracts` — smart contract operations

**Impact**: These pallets cannot have benchmarks run, meaning their weights are entirely from Substrate defaults (which may not match BelizeChain's storage layout).

### RT-L-2: LOW — `Historical` Pallet Placement After `Session`

**Location**: `construct_runtime!`, lines 1184–1185  

**Finding**: `pallet_session::historical` is placed after `Session` in `construct_runtime!`. While this generally works, the `Historical` pallet should ideally be declared immediately after `Session` to ensure its `on_initialize` hook runs in the right order. The current placement appears correct but relies on implicit ordering guarantees.

---

## Section 6: SignedExtension Pipeline Audit

### RT-L-3: LOW — No Compliance Check in Transaction Validation Pipeline

**Location**: `SignedExtra` type alias, lines 1700–1709  

**Finding**: The `SignedExtra` pipeline is:
```rust
pub type SignedExtra = (
    CheckNonZeroSender,
    CheckSpecVersion,
    CheckTxVersion,
    CheckGenesis,
    CheckEra,
    CheckNonce,
    CheckWeight,
    ChargeTransactionPayment,
);
```

This is a standard Substrate pipeline with no custom extensions. Notably, there is **no sanctions check** or **KYC verification** at the transaction validation layer. Sanctioned accounts can submit transactions freely — sanctions are only checked within individual pallet extrinsics.

**Impact**: Sanctioned accounts can still transfer DALLA via `pallet_balances::transfer`, interact with contracts, and participate in staking/governance until individual pallet checks reject them. A system-wide sanctions firewall would prevent any interaction.

**Recommendation**: Consider adding a `CheckSanctionStatus` signed extension that rejects transactions from sanctioned accounts at the pool/validation layer. This is a design decision — per-extrinsic checks are more granular but less comprehensive.

### RT-L-4: LOW — `transaction_version: 1` Has Not Been Bumped

**Location**: `RuntimeVersion`, line 97  

**Finding**: `transaction_version` is still 1. Since `SignedExtra` determines the transaction format and has not changed, this is correct. However, if any signed extension is added (e.g., the sanctions check above), `transaction_version` MUST be bumped.

---

## Section 7: Runtime API Audit

### RT-L-5: LOW — Compliance Runtime API Exposes Raw Internal Data

**Location**: `impl ComplianceApi<Block>`, lines 1970–2010  

**Finding**: The compliance runtime API exposes `AuditRecords` and `SuspiciousActivities` directly via RPC. While this is intentional for compliance reporting, the data includes account-level compliance history and suspicious activity reports that could be sensitive.

**Impact**: Any full node operator can query detailed compliance records for any account via RPC. This may conflict with data privacy regulations (GDPR, Belize Data Protection Act).

**Recommendation**: Consider access control for compliance APIs (e.g., require an API key or restrict to authorized nodes).

---

## Section 8: Migrations Module Audit

### RT-I-1: INFO — Migration Version Uses Non-Standard Storage Key

**Location**: `migrations.rs`, lines 47–60  

**Finding**: The migration version is stored using `sp_core::hashing::twox_128(MIGRATION_VERSION_KEY)` with `frame_support::storage::unhashed`. This is a non-standard pattern — Substrate migrations typically use `StorageVersion` per-pallet. The custom approach works but:
1. Does not integrate with Substrate's `try-runtime` `StorageVersion` checks.
2. The key is not visible in standard Substrate tooling (Polkadot.js, Sidecar).
3. `twox_128` of the raw key produces a 16-byte prefix, which could collide with pallet storage prefixes.

**Recommendation**: Consider migrating to per-pallet `StorageVersion` for alignment with Substrate best practices.

### RT-I-2: INFO — `MigrationStatus` and `MigrationStep` Structs Are Dead Code

**Location**: `migrations.rs`, lines 30–42  

**Finding**: `MigrationStatus` and `MigrationStep` are defined but never used in the `CoordinatedUpgrade` implementation. They appear to be remnants of a planned multi-step migration system that was simplified.

### RT-I-3: INFO — `StorageMigration<T>` Template Is Never Instantiated

**Location**: `migrations.rs`, lines 80–98  

**Finding**: The `StorageMigration<T>` struct provides `needs_migration()` and `post_upgrade_check()` helpers but is never used by any actual migration. It exists only as documentation/template.

---

## Section 9: Cross-Pallet Provider Audit

### RT-I-4: INFO — 14 Provider Structs with Significant Duplication

**Location**: Lines 1290–1700  

**Finding**: The runtime defines 14 cross-pallet provider structs, many with duplicated logic:
- 6 providers implement KYC checks in slightly different ways
- 4 providers implement sanctions checks
- 3 providers wrap the same `Identity::get_verified_kyc_level` call
- The `providers` module (lines 1290–1345) was created to consolidate, but not all providers use it

**Providers using consolidated `providers::` module**: `EconomyOracleProvider`, `IdentityOracleProvider`, `PayrollOracleProvider`, `StakingIdentityProvider`, `GovernanceComplianceProvider`, `LandLedgerOracleProvider`.

**Providers NOT using consolidated module** (still inline): `CommunityKycProvider`, `BelizeXKycProvider`, `InteroperabilityIdentityProvider`, `MeshIdentityProviderImpl`, `BnsIdentityProvider`.

**Impact**: Inconsistency — some providers check `Oracle::is_sanctioned()` directly, others use `providers::runtime_is_sanctioned()` which checks both Identity AND Oracle. This means:
- `InteroperabilityIdentityProvider::is_sanctioned` only checks Identity pallet
- `BnsIdentityProvider::is_sanctioned` only checks Oracle pallet
- `StakingIdentityProvider::is_sanctioned` checks both (via providers module)

**Recommendation**: Migrate all providers to use the consolidated `providers::` module for consistency.

---

## Section 10: Positive Findings

1. **Sudo Feature Gate** (CONS-004): `pallet_sudo` correctly excluded from production via `#[cfg(feature = "dev")]`, with matching guards in `construct_runtime!`, benchmarks, and contract debug output.

2. **PoUW Weight Decoupling** (S6-1): `inject_pouw_weights()` correctly sets all BABE authority weights to 1, preventing quality_score gaming from affecting consensus slot selection.

3. **Session/BABE/GRANDPA Integration**: `BelizeSessionManager` correctly bridges `pallet_belize_staking::Validators` to `pallet_session`, enabling on-chain validator rotation with bounded enumeration (`.take(100)`).

4. **Equivocation Infrastructure**: Full BABE + GRANDPA equivocation reporting pipeline with `BelizeSlashHandler` forwarding to `pallet_belize_staking::slash_validator`. Historical session proofs retained for 7 set IDs.

5. **Congestion-Responsive Fees** (DOS-007): `TargetedFeeAdjustment` with 25% target fullness, bounded multiplier range [0.1, 10.0] — prevents both fee collapse and fee explosion.

6. **Block Weight/Length Limits**: Proof-of-validation (PoV) size correctly set to 5 MiB (matching BlockLength), fixing the previous `u64::MAX` that disabled PoV metering entirely (DOS-006).

7. **Multi-House Governance**: TechnicalCouncil (≤12) and GovernanceCouncil (≤32) with proper graduated threshold system: Member < Majority < SuperMajority < ThreeQuarters.

8. **Seed Validation** (CONS-038): `generate_session_keys()` validates seed length ≥32 bytes, preventing DoS via short seeds causing panics.

9. **Migration Framework**: `CoordinatedUpgrade` uses per-step version advancement with `try-runtime` pre/post upgrade hooks. Version guards prevent re-running completed migrations.

10. **Benchmark KYC Bypass**: Consistent `#[cfg(feature = "runtime-benchmarks")]` guards in all provider structs return permissive values, enabling benchmarks to run without KYC scaffolding.

---

## Remediation Priority Matrix

### P0 — Fix Before Mainnet

| ID | Finding | Effort |
|----|---------|--------|
| RT-C-1 | Decimal denomination errors (0.1 DALLA vs 100 DALLA) | Low (~10 lines) |
| RT-C-2 | Contract deposit values 1,000× too low | Low (~3 lines) |
| RT-C-3 | Explicit pallet indices in construct_runtime! | Medium (~30 lines) |
| RT-H-4 | Council SetMembersOrigin pathway for post-Sudo | High (design) |

### P1 — Fix Before Testnet Public Launch

| ID | Finding | Effort |
|----|---------|--------|
| RT-C-4 | Deterministic validator set ordering (sort by stake) | Medium |
| RT-H-1 | AIAuthorityOrigin → TechnicalCouncilMajority | Low |
| RT-H-2 | OracleAdminOrigin → TechnicalCouncilMajority | Low |
| RT-H-3 | ReviewerOrigin → TechnicalCouncilMajority | Low |
| RT-H-5 | SurveyorOrigin → TechnicalCouncilMajority | Low |
| RT-M-4 | BABE/GRANDPA WeightInfo → SubstrateWeight | Low |
| RT-M-6 | Contract CallFilter restriction | Medium |
| RT-M-7 | BelizeSlashHandler weight accounting | Low |

### P2 — Fix Before Mainnet

| ID | Finding | Effort |
|----|---------|--------|
| RT-M-5 | Document predictable contract randomness | Low |
| RT-M-8 | Session key spam mitigation | Low |
| RT-M-9 | MaxConsumers audit/increase | Low |
| RT-L-3 | Sanctions SignedExtension (design decision) | High |
| RT-I-4 | Provider consolidation | Medium |

### P3 — Maintenance

| ID | Finding | Effort |
|----|---------|--------|
| RT-M-1 | Contracts debug feature separation | Low |
| RT-M-2 | Cargo.toml pallet-sudo try-runtime guard | Low |
| RT-M-3 | Cargo.toml pallet-sudo std guard | Low |
| RT-L-1 | Add missing benchmarks to define_benchmarks! | Medium |
| RT-L-2 | Verify Historical pallet hook ordering | Low |
| RT-L-4 | Bump transaction_version when SignedExtra changes | N/A |
| RT-L-5 | Compliance API access control | Medium |
| RT-I-1 | Migrate to per-pallet StorageVersion | Medium |
| RT-I-2/3 | Remove dead migration code | Low |

---

## Cross-Reference with Phase 2 Findings

Several Phase 3 findings confirm and explain Phase 2 pallet-level issues:

| Phase 2 ID | Phase 3 Confirms | Detail |
|------------|------------------|--------|
| Economy M-6 | RT-C-1 | Travel rule threshold decimal inconsistency traced to runtime constant |
| Staking P2 §2.1 | RT-C-4 | Validator set truncation ordering now identified as non-deterministic |
| Governance §1.2 | RT-H-4 | Council membership immutability after Sudo removal |
| Oracle §3.1 | RT-H-2 | Single-member oracle admin traced to runtime origin mapping |
| Consensus §2.3 | RT-H-1 | Single AI authority centralization traced to runtime origin |
| BelizeX P2 §2.5 | RT-M-6 | Contract call filter allows bypassing DEX restrictions |
| All pallets (weight) | RT-M-4 | BABE/GRANDPA zero-weight pattern same as pallet zero-weight issue |

---

## Summary Statistics

| Category | Finding |
|----------|---------|
| Parameters audited | 52 constants across 6 parameter_types! blocks |
| Pallets in construct_runtime! | 25 (26 with dev Sudo) |
| Origin types defined | 7 (Root, 4 TechnicalCouncil tiers, 2 GovernanceCouncil tiers) |
| Cross-pallet providers | 14 structs implementing 17 trait methods |
| Runtime APIs implemented | 12 (Core, Metadata, BlockBuilder, TaggedTransactionQueue, OffchainWorker, BabeApi, SessionKeys, GrandpaApi, AccountNonce, TransactionPayment, ComplianceApi, ContractsApi) |
| SignedExtension checks | 8 (standard Substrate pipeline) |
| Migration steps | 1 (V0→V1 bootstrap, no-op) |
| Decimal denomination errors | 5 constants using 9-decimal math in a 12-decimal system |
