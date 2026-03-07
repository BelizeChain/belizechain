# BelizeChain Benchmarking Preparation Report

> **Generated**: Research extraction of all 167 extrinsic signatures across 13 pallets  
> **Purpose**: Pre-populate storage, write benchmark scaffolds, validate WeightInfo coverage  
> **Status**: Complete — all pallets fully analysed

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [WeightInfo Coverage Gap Analysis](#2-weightinfo-coverage-gap-analysis)
3. [Pallet-by-Pallet Reference](#3-pallet-by-pallet-reference)
   - [3.1 economy](#31-economy-8-extrinsics)
   - [3.2 identity](#32-identity-20-extrinsics)
   - [3.3 governance](#33-governance-37-extrinsics)
   - [3.4 compliance](#34-compliance-9-extrinsics)
   - [3.5 staking](#35-staking-11-extrinsics)
   - [3.6 interoperability](#36-interoperability-7-extrinsics)
   - [3.7 belizex](#37-belizex-9-extrinsics)
   - [3.8 landledger](#38-landledger-5-extrinsics)
   - [3.9 consensus](#39-consensus-6-extrinsics)
   - [3.10 payroll](#310-payroll-12-extrinsics)
   - [3.11 mesh](#311-mesh-14-extrinsics)
   - [3.12 quantum](#312-quantum-14-extrinsics)
   - [3.13 bns](#313-bns-15-extrinsics)
4. [Storage Pre-Population Requirements](#4-storage-pre-population-requirements)
5. [Priority Actions](#5-priority-actions)

---

## 1. Executive Summary

| Metric | Value |
|--------|-------|
| Total pallets | 13 |
| Total extrinsics | **167** |
| Total WeightInfo trait fns | **117** |
| Missing WeightInfo fns | **54** |
| Orphaned WeightInfo fns (no extrinsic) | **4** (economy) |
| Perfect coverage pallets | 6 (compliance, landledger, consensus, mesh, quantum + staking is close) |
| Critical gaps (>10 missing) | 2 (governance: 30, identity: 14) |

### Coverage by Pallet

| Pallet | Extrinsics | WeightInfo Fns | Missing | Coverage % |
|--------|:----------:|:--------------:|:-------:|:----------:|
| economy | 8 | 12 (+4 orphan) | 2 name-mismatch | ~75% |
| identity | 20 | 6 | **14** | **30%** |
| governance | 37 | 7 | **30** | **19%** |
| compliance | 9 | 9 | 0 | 100% |
| staking | 11 | 10 | 1 | 91% |
| interoperability | 7 | 6 | 1 | 86% |
| belizex | 9 | 8 | 1 | 89% |
| landledger | 5 | 5 | 0 | 100% |
| consensus | 6 | 6 | 0 | 100% |
| payroll | 12 | 7 | **5** | **58%** |
| mesh | 14 | 14 | 0 | 100% |
| quantum | 14 | 14 | 0 | 100% |
| bns | 15 | 13 | 2 | 87% |

---

## 2. WeightInfo Coverage Gap Analysis

### Critical: Extrinsics Without WeightInfo Benchmarks

These extrinsics use **inline weight constants** instead of `T::WeightInfo::*()` calls, meaning they will not benefit from runtime benchmarking.

#### governance (30 missing)

All of the extrinsics below currently use hard-coded inline `Weight::from_parts(...)`:

| call_index | Extrinsic | Needs WeightInfo |
|:----------:|-----------|:----------------:|
| 7 | `submit_department_proposal` | Yes |
| 8 | `approve_cross_department` | Yes |
| 9 | `add_board_member` | Yes |
| 10 | `remove_board_member` | Yes |
| 11 | `nominate_for_delegate` | Yes |
| 12 | `vote_for_delegate` | Yes |
| 13 | `execute_proposal` | Yes |
| 14 | `start_district_election` | Yes |
| 15 | `register_candidate` | Yes |
| 16 | `vote_in_district_election` | Yes |
| 17 | `finalize_district_election` | Yes |
| 18 | `delegate_vote` | Yes |
| 19 | `revoke_delegation` | Yes |
| 20 | `amend_proposal` | Yes |
| 21 | `claim_participation_reward` | Yes |
| 22 | `set_proposal_priority` | Yes |
| 23 | `declare_emergency` | Yes |
| 24 | `end_emergency` | Yes |
| 25 | `create_referendum` | Yes |
| 26 | `vote_on_referendum` | Yes |
| 27 | `finalize_referendum` | Yes |
| 28 | `allocate_district_budget` | Yes |
| 29 | `propose_treasury_spend` | Yes |
| 30 | `approve_treasury_spend` | Yes |
| 31 | `execute_treasury_proposal` | Yes |
| 32 | `transfer_district_budget` | Yes |
| 33 | `execute_emergency_proposal` | Yes |
| 34 | `fast_track_referendum` | Yes |
| 35 | `emergency_override_proposal` | Yes |
| 36 | `update_chain_parameter` | In trait but inline in dispatch |

#### identity (14 missing)

| call_index | Extrinsic | Needs WeightInfo |
|:----------:|-----------|:----------------:|
| 3 | `add_issuer` | Yes |
| 4 | `remove_issuer` | Yes |
| 5 | `set_standard_version` | Yes |
| 6 | `set_operation_fee` | Yes |
| 7 | `set_issuer_bond_amount` | Yes |
| 8 | `set_rate_limits` | Yes |
| 9 | `set_pause` | Yes |
| 13 | `revoke` | Yes (uses admin_simple) |
| 14 | `suspend` | Yes |
| 15 | `issuer_deposit_bond` | Yes |
| 16 | `issuer_withdraw_bond` | Yes |
| 17 | `flag_issuer` | Yes |
| 18 | `slash_issuer_bond` | Yes |
| 19 | `report_bad_attestation` | Yes |

#### payroll (5 missing)

| call_index | Extrinsic | Needs WeightInfo |
|:----------:|-----------|:----------------:|
| 7 | `verify_employer` | Yes |
| 8 | `toggle_employee_status` | Yes |
| 9 | `create_department` | Yes |
| 10 | `set_deduction` | Yes |
| 11 | `issue_bonus` | Yes |

#### Others (1 each)

| Pallet | call_index | Extrinsic |
|--------|:----------:|-----------|
| staking | 6 | `force_join_validator` |
| interoperability | 6 | `dispute_bridge_transaction` |
| belizex | 5 | `execute_multihop_trade` |
| bns | 13 | `rollback_content` |
| bns | 14 | `update_ssl_certificate` |

### Orphaned WeightInfo Functions (no corresponding extrinsic)

| Pallet | Function | Status |
|--------|----------|--------|
| economy | `send_remittance()` | No extrinsic — deprecated or planned |
| economy | `update_inflation()` | No extrinsic — deprecated or planned |
| economy | `update_peg_rate()` | No extrinsic — deprecated or planned |
| economy | `emergency_shutdown()` | No extrinsic — deprecated or planned |

### Name Mismatches (WeightInfo ↔ Extrinsic)

| Pallet | WeightInfo fn | Actual Extrinsic |
|--------|---------------|-----------------|
| economy | `issue_bbzd()` | `mint_bbzd` (#0) |
| economy | `pay_tourism_incentive()` | `process_tourism_payment` (#5) |

---

## 3. Pallet-by-Pallet Reference

### 3.1 economy (8 extrinsics)

**File**: `pallets/economy/src/lib.rs` (1070 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type Treasury: Get<PalletId>;
type TimeProvider: UnixTime;
type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type Oracle: OracleProvider;
type MaxSupply: Get<u128>;        // constant
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum TourismCategory { Accommodation, Tour, Transport, FoodBeverage, Shopping, Other }
enum RedemptionStatus { Pending, Processed }
```

#### Extrinsic Signatures

```rust
#[call_index(0)] pub fn mint_bbzd(origin, recipient: T::AccountId, amount: u128, deposit_reference: BoundedVec<u8, ConstU32<64>>) // Signed, authorized minter
#[call_index(1)] pub fn redeem_bbzd(origin, amount: u128, bank_account: BoundedVec<u8, ConstU32<64>>) // Signed
#[call_index(2)] pub fn process_redemption(origin, redemption_id: u64) // Signed, authorized minter
#[call_index(3)] pub fn set_minter_authorization(origin, account: T::AccountId, authorized: bool) // GovernanceOrigin
#[call_index(4)] pub fn update_reserves(origin, new_reserves: u128) // GovernanceOrigin
#[call_index(5)] pub fn process_tourism_payment(origin, vendor: T::AccountId, amount: BalanceOf<T>, category_id: u8) // Signed
#[call_index(6)] pub fn burn_dalla(origin, amount: BalanceOf<T>) // Signed
#[call_index(7)] pub fn governance_burn(origin, amount: BalanceOf<T>) // GovernanceOrigin
```

#### WeightInfo Trait (12 fns)

```rust
fn issue_bbzd() -> Weight;           // maps to mint_bbzd (name mismatch)
fn redeem_bbzd() -> Weight;
fn process_redemption() -> Weight;
fn set_minter_authorization() -> Weight;
fn update_reserves() -> Weight;
fn pay_tourism_incentive() -> Weight; // maps to process_tourism_payment (name mismatch)
fn send_remittance() -> Weight;       // ORPHAN — no extrinsic
fn update_inflation() -> Weight;      // ORPHAN — no extrinsic
fn update_peg_rate() -> Weight;       // ORPHAN — no extrinsic
fn emergency_shutdown() -> Weight;    // ORPHAN — no extrinsic
fn burn_dalla() -> Weight;
fn governance_burn() -> Weight;
```

#### Key Storage for Benchmark Pre-Population

- `AuthorizedMinters: StorageMap<AccountId, bool>` — must be set for `mint_bbzd`, `process_redemption`
- `PendingRedemptions: StorageMap<u64, RedemptionRequest>` — must be set for `process_redemption`
- `TotalBBZDIssued: StorageValue<u128>` — checked against MaxSupply in `mint_bbzd`

---

### 3.2 identity (20 extrinsics)

**File**: `pallets/identity/src/lib.rs` (1151 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type PalletId: Get<PalletId>;
type Treasury: Get<Self::AccountId>;
type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type RevokeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type Oracle: IdentityOracleProvider;
type MaxAccountsPerIdentity: Get<u32>;
type MaxNameLen: Get<u32>;
type MaxAnchorLen: Get<u32>;
type MaxIssuerCount: Get<u32>;
type KycValidityBlocks: Get<BlockNumberFor<Self>>;
type KycGraceBlocks: Get<BlockNumberFor<Self>>;
type MaxHistoryLen: Get<u32>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum AttributeType { Ssn = 0, Passport = 1, Biometrics = 2 }
enum AttestationStatus { Active, Revoked, Suspended }
enum KycLevel { L0, L1, L2, L3 }
```

#### Extrinsic Signatures

```rust
#[call_index(0)]  pub fn register_identity(origin, name: BoundedVec<u8, T::MaxNameLen>) // Signed
#[call_index(1)]  pub fn link_account(origin, new_account: T::AccountId) // Signed
#[call_index(2)]  pub fn update_did_doc(origin, cid: BoundedVec<u8, T::MaxAnchorLen>) // Signed
#[call_index(3)]  pub fn add_issuer(origin, attr: u8, issuer: T::AccountId) // AdminOrigin
#[call_index(4)]  pub fn remove_issuer(origin, attr: u8, issuer: T::AccountId) // AdminOrigin
#[call_index(5)]  pub fn set_standard_version(origin, attr: u8, version: u32) // AdminOrigin
#[call_index(6)]  pub fn set_operation_fee(origin, fee: BalanceOf<T>) // AdminOrigin
#[call_index(7)]  pub fn set_issuer_bond_amount(origin, amount: BalanceOf<T>) // AdminOrigin
#[call_index(8)]  pub fn set_rate_limits(origin, window: BlockNumberFor<T>, ssn: u32, passport: u32, biometrics: u32) // AdminOrigin
#[call_index(9)]  pub fn set_pause(origin, paused: bool) // AdminOrigin
#[call_index(10)] pub fn issue_ssn(origin, target: T::AccountId, hash: H256, anchor: BoundedVec<u8, T::MaxAnchorLen>, format_ok: bool) // Signed, authorized issuer
#[call_index(11)] pub fn issue_passport(origin, target: T::AccountId, hash: H256, anchor: BoundedVec<u8, T::MaxAnchorLen>, format_ok: bool) // Signed, authorized issuer
#[call_index(12)] pub fn issue_biometrics(origin, target: T::AccountId, anchor: BoundedVec<u8, T::MaxAnchorLen>) // Signed, authorized issuer
#[call_index(13)] pub fn revoke(origin, account: T::AccountId, attr: u8) // RevokeOrigin
#[call_index(14)] pub fn suspend(origin, account: T::AccountId, attr: u8) // RevokeOrigin
#[call_index(15)] pub fn issuer_deposit_bond(origin, attr: u8) // Signed
#[call_index(16)] pub fn issuer_withdraw_bond(origin, attr: u8) // Signed
#[call_index(17)] pub fn flag_issuer(origin, attr: u8, issuer: T::AccountId, flagged: bool) // AdminOrigin
#[call_index(18)] pub fn slash_issuer_bond(origin, attr: u8, issuer: T::AccountId, amount: BalanceOf<T>) // AdminOrigin
#[call_index(19)] pub fn report_bad_attestation(origin, attr: u8, issuer: T::AccountId, flag: bool, slash_amount: BalanceOf<T>) // AdminOrigin
```

#### WeightInfo Trait (6 fns)

```rust
fn register_identity() -> Weight;
fn link_account() -> Weight;
fn update_did() -> Weight;          // maps to update_did_doc
fn admin_simple() -> Weight;        // shared by admin extrinsics
fn issue_attestation() -> Weight;   // shared by issue_ssn/passport/biometrics
fn revoke() -> Weight;
```

#### Key Storage for Benchmark Pre-Population

- `Identities: StorageMap<AccountId, IdentityRecord>` — needed for link_account, update_did_doc, issue_*
- `AuthorizedIssuers: StorageDoubleMap<u8, AccountId, IssuerInfo>` — needed for issue_ssn/passport/biometrics, remove_issuer
- `Attestations: StorageDoubleMap<AccountId, u8, Attestation>` — needed for revoke, suspend
- `IssuerBonds: StorageDoubleMap<u8, AccountId, Balance>` — needed for issuer_withdraw_bond, slash_issuer_bond
- `OperationFee: StorageValue<Balance>` — fee charged on operations
- `Paused: StorageValue<bool>` — must be false for most ops

---

### 3.3 governance (37 extrinsics)

**File**: `pallets/governance/src/lib.rs` (6044 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type ComplianceProvider: ComplianceCheck<Self::AccountId>;
type CommunityParticipation: CommunityParticipationProvider<Self::AccountId>;
type MinimumDeposit: Get<BalanceOf<Self>>;
type LaunchPeriod: Get<BlockNumberFor<Self>>;
type VotingPeriod: Get<BlockNumberFor<Self>>;
type MaxCandidatesPerElection: Get<u32>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum ProposalType { Constitutional, Legislative, Budget, Administrative, Emergency, Regulatory, Environmental, Social }
enum VoteChoice { Approve, Reject, Abstain }
enum VotingThreshold { SimpleMajority, Supermajority, Unanimous, DepartmentApproval }
enum Department { Finance, Education, Health, Works, Justice, Tourism, Agriculture, Defense }
enum BoardRole { Chairman, Secretary, Member, TechnicalAdvisor, LegalCounsel, FinancialOfficer, CommunityRepresentative }
enum BelizeDistrict { Belize, Cayo, Corozal, OrangeWalk, StannCreek, Toledo }
enum EmergencyType { NaturalDisaster, SecurityBreach, EconomicCrisis, PublicHealth, InfrastructureFailure, Other }
enum ProposalPriority { Low = 0, Medium = 1, High = 2, Critical = 3 }
enum ElectionStatus { Registration, Voting, Finalized, Cancelled }
enum GovernanceParameter { VotingPeriod, LaunchPeriod, MinimumDeposit, SupermajorityThreshold, CouncilSize, EmergencyTimeout }
enum EmergencyActionType { ActivateEmergency, DeactivateEmergency, FreezeAccount, UnfreezeAccount, HaltGovernance, ResumeGovernance }
```

#### Extrinsic Signatures

```rust
// ===== CORE GOVERNANCE (0-6) =====
#[call_index(0)]  pub fn submit_proposal(origin, title: Vec<u8>, description: Vec<u8>, proposal_type_index: u8, threshold_index: u8, is_emergency: bool, district_index: Option<u8>) // Signed
#[call_index(1)]  pub fn cast_vote(origin, proposal_id: u32, vote_choice_index: u8, conviction: u8) // Signed
#[call_index(2)]  pub fn finalize_proposal(origin, proposal_id: u32) // Signed
#[call_index(3)]  pub fn update_community_rank(origin, account: T::AccountId, new_rank: u32) // Root
#[call_index(4)]  pub fn update_pouw_contribution(origin, account: T::AccountId, new_contribution: u32) // Root
#[call_index(5)]  pub fn council_override(origin, proposal_id: u32, override_reason: Vec<u8>) // CouncilOrigin
#[call_index(6)]  pub fn set_department_manager(origin, department_index: u8, manager: T::AccountId) // Root

// ===== DEPARTMENT SYSTEM (7-8) =====
#[call_index(7)]  pub fn submit_department_proposal(origin, department_index: u8, title: Vec<u8>, description: Vec<u8>, proposal_type_index: u8, threshold_index: u8, requires_cross_approval: bool) // Signed, dept manager
#[call_index(8)]  pub fn approve_cross_department(origin, proposal_id: u32, department_index: u8) // Signed, dept manager

// ===== BOARD SYSTEM (9-12) =====
#[call_index(9)]  pub fn add_board_member(origin, account: T::AccountId, role_index: u8, term_years: u8) // CouncilOrigin
#[call_index(10)] pub fn remove_board_member(origin, account: T::AccountId, reason: Vec<u8>) // CouncilOrigin
#[call_index(11)] pub fn nominate_for_delegate(origin, nominee: T::AccountId) // Signed
#[call_index(12)] pub fn vote_for_delegate(origin, nominee: T::AccountId) // Signed

// ===== EXECUTION LAYER (13) =====
#[call_index(13)] pub fn execute_proposal(origin, proposal_id: u32) // Signed, council member

// ===== DISTRICT ELECTIONS (14-17) =====
#[call_index(14)] pub fn start_district_election(origin, district_index: u8, seats: u32, registration_period_blocks: u32, voting_period_blocks: u32) // Root
#[call_index(15)] pub fn register_candidate(origin, district_index: u8, platform: Vec<u8>) // Signed
#[call_index(16)] pub fn vote_in_district_election(origin, district_index: u8, candidate: T::AccountId) // Signed
#[call_index(17)] pub fn finalize_district_election(origin, district_index: u8) // Root

// ===== DELEGATION (18-19) =====
#[call_index(18)] pub fn delegate_vote(origin, delegate: T::AccountId, expires_at: Option<BlockNumberFor<T>>) // Signed
#[call_index(19)] pub fn revoke_delegation(origin) // Signed, no params

// ===== AMENDMENTS & PRIORITY (20-22) =====
#[call_index(20)] pub fn amend_proposal(origin, proposal_id: u32, new_title: Option<BoundedVec<u8, ConstU32<256>>>, new_description: Option<BoundedVec<u8, ConstU32<1024>>>) // Signed, proposer only
#[call_index(21)] pub fn claim_participation_reward(origin, reward_type: u8) // Signed
#[call_index(22)] pub fn set_proposal_priority(origin, proposal_id: u32, priority: u8) // Root

// ===== EMERGENCY (23-24) =====
#[call_index(23)] pub fn declare_emergency(origin, emergency_type_index: u8, description: Vec<u8>, duration_hours: u32) // CouncilOrigin or Root
#[call_index(24)] pub fn end_emergency(origin) // CouncilOrigin or Root, no params

// ===== REFERENDA (25-27) =====
#[call_index(25)] pub fn create_referendum(origin, title: Vec<u8>, description: Vec<u8>, options: Vec<Vec<u8>>, quorum_percentage: u8, duration_blocks: u32, district_index: Option<u8>) // Signed
#[call_index(26)] pub fn vote_on_referendum(origin, referendum_id: u32, option_index: u8) // Signed
#[call_index(27)] pub fn finalize_referendum(origin, referendum_id: u32) // Signed

// ===== TREASURY & BUDGET (28-32) =====
#[call_index(28)] pub fn allocate_district_budget(origin, district_index: u8, amount: BalanceOf<T>, fiscal_year_blocks: u32) // Root
#[call_index(29)] pub fn propose_treasury_spend(origin, recipient: T::AccountId, amount: BalanceOf<T>, description: Vec<u8>, district_index: Option<u8>) // Signed, compliant
#[call_index(30)] pub fn approve_treasury_spend(origin, proposal_id: u32) // Signed, council member
#[call_index(31)] pub fn execute_treasury_proposal(origin, proposal_id: u32) // Signed
#[call_index(32)] pub fn transfer_district_budget(origin, from_district_index: u8, to_district_index: u8, amount: BalanceOf<T>, reason: Vec<u8>) // Root

// ===== EMERGENCY FAST-TRACK (33-35) =====
#[call_index(33)] pub fn execute_emergency_proposal(origin, proposal_id: u32) // Root or CouncilOrigin
#[call_index(34)] pub fn fast_track_referendum(origin, referendum_id: u32) // Root or CouncilOrigin
#[call_index(35)] pub fn emergency_override_proposal(origin, proposal_id: u32, execute: bool, justification: Vec<u8>) // Root

// ===== PARAMETER UPDATES (36) =====
#[call_index(36)] pub fn update_chain_parameter(origin, key: Vec<u8>, value: u64) // Root
```

#### WeightInfo Trait (7 fns)

```rust
fn submit_proposal() -> Weight;
fn cast_vote() -> Weight;
fn finalize_proposal() -> Weight;
fn update_community_rank() -> Weight;
fn update_pouw_contribution() -> Weight;
fn council_override() -> Weight;
fn update_chain_parameter() -> Weight;
```

#### Key Storage for Benchmark Pre-Population

- `Proposals: StorageMap<u32, Proposal>` — needed for vote, finalize, execute, amend, fast_track
- `CouncilMembers: StorageMap<AccountId, CouncilMember>` — needed for council_override, execute_proposal, approve_treasury_spend
- `DepartmentManagers: StorageMap<Department, AccountId>` — needed for submit_department_proposal, approve_cross_department
- `BoardMembers: StorageMap<AccountId, BoardMember>` — needed for remove_board_member
- `JaguarMode: StorageValue<EmergencyStatus>` — needed for end_emergency
- `DistrictElections: StorageMap<BelizeDistrict, Election>` — needed for register_candidate, vote_in_district_election, finalize_district_election
- `VoteDelegations: StorageMap<AccountId, VoteDelegation>` — needed for revoke_delegation
- `Referenda: StorageMap<u32, Referendum>` — needed for vote_on_referendum, finalize_referendum
- `TreasuryProposals: StorageMap<u32, TreasuryProposal>` — needed for approve/execute_treasury_spend
- `DistrictBudgets: StorageMap<BelizeDistrict, Budget>` — needed for transfer_district_budget
- `ChainParameters: StorageMap<BoundedVec<u8,32>, u64>` — needed for update_chain_parameter

---

### 3.4 compliance (9 extrinsics)

**File**: `pallets/compliance/src/lib.rs` (1017 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type TimeProvider: UnixTime;
type ComplianceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type SanctionsOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type MinKycVerification: Get<u32>;
type MinAmlVerification: Get<u32>;
type MinCftVerification: Get<u32>;
type TravelRuleThreshold: Get<BalanceOf<Self>>;
type MaxAuditRecords: Get<u32>;
type MaxSuspiciousActivityReports: Get<u32>;
type WeightInfo: WeightInfo;
```

#### Extrinsic Signatures

```rust
#[call_index(0)] pub fn verify_account(origin, account: T::AccountId, verification_level: u8, kyc_hash: [u8; 32]) // ComplianceOrigin
#[call_index(1)] pub fn update_risk_level(origin, account: T::AccountId, new_risk_level: u8) // ComplianceOrigin
#[call_index(2)] pub fn whitelist_account(origin, account: T::AccountId) // ComplianceOrigin
#[call_index(3)] pub fn restrict_account(origin, account: T::AccountId, reason: Vec<u8>) // ComplianceOrigin
#[call_index(4)] pub fn lift_restriction(origin, account: T::AccountId) // ComplianceOrigin
#[call_index(5)] pub fn flag_suspicious_activity(origin, account: T::AccountId, activity_type: u8, description: Vec<u8>) // ComplianceOrigin
#[call_index(6)] pub fn add_sanctions_entry(origin, entity: Vec<u8>, reason: Vec<u8>) // SanctionsOrigin
#[call_index(7)] pub fn remove_sanctions_entry(origin, entity: Vec<u8>) // SanctionsOrigin
#[call_index(8)] pub fn sync_verification_from_identity(origin, account: T::AccountId) // ComplianceOrigin
```

#### WeightInfo Trait (9 fns) — **100% coverage**

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

---

### 3.5 staking (11 extrinsics)

**File**: `pallets/staking/src/lib.rs` (1481 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
type OracleVerifier: OracleVerification;
type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
type Identity: StakingIdentityProvider<Self::AccountId>;
type MaxValidators: Get<u32>;
type MinValidatorStake: Get<BalanceOf<Self>>;
type BaseReward: Get<BalanceOf<Self>>;
type EpochDuration: Get<BlockNumberFor<Self>>;
type UnbondingPeriod: Get<BlockNumberFor<Self>>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum SlashReason { Equivocation, Unavailability, InvalidFL, MaliciousFL }
// Domain index: AgriTech=0, Marine=1, Education=2, Tech=3, General=4
```

#### Extrinsic Signatures

```rust
#[call_index(0)]  pub fn join_validators(origin, stake: BalanceOf<T>, compute_capacity: u32, location: BoundedVec<u8, ConstU32<64>>) // Signed
#[call_index(1)]  pub fn leave_validators(origin) // Signed, no params
#[call_index(2)]  pub fn submit_model_delta(origin, task_id: u32, encrypted_delta: BoundedVec<u8, ConstU32<1024>>, computation_commitment: [u8; 32], computation_log: [u8; 32]) // Signed
#[call_index(3)]  pub fn assign_fl_task(origin, task_id: u32, model_hash: [u8; 32], computation_time: u32, reward_multiplier: Perbill, deadline_blocks: BlockNumberFor<T>) // Root
#[call_index(4)]  pub fn distribute_rewards(origin) // Root, no params
#[call_index(5)]  pub fn record_quantum_contribution(origin, job_id: BoundedVec<u8, ConstU32<64>>, validator: T::AccountId, num_qubits: u16, circuit_depth: u32, num_shots: u32, accuracy_score: u8) // Signed/Root, oracle
#[call_index(6)]  pub fn force_join_validator(origin, who: T::AccountId, stake: BalanceOf<T>, compute_capacity: u32, location: BoundedVec<u8, ConstU32<64>>) // Root
#[call_index(7)]  pub fn record_domain_contribution(origin, operator: T::AccountId, domain: u8, quality_score: u8, volume_kb: u32) // Signed/Root, oracle
#[call_index(8)]  pub fn claim_pouw_with_domain_bonus(origin) // Signed, no params
#[call_index(9)]  pub fn withdraw_unbonded(origin) // Signed, no params
#[call_index(10)] pub fn report_validator_offense(origin, validator: T::AccountId, slash_percent: u32, reason_code: u8) // Root
```

#### WeightInfo Trait (10 fns)

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
// MISSING: force_join_validator
```

#### Key Storage for Benchmark Pre-Population

- `Validators: StorageMap<AccountId, ValidatorInfo>` — needed for leave_validators, submit_model_delta, distribute_rewards
- `FLTasks: StorageMap<u32, FLTask>` — needed for submit_model_delta
- `ValidatorQuantumStatsMap: StorageMap<AccountId, QuantumStats>` — updated by record_quantum_contribution
- `OperatorDomains: StorageMap<AccountId, OperatorDomainBreakdown>` — updated by record_domain_contribution
- `UnbondingQueue: StorageMap<AccountId, UnbondingInfo>` — needed for withdraw_unbonded
- `SlashingSpans: StorageMap<AccountId, u32>` — updated by report_validator_offense

---

### 3.6 interoperability (7 extrinsics)

**File**: `pallets/interoperability/src/lib.rs` (1178 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
type Time: UnixTime;
type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type Treasury: Get<Self::AccountId>;
type MaxBridgeValidators: Get<u32>;
type MinBridgeAmount: Get<BalanceOf<Self>>;
type BridgeFeeRate: Get<Permill>;
type PQSignatureThreshold: Get<u32>;
type ChallengePeriod: Get<BlockNumberFor<Self>>;
type Identity: IdentityProvider<Self::AccountId>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum BridgeChain { /* 51 variants */ }
enum BridgeAsset { DALLA, BBZD }
enum BridgeStatus { Pending, PartiallyConfirmed, Confirmed, Completed, Disputed, Cancelled }
```

#### Extrinsic Signatures

```rust
#[call_index(0)] pub fn initiate_bridge(origin, destination_chain_index: u8, asset_index: u8, amount: BalanceOf<T>, recipient: BoundedVec<u8, ConstU32<64>>) // Signed
#[call_index(1)] pub fn provide_pq_signature(origin, tx_id: u64, signature: BoundedVec<u8, ConstU32<256>>) // Signed, bridge validator
#[call_index(2)] pub fn create_liquidity_pool(origin, chain_index: u8, asset_index: u8, initial_amount: BalanceOf<T>) // GovernanceOrigin
#[call_index(3)] pub fn process_unlock(origin, tx_id: u64) // Signed
#[call_index(4)] pub fn send_cross_chain_message(origin, destination_chain_index: u8, message: BoundedVec<u8, ConstU32<1024>>, fee: BalanceOf<T>) // Signed
#[call_index(5)] pub fn update_bridge_config(origin, new_fee_rate: Option<Permill>, new_min_amount: Option<BalanceOf<T>>, new_pq_threshold: Option<u32>) // GovernanceOrigin
#[call_index(6)] pub fn dispute_bridge_transaction(origin, tx_id: u64, reason: BoundedVec<u8, ConstU32<256>>) // Signed
```

#### WeightInfo Trait (6 fns)

```rust
fn initiate_bridge() -> Weight;
fn provide_signature() -> Weight;
fn create_liquidity_pool() -> Weight;
fn process_unlock() -> Weight;
fn send_message() -> Weight;
fn update_config() -> Weight;
// MISSING: dispute_bridge_transaction
```

#### Key Storage for Benchmark Pre-Population

- `BridgeTransactions: StorageMap<u64, BridgeTransaction>` — needed for provide_pq_signature, process_unlock, dispute
- `BridgeValidators: StorageMap<AccountId, bool>` — needed for provide_pq_signature
- `LiquidityPools: StorageDoubleMap<BridgeChain, BridgeAsset, Pool>` — needed for initiate_bridge
- `TransactionCounter: StorageValue<u64>` — auto-incremented

---

### 3.7 belizex (9 extrinsics)

**File**: `pallets/belizex/src/lib.rs` (1215 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
type TourismOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type PairListingOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type Treasury: Get<Self::AccountId>;
type TradingFeeRate: Get<Permill>;
type TourismDiscountRate: Get<Permill>;
type MinLiquidityAmount: Get<BalanceOf<Self>>;
type Kyc: KycProvider<Self::AccountId>;
type ProtocolFeeToTreasuryBps: Get<u32>;
type Oracle: OracleProvider;
type MaxOracleDeviationBps: Get<u32>;
type DexPalletId: Get<PalletId>;
type WeightInfo: WeightInfo;
```

#### Extrinsic Signatures

```rust
#[call_index(0)] pub fn create_trading_pair(origin, base_asset_id: u32, quote_asset_id: u32) // PairListingOrigin
#[call_index(1)] pub fn add_liquidity(origin, pair_id: u32, base_amount: BalanceOf<T>, quote_amount: BalanceOf<T>) // Signed
#[call_index(2)] pub fn execute_trade(origin, pair_id: u32, is_buy: bool, amount: BalanceOf<T>, min_received: BalanceOf<T>) // Signed
#[call_index(3)] pub fn register_tourism_trader(origin, business_name: Vec<u8>, business_type: u8)  // Signed
#[call_index(4)] pub fn place_limit_order(origin, pair_id: u32, is_buy: bool, price: BalanceOf<T>, amount: BalanceOf<T>) // Signed
#[call_index(5)] pub fn execute_multihop_trade(origin, path: Vec<u32>, amount_in: BalanceOf<T>, min_amount_out: BalanceOf<T>) // Signed
#[call_index(6)] pub fn pause_global(origin) // PairListingOrigin
#[call_index(7)] pub fn resume_global(origin) // PairListingOrigin
#[call_index(8)] pub fn set_pair_status(origin, pair_id: u32, active: bool) // PairListingOrigin
```

#### WeightInfo Trait (8 fns)

```rust
fn create_pair() -> Weight;
fn add_liquidity() -> Weight;
fn execute_trade() -> Weight;
fn register_tourism_trader() -> Weight;
fn place_order() -> Weight;
fn pause() -> Weight;
fn resume() -> Weight;
fn set_pair_status() -> Weight;
// MISSING: execute_multihop_trade
```

---

### 3.8 landledger (5 extrinsics)

**File**: `pallets/landledger/src/lib.rs` (946 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type GovernmentOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type SurveyorOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type EnvironmentalOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type Oracle: OracleProvider;
type RegistrationDeposit: Get<BalanceOf<Self>>;
type TransferTaxRate: Get<Permill>;
type MaxDescriptionLength: Get<u32>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum PropertyType { Residential, Commercial, Agricultural, Government, Industrial, Conservation, Tourism, MixedUse }
enum ZoningType { Residential, Commercial, Agricultural, Industrial, Conservation, Tourism, MixedUse, Special, Unzoned }
enum EncumbranceType { Mortgage, Lien, Easement, Restriction, TaxLien, EnvironmentalRestriction }
enum TransferType { Sale, Gift, Inheritance, GovernmentTransfer, CourtOrder, Foreclosure }
type PropertyId = [u8; 32];
```

#### Extrinsic Signatures

```rust
#[call_index(0)] pub fn register_property(origin, title_number: Vec<u8>, description: Vec<u8>, coordinates: (i64, i64), area_sqm: u32, property_type_index: u8, assessed_value: u128) // Signed
#[call_index(1)] pub fn transfer_property(origin, property_id: PropertyId, new_owner: T::AccountId, transfer_price: u128, transfer_type_index: u8) // Signed
#[call_index(2)] pub fn verify_property(origin, property_id: PropertyId) // GovernmentOrigin
#[call_index(3)] pub fn survey_property(origin, property_id: PropertyId, verified_area_sqm: u32, updated_coordinates: Option<(i64, i64)>) // Signed, authorized surveyor
#[call_index(4)] pub fn register_surveyor(origin, surveyor: T::AccountId) // GovernmentOrigin
```

#### WeightInfo Trait (5 fns) — **100% coverage**

```rust
fn register_property() -> Weight;
fn transfer_property() -> Weight;
fn verify_property() -> Weight;
fn survey_property() -> Weight;
fn register_surveyor() -> Weight;
```

#### Key Storage for Benchmark Pre-Population

- `Properties: StorageMap<PropertyId, Property>` — needed for transfer, verify, survey
- `AuthorizedSurveyors: StorageMap<AccountId, bool>` — needed for survey_property
- `PropertyOwners: StorageDoubleMap<AccountId, PropertyId, ()>` — ownership index

---

### 3.9 consensus (6 extrinsics)

**File**: `pallets/consensus/src/lib.rs` (986 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
type TimeProvider: UnixTime;
type AIAuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type MaxValidators: Get<u32>;
type MinConsensusStake: Get<BalanceOf<Self>>;
type MinModelQualityScore: Get<u32>;
type ConsensusReward: Get<BalanceOf<Self>>;
type Staking: ConsensusStakingProvider<Self::AccountId, BalanceOf<Self>>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum ModelType { NLP, ComputerVision, TimeSeries, Reinforcement, GAN, Transformer, GraphNeural, Quantum }
enum WorkType { Training, Inference, Validation, Aggregation, Optimization, Verification }
enum RoundStatus { Active, Completed, Failed, Pending }
```

#### Extrinsic Signatures

```rust
#[call_index(0)] pub fn register_ai_model(origin, model_type_index: u8, parameters_hash: [u8; 32], training_data_size: u32, pq_signature: Vec<u8>) // Signed
#[call_index(1)] pub fn join_consensus_validator(origin, stake_amount: Balance, pq_public_key: Vec<u8>) // Signed
#[call_index(2)] pub fn validate_ai_model(origin, model_id: u32, accuracy_score: u32) // AIAuthorityOrigin
#[call_index(3)] pub fn start_consensus_round(origin, duration_blocks: BlockNumberFor<T>) // AIAuthorityOrigin
#[call_index(4)] pub fn submit_ai_work(origin, model_id: u32, work_type_index: u8, result_hash: [u8; 32], computation_time: u32, pq_signature: Vec<u8>) // Signed, validator
#[call_index(5)] pub fn finalize_consensus_round(origin) // AIAuthorityOrigin, no params
```

#### WeightInfo Trait (6 fns) — **100% coverage**

```rust
fn register_ai_model() -> Weight;
fn join_validator() -> Weight;
fn validate_model() -> Weight;
fn start_consensus_round() -> Weight;
fn submit_ai_work() -> Weight;
fn finalize_consensus_round() -> Weight;
```

#### Key Storage for Benchmark Pre-Population

- `AIModels: StorageMap<u32, AIModel>` — needed for validate_ai_model, submit_ai_work
- `ConsensusValidators: StorageMap<AccountId, ValidatorInfo>` — needed for submit_ai_work
- `CurrentRound: StorageValue<RoundInfo>` — needed for submit_ai_work, finalize_consensus_round

---

### 3.10 payroll (12 extrinsics)

**File**: `pallets/payroll/src/lib.rs` (1458 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type TimeProvider: UnixTime;
type PalletId: Get<PalletId>;
type MaxEmployees: Get<u32>;
type MaxDeductions: Get<u32>;
type MaxDepartments: Get<u32>;
type MinimumPayment: Get<BalanceOf<Self>>;
type MaxSchedulesPerBlock: Get<u32>;
type VerifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type Oracle: PayrollOracleProvider;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum EmployerType { Individual, SmallBusiness, Corporation, NGO, GovernmentAgency, Cooperative }
enum WorkerType { FullTime, PartTime, Contract, Seasonal, Gig, Intern }
enum DeductionType { Tax, SocialSecurity, Insurance, Pension, Other }
enum PaymentFrequency { Weekly, BiWeekly, Monthly, Custom }
enum TokenType { Dalla, BBZD }
enum PaymentCategory { Salary, Bonus, Overtime, Commission, Reimbursement, Other }
```

#### Extrinsic Signatures

```rust
#[call_index(0)]  pub fn add_employee(origin, employee: T::AccountId, salary: BalanceOf<T>, worker_type: WorkerType, department_id: u32, metadata_hash: [u8; 32]) // Signed, verified employer
#[call_index(1)]  pub fn remove_employee(origin, employee: T::AccountId) // Signed
#[call_index(2)]  pub fn update_salary(origin, employee: T::AccountId, new_salary: BalanceOf<T>) // Signed
#[call_index(3)]  pub fn execute_payment(origin, employee: T::AccountId) // Signed
#[call_index(4)]  pub fn batch_payment(origin) // Signed, no params
#[call_index(5)]  pub fn create_schedule(origin, interval_blocks: u32, department_id: u32) // Signed, verified employer
#[call_index(6)]  pub fn update_schedule(origin, schedule_id: u32, interval_blocks: u32, active: bool) // Signed
#[call_index(7)]  pub fn verify_employer(origin, employer: T::AccountId, employer_type: EmployerType) // VerifierOrigin
#[call_index(8)]  pub fn toggle_employee_status(origin, employee: T::AccountId, active: bool) // Signed
#[call_index(9)]  pub fn create_department(origin, name_hash: [u8; 32]) // Signed, verified employer
#[call_index(10)] pub fn set_deduction(origin, employee: T::AccountId, deduction_type: DeductionType, amount: BalanceOf<T>) // Signed
#[call_index(11)] pub fn issue_bonus(origin, employee: T::AccountId, amount: BalanceOf<T>, category: PaymentCategory) // Signed
```

#### WeightInfo Trait (7 fns)

```rust
fn add_employee() -> Weight;
fn remove_employee() -> Weight;
fn update_salary() -> Weight;
fn execute_payment() -> Weight;
fn batch_payment(n: u32) -> Weight;  // parameterized
fn create_schedule() -> Weight;
fn update_schedule() -> Weight;
// MISSING: verify_employer, toggle_employee_status, create_department, set_deduction, issue_bonus
```

#### Key Storage for Benchmark Pre-Population

- `VerifiedEmployers: StorageMap<AccountId, EmployerInfo>` — needed for add_employee, create_schedule, create_department
- `Employees: StorageDoubleMap<AccountId(employer), AccountId(employee), Employee>` — needed for most ops
- `PaymentSchedules: StorageMap<u32, Schedule>` — needed for update_schedule
- `Departments: StorageDoubleMap<AccountId, u32, Department>` — needed for add_employee with dept

---

### 3.11 mesh (14 extrinsics)

**File**: `pallets/mesh/src/lib.rs` (1432 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type TimeProvider: UnixTime;
type Identity: MeshIdentityProvider<Self::AccountId>;
type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type EmergencyOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type MaxMeshNodes: Get<u32>;
type MaxPendingMeshTx: Get<u32>;
type MaxActiveAlerts: Get<u32>;
type MaxRelayProofsPerClaim: Get<u32>;
type RelayRewardPerTransaction: Get<BalanceOf<Self>>;
type RelayRewardPerBlockHeader: Get<BalanceOf<Self>>;
type RelayRewardPerEmergencyAlert: Get<BalanceOf<Self>>;
type NodeRegistrationDeposit: Get<BalanceOf<Self>>;
type HeartbeatTimeout: Get<BlockNumberFor<Self>>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
type MeshtasticNodeId = u32;
enum MeshNodeRole { Gateway, Relay, Validator, Sensor, Emergency, Router, Client }
enum MeshHardware { /* variants */ }
enum LoRaRegion { /* variants */ }
enum MeshTxType { /* variants */ }
enum RelayType { Transaction, BlockHeader, StateProof, EmergencyAlert, Heartbeat }
enum AlertSeverity { /* variants */ }
enum EmergencyType { /* variants */ }
enum ChannelPreset { /* variants */ }
enum BelizeDistrict { /* variants */ }
enum TerrainType { /* variants */ }
enum RelayDestination { /* variants */ }
```

#### Extrinsic Signatures

```rust
#[call_index(0)]  pub fn register_node(origin, node_id: MeshtasticNodeId, role: MeshNodeRole, hardware: MeshHardware, region: LoRaRegion, latitude: i32, longitude: i32, altitude: i16, district: BelizeDistrict, _terrain: TerrainType) // Signed
#[call_index(1)]  pub fn deregister_node(origin, node_id: MeshtasticNodeId) // Signed
#[call_index(2)]  pub fn update_node_location(origin, node_id: MeshtasticNodeId, latitude: i32, longitude: i32, altitude: i16) // Signed
#[call_index(3)]  pub fn node_heartbeat(origin, node_id: MeshtasticNodeId) // Signed
#[call_index(4)]  pub fn submit_mesh_transaction(origin, tx_hash: H256, tx_type: MeshTxType, sender_compact: [u8; 4], recipient_compact: [u8; 4], amount: u64, nonce: u32, signature_hash: H256, gateway_node_id: MeshtasticNodeId, relay_path: Vec<MeshtasticNodeId>, hop_count: u8, rssi: i16, snr: i16) // Signed, gateway owner
#[call_index(5)]  pub fn submit_relay_proof(origin, node_id: MeshtasticNodeId, relay_type: RelayType, content_hash: H256, source_node: MeshtasticNodeId, destination: RelayDestination, rssi: i16, snr: i16) // Signed
#[call_index(6)]  pub fn issue_emergency_alert(origin, severity: AlertSeverity, alert_type: EmergencyType, latitude: i32, longitude: i32, radius_meters: u32, message: Vec<u8>, duration_blocks: u32, district: BelizeDistrict) // EmergencyOrigin or Signed+authority
#[call_index(7)]  pub fn resolve_emergency_alert(origin, alert_id: u32) // EmergencyOrigin or Signed+authority
#[call_index(8)]  pub fn confirm_emergency_alert(origin, alert_id: u32, confirmer_node_id: MeshtasticNodeId) // Signed
#[call_index(9)]  pub fn relay_block_header(origin, relayer_node_id: MeshtasticNodeId, block_number: u32, block_hash: H256, parent_hash: H256, state_root: H256, extrinsics_root: H256, author_compact: [u8; 4], extrinsic_count: u16, timestamp: u32) // Signed, validator relay
#[call_index(10)] pub fn claim_relay_rewards(origin) // Signed, no params
#[call_index(11)] pub fn update_mesh_config(origin, max_hops: u8, channel_preset: ChannelPreset, relay_mining_active: bool, emergency_system_active: bool, validator_relay_active: bool, min_kyc_for_registration: u8, min_kyc_for_gateway: u8) // GovernanceOrigin
#[call_index(12)] pub fn fund_relay_rewards(origin, #[pallet::compact] amount: Balance) // Signed
#[call_index(13)] pub fn confirm_relay_proof(origin, relayer_node_id: MeshtasticNodeId, proof_index: u32) // Signed, different owner
```

#### WeightInfo Trait (14 fns) — **100% coverage** (inline T::WeightInfo)

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

#### Key Storage for Benchmark Pre-Population

- `MeshNodes: StorageMap<MeshtasticNodeId, MeshNode>` — needed for most node ops
- `NodeOwners: StorageMap<MeshtasticNodeId, AccountId>` — ownership check
- `PendingMeshTransactions: StorageMap<H256, MeshTransaction>` — needed for submit_mesh_transaction
- `ActiveAlerts: StorageMap<u32, EmergencyAlert>` — needed for resolve/confirm alert
- `RelayProofs: StorageMap<(MeshtasticNodeId, u32), RelayProof>` — needed for confirm_relay_proof
- `PendingRewards: StorageMap<AccountId, Balance>` — needed for claim_relay_rewards
- `RelayRewardPool: StorageValue<Balance>` — must have funds for claim_relay_rewards
- `MeshConfig: StorageValue<MeshConfiguration>` — read by most ops

---

### 3.12 quantum (14 extrinsics)

**File**: `pallets/quantum/src/lib.rs` (2148 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type MaxActiveJobs: Get<u32>;
type DallaPerQubit: Get<BalanceOf<Self>>;
type DallaPerShot: Get<BalanceOf<Self>>;
type NFTMintingFee: Get<BalanceOf<Self>>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum QuantumBackend { Simulator, IBM, Google, Rigetti, IonQ, AmazonBraket, Azure, Custom }
enum AchievementType { FirstQuantumJob, VolumeContributor100, VolumeContributor1000, Accuracy95, Accuracy99, GroverAlgorithm, ShorAlgorithm, QuantumFourierTransform, VQEAlgorithm, QAOAAlgorithm, ErrorMitigationChampion, Custom }
enum NFTRarity { Common, Rare, Epic, Legendary }
enum NFTCategory { Volume, Accuracy, Complexity, Speed, Algorithm, Special }
enum JobStatus { Pending, Running, Completed, Failed, Cancelled }
enum VerificationStatus { Unverified, Verified, Failed }
enum VerificationVote { Approve, Reject, Abstain }
enum ChainDestination { Ethereum, Parachain(u32) }
const MAX_JOB_ID_LENGTH: u32 = 64;
const MAX_PROOF_SIZE: u32 = 512;
const MAX_METADATA_URI_LENGTH: u32 = 256;
```

#### Extrinsic Signatures

```rust
// ===== QUANTUM JOB MANAGEMENT (0-3) =====
#[call_index(0)]  pub fn submit_quantum_job(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, backend_index: u8, circuit_hash: [u8; 32], num_qubits: u16, circuit_depth: u32, num_shots: u32) // Signed
#[call_index(1)]  pub fn update_job_status(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, new_status_index: u8) // Signed, submitter/executor
#[call_index(2)]  pub fn record_quantum_result(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, result_data_hash: [u8; 32], verification_proof: BoundedVec<u8, ConstU32<MAX_PROOF_SIZE>>, accuracy_score: u8) // Signed, executor
#[call_index(3)]  pub fn verify_quantum_result(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, verification_passed: bool) // Root

// ===== NFT MANAGEMENT (4-8) =====
#[call_index(4)]  pub fn mint_achievement_nft(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, achievement_type_index: u8, transferable: bool, circuit_qubits: u16, accuracy: u8) // Signed
#[call_index(5)]  pub fn transfer_nft(origin, nft_id: u64, to: T::AccountId) // Signed
#[call_index(6)]  pub fn list_nft(origin, nft_id: u64, price: Balance, duration: BlockNumberFor<T>) // Signed
#[call_index(7)]  pub fn buy_nft(origin, nft_id: u64) // Signed
#[call_index(8)]  pub fn delist_nft(origin, nft_id: u64) // Signed

// ===== MULTI-VALIDATOR VERIFICATION (9-10) =====
#[call_index(9)]  pub fn request_verification(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, required_verifications: u8) // Signed
#[call_index(10)] pub fn submit_verification(origin, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, vote_index: u8, confidence: u8, result_hash: [u8; 32]) // Signed

// ===== CROSS-CHAIN BRIDGE (11-13) =====
#[call_index(11)] pub fn bridge_to_ethereum(origin, nft_id: u64, recipient: BoundedVec<u8, ConstU32<64>>) // Signed
#[call_index(12)] pub fn bridge_to_parachain(origin, nft_id: u64, parachain_id: u32, recipient: BoundedVec<u8, ConstU32<64>>) // Signed
#[call_index(13)] pub fn cancel_bridge(origin, nft_id: u64) // Signed
```

#### WeightInfo Trait (14 fns) — **100% coverage**

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

#### Key Storage for Benchmark Pre-Population

- `QuantumJobs: StorageMap<BoundedVec<u8,64>, QuantumJob>` — needed for most job ops
- `QuantumResults: StorageMap<BoundedVec<u8,64>, QuantumResult>` — needed for verify, request_verification
- `QuantumAchievements: StorageMap<u64, QuantumNFT>` — needed for transfer/list/buy/bridge NFT ops
- `NFTListings: StorageMap<u64, NFTListing>` — needed for buy_nft, delist_nft
- `NFTAuctions: StorageMap<u64, NFTAuction>` — checked in bridge ops
- `BridgeRequests: StorageMap<u64, BridgeRequest>` — needed for cancel_bridge
- `VerificationRequests: StorageMap<BoundedVec<u8,64>, VerificationRequest>` — needed for submit_verification
- `ValidatorReputation: StorageMap<AccountId, u32>` — updated by consensus
- `JobsByAccount: StorageMap<AccountId, Vec<JobId>>` — auto-populated

---

### 3.13 bns (15 extrinsics)

**File**: `pallets/bns/src/lib.rs` (1388 lines)

#### Config Trait Bounds

```rust
type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
type TimeProvider: Time;
type Treasury: Get<Self::AccountId>;
type MaxDomainsPerAccount: Get<u32>;
type MaxDomainLength: Get<u32>;
type MaxTextRecords: Get<u32>;
type MinDomainLength: Get<u32>;
type Identity: BnsIdentityProvider<Self::AccountId>;
type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
type WeightInfo: WeightInfo;
```

#### Key Types

```rust
enum DomainTier { Standard, Premium, Government, Verified }
enum HostingTier { Free, Basic, Pro, Enterprise }
```

#### Extrinsic Signatures

```rust
#[call_index(0)]  pub fn register_domain(origin, domain_name: Vec<u8>, tier: u8) // Signed
#[call_index(1)]  pub fn set_resolution(origin, domain_name: Vec<u8>, wallet_address: Option<T::AccountId>, content_hash: Option<[u8; 32]>, metadata: Vec<u8>) // Signed
#[call_index(2)]  pub fn transfer_domain(origin, domain_name: Vec<u8>, new_owner: T::AccountId) // Signed
#[call_index(3)]  pub fn list_domain(origin, domain_name: Vec<u8>, price: u128, min_offer: Option<u128>, duration_blocks: BlockNumberFor<T>) // Signed
#[call_index(4)]  pub fn buy_domain(origin, domain_name: Vec<u8>, offer_price: u128) // Signed
#[call_index(5)]  pub fn unlist_domain(origin, domain_name: Vec<u8>) // Signed
#[call_index(6)]  pub fn activate_hosting(origin, domain_name: Vec<u8>, tier: u8, content_hash: [u8; 32], auto_renew: bool) // Signed
#[call_index(7)]  pub fn renew_hosting(origin, domain_name: Vec<u8>, months: u32) // Signed
#[call_index(8)]  pub fn deactivate_hosting(origin, domain_name: Vec<u8>) // Signed
#[call_index(9)]  pub fn update_hosting_content(origin, domain_name: Vec<u8>, new_content_hash: [u8; 32], description: Vec<u8>, size_bytes: u64) // Signed
#[call_index(10)] pub fn register_external_domain(origin, external_domain: Vec<u8>, linked_bns_domain: Vec<u8>, tier: u8) // Signed
#[call_index(11)] pub fn verify_external_domain(origin, external_domain: Vec<u8>) // Signed
#[call_index(12)] pub fn create_subdomain(origin, parent_domain: Vec<u8>, subdomain: Vec<u8>, delegate_to: Option<T::AccountId>) // Signed
#[call_index(13)] pub fn rollback_content(origin, domain_name: Vec<u8>, target_version: u32) // Signed
#[call_index(14)] pub fn update_ssl_certificate(origin, domain_name: Vec<u8>, cert_hash: [u8; 32], serial_number: Vec<u8>, issuer: Vec<u8>, expires_at: BlockNumberFor<T>) // Signed
```

#### WeightInfo Trait (13 fns, in `weights.rs`)

```rust
fn register_domain() -> Weight;
fn set_resolution() -> Weight;
fn transfer_domain() -> Weight;
fn list_domain() -> Weight;
fn buy_domain() -> Weight;
fn unlist_domain() -> Weight;
fn activate_hosting() -> Weight;
fn renew_hosting() -> Weight;
fn deactivate_hosting() -> Weight;
fn register_external_domain() -> Weight;
fn verify_external_domain() -> Weight;
fn update_hosting_content() -> Weight;
fn create_subdomain() -> Weight;
// MISSING: rollback_content, update_ssl_certificate
```

#### Key Storage for Benchmark Pre-Population

- `DomainRegistry: StorageMap<BoundedVec<u8>, DomainInfo>` — needed for most ops
- `AccountDomains: StorageMap<AccountId, BoundedVec<DomainName>>` — domain count check
- `DomainListings: StorageMap<BoundedVec<u8>, DomainListing>` — needed for buy_domain, unlist_domain
- `HostedWebsites: StorageMap<BoundedVec<u8>, HostingInfo>` — needed for renew/deactivate/update hosting
- `ExternalDomains: StorageMap<BoundedVec<u8>, ExternalDomainInfo>` — needed for verify_external_domain
- `ContentHistory: StorageDoubleMap<DomainName, u32, ContentVersion>` — needed for rollback_content
- `CurrentContentVersion: StorageMap<DomainName, u32>` — needed for rollback_content

---

## 4. Storage Pre-Population Requirements

Summary of storage items that **must be pre-populated** for benchmarks to exercise worst-case paths:

| Category | Storage Items | Pallets |
|----------|:------------:|:-------:|
| Account authorization maps | 8 | economy, identity, staking, landledger, payroll |
| Primary entity registries | 13 | All pallets — the main "thing" being operated on |
| Counter/index values | 5 | economy, interoperability, quantum, bns, mesh |
| Configuration/parameter stores | 6 | governance, mesh, consensus, belizex, bns |
| Cross-pallet identity/KYC state | 4 | identity→compliance, identity→mesh, identity→bns, compliance→governance |
| Financial pool/treasury state | 4 | interoperability, belizex, mesh, governance |
| Queue/pending operation stores | 5 | economy, staking, quantum, mesh, governance |

### Cross-Pallet Dependencies

```
identity ──provides──▶ compliance (KYC level)
identity ──provides──▶ mesh (MeshIdentityProvider)
identity ──provides──▶ bns (BnsIdentityProvider)
identity ──provides──▶ staking (StakingIdentityProvider)
identity ──provides──▶ belizex (KycProvider)
compliance ──provides──▶ governance (ComplianceCheck)
staking ──provides──▶ consensus (ConsensusStakingProvider)
```

---

## 5. Priority Actions

### P0 — Critical (before any benchmarks run)

1. **Add 30 missing governance WeightInfo functions** — largest gap; all using hardcoded weights
2. **Add 14 missing identity WeightInfo functions** — second largest gap
3. **Fix economy name mismatches** — `issue_bbzd` → `mint_bbzd`, `pay_tourism_incentive` → `process_tourism_payment`

### P1 — Important

4. **Add 5 missing payroll WeightInfo functions** — verify_employer, toggle_employee_status, create_department, set_deduction, issue_bonus
5. **Add missing single-function WeightInfo entries** in staking (`force_join_validator`), interoperability (`dispute_bridge_transaction`), belizex (`execute_multihop_trade`), bns (`rollback_content`, `update_ssl_certificate`)

### P2 — Cleanup

6. **Remove 4 orphaned economy WeightInfo functions** or implement corresponding extrinsics
7. **Audit `batch_payment(n: u32)`** — only parameterized weight function; ensure benchmark iterates over employee count
8. **Verify all pallets compile with `--features runtime-benchmarks`** — quantum benchmarking module is currently commented out

### Benchmark Execution Order (recommended)

```
1. landledger (5 extrinsics, 100% coverage, simple types)
2. consensus  (6 extrinsics, 100% coverage)
3. compliance (9 extrinsics, 100% coverage)
4. mesh       (14 extrinsics, 100% coverage)
5. quantum    (14 extrinsics, 100% coverage)
6. economy    (8 extrinsics, fix names first)
7. staking    (11 extrinsics, add 1 function)
8. belizex    (9 extrinsics, add 1 function)
9. interop    (7 extrinsics, add 1 function)
10. bns       (15 extrinsics, add 2 functions)
11. payroll   (12 extrinsics, add 5 functions)
12. identity  (20 extrinsics, add 14 functions)
13. governance(37 extrinsics, add 30 functions) ← LAST, biggest effort
```
