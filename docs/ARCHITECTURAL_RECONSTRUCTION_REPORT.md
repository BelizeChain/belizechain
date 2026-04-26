# BelizeChain — Comprehensive Architectural Reconstruction Report

**Generated**: 2025-07 (historical snapshot)
**Scope**: Full codebase audit of `/home/wicked/Projects/belizechain-belizechain` at snapshot time
**Snapshot SDK**: Polkadot SDK `stable2512` (September 2025)
**Snapshot Runtime**: `belizechain_runtime` — spec_version=100, impl_version=1, tx_version=1
**Current Runtime Note**: The active branch has since moved to Polkadot SDK `stable2603` and runtime `spec_version=104`; use `Cargo.toml` and `runtime/src/lib.rs` as the live sources of truth.

---

## Table of Contents

1. [Runtime Topology](#1-runtime-topology)
2. [Per-Pallet Inventory](#2-per-pallet-inventory)
3. [Cross-Pallet Dependency Graph](#3-cross-pallet-dependency-graph)
4. [Trust & Security Model](#4-trust--security-model)
5. [Economic Model](#5-economic-model)
6. [Upgrade Strategy](#6-upgrade-strategy)
7. [Patterns & Anti-Patterns](#7-patterns--anti-patterns)

---

## 1. Runtime Topology

### 1.1 Block Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Block Time | 6 seconds | `MILLISECS_PER_BLOCK = 6000` |
| Blocks/Minute | 10 | `MINUTES = 10` |
| Blocks/Day | 14,400 | `DAYS = 14_400` |
| BlockHashCount | 2,400 | ~4 hours of recent block hashes |
| SS58 Prefix | **1981** | Year of Belizean independence |
| Max Block Weight | 2×WEIGHT_REF_TIME_PER_SECOND | 2 seconds of ref time |
| Max Block Length | 5 MB | `5 * 1024 * 1024` |
| Normal Dispatch Ratio | 75% | `Perbill::from_percent(75)` |

### 1.2 Currency

| Parameter | Value |
|-----------|-------|
| Native Token | **DALLA** |
| Decimals | 12 |
| Base Unit | `1 DALLA = 10^12` |
| Existential Deposit | 0.001 DALLA (`1_000_000_000`) |
| Max Supply | 501 billion DALLA (`501_000_000_000 * DOLLARS`) |
| Stablecoin | bBZD (Belizean Dollar, 1:1 BZD peg, Central Bank mint/burn) |

### 1.3 Consensus

> **UPDATE (2026-03):** Block production migrated from Aura to BABE (VRF-based slot assignment).
> PoUW `quality_score` now drives BABE authority weights. `pallet_session` + `pallet_offences`
> wired for validator rotation and equivocation slashing.

| Component | Configuration |
|-----------|---------------|
| Block Production | **BABE** (VRF-based slot assignment with PoUW-weighted authorities) |
| Finality | **GRANDPA** (BFT finality gadget) |
| MaxAuthorities | 32 |
| EpochDuration | 14,400 blocks (~24h at 6s/block) |
| MinimumPeriod | 3,000 ms |
| Justification Period | 512 blocks |
| Gossip Duration | 333 ms |
| Chain Selection | LongestChain |

### 1.4 Smart Contracts

Enabled via `pallet_contracts` ("GEM Smart Contract Platform"):
- Max Code Size: 128 KB
- Call Stack Depth: 5
- Environment: ink! / Wasm

### 1.5 Randomness

> **UPDATE (2026-03):** Migrated to `pallet_babe::RandomnessFromOneEpochAgo` — VRF-based,
> unbiasable epoch randomness. `pallet_insecure_randomness_collective_flip` removed.

~~**WARNING**: Uses `pallet_insecure_randomness_collective_flip` — block-hash based, manipulable by validators. Must migrate to BABE/VRF before mainnet.~~ **RESOLVED.**

### 1.6 construct_runtime! Pallet Index

| Index | Pallet | Type |
|-------|--------|------|
| 0 | System | Substrate |
| 1 | Timestamp | Substrate |
| 2 | Babe | Substrate |
| 3 | Grandpa | Substrate |
| 4 | Balances | Substrate |
| 5 | TransactionPayment | Substrate |
| 6 | Sudo | Substrate |
| 7 | *(removed)* | *(randomness now via BABE VRF)* |
| 8 | Contracts | Substrate |
| 9 | Economy | **BelizeChain** |
| 10 | Identity | **BelizeChain** |
| 11 | Governance | **BelizeChain** |
| 12 | Compliance | **BelizeChain** |
| 13 | Staking | **BelizeChain** |
| 14 | Oracle | **BelizeChain** |
| 15 | Payroll | **BelizeChain** |
| 16 | Interoperability | **BelizeChain** |
| 17 | BelizeX | **BelizeChain** |
| 18 | LandLedger | **BelizeChain** |
| 19 | Consensus | **BelizeChain** |
| 20 | Quantum | **BelizeChain** |
| 21 | Community | **BelizeChain** |
| 22 | Bns | **BelizeChain** |
| 23 | Mesh | **BelizeChain** |

### 1.7 Executive & Migrations

```
Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    migrations::CoordinatedUpgrade<Runtime>,
>;
```

### 1.8 SignedExtra (Transaction Extensions)

1. `CheckNonZeroSender` — reject zero-sender
2. `CheckSpecVersion` — runtime version gate
3. `CheckTxVersion` — transaction format version
4. `CheckGenesis` — genesis hash binding
5. `CheckEra` — mortality/immortal era
6. `CheckNonce` — replay prevention
7. `CheckWeight` — block weight limits
8. `ChargeTransactionPayment` — fee deduction

### 1.9 Runtime APIs Implemented

Core, Metadata, BlockBuilder, TaggedTransactionQueue, OffchainWorkerApi, BabeApi, SessionKeys, GrandpaApi, AccountNonceApi, TransactionPaymentApi, GenesisBuilder, ContractsApi.

### 1.10 Node Service

- **RPC**: System + TransactionPayment + custom `belizechain_getChainInfo`
- **Offchain Workers**: Enabled when configured
- **Benchmarking**: `new_benchmark_partial` available under `runtime-benchmarks` feature
- **Network Configs**: Development, Local Testnet, Public Testnet, Staging, Mainnet
- **Mainnet Guard**: `MAINNET_KEYS_CONFIGURED = false` prevents accidental deployment with placeholder keys

---

## 2. Per-Pallet Inventory

### 2.1 Economy Pallet (`pallet_economy`, index 9)

**Purpose**: bBZD stablecoin management, tourism-incentivized payments, and economic policy.

**File**: `pallets/economy/src/lib.rs` (980 lines)  
**PalletId**: `py/trsry`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | `pallet_balances` (DALLA) |
| `TourismDiscount` | Tourism payment DALLA incentive (default 0.5%) |
| `Treasury` | `PalletId` for fee collection |

#### Storage Items (8)

| Storage | Type | Description |
|---------|------|-------------|
| `TotalBbzdSupply` | `ValueQuery<u128>` | Total bBZD minted |
| `BbzdAccounts` | `Map<AccountId → BbzdAccount>` | Per-account bBZD balance & type |
| `ExchangeRate` | `ValueQuery<u128>` | BZD/DALLA exchange rate |
| `AuthorizedMinters` | `Map<AccountId → bool>` | Central Bank authorized minters |
| `AuthorizedBurners` | `Map<AccountId → bool>` | Authorized bBZD burners |
| `TransactionLog` | `Map<u64 → TransactionRecord>` | Audit trail |
| `TransactionCount` | `ValueQuery<u64>` | Next transaction ID |
| `EconomicHealth` | `ValueQuery<EconomicHealthMetrics>` | GDP-analogues |

#### Extrinsics (8)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `mint_bbzd` | Root | Create bBZD (Central Bank privilege) |
| 1 | `burn_bbzd` | Root | Destroy bBZD |
| 2 | `transfer_bbzd` | Signed | Peer-to-peer bBZD transfer |
| 3 | `set_exchange_rate` | Root | Update BZD/DALLA rate |
| 4 | `authorize_minter` | Root | Grant minting privilege |
| 5 | `tourism_payment` | Signed | Tourism payment with 2-8% DALLA incentive |
| 6 | `create_economic_account` | Signed | Open typed economic account (Personal/Business/Government/Tourism/Foreign) |
| 7 | `update_economic_health` | Root | Update GDP/inflation/employment metrics |

#### Events (8+)

BbzdMinted, BbzdBurned, BbzdTransferred, ExchangeRateUpdated, MinterAuthorized, TourismPaymentProcessed, EconomicAccountCreated, EconomicHealthUpdated.

#### Errors (14)

InsufficientBbzdBalance, AccountNotFound, InvalidAmount, UnauthorizedMinter, InvalidExchangeRate, AccountAlreadyExists, InvalidAccountType, MaxAccountsExceeded, MinterAlreadyAuthorized, MinterNotFound, InsufficientDallaForTourism, TourismIncentiveNotAvailable, SupplyOverflow, etc.

#### Hooks

None (no `on_initialize`/`on_finalize`).

#### Audit Fixes

- **H-20**: ExchangeRate sync safety check
- **H-21**: bBZD/DALLA exchange rate consistency

---

### 2.2 Identity Pallet (`pallet_identity`, index 10)

**Purpose**: Sovereign digital identity (DID), KYC/AML, SSN registry, passport registry, biometrics.

**File**: `pallets/identity/src/lib.rs` (1180 lines)  
**PalletId**: `py/ident`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `MaxAccountsPerIdentity` | 5 |
| `KycValidityBlocks` | 1,051,200 (~2 years) |
| `KycGraceBlocks` | 14,400 (~1 day) |
| `MaxHistoryLen` | 100 |
| `OperationFee` | 10 DALLA (testnet), 50 DALLA (mainnet) |
| `IssuerBondAmount` | 1,000 DALLA (testnet), 100,000 DALLA (mainnet) |

#### Storage Items (21)

Identities, SsnRecords, PassportRecords, BiometricRecords, LinkedAccounts, AccountToIdentity, NextIdentityId, SsnIssuers, PassportIssuers, BiometricIssuers, SsnStandardVersion, PassportStandardVersion, OperationFee, Paused, IssuerBonds, RateWindowStart, SsnIssueCount, PassportIssueCount, BiometricIssueCount, RateLimitSsn, RateLimitPassport, RateLimitBiometrics.

#### Extrinsics (20)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `create_identity` | Signed | Create DID (format: `did:blz:{hex}`) |
| 1 | `update_identity` | Signed | Update identity fields |
| 2 | `register_ssn` | Signed | Register SSN (issuer-only, rate-limited) |
| 3 | `register_passport` | Signed | Register passport (issuer-only) |
| 4 | `register_biometrics` | Signed | Register biometrics (issuer-only) |
| 5 | `verify_identity` | Signed | Verify identity (issuer-only) |
| 6 | `link_account` | Signed | Link secondary account (max 5) |
| 7 | `unlink_account` | Signed | Unlink secondary account |
| 8 | `add_ssn_issuer` | Root | Authorize SSN issuer |
| 9 | `remove_ssn_issuer` | Root | Remove SSN issuer |
| 10 | `add_passport_issuer` | Root | Authorize passport issuer |
| 11 | `remove_passport_issuer` | Root | Remove passport issuer |
| 12 | `add_biometric_issuer` | Root | Authorize biometric issuer |
| 13 | `remove_biometric_issuer` | Root | Remove biometric issuer |
| 14 | `set_operation_fee` | Root | Update operation fee |
| 15 | `toggle_pause` | Root | Emergency pause switch |
| 16 | `update_ssn_standard` | Root | Bump SSN standard version |
| 17 | `update_passport_standard` | Root | Bump passport standard version |
| 18 | `set_rate_limits` | Root | Configure rate limits per issuer type |
| 19 | `revoke_identity_verification` | Root | Revoke identity verification status |

#### Privacy Model

- SSN data stored as `blake2_256(ssn_number, salt)` — never plaintext on-chain
- Biometric hashes only; raw biometric data never enters runtime
- Passport numbers zero-knowledge committed

#### Exported Trait

```rust
pub trait BelizeKyc<AccountId> {
    fn kyc_level(who: &AccountId) -> u8;              // 0-4
    fn is_sanctioned(who: &AccountId) -> bool;
    fn compute_capacity(who: &AccountId) -> u32;
    fn can_participate_in_governance(who: &AccountId) -> bool;
    fn is_verified_at_level(who: &AccountId, level: u8) -> bool;
    fn record_community_participation(who: &AccountId, activity_type: u8);
}
```

KYC Levels: 0=None, 1=Basic, 2=Standard, 3=Enhanced, 4=Government.

---

### 2.3 Governance Pallet (`pallet_governance`, index 11)

**Purpose**: Multi-layered democratic governance with council, departments, foundation board, district elections, referendums, treasury management, and emergency "Jaguar Mode."

**File**: `pallets/governance/src/lib.rs` (6,067 lines — largest pallet)  
**PalletId**: `py/gover`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `ComplianceProvider` | KYC gate for governance participation |
| `CouncilOrigin` | Council collective origin |
| `MinimumDeposit` | 10 DALLA (runtime), 1,000 DALLA (genesis) |
| `VotingPeriod` | 7,200 blocks (~12h runtime) |
| `LaunchPeriod` | 14,400 blocks (~24h runtime) |
| `MaxCandidatesPerElection` | 200 |

#### Architecture

- **6 Districts**: Belize, Cayo, Corozal, OrangeWalk, StannCreek, Toledo
- **12 Council Seats**: 2 per district, max 5 representatives per district
- **8 Ministry Departments**: Finance, Education, Health, Works, Justice, Tourism, Agriculture, Defense
- **7-Member Foundation Board**: Founder(1), TechnicalSteward(2), FSCRepresentative(1), BTBDelegate(1), CitizenDelegate(3), SecurityAuditor(2), CultureEthicsAdvisor(1) = 11 slots total
- **8 Proposal Types**: Constitutional, Economic, Council, Technical, Emergency, International, Community, DistrictLocal
- **3 Voting Thresholds**: SimpleMajority(50%+), Supermajority(66%+), Unanimous(100%)

#### Storage Items (35+)

CouncilMembers, Proposals, Votes, VoteCommitments (pre-provisioned commit-reveal), CommunityRanks, PoUWContributions, NextProposalId, CouncilTerm, JaguarMode, DepartmentManagers (8 departments), DepartmentProposalCount, CrossDepartmentApprovals, BoardComposition, TermExpiryQueue, DelegateNominees, DelegateVoters, CurrentElection, GovernanceParameters (6 params), DepartmentTreasuryBalances, ExecutedProposals, PendingRuntimeUpgrade, DistrictElections, ElectionCandidates, ElectionVotes, DistrictRepresentation (max 5/district), VoteDelegations, DelegationReceivers (max 100/delegate), ProposalAmendments, ProposalQueue (max 50/priority), RewardsClaimed (H-26 per-type), TotalRewardsDistributed, Referendums, NextReferendumId, ReferendumVotes, ReferendumEligibleVoters, DistrictBudgets, TreasurySpendProposals, NextTreasuryProposalId, NationalTreasuryReserve, ChainParameters.

#### Extrinsics (37)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `submit_proposal` | Signed | Submit governance proposal (KYC, 8 types, deposit) |
| 1 | `cast_vote` | Signed | Vote with conviction (0=1x, 1=2x, 2=3x, 3=6x), weighted by rank+PoUW |
| 2 | `finalize_proposal` | Signed | Tally votes, approve/reject |
| 3 | `update_community_rank` | Root | Set council member rank |
| 4 | `update_pouw_contribution` | Root | Set PoUW contribution score |
| 5 | `council_override` | Council | Emergency override (Jaguar Mode only) |
| 6 | `set_department_manager` | Root | Assign department manager |
| 7 | `submit_department_proposal` | Signed | Department-specific proposal (manager only) |
| 8 | `approve_cross_department` | Signed | Cross-department approval |
| 9 | `add_board_member` | Council | Add foundation board member (term 1-2 years) |
| 10 | `remove_board_member` | Council | Remove foundation board member |
| 11 | `nominate_for_delegate` | Signed | Nominate for delegate election |
| 12 | `vote_for_delegate` | Signed | Vote in delegate election |
| 13 | `execute_proposal` | Signed | Execute approved proposal action |
| 14 | `start_district_election` | Root | Start district council election |
| 15 | `register_candidate` | Signed | Register as election candidate |
| 16 | `vote_in_district_election` | Signed | Cast vote in district election |
| 17 | `finalize_district_election` | Root | Tally results, assign seats (2-year terms) |
| 18 | `delegate_vote` | Signed | Liquid democracy delegation (max 100/delegate, 1yr expiry) |
| 19 | `revoke_delegation` | Signed | Revoke vote delegation |
| 20 | `amend_proposal` | Signed | Amend proposal before voting starts |
| 21 | `claim_participation_reward` | Signed | Claim: Vote=10 DALLA, Proposal=100 DALLA, Council=500 DALLA |
| 22 | `set_proposal_priority` | Root | Assign proposal priority (Low/Normal/High/Critical) |
| 23 | `declare_emergency` | Council/Root | Activate "Jaguar Mode" (600 blocks/hour) |
| 24 | `end_emergency` | Council/Root | Deactivate Jaguar Mode |
| 25 | `create_referendum` | Signed | Multi-option referendum (2-10 options, quorum) |
| 26 | `vote_on_referendum` | Signed | Weighted referendum vote |
| 27 | `finalize_referendum` | Signed | Tally referendum (quorum check) |
| 28 | `allocate_district_budget` | Root | Allocate fiscal year budget to district |
| 29 | `propose_treasury_spend` | Signed | Multi-sig treasury spend (<10K=1-of-1; <100K=3-of-7; ≥100K=4-of-7) |
| 30 | `approve_treasury_spend` | Signed | Sign multi-sig treasury proposal |
| 31 | `execute_treasury_proposal` | Signed | Execute fully-approved treasury spend |
| 32 | `transfer_district_budget` | Root | Transfer budget between districts |
| 33 | `execute_emergency_proposal` | Council/Root | Fast-track emergency proposal (Jaguar Mode, 66%+ supermajority) |
| 34 | `fast_track_referendum` | Council/Root | Reduce referendum deadline to 3 hours (Jaguar Mode) |
| 35 | `emergency_override_proposal` | Root | Force execute/reject proposal (Jaguar Mode, justification required) |
| 36 | `update_chain_parameter` | Root | Set governance-controlled chain parameter (key→u64) |

#### Execution Layer (Phase 5)

`execute_action()` dispatches 5 action types:
1. **TreasurySpend** — Transfer DALLA from governance treasury
2. **RuntimeUpgrade** — Store code hash in `PendingRuntimeUpgrade` (manual upgrade required)
3. **ParameterChange** — Update `GovernanceParameters` with range validation
4. **DepartmentAction** — Department-specific encoded call data (stub)
5. **EmergencyAction** — Activate/deactivate emergency, freeze/unfreeze (partial stub)

#### Emergency "Jaguar Mode"

- **EmergencyType**: Hurricane, HealthCrisis, EconomicCrisis, SecurityThreat, InfrastructureFailure, Other
- **Fast-track**: 1-hour launch / 3-hour voting for emergency proposals
- **Duration**: 600 blocks (~1 hour) per declaration
- **Override**: Root can force-execute/reject with justification

#### WeightInfo (7 functions, placeholder implementations)

---

### 2.4 Compliance Pallet (`pallet_compliance`, index 12)

**Purpose**: KYC/AML enforcement, sanctions screening, travel rule, verification tiers.

**File**: `pallets/compliance/src/lib.rs` (1,064 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `MinValidatorVerification` | Enhanced (L3) |
| `MinGovernanceVerification` | Government (L4) |
| `MinTreasuryVerification` | Government (L4) |
| `TravelRuleThreshold` | 100 DALLA |
| `VerificationValidityBlocks` | 100,800 (~1 week) |

#### Storage Items

ComplianceStatus, SanctionedEntities, TravelRuleRecords, VerificationProviders, WatchList.

#### Extrinsics (9)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_verification_provider` | Root | Register KYC provider |
| 1 | `submit_verification` | Signed | Submit KYC verification (tiers: Basic→Government) |
| 2 | `approve_verification` | Signed | Approve pending verification (provider only) |
| 3 | `apply_sanctions` | Root | Add entity to sanctions list |
| 4 | `remove_sanctions` | Root | Remove from sanctions list |
| 5 | `record_travel_rule` | Signed | FATF travel rule disclosure |
| 6 | `add_to_watch_list` | Root | Add account to monitoring watch list |
| 7 | `remove_from_watch_list` | Root | Remove from watch list |
| 8 | `update_compliance_status` | Root | Force-update compliance status |

#### Exported Traits

- `ComplianceCheck`: `can_transact()`, `transaction_limit()`, `is_sanctioned()`
- `OracleVerifier`: `is_authorized_operator()`

---

### 2.5 Staking Pallet (`pallet_staking` [custom], index 13)

**Purpose**: PoUW (Proof of Useful Work) validator staking with federated learning, quantum computing integration, and domain-specific bonus rewards.

**File**: `pallets/staking/src/lib.rs` (1,514 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA (lock-based staking) |
| `MinValidatorStake` | 100 DALLA |
| `MaxValidators` | Bounded set size |
| `BaseReward` | 1 DALLA/block |
| `MaxSupply` | 501B DALLA |
| `UnbondingPeriod` | Configurable blocks |
| `EpochDuration` | 14,400 blocks (~1 day) |
| `Identity` | `BelizeKyc<AccountId>` |
| `OracleVerifier` | `OracleVerifier<AccountId>` |

#### Storage Items (14+)

Validators, ValidatorCount, Delegations, ActiveFLTask, ModelSubmissions, EpochRewards, CurrentEpoch, SlashingSpans, PendingUnbonds, QuantumContributions, ValidatorQuantumStatsMap, EpochQuantumJobs, OperatorDomainStatsMap, EpochDomainContributions, LastClaimedEpoch.

#### Extrinsics (11)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `join_validators` | Signed | Join validator set (L3 KYC, sanctions check, min stake, compute_capacity ≥ 50) |
| 1 | `leave_validators` | Signed | Exit with unbonding period |
| 2 | `submit_model_delta` | Signed | Submit FL model update (entropy validation, S-5 quality score) |
| 3 | `assign_fl_task` | Root | Assign federated learning task |
| 4 | `distribute_rewards` | Root | Distribute epoch rewards (H-19 supply cap, bounded by MaxValidators) |
| 5 | `record_quantum_contribution` | Signed/Root | Record quantum computing contribution (oracle auth) |
| 6 | `force_join_validator` | Root | Bypass KYC for admin/testing |
| 7 | `record_domain_contribution` | Signed | Record domain-specific contribution (oracle auth) |
| 8 | `claim_pouw_with_domain_bonus` | Signed | Claim per-epoch reward with domain multiplier |
| 9 | `withdraw_unbonded` | Signed | Withdraw after unbonding period |
| 10 | `report_validator_offense` | Root | Report & slash validator |

#### Reward Formula (Phase 2.2 Enhanced)

```
total_score = FL_Quality × 25%
            + FL_Timeliness × 20%
            + FL_Honesty × 20%
            + Quantum_Score × 30%
            + Bonus × 5%
```

#### Domain Bonus Multipliers

| Domain | Index | Multiplier |
|--------|-------|------------|
| AgriTech | 1 | 1.5x |
| Marine | 2 | 1.4x |
| Education | 3 | 1.3x |
| Tech | 4 | 1.1x |
| General | 0 | 1.0x |

Fixed-point: base 10,000 = 1.0x

#### Quantum Score Formula

```
quantum_score = (jobs_completed / 100 × 40%)
              + (avg_accuracy × 35%)
              + (complexity / 1000 × 25%)
max: 100
```

#### Slashing

- `SlashReason`: InvalidProof(0), MissedDeadline(1), PrivacyBreach(2), ModelPoisoning(3), ConsensusViolation(4)
- `slash_validator`: Reduces stake, slashes free balance, updates lock, increments `SlashingSpans`
- **S-4 FIX**: Improved entropy analysis in `evaluate_model_quality` (byte distribution, unique bytes ratio)
- **S-5 FIX**: Use computed quality_score in event (not hardcoded 80)
- **H-19**: Supply-cap guard clamps reward to remaining headroom before MaxSupply

---

### 2.6 Oracle Pallet (`pallet_oracle`, index 14)

**Purpose**: Multi-operator oracle consensus, price feeds, IoT data, exchange rates, Nawal integration.

**File**: `pallets/oracle/src/lib.rs` (1,254 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `MaxOperators` | 10 |
| `MaxDataStaleness` | 100 blocks |
| `MinConsensusOperators` | 3 |
| `OracleStake` | Configurable |

#### Storage Items

DataFeeds, Operators, OperatorStake, ConsensusThreshold, OperatorStats, SecondaryOperators, AggregatedPrices, FeedProviders, ExchangeRates, PendingRewards, OperatorRewards, ClaimedRewards, KycLevel.

#### Extrinsics (13)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_operator` | Root | Register oracle operator |
| 1 | `submit_data` | Signed | Submit price/IoT data feed |
| 2 | `aggregate_prices` | Root | Aggregate via median consensus |
| 3 | `set_exchange_rate` | Root | Manual exchange rate (hierarchy: oracle → admin → genesis) |
| 4 | `add_secondary_operator` | Root | Add secondary operator |
| 5 | `remove_operator` | Root | Remove operator |
| 6 | `submit_iot_data` | Signed | IoT sensor data submission |
| 7 | `claim_operator_rewards` | Signed | Claim oracle rewards (M-49: stub, deferred to Phase 3) |
| 8 | `set_kyc_level` | Root | Set account KYC level |
| 9 | `update_consensus_threshold` | Root | Update min consensus operators |
| 10 | `get_aggregated_price` | Signed | Query aggregated price |
| 11 | `verify_data_freshness` | Signed | Check data staleness |
| 12 | `submit_nawal_telemetry` | Signed | Nawal drone telemetry feed |

#### Price Aggregation

Median-based consensus: collect from all operators, sort, take middle value. Deviation check against `MaxOracleDeviationBps` (500 bps = 5%).

#### Supported Currencies

BZD, USD, EUR, CAD, MXN — with exchange rate hierarchy.

#### Audit Fix

- **M49**: `claim_operator_rewards` marks stats but doesn't transfer DALLA — deferred to Phase 3 treasury integration.

---

### 2.7 Payroll Pallet (`pallet_payroll`, index 15)

**Purpose**: Privacy-preserving government payroll with scheduled payments, deductions, and department management.

**File**: `pallets/payroll/src/lib.rs` (1,464 lines)  
**PalletId**: `py/payrl`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `Treasury` | PalletId |
| `MaxEmployees` | Bounded (PR-2 fix) |

#### Storage Items

Employees, PaySchedules, PayHistory, Deductions, DepartmentBudgets, CostCenters, PayrollAdmin, PayrollPaused, NextPaymentId, ScheduledPayments.

#### Extrinsics (12)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_employee` | Root | Register employee (department, salary, schedule) |
| 1 | `remove_employee` | Root | Remove employee |
| 2 | `update_salary` | Root | Update salary amount |
| 3 | `process_payroll` | Root | Process batch payroll |
| 4 | `add_deduction` | Root | Add payroll deduction (tax, insurance, pension) |
| 5 | `remove_deduction` | Root | Remove deduction |
| 6 | `schedule_payment` | Root | Schedule future payment |
| 7 | `cancel_scheduled_payment` | Root | Cancel scheduled payment |
| 8 | `set_department_budget` | Root | Set department budget |
| 9 | `transfer_between_departments` | Root | Cross-department budget transfer |
| 10 | `set_payroll_admin` | Root | Set admin account |
| 11 | `toggle_payroll_pause` | Root | Pause/unpause payroll |

#### Hooks

- **`on_initialize`**: Processes scheduled payments due at current block.

#### Privacy

- Salary commitments stored as `blake2_256(salary, salt)` — amount not visible on-chain.
- **PR-2 FIX**: MaxEmployees bound prevents unbounded iteration DoS.

---

### 2.8 Interoperability Pallet (`pallet_interoperability`, index 16)

**Purpose**: Cross-chain bridge with multi-signature verification, challenge periods, and post-quantum signature support.

**File**: `pallets/interoperability/src/lib.rs` (1,486 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `MinBridgeAmount` | 50 DALLA |
| `BridgeFeeRate` | 1% |
| `PQSignatureThreshold` | 3-of-5 |
| `ChallengePeriod` | 100 blocks |
| `MaxChallengesPerBlock` | Bounded |

#### Storage Items (15+)

BridgeTransactions, TransactionCount, SupportedChains (51 chains), RegisteredAssets (DALLA, BBZD), Signatures, ChallengeRecords, CompletedBridges, PendingFinalizations, ActiveBridges, BridgeOperators, BridgeVolume, DailyBridgeVolume, PostQuantumSignatures, PQVerificationStatus, ChallengePeriodEnd.

#### Supported Chains (51)

Ethereum, Polkadot, Cosmos, BSC, Solana, Avalanche, + 45 more.

#### Extrinsics (11)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `initiate_bridge` | Signed | Start cross-chain transfer (min amount, fee deduction) |
| 1 | `submit_bridge_signature` | Signed | Multi-sig bridge verification |
| 2 | `challenge_bridge` | Signed | Challenge bridged transaction (deposit required) |
| 3 | `resolve_challenge` | Root | Resolve challenge (fund return or slash) |
| 4 | `finalize_bridge` | Signed | Finalize after challenge period |
| 5 | `register_bridge_operator` | Root | Register bridge relayer |
| 6 | `remove_bridge_operator` | Root | Remove bridge relayer |
| 7 | `register_supported_chain` | Root | Add supported chain |
| 8 | `register_asset` | Root | Register bridgeable asset |
| 9 | `submit_pq_signature` | Signed | Post-quantum CRYSTALS-Dilithium signature |
| 10 | `verify_pq_signatures` | Root | Verify PQ signature threshold |

#### Hooks

- **`on_initialize`**: Finalizes up to 20 bridges per block whose challenge period has elapsed.

#### Bridge Lifecycle

```
initiate → multi-sig (3-of-5) → challenge period (100 blocks) → finalize → unlock
```

#### Audit Fixes

B-1, B-2, B-9, M52, M53, H-35, H-36, H-37.

---

### 2.9 BelizeX Pallet (`pallet_belizex`, index 17)

**Purpose**: Automated Market Maker (AMM) DEX with liquidity pools, tourism discount, and protocol fee routing.

**File**: `pallets/belizex/src/lib.rs` (1,409 lines)  
**PalletId**: `py/bzdex`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `TradingFeeRate` | 0.3% |
| `TourismDiscount` | 0.5% |
| `ProtocolFeeToTreasuryBps` | 30% of trading fee → treasury |
| `Treasury` | PalletId |

#### Storage Items

LiquidityPools, LiquidityProviders, LPTokens, TotalLPSupply, TradePairs, TradeHistory, TradeCount, ProtocolFees, PoolCount, TourismAccounts.

#### Extrinsics (11)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `create_pool` | Root | Create AMM liquidity pool |
| 1 | `add_liquidity` | Signed | Provide liquidity, receive LP tokens |
| 2 | `remove_liquidity` | Signed | Burn LP tokens, withdraw assets |
| 3 | `swap` | Signed | Constant-product AMM swap (x*y=k) |
| 4 | `set_trading_fee` | Root | Update trading fee rate |
| 5 | `register_tourism_account` | Root | Register for tourism discount |
| 6 | `claim_protocol_fees` | Root | Withdraw accumulated protocol fees |
| 7 | `update_pool_status` | Root | Enable/disable pool |
| 8 | `get_price_quote` | Signed | Query estimated swap output |
| 9 | `batch_swap` | Signed | Multi-hop swap path |
| 10 | `set_pool_fee_override` | Root | Per-pool fee override |

#### AMM Model

Constant product: `x × y = k`. Slippage protection via `min_amount_out`.

---

### 2.10 LandLedger Pallet (`pallet_landledger`, index 18)

**Purpose**: On-chain land title registry with GPS coordinates, temporal anchoring, and title transfer.

**File**: `pallets/landledger/src/lib.rs` (993 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `LandRegistrationDeposit` | 100 DALLA |
| `TransferTaxRate` | 0.1% |

#### Storage Items

LandTitles, TitleCount, TransferHistory, Disputes, DisputeCount, TemporalAnchors.

#### Extrinsics (6)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_title` | Signed | Register land title with GPS bounds (Belize GPS validation) |
| 1 | `transfer_title` | Signed | Transfer with tax and deposit |
| 2 | `update_title` | Signed | Update title metadata |
| 3 | `file_dispute` | Signed | File title dispute |
| 4 | `resolve_dispute` | Root | Resolve title dispute |
| 5 | `create_temporal_anchor` | Signed | Create temporal proof anchor |

#### GPS Validation

Belize bounding box:
- Latitude: 15.8° to 18.5°
- Longitude: -89.3° to -87.5°

#### TemporalAnchoring

Uses `pallet_common::TemporalAnchoring` trait. Anchor types: Registration, Transfer, Dispute, Resolution.

#### Audit Fix

- **M65**: Depth-limit fix for nested temporal anchors.

---

### 2.11 Consensus Pallet (`pallet_consensus`, index 19)

**Purpose**: Federated AI PoUW consensus with model quality evaluation and encrypted model submission.

**File**: `pallets/consensus/src/lib.rs` (1,020 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `MinConsensusStake` | 50 DALLA |
| `MinModelQualityScore` | 60% |
| `ConsensusReward` | 5 DALLA/epoch |
| `Identity` | `BelizeKyc<AccountId>` |

#### Storage Items

ConsensusValidators, ModelSubmissions, ConsensusEpoch, EpochRewards, ValidatorScores, SlashingRecords.

#### Extrinsics (6)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_consensus_node` | Signed | Register as AI consensus validator |
| 1 | `submit_model_update` | Signed | Submit encrypted FL model delta |
| 2 | `evaluate_model` | Root | Evaluate model quality (min 60% score) |
| 3 | `distribute_consensus_rewards` | Root | Distribute epoch rewards |
| 4 | `report_consensus_violation` | Root | Report & slash validator |
| 5 | `update_consensus_parameters` | Root | Update consensus config |

---

### 2.12 Quantum Pallet (`pallet_quantum`, index 20)

**Purpose**: Quantum computing job management, NFT achievement system, marketplace with royalties, multi-validator verification, and cross-chain NFT bridge.

**File**: `pallets/quantum/src/lib.rs` (2,158 lines)

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `DallaPerQubit` | 0.000001 DALLA |
| `DallaPerShot` | Per-shot pricing |
| `NFTMintingFee` | 0.5 DALLA (burned = deflationary) |
| `Treasury` | PalletId |

#### Storage Items (17)

QuantumJobs, JobsByAccount (max 100), QuantumResults, QuantumAchievements, NFTCounter, TotalQuantumJobs, TotalDallaSpent, AccountStats, VerificationRequests, ValidatorReputation (0-1000), JobCounter, NFTListings, NFTAuctions, ListingCounter, BridgeRequests, BridgeCounter.

#### Quantum Backends (8)

IonQProvider, QuantinuumProvider, RigettiProvider, IBMQuantum, Qiskit, SpinQGemini, SpinQTriangulum, Other.

#### Achievement Types (12)

FirstQuantumJob, GroverAlgorithm, ShorAlgorithm, QFT, VQE, QAOA, Accuracy95, Accuracy99, Volume100, Volume1000, ErrorMitigationChampion, Custom.

#### Extrinsics (14)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `submit_quantum_job` | Signed | Submit job (qubits: 1-100, shots: 1-1M, cost = qubits×rate + shots×rate) |
| 1 | `update_job_status` | Root | State machine: Pending→Running→Completed/Failed (refund on failure) |
| 2 | `record_quantum_result` | Signed | Record result (proof ≥ 32 bytes) → enters Verifying status |
| 3 | `verify_quantum_result` | Root | Emergency override verification |
| 4 | `mint_achievement_nft` | Signed | Mint NFT (Q-2: immutable original_minter, rarity algorithm, fee burned) |
| 5 | `transfer_nft` | Signed | Transfer NFT (bridge lock check) |
| 6 | `list_nft` | Signed | List on marketplace (fixed-price with expiry) |
| 7 | `buy_nft` | Signed | Buy NFT (Q-1: 5% royalty to original_minter, 2% marketplace fee to treasury) |
| 8 | `delist_nft` | Signed | Remove from marketplace |
| 9 | `request_verification` | Signed | Multi-validator verification (2-10 validators, 100 block deadline) |
| 10 | `submit_verification` | Signed | Vote: Approve/Reject/Abstain, confidence 0-100 |
| 11 | `bridge_to_ethereum` | Signed | Bridge NFT to Ethereum (lock, 20/42 byte address) |
| 12 | `bridge_to_parachain` | Signed | Bridge NFT via XCM (lock) |
| 13 | `cancel_bridge` | Signed | Cancel bridge (unlock NFT) |

#### NFT Rarity Algorithm

```
base_score = qubit_count × 10
accuracy_bonus = if accuracy > 95% → 300, elif > 90% → 200, elif > 80% → 100, else 0
qubit_bonus = if qubits > 50 → 300, elif > 20 → 200, elif > 10 → 100, else 0
verification_bonus = if verified → 100, else 0
total = base_score + accuracy_bonus + qubit_bonus + verification_bonus
```

| Score Range | Rarity |
|------------|--------|
| 0-249 | Common |
| 250-499 | Rare |
| 500-749 | Epic |
| 750+ | Legendary |

#### NFT Categories

Volume, Accuracy, Complexity, Speed, Algorithm, Special.

#### Verification Reputation

- Correct vote: +10 (max 1000)
- Incorrect vote: -20 (min 0)

#### Metadata URI

`ipfs://QmBelizeChain/{rarity}/{category}/{nft_id}.json`

#### Audit Fixes

- **Q-1**: Marketplace fee routing to treasury (2% to treasury account)
- **Q-2**: Immutable `original_minter` field for royalty tracking

---

### 2.13 Community Pallet (`pallet_community`, index 21)

**Purpose**: Social Responsibility Score (SRS), education modules, green projects, cooperatives, referrals, micro-lending, and cultural preservation.

**File**: `pallets/community/src/lib.rs` (2,247 lines)  
**PalletId**: `py/comty`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `ProposalDepositPercentage` | 10% |
| `FeeExemptionMonthlyLimit` | 100 dBZD |
| `EducationRewardAmount` | 50 DALLA |
| `ReferralRewardAmount` | 100 DALLA |
| `CommunityVotingPeriod` | 7 days |
| `Treasury` | PalletId |
| `IdentityProvider` | `BelizeKyc` |

#### Storage Items (25+)

SocialScores, CommunityProposals, EducationModules, GreenProjects, Cooperatives, CooperativeMembers, ReferralCodes, Referrals, MicroLoans, LoanRepayments, FeeExemptions, CulturalRecords, ParticipationHistory, TransitPayments, VolunteerHours, EducationCompletions, GreenContributions, CommunityChallenges, StartupIncubator, DisasterReliefFund, NextProposalId, NextModuleId, NextProjectId, NextCooperativeId, NextLoanId, NextCulturalRecordId.

#### Extrinsics (13)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `submit_community_proposal` | Signed | Community governance proposal |
| 1 | `vote_community_proposal` | Signed | Community proposal vote |
| 2 | `complete_education_module` | Signed | Complete module, earn DALLA reward |
| 3 | `contribute_to_green_project` | Signed | Contribute DALLA to environmental project |
| 4 | `create_cooperative` | Signed | Form community cooperative |
| 5 | `join_cooperative` | Signed | Join existing cooperative |
| 6 | `generate_referral_code` | Signed | Generate unique referral code |
| 7 | `apply_referral` | Signed | Apply referral (both parties earn 100 DALLA) |
| 8 | `request_micro_loan` | Signed | Request micro-loan (SRS-based approval) |
| 9 | `repay_micro_loan` | Signed | Repay micro-loan installment |
| 10 | `register_fee_exemption` | Root | Register fee exemption (low-income, monthly limit) |
| 11 | `record_cultural_heritage` | Signed | Register cultural artifact/tradition |
| 12 | `create_education_module` | Root | Create new education module |

#### SRS Score Calculation

```
base_score = 0
+ governance_participation × 200
+ education_completions × 150
+ green_contributions × 100
+ volunteer_hours × 50
+ cooperative_membership × 100
+ referrals × 75
+ loan_repayment_rate × 250
```

Capped at 1000. Influences: micro-loan eligibility, fee exemptions, governance weight.

#### Genesis Data

- 4 Education Modules: Financial Literacy, Sustainable Farming, Digital Democracy, Blockchain/Web3
- 5 Green Projects: Barrier Reef Conservation (1M DALLA), Rainforest Protection (750K), Community Solar (500K), Zero-Waste (300K), Mangrove Restoration (400K)

#### Exported Traits

- `CommunityParticipation`: `record_participation()`, `get_social_score()`
- `FeeExemptionProvider`: `is_fee_exempt()`

---

### 2.14 BNS Pallet (`pallet_bns`, index 22)

**Purpose**: Blockchain Name Service — `.bz` domain registration, resolution, text records, and subdomains.

**File**: `pallets/bns/src/lib.rs` (1,399 lines)  
**PalletId**: `py/bznss`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `MaxDomainsPerAccount` | 100 |
| `MaxDomainLength` | 64 |
| `MinDomainLength` | 3 |
| `MaxTextRecords` | 20 per domain |

#### Storage Items

Domains, DomainsByAccount, DomainCount, TextRecords, Subdomains, SubdomainCount, ReservedNames, DomainAuctions, RegistrationFees, GracePeriodBlocks, RenewalPeriodBlocks.

#### Extrinsics (15)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_domain` | Signed | Register `.bz` domain (fee based on length) |
| 1 | `renew_domain` | Signed | Renew domain registration |
| 2 | `transfer_domain` | Signed | Transfer domain ownership |
| 3 | `set_resolver` | Signed | Set domain resolver address |
| 4 | `set_text_record` | Signed | Set text record (twitter, github, email, etc.) |
| 5 | `remove_text_record` | Signed | Remove text record |
| 6 | `register_subdomain` | Signed | Register subdomain (parent owner only) |
| 7 | `remove_subdomain` | Signed | Remove subdomain |
| 8 | `set_primary_domain` | Signed | Set primary domain for reverse resolution |
| 9 | `reserve_name` | Root | Reserve protected name |
| 10 | `release_reserved_name` | Root | Release reserved name |
| 11 | `start_auction` | Root | Start domain auction |
| 12 | `place_bid` | Signed | Place auction bid |
| 13 | `finalize_auction` | Root | Finalize auction |
| 14 | `set_registration_fees` | Root | Update fee schedule |

#### Fee Schedule (by domain length)

| Length | Fee |
|--------|-----|
| 3 chars | 1,000 DALLA |
| 4 chars | 500 DALLA |
| 5 chars | 100 DALLA |
| 6+ chars | 10 DALLA |

---

### 2.15 Mesh Pallet (`pallet_mesh`, index 23)

**Purpose**: Meshtastic LoRa mesh network integration with node management, relay mining, emergency alerts, and offline transaction bridging.

**File**: `pallets/mesh/src/lib.rs` (1,482 lines)  
**PalletId**: `py/meshx`

#### Config Trait

| Bound | Description |
|-------|-------------|
| `Currency` | DALLA |
| `MaxMeshNodes` | 5,000 |
| `MaxPendingMeshTx` | 1,000 |
| `MaxActiveAlerts` | 50 |
| `RelayRewardPerTx` | 0.1 DALLA |
| `NodeRegistrationDeposit` | 10 DALLA |
| `HeartbeatTimeout` | 10 minutes |
| `Identity` | `BelizeKyc<AccountId>` |

#### Node Roles (6)

Client, Router, RouterClient, Gateway, ValidatorRelay, EmergencyBeacon.

#### Storage Items (16+)

MeshNodes, NodesByRole, PendingTransactions, CompletedRelays, EmergencyAlerts, AlertsByDistrict, NodeHeartbeats, MeshMetrics, RelayMiningRewards, NodeCount, AlertCount, PendingTxCount, NetworkCoverage, SignalStrength, MeshGateways, OfflineTransactionQueue.

#### Extrinsics (14)

| # | Call | Origin | Description |
|---|------|--------|-------------|
| 0 | `register_mesh_node` | Signed | Register LoRa node (deposit, KYC, GPS, role) |
| 1 | `deregister_mesh_node` | Signed | Remove node (return deposit) |
| 2 | `submit_heartbeat` | Signed | Node liveness proof (timeout = 10 min) |
| 3 | `relay_transaction` | Signed | Two-phase relay mining (initiate relay + verify) |
| 4 | `verify_relay` | Root | Verify relay completion, issue reward |
| 5 | `submit_emergency_alert` | Signed | Emergency beacon alert (location, severity, district) |
| 6 | `resolve_emergency_alert` | Root | Resolve active alert |
| 7 | `update_node_firmware` | Root | Broadcast firmware update |
| 8 | `set_network_parameters` | Root | Update mesh config |
| 9 | `bridge_offline_transaction` | Signed | Bridge LoRa-relayed transaction to chain |
| 10 | `update_signal_map` | Signed | Submit signal strength data |
| 11 | `register_gateway` | Signed | Register mesh-to-chain gateway |
| 12 | `deregister_gateway` | Signed | Remove gateway |
| 13 | `submit_coverage_report` | Signed | Submit network coverage data |

#### Hooks

None (heartbeat verification is extrinsic-driven).

#### Audit Fixes

M57, M58, MH-6, #80, H-40.

---

### 2.16 Common Pallet (`pallet_common`)

**Purpose**: Shared types and traits used across pallets.

**File**: `pallets/common/src/lib.rs` (21 lines)

#### Exports

```rust
pub struct TemporalAnchor<AccountId, BlockNumber> {
    pub creator: AccountId,
    pub block_number: BlockNumber,
    pub anchor_type: AnchorType,
    pub data_hash: [u8; 32],
}

pub enum AnchorType { Registration, Transfer, Dispute, Resolution }

pub trait TemporalAnchoring<AccountId, BlockNumber> {
    fn create_anchor(creator: AccountId, anchor_type: AnchorType, data: &[u8]) -> TemporalAnchor<AccountId, BlockNumber>;
    fn verify_anchor(anchor: &TemporalAnchor<AccountId, BlockNumber>) -> bool;
}
```

---

## 3. Cross-Pallet Dependency Graph

### 3.1 Trait Dependency Matrix

```
                    Identity  Compliance  Oracle  Community
                    (BelizeKyc) (ComplianceCheck) (OracleVerifier) (CommunityParticip.)
                    ────────  ──────────  ──────  ─────────
Governance          ✓ (KYC)   ✓ (can_govern)                ✓ (record_participation)
Staking             ✓ (L3)                ✓ (auth)
Consensus           ✓ (KYC)
Community           ✓ (KYC)
Mesh                ✓ (KYC)
Compliance                                ✓ (auth)
Payroll                                                     
Economy                                   
BelizeX                                   
Quantum                                   
LandLedger          
Oracle                                    
BNS                 
Interoperability    
```

### 3.2 Runtime Adapter Structs (16 Cross-Pallet Bridges)

The runtime defines adapter structs that implement one pallet's trait using another pallet's storage. These are the primary integration points:

| Adapter | Implements | For Pallet | Using |
|---------|-----------|------------|-------|
| `IdentityKycAdapter` | `BelizeKyc<AccountId>` | Staking, Governance, Consensus, Community, Mesh | Identity pallet storage |
| `ComplianceAdapter` | `ComplianceCheck<AccountId>` | Governance (proposal eligibility) | Compliance pallet storage |
| `OracleVerifierAdapter` | `OracleVerifier<AccountId>` | Staking (oracle auth) | Oracle pallet storage |
| `CommunityParticipationAdapter` | `CommunityParticipation<AccountId>` | Governance (record_participation) | Community pallet storage |
| `IdentityForComplianceAdapter` | `BelizeKyc<AccountId>` | Compliance | Identity pallet storage |
| `IdentityForCommunityAdapter` | `BelizeKyc<AccountId>` | Community | Identity pallet storage |
| `IdentityForMeshAdapter` | `BelizeKyc<AccountId>` | Mesh | Identity pallet storage |
| `ComplianceForGovernanceAdapter` | `ComplianceCheck<AccountId>` | Governance | Compliance pallet storage |
| `OracleVerifierForStakingAdapter` | `OracleVerifier<AccountId>` | Staking | Oracle pallet storage |
| `CommunityForGovernanceAdapter` | `CommunityParticipation<AccountId>` | Governance | Community pallet storage |

**Architectural Concern (Audit §3.8)**: 14+ adapter structs with duplicated delegate implementations. Consider consolidating into a shared trait resolver pattern.

### 3.3 PalletId → Treasury Account Mapping

| Pallet | PalletId Bytes | Treasury Account |
|--------|---------------|-----------------|
| Economy | `py/trsry` | Fee collection, bBZD backing |
| Community | `py/comty` | Education rewards, green project funding |
| Identity | `py/ident` | Operation fees, issuer bonds |
| Payroll | `py/payrl` | Salary disbursement |
| BelizeX | `py/bzdex` | Protocol fees (30% of trading fees) |
| BNS | `py/bznss` | Domain registration fees |
| Mesh | `py/meshx` | Relay mining rewards, node deposits |
| Governance | `py/gover` | Governance rewards, treasury spends |

### 3.4 Data Flow Diagram

```
                              ┌──────────────┐
                              │   Identity   │ ◄──── SSN/Passport/Biometric issuers
                              │  (KYC Hub)   │
                              └──────┬───────┘
                                     │ BelizeKyc trait
                     ┌───────────────┼───────────────┬──────────────┐
                     ▼               ▼               ▼              ▼
              ┌──────────┐   ┌──────────────┐ ┌───────────┐  ┌──────────┐
              │ Staking  │   │  Governance  │ │ Consensus │  │   Mesh   │
              │ (PoUW)   │   │(Council/Dept)│ │ (Fed. AI) │  │(Meshtastic)│
              └────┬─────┘   └──────┬───────┘ └───────────┘  └──────────┘
                   │                │
         Oracle ◄──┘                │◄── Compliance (can_govern, sanctions)
         (Price/IoT)                │◄── Community (SRS, record_participation)
                                    │
                              ┌─────▼─────┐
                              │  Economy   │ ──── bBZD stablecoin
                              │ (Treasury) │
                              └─────┬──────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
             ┌──────────┐   ┌──────────┐    ┌──────────────┐
             │ BelizeX  │   │  Payroll  │   │Interoperability│
             │  (DEX)   │   │ (Salary)  │   │   (Bridge)     │
             └──────────┘   └──────────┘   └──────────────┘
                                                    │
                    ┌───────────────┬───────────────┤
                    ▼               ▼               ▼
             ┌──────────┐   ┌──────────┐    ┌──────────┐
             │   BNS    │   │LandLedger│    │ Quantum  │
             │(.bz DNS) │   │(Titles)  │    │(Jobs/NFT)│
             └──────────┘   └──────────┘    └──────────┘
```

---

## 4. Trust & Security Model

### 4.1 Origin Requirements by Pallet

| Pallet | Root-Only Calls | Council-Only | Signed (KYC-Gated) | Signed (Open) |
|--------|----------------|-------------|---------------------|---------------|
| Economy | 5/8 | 0 | 0 | 3/8 |
| Identity | 11/20 | 0 | 9/20 | 0 |
| Governance | 7/37 | 5/37 | 25/37 | 0 |
| Compliance | 6/9 | 0 | 3/9 | 0 |
| Staking | 4/11 | 0 | 7/11 | 0 |
| Oracle | 8/13 | 0 | 5/13 | 0 |
| Payroll | 12/12 | 0 | 0 | 0 |
| Interoperability | 5/11 | 0 | 6/11 | 0 |
| BelizeX | 5/11 | 0 | 6/11 | 0 |
| LandLedger | 1/6 | 0 | 5/6 | 0 |
| Consensus | 4/6 | 0 | 2/6 | 0 |
| Quantum | 1/14 | 0 | 13/14 | 0 |
| Community | 1/13 | 0 | 12/13 | 0 |
| BNS | 5/15 | 0 | 10/15 | 0 |
| Mesh | 3/14 | 0 | 11/14 | 0 |

**Key observation**: All `CouncilOrigin` checks currently resolve to `EnsureRoot<AccountId>` in the runtime — this is a pre-governance bootstrap configuration. Before mainnet, must be replaced with actual collective pallets or multi-sig.

### 4.2 KYC Level Requirements

| Level | Name | Required For |
|-------|------|-------------|
| 0 | None | Read-only access |
| 1 | Basic (Observer) | Basic transactions, delegation |
| 2 | Standard (Contributor) | Token transfers within limits |
| 3 | Enhanced | Validator staking, governance proposals |
| 4 | Government | Treasury operations, compliance administration |

### 4.3 Sanctions & Compliance Gates

- **Validator joining**: L3 KYC + sanctions check + compute_capacity ≥ 50
- **Governance proposals**: `can_participate_in_governance()` (Compliance adapter)
- **Treasury operations**: L4 (Government) verification required
- **Travel rule**: Automatic disclosure for transfers ≥ 100 DALLA
- **Bridge operations**: Min 50 DALLA, fee 1%, multi-sig verification

### 4.4 Slashing Conditions

| Pallet | Offense | Consequence |
|--------|---------|-------------|
| Staking | InvalidProof | Stake slash, free balance slash |
| Staking | MissedDeadline | Stake slash |
| Staking | PrivacyBreach | Stake slash |
| Staking | ModelPoisoning | Stake slash |
| Staking | ConsensusViolation | Stake slash |
| Consensus | Low quality score (<60%) | Reward withholding |
| Interoperability | Failed bridge challenge | Challenge deposit forfeited |
| Quantum | Incorrect verification vote | Reputation -20 |

### 4.5 Emergency Powers ("Jaguar Mode")

| Capability | Conditions |
|-----------|-----------|
| Fast-track proposals | 1hr launch / 3hr voting |
| Emergency execution | 66%+ supermajority |
| Override proposals | Root-only, justification required |
| Fast-track referendums | Reduce to 3-hour deadline |
| Council override | JaguarMode active |
| Duration | 600 blocks (~1 hour) per activation |

### 4.6 Critical Security Warnings

1. **RandomnessCollectiveFlip**: Block-hash based, manipulable by validators — must migrate to BABE/VRF
2. **Placeholder Weights**: All 16 custom pallets use `WeightInfo = ()` — zero weight allows block stuffing
3. **Sudo Present**: Must remove `pallet_sudo` before mainnet (mainnet config correctly sets `enable_sudo: false`)
4. **Oracle Rewards Stub**: `claim_operator_rewards` marks stats but doesn't transfer DALLA — incomplete
5. **Department Action Stub**: `execute_department_action` only emits event, doesn't decode call_data
6. **Mainnet Keys Guard**: `MAINNET_KEYS_CONFIGURED = false` prevents accidental deployment — correct

---

## 5. Economic Model

### 5.1 Token Supply & Distribution

| Parameter | Value |
|-----------|-------|
| Total Max Supply | 501 billion DALLA |
| Genesis Treasury (mainnet) | 100M DALLA |
| Staking Rewards Pool | 300M DALLA |
| Development Fund | 100M DALLA |
| Public Distribution | 100M DALLA |
| Existential Deposit | 0.001 DALLA |

### 5.2 Fee Schedule

| Activity | Fee | Destination |
|----------|-----|-------------|
| Transaction fee | Weight-based | TransactionPayment |
| Identity operation | 10-50 DALLA | Identity treasury |
| Issuer bond | 1K-100K DALLA | Locked |
| Land registration | 100 DALLA deposit | Locked |
| Land transfer tax | 0.1% | Treasury |
| Domain registration | 10-1,000 DALLA (by length) | BNS treasury |
| Trading fee (BelizeX) | 0.3% | 70% LP / 30% treasury |
| Bridge fee | 1% | Bridge treasury |
| NFT minting fee | 0.5 DALLA | **Burned** (deflationary) |
| NFT marketplace | 2% buyer / 5% royalty to minter | Treasury / Creator |
| Mesh node deposit | 10 DALLA | Locked |
| Governance proposal | Deposit (returned on approval) | Locked |

### 5.3 Reward Schedule

| Activity | Reward | Source |
|----------|--------|--------|
| Block validation (base) | 1 DALLA/block | Inflation |
| PoUW staking (per epoch) | Formula-based (§2.5) | Inflation (capped) |
| Domain bonus: AgriTech | 1.5x base | Inflation |
| Domain bonus: Marine | 1.4x base | Inflation |
| Domain bonus: Education | 1.3x base | Inflation |
| Consensus reward | 5 DALLA/epoch | Inflation |
| Relay mining | 0.1 DALLA/tx | Mesh treasury |
| Governance vote | 10 DALLA | Governance treasury |
| Governance proposal | 100 DALLA | Governance treasury |
| Council service | 500 DALLA/month | Governance treasury |
| Education completion | 50 DALLA | Community treasury |
| Referral (both parties) | 100 DALLA | Community treasury |
| Tourism payment | 2-8% DALLA incentive | Economy treasury |

### 5.4 Deflationary Mechanisms

1. **NFT Minting Fee Burn**: 0.5 DALLA burned per NFT mint
2. **Existential Deposit Reaping**: Accounts below 0.001 DALLA are reaped
3. **Slashing**: Slashed stakes removed from circulating supply

### 5.5 Supply Cap Guard (H-19)

`distribute_rewards` in the Staking pallet clamps each validator's reward to the remaining headroom before `MaxSupply`:

```
headroom = MaxSupply - TotalIssuance
reward = min(calculated_reward, headroom)
```

---

## 6. Upgrade Strategy

### 6.1 Migration Framework

**File**: `runtime/src/migrations.rs` (242 lines)

#### Architecture

- **CoordinatedUpgrade<T>**: Multi-step `OnRuntimeUpgrade` implementation
- **Version Tracking**: `CURRENT_RUNTIME_VERSION = 1` (on-chain storage via `twox_128` hash)
- **MG-1 FIX**: Hashed storage key instead of raw unhashed bytes
- **MG-2 FIX**: Per-step version advancement for rollback safety

#### Migration Lifecycle

```
1. Bump CURRENT_RUNTIME_VERSION
2. Add migration struct implementing OnRuntimeUpgrade
3. Register behind version guard: `if on_chain < N { ... }`
4. Implement pre_upgrade / post_upgrade for try-runtime validation
5. Submit runtime upgrade via governance proposal
```

#### Current State

- V0 → V1: Bootstrap migration (no-op, establishes version tracking)
- All subsequent migrations follow the same pattern

#### try-runtime Support

```rust
fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError>
fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError>
```

### 6.2 Runtime Upgrade via Governance

The governance pallet's `execute_proposal` can trigger `ProposalAction::RuntimeUpgrade`:
1. Stores code hash in `PendingRuntimeUpgrade`
2. **Does NOT call `frame_system::set_code`** — manual upgrade required
3. This is a safety measure: code hash is verified off-chain before actual upgrade

### 6.3 Chain Parameter Updates

`update_chain_parameter` (call_index 36) allows governance to modify operational constants without runtime upgrades:
- Key: up to 32 bytes (e.g., `b"blocks_per_yr"`)
- Value: u64
- Queryable via `chain_param(key, default)` helper

### 6.4 Build Profiles

| Profile | LTO | Codegen Units | Use Case |
|---------|-----|---------------|----------|
| `debug` | No | Default | Development |
| `release` | No | Default | Testing |
| `production` | Yes | 1 | Mainnet deployment |

---

## 7. Patterns & Anti-Patterns

### 7.1 Positive Patterns ✓

| Pattern | Description | Examples |
|---------|-------------|----------|
| **Bounded Iteration** | All storage iterations use `.take(N)` or bounded collections | Governance: `.take(1_000)`, Staking: bounded by MaxValidators |
| **Supply Cap Guard** | Rewards never exceed MaxSupply | H-19 in Staking `distribute_rewards` |
| **Privacy-by-Design** | Sensitive data stored as salted blake2_256 hashes | Identity: SSN, Payroll: salary commitments |
| **Per-Type Reward Claiming** | Prevents double-claiming across reward types | H-26 in Governance `RewardsClaimed<(AccountId, u8)>` |
| **Multi-Sig Treasury** | Large treasury spends require Foundation Board approval | Governance: 4-of-7 for ≥100K DALLA |
| **Challenge Period** | Bridge operations have dispute window | Interoperability: 100 blocks challenge |
| **Mainnet Key Guard** | Compile-time flag prevents placeholder key deployment | `MAINNET_KEYS_CONFIGURED = false` |
| **GPS Validation** | Land titles validated against Belize bounding box | LandLedger: lat 15.8°-18.5°, lon -89.3° to -87.5° |
| **Per-Step Migration** | Version advanced per migration step for rollback safety | MG-2 in migrations.rs |
| **Conviction Voting** | Higher conviction = longer lock = more weight | Governance: 0=1x, 1=2x, 2=3x, 3=6x |
| **Deflationary NFT Minting** | Minting fee is burned, not collected | Quantum: 0.5 DALLA burned per mint |
| **Entropy Validation** | FL model submissions checked for non-trivial entropy | S-4 in Staking `evaluate_model_quality` |
| **Immutable Royalties** | NFT `original_minter` set once, never modified | Q-2 in Quantum |

### 7.2 Anti-Patterns & Risks ⚠

| Severity | Issue | Location | Impact | Recommendation |
|----------|-------|----------|--------|----------------|
| **CRITICAL** | Placeholder weights (`WeightInfo = ()`) | All 16 custom pallets | Block stuffing, DoS, incorrect fee calculation | Run FRAME benchmarking for all extrinsics before mainnet |
| **CRITICAL** | Insecure randomness | `pallet_insecure_randomness_collective_flip` | Validator-manipulable randomness | Migrate to BABE/VRF-based randomness |
| **HIGH** | `pallet_sudo` present | Runtime construct_runtime | Single point of total control | Remove for mainnet (config already disables, but code remains) |
| **HIGH** | CouncilOrigin = EnsureRoot | Runtime Config | No actual collective governance | Replace with pallet_collective or multi-sig |
| **HIGH** | Oracle rewards stub | Oracle `claim_operator_rewards` | Operators cannot receive rewards | Complete Phase 3 treasury integration |
| **MEDIUM** | 14+ adapter structs | Runtime lib.rs | Code duplication, maintenance burden | Consolidate into trait resolver pattern |
| **MEDIUM** | Department action stub | Governance `execute_department_action` | Only emits event, no dispatch | Implement call_data decoding and dispatch |
| **MEDIUM** | `saturated_into` usage | Multiple pallets | Silent truncation on overflow | Prefer `try_into` with error handling |
| **MEDIUM** | RuntimeUpgrade stores hash only | Governance execution layer | Does not actually upgrade runtime | Implement or document manual upgrade procedure |
| **MEDIUM** | Commit-reveal voting pre-provisioned | Governance `VoteCommitments` | Storage allocated but not wired | Complete commit-reveal implementation or remove dead storage |
| **LOW** | Tests partially disabled | Governance: `// mod tests;` | Reduced test coverage | Fix test compilation issues |
| **LOW** | Hardcoded block times | Multiple pallets | Assumes 6-second blocks | Use `pallet_timestamp` for dynamic calculation |
| **LOW** | IPFS metadata URI template | Quantum NFTs | Not actually pinned to IPFS | Implement IPFS integration or use on-chain metadata |
| **LOW** | ValidatorCount lazy backfill | Staking | Extra computation on first access | Initialize properly in genesis |

### 7.3 Documentation Quality

**Strengths**:
- Extensive inline rustdoc with tables, complexity annotations, examples
- Audit fix tags (H-XX, M-XX, S-XX, etc.) consistently referenced
- Phase-based development clearly delineated (Phase 1-8)
- SAFETY comments on all `saturated_into` usage

**Gaps**:
- No runtime API documentation for custom RPC endpoints (TODO in rpc.rs)
- No formal threat model document
- Missing benchmark results documentation
- No cross-pallet integration test documentation

### 7.4 Test Infrastructure

- Unit tests: Per-pallet `mod tests`, `mod tests_phase3..8` (governance)
- Integration tests: `tests/` directory with Python-based E2E tests
- Benchmark infrastructure: Feature-gated `mod benchmarking` in each pallet
- try-runtime: Full `pre_upgrade`/`post_upgrade` support in migrations

### 7.5 Mainnet Readiness Checklist

| Item | Status | Blocking |
|------|--------|----------|
| Benchmark all extrinsics | ❌ Not done | **YES** |
| Remove pallet_sudo | ❌ Code present | **YES** |
| Replace insecure randomness | ❌ Not done | **YES** |
| Replace CouncilOrigin with collective | ❌ Not done | **YES** |
| Generate real validator keys | ❌ Placeholder | **YES** |
| Complete oracle rewards | ❌ Stub | No (Phase 3) |
| Complete department actions | ❌ Stub | No (Phase 3) |
| Wire commit-reveal voting | ❌ Pre-provisioned | No (Phase 3) |
| Custom RPC APIs | ❌ Not done | No |
| Production telemetry | ❌ Basic only | No |
| Formal security audit | Partial | **YES** |
| Load/stress testing | ❌ Not done | **YES** |

---

## Appendix A: Workspace Dependency Graph (Cargo.toml)

```
workspace
├── node (binary)
│   └── depends on: runtime, sc-*, sp-*
├── runtime (wasm)
│   └── depends on: all pallets, frame-*, sp-*, pallet-*
├── pallets/common (lib, no-std)
├── pallets/economy (lib, no-std)
├── pallets/identity (lib, no-std)
├── pallets/governance (lib, no-std)
├── pallets/compliance (lib, no-std)
├── pallets/staking (lib, no-std)
├── pallets/oracle (lib, no-std)
├── pallets/payroll (lib, no-std)
├── pallets/interoperability (lib, no-std)
├── pallets/belizex (lib, no-std)
├── pallets/landledger (lib, no-std)
├── pallets/consensus (lib, no-std)
├── pallets/quantum (lib, no-std)
├── pallets/community (lib, no-std)
├── pallets/bns (lib, no-std)
└── pallets/mesh (lib, no-std)
```

**Snapshot SDK**: Polkadot SDK `stable2512` (git, branch-locked)
**Codec**: parity-scale-codec 3.6.12
**scale-info**: 2.11.1

## Appendix B: Genesis Configuration Summary

### Development/Local Testnet

| Parameter | Value |
|-----------|-------|
| Authorities | Alice (dev) / Alice+Bob (local) |
| Endowment | 1M DALLA per account |
| Sudo | Alice |
| Identity fee | 10 DALLA |
| Issuer bond | 1,000 DALLA |
| Governance deposit | 1,000 DALLA |
| Governance council | Alice, Bob, Charlie |
| Launch period | 28,800 blocks (2 days) |
| Voting period | 43,200 blocks (3 days) |
| Education modules | 4 (Financial Literacy, Sustainable Farming, Digital Democracy, Blockchain/Web3) |
| Green projects | 5 (Barrier Reef 1M, Rainforest 750K, Solar 500K, Zero-Waste 300K, Mangrove 400K) |

### Mainnet (Planned)

| Parameter | Value |
|-----------|-------|
| Authorities | 3-5 validators (real keys required) |
| Treasury | 100M DALLA (multi-sig 4-of-7) |
| Staking pool | 300M DALLA |
| Identity fee | 50 DALLA |
| Issuer bond | 100,000 DALLA |
| Governance deposit | 10,000 DALLA |
| Launch period | 100,800 blocks (7 days) |
| Voting period | 201,600 blocks (14 days) |
| Sudo | **Disabled** |
| Faucet | **Disabled** |

## Appendix C: Audit Fix Registry

| ID | Severity | Pallet | Description |
|----|----------|--------|-------------|
| H-19 | High | Staking | Supply-cap guard in distribute_rewards |
| H-20 | High | Economy | ExchangeRate sync safety |
| H-21 | High | Economy | bBZD/DALLA rate consistency |
| H-24 | High | Identity | Rate limiting for issuers |
| H-26 | High | Governance | Per-type reward claiming |
| H-29 | High | Community | SRS score overflow protection |
| H-31 | High | Compliance | Travel rule threshold enforcement |
| H-35 | High | Interoperability | Bridge multi-sig threshold |
| H-36 | High | Interoperability | Challenge period enforcement |
| H-37 | High | Interoperability | Bridge finalization safety |
| H-38 | High | BNS | Domain ownership validation |
| H-39 | High | BNS | Reserved name protection |
| H-40 | High | Mesh | Heartbeat timeout enforcement |
| MH-6 | Medium-High | Mesh | Node deregistration deposit return |
| M47 | Medium | Identity | DID format validation |
| M48 | Medium | Governance | Delegation KYC check |
| M49 | Medium | Oracle | Reward claim stub |
| M50 | Medium | Payroll | Payment scheduling safety |
| M51 | Medium | Community | Education module capacity |
| M52 | Medium | Interoperability | Supported chain validation |
| M53 | Medium | Interoperability | Asset registration |
| M56 | Medium | BelizeX | LP token accounting |
| M57 | Medium | Mesh | Node role validation |
| M58 | Medium | Mesh | Relay verification |
| M59 | Medium | LandLedger | GPS bounds validation |
| M60 | Medium | Consensus | Model quality threshold |
| M61 | Medium | Quantum | Job cost calculation |
| M64 | Medium | BNS | Domain length pricing |
| M65 | Medium | LandLedger | Temporal anchor depth limit |
| S-4 | Standard | Staking | Entropy analysis in model quality |
| S-5 | Standard | Staking | Quality score event accuracy |
| Q-1 | Quantum | Quantum | Marketplace fee to treasury |
| Q-2 | Quantum | Quantum | Immutable original_minter |
| B-1 | Bridge | Interoperability | Bridge initiation validation |
| B-2 | Bridge | Interoperability | Bridge signature verification |
| B-9 | Bridge | Interoperability | Bridge completion finality |
| MG-1 | Migration | Runtime | Hashed storage key |
| MG-2 | Migration | Runtime | Per-step version advancement |
| PR-2 | Payroll | Payroll | MaxEmployees bounded iteration |
| P3§32 | Audit | Governance | ChainParameters governance-controlled |

---

*Report generated from complete source analysis of 16 custom pallets (19,177+ lines), runtime configuration (1,337 lines), migration framework (242 lines), node service (426 lines), chain specification (439 lines), and supporting infrastructure.*
