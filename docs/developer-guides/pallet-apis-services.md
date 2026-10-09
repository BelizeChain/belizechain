# Services Pallet APIs

**BNS • LandLedger • Payroll • Community • Contracts**

Comprehensive API reference for BelizeChain's service-oriented pallets.

---

## BNS Pallet (Belize Name Service)

**.bz domain registry with Pakit DAG hosting, marketplace, subdomains, and SSL records**

### Extrinsics

```rust
// call_index(0) — register a .bz domain. `tier` is a u8 index:
//   0=Standard, 1=Premium, 2=Government, 3=Verified
pub fn register_domain(origin: OriginFor<T>, domain_name: Vec<u8>, tier: u8) -> DispatchResult;

// call_index(1) — set wallet/content resolution
pub fn set_resolution(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    wallet_address: Option<T::AccountId>,
    content_hash: Option<[u8; 32]>,
    metadata: Vec<u8>,
) -> DispatchResult;

// call_index(2) — ownership
pub fn transfer_domain(origin: OriginFor<T>, domain_name: Vec<u8>, new_owner: T::AccountId) -> DispatchResult;

// call_index(3)–(5) — marketplace
pub fn list_domain(origin: OriginFor<T>, domain_name: Vec<u8>, price: u128, min_offer: Option<u128>, duration_blocks: BlockNumberFor<T>) -> DispatchResult;
pub fn buy_domain(origin: OriginFor<T>, domain_name: Vec<u8>, offer_price: u128) -> DispatchResult;
pub fn unlist_domain(origin: OriginFor<T>, domain_name: Vec<u8>) -> DispatchResult;

// call_index(6)–(9) — hosting. Hosting `tier` is a u8 index:
//   0=Free, 1=Basic, 2=Pro, 3=Enterprise
pub fn activate_hosting(origin: OriginFor<T>, domain_name: Vec<u8>, tier: u8, content_hash: [u8; 32], auto_renew: bool) -> DispatchResult;
pub fn renew_hosting(origin: OriginFor<T>, domain_name: Vec<u8>, months: u32) -> DispatchResult;
pub fn deactivate_hosting(origin: OriginFor<T>, domain_name: Vec<u8>) -> DispatchResult;
pub fn update_hosting_content(origin: OriginFor<T>, domain_name: Vec<u8>, new_content_hash: [u8; 32], description: Vec<u8>, size_bytes: u64) -> DispatchResult;

// call_index(10)–(12) — external domains and subdomains
pub fn register_external_domain(origin: OriginFor<T>, external_domain: Vec<u8>, linked_bns_domain: Vec<u8>, tier: u8) -> DispatchResult;
pub fn verify_external_domain(origin: OriginFor<T>, external_domain: Vec<u8>) -> DispatchResult;
pub fn create_subdomain(origin: OriginFor<T>, parent_domain: Vec<u8>, subdomain: Vec<u8>, delegate_to: Option<T::AccountId>) -> DispatchResult;

// call_index(13)–(14) — versioning and TLS
pub fn rollback_content(origin: OriginFor<T>, domain_name: Vec<u8>, target_version: u32) -> DispatchResult;
pub fn update_ssl_certificate(origin: OriginFor<T>, domain_name: Vec<u8>, cert_hash: [u8; 32], serial_number: Vec<u8>, issuer: Vec<u8>, expires_at: BlockNumberFor<T>) -> DispatchResult;

// call_index(15)–(19) — profile records
pub fn set_text_record(origin: OriginFor<T>, domain_name: Vec<u8>, key: Vec<u8>, value: Vec<u8>) -> DispatchResult;
pub fn remove_text_record(origin: OriginFor<T>, domain_name: Vec<u8>, key: Vec<u8>) -> DispatchResult;
pub fn set_avatar(origin: OriginFor<T>, domain_name: Vec<u8>, avatar: [u8; 32]) -> DispatchResult;
pub fn set_primary_domain(origin: OriginFor<T>, domain_name: Vec<u8>) -> DispatchResult;
pub fn clear_primary_domain(origin: OriginFor<T>) -> DispatchResult;
```

**Pakit integration:** uploading a website means `activate_hosting` (first time) or
`update_hosting_content` — there is no `update_domain_content` extrinsic.

### Storage

| Storage | Purpose |
|---|---|
| `DomainRegistry` | Domain records (`BoundedVec<u8, MaxDomainLength> → _`) |
| `AccountDomains` | Domains per account |
| `DomainResolution` | Wallet/content/metadata resolution |
| `PrimaryDomain` | Account → primary domain |
| `DomainListings` | Marketplace listings |
| `HostedWebsites` | Hosting records (tier, expiry, content) |
| `ExternalDomains` / `DomainVerification` | External domain linking + verification |
| `SSLCertificates` | TLS certificate records |
| `ContentHistory` / `CurrentContentVersion` | Versioned content (rollback support) |
| `TotalDomains` / `TotalHostingRevenue` / `TotalMarketplaceRevenue` | Aggregate counters |

Exact struct shapes: see `pallets/bns/src/lib.rs`.

### Events

`DomainRegistered`, `ResolutionUpdated`, `DomainTransferred`, `DomainListed`, `DomainSold`,
`HostingActivated`, `HostingRenewed`, `HostingFeeCollected`, `ExternalDomainRegistered`,
`ExternalDomainVerified`, `ExternalDomainVerificationRequested`, `SubdomainCreated`,
`ContentUpdated`, `ContentRolledBack`, `SSLCertificateUpdated`, `TextRecordSet`,
`TextRecordRemoved`, `AvatarUpdated`, `PrimaryDomainSet`, `PrimaryDomainCleared`

> Weights: `pallets/bns/src/weights.rs` (`./scripts/bench_weights.sh pallet_belize_bns`).

---

## LandLedger Pallet

**On-chain property registry with surveyor attestation, encumbrances, and Pakit anchor proofs**

### Extrinsics

```rust
// call_index(0) — register a property (title held off-chain / in Pakit)
pub fn register_property(
    origin: OriginFor<T>,
    title_number: Vec<u8>,
    description: Vec<u8>,
    coordinates: (i64, i64),
    area_sqm: u32,
    property_type_index: u8,
    assessed_value: u128,
) -> DispatchResult;

// call_index(1) — transfer ownership
pub fn transfer_property(
    origin: OriginFor<T>,
    property_id: PropertyId,
    new_owner: T::AccountId,
    transfer_price: u128,
    transfer_type_index: u8,
) -> DispatchResult;

// call_index(2) — governance verifies a property
pub fn verify_property(origin: OriginFor<T>, property_id: PropertyId) -> DispatchResult;

// call_index(3) — surveyor records a verified survey
pub fn survey_property(origin: OriginFor<T>, property_id: PropertyId, verified_area_sqm: u32, updated_coordinates: Option<(i64, i64)>) -> DispatchResult;

// call_index(4)–(5) — surveyor management
pub fn register_surveyor(origin: OriginFor<T>, surveyor: T::AccountId) -> DispatchResult;
pub fn remove_surveyor(origin: OriginFor<T>, surveyor: T::AccountId) -> DispatchResult;

// call_index(6)–(7) — encumbrances
pub fn add_encumbrance(origin: OriginFor<T>, property_id: u32, encumbrance_type: EncumbranceType, holder: T::AccountId, amount: Option<u128>, description: Vec<u8>) -> DispatchResult;
pub fn remove_encumbrance(origin: OriginFor<T>, property_id: u32, encumbrance_index: u32) -> DispatchResult;
```

**Pakit integration:** there is **no `register_document_proof` extrinsic**. Pakit storage proofs
are registered on the `StorageProof` pallet (`submit_storage_proof`); LandLedger keeps anchor
hashes (`LandAnchors`, `PropertyAnchorChain`, `ContentHashLatestAnchor`) for temporal anchoring.

### Storage

| Storage | Purpose |
|---|---|
| `Properties` | Property records (`PropertyId → _`) |
| `PropertyOwners` | Owner index |
| `TransferRecords` | Transfer history |
| `PropertyByTitle` | Title hash → property id |
| `NextPropertyId` / `NextTransferId` | Id counters |
| `GovernmentSurveyors` | Authorised surveyors |
| `ZoningMap` | Zoning records |
| `LandAnchors` / `PropertyAnchorChain` / `ContentHashLatestAnchor` | Temporal anchor proofs |

Encumbrances are a **field on `Properties`**, not separate storage.

### Events

`PropertyRegistered`, `PropertyTransferred`, `PropertyVerified`, `PropertySurveyed`,
`EnvironmentalClearanceGranted`, `EncumbranceAdded`, `EncumbranceRemoved`,
`SurveyorRegistered`, `SurveyorRemoved`

> Weights: `pallets/landledger/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_landledger`).

---

## Payroll Pallet

**Enterprise payroll: employees, departments, schedules, deductions, and bonuses**

### Extrinsics

```rust
// call_index(0) — add an employee
pub fn add_employee(
    origin: OriginFor<T>,
    employee: T::AccountId,
    salary: BalanceOf<T>,
    worker_type: WorkerType,
    department_id: u32,
    metadata_hash: [u8; 32],
) -> DispatchResult;

// call_index(1) — remove an employee
pub fn remove_employee(origin: OriginFor<T>, employee: T::AccountId) -> DispatchResult;

// call_index(2) — change salary
pub fn update_salary(origin: OriginFor<T>, employee: T::AccountId, new_salary: BalanceOf<T>) -> DispatchResult;

// call_index(3)–(4) — pay now
pub fn execute_payment(origin: OriginFor<T>, employee: T::AccountId) -> DispatchResult;
pub fn batch_payment(origin: OriginFor<T>) -> DispatchResult;

// call_index(5)–(6) — recurring schedules
pub fn create_schedule(origin: OriginFor<T>, interval_blocks: u32, department_id: u32) -> DispatchResult;
pub fn update_schedule(origin: OriginFor<T>, schedule_id: u32, interval_blocks: u32, active: bool) -> DispatchResult;

// call_index(7) — governance verifies an employer
pub fn verify_employer(origin: OriginFor<T>, employer: T::AccountId, employer_type: EmployerType) -> DispatchResult;

// call_index(8) — enable/disable an employee
pub fn toggle_employee_status(origin: OriginFor<T>, employee: T::AccountId, active: bool) -> DispatchResult;

// call_index(9) — create a department
pub fn create_department(origin: OriginFor<T>, name_hash: [u8; 32]) -> DispatchResult;

// call_index(10) — set a deduction
pub fn set_deduction(origin: OriginFor<T>, employee: T::AccountId, deduction_type: DeductionType, amount: BalanceOf<T>) -> DispatchResult;

// call_index(11) — issue a bonus
pub fn issue_bonus(origin: OriginFor<T>, employee: T::AccountId, amount: BalanceOf<T>, category: PaymentCategory) -> DispatchResult;
```

### Storage

| Storage | Purpose |
|---|---|
| `Employees` | `(employer, employee) → Employee` |
| `EmployeeDeductions` | Deductions per employee |
| `EmployerProfiles` | `AccountId → EmployerProfile` |
| `VerifiedEmployers` | Verified employer set |
| `Departments` | Department records |
| `PayrollSchedules` | Recurring schedules |
| `PayrollRecords` | Executed payment records (`id → _`) |
| `NextScheduleId` / `NextRecordId` | Id counters |
| `GlobalStats` · `EmployerCount` · `EmployeeCountPerEmployer` · `DeptEmployeeCount` | Aggregate counters |

Exact struct shapes: see `pallets/payroll/src/lib.rs`.

### Events

`EmployeeAdded`, `EmployeeRemoved`, `EmployeeStatusChanged`, `SalaryUpdated`, `PaymentExecuted`,
`BatchPaymentCompleted`, `ScheduleCreated`, `ScheduleUpdated`, `EmployerVerified`,
`ScheduledPaymentProcessed`, `DeductionUpdated`, `DepartmentCreated`, `BonusIssued`

> Weights: `pallets/payroll/src/weights.rs` (`./scripts/bench_weights.sh pallet_belize_payroll`).

---

## Community Pallet

**Social responsibility scoring, community proposals, ethics review, education modules, green projects, referrals, and peer attestations**

### Extrinsics

```rust
// call_index(0) — record a participation event for an account
pub fn record_participation(origin: OriginFor<T>, account: T::AccountId, activity_code: u8) -> DispatchResult;

// call_index(1) — recompute an account's Social Responsibility Score
pub fn update_srs(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult;

// call_index(2)–(3) — endorsements and privacy
pub fn endorse_peer(origin: OriginFor<T>, endorsee: T::AccountId, endorsement_code: u8) -> DispatchResult;
pub fn set_srs_privacy(origin: OriginFor<T>, public: bool) -> DispatchResult;

// call_index(4) — submit a community proposal
pub fn submit_community_proposal(
    origin: OriginFor<T>,
    proposal_type_code: u8,
    beneficiary: T::AccountId,
    amount: BalanceOf<T>,
    title: BoundedVec<u8, ConstU32<128>>,
    description: BoundedVec<u8, ConstU32<1024>>,
) -> DispatchResult;

// call_index(5)–(7) — proposal voting lifecycle
pub fn vote_community_proposal(origin: OriginFor<T>, proposal_id: u32, approve: bool) -> DispatchResult;
pub fn withdraw_vote(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult;
pub fn finalize_community_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult;

// call_index(8)–(9) — sanctions
pub fn sanction_account(origin: OriginFor<T>, account: T::AccountId, reason: BoundedVec<u8, ConstU32<128>>) -> DispatchResult;
pub fn lift_sanction(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult;

// call_index(10) — ethics council review
pub fn ethics_council_vote(origin: OriginFor<T>, proposal_id: u32, approve: bool) -> DispatchResult;

// call_index(11) — complete an education module
pub fn complete_education_module(origin: OriginFor<T>, module_id: u32, completion_proof: BoundedVec<u8, ConstU32<256>>) -> DispatchResult;

// call_index(12) — contribute to a green project
pub fn contribute_to_green_project(origin: OriginFor<T>, project_id: u32, amount: u64) -> DispatchResult;

// call_index(13) — claim a referral reward
pub fn claim_referral_reward(origin: OriginFor<T>, referee: T::AccountId) -> DispatchResult;

// call_index(14) — attest another account's participation
pub fn attest_participation(origin: OriginFor<T>, subject: T::AccountId, activity_code: u8) -> DispatchResult;
```

**Proposal types** (`proposal_type_code` 0–5): LocalProject, EducationModule,
GreenInitiative, CulturalPreservation, DisasterRelief, CommunityBounty. Voting is
approve/no-abstain via `vote_community_proposal`.

### Storage

| Storage | Purpose |
|---|---|
| `SocialResponsibilityScores` | `AccountId → score breakdown` |
| `ParticipationHistory` | Per-account participation records |
| `PeerEndorsements` · `LastEndorsement` | Endorsement records + cooldown |
| `LastSrsUpdate` | SRS recompute cooldown |
| `UserProposals` · `ProposalCount` | Proposal bookkeeping |
| `CommunityProposals` · `ProposalVotes` | Proposals and votes |
| `EthicsFilterConfig` · `EthicsCouncilVotes` | Ethics review |
| `SanctionedAccounts` | Sanctioned accounts |
| `EducationModules` · `CompletedEducation` · `CompletedEducationCount` | Education |
| `GreenProjects` · `GreenContributions` · `GreenContributionStats` | Green projects |
| `ReferralData` · `RefereeHasReferrer` · `ReferralClaimed` | Referrals |
| `PendingAttestations` · `AttestedActivities` · `AttestationCount` | Attestations |
| `FeeExemptionUsage` | Fee-exemption tracking |

Exact struct shapes: see `pallets/community/src/lib.rs`. There is no `CommunityId` or
`Communities` storage — community proposals are keyed by `u32` proposal id.

### Events

`SRSUpdated`, `ParticipationRecorded`, `PeerEndorsed`, `SRSPrivacyUpdated`, `FeeExemptionReset`,
`ProposalSubmitted`, `ProposalVoted`, `VoteWithdrawn`, `ProposalFinalized`, `ProposalExecuted`,
`ProposalRejected`, `DepositReturned`, `ProposalFlaggedForReview`, `EthicsCouncilVoted`,
`EthicsDecisionFinalized`, `AccountSanctioned`, `SanctionLifted`, `EducationModuleCompleted`,
`EducationRewardClaimed`, `GreenContributionMade`, `GreenMilestoneReached`, `ReferralClaimed`,
`ReferralRewardPaid`, `AttestationSubmitted`, `ActivityAttested`

> Weights: `pallets/community/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_community`).

---

## Contracts Pallet

**Standard Substrate `pallet-contracts` (runtime index 13) — ink! Wasm execution. Not a BelizeChain custom pallet.**

### Extrinsics

The full standard surface is available, including:

```rust
pub fn instantiate_with_code(
    origin: OriginFor<T>,
    value: BalanceOf<T>,
    gas_limit: Weight,
    storage_deposit_limit: Option<BalanceOf<T>>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
) -> DispatchResult;

pub fn call(
    origin: OriginFor<T>,
    dest: AccountIdLookupOf<T>,
    value: BalanceOf<T>,
    gas_limit: Weight,
    storage_deposit_limit: Option<BalanceOf<T>>,
    data: Vec<u8>,
) -> DispatchResult;
```

**Events** (standard `pallet-contracts`): `Instantiated`, `ContractEmitted`, `Called`,
`CodeStored`, `ContractCodeUpdated`. There is no `ContractExecution` event.

**Storage deposit** is priced by `pallet-contracts` from the configured deposit-per-byte
constant, not a fixed "100 DALLA per KB".

```bash
# Deploy via cargo-contract
cargo contract instantiate --suri //Alice --constructor new --args 1000000000000 --execute
```

### Storage

Managed entirely by `pallet-contracts`. See the GEM documentation for the ink! contracts
themselves.

---

## Type Definitions

Types referenced above are defined in the pallets; do not copy them into docs. Key ones:

- **BNS** — domain tier is a `u8` index (`0=Standard, 1=Premium, 2=Government, 3=Verified`);
  hosting tier is `HostingTier` (`Free`, `Basic`, `Pro`, `Enterprise`).
- **LandLedger** — `PropertyId` (`u32`), `EncumbranceType`.
- **Payroll** — `EmployerType`, `WorkerType`, `DeductionType`, `PaymentCategory`.
- **Community** — proposal types are a `u8` code (0–5); there is no `CommunityId` or
  `DistrictId` type — proposals are keyed by `u32` proposal id.

For authoritative shapes, read `pallets/<name>/src/lib.rs` or query runtime metadata.

---

## Related Documentation

- [Core Pallet APIs](./pallet-apis-core.md)
- [Financial Pallet APIs](./pallet-apis-financial.md)
- [Infrastructure Pallet APIs](./pallet-apis-infrastructure.md)
- [BNS Overview](../services/bns-overview.md)
- LandLedger Guide
- [GEM Contracts](../smart-contracts/gem-platform.md)
