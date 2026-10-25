# Changelog

All notable changes to the BelizeChain project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed - Every KYC/sanctions weight was measured on a path production never takes (October 2026)

`runtime/src/lib.rs` carried **15 `#[cfg(feature = "runtime-benchmarks")]` early
returns** inside the provider adapters (`CommunityKycProvider`, `EconomyOracleProvider`,
`BelizeXKycProvider`, `InteroperabilityIdentityProvider`, `InteropOracleCheck`,
`MeshIdentityProviderImpl`, `BnsIdentityProvider`). Under a benchmark build every
KYC, sanctions and oracle-operator gate answered permissively **without touching
storage**, so the generated weights charged for a code path the chain never executes.
That is an under-charge — the exploitable direction — across eight pallets.

The bypasses are gone. Benchmarks now write the **real** state the gate reads, via a
Config hook that follows the idiom already used by `make_council_member` /
`make_mediator` / `make_house_member`:

```rust
#[cfg(feature = "runtime-benchmarks")]
fn seed_bench_kyc(_account: &Self::AccountId, _level: u8) {}
```

Wired in `economy`, `governance`, `community`, `interoperability`, `belizex`,
`landledger`, `bns`, `payroll`. `identity` gained `Identity::bench_seed_kyc`, which
writes `IdentityOf`, `Identities` and the SSN/passport/biometric attestation maps.
Interoperability additionally gained `seed_bench_oracle_operator` for
`confirm_burn_proof`, which gates on `Oracle::OracleOperators`.

**The seeding writes both KYC sources, because pallets consult different ones.**
`Identity` reads its own attestation maps; `Economy` and anything behind
`EconomyOracleProvider` read `Oracle::IdentityVerifications`. Seeding only one leaves the
other gate rejecting the call.

#### Seed the *cheapest* level a gate accepts — not the maximum

This is counter-intuitive and gets it wrong in the unsafe direction.
`Identity::get_verified_kyc_level` probes **L3 → L2 → L1** and stops at the first match,
so read cost is *inversely* related to the account's level:

| Seeded level | `kyc_state` calls | `Identity` reads |
|---|---|---|
| none | 1 (early `None`) | 1 |
| **L1** (just passes `>= 1`) | 3 | **8** |
| L2 | 2 | 7 |
| L3 | 1 | 4 |

A fully-verified L3 account is therefore the **cheapest** caller, not the worst case.
Seeding L3 for a `>= 1` gate under-measures by four reads. `meets_kyc_requirement(who, N)`
is the opposite: it probes only level `N`, so the cost is set by the argument.

Per-gate levels now seeded: governance `can_participate_in_governance` → L1;
interop `get_kyc_level() >= 2` (`initiate_bridge`, `send_message`) → L2; interop
`verify_bridge_operator` → L3; `landledger` → L1; bns `can_register_domain` (L1) and
`can_register_verified` (L3); belizex `is_kyc_ok` → L1; economy/payroll by the
argument their gate passes.

#### Also fixed while scoping

- **`Identity::bench_seed_kyc` hardcoded `id = 1u128`**, so every seeded account
  collided on IdentityId 1 and overwrote the previous record and attestation maps. It
  now allocates through `NextIdentityId` (the counter production uses) and reuses an
  existing id when an account is re-seeded.
- **`governance`'s `eligible_caller` helper never seeded KYC** — 38 call sites, and the
  single root cause of 16 failing benchmarks. It now seeds L1, the most expensive
  *permitted* caller.

#### Measured effect

All eight pallets were regenerated on AC power (`ADP1/online = 1`, `BENCH_MUL=1.25`),
and **no function's read count dropped** — every change is upward:

| Pallet | Entry | Reads | ref_time |
|---|---|---|---|
| `economy` | `transfer_bbzd` | 3 → **11** | +168 % |
| `economy` | `issue_bbzd` | 7 → **11** | +66 % |
| `economy` | `pay_tourism_incentive` | 4 → **9** | +21 % |
| `interoperability` | `acknowledge_message` | 4 → **8** | +106 % |
| `interoperability` | `initiate_bridge` | 7 → **13** | +46 % |
| `interoperability` | `confirm_burn_proof` | 3 → **7** | +122 % |
| `governance` | `submit_proposal` | 3 → **9** | +110 % |
| `governance` | `nominate_for_delegate` | 3 → **9** | +292 % |
| `belizex` | `place_order` | 7 → **15** | +105 % |
| `belizex` | `add_liquidity` | 8 → **16** | +55 % |
| `landledger` | `transfer_property` | 7 → **16** | +68 % |
| `bns` | `register_domain` | 5 → **9** | +54 % |
| `payroll` | `add_employee` | 6 → **8** | +44 % |
| `community` | `endorse_peer` | 8 → **10** | +17 % |

`interoperability::acknowledge_message` was the one entry the previous AC run had to
revert as a known under-charge. It is now **measured**, and its hand-written
`AMENDMENT` block is gone.

The regenerated files record storage the bypasses were hiding —
`Oracle::SanctionedEntities`, `Oracle::IdentityVerifications`,
`Identity::IdentityOf`, `Identity::Identities`, `Oracle::MerchantCategories`.

**Benchmark smoke sweep: 19/19 pallets pass with zero bypasses** (was 11/19 before the
fixes). Workspace suite unchanged at 1,528 passed / 0 failed.

### Added - BelizeX Order Matching Engine (October 2026) ⚠️ STORAGE BREAKING

`pallet-belize-belizex` previously accepted limit orders and never filled them: the
book was an append-only list and no bBZD ever moved. Orders now escrow at placement
and fill against each other in price-time order.

- **Escrow on placement.** `OrderEntry.escrow` holds the funds; bids escrow
  `ceil(amount × price / PRICE_SCALE)`, asks escrow `amount`. `PRICE_SCALE` is
  `1_000_000`, so prices are now scaled and a raw `1_000_000_000_000` is no longer
  "1:1".
- **Matching** runs inside `place_limit_order`, bounded by `T::MaxOrdersPerPair`
  (256 in the runtime), walking `PairOrders` best-price-first. The fill price is the
  **maker's**. Self-trade is **skipped**, so `SelfTradeNotAllowed` is now unraised by
  design rather than by omission.
- **Settlement order is plan → settle → apply.** Book state is written only after the
  funds move. If the second leg is rejected, the first leg is reversed, so an order
  can never claim a fill that did not happen.
- **Rounding:** fills round their quote **down**, placement rounds the escrow **up**.
  Per-slice ceilings summed over an order can exceed the single ceiling taken at
  placement, which stranded a fractional-price bid permanently after ~7 fills.
- **Storage migration required.** `OrderEntry`'s encoding changed, so `STORAGE_VERSION`
  goes 0 → 1 and `pallets/belizex/src/migrations.rs` **purges** the book rather than
  reinterpreting old bytes. `runtime/src/migrations.rs` `CURRENT_RUNTIME_VERSION`
  goes 2 → 3.
- **Weights measured** (`match_orders` 116.4 µs + 57.75 µs/order, reads `8 + n`,
  writes `8 + n`). The benchmark measures the fill shape only, and a walk can consist
  entirely of expired-order purges, so `place_limit_order` additionally charges
  `cancel_order() × MaxOrdersPerPair` at the **call site** — the one place
  `scripts/bench_weights.sh` does not overwrite. Composed at 256: ~25.9 ms.

**Not deployed.** The live chain still runs the stub until this ships as a runtime
upgrade.

### Fixed - mesh: `submit_mesh_transaction` did not charge for signature verification (October 2026)

P1-07 added sr25519 verification to this extrinsic and updated its weight by one
storage **read** for the sender-key lookup — but added nothing for the verification
itself, the most expensive thing the call does. It was charged as if it verified
nothing, which is the exact state P1-07 existed to fix.

Committed raw ref_time was 29.08 ms against a measured 68.84 ms, and that measurement
was taken on a battery-powered host, so the true AC figure is higher still. The
under-charge is real independent of power state. Now committed at 86.05 ms as a
deliberate **upper bound**; tighten with `scripts/bench_weights.sh` on AC power.

`register_mesh_sender_key` had **no benchmark at all**, which left the pallet at 17
`WeightInfo` methods against 16 benchmarks. `bench_weights.sh` requires an exact match
before it regenerates, so it skipped `mesh` entirely and the hand-estimated value could
not be replaced by any number of script runs. The benchmark is added; coverage is now
17/17.

> **Rule:** adding a `#[pallet::call]` without adding its benchmark silently locks that
> pallet out of weight regeneration. Nothing surfaces it until
> `scripts/check_weights_freshness.sh` fires.

### Changed - `bench_weights.sh` refuses to silently delete hand-written weights (October 2026)

The benchmark CLI writes `--output` wholesale, so an `AMENDMENT` block a human added to
a `weights.rs` — a derived weight, extra read/write terms — is deleted by a regeneration
with no warning. This had already happened once, to `belizex`.

The script now fails the pallet unless `ACCEPT_AMENDMENT_LOSS=1` is set. It is
deliberately not a veto: replacing a derived value with a measured one is usually
exactly right, and the guard exists to make that a decision rather than a surprise.

Six pallets carried an amendment — `community`, `governance`,
`interoperability`, `mesh`, `oracle`, `staking`. **All six have since been regenerated on
AC power; four are now measured, and two were reverted** (see below).

> **Do not** "fix" an amendment by moving its derived term to the `#[pallet::weight]`
> call site. The call site already reads `T::WeightInfo::<fn>()`, so an added term would
> stack on the post-regeneration measured value and **double-charge**. The call site is
> for terms the generated file cannot express — a loop bound, a variant count — not for
> values awaiting measurement.

### Changed - Six derived weights replaced with measurements; one rejected as an under-charge (October 2026)

The `ACCEPT_AMENDMENT_LOSS=1` regeneration ran on AC power (`ADP1/online = 1`, cores
verified at 4.6–4.8 GHz). Every entry in the table below except one is now measured:

| Pallet | Entry | Before | After |
|---|---|---|---|
| `community` | `finalize_community_proposal` | 102.60 ms | **91.83 ms** |
| `governance` | `cast_vote` | 62.36 ms | **82.81 ms** |
| `governance` | `reveal_vote` | 57.69 ms | **58.46 ms** |
| `governance` | `unlock_conviction` | 19.80 ms | **35.11 ms** |
| `mesh` | `submit_mesh_transaction` | 86.05 ms | **85.90 ms** |
| `mesh` | `register_mesh_sender_key` | 15.00 ms | **18.11 ms** |
| `staking` | `distribute_rewards` | 39.30 ms | **40.23 ms** |
| `oracle` | `verify_merchant` | 19.49 ms / 1 read | **11.12 ms / 0 reads** |

**One was reverted, because the benchmark measures a cheaper path than production
takes.** `interoperability::acknowledge_message` calls `T::Identity::verify_bridge_operator`
and `T::Identity::is_sanctioned`, and both runtime adapters are bypassed under
`runtime-benchmarks` (`return true` / `return false`). The benchmark build does **2 reads
fewer** than production — measured 2, and the derived 4 it keeps was itself only a partial
count. **Since resolved:** the bypasses were removed on 2026-10-25 and it now measures
8 reads (see the entry at the top of this file).

> **The earlier "all six are biased upward" claim was wrong**, and this run disproved it.
> `unlock_conviction` was derived **44 % under**, `cast_vote` **25 % under**, and
> `register_mesh_sender_key` **21 % under**, while their read counts were overstated. A
> derived weight can be conservative on one axis and optimistic on the other, so upward
> bias has to be checked per axis, not asserted per entry.

> **A benchmark is only as good as the code path it exercises.** `#[cfg(feature =
> "runtime-benchmarks")]` bypasses that remove real storage access, and mock verifiers that
> accept anything, both produce a number that looks measured and **under-charges** — the
> unsafe direction. Treat a *drop* in regenerated read counts as a red flag to explain, not
> an improvement to accept.

> **But an origin is not such a case, and assuming it was cost a wrong revert.**
> `oracle::verify_merchant` benchmarks with `RawOrigin::Root` while the runtime gates it on
> `EitherOfDiverse<EnsureRoot, EnsureProportionMoreThan<TechnicalCouncil, 1, 2>>`. That
> looked like Root short-circuiting past the collective's storage reads. It is not:
> `EnsureProportionMoreThan` matches the origin payload (`RawOrigin::Members(n, m)`) and
> reads no storage at all. Both branches are O(1), so the measured 0 reads / 1 write is
> correct and the derived 19.49 ms / 1 read was over-charging. Check the SDK source before
> reverting a measurement on origin grounds.

### Fixed - Deployment Tooling Targeted a Decommissioned Host (October 2026)

Ten scripts under `scripts/` carried the pre-migration Ceiba address
(`100.81.45.25`, which now runs a dead chain) as either a hardcoded literal or a
`${VAR:-…}` default. A default is exactly what applies when the variable is
unset, so each of these silently targeted the wrong machine:

- **`scripts/upgrade-runtime.py`** — `--rpc` defaulted to the old host. For a
  runtime upgrade that means submitting to the wrong chain. `--rpc` (or
  `CEIBA_RPC_URL`) is now required, and the error names the live endpoint.
- **`scripts/regenesis-ceiba-emptyauth.sh`** — `CEIBA_HOST` defaulted to
  `wicked@ceiba` and three RPC calls hardcoded the address. This script **wipes a
  chain database**, so the target is now required and validated *before* the
  dry-run summary — it previously printed an empty `host :` line and still looked
  like a valid plan.
- **`scripts/register-nawal-signer.js`**, **`scripts/assign-fl-task.js`** —
  dropped the `RPC_ENDPOINT` fallback; unset now exits 2 naming the live endpoint.
- **`scripts/test/fund_founder.py`**, **`scripts/test/pallet_storage_sweep.py`**,
  **`scripts/test/smoke_extrinsic.py`**, **`scripts/test/quick_test.sh`** — dropped
  the literal/default. Unset is reported as its own skip/error rather than
  querying a dead host and producing a misleading result.
- **`scripts/assign_fl_task.py`**, **`scripts/deploy/launch_devpc_validator.sh`** —
  stale example addresses updated.

No script in the repo now embeds a host address; each requires the target
explicitly, so the next host move cannot silently retarget anything.

### Changed - CONS-006: AI Authority M-of-N Gating (October 2026)

`AIAuthorityOrigin` moves from any single TechnicalCouncil member to
`TechnicalCouncilSuperMajority` — a >2/3 council motion, or Root. No single
account can validate models or drive the federated-AI round lifecycle alone
(audit C-CON-2 / CONS-006).

- **Runtime** — `spec_version` 108 → 109
  - `pallet_belize_consensus::Config::AIAuthorityOrigin = TechnicalCouncilSuperMajority`
  - Affects `validate_ai_model`, `start_consensus_round`, `finalize_consensus_round`
  - No storage migration; benchmark smoke path unchanged (originates as `RawOrigin::Root`)

### Changed - ZK Proof Audit & Privacy Remediation (February 2026) ⚠️ BREAKING

Comprehensive audit of zero-knowledge proof usage across all 16 pallets. Removed
fake/placeholder ZK claims, replaced plaintext event data with blake2_256 commitment
hashes, and added structural proof validation. See `docs/security/` for full audit.

- **Staking** (`pallet-belize-staking`) — BREAKING
  - `ModelDelta.zk_proof: BoundedVec<u8, 256>` → `computation_commitment: [u8; 32]`
  - Removed fake byte-length ZK validation (`ensure!(zk_proof.len() >= 32)`)
  - Added structural commitment checks: non-zero, non-uniform bytes, encrypted_delta ≥ 16 bytes
  - Added `evaluate_model_quality()` entropy-aware scoring (delta size + penalty for low-entropy)
  - Error renamed: `InvalidZKProof` → `InvalidComputationCommitment`
  - Docs: "zero-knowledge proofs" → "computation commitments" (honest labeling)
- **Quantum** (`pallet-belize-quantum`) — BREAKING
  - `record_quantum_result()` now validates: proof ≥ 32 bytes, result_hash ≠ all-zeros, accuracy ≤ 100
  - `verify_quantum_result()` docs: clarified as root emergency override, not real ZK verification
  - Tests: All proof_data updated from 10-byte literals to 32-byte commitments; 3 new validation tests
  - 20 tests pass (up from 17)
- **Payroll** (`pallet-belize-payroll`) — BREAKING
  - `Employee` struct: Added `salary_commitment: [u8; 32]` (blake2_256 hash of salary + employer + id)
  - `PayrollRecord` struct: Added `payment_commitment: [u8; 32]`
  - Events no longer emit plaintext amounts:
    - `EmployeeAdded`: `salary` → `salary_commitment`
    - `SalaryUpdated`: `old_salary/new_salary` → `new_commitment`
    - `PaymentExecuted`: `gross_amount/deductions/net_amount` → `payment_commitment`
    - `BatchPaymentCompleted`: `total_amount` → `batch_commitment`
    - `ScheduledPaymentProcessed`: `total_amount` → `batch_commitment`
    - `DeductionUpdated`: `amount` → `deduction_commitment`
    - `BonusIssued`: `amount` → `amount_commitment`
  - Helper functions: `compute_salary_commitment()`, `compute_payment_commitment()`
  - Uses `sp_core::hashing::blake2_256` (not `sp_io` — `no_std` compatible)
  - Known limitation: `Currency::transfer` still emits plaintext in system Transfer event (Substrate constraint; Pedersen commitments roadmapped 2028)
  - 46 tests updated to verify commitment hashes
- **Governance** (`pallet-belize-governance`)
  - Honest docs: "Shielded Voting" → "Roadmapped 2028, NOT YET IMPLEMENTED — current votes are public for accountability"
  - Pre-provisioned `VoteCommitments<T>` storage for future commit-reveal scheme
  - Privacy notes added to `Votes`, `ElectionVotes`, `ReferendumVotes` storage docs
- **Identity** (`pallet-belize-identity`)
  - Clarified ZK selective disclosure roadmap: current model uses salted blake2_256 hashes; true ZK circuits (sp-arkworks) roadmapped for 2028

### Added - Enterprise Payroll Extension (February 2026)
- **Employer Types**: Government, Enterprise, SME, Cooperative, GigPlatform, NonProfit classification
- **Worker Types**: FullTime, PartTime, Contractor, Freelancer, Seasonal, Intern
- **Department Management**: `create_department` extrinsic, per-department scheduling, employee count tracking
- **Automatic Deductions**: IncomeTax, SocialSecurity, Pension, HealthInsurance, Custom — auto-applied on payment
- **Bonus/One-Time Payments**: `issue_bonus` extrinsic with categories (Bonus, Overtime, Commission, Reimbursement, Severance)
- **Employee Status Toggle**: `toggle_employee_status` for suspend/reactivate without removal
- **Security Fix**: `verify_employer` now requires `VerifierOrigin` (root) instead of any signed account
- **Multiple Schedules**: Employers can have per-department payroll schedules (StorageDoubleMap)
- **Net Payment Tracking**: PayrollRecord now tracks deductions and net_amount alongside gross
- **Test Coverage**: 46 unit tests (up from 24), covering all new enterprise features

### Added - Meshtastic Mesh Network Pallet (February 2026)
- **`pallet-belize-mesh`**: Complete Meshtastic LoRa mesh networking pallet
  - **Off-Grid P2P Payments**: Maya Wallet → BLE → Meshtastic radio → LoRa 915 MHz mesh → Gateway → blockchain
  - **Mesh Node Registry**: On-chain registration of Meshtastic nodes (T-Beam, Heltec V3, RAK WisBlock, Station G2)
  - **Node Roles**: Client, Router, Gateway, ValidatorRelay, EmergencyBeacon
  - **Relay Mining**: DALLA rewards for mesh operators relaying transactions, block headers, and emergency alerts
  - **Emergency Broadcast System**: NEMO-integrated hurricane/disaster alerts via mesh with geo-targeting to districts
  - **Validator Mesh Relay**: Compressed block header relay through LoRa for consensus fallback during internet outages
  - **Coverage**: Rural Belize, cayes, jungle communities, disaster zones across 6 districts
  - **Transaction Compression**: 87-byte compressed format fits LoRa 237-byte payload with replay protection
  - **Node Heartbeat Monitoring**: Configurable timeout, reputation scoring, and coverage zone mapping
  - Architecture doc: `docs/architecture/meshtastic-integration.md`
- **Runtime Integration**: `MeshIdentityProviderImpl` with KYC verification, NEMO authority checks, and validator validation
- **Constants**: `MINUTES` block time constant added to runtime

### Added - Pakit DAG Storage (Phase 1 Complete - January 2026)
- **DAG Backend**: SQLite-based persistent storage with LRU caching
  - `backends/dag_backend.py`: 600+ lines, full CRUD operations
  - LRU cache (configurable size, default 1000 blocks)
  - Write-Ahead Logging (WAL) for concurrency
  - Fsync for durability guarantees
  - Backup/restore functionality
  - Query performance: <10ms (hash), <100ms (depth range) ✅
- **DAG Core Layer**: Complete implementation of multi-parent DAG structure
  - `core/dag_storage.py`: DagBlock dataclass, MerkleDAG, genesis block
  - `core/dag_builder.py`: Balanced random parent selection (2-5 parents)
  - `core/dag_index.py`: HashMap (O(1)) + BTreeMap (O(log n)) + Bloom filter
  - Multi-parent Merkle proofs with cryptographic verification
- **Merkle Proof Enhancements**:
  - Delta-encoded proof compression (66.7% reduction in tests)
  - Batch verification with multiprocessing (parallel proof checking)
  - Common ancestor finding (BFS algorithm)
  - Strict mode verification (validates every hash in path)
- **Storage Engine Integration**:
  - DAG as primary backend (sovereign storage)
  - IPFS/Arweave marked as LEGACY (migration fallback only)
  - Hybrid mode: Auto-migration from legacy to DAG on retrieval
  - Comprehensive efficiency reporting (compression, dedup, DAG stats)
- **Performance Benchmarks**: All Phase 1 targets achieved ✅
  - `tests/benchmark_dag.py`: Comprehensive benchmark suite
  - Store 1000 blocks: ~5 seconds (200 blocks/sec)
  - Hash lookup (cache hit): <1ms
  - Hash lookup (cache miss): <10ms ✅
  - Merkle proof verification: <5ms ✅
  - Compression ratio: >3x (ZSTD default) ✅
  - 10K block stress test: ~50 seconds
- **Documentation**:
  - `docs/DAG_DESIGN.md`: 600+ lines comprehensive design doc
  - Architecture diagrams, algorithms, protocols
  - Migration strategy (Phase 1-5 roadmap)
  - Security considerations and threat model
  - Performance characteristics and scalability analysis

### Changed - Pakit Architecture Transformation
- **DEPRECATED**: IPFS and Arweave backends (now legacy-only)
  - No longer primary storage (migration mode only)
  - Target: Complete removal by Month 6 (June 2026)
- **Updated README.md**: Complete rewrite for DAG architecture
  - New mission: "National peer-to-peer file-sharing protocol"
  - DAG architecture as key feature #1
  - Phase 1 completion status (9/9 core components ✅)
  - Updated Quick Start with DAG examples
  - Roadmap updated through 2028

### Fixed - Critical Bugs
- **Deadlock in deduplication.py**: Changed `Lock()` → `RLock()`
  - Root cause: `estimate_space_saved()` called `get_stats()` while holding lock
  - Impact: 10+ minute hangs in efficiency report generation
  - Fix: Reentrant lock allows same thread to acquire lock multiple times
  - Result: Statistics now complete in <1ms ✅
- **Bloom filter optimization**: Reduced default size 1M → 10K
  - Prevents expensive math.exp() calculations in small DAGs
  - Configurable via DagIndex initialization
- **Statistics gathering**: Made O(1) only (removed expensive iterations)
  - Efficiency report now completes instantly
  - All stats components wrapped in try-except for graceful degradation

### Performance Improvements
- **LRU Cache**: 10x speedup on cache hits vs. DB queries
- **DAG Indexing**: O(1) hash lookups, O(log n) depth queries
- **Batch Operations**: Parallel Merkle proof verification with multiprocessing
- **Compression**: ZSTD achieving 3-5x ratios in production

## [0.2.0-alpha] - 2026-01-27 (Phase 1 Release)

**Pakit DAG Storage - Phase 1 Foundation Complete**

This release marks the completion of the DAG (Directed Acyclic Graph) storage foundation for Pakit, achieving complete data sovereignty by eliminating dependencies on external services (IPFS/Arweave).

**Success Criteria** (all met ✅):
- Store 1M blocks successfully
- Query by hash <10ms
- Merkle proof verification <5ms
- Compression ratio >3x
- Zero regressions from legacy backends

**Next Phase**: Phase 2 - Distributed DAG with P2P protocol (Q2-Q3 2026)

## [Unreleased - Previous]
- Economy: Switch bBZD peg from GBP → BZD (Belize Dollar)
  - Governance-controlled peg target via new extrinsic `update_stablecoin_config`
  - Documentation updated across README, runtime guide, developer guide
  - Runtime EconomyOracleProvider continues supplying merchant verification; peg target now read from Economy config


## [0.2.0] - 2025-10-19

### Added

#### Federated Learning Security (Priority 1-5)
- **Byzantine Detection** (Priority 1 - CRITICAL): Complete implementation with 14 tests
  - Cosine similarity-based update verification
  - Gradient norm checking with historical analysis
  - Statistical outlier detection (MAD-based)
  - Weight magnitude verification with history tracking
  - Integrated into honesty scoring (30% weight)
- **Gradient Verification** (Priority 2 - HIGH): Integrated into Byzantine detection
  - Gradient norm analysis for suspicious patterns
  - Historical gradient tracking with 50-item rolling window
- **Data Poisoning Detection** (Priority 3 - HIGH): 21 comprehensive tests
  - Prediction distribution analysis (KL divergence)
  - Loss spike detection with Z-score thresholds
  - Activation pattern analysis for backdoors
  - Feature distribution monitoring
  - Clean data baseline establishment
  - Integrated into honesty scoring (25% weight)
- **Differential Privacy Verification** (Priority 4 - HIGH): 20 comprehensive tests
  - Gradient clipping compliance checking (per-element norm)
  - Privacy budget tracking (epsilon exhaustion detection)
  - Noise consistency verification (deviation analysis)
  - Rolling window storage (30 gradients, 30 noise scales, 50 budget states)
  - Integrated into honesty scoring (20% weight)
- **Data Leakage Detection** (Priority 5 - HIGH): 10 comprehensive tests
  - Membership inference detection (train/val loss gap analysis)
  - Gradient inversion detection (early layer magnitude analysis)
  - Information leakage detection (prediction confidence analysis)
  - Rolling window storage (50 train losses, 50 val losses, 100 confidences)
  - Integrated into honesty scoring (10% weight)

#### Dynamic Honesty Scoring System
- **6-Layer Security Architecture**:
  - Gradient norms: 12% weight
  - Byzantine behavior: 28% weight (highest priority)
  - Weight magnitudes: 10% weight
  - Data poisoning: 22% weight
  - Differential privacy: 18% weight
  - Data leakage: 10% weight (NEW)
- **Graceful Degradation**: Adapts weighting when checks unavailable (6/5/4/3-check modes)
- **Exception Handling**: All checks wrapped with logging fallbacks
- **Total Security Tests**: 65 tests (14+21+20+10) - ALL PASSING

### Changed
- **GenomeTrainer Coverage**: Increased from 52.5% to 71% with security implementations
- **Honesty Score Weighting**: Updated to accommodate 6-check architecture
- **Rolling Window Limits**: Standardized across all detection methods (30-100 items)

### Security
- **Multi-Layer Defense**: 5 independent security checks with weighted scoring
- **Privacy Compliance**: DP verification ensures epsilon/delta budget adherence
- **Attack Resistance**: Comprehensive detection for poisoning, backdoors, and memorization
- **Data Sovereignty**: Leakage detection prevents training data exposure

## [0.1.0] - 2025-10-19

### Added

#### Core Blockchain Infrastructure
- **12 Custom Substrate Pallets** implementing Belize's sovereign blockchain:
  - `pallet-belize-economy`: Multi-currency system (DALLA, bBZD) with multi-signature treasury
  - `pallet-belize-identity`: Decentralized identity with KYC/AML compliance
  - `pallet-belize-governance`: District-based democratic governance system
  - `pallet-belize-compliance`: Legal compliance framework for FSC oversight
  - `pallet-belize-staking`: Proof of Useful Work (PoUW) consensus mechanism
  - `pallet-belize-interoperability`: Cross-chain bridges (Ethereum, Polkadot)
  - `pallet-belize-belizex`: Decentralized exchange for DALLA/bBZD trading
  - `pallet-belize-landledger`: Immutable land registry for property rights
  - `pallet-belize-consensus`: Hybrid consensus with federated AI integration
  - `pallet-belize-oracle`: External data feeds for price oracles and real-world data
  - `pallet-belize-payroll`: Government payroll with automated tax calculation
  - `pallet-belize-quantum`: Quantum computing workload orchestration
  - `pallet-belize-community`: Community proposals with ethics council oversight

#### Federated AI Evolution System (Nawal)
- **Evolution Orchestrator**: Multi-generation genome evolution for AI model training
- **Genome Operators**: Complete mutation and crossover operators including:
  - Layer addition/modification/removal
  - Attention mechanism integration
  - Mixture of Experts (MoE) support
  - State Space Model (SSM) support
  - Architecture optimization
- **Population Management**: Dict-based genome storage with fitness tracking
- **Mock Federated Training**: Deterministic fitness evaluation for testing
- **Blockchain Integration**: Staking rewards tied to federated learning contributions

#### Testing & Quality Assurance
- **148 Python tests** with 100% executable test coverage (98% overall with 3 slow tests skipped)
- **Type hints**: 89-95% coverage across critical modules
- **Async/await**: 69 async functions properly implemented
- **Zero unsafe blocks** in Rust production code
- **Zero memory safety issues** across all pallets

#### Infrastructure & Tooling
- **WASM Runtime**: Parachain-ready with wasm32-unknown-unknown target
- **Runtime Benchmarking**: Complete benchmarking support across all 12 pallets
- **Data Sovereignty**: Compliance filtering for Belizean data protection laws
- **Multi-language UI**: TypeScript interfaces for government, citizens, and developers
- **Development Scripts**: Automated build and deployment tooling

### Changed
- **Fitness Validation**: Clamped fitness scores to [0.0, 100.0] range to prevent validation errors
- **None-Safety**: Added None checks for optional layer properties during mutations
- **Error Handling**: Replaced production `unwrap()` with descriptive `expect()` messages

### Fixed
- **Missing Mutation Handlers**: Implemented `_mutate_add_attention()`, `_mutate_add_moe()`, and `_mutate_add_ssm()`
- **Type Multiplication Error**: Fixed None type handling in layer modification mutations
- **Fitness Overflow**: Prevented fitness values exceeding 100.0 in mock training
- **Frame Benchmarking**: Added missing frame-benchmarking dependency to runtime
- **Runtime Benchmarks Feature**: Added runtime-benchmarks feature to 8 pallets missing it
- **WASM Compilation**: Installed wasm32-unknown-unknown target for runtime compilation

### Security
- **Zero hardcoded secrets**: All sensitive data loaded from environment or configuration
- **No eval/exec usage**: All dynamic code execution safely handled through PyTorch model loading
- **Proper error propagation**: Production unwraps replaced with explicit error messages
- **Data sovereignty compliance**: PII detection and geographic restriction enforcement
- **Byzantine resistance**: Foundation laid for future Byzantine behavior detection

### Technical Debt Eliminated
- ✅ All evolution orchestrator API compatibility issues resolved
- ✅ Mock federated training properly integrated
- ✅ Population management using correct dict-based API
- ✅ All pallets compile with zero errors
- ✅ All dependencies properly declared and featured
- ✅ Type hint coverage at production quality levels

## Future Enhancements

### Planned for v0.3.0
- Enhanced ethics council automation
- Cross-chain interoperability expansion
- Model compression and quantization
- Advanced federated aggregation strategies

---

## Version History

- **v0.2.0** (2025-10-19): Complete federated learning security implementation with 5-layer defense system (65 tests, 100% passing)
- **v0.1.0** (2025-10-19): Initial production-ready release with complete blockchain runtime and federated AI evolution system

## [Unreleased]

[Unreleased]: https://github.com/BelizeChain/belizechain/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/BelizeChain/belizechain/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/BelizeChain/belizechain/releases/tag/v0.1.0
