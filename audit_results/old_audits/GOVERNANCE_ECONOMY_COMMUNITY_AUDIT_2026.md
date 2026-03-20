# BelizeChain Security Audit — Governance · Economy · Community Pallets

**Date**: 2026-01-XX  
**Auditor**: AI Security Analysis (Claude Opus 4.6)  
**Scope**: `pallets/governance/src/lib.rs`, `pallets/economy/src/lib.rs`, `pallets/community/src/lib.rs`  
**Total Lines**: 10,862 (governance 7,360 + economy 1,133 + community 2,369)  
**Methodology**: Full source read, line-by-line analysis of access control, arithmetic, storage patterns, economic invariants, governance lifecycle

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Economy Pallet](#2-economy-pallet)
3. [Community Pallet](#3-community-pallet)
4. [Governance Pallet](#4-governance-pallet)
5. [Cross-Pallet Integration Analysis](#5-cross-pallet-integration-analysis)
6. [Findings Summary Table](#6-findings-summary-table)
7. [Recommendations](#7-recommendations)

---

## 1. Executive Summary

All three pallets demonstrate a **mature security posture** with evidence of multiple prior audit fix rounds (H-20/H-21, H-26, AR-8/13/14/15, CM-1/CM-2, M48/M59/M60/M61, S5-4, CONS-029/030, DOS-004/009/010/015/018). Saturating arithmetic is used throughout; no `unwrap()` calls appear in production paths. Bounded collections (`BoundedVec`) are used for all user-supplied data.

### Severity Distribution

| Severity | Count | Description |
|----------|-------|-------------|
| **CRITICAL** | 0 | No exploitable vulnerabilities found |
| **HIGH** | 3 | Economic logic risks, edge-case bypasses |
| **MEDIUM** | 7 | Missing validations, incomplete features, weight inaccuracies |
| **LOW** | 9 | Cosmetic, TODO items, defense-in-depth improvements |
| **INFORMATIONAL** | 6 | Observations, patterns worth noting |

---

## 2. Economy Pallet

**File**: `pallets/economy/src/lib.rs`  
**Lines**: 1,133  
**Purpose**: National economic system — DALLA token (native, 6 decimals, 501B max supply, 2% inflation), bBZD stablecoin (fiat-backed, 1:1 BZD peg), treasury routing, tourism incentives.

### 2.1 Key Types & Storage

| Storage Item | Type | Purpose |
|---|---|---|
| `TotalSupply` | `ValueQuery` | Tracks total DALLA in circulation |
| `TotalBbzdSupply` | `ValueQuery` | Total bBZD minted |
| `CentralBankReserves` | `ValueQuery` | BZD backing for bBZD |
| `AuthorizedMinters` | `StorageMap<AccountId, bool>` | Permissioned bBZD minters |
| `BBZDBalances` | `StorageMap<AccountId, Balance>` | bBZD ledger |
| `RedemptionRequests` | `StorageMap<u32, RedemptionRequest>` | Pending bBZD→BZD redemptions |
| `MintCallsThisBlock` | `ValueQuery` | AR-15 rate limiter |
| `LastInflationBlock` | `ValueQuery` | Prevents double-inflation |
| `CumulativePublicGoodsInflation` / `CumulativeWellbeingInflation` | `ValueQuery` | Treasury routing trackers |

### 2.2 Extrinsic Signatures & Access Control

| # | Extrinsic | Origin | Key Checks |
|---|-----------|--------|------------|
| 0 | `mint_bbzd(to, amount)` | `ensure_signed` + `AuthorizedMinters` | AR-15 rate limit (max 10/block), supply cap, reserves ≥ supply |
| 1 | `redeem_bbzd(amount)` | `ensure_signed` | KYC compliance check, sufficient bBZD balance |
| 2 | `process_redemption(request_id, approved)` | `ensure_signed` + `AuthorizedMinters` | Redemption exists, not processed, reserves check |
| 3 | `set_minter_authorization(account, authorized)` | `GovernanceOrigin` | Authorization toggle |
| 4 | `update_reserves(amount, increase)` | `GovernanceOrigin` | Saturating add/sub |
| 5 | `process_tourism_payment(amount, hotel_id, booking_ref)` | `ensure_signed` | Oracle verification, tourism rate via `Permill` |
| 6 | `burn_dalla(amount)` | `ensure_signed` | Balance check, H-20 TotalSupply sync |
| 7 | `governance_burn(amount)` | `GovernanceOrigin` | Treasury burn, H-21 TotalSupply sync |

### 2.3 `on_initialize` — Inflation Logic

```
Annual inflation: 2% of TotalSupply
Split: 60% PublicGoods + 40% Wellbeing (configurable via Permill)
Guard: LastInflationBlock prevents multi-inflation per block
Cap: TotalSupply + new_tokens <= MaxSupply (501B DALLA)
```

### 2.4 Security Findings — Economy

#### [E-1] HIGH — `integrity_test` assert is development-only

**Location**: `integrity_test()` block  
**Issue**: The invariant `TotalBbzdSupply <= CentralBankReserves` is checked via `integrity_test`, which only runs during runtime construction (not at every block). A logic bug in `process_redemption` that incorrectly handles the `approved=false` path could break this invariant at runtime without detection.  
**Status**: The redemption logic appears correct (only reduces reserves when `approved=true`), but there is no runtime assertion to catch regressions.  
**Recommendation**: Add a defensive check in `on_initialize` or `on_finalize` that logs a warning (not panic) if `TotalBbzdSupply > CentralBankReserves`.

#### [E-2] MEDIUM — Tourism oracle trust model undocumented

**Location**: `process_tourism_payment`, line ~900  
**Issue**: Oracle verification is delegated to `T::TourismOracle::verify_booking()` but the trust model (who operates the oracle, how it's updated, what happens if compromised) is not documented in code or in security notes.  
**Recommendation**: Add a doc comment explaining the oracle trust assumptions and failure modes.

#### [E-3] LOW — `MintCallsThisBlock` reset timing

**Location**: `on_initialize`  
**Issue**: `MintCallsThisBlock` is reset at block start. If `on_initialize` runs but `on_finalize` doesn't complete (e.g., due to block import failure), the counter could be stale. This is a standard Substrate pattern and unlikely to cause issues, but worth noting.

#### [E-4] INFORMATIONAL — Saturating arithmetic is consistent

All balance operations use `saturating_add`/`saturating_sub`. No checked arithmetic with error propagation — consistent with Substrate conventions for balance tracking that can't meaningfully fail.

#### [E-5] INFORMATIONAL — No `unwrap()` in production paths

Confirmed: zero `unwrap()` calls in production code. Only test modules use `unwrap()`.

### 2.5 Events Emitted

`BBZDMinted`, `BBZDRedeemed`, `RedemptionProcessed`, `MinterAuthorizationChanged`, `ReservesUpdated`, `TourismPaymentProcessed`, `DALLABurned`, `GovernanceBurn`, `InflationDistributed`

---

## 3. Community Pallet

**File**: `pallets/community/src/lib.rs`  
**Lines**: 2,369  
**Purpose**: Social Responsibility Score (SRS) system, zero-fee protocol, community fund, education/referral rewards, ethics filter, Proof of Useful Work (PoUW) integration.

### 3.1 Key Types & Storage

**SRS System** (0–10,000 scale, 6 components):
- `governance` (0–2500), `education` (0–2000), `sustainability` (0–1500)
- `participation` (0–2500), `endorsements` (0–1000), `honesty` (0–1000)

**Tiers**: Bronze (<2500), Silver (2500–4999), Gold (5000–7499), Platinum (7500–9999), Diamond (10000)

| Storage Item | Type | Purpose |
|---|---|---|
| `SocialResponsibilityScores` | `StorageMap<AccountId, SRS>` | Per-account SRS |
| `ParticipationHistory` | `StorageMap<AccountId, Vec<Activity>>` | Bounded activity log |
| `PeerEndorsements` | `StorageMap<AccountId, u32>` | Endorsement count |
| `LastEndorsement` | `StorageDoubleMap<Endorser, Endorsee, Block>` | Monthly cooldown |
| `CommunityProposals` | `StorageMap<u32, Proposal>` | Community-level proposals |
| `FeeExemptionUsage` | `StorageMap<AccountId, (u32, Block)>` | Monthly fee exemption tracker |
| `SanctionedAccounts` | `StorageMap<AccountId, bool>` | Ethics sanctions |
| `EducationModules` / `CompletedEducation` | Maps | Education tracking |
| `GreenProjects` / `GreenContributions` | Maps | Sustainability tracking |
| `ReferralData` / `ReferralClaimed` | Maps | Referral reward system |
| `PendingAttestations` / `AttestedActivities` | Maps | Oracle attestation pipeline |

### 3.2 Extrinsic Signatures & Access Control

| # | Extrinsic | Origin | Key Checks |
|---|-----------|--------|------------|
| 0 | `record_participation(activity)` | Root or self-signed | Phase 3B: attestation required for high-value activities |
| 1 | `update_srs(account)` | Permissionless | Recalculates from components |
| 2 | `endorse_peer(endorsee)` | `ensure_signed` | Silver+ SRS, monthly cooldown, no self-endorse |
| 3 | `set_srs_privacy(private)` | `ensure_signed` | Privacy toggle |
| 4 | `submit_community_proposal(title, desc, amount)` | `ensure_signed` | KYC, 10% deposit reserved |
| 5 | `vote_community_proposal(id, approve, weight)` | `ensure_signed` | SRS-weighted, one vote per account |
| 6 | `finalize_community_proposal(id)` | Permissionless | After deadline only |
| 7 | `sanction_account(account)` | `GovernanceOrigin` | Ethics enforcement |
| 8 | `lift_sanction(account)` | `GovernanceOrigin` | Sanction removal |
| 9 | `ethics_council_vote(account, approve)` | `ensure_signed` + council check | Ethics review voting |
| 10 | `complete_education_module(id, proof)` | `ensure_signed` | CM-1: supply cap check, M59: non-empty proof |
| 11 | `contribute_to_green_project(id, amount)` | `ensure_signed` | M60: funds reserved, project exists |
| 12 | `claim_referral_reward(referee)` | `ensure_signed` | CM-2: supply cap check, not already claimed |
| 13 | `attest_participation(account, activity)` | `OracleAttestationOrigin` | Oracle-only attestation |

### 3.3 Exported Trait Implementations

| Trait | Implementation Notes |
|---|---|
| `CommunityRank` | **M61 FIX**: Diminishing returns — Bronze=100, Silver=175, Gold=250, Platinum=325, Diamond=400. Max 4x multiplier (capped at 400). |
| `FeeCalculator` | Tier-based discounts: Bronze 0%, Silver 25%, Gold 50%, Platinum 75%, Diamond 90%. Monthly exemption limit with monthly reset. |
| `PoUWContributor` | Quality 40% + Timeliness 30% + Honesty 30%. Score out of 100. |
| `GovernanceParticipation` | Records vote cast, checks if participated (via `ParticipationHistory`). |

### 3.4 Security Findings — Community

#### [C-1] HIGH — `update_srs` is permissionless and recalculates from mutable components

**Location**: `update_srs` extrinsic + `calculate_srs` helper  
**Issue**: Any account can call `update_srs(target)` for any other account. The SRS is recalculated from component scores that can be individually boosted by other extrinsics (`complete_education_module`, `contribute_to_green_project`, `record_participation`). While each component has caps (governance 0–2500, education 0–2000, etc.), the permissionless nature means SRS can be updated to reflect any legitimate state change at any time.  
**Risk**: Not a direct vulnerability since components are capped and modifications require valid actions, but it means an attacker who finds a way to inflate any component (e.g., fake education completions) immediately gets the SRS benefit without delay.  
**Mitigation**: The education module requires non-empty proof (M59 fix) and supply cap (CM-1). Green contributions require reserved funds (M60). Endorsements require Silver+ SRS and monthly cooldown.  
**Severity**: HIGH (design consideration) — if any single component input is compromisable, the entire SRS cascades.

#### [C-2] MEDIUM — Endorsement sybil resistance relies solely on SRS threshold

**Location**: `endorse_peer` extrinsic  
**Issue**: Endorsements require Silver+ SRS (≥2500). An account that achieves Silver through education completions alone (education component = 2000 contributes significantly) could then endorse confederates. The monthly cooldown (one endorsement per pair per month) provides rate limiting but not sybil protection.  
**Recommendation**: Consider requiring a minimum account age or staking requirement for endorsement eligibility.

#### [C-3] MEDIUM — `finalize_community_proposal` has no quorum check

**Location**: `finalize_community_proposal` extrinsic  
**Issue**: A community proposal can be finalized (approved or rejected) based solely on whether `ayes > nays`, with no minimum quorum. A proposal with 1 aye and 0 nay would pass.  
**Recommendation**: Add a minimum vote count or minimum total weight threshold.

#### [C-4] LOW — Fee exemption monthly reset uses block number modular arithmetic

**Location**: `check_fee_exemption_limit` helper  
**Issue**: Monthly reset is calculated as `current_block / BLOCKS_PER_MONTH`. This works correctly but means the "month" boundary is fixed to genesis rather than calendar months. Accounts get a reset at every multiple of `BLOCKS_PER_MONTH`, which could be mid-day.  
**Status**: Acceptable design — standard Substrate pattern.

#### [C-5] LOW — `record_participation` dual-origin pattern

**Location**: `record_participation` extrinsic  
**Issue**: Accepts either Root or self-signed origin. The self-signed path allows users to record their own participation without oracle attestation for low-value activities, relying on Phase 3B attestation requirements for high-value ones. The boundary between "low-value" and "high-value" should be clearly documented.

#### [C-6] INFORMATIONAL — CM-1/CM-2 supply cap enforcement is sound

Both `complete_education_module` and `claim_referral_reward` use `checked_add` against `TotalSupply` before calling `deposit_creating`. If `TotalSupply + reward > MaxSupply`, the extrinsic fails. This is the correct pattern.

### 3.5 Events Emitted

`ParticipationRecorded`, `SRSUpdated`, `PeerEndorsed`, `SRSPrivacyUpdated`, `CommunityProposalSubmitted`, `CommunityProposalVoted`, `CommunityProposalFinalized`, `AccountSanctioned`, `SanctionLifted`, `EthicsCouncilVoteCast`, `EducationModuleCompleted`, `GreenContributionMade`, `ReferralRewardClaimed`, `ParticipationAttested`, `EthicsReviewFinalized`

---

## 4. Governance Pallet

**File**: `pallets/governance/src/lib.rs`  
**Lines**: 7,360  
**Purpose**: Multi-tiered democratic governance — district elections, Foundation Board, proposals (9 types), quadratic voting, referendums, liquid democracy, emergency powers (Jaguar Mode), treasury management, constitutional safeguards.

### 4.1 Key Types & Enums

| Type | Variants | Purpose |
|---|---|---|
| `ProposalType` | Constitutional, Economic, Council, Technical, Emergency, International, Community, DistrictLocal, WellbeingFunding | 9 proposal categories with different thresholds |
| `Department` | Finance, Education, Health, Works, Justice, Tourism, Agriculture, Defense | 8 government departments |
| `BoardRole` | Chair, ViceChair, Secretary, Treasurer, TechnicalLead, CommunityLead, CitizenDelegate | 7 foundation board roles |
| `BelizeDistrict` | Belize, Cayo, Corozal, OrangeWalk, StannCreek, Toledo | 6 geographic districts |
| `ProposalAction` | TreasurySpend, RuntimeUpgrade, ParameterChange, DepartmentAction, EmergencyAction | executable actions |
| `VotingThreshold` | SimpleMajority, Supermajority, Unanimous, Custom(u8) | threshold types |
| `GovernanceParameter` | VotingPeriod, LaunchPeriod, MinimumDeposit, SupermajorityThreshold, CouncilSize, EmergencyTimeout | on-chain tunable parameters |
| `EmergencyType` | SecurityBreach, NaturalDisaster, SystemFailure, ConsensusEmergency, Other | emergency classifications |

### 4.2 Storage (~50+ items)

**Core Governance**:
`CouncilMembers`, `Proposals`, `Votes`, `NextProposalId`, `CouncilTerm`, `GovernanceParameters`, `ExecutedProposals`, `ProposalEnactmentBlock`, `AccountLastProposal`

**Voting System**:
`VoteCommitments` (AR-13 pre-provisioned), `CommunityRanks`, `PoUWContributions`, `AccountVoteParticipation`, `EffectiveVotingMultiplier`, `LastParticipationCheckBlock`

**Emergency**:
`JaguarMode`, `EmergencyVetoes`

**Board & Departments**:
`BoardComposition`, `TermExpiryQueue`, `DepartmentManagers`, `CrossDepartmentApprovals`, `DepartmentPolicies` (AR-8), `DepartmentTreasuryBalances`

**Elections & Delegation**:
`DistrictElections`, `ElectionCandidates`, `ElectionVotes`, `DistrictRepresentation`, `VoteDelegations`, `DelegationReceivers`, `DelegateNominees`, `DelegateVoters`, `CurrentElection`

**Proposals & Amendments**:
`ProposalAmendments`, `ProposalQueue`, `RewardsClaimed` (H-26 per-type), `TotalRewardsDistributed`

**Referendums**:
`Referendums`, `NextReferendumId`, `ReferendumVotes`, `ReferendumEligibleVoters`

**Treasury**:
`DistrictBudgets`, `TreasurySpendProposals`, `NextTreasuryProposalId`, `NationalTreasuryReserve`, `TreasurySpendTracker` (S5-4)

**Constitutional**:
`ConstitutionalRatifications`, `LockedParameters`, `ChainParameters`, `PendingRuntimeUpgrade`

### 4.3 Config Trait — Security-Critical Parameters

| Parameter | Purpose |
|---|---|
| `QuadraticVotingEnabled` | Toggle for sqrt-based voting weight |
| `MaxVotingUnits` | Hard cap on any single vote weight |
| `StakeUnitSize` | Balance → stake unit conversion denominator |
| `MaxConsecutiveTerms` | Board member term limit |
| `ProposalCooldown` | Blocks between proposals per account |
| `EnactmentPeriod{Constitutional,Economic,Standard}` | Type-specific execution delays |
| `LargeHolderStakeThreshold` | Wealth-responsibility trigger threshold |
| `LargeHolderMinParticipationRate` | Required participation rate for large holders |
| `ExitProofValidity` | Block window for exit proofs (~30 days) |
| `BehaviorFlags` (BehaviorFlagProvider) | Circuit breaker for misbehaving accounts |
| `DualHouseProvider` | Constitutional dual-house membership check |
| `ConstitutionalAdminOrigin` | Origin for parameter locks (TODO: dual-council compound) |
| `EmergencyVetoWindow` | Blocks before emergency activates (CONS-029) |
| `RuntimeUpgradeMinTimelock` / `ParameterChangeMinTimelock` | CONS-030 action timelocks |
| `MaxTreasurySpendPerPeriod` / `TreasurySpendPeriod` | S5-4 spend caps |
| `MaxCandidatesPerElection` | DoS bound for elections |

### 4.4 Extrinsic Signatures & Access Control

| Index | Extrinsic | Origin | Key Checks |
|---|-----------|--------|------------|
| 0 | `submit_proposal(title, desc, type, deposit, action, threshold, emergency)` | `ensure_signed` | Compliance, BehaviorFlags, ProposalCooldown, deposit reserve, BoundedVec |
| 1 | `cast_vote(proposal_id, vote, conviction)` | `ensure_signed` | Voting period, no duplicate, **QV**: balance→stake_units→isqrt→MaxVotingUnits, conviction multiplier, EffectiveVotingMultiplier |
| 2 | `finalize_proposal(proposal_id)` | `ensure_signed` | **Operational** (DOS-009), after voting end, threshold check, enactment delay (CONS-030 timelocks) |
| 3 | `update_community_rank(account, rank)` | `ensure_root` | Root only |
| 4 | `update_pouw_contribution(account, contribution)` | `ensure_root` | Root only |
| 5 | `council_override(proposal_id, decision)` | `CouncilOrigin` | Emergency-only, **Operational** dispatch |
| 6 | `set_department_manager(dept, account)` | `ensure_root` | Root only |
| 7 | `submit_department_proposal(dept, title, desc, call_data)` | `ensure_signed` | Department manager only |
| 8 | `approve_cross_department(dept, proposal_id)` | `ensure_signed` | Department manager only |
| 9 | `add_board_member(account, role, weight)` | `CouncilOrigin` | MaxConsecutiveTerms enforcement |
| 10 | `remove_board_member(account)` | `CouncilOrigin` | Existence check |
| 11 | `nominate_for_delegate(election_block)` | `ensure_signed` | Active election, registration period |
| 12 | `vote_for_delegate(election_block, candidate)` | `ensure_signed` | Active election, voting period, no duplicate |
| 13 | `execute_proposal(proposal_id)` | `ensure_signed` | Council member only, double-exec prevention, enactment delay, Constitutional dual-house check, S5-4 treasury cap |
| 14 | `start_district_election(district, duration)` | `ensure_root` | No active election |
| 15 | `register_candidate(district, platform)` | `ensure_signed` | Compliance, active registration |
| 16 | `vote_in_district_election(district, candidate)` | `ensure_signed` | Active voting, no duplicate, compliance |
| 17 | `finalize_district_election(district)` | `ensure_root` | **Parameterized weight** (DOS-015), bounded candidate iteration |
| 18 | `delegate_vote(delegate, expires_at)` | `ensure_signed` | M48: compliance for both parties, no self-delegate, max 100 delegators |
| 19 | `revoke_delegation()` | `ensure_signed` | Delegation exists |
| 20 | `amend_proposal(id, new_title, new_desc)` | `ensure_signed` | Original proposer only, before voting starts |
| 21 | `claim_participation_reward(reward_type)` | `ensure_signed` | H-26: per-type tracking, bounded scan (50), treasury balance check |
| 22 | `set_proposal_priority(id, priority)` | `ensure_root` | Priority 1–4, queue max 50 |
| 23 | `declare_emergency(type, description)` | `ensure_root` | CONS-029: veto window before activation |
| 24 | `end_emergency()` | `ensure_root` | Emergency must be active |
| 25 | `create_referendum(title, desc, options, duration, quorum, threshold)` | `ensure_signed` | Council member only |
| 26 | `vote_on_referendum(id, option_idx, weight)` | `ensure_signed` | Active, no duplicate, eligible voter or open |
| 27 | `finalize_referendum(id)` | `ensure_signed` | After voting end, quorum check |
| 28 | `allocate_district_budget(district, amount, fiscal_year)` | `ensure_root` | No active budget |
| 29 | `propose_treasury_spend(recipient, amount, desc, district)` | `ensure_signed` | Compliance, tiered approval threshold (1/3/4-of-7) |
| 30 | `approve_treasury_spend(proposal_id)` | `ensure_signed` | Council member, not expired, not duplicate approval |
| 31 | `execute_treasury_proposal(proposal_id)` | `ensure_signed` | Threshold met, not expired, district/national budget check |
| 32 | `transfer_district_budget(from, to, amount, reason)` | `ensure_root` | Sufficient available funds |
| 33 | `execute_emergency_proposal(proposal_id)` | Root or `CouncilOrigin` | JaguarMode active, emergency flag, 66% supermajority |
| 34 | `fast_track_referendum(referendum_id)` | Root or `CouncilOrigin` | JaguarMode active, referendum active |
| 35 | `emergency_override_proposal(id, execute, justification)` | `ensure_root` | JaguarMode active |
| 36 | `update_chain_parameter(key, value)` | `ensure_root` | Phase 6B: rejects if constitutionally locked |
| 37 | `commit_vote(proposal_id, commitment)` | `ensure_signed` | AR-13: voting period, no prior commit, no prior reveal |
| 38 | `reveal_vote(proposal_id, choice, salt, conviction)` | `ensure_signed` | AR-13: blake2_256 hash verification, double-reveal guard |
| 39 | `apply_pending_runtime_upgrade(code)` | `ensure_root` | AR-14: blake2_256 code hash vs approved hash, clears pending |
| 40 | `generate_exit_proof()` | `ensure_signed` | Permissionless, deterministic hash, no storage writes |
| 41 | `ratify_constitutional_proposal(proposal_id)` | `ensure_signed` | Phase 6A: DualHouseProvider membership, one ratification per house |
| 42 | `lock_chain_parameter(key)` | `ConstitutionalAdminOrigin` | Not already locked |
| 43 | `unlock_chain_parameter(key)` | `ConstitutionalAdminOrigin` | Currently locked |
| 44 | `veto_emergency(emergency_block)` | `ensure_signed` | Council member, CONS-029 veto window, **Operational** (DOS-010), bounded council iteration, 1/3 threshold |

### 4.5 `on_initialize` Logic

1. **CONS-029 Emergency Finalization**: If `JaguarMode` is pending and veto window has passed, activate emergency. Bounded veto count check.
2. **Quarterly Participation Audit**: For accounts with stake ≥ `LargeHolderStakeThreshold`, check participation rate. If below `LargeHolderMinParticipationRate`, set `EffectiveVotingMultiplier` to 50 (50% weight reduction). Bounded to 20 accounts per quarterly check.
3. **Term Expiry Processing**: Process up to 10 expired council terms per block. Overflow entries are re-queued with `saturating_add(1)` to prevent infinite loops.

### 4.6 Helper Functions

| Function | Purpose | Security Notes |
|---|---|---|
| `isqrt(n: u128) -> u128` | Newton's method integer sqrt for QV | Deterministic, no FP, converges for all u128 |
| `is_council_member(account)` | Council membership check | O(1) storage lookup |
| `total_council_weight()` | Sum of council voting weights | DOS-018: bounded by `MaxCandidatesPerElection` |
| `council_size()` | Council member count | DOS-018: bounded iteration |
| `execute_action(action, proposal_id)` | Dispatch proposal actions | Routes to specialized handlers |
| `execute_treasury_spend(recipient, amount, id)` | Treasury transfer | S5-4 per-period cap enforcement |
| `execute_runtime_upgrade(hash, id)` | Store pending upgrade hash | Requires `apply_pending_runtime_upgrade` for actual code swap |
| `execute_parameter_change(param, value)` | Update governance parameter | Range validation per parameter type |
| `execute_department_action(dept, data, id)` | SCALE-decode and dispatch | AR-8: InvalidCallData on decode failure |
| `execute_emergency_action(type, id)` | Emergency action routing | CONS-029: ActivateEmergency enters veto window |
| `account_id()` | Treasury PalletId = `py/gover` | `into_account_truncating` |
| `calculate_voting_power(account)` | Base power + delegations | Simple 1+N model |
| `chain_param(key, default)` | Read on-chain parameter | Fallback to default |
| `is_constitutionally_ratified(id)` | Check dual-house ratification | Returns `(tech, gov)` booleans |

### 4.7 Security Findings — Governance

#### [G-1] HIGH — `reveal_vote` allows reveal after voting period ends

**Location**: `reveal_vote` (call_index 38), line ~6530  
**Issue**: The reveal phase allows votes **after** `voting_end`:
```rust
// Allow reveal during *or* after the voting period (common pattern)
ensure!(current_block >= proposal.voting_start, Error::<T>::VotingPeriodNotStarted);
```
There is no upper bound on when a vote can be revealed. This means committed votes can be revealed at any time after the voting period starts, including **after** `finalize_proposal` has already been called. While `finalize_proposal` snapshots the tally and changes the proposal status, `reveal_vote` does not check `proposal.status` — it only updates the tally on the stored proposal.  
**Impact**: A late reveal after finalization would update the tally in storage but not change the already-decided outcome. However, this creates a storage inconsistency where the recorded tally doesn't match the decision.  
**Recommendation**: Add `ensure!(proposal.status == ProposalStatus::Active, ...)` or enforce `current_block <= proposal.voting_end + REVEAL_WINDOW`.

#### [G-2] MEDIUM — `reveal_vote` skips QV/stake calculation that `cast_vote` performs

**Location**: `reveal_vote` (call_index 38), lines ~6545-6560  
**Issue**: `cast_vote` performs full QV calculation: `balance → stake_units → isqrt → MaxVotingUnits cap`, with community_rank + pouw + stake_weight base, conviction multiplier, and EffectiveVotingMultiplier. However, `reveal_vote` uses a **simplified** formula:
```rust
let community_rank = Self::community_ranks(&who);
let pouw_contribution = Self::pouw_contributions(&who);
let voting_weight = community_rank.saturating_add(pouw_contribution);
let conviction_multiplier = (conviction as u8).max(1);
let final_weight = voting_weight.saturating_mul(conviction_multiplier);
```
This omits: (1) balance-based stake units, (2) quadratic voting sqrt, (3) `MaxVotingUnits` cap, (4) `EffectiveVotingMultiplier` wealth-responsibility adjustment.  
**Impact**: Votes submitted via commit-reveal have different weight calculation than direct votes. Commit-reveal voters get only community_rank + pouw weight, which is typically much smaller than balance-based QV weight.  
**Recommendation**: Harmonize the weight calculation between `cast_vote` and `reveal_vote`. Extract the QV logic into a shared helper.

#### [G-3] MEDIUM — `claim_participation_reward` bounded scan doesn't cover all proposals

**Location**: `claim_participation_reward` (call_index 21), reward_type 0  
**Issue**: Vote reward eligibility uses `(0..max_check).any(|pid| Votes::<T>::contains_key(pid, &claimer))` where `max_check = next_id.min(50)`. This only checks proposal IDs 0–49. If a user voted on proposal #100 but not on #0–49, they cannot claim the vote reward.  
**Severity**: MEDIUM — functional limitation rather than vulnerability.  
**Recommendation**: Use `AccountVoteParticipation` storage (already exists) for eligibility check instead of scanning proposals.

#### [G-4] MEDIUM — `claim_participation_reward` reward_type 1 uses unbounded `Proposals::iter()`

**Location**: `claim_participation_reward` (call_index 21), reward_type 1  
**Issue**: The proposal reward check uses `Proposals::<T>::iter().take(50).any(...)`. While `.take(50)` bounds the iteration to 50, `Proposals::iter()` itself is nondeterministic in ordering — it iterates storage in hash order. This means the 50 proposals checked are arbitrary, not the most recent ones.  
**Recommendation**: Use `AccountLastProposal` storage to verify authorship instead.

#### [G-5] MEDIUM — `propose_treasury_spend` doesn't verify proposer is council member

**Location**: `propose_treasury_spend` (call_index 29)  
**Issue**: The extrinsic only checks `ComplianceProvider::can_participate_in_governance`, but the doc comment says "must be council member or FSC." Any compliant account can propose treasury spends of any size.  
**Recommendation**: Add `ensure!(Self::is_council_member(&proposer), ...)` or equivalent authorization check.

#### [G-6] MEDIUM — Treasury spend `description` accepts raw `Vec<u8>` before bounded conversion

**Location**: `propose_treasury_spend` (call_index 29)  
**Issue**: The `description: Vec<u8>` parameter is validated for length (`<= 512`) and then converted to `BoundedVec`. However, the initial `Vec<u8>` is unbounded in the call data — the length check happens after deserialization. This means a malicious caller could submit a very large `Vec<u8>` that passes SCALE-decoding but consumes excessive memory before the length check.  
**Recommendation**: Use `BoundedVec<u8, ConstU32<512>>` directly as the extrinsic parameter type.

#### [G-7] MEDIUM — `transfer_district_budget` uses raw subtraction for source budget

**Location**: `transfer_district_budget` (call_index 32), line ~6104  
**Issue**: While `available = from_budget.allocated.saturating_sub(from_budget.spent)` uses saturating sub, the transfer itself uses `from_budget.allocated.saturating_sub(amount)`. If `amount > allocated` (which the `ensure!` should prevent), the saturating_sub would silently set allocated to 0 rather than failing. The `ensure!(available >= amount)` guard should prevent this, but the use of saturating_sub masks potential bugs.  
**Status**: Likely safe due to the prior ensure, but could use checked_sub for defense-in-depth.

#### [G-8] LOW — `ConstitutionalAdminOrigin` TODO for mainnet

**Location**: Config trait, line ~1200  
**Issue**: `TODO(MAINNET): Replace with dual-council compound origin` — currently configured as a simple origin rather than the intended compound origin of TechnicalCouncil + GovernanceCouncil.  
**Recommendation**: Resolve before mainnet launch.

#### [G-9] LOW — Emergency override justification stored as event parameter, not storage

**Location**: `emergency_override_proposal` (call_index 35)  
**Issue**: The justification for emergency overrides is only emitted in the event, not stored in proposal metadata. This means the justification is only available via event indexing, not via direct storage query.

#### [G-10] LOW — `execute_emergency_proposal` doesn't call `execute_action`

**Location**: `execute_emergency_proposal` (call_index 33)  
**Issue**: This extrinsic marks the proposal as executed (sets `status = Executed`) but never calls `execute_action` — meaning the actual proposal actions (treasury spend, parameter change, etc.) are not performed. The proposal is recorded as executed without side effects.  
**Recommendation**: Either call `execute_action` after supermajority validation, or rename to clarify this is an approval/status-change only.

#### [G-11] LOW — `fast_track_referendum` could extend deadline instead of shortening

**Location**: `fast_track_referendum` (call_index 34)  
**Issue**: The fast-track sets `voting_end = current_block + 10_800`. If the referendum was already close to ending (e.g., 100 blocks away), this could actually **extend** the voting period. No check ensures the new deadline is earlier than the original.

#### [G-12] LOW — Delegation expiry not enforced during vote counting

**Location**: `delegate_vote` and `calculate_voting_power`  
**Issue**: `calculate_voting_power` checks `is_delegation_active` (which verifies expiry), but `cast_vote` does not incorporate delegated vote power. The delegation system stores delegations but the actual QV voting path in `cast_vote` doesn't aggregate delegated power. Delegations appear to be informational rather than mechanically enforced in the voting path.

#### [G-13] LOW — `execute_treasury_proposal` is permissionless once threshold met

**Location**: `execute_treasury_proposal` (call_index 31)  
**Issue**: Any signed account can trigger execution of an approved treasury spend. This is by design (anyone can be the "executor"), but it means the proposer cannot cancel an approved spend once the threshold is met.

#### [G-14] INFORMATIONAL — `isqrt` implementation is correct and deterministic

Newton's method with convergence guard (`while y < x`). For `n=0` returns 0. Invariant `isqrt(n)^2 <= n < (isqrt(n)+1)^2` holds for all u128 inputs. No floating-point operations.

#### [G-15] INFORMATIONAL — CONS-029 veto window is well-implemented

Emergency declaration enters `is_pending=true` state. Council members can veto before `veto_window_ends_at`. If 1/3 of council vetoes, emergency is cancelled. After window passes, `on_initialize` activates. `EmergencyVetoes::clear()` prevents stale vetoes.

#### [G-16] INFORMATIONAL — S5-4 treasury spend cap enforcement is sound

`TreasurySpendTracker` stores `(period_number, cumulative_spent)`. Period rolls over via `current_block / period_len`. Spend cap checked in `execute_treasury_spend` which is called by both direct treasury operations and proposal-based actions.

#### [G-17] INFORMATIONAL — Term expiry overflow protection

`on_initialize` processes max 10 expired terms per block. If more exist, they're re-queued at `current_block + 1` using `saturating_add` to prevent infinite loops on block number overflow.

### 4.8 Events Emitted

`ProposalSubmitted`, `VoteCast`, `ProposalFinalized`, `ProposalExecuted`, `CommunityRankUpdated`, `PoUWContributionUpdated`, `CouncilOverride`, `DepartmentManagerSet`, `DepartmentProposalSubmitted`, `CrossDepartmentApproval`, `BoardMemberAdded`, `BoardMemberRemoved`, `EmergencyDeclared`, `EmergencyEnded`, `EmergencyVetoCast`, `EmergencyVetoCancellation`, `DelegateNominated`, `DelegateVoteCast`, `VoteDelegationCreated`, `VoteDelegationRevoked`, `ProposalAmendmentSubmitted`, `RewardClaimed`, `ProposalQueuedWithPriority`, `DistrictElectionStarted`, `CandidateRegistered`, `DistrictVoteCast`, `DistrictElectionFinalized`, `DistrictRepresentativeElected`, `ReferendumCreated`, `ReferendumVoteCast`, `ReferendumFinalized`, `TreasurySpendExecuted`, `TreasurySpendProposed`, `TreasurySpendApproved`, `DistrictBudgetAllocated`, `DistrictBudgetSpent`, `DistrictBudgetTransferred`, `EmergencyProposalExecuted`, `ReferendumFastTracked`, `ProposalEmergencyOverride`, `ChainParameterUpdated`, `DepartmentActionExecuted`, `DepartmentFundsAllocated`, `ParameterChanged`, `RuntimeUpgradeExecuted`, `EmergencyActionExecuted`, `VoteCommitted`, `VoteRevealed`, `RuntimeUpgradeApplied`, `ExitProofGenerated`, `ConstitutionalRatificationCast`, `ConstitutionalRatificationComplete`, `ParameterLocked`, `ParameterUnlocked`

### 4.9 TODO / FIXME / HACK Comments

| Location | Comment | Severity |
|---|---|---|
| Config trait (~line 1200) | `TODO(MAINNET): Replace with dual-council compound origin` | LOW — must resolve before mainnet |

---

## 5. Cross-Pallet Integration Analysis

### 5.1 Trait Dependencies

```
governance ─→ CommunityRank (from community)    [voting weight base]
governance ─→ PoUWContributor (from community)  [voting weight base]
governance ─→ GovernanceParticipation (from community) [participation tracking]
governance ─→ FeeCalculator (from community)    [SRS-based fee discounts]
governance ─→ ComplianceProvider (KYC)           [participation gates]
governance ─→ BehaviorFlagProvider               [circuit breaker]
governance ─→ DualHouseProvider                  [constitutional ratification]
economy   ─→ GovernanceOrigin                    [minter auth, reserves]
economy   ─→ TourismOracle                      [booking verification]
community ─→ ComplianceProvider (KYC)            [endorsement, proposals]
community ─→ OracleAttestationOrigin             [activity attestation]
```

### 5.2 Cross-Pallet Security Concerns

#### [X-1] HIGH — `CommunityRank` weight used without re-validation in governance

**Issue**: Governance reads `CommunityRanks` storage (populated via `update_community_rank` root call, which mirrors the community pallet's SRS). The governance pallet trusts whatever value was set. If a bug in the community pallet's `CommunityRank::get_rank_weight()` returns an inflated value, governance voting weights are directly affected.  
**Mitigation**: The M61 fix caps rank weights (max 400), and `MaxVotingUnits` provides an absolute cap in `cast_vote`. However, this cap is not applied in `reveal_vote`.

#### [X-2] INFORMATIONAL — Economy pallet is independent

The economy pallet has no direct dependency on governance or community pallet storage. It only uses external origins (`GovernanceOrigin`) and oracles (`TourismOracle`). This is good isolation.

#### [X-3] INFORMATIONAL — Supply cap enforcement spans multiple pallets

DALLA supply cap (`MaxSupply = 501B`) is enforced in:
- `economy::on_initialize` (inflation)
- `community::complete_education_module` (CM-1)
- `community::claim_referral_reward` (CM-2)
- `governance::claim_participation_reward` (treasury-based, limited by balance)

All use either `checked_add` or treasury balance checks. No direct minting occurs in governance — rewards are transferred from treasury.

---

## 6. Findings Summary Table

| ID | Severity | Pallet | Title |
|----|----------|--------|-------|
| G-1 | HIGH | Governance | `reveal_vote` allows reveal after voting period and finalization |
| G-2 | MEDIUM | Governance | `reveal_vote` skips QV/stake weight calculation |
| C-1 | HIGH | Community | Permissionless `update_srs` cascades compromised components |
| X-1 | HIGH | Cross-pallet | CommunityRank weight trusted without re-validation |
| G-3 | MEDIUM | Governance | Bounded scan misses proposals beyond ID 49 |
| G-4 | MEDIUM | Governance | `Proposals::iter()` nondeterministic ordering |
| G-5 | MEDIUM | Governance | `propose_treasury_spend` missing council check |
| G-6 | MEDIUM | Governance | Raw `Vec<u8>` before bounded conversion |
| C-2 | MEDIUM | Community | Endorsement sybil resistance is SRS-only |
| C-3 | MEDIUM | Community | No quorum check for community proposals |
| G-10 | LOW | Governance | `execute_emergency_proposal` doesn't call `execute_action` |
| G-11 | LOW | Governance | `fast_track_referendum` could extend deadline |
| G-12 | LOW | Governance | Delegation not mechanically enforced in voting |
| G-8 | LOW | Governance | ConstitutionalAdminOrigin TODO for mainnet |
| G-9 | LOW | Governance | Emergency override justification not in storage |
| G-13 | LOW | Governance | Permissionless treasury execution |
| G-7 | LOW | Governance | Saturating sub masks potential budget bugs |
| E-1 | HIGH | Economy | `integrity_test` assert is compile-time only |
| E-2 | MEDIUM | Economy | Oracle trust model undocumented |
| C-4 | LOW | Community | Fee exemption reset timing |
| C-5 | LOW | Community | Dual-origin participation recording |
| E-3 | LOW | Economy | MintCallsThisBlock reset timing |
| G-14 | INFO | Governance | isqrt implementation correct |
| G-15 | INFO | Governance | CONS-029 veto window sound |
| G-16 | INFO | Governance | S5-4 treasury cap sound |

---

## 7. Recommendations

### Priority 1 — Before Mainnet

1. **[G-1] Fix `reveal_vote` timing**: Add `ensure!(proposal.status == ProposalStatus::Active, ...)` or a bounded reveal window (`voting_end + REVEAL_GRACE_PERIOD`).

2. **[G-2] Harmonize QV calculation**: Extract the full QV weight computation (balance→stake_units→isqrt→MaxVotingUnits→EffectiveVotingMultiplier) into a `fn calculate_vote_weight(who, conviction) -> u32` helper. Call it from both `cast_vote` and `reveal_vote`.

3. **[G-5] Gate `propose_treasury_spend`**: Add council membership or FSC role check for proposers.

4. **[G-8] Resolve `ConstitutionalAdminOrigin`**: Replace with compound origin from TechnicalCouncil + GovernanceCouncil.

5. **[G-10] Wire `execute_emergency_proposal`**: Either call `execute_action` or clearly document that this is an approval-only status change that requires a separate `execute_proposal` call.

### Priority 2 — Hardening

6. **[G-6] Use `BoundedVec` parameter types**: Change `description: Vec<u8>` to `BoundedVec<u8, ConstU32<512>>` in `propose_treasury_spend` and `transfer_district_budget` to prevent large payload deserialization.

7. **[G-3/G-4] Fix reward eligibility**: Use `AccountVoteParticipation` / `AccountLastProposal` for eligibility instead of bounded scans.

8. **[C-3] Add quorum to community proposals**: Require minimum vote count or weight.

9. **[E-1] Add runtime invariant check**: Log warning in `on_finalize` if `TotalBbzdSupply > CentralBankReserves`.

### Priority 3 — Defense in Depth

10. **[G-11] Clamp fast-track deadline**: `referendum.voting_end = fast_track_deadline.min(referendum.voting_end)`.

11. **[C-2] Strengthen endorsement sybil resistance**: Consider minimum account age or deposit.

12. **[G-7] Use checked arithmetic**: Replace `saturating_sub` with `checked_sub` + error propagation for budget transfers.

---

*End of audit report.*
