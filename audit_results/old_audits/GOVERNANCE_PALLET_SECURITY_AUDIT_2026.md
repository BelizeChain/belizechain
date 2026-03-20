# BelizeChain Governance Pallet — Security Audit Report

**Audit ID:** GOV-AUDIT-2026-001  
**Date:** 2026-07-12  
**Auditor:** AI Security Audit (line-by-line)  
**Scope:** `pallets/governance/src/` — 10,817 lines across 5 files  
**Prior Audit:** CONS-029/030 (38/38 remediated), PHASE_2 pallet audit  

---

## Executive Summary

The governance pallet implements a multi-house legislative system with 45 extrinsics covering proposals, voting, delegation, elections, treasury, emergency powers, commit-reveal voting, and constitutional ratification. Prior audit remediations (P0-16 through M48) are confirmed applied. This audit identifies **4 CRITICAL**, **14 WARNING**, and **8 INFO** findings. The most severe involve a PalletId mismatch between production and test code, an emergency override that marks proposals executed without performing the action, consecutive term limit bypass for elected officials, and single-person dual-house ratification.

| Severity | Count | Remediated |
|----------|-------|------------|
| CRITICAL | 4 | 0 |
| WARNING | 14 | 0 |
| INFO | 8 | 0 |
| **Total** | **26** | **0** |

---

## Per-File Scores

| File | Lines | Score | Rationale |
|------|-------|-------|-----------|
| `lib.rs` | 7,450 | **62/100** | 4 CRITICAL + 12 WARNING findings; solid access-control on most extrinsics but significant logic gaps in emergency override, term limits, and ratification |
| `mock.rs` | 272 | **55/100** | PalletId mismatch with production code; QuadraticVoting and ProposalCooldown disabled — masks real bugs |
| `benchmarking.rs` | 149 | **35/100** | Only 7/45 extrinsics benchmarked; one benchmark would fail production checks |
| `weights.rs` | 76 | **45/100** | All hand-estimated, not derived from actual benchmarking runs |
| `tests.rs` | 2,870 | **68/100** | ~100 tests with good error-path coverage, but 7 extrinsics completely untested including veto_emergency and ratify_constitutional_proposal |

**Aggregate Score: 58/100**

---

## Findings

### CRITICAL

#### GOV-C01: PalletId Mismatch Between Production and Test Code

- **Files:** `lib.rs` L~7230, `mock.rs` L~165
- **Category:** Configuration / Treasury
- **Description:** The `account_id()` function in `lib.rs` hardcodes `PalletId(*b"py/gover")` to derive the governance treasury account. However, `mock.rs` configures the pallet with `PalletId(*b"bz/govnc")`. In production, the runtime's `Config` provides the PalletId, so the hardcoded value in `account_id()` would override the runtime configuration. All treasury operations (reward claims, treasury spends, department budget allocations) would use the wrong account unless the runtime Config happens to match `*b"py/gover"`.
- **Impact:** Funds sent to/from the governance treasury in production could go to an uncontrolled account, or treasury operations could fail with insufficient balance. Tests pass because they use the hardcoded account, masking runtime configuration mismatches.
- **Remediation:** Replace hardcoded PalletId in `account_id()` with `T::PalletId::get()` to use the runtime-configured value. Verify the runtime Config PalletId matches the intended treasury account.

---

#### GOV-C02: `emergency_override_proposal` Does Not Execute the Action

- **Files:** `lib.rs` L~6520–6590
- **Category:** Logic Error / Emergency Powers
- **Description:** When `execute=true`, extrinsic `emergency_override_proposal` (call_index 35) sets `proposal.status = ProposalStatus::Executed` and records `executed_at`, but **never calls `execute_action()`**. The on-chain action (treasury spend, parameter change, runtime upgrade) is NOT performed — only the status label changes. This is inconsistent with `execute_emergency_proposal` (call_index 33) which correctly calls `execute_action()`.
- **Impact:** Root can mark any proposal as "Executed" during emergencies without actually performing the approved action. If relied upon for critical operations (e.g., emergency treasury disbursement), the funds never move despite the proposal showing as executed. A malicious or confused operator could "execute" a runtime upgrade proposal without actually upgrading, creating false expectations.
- **Remediation:** Add `Self::execute_action(&proposal)?;` before setting the status to Executed, or rename the status to `OverriddenApproved` to clarify no action was taken.

---

#### GOV-C03: `finalize_district_election` Resets `consecutive_terms` for Re-Elected Officials

- **Files:** `lib.rs` L~5430–5450
- **Category:** Term Limit Bypass
- **Description:** When `finalize_district_election` seats winning candidates as CitizenDelegate council members, it always creates `CouncilMember { consecutive_terms: 0, ... }`. If a sitting council member wins re-election, their `consecutive_terms` counter resets to 0 instead of incrementing. The `add_board_member` extrinsic (call_index 9) correctly increments consecutive terms for appointed members, but the election path does not.
- **Impact:** Elected officials can serve unlimited consecutive terms, completely bypassing the governance safeguard designed to prevent power consolidation. The `MaxConsecutiveTerms` check in `add_board_member` (L~4190) is rendered ineffective for anyone who enters office through elections.
- **Remediation:** Before inserting a new `CouncilMember` in `finalize_district_election`, check if the account already has a `CouncilMembers` entry and increment `consecutive_terms` accordingly. Enforce `MaxConsecutiveTerms` for elected officials as well.

---

#### GOV-C04: Single Dual-House Member Can Complete Constitutional Ratification Alone

- **Files:** `lib.rs` L~6920–6990
- **Category:** Separation of Powers
- **Description:** `ratify_constitutional_proposal` (call_index 41) checks if the caller is a member of the TechnicalCouncil and/or GovernanceCouncil via `T::DualHouseProvider`. If the caller is a member of both houses (dual-member), they can set both `technical_ratified` and `governance_ratified` to `true` in a single call, completing the entire ratification process alone.
- **Impact:** Constitutional amendments — the highest-stakes governance actions — can be unilaterally ratified by one person who holds membership in both houses. This defeats the "four-eyes" separation of powers intent. In the mock, account 2 is a dual-member and can single-handedly ratify any constitutional proposal.
- **Remediation:** Track which specific accounts (not just which houses) have ratified. Require at least one distinct signer from each house, prohibiting the same account from counting for both. Alternatively, add `ensure!(!(is_technical && is_governance), Error::<T>::DualMemberCannotRatifyBothHouses)`.

---

### WARNING

#### GOV-W01: No Deposit Required for Referendum Creation

- **Files:** `lib.rs` L~4560–4690
- **Category:** Spam / Storage Bomb
- **Description:** `create_referendum` (call_index 25) requires no deposit, unlike `submit_proposal` which requires `MinimumDeposit`. Any compliant account can create unlimited referendums at zero cost. Each referendum contains `BoundedVec` option lists and metadata stored on-chain.
- **Impact:** Storage spam vector. An attacker can fill `Referendums` storage with minimal cost.
- **Remediation:** Require a deposit for referendum creation, similar to proposals. Return deposit upon finalization.

---

#### GOV-W02: Participation Rewards Are One-Time, Not Periodic

- **Files:** `lib.rs` L~5730–5850
- **Category:** Economic Design
- **Description:** `claim_participation_reward` checks `claimed == Zero::zero()` to determine eligibility. Once claimed (any amount), the account can never claim again for that reward type. The code comment says "500 DALLA/month" for council members, but there is no time-based reset mechanism.
- **Impact:** Council members receive 500 DALLA once total, not monthly as documented. This may be intentional but contradicts comments and likely user expectations.
- **Remediation:** Either update comments to reflect one-time rewards, or implement epoch-based reward tracking with periodic resets.

---

#### GOV-W03: Vote Reward Check Only Scans First 50 Proposals

- **Files:** `lib.rs` L~5760–5790
- **Category:** Fairness
- **Description:** The vote participation reward (type 0) iterates `Proposals` storage with `.take(50)` to find if the caller has voted on any proposal. If a user's first (and only) vote is on proposal #50 or higher, the scan will never find it.
- **Impact:** Late-joining voters who voted only on high-numbered proposals are permanently ineligible for the 10 DALLA vote reward despite having participated.
- **Remediation:** Use a dedicated per-account `HasVoted` flag storage item instead of scanning proposals.

---

#### GOV-W04: `fast_track_referendum` Can Extend Voting Period

- **Files:** `lib.rs` L~6490
- **Category:** Logic Error
- **Description:** `fast_track_referendum` sets `voting_end = current_block + 10_800`. If the referendum was created with a short duration (e.g., 5,000 blocks) and fast-tracking occurs early, the new deadline could exceed the original `voting_end`, effectively _extending_ the voting period instead of shortening it.
- **Impact:** Nominally a "fast-track" (emergency shortening) tool could be used to grant more voting time. During JaguarMode, this could delay resolution of critical referendums.
- **Remediation:** Set `voting_end = min(current_block + 10_800, original_voting_end)`.

---

#### GOV-W05: `NextTreasuryProposalId` Uses Non-Saturating Increment

- **Files:** `lib.rs` L~6086
- **Category:** Arithmetic
- **Description:** `NextTreasuryProposalId::put(proposal_id + 1)` — at `u32::MAX`, this overflows (wraps to 0 in release, panics in debug).
- **Impact:** After 4 billion treasury proposals, the ID would wrap and overwrite existing proposals. Extremely unlikely in practice but violates determinism guarantees.
- **Remediation:** Use `proposal_id.saturating_add(1)` or `checked_add` with an error.

---

#### GOV-W06: `NationalTreasuryReserve` Non-Saturating Subtraction

- **Files:** `lib.rs` L~6230
- **Category:** Arithmetic
- **Description:** `execute_treasury_proposal` computes `reserves - proposal.amount` after checking `reserves >= proposal.amount`. While the sequential single-threaded execution model prevents TOCTOU, the non-saturating subtraction is a code smell.
- **Impact:** Theoretical panic if reserves were somehow modified between the check and the subtraction (impossible in current architecture, but fragile against future refactoring).
- **Remediation:** Use `reserves.saturating_sub(proposal.amount)` or `checked_sub().ok_or(Error::InsufficientTreasuryBalance)?`.

---

#### GOV-W07: No Transitive/Circular Delegation Guard

- **Files:** `lib.rs` L~5480–5600
- **Category:** Vote Manipulation
- **Description:** `delegate_vote` allows A→B delegation and B→C delegation independently. Self-delegation is blocked, but A→B→C→A circular chains are not detected. When `calculate_voting_power` counts delegators for account B, it includes A. If B then delegates to C, A's power remains with B (not transferred to C), but the documentation implies liquid democracy with transitive delegation.
- **Impact:** If transitive delegation is intended: power loss (A's delegation doesn't flow to C). If non-transitive is intended: an account that is both a delegate (receiving power) and a delegator (giving power away) creates confusing semantics — they vote with amplified power from delegators but also delegate their base vote elsewhere.
- **Remediation:** Either implement transitive delegation with cycle detection (DFS/BFS bounded by max chain length), or explicitly prevent delegates from also being delegators via `ensure!(!Delegations::<T>::contains_key(&delegate), Error::<T>::DelegateCannotRedelegate)`.

---

#### GOV-W08: 38 of 45 Extrinsics Lack Benchmarks

- **Files:** `benchmarking.rs` (all), `weights.rs` (all)
- **Category:** Weight / DoS
- **Description:** Only 7 extrinsics have FRAME benchmarks: `submit_proposal`, `cast_vote`, `finalize_proposal`, `update_community_rank`, `update_pouw_contribution`, `council_override`, `update_chain_parameter`. The remaining 38 extrinsics use hand-estimated weights from `weights.rs`. Notably missing: `execute_proposal` (dispatches arbitrary actions), `declare_emergency` (200+ storage reads for veto counting), `delegate_vote` (delegator list mutations), `finalize_district_election` (sorts all candidates), `ratify_constitutional_proposal`, all treasury operations, and commit-reveal voting.
- **Impact:** Under-weighted extrinsics allow attackers to consume more block time than they pay for, enabling weight-based DoS. Over-weighted extrinsics waste block capacity.
- **Remediation:** Run `frame-benchmarking` for all 45 extrinsics and generate proper `WeightInfo` implementations.

---

#### GOV-W09: `approve_cross_department` Has No Proposal Status Check

- **Files:** `lib.rs` L~4060–4120
- **Category:** Logic Error
- **Description:** `approve_cross_department` (call_index 8) verifies the caller is a department manager and that the proposal requires cross-approval, but does not check `proposal.status`. Approvals can be recorded on proposals that are already `Approved`, `Rejected`, `Executed`, or `Expired`.
- **Impact:** Stale approvals accumulate on finalized proposals. While this doesn't directly change outcomes, it pollutes storage and could mislead off-chain systems reading `cross_approved_by`.
- **Remediation:** Add `ensure!(proposal.status == ProposalStatus::Pending || proposal.status == ProposalStatus::Voting, Error::<T>::ProposalAlreadyFinalized)`.

---

#### GOV-W10: `nominate_for_delegate` Has No Nominee Count Limit

- **Files:** `lib.rs` L~4640–4680
- **Category:** Storage Bomb
- **Description:** `DelegateNominees` is a `StorageMap<AccountId, u32>` with no bound on the number of entries. Any compliant account can nominate any other compliant account during an active election, with no limit on total nominees.
- **Impact:** Unbounded storage growth during delegate elections. An attacker can nominate thousands of accounts, bloating `DelegateNominees` storage.
- **Remediation:** Use `CountedStorageMap` or add a `MaxDelegateNominees` bound with an error when exceeded.

---

#### GOV-W11: Abstentions Have Zero Impact on Proposal Outcomes

- **Files:** `lib.rs` L~3690
- **Category:** Governance Design
- **Description:** `finalize_proposal` computes approval as `ayes * 100 / (ayes + nays)`, completely excluding abstentions from the denominator. A single aye vote with 0 nays passes at 100% approval — even with 10,000 abstention votes.
- **Impact:** Abstentions are recorded but functionally meaningless. A proposal can pass with minimal support if no one votes nay. Quorum is checked separately (participation includes abstentions), but approval threshold only considers directional votes.
- **Remediation:** If abstentions should "count against" approval, include them in the denominator: `ayes * 100 / (ayes + nays + abstentions)`. Otherwise, document this as intentional behavior.

---

#### GOV-W12: No Deposit Slashing for Rejected/Spam Proposals

- **Files:** `lib.rs` L~3710
- **Category:** Economic Design
- **Description:** `finalize_proposal` always returns the deposit to the proposer via `Currency::unreserve`, regardless of whether the proposal passed or was rejected. There is no economic penalty for submitting proposals that fail to reach quorum or are overwhelmingly rejected.
- **Impact:** Spam deterrence relies solely on the opportunity cost of locking the deposit during the voting period. A well-funded attacker can submit and recoup deposits on unlimited spam proposals.
- **Remediation:** Slash some percentage of the deposit for proposals that fail to reach quorum or receive <10% approval. Send slashed amount to treasury.

---

#### GOV-W13: Conviction Multiplier Has No Upper Bound Check

- **Files:** `lib.rs` L~3610–3630
- **Category:** Vote Manipulation
- **Description:** `cast_vote` applies `weight * conviction` where `conviction` is a `u8` parameter (0–255). There is no clamping. A voter can supply `conviction=255` for a 255x multiplier on their vote weight, massively amplifying their influence.
- **Impact:** A single voter with conviction=255 can outweigh hundreds of conviction=1 voters. While conviction is intended to represent lockup commitment, there is no corresponding lock mechanism — it's a free amplifier.
- **Remediation:** Cap conviction to a reasonable range (e.g., 0–6 matching Polkadot's model) and implement conviction-based token lockups. Add `ensure!(conviction <= T::MaxConviction::get(), Error::<T>::InvalidConviction)`.

---

#### GOV-W14: `on_initialize` Quarterly Wealth Check Limited to 20 Accounts

- **Files:** `lib.rs` L~2930
- **Category:** Fairness / Sybil Resistance
- **Description:** The quarterly wealth-responsibility check iterates governance-active accounts with `.take(20)` to apply penalties for large holders exceeding a wealth threshold. Only the first 20 accounts in storage iteration order are checked.
- **Impact:** Large token holders can avoid penalties by: (a) creating sybil accounts to dilute each below the threshold, or (b) ensuring their account appears after position 20 in storage iteration. This creates an unfair advantage for sophisticated actors.
- **Remediation:** Process all accounts (bounded read per block using `on_idle` or chunked processing across multiple blocks), or use a sorted index to prioritize the largest holders.

---

### INFO

#### GOV-I01: `remove_board_member` Silently Truncates Reason String

- **Files:** `lib.rs` L~4310
- **Category:** Data Integrity
- **Description:** The `reason` parameter in `remove_board_member` uses `BoundedVec::try_from().unwrap_or_default()`, which silently replaces over-length reasons with an empty string instead of returning an error.
- **Remediation:** Return `Error::<T>::ReasonTooLong` if the reason exceeds the bound.

---

#### GOV-I02: `register_candidate` Has Side-Effect Status Transition

- **Files:** `lib.rs` L~5230
- **Category:** Unexpected Behavior
- **Description:** If `current_block >= election.registration_end` during a `register_candidate` call, the function transitions the election status from `Registration` to `Voting` as a side effect before rejecting the registration.
- **Remediation:** Move status transitions to `on_initialize` or a dedicated `advance_election_phase` extrinsic for predictability.

---

#### GOV-I03: Referendum Tie Resolution Biased Toward Lower Index

- **Files:** `lib.rs` L~4840
- **Category:** Fairness
- **Description:** When multiple referendum options have the same highest vote count, the first option (lowest index) wins. This creates a positional advantage for option 0.
- **Remediation:** Document this behavior. Consider randomized tie-breaking using the `Randomness` pallet, or declare a tie result requiring a re-vote.

---

#### GOV-I04: Four Emergency Action Types Are No-Ops

- **Files:** `lib.rs` L~7190–7200
- **Category:** Incomplete Implementation
- **Description:** `execute_emergency_action` handles `FreezeAccount`, `UnfreezeAccount`, `HaltGovernance`, and `ResumeGovernance` by only emitting events — no actual state changes occur. These appear to be placeholder implementations.
- **Remediation:** Implement actual freeze/unfreeze logic (integrate with `pallet_balances` freezes or a custom freeze map). Implement halt/resume via a storage guard checked by all extrinsics.

---

#### GOV-I05: QuadraticVoting Entirely Untested

- **Files:** `mock.rs` L~212, `tests.rs` (all)
- **Category:** Test Coverage
- **Description:** `QuadraticVotingEnabled` is set to `false` in the mock configuration. The `isqrt()` function and the quadratic voting branch in `cast_vote` (L~3580) are never exercised by any test.
- **Remediation:** Add a second mock configuration with `QuadraticVotingEnabled = true` and write targeted tests for the QV code path.

---

#### GOV-I06: 7 Extrinsics Have Zero Test Coverage

- **Files:** `tests.rs` (all)
- **Category:** Test Coverage
- **Description:** The following extrinsics have no tests: `veto_emergency` (call_index 44), `ratify_constitutional_proposal` (call_index 41), `lock_chain_parameter` (call_index 42), `unlock_chain_parameter` (call_index 43), `generate_exit_proof` (call_index 40), `emergency_override_proposal` with `execute=false` (rejection path), and `fast_track_referendum` deadline-extension scenario.
- **Impact:** `veto_emergency` is the core CONS-029 democratic safeguard — its untested status means the 1/3 veto threshold logic is unverified. `ratify_constitutional_proposal` is the dual-house ratification mechanism with GOV-C04 vulnerability — also unverified by tests.
- **Remediation:** Write tests for all 7 untested extrinsics, including both happy-path and error-path scenarios. Prioritize `veto_emergency` and `ratify_constitutional_proposal`.

---

#### GOV-I07: `ProposalCooldown` Disabled in Tests

- **Files:** `mock.rs` L~210
- **Category:** Test Coverage
- **Description:** `type ProposalCooldown = ConstU64<0>` disables the per-account proposal cooldown in tests. Any bugs in cooldown enforcement would be masked.
- **Remediation:** Add tests with non-zero `ProposalCooldown` to verify enforcement.

---

#### GOV-I08: Multiple Extrinsics Use Hardcoded Weights

- **Files:** `lib.rs` L~3890, L~4060, L~4640, L~4690, L~5920, L~6260, L~6600, L~6880, L~6920, L~7000, L~7030
- **Category:** Weight Accuracy
- **Description:** At least 11 extrinsics use `Weight::from_parts(X, Y)` with hardcoded constants instead of `WeightInfo` trait calls. Examples: `set_department_manager` (10M, 512), `approve_cross_department` (15M, 1024), `nominate_for_delegate` (15M, 2048).
- **Remediation:** Add all extrinsics to `WeightInfo` trait and benchmarks. Replace hardcoded weights with `T::WeightInfo::extrinsic_name()` calls.

---

## Vulnerability Category Coverage

| Category | Findings | Assessment |
|----------|----------|------------|
| **Vote Manipulation** | GOV-W07 (delegation), GOV-W11 (abstentions), GOV-W13 (conviction) | Medium risk — conviction amplifier is the most exploitable |
| **Access Control** | GOV-C04 (dual ratification) | Well-enforced for most extrinsics; constitutional ratification is the exception |
| **Timelocks** | GOV-W04 (fast-track extension) | CONS-029/030 properly applied; fast-track is the outlier |
| **Storage Bombs** | GOV-W01 (referendums), GOV-W10 (nominees) | Two unbounded growth vectors remain |
| **Arithmetic** | GOV-W05 (overflow), GOV-W06 (underflow), GOV-W13 (conviction) | No `unwrap()` in production code; saturating arithmetic mostly applied |
| **Error Handling** | GOV-I01 (silent truncation) | Generally good — `ensure!` macro used throughout |
| **Weight / DoS** | GOV-W08 (38 missing benchmarks), GOV-W14 (20-account limit), GOV-I08 (hardcoded) | Major concern — majority of extrinsics lack proper weight calibration |
| **Race Conditions** | GOV-W09 (stale approvals) | Single-threaded block execution prevents most race conditions |
| **Proxy / Delegation** | GOV-W07 (no circular guard), GOV-C03 (term bypass) | Delegation logic needs hardening |

---

## Prior Audit Remediation Verification

All previously identified findings have been confirmed as remediated:

| Finding | Status | Verification |
|---------|--------|-------------|
| P0-16 (execute_emergency_proposal no-op) | ✅ Fixed | `execute_action()` called at L~6430 |
| P0-17 (MinQuorumPercentage bypass) | ✅ Fixed | Floor enforced at L~4630 |
| P0-18 (council_override without JaguarMode) | ✅ Fixed | JaguarMode check at L~3780 |
| HIGH-4 (override bypasses enactment delay) | ✅ Fixed | Delay applied in `execute_proposal` |
| CONS-029 (emergency veto window) | ✅ Fixed | `is_pending` flag + 100-block veto window implemented |
| CONS-030 (per-type timelocks) | ✅ Fixed | `ProposalTypeTimelocks` enforced at L~3140 |
| CRIT-2 (static eligible voter count) | ✅ Fixed | Dynamic count via `T::ComplianceProvider` |
| S5-4 (treasury spend cap) | ✅ Fixed | `PeriodSpendTotal` + `MaxPeriodSpend` enforced |
| H-26 (cross-type reward claim bypass) | ✅ Fixed | Per-type tracking via `RewardType_{0,1,2}Claimed` |
| DOS-003/004/009/010/015/018 | ✅ Fixed | Bounded iterations throughout |
| G-1 (reveal window unbounded) | ✅ Fixed | `voting_end + VotingPeriod` bound at L~6760 |
| G-2 (reveal weight mismatch) | ✅ Fixed | Consistent weight calculation in `reveal_vote` |
| G-8 (amendment after voting) | ✅ Fixed | `voting_start` check at L~5680 |
| AR-8 (department action decoding) | ✅ Fixed | Safe SCALE decode with `InvalidCallData` error |
| AR-13 (commit-reveal implementation) | ✅ Fixed | `commit_vote` + `reveal_vote` at L~6650–6820 |
| AR-14 (2-step runtime upgrade) | ✅ Fixed | Hash storage + `apply_pending_runtime_upgrade` |
| M48 (delegation compliance) | ✅ Fixed | Both delegator and delegate checked |

---

## Recommended Remediation Priority

### Immediate (before mainnet)
1. **GOV-C01** — PalletId mismatch: verify runtime Config and fix `account_id()`
2. **GOV-C02** — Add `execute_action()` call to `emergency_override_proposal`
3. **GOV-C03** — Increment `consecutive_terms` in `finalize_district_election`
4. **GOV-C04** — Prevent single dual-member from completing constitutional ratification
5. **GOV-W13** — Cap conviction multiplier and implement token lockups

### High Priority (within 30 days)
6. **GOV-W08** — Benchmark all 45 extrinsics
7. **GOV-W07** — Implement circular delegation guard
8. **GOV-W01** — Add deposit requirement for referendums
9. **GOV-W10** — Bound `DelegateNominees` storage
10. **GOV-I06** — Write tests for 7 untested extrinsics

### Medium Priority (within 90 days)
11. **GOV-W04** — Fix fast-track to use `min()` for deadline
12. **GOV-W09** — Add status check to `approve_cross_department`
13. **GOV-W05/W06** — Use saturating arithmetic throughout
14. **GOV-W11** — Document or change abstention semantics
15. **GOV-W12** — Implement deposit slashing for spam proposals
16. **GOV-W14** — Expand quarterly wealth check beyond 20 accounts
17. **GOV-W02/W03** — Fix reward claim semantics and documentation

### Low Priority (maintenance)
18. **GOV-I01 through GOV-I08** — Informational items

---

## Methodology

- **Line-by-line manual review** of all 10,817 lines across 5 source files
- **9 vulnerability categories** checked: vote manipulation, access control, timelocks, storage bombs, arithmetic, error handling, weight/DoS, race conditions, proxy/delegation abuse
- **Cross-reference** with prior audit findings (CONS-029/030, PHASE_2, pallet_security_audit_2025)
- **Test coverage gap analysis** comparing 45 extrinsics against ~100 test cases
- **No automated tooling** — pure code review

---

*End of Audit Report*
