# Core Pallet APIs

**Economy • Identity • Governance • Compliance**

Comprehensive API reference for BelizeChain's foundational pallets.

---

## Economy Pallet

**DALLA (native) + bBZD (BZD-pegged stablecoin): Central Bank reserves, redemptions, tourism cashback**

### Extrinsics

```rust
// call_index(0) — Central Bank mints bBZD against off-chain BZD reserves
pub fn mint_bbzd(origin: OriginFor<T>, recipient: T::AccountId, amount: u128, deposit_reference: BoundedVec<u8, ConstU32<64>>) -> DispatchResult;

// call_index(1) — burn bBZD and queue an off-chain settlement
pub fn redeem_bbzd(origin: OriginFor<T>, amount: u128, bank_account: BoundedVec<u8, ConstU32<64>>) -> DispatchResult;

// call_index(2) — Central Bank confirms the off-chain transfer
pub fn process_redemption(origin: OriginFor<T>, redemption_id: u64) -> DispatchResult;

// call_index(3) — governance authorises/revokes a minter
pub fn set_minter_authorization(origin: OriginFor<T>, account: T::AccountId, authorized: bool) -> DispatchResult;

// call_index(4) — governance updates attested Central Bank reserves
pub fn update_reserves(origin: OriginFor<T>, new_reserves: u128) -> DispatchResult;

// call_index(5) — tourism payment with category-based cashback (2–8%)
pub fn process_tourism_payment(origin: OriginFor<T>, vendor: T::AccountId, amount: <T::Currency as Currency<T::AccountId>>::Balance, category_id: u8) -> DispatchResult;

// call_index(6) — user-initiated DALLA burn
pub fn burn_dalla(origin: OriginFor<T>, amount: <T::Currency as Currency<T::AccountId>>::Balance) -> DispatchResult;

// call_index(7) — governance-controlled burn
pub fn governance_burn(origin: OriginFor<T>, amount: <T::Currency as Currency<T::AccountId>>::Balance) -> DispatchResult;

// call_index(8) — peer-to-peer bBZD transfer (KYC + travel rule enforced)
pub fn transfer_bbzd(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult;
```

> **Corrections:** there is no `apply_tourism_cashback` (the real extrinsic is
> `process_tourism_payment`) and no `register_merchant` — merchant registration lives on the
> **Oracle** pallet (`oracle.verify_merchant`, `MerchantCategories`).

### Storage

| Storage | Purpose |
|---|---|
| `TotalSupply` | DALLA supply |
| `TotalBbzdSupply` | Circulating bBZD |
| `CentralBankReserves` | Attested off-chain BZD reserves |
| `MintingHalted` | Halt flag when supply exceeds reserves |
| `AuthorizedMinters` | Central Bank minters |
| `BBZDBalances` | Per-account bBZD balances |
| `RedemptionRequests` · `NextRedemptionId` · `PendingRedemptionIds` | Redemption lifecycle |
| `TourismPayments` · `NextPaymentId` | Recorded tourism payments |
| `LastInflationBlock` | Annual inflation bookkeeping |
| `MintCallsThisBlock` · `LastMintRateLimitBlock` | Mint rate limiting |

Exact value structs: see `pallets/economy/src/lib.rs`.

### Events

`BbzdMinted`, `BbzdRedeemed`, `RedemptionProcessed`, `ReservesUpdated`, `MinterAuthorization`,
`TourismIncentivePaid`, `TravelRuleTriggered`, `BbzdTransferred`, `AnnualInflationApplied`,
`DallaBurned`, `GovernanceBurn`, `BbzdInvariantViolation`

> Weights: `pallets/economy/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_economy`).

---

## Identity Pallet

**BelizeID: identity records, DID documents, SSN/Passport/Biometric attestations issued by bonded issuers**

### Extrinsics

```rust
// call_index(0)–(2) — identity lifecycle
pub fn register_identity(origin: OriginFor<T>, name: BoundedVec<u8, T::MaxNameLen>) -> DispatchResult;
pub fn link_account(origin: OriginFor<T>, new_account: T::AccountId) -> DispatchResult;
pub fn update_did_doc(origin: OriginFor<T>, cid: BoundedVec<u8, T::MaxAnchorLen>) -> DispatchResult;

// call_index(3)–(5) — issuer registry (governance)
pub fn add_issuer(origin: OriginFor<T>, attr: u8, issuer: T::AccountId) -> DispatchResult;
pub fn remove_issuer(origin: OriginFor<T>, attr: u8, issuer: T::AccountId) -> DispatchResult;
pub fn set_standard_version(origin: OriginFor<T>, attr: u8, version: u32) -> DispatchResult;

// call_index(6)–(9) — policy: fees, bonds, rate limits, pause
pub fn set_operation_fee(origin: OriginFor<T>, fee: BalanceOf<T>) -> DispatchResult;
pub fn set_issuer_bond_amount(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult;
pub fn set_rate_limits(origin: OriginFor<T>, window: BlockNumberFor<T>, ssn: u32, passport: u32, biometrics: u32) -> DispatchResult;
pub fn set_pause(origin: OriginFor<T>, paused: bool) -> DispatchResult;

// call_index(10)–(12) — attesting (issuer only; `attr` selects SSN/Passport/Biometrics)
pub fn issue_ssn(origin: OriginFor<T>, target: T::AccountId, hash: H256, anchor: BoundedVec<u8, T::MaxAnchorLen>, format_ok: bool) -> DispatchResult;
pub fn issue_passport(origin: OriginFor<T>, target: T::AccountId, hash: H256, anchor: BoundedVec<u8, T::MaxAnchorLen>, format_ok: bool) -> DispatchResult;
pub fn issue_biometrics(origin: OriginFor<T>, target: T::AccountId, anchor: BoundedVec<u8, T::MaxAnchorLen>) -> DispatchResult;

// call_index(13)–(14) — revoke / suspend an attestation
pub fn revoke(origin: OriginFor<T>, account: T::AccountId, attr: u8) -> DispatchResult;
pub fn suspend(origin: OriginFor<T>, account: T::AccountId, attr: u8) -> DispatchResult;

// call_index(15)–(16) — issuer bond lifecycle
pub fn issuer_deposit_bond(origin: OriginFor<T>, attr: u8) -> DispatchResult;
pub fn issuer_withdraw_bond(origin: OriginFor<T>, attr: u8) -> DispatchResult;

// call_index(17)–(19) — issuer accountability
pub fn flag_issuer(origin: OriginFor<T>, attr: u8, issuer: T::AccountId, flagged: bool) -> DispatchResult;
pub fn slash_issuer_bond(origin: OriginFor<T>, attr: u8, issuer: T::AccountId, amount: BalanceOf<T>) -> DispatchResult;
pub fn report_bad_attestation(origin: OriginFor<T>, attr: u8, issuer: T::AccountId, flag: bool, slash_amount: BalanceOf<T>) -> DispatchResult;
```

`attr` is a `u8` index selecting the attribute type (SSN / Passport / Biometrics).

> **Corrections:** there is no `register_belizeid`, no `update_kyc_level`, and no
> `verify_identity` extrinsic. KYC levels are recorded by the **Oracle** pallet
> (`oracle.verify_identity`); this pallet issues the SSN/Passport/Biometric attestations.

### Storage

| Storage | Purpose |
|---|---|
| `Identities` · `IdentityOf` · `NextIdentityId` | Identity records (`IdentityId → IdentityRecord`) and `AccountId → IdentityId` |
| `DidOf` | DID document pointer per identity |
| `SsnIssuers` · `PassportIssuers` · `BiometricIssuers` | Issuer registries per attribute |
| `SsnAttestations` · `PassportAttestations` · `BiometricAttestations` | Attestations, keyed by `u128` identity id |
| `SsnHashIndex` · `PassportHashIndex` | Document-hash → identity lookups |
| `AttributeHistory` | Attestation history |
| `SsnStandardVersion` · `PassportStandardVersion` | Standard versions (note: `Ssn...`, not `SSN...`) |
| `IssuerBonds` · `IssuerBondAmount` · `IssuerRate` · `FlaggedIssuers` | Issuer bond/accountability |
| `OperationFee` · `GlobalPaused` | Policy |
| `RateWindowBlocks` · `RateMaxPerWindowSsn` · `RateMaxPerWindowPassport` · `RateMaxPerWindowBiometrics` | Rate limiting |

Exact value structs: see `pallets/identity/src/lib.rs`.

### Events

`IdentityRegistered`, `AccountLinked`, `DidDocUpdated`, `IssuerAdded`, `IssuerRemoved`,
`StandardVersionUpdated`, `OperationFeeUpdated`, `Paused`, `Resumed`, `Attested`, `Suspended`,
`Revoked`, `IssuerFlagged`, `IssuerBondDeposited`, `IssuerBondWithdrawn`, `IssuerBondSlashed`,
`RateLimitUpdated`, `IssuerBondAmountUpdated`

> Weights: `pallets/identity/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_identity`).

---

## Governance Pallet

**Proposal lifecycle, conviction voting, referenda, districts, treasury, departments, emergency powers, runtime upgrades, and chain-parameter control**

> This pallet holds the **treasury** — there is no separate Treasury pallet. The national reserve
> is `NationalTreasuryReserve`; spend proposals are `TreasurySpendProposals`.

### Extrinsics (46)

```rust
// ── Proposals & voting ───────────────────────────────────────────────
0  submit_proposal(title: Vec<u8>, description: Vec<u8>, proposal_type_index: u8, threshold_index: u8, is_emergency: bool, district_index: Option<u8>)
1  cast_vote(proposal_id: u32, vote_choice_index: u8, conviction: u8)
2  finalize_proposal(proposal_id: u32)
13 execute_proposal(proposal_id: u32)
20 amend_proposal(proposal_id: u32, new_title: Option<BoundedVec<u8, 256>>, new_description: Option<BoundedVec<u8, 1024>>)
22 set_proposal_priority(proposal_id: u32, priority: u8)
37 commit_vote(proposal_id: u32, commitment: [u8; 32])          // confidential voting
38 reveal_vote(proposal_id: u32, vote_choice_index: u8, salt: [u8; 32], conviction: u8)

// ── Rank / participation inputs ──────────────────────────────────────
3  update_community_rank(account, new_rank: u32)
4  update_pouw_contribution(account, new_contribution: u32)
21 claim_participation_reward(reward_type: u8)

// ── Departments ──────────────────────────────────────────────────────
6  set_department_manager(department_index: u8, manager: T::AccountId)
7  submit_department_proposal(department_index: u8, title, description, proposal_type_index: u8, threshold_index: u8, requires_cross_approval: bool)
8  approve_cross_department(proposal_id: u32, department_index: u8)
5  council_override(proposal_id: u32, override_reason: Vec<u8>)

// ── Board ────────────────────────────────────────────────────────────
9  add_board_member(account, role_index: u8, term_years: u8)
10 remove_board_member(account, reason: Vec<u8>)

// ── Delegation ───────────────────────────────────────────────────────
11 nominate_for_delegate(nominee: T::AccountId)
12 vote_for_delegate(nominee: T::AccountId)
18 delegate_vote(delegate: T::AccountId, expires_at: Option<BlockNumberFor<T>>)
19 revoke_delegation()

// ── Referenda ────────────────────────────────────────────────────────
25 create_referendum(title, description, options: Vec<Vec<u8>>, quorum_percentage: u8, duration_blocks: u32, district_index: Option<u8>)
26 vote_on_referendum(referendum_id: u32, option_index: u8)
27 finalize_referendum(referendum_id: u32)
34 fast_track_referendum(referendum_id: u32)

// ── District elections & budgets ─────────────────────────────────────
14 start_district_election(district_index: u8, seats: u32, registration_period_blocks: u32, voting_period_blocks: u32)
15 register_candidate(district_index: u8, platform: Vec<u8>)
16 vote_in_district_election(district_index: u8, candidate: T::AccountId)
17 finalize_district_election(district_index: u8)
28 allocate_district_budget(district_index: u8, amount: BalanceOf<T>, fiscal_year_blocks: u32)
32 transfer_district_budget(from_district_index: u8, to_district_index: u8, amount: BalanceOf<T>, reason: Vec<u8>)

// ── Treasury ─────────────────────────────────────────────────────────
29 propose_treasury_spend(recipient, amount: BalanceOf<T>, description: Vec<u8>, district_index: Option<u8>)
30 approve_treasury_spend(proposal_id: u32)
31 execute_treasury_proposal(proposal_id: u32)

// ── Emergency powers ─────────────────────────────────────────────────
23 declare_emergency(emergency_type_index: u8, description: Vec<u8>, duration_hours: u32)
24 end_emergency()
44 veto_emergency()
33 execute_emergency_proposal(proposal_id: u32)
35 emergency_override_proposal(proposal_id: u32, execute: bool, justification: Vec<u8>)

// ── Chain parameters & runtime upgrades ──────────────────────────────
36 update_chain_parameter(key: Vec<u8>, value: u64)
42 lock_chain_parameter(key: Vec<u8>)
43 unlock_chain_parameter(key: Vec<u8>)
39 apply_pending_runtime_upgrade(code: Vec<u8>)
45 ratify_runtime_upgrade(code_hash: [u8; 32])
41 ratify_constitutional_proposal(proposal_id: u32)
40 generate_exit_proof()
```

> **Corrections:** there is no `propose`, no `vote`, and no `execute_proposal(ProposalIndex)`
> on a `treasury` pallet — the real names are `submit_proposal`, `cast_vote`, and
> `execute_treasury_proposal`. `Proposals` storage is real but keyed by `u32`.

### Storage

| Storage | Purpose |
|---|---|
| `Proposals` · `NextProposalId` · `ExecutedProposals` · `ProposalQueue` | Proposal lifecycle |
| `Votes` · `VoteCommitments` · `VoteDelegations` · `AccountVoteParticipation` | Voting & delegation |
| `Referendums` · `ReferendumVotes` · `NextReferendumId` · `ReferendumEligibleVoters` | Referenda |
| `CouncilMembers` · `CouncilTerm` · `BoardComposition` · `TermExpiryQueue` | Council / board |
| `DistrictElections` · `ElectionCandidates` · `ElectionVotes` · `CurrentElection` | District elections |
| `DistrictRepresentation` · `DistrictBudgets` | Districts |
| `NationalTreasuryReserve` · `TreasurySpendProposals` · `TreasurySpendTracker` · `NextTreasuryProposalId` | Treasury |
| `DepartmentManagers` · `DepartmentPolicies` · `DepartmentTreasuryBalances` · `CrossDepartmentApprovals` | Departments |
| `EmergencyVetoes` · `JaguarMode` | Emergency state |
| `ChainParameters` · `LockedParameters` · `PendingRuntimeUpgrade` · `RuntimeUpgradeRatifications` · `ConstitutionalRatifications` | Parameters & upgrades |
| `CommunityRanks` · `PoUWContributions` · `RewardsClaimed` · `TotalRewardsDistributed` | Participation |
| `GovernanceParameters` · `EffectiveVotingMultiplier` · `AccountLastProposal` · `LastParticipationCheckBlock` | Policy / bookkeeping |

Exact value structs: see `pallets/governance/src/lib.rs`.

### Events

`ProposalSubmitted`, `VoteCast`, `ProposalApproved`, `ProposalRejected`, `CommunityRankUpdated`,
`PoUWContributionUpdated`, `CouncilOverrideExecuted`, `DepartmentManagerSet`,
`DepartmentProposalSubmitted`, `CrossDepartmentApproved`, `BoardMemberAdded`,
`BoardMemberRemoved`, `DelegateNominated`, `DelegateVoteCast`, `ProposalExecuted`,
`RuntimeUpgradeExecuted`, `VoteCommitted`, `VoteRevealed`, `RuntimeUpgradeApplied`,
`RuntimeUpgradeRatified`, `ParameterChanged`, `DepartmentActionExecuted`,
`EmergencyActionExecuted`, `DepartmentFundsAllocated`, `DistrictElectionStarted`,
`CandidateRegistered`, `ElectionVoteCast`, `DistrictElectionFinalized`,
`CouncilMemberElectedFromDistrict`, `VoteDelegationCreated`, `VoteDelegationRevoked`,
`ProposalAmendmentSubmitted`, `RewardClaimed`, `ProposalQueuedWithPriority`,
`JaguarModeDeactivated`, `EmergencyPendingActivation`, `EmergencyVetoed`,
`EmergencyFinalizedAfterVetoWindow`, `ReferendumCreated`, `ReferendumVoteCast`,
`ReferendumPassed`, `ReferendumFailed`, `DistrictBudgetAllocated`, `TreasurySpendProposed`,
`TreasurySpendApproved`, `TreasurySpendExecuted`, `DistrictBudgetTransferred`,
`DistrictBudgetSpent`, `EmergencyProposalExecuted`, `ReferendumFastTracked`,
`ProposalEmergencyOverride`, `ChainParameterUpdated`, `CouncilTermAutoExpired`,
`TermExpiryOverflow`, `ProposalEnqueued`, `LargeHolderParticipationPenalty`,
`QuarterlyParticipationReset`, `ExitProofGenerated`, `ConstitutionalRatificationCast`,
`ConstitutionalRatificationComplete`, `ParameterLocked`, `ParameterUnlocked`

> Weights: `pallets/governance/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_governance`).

---

## Compliance Pallet

**Verification levels, risk ratings, whitelist/restriction, SARs, sanctions, and structuring detection**

### Extrinsics

```rust
// call_index(0) — set an account's verification level and risk rating
pub fn verify_account(origin: OriginFor<T>, account: T::AccountId, level: u8, risk_level: u8) -> DispatchResult;

// call_index(1) — update risk rating only
pub fn update_risk_level(origin: OriginFor<T>, account: T::AccountId, risk_level: u8) -> DispatchResult;

// call_index(2)–(4) — account status
pub fn whitelist_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult;
pub fn restrict_account(origin: OriginFor<T>, account: T::AccountId, reason: Vec<u8>) -> DispatchResult;
pub fn lift_restriction(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult;

// call_index(5) — file a suspicious-activity report
pub fn flag_suspicious_activity(origin: OriginFor<T>, account: T::AccountId, activity_type: u8, details: Vec<u8>) -> DispatchResult;

// call_index(6)–(7) — sanctions list (keyed by entity HASH, not account)
pub fn add_sanctions_entry(origin: OriginFor<T>, entity_hash: [u8; 32], list_source: Vec<u8>) -> DispatchResult;
pub fn remove_sanctions_entry(origin: OriginFor<T>, entity_hash: [u8; 32]) -> DispatchResult;

// call_index(8) — pull verification state from Identity/Oracle
pub fn sync_verification_from_identity(origin: OriginFor<T>) -> DispatchResult;
```

> **Corrections:** there is no `submit_kyc_application`, no `approve_kyc`, and no
> `FscOfficers` registry. KYC applications are handled on the **Oracle** pallet; this pallet
> records the resulting *status* and enforces AML controls. Sanctions are keyed by
> `entity_hash`, not by `AccountId`.

### Storage

| Storage | Purpose |
|---|---|
| `ComplianceStatusOf` | `AccountId → { verification_level, risk_level, whitelisted, restricted, last_verification }` |
| `WhitelistedAccounts` · `RestrictedAccounts` | Account status sets |
| `SanctionsList` | Sanctions entries, keyed by entity hash |
| `SuspiciousActivities` | Filed SARs |
| `AuditRecords` | Per-account compliance audit trail |
| `TransactionWindow` | Sliding window used for structuring detection |
| `ComplianceStats` | Aggregate counters |

Exact value structs: see `pallets/compliance/src/lib.rs`.

### Events

`VerificationLevelUpdated`, `RiskLevelUpdated`, `AccountWhitelisted`, `AccountRestricted`,
`RestrictionLifted`, `SuspiciousActivityReported`, `SanctionsEntryAdded`, `SanctionsEntryRemoved`,
`ComplianceCheckPerformed`, `AuditRecordCreated`, `StructuringDetected`

> Weights: `pallets/compliance/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_compliance`).

---

## Type Definitions

Enums are defined in the pallets. Do not copy them by hand — read the source or query runtime
metadata (`@polkadot/api`) for the authoritative shape:

- **`TourismCategory`** — `pallets/economy`: `Accommodation`, `Dining`, `Tours`,
  `Transportation`, `Shopping`, `Cultural`. Tourism cashback rates are 2–8% by category,
  returned by `Economy::get_tourism_incentive_rate`.
- **`AttributeType`** — `pallets/identity`: selects SSN / Passport / Biometrics; extrinsics take
  it as a `u8` index.
- **`VerificationLevel`**, **`RiskLevel`**, **`ComplianceStatus`** — `pallets/compliance`.
- **`BelizeDistrict`** — `pallets/governance`: `Belize`, `Cayo`, `Corozal`, `OrangeWalk`,
  `StannCreek`, `Toledo` (exactly **six** districts; the pallet enforces `idx < 6`).

> **Corrections:** `MerchantCategory`, `IdentityType`, `KycLevel`, `Vote{Aye,Nay,Abstain}` and
> `SuspicionReason` as previously documented here do not match the runtime. Merchant categories
> live on the **Oracle** pallet; governance vote choice is a `u8` index with a conviction
> multiplier (`cast_vote(.., vote_choice_index: u8, conviction: u8)`).

---

## Error Handling

Each pallet defines its own `#[pallet::error] enum Error<T>`. Query the real variants from
source or from runtime metadata rather than relying on a hand-copied list:

```bash
# Real error variants for a pallet
grep -A40 'pub enum Error<T>' pallets/<name>/src/lib.rs
```

A few that callers commonly need:

| Pallet | Error | Meaning |
|---|---|---|
| economy | `InsufficientBbzdBalance` | bBZD balance too low for redemption/transfer |
| economy | `PaymentIdOverflow` | Tourism payment id counter would overflow `u32` |
| economy | `TravelRuleEnhancedKycRequired` | Amount ≥ travel-rule threshold needs KYC L2 |
| economy | `MintingHaltedUndercollateralized` | bBZD supply exceeds attested reserves |
| economy | `RateLimitExceeded` | Per-block `mint_bbzd` cap hit |
| compliance | `SanctionedEntity` | Account is sanctioned |
| consensus | `InvalidComputationCommitment` | Work commitment does not match the submission |
| consensus | `RateLimitExceeded` | Per-block `submit_ai_work` cap hit |
| governance | `RoundNotInProgress` | (consensus) no active round |

---

## Related Documentation

- [Multi-Repo Overview](../architecture/multi-repo-overview.md)
- [Economics Documentation](../economics/tokenomics.md)
- [Governance System](../governance/democracy.md)
- [Compliance Guide](../security/COMPLIANCE_REGULATORY.md)
