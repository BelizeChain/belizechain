# BelizeChain Ethical Safeguards — Full Implementation Plan

**Created:** 2026-03-03  
**Status:** Planning — Phase 0 not yet started  
**Scope:** All 13 identified ethical safeguard gaps + 4 additional items (addiction loops, wellbeing funding, anti-dehumanization, voting power cap).  
**Constraint:** Zero breaking changes to existing consensus or state transition logic. Every phase ends with a mandatory verification pass before proceeding.

---

## Current Gap Summary

| Your Measure | Status Before This Plan | Key Gap |
|---|---|---|
| Progressive reward / diminishing returns | Partial | SRS rank capped at 4× but raw DALLA stake has no ceiling — whales can dominate votes with money alone |
| Wealth → responsibility | Absent | No coded pathway where large holders are required to take on governance roles or lose influence |
| Multi-house governance | Partial | The architecture is designed (Council / Departments / Community) but all houses resolve to one Root key |
| Constitution / unamendable rights | Docs only | `ProposalType::Constitutional` just sets a 75% threshold — Root bypasses it entirely; no hard invariants exist |
| Term limits / rotation | Partial | Term data tracked, `TermExpiryQueue` exists, but nothing auto-expires anyone (no `on_initialize` hook); Founder/FSC/BTB have no limits at all |
| Anti-sybil / proof of personhood | Partial | KYC L1–L3 framework exists, but compliance↔identity sync is a stub; a compromised oracle can override all identity gates |
| Rate-limiting / cooling-off on governance | Partial | Voting periods and deposits enforced but no per-account proposal cooldown, no enactment delay after a vote passes |
| Restorative justice | Absent | Only punitive slashing exists. No cooling-off, no rehabilitation period, no graduated response |
| Whistleblower incentive | Absent | One doc mention. No extrinsic, no reward logic, no anonymity protection anywhere in code |
| Fork / exit as a right | Absent | Only forkless upgrades (opposite of exit rights). No on-chain mechanism for minority fork or exit-with-assets |
| Reputation tied to contribution | Partial | SRS tracks governance participation, education, etc. but it is partly self-reportable — you can inflate your own governance weight |
| Epistemic infrastructure / oracle plurality | Partial | Oracle pallet exists but it is a single oracle that can override on-chain KYC — no cross-checking, no contest mechanism |
| Cultural scaffolding / anti-dehumanization | Absent in code | No content moderation mechanism exists |
| No addiction loops | Absent | No token-level mechanic prevents casino-style reward patterns |
| Wellbeing / despair support primitives | Absent | No on-chain funding mandate for mutual aid or community support |
| Hard caps on voting power | Absent | Quadratic voting mentioned in 3 places in docs but zero code |

---

## Phase Ordering — STRICT

```
Phase 0  →  Phase 1  →  Phase 2  →  Phase 3  →  Phase 4  →  Phase 5  →  Phase 6
```

**Phase 0 is a hard blocker for all other phases.**  
If you skip Phase 0, every other safeguard built in Phases 1–6 can be overridden by a single Root private key (`pallet_sudo`). The elaborate multi-house governance, constitutional types, term limits, and compliance gates are currently all bypassable by one key.

---

## Phase 0 — Break the Single Root Key

**The problem:** Every origin in `runtime/src/lib.rs` — `CouncilOrigin`, `CommunityOrigin`, `OracleAdminOrigin`, `ComplianceOrigin`, 15+ total — resolves to `EnsureRoot<AccountId>`. One private key overwrites your constitution, bypasses term limits, and ignores all other safeguards.

**Files touched:** `Cargo.toml`, `runtime/Cargo.toml`, `runtime/src/lib.rs`

### Step 0.1 — Add `pallet_collective` as a dependency

`pallet-collective` is already in the Polkadot SDK (BelizeChain is Substrate-based). Add it to both root `Cargo.toml` and `runtime/Cargo.toml`. This is a dependency add only — no logic changes yet.

### Step 0.2 — Define three Collective instances in the runtime

In `runtime/src/lib.rs`, instantiate three collectives using Substrate's instance system:

```
TechnicalCouncil   → max 12 members, voting threshold: >50% standard, >66% technical
GovernanceCouncil  → max 32 members (maps to existing CouncilMember storage), threshold: >51%
CommunityHouse     → max unbounded (maps to verified identity holders), threshold: >50%
```

Each collective gets its own instance via `pallet_collective::Instance1/2/3`.

### Step 0.3 — Map each origin to the appropriate collective

Replace every `EnsureRoot<AccountId>` in `runtime/src/lib.rs`:

| Current Origin | Replace With | Rationale |
|---|---|---|
| `CouncilOrigin` | `EnsureProportionMoreThan<TechnicalCouncil, 1, 2>` | Majority of technical council |
| `CommunityOrigin` | `EnsureProportionMoreThan<GovernanceCouncil, 1, 2>` | Majority of governance council |
| `OracleAdminOrigin` | `EnsureMember<TechnicalCouncil>` | Any technical council member |
| `ComplianceOrigin` | `EnsureProportionMoreThan<TechnicalCouncil, 2, 3>` | Supermajority for compliance changes |
| `SanctionsOrigin` | `EnsureProportionMoreThan<GovernanceCouncil, 2, 3>` | Supermajority for sanctions |
| `AIAuthorityOrigin` | `EnsureMember<TechnicalCouncil>` | Any technical council member |
| `EmergencyOrigin` | `EnsureProportionAtLeast<TechnicalCouncil, 3, 4>` | 75% of tech council for emergencies |
| `GovernmentOrigin` | `EnsureProportionMoreThan<GovernanceCouncil, 1, 2>` | Majority governance |
| Constitutional change | `EnsureProportionAtLeast<TechnicalCouncil, 3, 4>` AND `EnsureProportionAtLeast<GovernanceCouncil, 3, 5>` | Multi-house AND requirement |

### Step 0.4 — Sudo transition plan (scheduled removal, not immediate deletion)

Do **not** remove `pallet_sudo` immediately — that risks locking out the chain if a bug surfaces before governance is proven stable.

Instead:
- Add `SudoScheduledRemovalBlock: StorageValue<BlockNumber>` in the governance pallet
- Set it to `mainnet_launch_block + 50,400` (3.5 days of buffer after launch)
- In governance `on_initialize`: at that block, emit `SudoRemovalExecuted` event; from that point governance pallet refuses to execute any proposal that came from Sudo origin
- The actual `pallet_sudo` removal from `construct_runtime!` happens via a runtime upgrade after the removal block passes and is confirmed safe

### Phase 0 Verification Checklist

- [ ] `cargo test -p runtime` — all existing runtime tests pass
- [ ] `cargo check --all-features` — full compile with new dependency
- [ ] Manual: submit a test governance proposal through collective origin and confirm it routes correctly
- [ ] Grep check: no `EnsureRoot` remains outside `pallet_sudo` itself after this phase
- [ ] Invariant: `pallet_sudo` is still in `construct_runtime!` (removal is scheduled, not immediate)

---

## Phase 1 — Governance Structural Fixes

Five independent sub-tasks within Phase 1 — all in `pallets/governance/src/lib.rs`.

### 1A — Wire TermExpiryQueue to `on_initialize`

The `TermExpiryQueue` storage already exists and is populated at member appointment. The data is there — nothing fires it.

**What to add:**

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
        let mut weight = Weight::zero();
        // Drain expired members at this block
        if let Some(expired_accounts) = TermExpiryQueue::<T>::take(n) {
            for account in expired_accounts.iter() {
                CouncilMembers::<T>::remove(account);
                Self::deposit_event(Event::CouncilTermExpired { account: account.clone() });
                weight = weight.saturating_add(T::WeightInfo::expire_council_member());
            }
        }
        // Also drain ExecutionQueue (see 1E)
        weight
    }
}
```

Budget guard: max 10 expirations per block to prevent weight exhaustion. If more than 10 members expire on the same block (mass election), emit `TermExpiryOverflow` event and spill remainder to the next block via a priority queue.

### 1B — Add consecutive term cap

Add `consecutive_terms: u8` field to the `CouncilMember` struct.  
Add config constant `MaxConsecutiveTerms: Get<u8>` (default: 2).

In `appoint_council_member`: when re-appointing a rotating member, check `consecutive_terms < T::MaxConsecutiveTerms::get()`. If at cap, they must sit out one full term before re-appointment. Reset `consecutive_terms = 0` after mandatory break.

**Migration note:** This is additive to the struct, so existing encoded members in storage need a `StorageVersion` migration (`pre_upgrade` / `post_upgrade` hooks). Field defaults to `0` for existing members — safe.

### 1C — Term limits for all permanent roles

Change `BoardRole::is_rotating()` to return `true` for **all** roles.

- Permanent roles get a default term of 5 years (5 × `BLOCKS_PER_YEAR`)
- Re-extension requires a `CouncilOrigin` governance vote — it does not happen automatically
- Founder role specifically: re-extension requires Community house vote at 60% threshold in addition to council vote
- Add `MaxPermanentTermExtensions: Get<u8>` config constant (default: 3) — permanent role can be extended at most 3 times for a total of 20 years maximum

### 1D — Per-account proposal cooldown

**New storage:**
- `AccountLastProposal<T: Config>: StorageMap<AccountId, BlockNumber, ValueQuery>`

**New config constants:**
- `ProposalCooldown: Get<BlockNumber>` (default: 43,200 blocks = 3 days)
- `EmergencyProposalCooldown: Get<BlockNumber>` (default: 7,200 blocks = 12 hours, with 5× deposit requirement)

**In `create_proposal`:**
```rust
ensure!(
    current_block.saturating_sub(AccountLastProposal::<T>::get(&who)) >= T::ProposalCooldown::get()
    || proposal_type == ProposalType::Emergency,
    Error::<T>::ProposalCooldownActive
);
```
Update `AccountLastProposal` on successful submission.

### 1E — Enactment delay

**New storage:**
- `ExecutionQueue<T: Config>: StorageMap<BlockNumber, BoundedVec<u32, ConstU32<50>>, ValueQuery>` — keyed by execute-at block number

**Enactment delay by proposal type:**
- `Constitutional` → 201,600 blocks (14 days)
- `Economic` → 100,800 blocks (7 days)
- `Emergency` → 0 (immediate)
- All others → 50,400 blocks (3.5 days)

**When a proposal passes vote count:** instead of executing inline, compute `execute_at = current_block + delay`, insert `proposal_id` into `ExecutionQueue[execute_at]`. In `on_initialize` (from 1A), drain the execution queue at the current block.

**Veto window:** During the enactment window, any `CouncilOrigin` veto vote with >33% council opposition can cancel execution before it fires.

### Phase 1 Verification Checklist

- [ ] `cargo test -p pallet-belize-governance` passes including new tests for: auto-expiry, consecutive term cap, cooldown enforcement, enactment delay
- [ ] Regression: all existing proposal/vote/elect tests still pass
- [ ] Invariant: no member with `consecutive_terms > MaxConsecutiveTerms` exists in `CouncilMembers` storage after test
- [ ] Invariant: `ExecutionQueue` is always empty after `on_initialize` for that block
- [ ] Invariant: `TermExpiryOverflow` event fires when > 10 expirations queued on same block

---

## Phase 2 — Economic Fairness

### 2A — DALLA voting weight cap + quadratic voting formula

**File:** `pallets/governance/src/lib.rs`

**New config constants:**
- `QuadraticVotingEnabled: Get<bool>` (default: `true`)
- `MaxVotingUnits: Get<u32>` (default: `10_000`) — hard ceiling on any account's vote weight regardless of stake
- `StakeUnitSize: Get<Balance>` — how many DALLA tokens = 1 vote unit (e.g., 100 DALLA = 1 unit)

**Integer square root (deterministic, no floats — consensus-safe):**

```rust
fn isqrt(n: u128) -> u128 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
```

**In vote weight calculation:**

```rust
let stake_units = (balance / T::StakeUnitSize::get()).saturated_into::<u128>();
let raw_weight = if T::QuadraticVotingEnabled::get() {
    isqrt(stake_units)
} else {
    stake_units
};
let stake_weight = raw_weight.min(T::MaxVotingUnits::get() as u128) as u32;
let total_weight = stake_weight.saturating_add(srs_weight); // srs already capped at 4x
```

> **Consensus safety:** This changes existing vote weight calculation. Must be introduced via a governance proposal + enactment delay, not a hot-patch, so all nodes upgrade simultaneously at the same block.

### 2B — Wealth → responsibility: large holder participation requirement

**Files:** `pallets/governance/src/lib.rs`, `pallets/economy/src/lib.rs`

**New config constants:**
- `LargeHolderStakeThreshold: Get<Balance>` (e.g., 50,000 DALLA)
- `LargeHolderMinParticipationRate: Get<u8>` (e.g., 30 — meaning 30% of proposals they are eligible to vote on)

**New storage:**
- `AccountVoteParticipation<T>: StorageMap<AccountId, (u32, u32)>` — `(eligible, voted)` — updated on each vote opportunity
- `EffectiveVotingMultiplier<T>: StorageMap<AccountId, u8>` — default 100 (no penalty); set to 50 if participation falls below required rate

**Quarterly check** (every ~1,296,000 blocks): in `on_initialize`, compute participation rate for large holders; if below threshold, set `EffectiveVotingMultiplier` to 50. Resets when participation recovers.

**KYC requirement:** Large holders who have never registered a governance identity get `0` voting weight until they complete at minimum L1 KYC.

### 2C — Progressive reward routing above accumulation thresholds

**File:** `pallets/economy/src/lib.rs`

**New config constants:**
- `NormalRewardThreshold: Get<Balance>` (e.g., 50,000 DALLA in staking rewards accumulated)
- `ReducedRewardThreshold: Get<Balance>` (e.g., 200,000 DALLA)
- `PublicGoodsRoutingPercent: Get<u8>` (e.g., 30 — 30% of rewards above the reduced threshold go to PG treasury)

**New storage:**
- `PublicGoodsTreasury<T>: StorageValue<Balance>` — accumulator

At reward distribution time: rewards above `ReducedRewardThreshold` have `PublicGoodsRoutingPercent`% routed to `PublicGoodsTreasury` instead of paid to the account. This does not affect the validator's staking bond — only the inflationary reward component.

### Phase 2 Verification Checklist

- [ ] `cargo test -p pallet-belize-economy`
- [ ] `cargo test -p pallet-belize-governance`
- [ ] Invariant: `isqrt` function passes fuzz test with u128 edge cases (0, 1, MAX_U128)
- [ ] Invariant: no account's vote weight exceeds `MaxVotingUnits + srs_cap` after Phase 2
- [ ] Regression: all existing staking reward tests pass with correct totals (public goods routing is additive — total distributed does not change, only destination changes)

---

## Phase 3 — Identity & Trust

### 3A — Oracle plurality (multi-oracle consensus, break single-oracle override)

**File:** `pallets/oracle/src/lib.rs`

Current problem: single oracle can override KYC. Nawal has a `submit_nawal_telemetry` path already.

**New storage:**
- `RegisteredOracles<T>: StorageMap<OracleId, OracleMetadata>` — governance-curated list of trusted oracle providers
- `OracleDataSubmissions<T>: StorageDoubleMap<DataKey, OracleId, (DataValue, BlockNumber)>` — each oracle's submission per key
- `ConsensusDataValues<T>: StorageMap<DataKey, DataValue>` — finalized only when `MinOracleAgreement` met
- `OracleDisputeFlag<T>: StorageMap<DataKey, bool>` — set true when oracles disagree, triggers governance review

**New config constant:**
- `MinOracleAgreement: Get<u32>` (default: 2 out of N registered oracles must agree within 1 block window)

**Rules:**
- For KYC-affecting data: any single oracle submission is **staged** (not applied) until agreement threshold is met
- If oracles disagree: `OracleDisputeFlag` set true → governance review triggered
- Nawal AI is registered as oracle provider ID 1 (`NawalOracleId`) — still provides assessments but no longer has unilateral override

### 3B — SRS self-reporting prevention

**File:** `pallets/community/src/lib.rs`

`record_participation` is currently any-signed — you can call it on yourself.

**New config:**
- `ActivityAttestationRequired: BoundedVec<u8, ConstU32<16>>` — list of activity type codes that require external attestation
- `AttestationWindow: Get<BlockNumber>` (default: 3 days)

**New storage:**
- `PendingAttestations<T>: StorageDoubleMap<AccountId, (ActivityType, BlockNumber), u32>` — attestation vote count

**New process for attested activities:**
1. `record_participation` from self creates a pending attestation request (not applied yet)
2. A second call from a **different** account with SRS rank ≥ Silver calls `attest_participation(account, activity_type, block)`
3. When 2 independent attestations are gathered within the window: participation is applied to SRS scores

**Activities requiring attestation:** education completions, community mediation, governance moderation, conflict resolution.  
**Activities NOT requiring attestation:** regular on-chain governance voting (auto-recorded from governance pallet vote events — on-chain and verifiable).

### Phase 3 Verification Checklist

- [ ] `cargo test -p pallet-belize-oracle`
- [ ] `cargo test -p pallet-belize-community`
- [ ] Invariant: no single oracle submission can finalize KYC-sensitive data without `MinOracleAgreement` — test with mock oracles
- [ ] Regression: `submit_nawal_telemetry` still works; existing oracle data tests pass
- [ ] Invariant: self-attesting a high-weight activity does NOT change SRS score — requires second independent attestor in tests

---

## Phase 4 — Justice & Support

Three items, two new pallets plus an extension to the governance pallet.

### 4A — Restorative Justice Pallet

**New pallet:** `pallets/justice/` → `pallet_belize_justice`

**Core concept:** Before a punitive slash becomes permanent or before a ban is issued, the justice pallet intervenes and requires at least a cooling-off period + mediation attempt for first offenses.

**New storage:**

```rust
Disputes<T>: StorageMap<DisputeId, DisputeRecord<AccountId, BlockNumber, Balance>>
CoolingOffEnd<T>: StorageMap<AccountId, BlockNumber>
RehabilitationStatus<T>: StorageMap<AccountId, RehabStatus>
// enum: Clean | InCoolingOff | InRehabilitation | Reinstated
SlashPendingJusticeReview<T>: StorageMap<AccountId, Balance>  // held slash amount
MediatorList<T>: StorageValue<BoundedVec<AccountId, ConstU32<20>>>  // governance-appointed
```

**New extrinsics:**
- `open_dispute(target, evidence_hash, severity)` — any signed, requires bond proportional to severity
- `mediator_ruling(dispute_id, resolution)` — mediator-signed only
- `appeal_ruling(dispute_id, counter_evidence_hash)` — by the account under dispute
- `complete_rehabilitation(account)` — governance or mediator, after cooling-off ends AND no new offenses

**Integration with staking pallet:** In `report_validator_offense` (currently Root-only), add a check: if `SlashPendingJusticeReview::<T>::contains_key(&target)` AND `consecutive_offenses <= 1` → route to justice pallet first (cooling-off path) instead of immediate slash. Slash amount is escrowed in `SlashPendingJusticeReview`. If justice pallet confirms wrongdoing: execute slash. If mediated: partial or zero slash.

**Slash decay in `on_initialize`:** Every `SlashDecayPeriod` blocks (configurable, default 90 days) of no new offense: `SlashingSpans` count decrements by 1 (down to 0). Represents cleared record after demonstrated good behavior.

### 4B — Whistleblower Pallet

**New pallet:** `pallets/whistleblower/` → `pallet_belize_whistleblower`

**Pseudonymity mechanism:** Reporter submits `alias_hash = blake2_256(account_bytes ++ nonce)`. Actual account is not stored on-chain. To claim reward later, they reveal `(account, nonce)` and the chain recomputes the hash to verify — identity revealed only at reward claim, never stored.

**New storage:**

```rust
Reports<T>: StorageMap<ReportId, Report<T>>
WhistleblowerPool<T>: StorageValue<Balance>  // funded by treasury mandate
EscrowedReward<T>: StorageMap<ReportId, Balance>
ReportStatus<T>: StorageMap<ReportId, ReportStatus>
// enum: Pending | UnderReview | Verified | Dismissed
```

**New extrinsics:**
- `submit_report(alias_hash, target, evidence_hash, category)` — any signed (alias protects identity); requires small `ReportBond` to prevent spam
- `review_report(report_id, verdict, reasoning_hash)` — council reviewer only
- `claim_reward(report_id, account, nonce)` — reveal identity to claim from escrow after Verified verdict
- `fund_whistleblower_pool(amount)` — governance-callable

**Treasury mandate:** Add `WhistleblowerFundingPercent: Get<u8>` config in economy pallet (default: 0.5% of block rewards). Automatic routing.

**Reward escrow levels (set by governance):**
- Fraud: 2,000 DALLA
- Systematic abuse: 5,000 DALLA
- Chain-threatening exploit: 20,000 DALLA

### 4C — Fork / Exit Right

**File:** `pallets/governance/src/lib.rs`

**Add `ProposalType::MinorityExitPetition` with rules:**
- Requires co-signatures from 15% of `total_governance_eligible_accounts()` to be considered valid
- Triggers a mandatory governance response within 60 days (emits `ExitPetitionResponseRequired` event)
- If governance does not respond or refuses: escalates to a full network referendum on chain direction

**Add `generate_exit_proof()` extrinsic — the real enforcer:** Any account can call this at any time. It produces a chain-signed, attested proof of their current:
- DALLA balance
- Staking positions
- BNS names
- Locked/vested amounts

This proof can be used in a fork's genesis configuration to import the account state. The exit proof is valid for 30 days (stamped with `valid_until: BlockNumber`). No permission required — you can always generate your own proof. This makes the exit right real: you don't need anyone's permission to leave, you just need your proof.

### Phase 4 Verification Checklist

- [ ] `cargo test -p pallet-belize-justice`
- [ ] `cargo test -p pallet-belize-whistleblower`
- [ ] Update `runtime/Cargo.toml` and `runtime/src/lib.rs` to include both new pallets
- [ ] `cargo test -p runtime` — full runtime integration
- [ ] Invariant: a first-offense slash routes through justice pallet and does NOT immediately execute — is escrowed
- [ ] Invariant: whistleblower report with wrong nonce in `claim_reward` returns error — hash verification works
- [ ] Invariant: `generate_exit_proof()` produces deterministic, verifiable output across nodes
- [ ] Regression: existing staking slash tests for **repeat offenders** still pass (justice pallet only intercepts first offense)

---

## Phase 5 — Nawal-Integrated Ethical Enforcement

### 5A — Anti-addiction loop enforcement

**Integration point:** Oracle pallet's Nawal telemetry path (`submit_nawal_telemetry` already exists).

**New oracle data type:** `OracleDataType::BehaviorPattern` with subtypes:
- `CompulsiveActivity` — >50 governance actions in a 24h period
- `RewardLoopDetected` — Nawal detects variable-reward exploitation pattern
- `AutomatedActingPattern` — bot-like behavior distinct from expected human variance

**New storage:**
- `BehaviorFlags<T>: StorageMap<AccountId, BoundedVec<BehaviorFlag, ConstU32<10>>>` — populated by Nawal oracle consensus
- `BehaviorFlagCooldown<T>: StorageMap<AccountId, BlockNumber>` — flag-triggered cooldown end block

**Integration in governance and economy pallets:** If `BehaviorFlags::<T>::contains_key(&who)` and flag is active → add `T::BehaviorCooldownPeriod` (default 24h) to vote/proposal cooldown for that account. This is a circuit breaker, not a ban.

**Lottery / variable-reward prohibition rule:** Add to `execute_proposal` validation in governance pallet: if the proposal's `action` field encodes any reward mechanic matching `ProposalActionFlags::VariableRewardLottery` → rejected via `ConstitutionalGuard` (Phase 6). Governance can whitelist specific approved variable-reward structures but they must be explicitly governance-ratified.

> **Safety note:** Behavior flags from Nawal do NOT apply unless multi-oracle consensus threshold is met (from Phase 3). This prevents Nawal being used to suppress accounts unilaterally.

### 5B — Public goods treasury mandate + wellbeing funding

**Files:** `pallets/economy/src/lib.rs`, `pallets/governance/src/lib.rs`

**New config constants in economy pallet:**
- `PublicGoodsMandatePercent: Get<u32>` (default: 10 — 10% of all block reward inflation goes to PG treasury)
- `WellbeingFundPercent: Get<u32>` (default: 3 — 3% of inflation specifically to wellbeing sub-treasury)

**New storage:**
- `PublicGoodsTreasury<T>: StorageValue<Balance>`
- `WellbeingTreasury<T>: StorageValue<Balance>`

**New governance additions:**
- `ProposalType::WellbeingFunding` — lower threshold (51%), shorter voting period (3 days), lower minimum deposit
- `MutualAidRequests<T>: StorageMap<RequestId, AidRequest<AccountId, Balance>>` — on-chain funding requests from accounts or community groups
- Nawal AI (via oracle) provides a triage assessment score for each request: weighted, non-binding input into the vote

### 5C — Content moderation mechanism (anti-dehumanization)

**New pallet:** `pallets/moderation/` → `pallet_belize_moderation`

**Scope:** On-chain content flagging, review, and enforcement. Content includes: BNS names, on-chain descriptions, oracle-submitted community data, governance proposal text.

**New storage:**

```rust
ContentFlags<T>: StorageDoubleMap<ContentHash, AccountId, FlagReason>
FlagCounts<T>: StorageMap<ContentHash, u32>
ModerationQueue<T>: StorageValue<BoundedVec<ContentHash, ConstU32<200>>>
ModeratorSet<T>: StorageValue<BoundedVec<AccountId, ConstU32<50>>>
NawalAssessments<T>: StorageMap<ContentHash, NawalModerationScore>  // 0-100 severity
RuledContent<T>: StorageMap<ContentHash, ModerationRuling>  // Cleared | Removed | Escalated
ActivePolicy<T>: StorageValue<PolicyHash>  // governance-updateable
DehumanizationPatterns<T>: StorageValue<BoundedVec<PatternHash, ConstU32<500>>>
```

**Moderation process:**
1. Any account with SRS rank ≥ Bronze can `flag_content(content_hash, reason_category)`
2. When `FlagCounts` exceeds `AutoReviewThreshold` (default: 5) OR Nawal oracle submits severity score > 50: content enters `ModerationQueue`
3. Nawal oracle scores all queued content via oracle pallet integration (0–100 severity)
4. Score routing:
   - 0–49: Dismissed (logged)
   - 50–79: Human moderator review required — moderator set votes by simple majority
   - 80–100: Immediate `ContentSuspended` flag + priority moderator review required within 48h
5. `rule_on_content(hash, ruling)` — moderators vote, simple majority executes ruling
6. `ContentRemoved` event is subscribed to by: BNS pallet, oracle pallet, governance pallet (for proposal title/description hash)

**Dehumanization pattern list:** `DehumanizationPatterns` is a list of known dehumanizing phrase patterns (hashed). Nawal AI maintains and proposes updates to this list via oracle submissions, which are then ratified via governance.

### Phase 5 Verification Checklist

- [ ] `cargo test -p pallet-belize-moderation`
- [ ] `cargo test -p pallet-belize-oracle` — verify Nawal telemetry path still works with multi-oracle changes from Phase 3
- [ ] Invariant: behavior flag from Nawal does NOT apply unless multi-oracle consensus threshold is met
- [ ] Invariant: wellbeing treasury receives correct percentage of block rewards — test with mock block reward distribution
- [ ] Regression: BNS pallet, governance pallet, oracle pallet integration tests all pass with moderation pallet in runtime

---

## Phase 6 — Constitutional Hard Invariants

### 6A — Encode unamendable rights as Rust `ensure!` guards

**File:** `pallets/governance/src/lib.rs`

Create a `ConstitutionalGuard` module within the governance pallet. This is called in `execute_proposal` **BEFORE** the origin check — meaning even a Root call cannot bypass it:

```rust
fn constitutional_guard_check<T: Config>(proposal: &Proposal<...>) -> DispatchResult {
    // 1. No punishment without transparent process
    if let Some(ProposalAction::BanAccount { .. }) = &proposal.action {
        ensure!(proposal.disputed_by.is_some(), Error::<T>::ConstitutionalViolation);
    }
    // 2. Constitutional voting period cannot be reduced below minimum
    if proposal.proposal_type == ProposalType::Constitutional {
        ensure!(
            proposal.voting_end.saturating_sub(proposal.voting_start)
                >= T::MinConstitutionalVotingPeriod::get(),
            Error::<T>::ConstitutionalViolation
        );
    }
    // 3. Fork/exit mechanism cannot be removed
    if let Some(ProposalAction::RemovePalletFeature { feature }) = &proposal.action {
        ensure!(
            *feature != PalletFeature::MinorityExitPetition,
            Error::<T>::ConstitutionalViolation
        );
    }
    // 4. No single account can be assigned vote weight above cap
    if let Some(ProposalAction::SetVotingWeight { weight, .. }) = &proposal.action {
        ensure!(*weight <= T::MaxVotingUnits::get(), Error::<T>::ConstitutionalViolation);
    }
    // 5. MinimumDeposit cannot be set to zero
    if let Some(ProposalAction::UpdateConfig { key, value }) = &proposal.action {
        if key == b"MinimumDeposit" {
            ensure!(*value > 0, Error::<T>::ConstitutionalViolation);
        }
    }
    // 6. Variable reward lotteries are prohibited
    if let Some(ProposalAction::EnableRewardMechanic { mechanic_type }) = &proposal.action {
        ensure!(
            *mechanic_type != RewardMechanicType::VariableLottery,
            Error::<T>::ConstitutionalViolation
        );
    }
    Ok(())
}
```

These are compiled guarantees. They cannot be overridden by any origin at runtime.

### 6B — Multi-house requirement for constitutional changes

**File:** `pallets/governance/src/lib.rs`

`ProposalType::Constitutional` now requires explicit approval from **both**:
- `TechnicalCouncil` collective (75% threshold)
- `GovernanceCouncil` collective (66% threshold)

Implementation:
- Add `const_approval_technical: bool` and `const_approval_governance: bool` fields to the `Proposal` struct
- Add extrinsics `approve_as_technical_council` and `approve_as_governance_council` — callable only by respective collective origins
- `execute_proposal` for `Constitutional` type: requires both flags true AND enactment delay passed

No single house can ratify a constitutional change alone.

### Phase 6 Verification Checklist

- [ ] `cargo test -p pallet-belize-governance` — all guard tests pass
- [ ] Write tests for EACH constitutional guard check — attempting to bypass each one returns `ConstitutionalViolation`
- [ ] Invariant: even with Root origin, `constitutional_guard_check` returns error for each prohibited action
- [ ] Invariant: constitutional proposal without dual-house approval cannot execute regardless of vote count or origin

---

## Full Cross-Cutting Regression (Run After All Phases Complete)

| Check | Command | What It Validates |
|---|---|---|
| Unit tests | `cargo test --workspace` | No individual pallet regressions |
| Compile check | `cargo check --all-features` | No type errors introduced |
| Clippy | `cargo clippy --all-targets -- -D warnings` | No new warnings |
| Integration (Python) | `pytest tests/ -v` | Python e2e tests still pass |
| Weight exhaustion | `cargo test -p pallet-belize-governance -- on_initialize_weight` | `on_initialize` doesn't overrun block weight budget |
| Storage migration | `cargo test -- storage_migration` | Struct additions (consecutive_terms, etc.) migrate cleanly |
| Oracle plurality | `cargo test -p pallet-belize-oracle -- multi_oracle` | KYC data requires consensus; single oracle rejected |
| Voting math | `cargo test -- quadratic_voting -- --nocapture` | sqrt results match reference table, no overflow |
| Constitutional guards | `cargo test -- constitutional_guard` | Each prohibited action returns `ConstitutionalViolation` |

---

## Summary: Full Fix Map

| Gap | Phase | Primary File(s) |
|---|---|---|
| All origins = Sudo | 0 | `runtime/src/lib.rs` |
| Multi-house governance | 0 | `runtime/src/lib.rs` + `pallet_collective` dep |
| Term limits (auto-expiry hook) | 1A | `pallets/governance/src/lib.rs` |
| Term limits (consecutive cap) | 1B | `pallets/governance/src/lib.rs` |
| Term limits (permanent roles) | 1C | `pallets/governance/src/lib.rs` |
| Rate-limiting / proposal cooldown | 1D | `pallets/governance/src/lib.rs` |
| Rate-limiting / enactment delay | 1E | `pallets/governance/src/lib.rs` |
| Voting power cap + quadratic voting | 2A | `pallets/governance/src/lib.rs` |
| Wealth → responsibility | 2B | `pallets/governance/src/lib.rs` |
| Progressive reward curves | 2C | `pallets/economy/src/lib.rs` |
| Oracle plurality (break single-oracle override) | 3A | `pallets/oracle/src/lib.rs` |
| SRS self-reporting prevention | 3B | `pallets/community/src/lib.rs` |
| Restorative justice | 4A | New: `pallets/justice/` |
| Whistleblower incentive | 4B | New: `pallets/whistleblower/` |
| Fork / exit right | 4C | `pallets/governance/src/lib.rs` |
| Anti-addiction loops (Nawal behavior flags) | 5A | `pallets/oracle/src/lib.rs` + `pallets/governance/src/lib.rs` |
| Wellbeing / mutual aid funding | 5B | `pallets/economy/src/lib.rs` + `pallets/governance/src/lib.rs` |
| Anti-dehumanization moderation pallet | 5C | New: `pallets/moderation/` |
| Constitution as hard invariants | 6A | `pallets/governance/src/lib.rs` |
| Multi-house constitutional change | 6B | `pallets/governance/src/lib.rs` |

---

## Progress Tracker

| Phase | Status |
|---|---|
| Phase 0 — Break the Single Root Key | ✅ Complete |
| Phase 1 — Governance Structural Fixes | ✅ Complete |
| Phase 2 — Economic Fairness | ⬜ Not started |
| Phase 3 — Identity & Trust | ⬜ Not started |
| Phase 4 — Justice & Support | ⬜ Not started |
| Phase 5 — Nawal-Integrated Ethical Enforcement | ⬜ Not started |
| Phase 6 — Constitutional Hard Invariants | ⬜ Not started |
| Full Regression Sweep | ⬜ Not started |

Update this table as work progresses. Each phase must be fully verified before the next begins.
