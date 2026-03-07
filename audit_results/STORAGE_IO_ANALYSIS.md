# BelizeChain Storage Read/Write Analysis

**Scope**: Every dispatchable function across all 11 pallets  
**Methodology**: Manual trace of storage access patterns in source  
**Date**: 2025-06-17  

**Legend**:
- **R** = Storage read (`::get`, `::contains_key`, `::iter`, getter via trait)
- **W** = Storage write (`::insert`, `::put`, `::mutate`, `::try_mutate`, `::remove`, `::kill`, `::clear`)
- **Cx** = Currency / external-pallet call (`transfer`, `reserve`, `unreserve`, `set_lock`, `remove_lock`, `deposit_creating`, `repatriate_reserved`, `withdraw`, `slash`, `free_balance`)
- Reads inside `::mutate` / `::try_mutate` count as **1 R + 1 W**
- `deposit_event` counts as **1 W**
- `(cond)` = conditional path; worst-case counted
- `(×N)` = per-item in bounded iteration

---

## 1. LANDLEDGER (`pallets/landledger/src/lib.rs` — 944 lines)

### WeightInfo Trait
```rust
fn register_property() -> Weight;
fn transfer_property() -> Weight;
fn verify_property() -> Weight;
fn survey_property() -> Weight;
fn register_surveyor() -> Weight;
```

### Storage Items
`Properties`, `PropertyOwners`, `TransferRecords`, `PropertyByTitle`, `NextPropertyId`, `NextTransferId`, `GovernmentSurveyors`, `ZoningMap`, `LandAnchors`, `PropertyAnchorChain`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `register_property` | 3 (`PropertyByTitle::contains_key`, `NextPropertyId::get`, `ZoningMap::get`) | 6 (`Properties::insert`, `PropertyByTitle::insert`, `PropertyOwners::try_mutate`→W, `NextPropertyId::put`, `PropertyAnchorChain::insert`, `LandAnchors::insert`) + 1 event | `reserve` ×1 | 40M | Anchor chain insert is conditional |
| 1 | `transfer_property` | 3+4 ext (`Properties::get`, `PropertyAnchorChain::get`, `NextTransferId::get`; Oracle×4: `verify_land_owner`, `is_sanctioned`×2, `get_kyc_level`) | 7 (`Properties::insert`, `TransferRecords::insert`, `PropertyOwners::mutate`×2, `NextTransferId::put`, `PropertyAnchorChain::insert`, `LandAnchors::insert`) + 1 event | `transfer`(tax) ×1 | 80M | 4 cross-pallet reads (Oracle) |
| 2 | `verify_property` | 1 R+W (`Properties::mutate` → read+write) | 0 + 1 event | — | 20M | Minimal |
| 3 | `survey_property` | 2+1 R+W (`GovernmentSurveyors::get`, `Properties::mutate`→R+W, `ZoningMap::get` cond) | 0 + 1 event | — | 25M | Conditional zoning check |
| 4 | `register_surveyor` | 0 | 1 (`GovernmentSurveyors::insert`) + 1 event | — | 15M | Root-only, very light |

**Totals**: 5 extrinsics, max 7R + 8W + 1 currency per call

---

## 2. CONSENSUS (`pallets/consensus/src/lib.rs` — 984 lines)

### WeightInfo Trait
```rust
fn register_ai_model() -> Weight;
fn join_validator() -> Weight;
fn validate_model() -> Weight;
fn start_consensus_round() -> Weight;
fn submit_ai_work() -> Weight;
fn finalize_consensus_round() -> Weight;
```

### Storage Items
`AIModels`, `ModelsByAccount`, `NextModelId`, `ConsensusValidators`, `ValidatorByAccount`, `NextValidatorId`, `ConsensusRounds`, `CurrentConsensusRound`, `NextRoundId`, `GlobalAIMetrics`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `register_ai_model` | 1 (`NextModelId::get`) | 4 (`AIModels::insert`, `ModelsByAccount::try_mutate`→W, `NextModelId::put`, `GlobalAIMetrics::mutate`→R+W) + 1 event | — | 25M | |
| 1 | `join_consensus_validator` | 2 (`ValidatorByAccount::contains_key`, `GlobalAIMetrics::get`) + 1 (`NextValidatorId::get`) | 4 (`ConsensusValidators::insert`, `ValidatorByAccount::insert`, `NextValidatorId::put`, `GlobalAIMetrics::mutate`→R+W) + 1 event | `set_lock` ×1 | 35M | |
| 2 | `validate_ai_model` | 1 R+W (`AIModels::mutate`→R+W) | 0 + 1 event; iterates `AIModels` for metrics | — | 60M | Unbounded iteration in `update_global_quality_metrics` |
| 3 | `start_consensus_round` | 2 (`CurrentConsensusRound::get`, `NextRoundId::get`) | 3 (`ConsensusRounds::insert`, `CurrentConsensusRound::put`, `NextRoundId::put`) + 1 event | — | 40M | Iterates `ConsensusValidators` |
| 4 | `submit_ai_work` | 3 (`ValidatorByAccount::get`, `CurrentConsensusRound::get`, `AIModels::get`) | 3 R+W (`ConsensusRounds::try_mutate`, `ConsensusValidators::mutate`, `AIModels::mutate`) + 1 event | — | 35M | |
| 5 | `finalize_consensus_round` | 2 + N×1 (`CurrentConsensusRound::get`, `ConsensusValidators::get` per submission) | 4 + N×1 (`ConsensusRounds::try_mutate`, validators×N mutate, `CurrentConsensusRound::kill`, `GlobalAIMetrics::mutate`) + 2 events | `deposit_creating` ×N | 100M+ | O(N) validator rewards; bounded by validator count |

**Totals**: 6 extrinsics, heaviest is `finalize_consensus_round` with O(N) iteration

---

## 3. INTEROPERABILITY (`pallets/interoperability/src/lib.rs` — 1176 lines)

### WeightInfo Trait
```rust
fn initiate_bridge() -> Weight;
fn provide_signature() -> Weight;
fn create_liquidity_pool() -> Weight;
fn process_unlock() -> Weight;
fn send_message() -> Weight;
fn update_config() -> Weight;
// Note: dispute_bridge_transaction reuses update_config() weight
```

### Storage Items
`BridgeTransactions`, `BridgeValidators`, `ChainConfigurations`, `TotalLockedAssets`, `LiquidityPools`, `CrossChainMessages`, `PendingFinalizations`, `NextTxId`, `NextPoolId`, `NextMessageId`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `initiate_bridge` | 2 + 2 ext (`ChainConfigurations::get`, `NextTxId::get`; Identity: `get_kyc_level`, `is_sanctioned`) | 3 (`BridgeTransactions::insert`, `NextTxId::put`, `TotalLockedAssets::mutate`→R+W) + 2 events | `set_lock` + `transfer`(fee) | 60M | Cross-pallet Identity calls |
| 1 | `provide_pq_signature` | 2 + 2 ext (`BridgeValidators::get`, `BridgeTransactions::get`; Identity: `verify_bridge_operator`, `is_sanctioned`) | 2 (`BridgeTransactions::insert`, `PendingFinalizations::insert` cond) + 1 event | — | 40M | Conditional finalization |
| 2 | `create_liquidity_pool` | 1 + 1 bal (`NextPoolId::get`, `Currency::free_balance`) | 2 (`LiquidityPools::insert`, `NextPoolId::put`) + 1 event | `reserve` ×1 | 30M | |
| 3 | `process_unlock` | 1 (`BridgeValidators::get`) | 1 (`TotalLockedAssets::mutate`→R+W) + 1 event | `remove_lock` + `deposit_creating` | 35M | |
| 4 | `send_cross_chain_message` | 1 (`NextMessageId::get`) | 2 (`CrossChainMessages::insert`, `NextMessageId::put`) + 1 event | — | 20M | Lightest bridge op |
| 5 | `update_bridge_config` | 0 | 1 R+W (`ChainConfigurations::mutate`) + 1 event | — | 15M | Admin only |
| 6 | `dispute_bridge_transaction` | 3 (`BridgeValidators::contains_key`, `BridgeTransactions::get`, `PendingFinalizations::contains_key`) | 2 (`BridgeTransactions::insert`, `PendingFinalizations::remove`) + 1 event | — | 30M | |

**Totals**: 7 extrinsics

---

## 4. BELIZEX (`pallets/belizex/src/lib.rs` — 1213 lines)

### WeightInfo Trait
```rust
fn create_pair() -> Weight;
fn add_liquidity() -> Weight;
fn execute_trade() -> Weight;
fn register_tourism_trader() -> Weight;
fn place_order() -> Weight;
fn pause() -> Weight;
fn resume() -> Weight;
fn set_pair_status() -> Weight;
// Note: execute_multihop_trade reuses execute_trade() weight
```

### Storage Items
`TradingPairs`, `LiquidityProviders`, `TourismTraders`, `DailyVolume`, `OrderBook`, `NextOrderId`, `GlobalPaused`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `create_trading_pair` | 1 (`TradingPairs::contains_key`) | 1 (`TradingPairs::insert`) + 1 event | — | 15M | Admin |
| 1 | `add_liquidity` | 3 + cond (`GlobalPaused::get`, `TradingPairs::get`, `Currency::free_balance`, `TourismTraders` cond) | 2 (`TradingPairs::insert`, `LiquidityProviders::mutate`→R+W) + 1 event | `transfer` ×1 | 40M | |
| 2 | `execute_trade` | 3 + 3 ext (`GlobalPaused::get`, `TradingPairs::get`, `TourismTraders` cond; Oracle: `get_crypto_exchange_rate`, `get_trading_volume_tier`, `is_tourism_merchant`) | 2 (`TradingPairs::insert`, `DailyVolume::mutate`→R+W) + 1 event | `transfer` ×3 (trade + treasury fee) | 80M | KYC check + 3 Oracle calls |
| 3 | `register_tourism_trader` | 0 | 1 (`TourismTraders::insert`) + 1 event | — | 10M | TourismOrigin |
| 4 | `place_limit_order` | 4 (`GlobalPaused::get`, `TradingPairs::contains_key`, `TourismTraders` cond, `NextOrderId::get`) | 2 (`OrderBook::insert`, `NextOrderId::put`) + 1 event | — | 30M | KYC check |
| 5 | `execute_multihop_trade` | 2 + H×2 (`GlobalPaused::get`, `TradingPairs::get` ×H hops, Oracle cond ×H) | H×1 (`TradingPairs::insert` per hop) + 1 event | `transfer`(treasury) ×1 | 120M | H = 2–5 hops; O(H) reads/writes |
| 6 | `pause_global` | 0 | 1 (`GlobalPaused::put`) + 1 event | — | 10M | Root |
| 7 | `resume_global` | 0 | 1 (`GlobalPaused::put`) + 1 event | — | 10M | Root |
| 8 | `set_pair_status` | 1 R+W (`TradingPairs::try_mutate`) | 0 + 1 event | — | 15M | |

**Totals**: 9 extrinsics

---

## 5. COMPLIANCE (`pallets/compliance/src/lib.rs` — 1072 lines)

### WeightInfo Trait
```rust
fn verify_account() -> Weight;
fn update_risk_level() -> Weight;
fn whitelist_account() -> Weight;
fn restrict_account() -> Weight;
fn flag_suspicious_activity() -> Weight;
fn add_sanctions_entry() -> Weight;
fn remove_sanctions_entry() -> Weight;
fn check_compliance() -> Weight;
fn update_verification_level() -> Weight;
```

### Storage Items
`ComplianceStatusOf`, `AuditRecords`, `SuspiciousActivities`, `SanctionsList`, `WhitelistedAccounts`, `RestrictedAccounts`, `ComplianceStats`

**Helper**: `create_audit_record` → `AuditRecords::try_mutate` (1R+1W) + `deposit_event` (1W)

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `verify_account` | 2 (`ComplianceStatusOf::get`, `ComplianceStats::get`) | 4 (`ComplianceStatusOf::insert`, `ComplianceStats::put`, `AuditRecords::try_mutate`→R+W) + 2 events | — | 30M | |
| 1 | `update_risk_level` | 1 R+W (`ComplianceStatusOf::try_mutate`) | 1 R+W (`AuditRecords::try_mutate`) + 2 events | — | 25M | |
| 2 | `whitelist_account` | 1 R+W (`ComplianceStatusOf::try_mutate`) | 2 (`WhitelistedAccounts::insert`, `AuditRecords::try_mutate`→R+W) + 2 events | — | 25M | |
| 3 | `restrict_account` | 1 R+W (`ComplianceStatusOf::try_mutate`) + 1 (`ComplianceStats::get`) | 3 (`RestrictedAccounts::insert`, `ComplianceStats::put`, `AuditRecords::try_mutate`→R+W) + 2 events | — | 30M | |
| 4 | `lift_restriction` | 1 R+W (`ComplianceStatusOf::try_mutate`) | 2 (`RestrictedAccounts::remove`, `AuditRecords::try_mutate`→R+W) + 2 events | — | 25M | |
| 5 | `flag_suspicious_activity` | 2 R+W (`SuspiciousActivities::try_mutate`, `ComplianceStatusOf::try_mutate`) + 1 (`ComplianceStats::get`) | 2 (`ComplianceStats::put`, `AuditRecords::try_mutate`→R+W) + 2 events | — | 35M | |
| 6 | `add_sanctions_entry` | 1 (`SanctionsList::contains_key`) | 1 (`SanctionsList::insert`) + 1 event | — | 15M | |
| 7 | `remove_sanctions_entry` | 1 (`SanctionsList::contains_key`) | 1 (`SanctionsList::remove`) + 1 event | — | 15M | |
| 8 | `sync_verification_from_identity` | 0 | 1 R+W (`AuditRecords::try_mutate`) + 1 event | — | 20M | Weight: `update_verification_level()` |

**Totals**: 9 extrinsics

---

## 6. STAKING (`pallets/staking/src/lib.rs` — 1479 lines)

### WeightInfo Trait
```rust
fn join_validators() -> Weight;
fn leave_validators() -> Weight;
fn withdraw_unbonded() -> Weight;
fn submit_model_delta() -> Weight;
fn assign_fl_task() -> Weight;
fn distribute_rewards() -> Weight;
fn record_quantum_contribution() -> Weight;
fn record_domain_contribution() -> Weight;
fn claim_pouw_with_domain_bonus() -> Weight;
fn report_validator_offense() -> Weight;
```

### Storage Items
`Validators`, `ValidatorCount`, `CurrentEpoch`, `ActiveFLTask`, `ModelSubmissions`, `EpochRewards`, `SlashingSpans`, `PendingUnbonds`, `QuantumContributions`, `ValidatorQuantumStatsMap`, `EpochQuantumJobs`, `OperatorDomainStatsMap`, `LastClaimedEpoch`, `EpochDomainContributions`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `join_validators` | 3 + 2 ext (`Validators::contains_key`, `ValidatorCount::get`, `Currency::free_balance`; Identity: `meets_validator_kyc`, `is_sanctioned`) | 2 (`Validators::insert`, `ValidatorCount::put`) + 1 event | `set_lock` ×1 | 40M | |
| 1 | `leave_validators` | 2 (`Validators::get`, `PendingUnbonds::contains_key`) | 3 (`Validators::remove`, `ValidatorCount::mutate`→R+W, `PendingUnbonds::insert`) + 1 event | — | 25M | |
| 2 | `submit_model_delta` | 3 (`Validators::get`, `ActiveFLTask::get`, `ModelSubmissions::contains_key`) | 2 (`ModelSubmissions::insert`, `Validators::insert`) + 1 event | — | 25M | |
| 3 | `assign_fl_task` | 0 | 2 (`ActiveFLTask::put`, `ModelSubmissions::clear`) + 1 event | — | 15M | Root |
| 4 | `distribute_rewards` | 2 + N×1 (`CurrentEpoch::get`, `Validators::iter` bounded, `ModelSubmissions::get` ×N) | 5 + N×0 (`EpochRewards::insert`, `CurrentEpoch::put`, `ModelSubmissions::clear`, `ValidatorQuantumStatsMap::clear`, `EpochQuantumJobs::kill`) + 2 events | `deposit_creating` ×N | 120M | O(N) validator rewards |
| 5 | `record_quantum_contribution` | 2 (`Validators::contains_key`, `QuantumContributions::contains_key`) | 3 (`QuantumContributions::insert`, `ValidatorQuantumStatsMap::mutate`→R+W, `EpochQuantumJobs::mutate`→R+W) + 1 event | — | 30M | |
| 6 | `force_join_validator` | 2 + bal (`Validators::contains_key`, `ValidatorCount::get`, `Currency::free_balance`) | 2 (`Validators::insert`, `ValidatorCount::mutate`→R+W) + 1 event | `set_lock` ×1 | 35M | Root |
| 7 | `record_domain_contribution` | 0 | 2 R+W (`OperatorDomainStatsMap::mutate`, `EpochDomainContributions::mutate`) + 1 event | — | 20M | |
| 8 | `claim_pouw_with_domain_bonus` | 5 (`Validators::get`, `CurrentEpoch::get`, `LastClaimedEpoch::get`, `OperatorDomainStatsMap::get`, `ValidatorQuantumStatsMap::get`) | 2 (`LastClaimedEpoch::insert`, `OperatorDomainStatsMap::remove`) + 1 event | `deposit_creating` ×1 | 50M | |
| 9 | `withdraw_unbonded` | 1 (`PendingUnbonds::get`) | 1 (`PendingUnbonds::remove`) + 1 event | `remove_lock` ×1 | 20M | |
| 10 | `report_validator_offense` | 3 (`Validators::contains_key`, `Validators::get`, `SlashingSpans::get`) | 2 (`Validators::insert`, `SlashingSpans::insert`) + 1 event | `slash` + `set_lock` | 45M | Root |

**Totals**: 11 extrinsics

---

## 7. PAYROLL (`pallets/payroll/src/lib.rs` — 1456 lines)

### WeightInfo Trait
```rust
fn add_employee() -> Weight;
fn remove_employee() -> Weight;
fn update_salary() -> Weight;
fn execute_payment() -> Weight;
fn batch_payment(n: u32) -> Weight;
fn create_schedule() -> Weight;
fn update_schedule() -> Weight;
```

### Storage Items
`Employees` (DoubleMap), `EmployeeDeductions` (DoubleMap), `EmployerProfiles`, `Departments` (DoubleMap), `PayrollSchedules` (DoubleMap), `NextScheduleId`, `PayrollRecords`, `NextRecordId`, `GlobalStats`, `VerifiedEmployers`, `EmployerCount`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 7 | `verify_employer` | 0 | 2 (`EmployerProfiles::insert`, `VerifiedEmployers::insert`) + 1 event | — | 15M | VerifierOrigin |
| 0 | `add_employee` | 4 + ext (`VerifiedEmployers::get`, `Departments::contains_key` cond, Oracle: `meets_kyc_requirement`, `Employees::iter_prefix` count, `Employees::contains_key`) | 2 (`Employees::insert`, `GlobalStats::mutate`→R+W) + cond (`EmployerCount::mutate`→R+W) + 1 event | — | 40M | Bounded iter for count |
| 1 | `remove_employee` | 2 (`Employees::contains_key`, `Employees::iter_prefix` count) | 3 (`Employees::remove`, `EmployeeDeductions::remove`, `GlobalStats::mutate`→R+W) + cond `EmployerCount::mutate` + 1 event | — | 35M | |
| 8 | `toggle_employee_status` | 1 (`Employees::get`) | 1 (`Employees::insert`) + 1 event | — | 15M | |
| 2 | `update_salary` | 1 (`Employees::get`) | 1 (`Employees::insert`) + 1 event | — | 15M | |
| 3 | `execute_payment` | 4 (`Employees::get`, `EmployeeDeductions::get`, `Currency::free_balance`, `NextRecordId::get`) | 3 (`Employees::insert`, `PayrollRecords::insert`, `NextRecordId::put`) + `GlobalStats::mutate`→R+W + 1 event | `transfer` ×1 | 45M | |
| 4 | `batch_payment` | 2 + N×2 (`Employees::iter_prefix`×2, `Currency::free_balance`, `EmployeeDeductions::get` ×N) | N×3 (`Employees::insert`, `PayrollRecords::insert`, `NextRecordId::put` ×N) + `GlobalStats::mutate` + 1 event | `transfer` ×N | 50M + 30M×N | O(N) employee payments |
| 5 | `create_schedule` | 3 (`VerifiedEmployers::get`, `Employees::iter_prefix` count, `NextScheduleId::get`) | 2 (`PayrollSchedules::insert`, `NextScheduleId::put`) + 1 event | — | 30M | |
| 6 | `update_schedule` | 1 (`PayrollSchedules::get`) | 1 (`PayrollSchedules::insert`) + 1 event | — | 15M | |
| 9 | `create_department` | 2 (`VerifiedEmployers::get`, `Departments::iter_prefix` count) | 2 (`Departments::insert`, `EmployerProfiles::mutate`→R+W) + 1 event | — | 30M | |
| 10 | `set_deduction` | 1 (`Employees::contains_key`) + 1 R+W (`EmployeeDeductions::try_mutate`) | 0 + 1 event | — | 20M | |
| 11 | `issue_bonus` | 3 (`Employees::get`, `Currency::free_balance`, `NextRecordId::get`) | 3 (`Employees::insert`, `PayrollRecords::insert`, `NextRecordId::put`) + `GlobalStats::mutate`→R+W + 1 event | `transfer` ×1 | 45M | |

**Totals**: 12 extrinsics; `batch_payment` is O(N)

**Note**: `on_initialize` hook iterates `PayrollSchedules` (bounded by `MaxSchedulesPerBlock × 10`)

---

## 8. QUANTUM (`pallets/quantum/src/lib.rs` — 2146 lines)

### WeightInfo Trait
```rust
fn submit_quantum_job() -> Weight;
fn update_job_status() -> Weight;
fn record_quantum_result() -> Weight;
fn verify_quantum_result() -> Weight;
fn mint_achievement_nft() -> Weight;
fn transfer_nft() -> Weight;
fn list_nft() -> Weight;
fn buy_nft() -> Weight;
fn delist_nft() -> Weight;
fn bridge_to_ethereum() -> Weight;
fn bridge_to_parachain() -> Weight;
fn request_verification() -> Weight;
fn submit_verification() -> Weight;
fn cancel_bridge() -> Weight;
```

### Storage Items
`QuantumJobs`, `JobsByAccount`, `QuantumResults`, `QuantumAchievements`, `NFTCounter`, `TotalQuantumJobs`, `TotalDallaSpent`, `AccountStats`, `VerificationRequests`, `ValidatorReputation`, `JobCounter`, `NFTListings`, `NFTAuctions`, `ListingCounter`, `BridgeRequests`, `BridgeCounter`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `submit_quantum_job` | 1 (`QuantumJobs::contains_key`) | 5 (`QuantumJobs::insert`, `JobsByAccount::try_mutate`→R+W, `TotalQuantumJobs::mutate`→R+W, `TotalDallaSpent::mutate`→R+W, `AccountStats::mutate`→R+W) + 1 event | `reserve` ×1 | 40M | |
| 1 | `update_job_status` | 1 R+W (`QuantumJobs::try_mutate`) | cond `AccountStats::mutate`→R+W + 1 event | `unreserve` cond (fail/cancel) | 25M | |
| 2 | `record_quantum_result` | 1 (`QuantumResults::contains_key`) + 1 R+W (`QuantumJobs::try_mutate`) | 1 (`QuantumResults::insert`) + 1 event | — | 25M | |
| 3 | `verify_quantum_result` | 1 (`QuantumResults::contains_key`) + 1 R+W (`QuantumJobs::try_mutate`) | 0 + 1 event | `unreserve` or `repatriate_reserved` | 30M | Root |
| 4 | `mint_achievement_nft` | 2 (`QuantumJobs::get`, `VerificationRequests::get`) | 2 (`NFTCounter::mutate`→R+W, `QuantumAchievements::insert`) + `AccountStats::mutate`→R+W + 1 event | `withdraw` ×1 (mint fee) | 40M | |
| 5 | `transfer_nft` | 1 (`BridgeRequests::contains_key`) + 1 R+W (`QuantumAchievements::try_mutate`) | 0 + 1 event | — | 20M | |
| 6 | `list_nft` | 3 (`QuantumAchievements::get`, `NFTListings::contains_key`, `NFTAuctions::contains_key`) | 1 (`NFTListings::insert`) + `ListingCounter::mutate`→R+W + 1 event | — | 25M | |
| 7 | `buy_nft` | 1 (`NFTListings::get`) + 1 R+W (`QuantumAchievements::try_mutate`) | 1 (`NFTListings::remove`) + 1 event | `transfer` ×2 (seller + royalty cond) | 40M | |
| 8 | `delist_nft` | 1 (`NFTListings::get`) | 1 (`NFTListings::remove`) + 1 event | — | 15M | |
| 9 | `request_verification` | 3 (`QuantumJobs::get`, `QuantumResults::contains_key`, `VerificationRequests::contains_key`) | 1 (`VerificationRequests::insert`) + 1 event | — | 25M | |
| 10 | `submit_verification` | 1 (`VerificationRequests::get`) | 1 (`VerificationRequests::insert`) + 1 event; on consensus: R+W `QuantumJobs::try_mutate`, R `QuantumResults::get`, W `ValidatorReputation::insert` ×N + N events | `repatriate_reserved` or `unreserve` | 80M | O(N) validator reputation updates on consensus |
| 11 | `bridge_to_ethereum` | 4 (`QuantumAchievements::get`, `NFTListings::contains_key`, `NFTAuctions::contains_key`, `BridgeRequests::contains_key`) | 2 (`BridgeRequests::insert`, `BridgeCounter::mutate`→R+W) + `QuantumAchievements::mutate`→R+W + 1 event | — | 35M | |
| 12 | `bridge_to_parachain` | 4 (same as bridge_to_ethereum) | 2 + R+W (same) + 1 event | — | 35M | |
| 13 | `cancel_bridge` | 1 (`BridgeRequests::get`) | 2 (`BridgeRequests::remove`, `QuantumAchievements::mutate`→R+W) + 1 event | — | 20M | |

**Totals**: 14 extrinsics

---

## 9. MESH (`pallets/mesh/src/lib.rs` — 1430 lines)

### WeightInfo Trait
```rust
fn register_node() -> Weight;
fn deregister_node() -> Weight;
fn update_node_location() -> Weight;
fn node_heartbeat() -> Weight;
fn submit_mesh_transaction() -> Weight;
fn submit_relay_proof() -> Weight;
fn issue_emergency_alert() -> Weight;
fn resolve_emergency_alert() -> Weight;
fn confirm_emergency_alert() -> Weight;
fn relay_block_header() -> Weight;
fn claim_relay_rewards() -> Weight;
fn update_mesh_config() -> Weight;
fn fund_relay_rewards() -> Weight;
fn confirm_relay_proof() -> Weight;
```

### Storage Items
`MeshNodes`, `NodesByOwner`, `PendingMeshTransactions`, `ProcessedMeshTransactions`, `EmergencyAlerts`, `NextAlertId`, `MeshBlockHeaders`, `RelayProofs`, `RelayRewards`, `NetworkStats`, `MeshConfig`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `register_node` | 4 + 2 ext (`MeshNodes::contains_key`, `MeshConfig::get`, `NodesByOwner::get`, `NetworkStats::get`; Identity: `get_kyc_level`, `is_validator` cond) | 3 (`MeshNodes::insert`, `NodesByOwner::insert`, `NetworkStats::put`) + 1 event | `reserve` ×1 | 45M | |
| 1 | `deregister_node` | 2 (`MeshNodes::get`, `NodesByOwner::get`) | 3 (`NodesByOwner::insert`, `MeshNodes::remove`, `NetworkStats::put`) + 1 event | `unreserve` ×1 | 30M | |
| 2 | `update_node_location` | 1 R+W (`MeshNodes::try_mutate`) | 0 + 1 event | — | 15M | |
| 3 | `node_heartbeat` | 1 R+W (`MeshNodes::try_mutate`) | cond `NetworkStats::put` + 1 event | — | 20M | |
| 4 | `submit_mesh_transaction` | 4 (`MeshNodes::get`, `MeshConfig::get`, `PendingMeshTransactions::contains_key`, `ProcessedMeshTransactions::contains_key`) | 3 (`PendingMeshTransactions::insert`, `MeshNodes::mutate`→R+W, `NetworkStats::mutate`→R+W) + 1 event | — | 40M | |
| 5 | `submit_relay_proof` | 2 (`MeshConfig::get`, `MeshNodes::get`) | 3 (`RelayProofs::try_mutate`→R+W, `MeshNodes::mutate`→R+W, `NetworkStats::mutate`→R+W) + 1 event | — | 40M | |
| 6 | `issue_emergency_alert` | 2 + ext (`MeshConfig::get`, `NextAlertId::get`; Identity: `is_emergency_authority` cond) | 2 (`EmergencyAlerts::insert`, `NextAlertId::put`) + `NetworkStats::mutate`→R+W + 1 event | — | 30M | |
| 7 | `resolve_emergency_alert` | ext (`Identity::is_emergency_authority` cond) + 1 R+W (`EmergencyAlerts::try_mutate`) | 0 + 1 event | — | 20M | |
| 8 | `confirm_emergency_alert` | 1 (`MeshNodes::get`) + 1 R+W (`EmergencyAlerts::try_mutate`) | 0 + 1 event | — | 20M | |
| 9 | `relay_block_header` | 2 (`MeshConfig::get`, `MeshNodes::get`) | 2 (`MeshBlockHeaders::insert`, `NetworkStats::mutate`→R+W) + 1 event | — | 30M | |
| 10 | `claim_relay_rewards` | 2 (`RelayRewards::get`, `NodesByOwner::get`) | N×1 (`RelayProofs::remove` per node) + 1 (`RelayRewards::remove`) + `NetworkStats::mutate`→R+W + 1 event | `transfer` ×1 (pallet→claimer) | 50M | O(N) proof cleanup |
| 11 | `update_mesh_config` | 0 | 1 (`MeshConfig::put`) + 1 event | — | 10M | GovernanceOrigin |
| 12 | `fund_relay_rewards` | 0 | 0 + 1 event | `transfer` ×1 (funder→pallet) | 15M | |
| 13 | `confirm_relay_proof` | 2 (`MeshNodes::get`, `NodesByOwner::get`) + 1 R+W (`RelayProofs::try_mutate`) | `RelayRewards::mutate`→R+W + 1 event | — | 30M | |

**Totals**: 14 extrinsics

---

## 10. IDENTITY (`pallets/identity/src/lib.rs` — 1149 lines)

### WeightInfo Trait
```rust
fn register_identity() -> Weight;
fn link_account() -> Weight;
fn update_did() -> Weight;
fn admin_simple() -> Weight;
fn issue_attestation() -> Weight;
fn revoke() -> Weight;
```

### Storage Items (24+)
`NextIdentityId`, `IdentityOf`, `Identities`, `SsnAttestations`, `PassportAttestations`, `BiometricAttestations`, `SsnHashIndex`, `PassportHashIndex`, `FlaggedIssuers` (DoubleMap), `IssuerBonds` (DoubleMap), `IssuerBondAmount`, `RateWindowBlocks`, `RateMaxPerWindowSsn`, `RateMaxPerWindowPassport`, `RateMaxPerWindowBiometrics`, `IssuerRate` (DoubleMap), `AttributeHistory` (DoubleMap), `SsnIssuers`, `PassportIssuers`, `BiometricIssuers`, `SsnStandardVersion`, `PassportStandardVersion`, `OperationFee`, `GlobalPaused`

**Helpers**: `ensure_not_paused` (1R), `is_authorized_issuer` (1R), `charge_fee` (1R + `Currency::transfer`), `append_history` (1W `AttributeHistory::mutate`), `check_and_bump_rate` (2R + 1W: `RateWindowBlocks`, `IssuerRate`, `IssuerRate::insert`)

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `register_identity` | 4 (`GlobalPaused`, `IdentityOf::contains_key`, `OperationFee`, `NextIdentityId`) | 3 (`Identities::insert`, `IdentityOf::insert`, `NextIdentityId::put`) + 1 event | `transfer`(fee) ×1 | 35M | |
| 1 | `link_account` | 3 (`GlobalPaused`, `IdentityOf::get`, `IdentityOf::contains_key`) + 1 R+W (`Identities::try_mutate`) | 1 (`IdentityOf::insert`) + 1 event | — | 25M | |
| 2 | `update_did_doc` | 1 (`IdentityOf::get`) + 1 R+W (`Identities::try_mutate`) | 0 + 1 event | — | 20M | |
| 3 | `add_issuer` | 2 (`IssuerBondAmount::get`, `IssuerBonds::get`) | 1 (`SsnIssuers`/`PassportIssuers`/`BiometricIssuers::try_mutate`) + 1 event | — | 20M | AdminOrigin; weight: `admin_simple()` |
| 4 | `remove_issuer` | 0 | 1 (`*Issuers::mutate`) + 1 event | — | 15M | AdminOrigin |
| 5 | `set_standard_version` | 0 | 1 (`*StandardVersion::put`) + 1 event | — | 10M | AdminOrigin |
| 6 | `set_operation_fee` | 0 | 1 (`OperationFee::put`) + 1 event | — | 10M | |
| 7 | `set_issuer_bond_amount` | 0 | 1 (`IssuerBondAmount::put`) + 1 event | — | 10M | |
| 8 | `set_rate_limits` | 0 | 4 (`RateWindowBlocks::put`, `RateMaxPerWindow{Ssn,Passport,Biometrics}::put`) + 1 event | — | 15M | |
| 9 | `set_pause` | 0 | 1 (`GlobalPaused::put`) + 1 event | — | 10M | |
| 10 | `issue_ssn` | 8 (`SsnIssuers`, `FlaggedIssuers`, `GlobalPaused`, `RateWindowBlocks`, `IssuerRate`, `IdentityOf`, `SsnHashIndex`, `SsnAttestations`) + 1 cond (`PassportAttestations::contains_key`) | 5 (`SsnAttestations::insert`, `SsnHashIndex::insert`, `SsnHashIndex::remove` cond, `AttributeHistory::mutate`, `IssuerRate::insert`) + 1 event | — | 60M | Heaviest attestation; rate-limited |
| 11 | `issue_passport` | 7 (`PassportIssuers`, `FlaggedIssuers`, `GlobalPaused`, `RateWindowBlocks`, `IssuerRate`, `IdentityOf`, `PassportHashIndex`) | 4 (`PassportAttestations::insert`, `PassportHashIndex::insert`, `AttributeHistory::mutate`, `IssuerRate::insert`) + 1 event | — | 55M | |
| 12 | `issue_biometrics` | 6 (`BiometricIssuers`, `FlaggedIssuers`, `GlobalPaused`, `RateWindowBlocks`, `IssuerRate`, `IdentityOf`) | 3 (`BiometricAttestations::insert`, `AttributeHistory::mutate`, `IssuerRate::insert`) + 1 event | — | 50M | |
| 13 | `revoke` | 1 (`IdentityOf::get`) + 1–3 R+W (`*Attestations::try_mutate`) | 1 (`AttributeHistory::mutate`) + 1 event | — | 35M | RevokeOrigin; weight: `revoke()` |
| 14 | `suspend` | (same as revoke) | (same as revoke) | — | 35M | |
| 15 | `issuer_deposit_bond` | 2 (`IssuerBondAmount`, `IssuerBonds::get`) | 1 (`IssuerBonds::insert`) + 1 event | `transfer` ×1 | 25M | |
| 16 | `issuer_withdraw_bond` | 3 (`is_authorized_issuer`, `FlaggedIssuers`, `IssuerBonds::get`) | 1 (`IssuerBonds::remove`) + 1 event | `transfer` ×1 | 25M | |
| 17 | `flag_issuer` | 0 | 1 (`FlaggedIssuers::insert`) + 1 event | — | 10M | AdminOrigin |
| 18 | `slash_issuer_bond` | 1 (`IssuerBonds::get`) | 1 (`IssuerBonds::insert`) + 1 event | `transfer`(to Treasury) ×1 | 25M | AdminOrigin |
| 19 | `report_bad_attestation` | 1 cond (`IssuerBonds::get`) | 2 cond (`FlaggedIssuers::insert`, `IssuerBonds::insert`) + 2 events cond | `transfer`(to Treasury) cond | 30M | AdminOrigin |

**Totals**: 20 extrinsics

---

## 11. GOVERNANCE (`pallets/governance/src/lib.rs` — 6042 lines)

### WeightInfo Trait
```rust
fn submit_proposal() -> Weight;
fn cast_vote() -> Weight;
fn finalize_proposal() -> Weight;
fn update_community_rank() -> Weight;
fn update_pouw_contribution() -> Weight;
fn council_override() -> Weight;
fn update_chain_parameter() -> Weight;
```

**Note**: 30 of 37 extrinsics use inline `Weight::from_parts(...)` instead of WeightInfo trait functions. Only call_index 0-5 and 36 use WeightInfo.

### Storage Items (40+)
`CouncilMembers`, `Proposals`, `Votes` (DoubleMap), `VoteCommitments` (DoubleMap, unused), `CommunityRanks`, `PoUWContributions`, `NextProposalId`, `CouncilTerm`, `JaguarMode`, `DepartmentManagers`, `DepartmentProposalCount`, `CrossDepartmentApprovals` (DoubleMap), `BoardComposition`, `TermExpiryQueue`, `DelegateNominees`, `DelegateVoters` (DoubleMap), `CurrentElection`, `GovernanceParameters`, `DepartmentTreasuryBalances`, `ExecutedProposals`, `PendingRuntimeUpgrade`, `DistrictElections`, `ElectionCandidates` (DoubleMap), `ElectionVotes` (DoubleMap), `DistrictRepresentation`, `VoteDelegations`, `DelegationReceivers`, `ProposalAmendments`, `ProposalQueue`, `RewardsClaimed`, `TotalRewardsDistributed`, `Referendums`, `NextReferendumId`, `ReferendumVotes` (DoubleMap), `ReferendumEligibleVoters`, `DistrictBudgets`, `TreasurySpendProposals`, `NextTreasuryProposalId`, `NationalTreasuryReserve`, `ChainParameters`

| # | Function | Reads | Writes | Currency Ops | Est. Base Weight | Notes |
|---|----------|-------|--------|-------------|-----------------|-------|
| 0 | `submit_proposal` | 2 + ext (`NextProposalId`, ext: `ComplianceProvider`) | 2 (`Proposals::insert`, `NextProposalId::put`) + 1 event | `reserve` ×1 | 30M | |
| 1 | `cast_vote` | 5 + ext (`Proposals::get`, `Votes::contains_key`, `CommunityRanks`, `PoUWContributions`; ext: `ComplianceProvider`) | 2 (`Votes::insert`, `Proposals::insert`) + 1 event | — | 25M | ext: `CommunityParticipation::record_vote_cast` |
| 2 | `finalize_proposal` | 1 (`Proposals::get`) | 1 (`Proposals::insert`) + 1 event | `unreserve` ×1 | 20M | |
| 3 | `update_community_rank` | 2 (`CommunityRanks::get`, `CouncilMembers::get` cond) | 2 (`CommunityRanks::insert`, `CouncilMembers::insert` cond) + 1 event | — | 15M | Root |
| 4 | `update_pouw_contribution` | 2 (`PoUWContributions::get`, `CouncilMembers::get` cond) | 2 (`PoUWContributions::insert`, `CouncilMembers::insert` cond) + 1 event | — | 15M | Root |
| 5 | `council_override` | 2 (`Proposals::get`, `JaguarMode::get`) | 1 (`Proposals::insert`) + 1 event | — | 20M | CouncilOrigin |
| 6 | `set_department_manager` | 0 | 1 (`DepartmentManagers::insert`) + 1 event | — | 10M | Root |
| 7 | `submit_department_proposal` | 3 + ext (`DepartmentManagers::get`, `NextProposalId`, `DepartmentProposalCount::get`; ext: `ComplianceProvider`) | 3 (`Proposals::insert`, `NextProposalId::put`, `DepartmentProposalCount::insert`) + 1 event | `reserve` ×1 | 35M | ext: `CommunityParticipation::record_proposal_submission` |
| 8 | `approve_cross_department` | 3 (`DepartmentManagers::get`, `Proposals::get`, `CrossDepartmentApprovals::get`) | 2 (`CrossDepartmentApprovals::insert`, `Proposals::insert`) + 1 event | — | 25M | |
| 9 | `add_board_member` | 1 (`BoardComposition::get`) | 3 (`CouncilMembers::insert`, `BoardComposition::insert`, `TermExpiryQueue::try_mutate`→R+W) + 1 event | — | 25M | CouncilOrigin |
| 10 | `remove_board_member` | 2 (`CouncilMembers::get`, `BoardComposition::get`) | 3 (`CouncilMembers::remove`, `BoardComposition::insert`, `TermExpiryQueue::mutate`→R+W) + 1 event | — | 25M | CouncilOrigin |
| 11 | `nominate_for_delegate` | 2 + ext (`CurrentElection::get`, `DelegateNominees::contains_key`; ext: `ComplianceProvider`) | 1 cond (`DelegateNominees::insert`) + 1 event | — | 15M | |
| 12 | `vote_for_delegate` | 3 + ext (`CurrentElection::get`, `DelegateVoters::contains_key`, `DelegateNominees::contains_key`; ext: `ComplianceProvider`) | 2 (`DelegateVoters::insert`, `DelegateNominees::mutate`→R+W) + 1 event | — | 20M | |
| 13 | `execute_proposal` | 3 (`CouncilMembers::contains_key`, `Proposals::get`, `ExecutedProposals::contains_key`) | 2 (`Proposals::insert`, `ExecutedProposals::insert`) + 1 event | varies by action | 50M | Delegates to `execute_action` helper |
| 14 | `start_district_election` | 1 (`DistrictElections::contains_key`) | 1 (`DistrictElections::insert`) + 1 event | — | 20M | Root |
| 15 | `register_candidate` | 3 + ext + iter (`DistrictElections::get`, `ElectionCandidates::contains_key`, iter count; ext: `ComplianceProvider`) | 2 (`ElectionCandidates::insert`, `DistrictElections::insert` cond) + 1 event | — | 30M | Bounded iter for count |
| 16 | `vote_in_district_election` | 3 + ext (`DistrictElections::get`, `ElectionVotes::contains_key`, `ElectionCandidates::get`; ext: `ComplianceProvider`) | 3 (`ElectionVotes::insert`, `ElectionCandidates::insert`, `DistrictElections::insert`) + 1 event | — | 30M | |
| 17 | `finalize_district_election` | 2 + N (`DistrictElections::get`, `ElectionCandidates::iter_prefix` bounded, `::get` per winner) | 3 + N (`CouncilMembers::insert` ×N, `DistrictRepresentation::insert`, `DistrictElections::insert`) + (1+N) events | — | 80M | Root; O(N) winners |
| 18 | `delegate_vote` | 2 (`VoteDelegations::contains_key`, `DelegationReceivers::get`) | 2 (`VoteDelegations::insert`, `DelegationReceivers::insert`) + 1 event | — | 25M | |
| 19 | `revoke_delegation` | 2 (`VoteDelegations::get`, `DelegationReceivers::get`) | 2 (`VoteDelegations::remove`, `DelegationReceivers::insert`) + 1 event | — | 20M | |
| 20 | `amend_proposal` | 2 (`Proposals::get`, `ProposalAmendments::contains_key`) | 2 (`ProposalAmendments::insert`, `Proposals::insert`) + 1 event | — | 25M | |
| 21 | `claim_participation_reward` | 3–1001 (`Votes::contains_key` ×1000 or `Proposals::iter` ×1000 or `CouncilMembers::contains_key`; `RewardsClaimed::get`, `Currency::free_balance`, `TotalRewardsDistributed::get`) | 2 (`RewardsClaimed::insert`, `TotalRewardsDistributed::put`) + 1 event | `transfer` ×1 | 60–200M | ⚠️ Bounded scan up to 1000 items |
| 22 | `set_proposal_priority` | 2 (`Proposals::contains_key`, `ProposalQueue::get`) | 1 (`ProposalQueue::insert`) + 1 event | — | 15M | Root |
| 23 | `declare_emergency` | 1 (`JaguarMode::get`) | 1 (`JaguarMode::put`) + 1 event | — | 15M | CouncilOrigin/Root |
| 24 | `end_emergency` | 1 (`JaguarMode::get`) | 1 (`JaguarMode::kill`) + 1 event | — | 10M | CouncilOrigin/Root |
| 25 | `create_referendum` | 1 + ext (`NextReferendumId::get`; ext: `ComplianceProvider`) | 3 (`Referendums::insert`, `NextReferendumId::put`, `ReferendumEligibleVoters::insert`) + 1 event | — | 30M | |
| 26 | `vote_on_referendum` | 4 + ext (`Referendums::get`, `ReferendumVotes::contains_key`, `CommunityRanks`, `PoUWContributions`; ext: `ComplianceProvider`) | 2 (`ReferendumVotes::insert`, `Referendums::insert`) + 1 event | — | 25M | ext: `CommunityParticipation::record_vote_cast` |
| 27 | `finalize_referendum` | 2 (`Referendums::get`, `ReferendumEligibleVoters::get`) | 1 (`Referendums::insert`) + 1 event | — | 20M | |
| 28 | `allocate_district_budget` | 1 (`DistrictBudgets::get`) | 1 (`DistrictBudgets::insert`) + 1 event | — | 20M | Root |
| 29 | `propose_treasury_spend` | 1 + ext (`NextTreasuryProposalId::get`; ext: `ComplianceProvider`) | 2 (`TreasurySpendProposals::insert`, `NextTreasuryProposalId::put`) + 1 event | — | 25M | |
| 30 | `approve_treasury_spend` | 2 (`CouncilMembers::contains_key`, `TreasurySpendProposals::get`) | 1 (`TreasurySpendProposals::insert`) + 1 event | — | 20M | |
| 31 | `execute_treasury_proposal` | 3–4 (`TreasurySpendProposals::get`, `DistrictBudgets::get` cond, `NationalTreasuryReserve` cond, `Currency::free_balance`) | 3 cond (`TreasurySpendProposals::insert`, `DistrictBudgets::insert`/`NationalTreasuryReserve::put`) + 2 events | `transfer` ×1 | 45M | |
| 32 | `transfer_district_budget` | 2 (`DistrictBudgets::get` ×2) | 2 (`DistrictBudgets::insert` ×2) + 1 event | — | 25M | Root |
| 33 | `execute_emergency_proposal` | 2 (`JaguarMode::get`, `Proposals::get`) | 1 (`Proposals::insert`) + 1 event | — | 25M | Root/CouncilOrigin |
| 34 | `fast_track_referendum` | 2 (`JaguarMode::get`, `Referendums::get`) | 1 (`Referendums::insert`) + 1 event | — | 20M | Root/CouncilOrigin |
| 35 | `emergency_override_proposal` | 2 (`JaguarMode::get`, `Proposals::get`) | 1 (`Proposals::insert`) + 1 event | — | 20M | Root |
| 36 | `update_chain_parameter` | 1 (`ChainParameters::get`) | 1 (`ChainParameters::insert`) + 1 event | — | 10M | Root |

**Totals**: 37 extrinsics; `claim_participation_reward` has bounded scan up to 1000 items; `finalize_district_election` has O(N) winner processing

### execute_action Helper (called by #13)
| Action Variant | Reads | Writes | Currency Ops |
|---------------|-------|--------|-------------|
| `TreasurySpend` | 1 (`Currency::free_balance`) | 0 + 1 event | `transfer` ×1 |
| `RuntimeUpgrade` | 0 | 1 (`PendingRuntimeUpgrade::put`) + 1 event | — |
| `ParameterChange` | 1 (`GovernanceParameters::get`) | 1 (`GovernanceParameters::insert`) + 1 event | — |
| `DepartmentAction` | 0 | 0 + 1 event | — |
| `EmergencyAction` | 0 | 1 (`JaguarMode::put` or `::kill`) + 1 event | — |

---

## Summary Statistics

| Pallet | Extrinsics | Storage Items | WeightInfo Fns | Heaviest Call |
|--------|-----------|---------------|----------------|---------------|
| landledger | 5 | 10 | 5 | `transfer_property` (7R + 8W + 1 Cx) |
| consensus | 6 | 10 | 6 | `finalize_consensus_round` (O(N) rewards) |
| interoperability | 7 | 10 | 6 | `initiate_bridge` (4R + 3W + 2 Cx) |
| belizex | 9 | 7 | 8 | `execute_multihop_trade` (O(H) hops) |
| compliance | 9 | 7 | 9 | `flag_suspicious_activity` (5 R+W) |
| staking | 11 | 14 | 10 | `distribute_rewards` (O(N) validators) |
| payroll | 12 | 11 | 7 | `batch_payment` (O(N) employees) |
| quantum | 14 | 16 | 14 | `submit_verification` (O(N) validators on consensus) |
| mesh | 14 | 11 | 14 | `claim_relay_rewards` (O(N) proof cleanup) |
| identity | 20 | 24+ | 6 | `issue_ssn` (8R + 5W, rate-limited) |
| governance | 37 | 40+ | 7 | `claim_participation_reward` (1000-item scan) |
| **TOTAL** | **144** | **160+** | **92** | |

### Key Observations

1. **WeightInfo Coverage Gap**: Governance has 37 extrinsics but only 7 WeightInfo functions — 30 extrinsics use hardcoded `Weight::from_parts()`. This is a benchmarking gap.

2. **O(N) Hotspots**: `finalize_consensus_round`, `distribute_rewards`, `batch_payment`, `submit_verification`, `finalize_district_election`, and `claim_participation_reward` all have linear-in-N costs.

3. **Cross-Pallet Reads**: `landledger` (Oracle ×4), `belizex` (Oracle ×3), `staking` (Identity ×2), `mesh` (Identity ×2), `interoperability` (Identity ×2), `governance` (ComplianceProvider throughout). These are not accounted for in WeightInfo.

4. **Currency Operations**: 9 of 11 pallets use Currency operations. `staking` and `consensus` use `deposit_creating` for rewards. `interoperability` and `staking` use `set_lock`/`remove_lock`.

5. **Largest Pallet**: Governance at 6042 lines, 37 extrinsics, 40+ storage items — accounts for ~26% of all dispatchable functions.
