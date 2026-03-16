# BelizeChain Governance-Layer Security Audit

**Auditor**: AI Security Review (Phase 2)  
**Date**: 2026-07-07  
**Scope**: All governance-related pallets — `governance`, `justice`, `community`, `moderation`, `whistleblower`  
**Method**: Full source read-through (read-only), cross-pallet origin analysis  
**Runtime Config**: `runtime/src/lib.rs` origin wiring verified  

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Pallet Inventory & Line Counts](#2-pallet-inventory)
3. [Critical Findings (CRITICAL)](#3-critical-findings)
4. [High-Severity Findings (HIGH)](#4-high-severity-findings)
5. [Medium-Severity Findings (MEDIUM)](#5-medium-severity-findings)
6. [Low-Severity Findings (LOW)](#6-low-severity-findings)
7. [Informational / Design Notes](#7-informational)
8. [Security Question Responses](#8-security-question-responses)
9. [Origin & Access Control Matrix](#9-origin-matrix)
10. [Positive Security Properties](#10-positive-properties)
11. [Recommendations Summary](#11-recommendations)

---

## 1. Executive Summary

Five governance-layer pallets totalling ~10,800 lines of Rust were audited. The governance pallet alone is ~7,500 lines containing 44 extrinsics and 30+ helper functions. The audit identified **4 CRITICAL**, **10 HIGH**, **9 MEDIUM**, and **5 LOW** severity findings.

**Most impactful issues:**
- Whistleblower anonymity is broken at the transaction layer (CRITICAL)
- Referendum quorum uses a hardcoded placeholder of 1000 (CRITICAL)
- Moderation has no appeal mechanism — permanent censorship risk (HIGH)
- Council override bypasses all timelocks (HIGH)
- Participation rewards can be claimed repeatedly across reward types without cooldown (HIGH)
- `execute_emergency_proposal` skips `execute_action()` — marks executed without doing anything (CRITICAL)

---

## 2. Pallet Inventory

| Pallet | File | Lines | Extrinsics | Origin Model |
|--------|------|-------|------------|-------------|
| governance | `pallets/governance/src/lib.rs` | ~7,500 | 44 (`call_index` 0–43) | Root, `CouncilOrigin` (TechnicalCouncilMajority), `CommunityOrigin` (GovernanceCouncilMajority), `ConstitutionalAdminOrigin` (TechnicalCouncilSuperMajority) |
| justice | `pallets/justice/src/lib.rs` | ~570 | 6 | Root, `MediatorOrigin` (TechnicalCouncilMember), `GovernanceOrigin` (GovernanceCouncilMajority) |
| community | `pallets/community/src/lib.rs` | ~2,200 | 14 | Root, `GovernanceOrigin` (GovernanceCouncilMajority), `OracleAttestationOrigin` (TechnicalCouncilMember), signed |
| moderation | `pallets/moderation/src/lib.rs` | ~510 | 5 | `ModeratorAdminOrigin` (GovernanceCouncilMajority), `NawalOracleOrigin` (TechnicalCouncilMember), signed |
| whistleblower | `pallets/whistleblower/src/lib.rs` | ~400 | 4 | `ReviewerOrigin` (TechnicalCouncilMember), `GovernanceOrigin` (GovernanceCouncilMajority), signed |

---

## 3. CRITICAL Findings

### CRIT-1: Whistleblower Anonymity Broken at Transaction Layer

**Pallet**: `whistleblower` — `submit_report()` (line ~223)  
**Severity**: CRITICAL  
**Category**: Privacy / Whistleblower Protection Failure

`submit_report` uses `ensure_signed(origin)` to get the submitter account, then stores an `alias_hash` in the `Report` struct. However, the submitter's real `AccountId` is visible on-chain in the transaction itself. Any block explorer can correlate `report_id` → transaction sender → real identity.

```rust
let submitter = ensure_signed(origin)?; // Real account visible on-chain
// ...
let report = Report {
    alias_hash: alias_hash.clone(), // Pseudonym is decorative
    // ...
};
```

**Impact**: Whistleblowers reporting government corruption are immediately identifiable, defeating the purpose of the entire pallet. This is a safety-critical failure in a jurisdiction-specific blockchain.

**Recommendation**: Implement a relay/proxy submission pattern (e.g., a designated relayer account submits on behalf of the whistleblower), or use a commitment scheme where the whistleblower identity is never revealed on-chain.

---

### CRIT-2: Referendum Quorum Uses Hardcoded Placeholder

**Pallet**: `governance` — `create_referendum()` (line ~4460)  
**Severity**: CRITICAL  
**Category**: Voting Integrity

`ReferendumEligibleVoters` is hardcoded to 1000 at referendum creation:

```rust
ReferendumEligibleVoters::<T>::insert(referendum_id, 1000u32); // Placeholder
```

`finalize_referendum` (line ~4770) then checks quorum against this fake number:

```rust
let eligible_voters = ReferendumEligibleVoters::<T>::get(referendum_id);
let participation_rate = (total_voters * 100) / eligible_voters;
ensure!(participation_rate >= referendum.quorum_percentage, ...);
```

**Impact**: With only 1000 "eligible voters", a referendum with 10% quorum needs only ~100 weighted votes to pass — regardless of actual token holder count. This makes referendums trivially gameable.

**Recommendation**: Derive eligible voter count from actual on-chain data (total accounts with balance, total staked, or total KYC-verified accounts).

---

### CRIT-3: `execute_emergency_proposal` Marks Executed Without Executing Action

**Pallet**: `governance` — `execute_emergency_proposal()` (line ~6172)  
**Severity**: CRITICAL  
**Category**: Logic Error / State Corruption

This extrinsic checks supermajority and sets `proposal.status = ProposalStatus::Executed`, but **never calls `execute_action()`**. Contrast with the normal `execute_proposal()` (line ~4870) which properly calls `Self::execute_action(&action, proposal_id)`.

```rust
// execute_emergency_proposal:
proposal.executed_at = Some(current_block);
proposal.status = ProposalStatus::Executed; // ← Marked as done
Proposals::<T>::insert(proposal_id, proposal.clone());
// No execute_action() call — proposal effect never applied
```

**Impact**: Emergency proposals (treasury spends, parameter changes, runtime upgrades) will appear executed in storage but their actual effects (fund transfers, parameter updates) never happen. This creates a false sense of action during emergencies.

**Recommendation**: Add `Self::execute_action(&proposal.action.clone().ok_or(Error::<T>::NoActionDefined)?, proposal_id)?;` before marking the proposal as executed.

---

### CRIT-4: `emergency_override_proposal` Bypasses All Checks

**Pallet**: `governance` — `emergency_override_proposal()` (line ~6281)  
**Severity**: CRITICAL  
**Category**: Governance Bypass

Root can force-execute or force-reject **any** proposal during Jaguar Mode with no enactment delay, no constitutional ratification, no treasury spend cap check, and no `execute_action()` call. Only marks status — same issue as CRIT-3.

Additionally, if `execute = true`, a constitutional amendment could be marked as executed without dual-house ratification, bypassing the four-eyes check in `execute_proposal()`.

**Impact**: During any emergency (which Root can declare), Root has unchecked power to mark any proposal as executed or rejected, with no actual effects applied and no constitutional safeguards.

**Recommendation**: Either (a) remove the `execute=true` path and only allow force-rejection, or (b) route execution through `execute_proposal()` with full checks but reduced timelocks.

---

## 4. HIGH Findings

### HIGH-1: Moderation Has No Appeal Process

**Pallet**: `moderation` (line ~360)  
**Severity**: HIGH  
**Category**: Censorship Risk

`review_content()` makes a final ruling (`Removed`, `Approved`, etc.) with no appeal mechanism. A single moderator (`ensure_signed` + `Moderators` storage check) can unilaterally remove content permanently.

**Impact**: A compromised or malicious moderator can censor any content with no recourse. In a sovereign nation-state blockchain, this is a civil liberties concern.

**Recommendation**: Add multi-moderator consensus for `Removed` rulings, and an appeal extrinsic that escalates to Governance Council vote.

---

### HIGH-2: Justice Escrow Double-Call Loses Funds

**Pallet**: `justice` — `escrow_slash()` (line ~542)  
**Severity**: HIGH  
**Category**: Fund Loss

`escrow_slash` uses `Currency::reserve` to lock funds, then `insert` to store the amount:

```rust
T::Currency::reserve(&account, amount)?;
EscrowedAmounts::<T>::insert(&account, amount); // Overwrites previous
```

If called twice for the same account, the first reserved amount is lost in storage (overwritten) but remains reserved on the account — permanently locked.

**Recommendation**: Use `mutate` to accumulate: `EscrowedAmounts::<T>::mutate(&account, |existing| *existing = existing.saturating_add(amount));`

---

### HIGH-3: Justice Appeal Has No Re-Ruling Mechanism

**Pallet**: `justice` — `appeal_ruling()` (line ~430)  
**Severity**: HIGH  
**Category**: Process Deadlock

`appeal_ruling` changes dispute status to `Appealed` but provides no mechanism for a second ruling. Appealed disputes are stuck permanently — no extrinsic can transition from `Appealed` to any other state.

**Recommendation**: Add a `review_appeal()` extrinsic for elevated mediators or governance council.

---

### HIGH-4: Council Override Bypasses All Timelocks

**Pallet**: `governance` — `council_override()` (line ~3826)  
**Severity**: HIGH  
**Category**: Timelock Bypass

`council_override` sets `proposal.status = ProposalStatus::Approved` without setting an enactment delay or storing in `ProposalEnactmentBlock`. When subsequently called via `execute_proposal`, the enactment check `ensure!(current_block >= enactment_block, ...)` uses the default (0), meaning immediate execution.

**Impact**: Council can approve + immediately execute a proposal bypassing CONS-030 timelocks (RuntimeUpgradeMinTimelock of ~7 days, ParameterChangeMinTimelock of ~48h).

**Recommendation**: Set `ProposalEnactmentBlock` with at minimum the action-specific timelock when council overrides a proposal.

---

### HIGH-5: Whistleblower Pool Can Be Funded Without Real Transfer

**Pallet**: `whistleblower` — `fund_whistleblower_pool()` (line ~360)  
**Severity**: HIGH  
**Category**: Unbacked Rewards

`fund_whistleblower_pool` does `GovernanceOrigin::ensure_origin(origin.clone())` then `ensure_signed(origin)`. If `GovernanceOrigin` is a collective (dispatch origin), `ensure_signed` may fail silently. The pool balance is incremented in storage but no actual token transfer occurs.

Combined with `claim_reward` using `deposit_creating` (minting) rather than transferring from pool balance, rewards are inflationary and unbacked.

**Recommendation**: Use `Currency::transfer` from a funded pool account, not `deposit_creating`.

---

### HIGH-6: Conviction Multiplier Off-By-One

**Pallet**: `governance` — `cast_vote()` (line ~3552) and `reveal_vote()` (line ~6410)  
**Severity**: HIGH  
**Category**: Voting Weight Miscalculation

```rust
let conviction_multiplier = (conviction as u32).max(1);
```

Conviction=0 and Conviction=1 both give 1x multiplier. Conviction=2 gives 2x, conviction=3 gives 3x, etc. Standard Substrate conviction voting uses Conviction=1 → 1x lock but 1x weight, Conviction=2 → 2x weight. The BelizeChain documentation describes conviction=1 as "2x", but code gives 1x.

**Impact**: Users selecting conviction=1 expecting 2x weight get only 1x. Voting power calculations across the system are inconsistent with documentation.

**Recommendation**: Use a lookup table: `[1, 1, 2, 3, 4, 5, 6]` for convictions 0–6, or document that conviction value IS the multiplier directly.

---

### HIGH-7: Quarterly Participation Check Bounded to 20 Accounts

**Pallet**: `governance` — `on_initialize` (line ~2510)  
**Severity**: HIGH  
**Category**: Accountability Gap

`MAX_CHECKS_PER_QUARTER = 20` means only 20 large-holder accounts are checked per quarter. If >20 large holders exist, some permanently avoid participation penalties.

**Impact**: Whale accounts can accumulate voting power without participating, undermining the "wealth → responsibility" mechanism.

**Recommendation**: Use a rotating cursor (stored in storage) to process all accounts across multiple blocks rather than a fixed per-quarter limit.

---

### HIGH-8: Participation Reward Unlimited One-Time Claim

**Pallet**: `governance` — `claim_participation_reward()` (line ~5453)  
**Severity**: HIGH  
**Category**: Treasury Drain

Each reward type (vote=10 DALLA, proposal=100 DALLA, council=500 DALLA) can be claimed once per account based on `RewardsClaimed::<T>::get(&claimer, reward_type)`. However:

1. **Vote reward** (type 0): Only checks if any vote exists in proposals 0..50. A single vote on any proposal = 10 DALLA forever.
2. **Proposal reward** (type 1): Only checks if any proposal authored. One proposal = 100 DALLA forever.
3. **Council reward** (type 2): Only checks current council membership. No period tracking.

There is no period-based reset, so rewards are one-time per account per type. While this limits per-account drain, with unlimited accounts (Sybil with minimum KYC), total drainage scales linearly.

**Impact**: With fee-free proposals (deposit always returned per MEDIUM-1), an attacker can create N KYC accounts, submit one proposal each, vote once each, and claim 110 DALLA per account.

**Recommendation**: Add per-epoch or per-period reward tracking, require minimum participation thresholds, and enforce supply caps on total rewards distributed.

---

### HIGH-9: `propose_treasury_spend` Has No Council-Member Check on Proposer

**Pallet**: `governance` — `propose_treasury_spend()` (line ~5832)  
**Severity**: HIGH  
**Category**: Access Control Gap

Any KYC-compliant account can create treasury spend proposals. The threshold mechanism (1/3/4 approvals) provides multi-sig protection, but proposals < 10K DALLA require only **1 approval** from any council member. A colluding council member + any KYC account can drain treasury 10K DALLA at a time.

```rust
let threshold = if amount_value < 10_000_000_000_000 { // < 10K DALLA
    1u8  // Single approval (!)
};
```

**Impact**: Single council member compromise enables sub-10K treasury drain with fabricated small proposals.

**Recommendation**: Require minimum 2-of-7 approval even for small spends, or restrict proposal creation to council/board members.

---

### HIGH-10: `finalize_district_election` Requires Root Only

**Pallet**: `governance` — `finalize_district_election()` (line ~5200)  
**Severity**: HIGH  
**Category**: Centralization Risk

District elections can only be finalized by Root. If Root (sudo) is removed post-launch, elections cannot be finalized unless a governance proposal that dispatches as Root is passed — but that requires an already-functioning governance council, creating a chicken-and-egg problem.

Similarly, `start_district_election` also requires Root (line ~4990).

**Recommendation**: Allow election finalization by `CouncilOrigin` or after a timeout, make it permissionless.

---

## 5. MEDIUM Findings

### MED-1: Proposal Deposit Always Refunded

**Pallet**: `governance` — `finalize_proposal()` (line ~3687)  
**Severity**: MEDIUM  
**Category**: Spam Deterrent Failure

```rust
// Always return deposit (whether approved or rejected)
T::Currency::unreserve(&proposal.proposer, T::MinimumDeposit::get());
```

Rejected proposals get full deposit back. The only cost of spam proposals is the temporary lock during the voting period.

**Impact**: Low barrier to governance spam — submit frivolous proposals with no economic risk. Combined with HIGH-8 (proposal reward), users are actually **paid** to submit proposals.

**Recommendation**: Slash a percentage of deposit on rejection (e.g., 10–50%).

---

### MED-2: Community `attest_participation` Dual Origin Pattern

**Pallet**: `community` — `attest_participation()` (line ~1588)  
**Severity**: MEDIUM  
**Category**: Origin Handling

```rust
T::OracleAttestationOrigin::ensure_origin(origin.clone())?;
let attester = ensure_signed(origin)?;
```

If `OracleAttestationOrigin` is `TechnicalCouncilMember` (EnsureMember), it consumes the origin to verify membership. The subsequent `ensure_signed` then fails with `BadOrigin` because the origin was already consumed.

Same pattern exists in `whistleblower::fund_whistleblower_pool` (line ~360).

**Impact**: These extrinsics may be uncallable in practice, or only work when the origin type unexpectedly allows both checks.

**Recommendation**: Use a single origin check and derive signer from it, or use `origin.clone()` properly with an origin type that supports both checks.

---

### MED-3: EnsureRoot Still Active for Collective Bootstrap

**Pallet**: Runtime — `runtime/src/lib.rs` (lines 705–729)  
**Severity**: MEDIUM  
**Category**: Privilege Retention

Both `TechnicalCouncil` and `GovernanceCouncil` use `EnsureRoot` for `SetMembersOrigin`, `DisapproveOrigin`, and `KillOrigin`. Comments say "bootstrap only, replaced by governance post-launch" but code still uses `EnsureRoot`.

**Impact**: Root (sudo) can unilaterally replace all council members, disapprove any motion, or kill any proposal — even after "decentralization".

**Recommendation**: Before mainnet, replace with dual-council supermajority origins or remove sudo entirely.

---

### MED-4: Community Proposal Deposit Slashed on Treasury Transfer Failure

**Pallet**: `community` — `finalize_community_proposal()` (line ~1159)  
**Severity**: MEDIUM  
**Category**: Unfair Penalty

If a community proposal is approved for treasury spend but the treasury is empty, the proposer's deposit is slashed as a penalty — even though the proposal legitimately passed the vote.

**Impact**: Proposers penalized for conditions beyond their control.

---

### MED-5: Moderation Single-Moderator Unilateral Removal

**Pallet**: `moderation` — `review_content()` (line ~360)  
**Severity**: MEDIUM  
**Category**: Access Control

A single moderator (any account in `Moderators` storage) can unilaterally rule content as `Removed`. No multi-moderator consensus, quorum, or cooldown.

**Impact**: Single point of censorship failure.

---

### MED-6: Nawal AI Assessment Can Overwrite Without Limit

**Pallet**: `moderation` — `submit_nawal_assessment()` (line ~451)  
**Severity**: MEDIUM  
**Category**: Oracle Manipulation

`submit_nawal_assessment` can repeatedly overwrite AI scores for flagged content. A compromised oracle (single `TechnicalCouncilMember`) can manipulate content moderation queues.

**Recommendation**: Add score finality or require multiple oracle confirmations.

---

### MED-7: `flag_content` Has No Sybil Protection

**Pallet**: `moderation` — `flag_content()` (line ~302)  
**Severity**: MEDIUM  
**Category**: Sybil Attack

No KYC check on flaggers. Multiple accounts can flag the same content to trigger auto-moderation thresholds. Only `ensure_signed` is required.

---

### MED-8: Delegation Votes Not Used in `cast_vote`

**Pallet**: `governance` — `cast_vote()` (line ~3552), `delegate_vote()` (line ~5393)  
**Severity**: MEDIUM  
**Category**: Feature Incomplete

Delegation infrastructure exists (`VoteDelegations`, `DelegationReceivers`, `calculate_voting_power()`), but `cast_vote` and `reveal_vote` never call `calculate_voting_power`. The delegate's weight is calculated from their personal balance/rank only — delegated votes are ignored.

**Impact**: Liquid democracy is marketed but non-functional. Delegation has zero effect on vote weight.

**Recommendation**: Incorporate `count_delegated_votes()` into `cast_vote` weight calculation.

---

### MED-9: `fast_track_referendum` Can Shorten Below Original End

**Pallet**: `governance` — `fast_track_referendum()` (line ~6240)  
**Severity**: MEDIUM  
**Category**: Voting Integrity

Sets `voting_end = current_block + 10_800`. If the referendum already had voters who planned to vote later in the original window, their votes are lost. No notification mechanism exists on-chain.

---

## 6. LOW Findings

### LOW-1: Justice `mediator_ruling` Dual Origin Consumption

**Pallet**: `justice` — `mediator_ruling()` (line ~337)  
**Severity**: LOW

Uses `origin.clone()` for dual origin check. Low impact if `MediatorOrigin` is `TechnicalCouncilMember`.

---

### LOW-2: Justice Mediated Slash Integer Division Dust

**Pallet**: `justice` (line ~337 ruling path)  
**Severity**: LOW

`escrowed * bps / 10_000` can lose up to 9999 planck units per operation due to integer division truncation. Economically negligible at 12-decimal precision.

---

### LOW-3: `amend_proposal` Only Allows One Amendment

**Pallet**: `governance` — `amend_proposal()` (line ~5493)  
**Severity**: LOW

```rust
ensure!(!ProposalAmendments::<T>::contains_key(proposal_id), Error::<T>::AmendmentAlreadyExists);
```

Only one amendment per proposal lifetime. Secondary improvements require a new proposal.

---

### LOW-4: `claim_participation_reward` Bounded Scan Uses Arbitrary Limit

**Pallet**: `governance` — `claim_participation_reward()` (line ~5453)  
**Severity**: LOW

Vote reward scans proposals 0..50, proposal reward scans `Proposals::iter().take(50)`. If the user's activity is in proposals >50, the check fails and reward is denied.

---

### LOW-5: Bond on Dismissed Whistleblower Reports Never Returned

**Pallet**: `whistleblower` — `review_report()` (line ~271)  
**Severity**: LOW

When a report is dismissed, the bond remains reserved on the submitter's account. It is never slashed (burned) or unreserved (returned). The submitter permanently loses access to those funds without an explicit slash event.

**Recommendation**: Either unreserve (return) on dismissal, or explicitly slash with a documented rationale.

---

## 7. Informational / Design Notes

### INFO-1: Commit-Reveal Voting Infrastructure Pre-Provisioned

`VoteCommitments` storage and `commit_vote`/`reveal_vote` extrinsics (call_index 37/38) are fully implemented with proper G-1/G-2 fixes. The reveal window is bounded to `voting_end + VotingPeriod`, and reveals on finalized proposals are rejected. Well-implemented.

### INFO-2: `apply_pending_runtime_upgrade` Has Proper Hash Verification

Line ~6560: `blake2_256(&code) == approved_hash` check prevents governance-unapproved code from being deployed. Good security property.

### INFO-3: Treasury Spend Cap (S5-4) Well-Implemented

`execute_treasury_spend` helper properly tracks per-period spending via `TreasurySpendTracker` and enforces `MaxTreasurySpendPerPeriod`. This is a strong defense against dramatic treasury drain even if multiple proposals pass.

### INFO-4: Constitutional Ratification Correctly Implements Four-Eyes

`ratify_constitutional_proposal` (call_index 41) properly requires independent ratification from both TechnicalCouncil and GovernanceCouncil houses, and `execute_proposal` correctly gates on `is_constitutionally_ratified` before executing Constitutional proposals.

### INFO-5: Supply Cap Checks Present in Community Rewards

Community pallet education rewards (CM-1 FIX) and referral rewards (CM-2 FIX) both check `MaxDallaSupply` before minting. Good.

---

## 8. Security Question Responses

### Q1: Can governance be captured by a small coalition?

**Partially.** TechnicalCouncil (max 12) majority = 7 members can:
- Override proposals during emergencies (council_override)
- Approve treasury spends (any council member can approve, 1 sig for <10K)
- Declare emergencies (via CouncilOrigin)
- Fast-track referendums
- Execute emergency proposals (without actually executing actions — CRIT-3)

GovernanceCouncil (max 32) majority = 17 members can sanction accounts, manage moderators, and approve economy changes. A coalition of 7 technical + 17 governance members controls the system. District elections provide some decentralization but require Root to initiate.

**Mitigations present**: Term limits, consecutive term limits, participation requirements (though bounded — HIGH-7), community rank weighting.

### Q2: Can the treasury be drained?

**Yes, through multiple vectors:**
1. Sub-10K proposals need only 1 council approval (HIGH-9)
2. Proposal deposits always returned (MED-1) + proposal rewards paid (HIGH-8)
3. Per-period spend cap exists (INFO-3) but does not cover treasury spend proposals (separate path)
4. National treasury reserve check exists in `execute_treasury_proposal`

**Best protection**: `MaxTreasurySpendPerPeriod` cap on `execute_treasury_spend` helper (used by governance proposals). Treasury spend proposals through the multi-sig path have separate approval flow.

### Q3: Can timelocks be bypassed?

**Yes:**
1. `council_override` sets no enactment delay (HIGH-4)
2. `execute_emergency_proposal` has no timelock (CRIT-3)
3. `emergency_override_proposal` has no timelock (CRIT-4)
4. Normal `execute_proposal` correctly enforces CONS-030 timelocks

### Q4: Is there a moderator appeal mechanism?

**No.** Moderation rulings are final and permanent (HIGH-1). No appeal extrinsic exists. Content ruled as `Removed` cannot be restored through any on-chain mechanism.

### Q5: Is whistleblower anonymity protected?

**No.** Transaction sender identity is visible on-chain (CRIT-1). The `alias_hash` provides only in-storage pseudonymity — not transaction-layer anonymity.

### Q6: Can justice outcomes be manipulated?

**Partially:**
- Single mediator can rule on disputes (line ~337)
- Appeals create permanent deadlocks (HIGH-3)
- Escrow double-call loses funds (HIGH-2)
- However, mediators must be in `AuthorizedMediators` storage (access-controlled)

### Q7: Is voting integrity maintained?

**Partially compromised:**
- Conviction multiplier off-by-one (HIGH-6)
- Referendum quorum uses placeholder (CRIT-2)
- Delegation votes not actually counted (MED-8)
- Commit-reveal properly implemented (INFO-1)
- Quadratic voting + MaxVotingUnits + wealth-responsibility multiplier are sound

---

## 9. Origin & Access Control Matrix

| Extrinsic | Origin | Runtime Binding |
|-----------|--------|----------------|
| `submit_proposal` | `ensure_signed` + KYC | Any KYC account |
| `cast_vote` / `reveal_vote` | `ensure_signed` + KYC | Any KYC account |
| `finalize_proposal` | `ensure_signed` | Any account (permissionless after voting ends) |
| `council_override` | `CouncilOrigin` | TechnicalCouncilMajority (>1/2 of max 12) |
| `declare_emergency` | `CouncilOrigin` or Root | TechnicalCouncilMajority or sudo |
| `veto_emergency` | `ensure_signed` + council check | Any council member |
| `execute_proposal` | `ensure_signed` + council check | Any council member |
| `execute_emergency_proposal` | Root or `CouncilOrigin` | sudo or TechnicalCouncilMajority |
| `emergency_override_proposal` | Root | sudo only |
| `start_district_election` | Root | sudo only |
| `finalize_district_election` | Root | sudo only |
| `allocate_district_budget` | Root | sudo only |
| `transfer_district_budget` | Root | sudo only |
| `update_chain_parameter` | Root | sudo only |
| `apply_pending_runtime_upgrade` | Root | sudo only |
| `lock_chain_parameter` | `ConstitutionalAdminOrigin` | TechnicalCouncilSuperMajority (>2/3) |
| `unlock_chain_parameter` | `ConstitutionalAdminOrigin` | TechnicalCouncilSuperMajority (>2/3) |
| `propose_treasury_spend` | `ensure_signed` + KYC | Any KYC account |
| `approve_treasury_spend` | `ensure_signed` + council check | Any council member |
| `execute_treasury_proposal` | `ensure_signed` | Any account (permissionless after approval) |
| `review_content` (moderation) | `ensure_signed` + moderator check | Authorized moderator |
| `submit_nawal_assessment` | `NawalOracleOrigin` | Single TechnicalCouncilMember |
| `mediator_ruling` (justice) | `MediatorOrigin` | Single TechnicalCouncilMember |
| `review_report` (whistleblower) | `ReviewerOrigin` | Single TechnicalCouncilMember |
| `ratify_constitutional_proposal` | `ensure_signed` + house membership | TechnicalCouncil or GovernanceCouncil member |
| `sanction_account` (community) | `GovernanceOrigin` | GovernanceCouncilMajority |

---

## 10. Positive Security Properties

1. **CONS-029 Emergency Veto**: Emergencies enter a pending state with a veto window. Council can cancel with 1/3+1 vetoes. Well-implemented.
2. **CONS-030 Action-Type Timelocks**: `finalize_proposal` correctly computes enactment delays per action type (7 days for runtime upgrades, 48h for parameter changes).
3. **S5-4 Treasury Spend Cap**: Per-period spend tracking prevents catastrophic single-period drainage.
4. **Dual Execution Check**: `execute_proposal` checks both `proposal.executed_at.is_some()` AND `ExecutedProposals::contains_key` — defense in depth against replay.
5. **Constitutional Dual-House Ratification**: Four-eyes principle properly enforced for constitutional amendments.
6. **Bounded Iterations**: All `iter()` calls have explicit `.take(max)` bounds (DOS-004, DOS-015, DOS-018). Good DoS protection.
7. **Supply Cap Enforcement**: Community education and referral rewards check `MaxDallaSupply` before minting (CM-1, CM-2 fixes).
8. **Quadratic Voting**: `isqrt()` uses deterministic Newton's method — no floating point in consensus.
9. **Parameter Range Validation**: `execute_parameter_change` validates all governance parameters within safe bounds.
10. **Commit-Reveal**: G-1 bounded reveal window + G-2 full weight calculation matching `cast_vote`.

---

## 11. Recommendations Summary

| ID | Severity | Finding | Recommendation |
|----|----------|---------|----------------|
| CRIT-1 | CRITICAL | Whistleblower identity exposed | Implement relay/proxy submission |
| CRIT-2 | CRITICAL | Referendum quorum placeholder | Derive from actual on-chain voter count |
| CRIT-3 | CRITICAL | Emergency execute marks done, doesn't act | Add `execute_action()` call |
| CRIT-4 | CRITICAL | Emergency override bypasses all safeguards | Remove force-execute or route through normal execution |
| HIGH-1 | HIGH | No moderation appeal | Add appeal extrinsic with council escalation |
| HIGH-2 | HIGH | Escrow double-call fund loss | Use `mutate` to accumulate |
| HIGH-3 | HIGH | Appeal creates permanent deadlock | Add `review_appeal()` extrinsic |
| HIGH-4 | HIGH | Council override bypasses timelocks | Set `ProposalEnactmentBlock` on override |
| HIGH-5 | HIGH | Whistleblower pool unbacked rewards | Use `Currency::transfer` from funded account |
| HIGH-6 | HIGH | Conviction multiplier off-by-one | Use lookup table or update documentation |
| HIGH-7 | HIGH | Participation check bounds to 20 | Use rotating cursor across blocks |
| HIGH-8 | HIGH | One-time reward unlimited accounts | Add per-period tracking and supply caps |
| HIGH-9 | HIGH | Treasury <10K needs 1 approval | Require minimum 2-of-7 |
| HIGH-10 | HIGH | District elections Root-only | Allow CouncilOrigin or permissionless timeout |
| MED-1 | MEDIUM | Deposit always refunded | Slash percentage on rejection |
| MED-2 | MEDIUM | Dual origin consumption | Single origin check |
| MED-3 | MEDIUM | EnsureRoot for collective bootstrap | Replace before mainnet |
| MED-4 | MEDIUM | Deposit slashed on treasury empty | Only slash on rejection, not treasury failure |
| MED-5 | MEDIUM | Single moderator removal | Multi-moderator consensus |
| MED-6 | MEDIUM | Nawal assessment unlimited overwrite | Score finality or multi-oracle |
| MED-7 | MEDIUM | Flag spam no KYC | Require KYC for flagging |
| MED-8 | MEDIUM | Delegation votes unused | Incorporate in `cast_vote` weight |
| MED-9 | MEDIUM | Fast-track shortens voting window | Add minimum floor (e.g., 1 hour) |

---

*End of Governance-Layer Security Audit Report*
