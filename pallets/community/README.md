# Community Pallet

## Overview

The **Community Pallet** implements BelizeChain's Social Responsibility Score (SRS) system, zero-fee transaction protocol, community treasury management, and citizen engagement programs. It provides a comprehensive framework for measuring and rewarding positive contributions to the BelizeChain ecosystem.

**Core Innovation**: Multi-factor scoring system (governance, education, sustainability, peer endorsements) that determines voting weight, fee exemptions, and access to community resources—creating economic incentives for productive citizenship.

### Key Features

- **Social Responsibility Score (SRS)**: 0-10,000 point system with 5 tiers (Bronze, Silver, Gold, Platinum, Diamond)
- **6-Factor Calculation**: Governance (25%), Education (15%), Sustainability (15%), Participation (25%), Endorsements (10%), Honesty (10%)
- **Zero-Fee Protocol**: Transaction fee exemptions based on SRS tier (up to 100 DALLA/month)
- **Community Treasury**: 10% of blockchain treasury allocated to citizen-driven proposals
- **SRS-Weighted Voting**: Higher SRS = more influence on community proposals (no council required)
- **Learn-to-Earn**: Rewards for completing education modules (configurable DALLA amounts)
- **Green Projects**: Sustainability contribution tracking with multipliers
- **Peer Endorsements**: Silver+ tier users can endorse others (limited to 1/month per pair)
- **Referral Rewards**: 100 DALLA for successful referrals (KYC L1+ required)
- **Privacy Controls**: Public or anonymous SRS display (hash-based)
- **Ethics Filter**: Automated screening for harmful proposals + sanctioned account list

### Business Use Cases

1. **Governance Participation**: High SRS citizens have weighted votes on community proposals
2. **Fee Optimization**: Active contributors save on transaction fees (up to 100 DALLA/month)
3. **Education Incentives**: Complete blockchain/economics courses, earn DALLA rewards
4. **Sustainability Tracking**: Solar installation, recycling programs earn green project points
5. **Community Funding**: Local projects (schools, roads, disaster relief) funded via SRS-weighted votes
6. **Reputation Building**: Tourism businesses build SRS through honesty, community service

## Architecture

### SRS Calculation Flow
```
Participation Record → Activity Type → Score Component → Weighted Sum 
→ Time Decay → Bonuses → Final SRS → Tier Assignment → Benefits Activation
```

### Components

1. **SRS Engine**: Calculates multi-factor score from participation history
2. **Participation Tracker**: Records activities from other pallets (staking, governance, etc.)
3. **Fee Calculator**: Determines transaction fee discounts based on tier
4. **Community Treasury**: Separate fund for citizen-driven proposals (10% of main treasury)
5. **Education System**: Tracks module completion and distributes rewards
6. **Green Project Registry**: Registers sustainability initiatives with verification
7. **Ethics Filter**: Screens proposals for harmful content, checks sanctioned accounts

### SRS Tiers & Benefits

| Tier | SRS Range | Fee Exemption | Voting Weight | Endorsements | Proposal Rights |
|------|-----------|---------------|---------------|--------------|-----------------|
| Bronze | 0-2,499 | 100 DALLA/month | 1x | None | 500 DALLA deposit |
| Silver | 2,500-4,999 | 100 DALLA/month | 1.5x | 1/month | 400 DALLA deposit |
| Gold | 5,000-7,499 | 100 DALLA/month | 2x | 3/month | 300 DALLA deposit |
| Platinum | 7,500-8,999 | 100 DALLA/month | 3x | 5/month | 200 DALLA deposit |
| Diamond | 9,000-10,000 | Unlimited | 5x | 10/month | 100 DALLA deposit |

**Note**: Monthly fee exemption amounts are governance-adjustable (default 100 DALLA)

## Storage

### SocialResponsibilityScores
- **Type**: `StorageMap<AccountId, SRSData>`
- **Purpose**: Master SRS records for all verified citizens
- **Data**:
  - `score: u32` - Current SRS (0-10,000)
  - `tier: SRSTier` - Bronze/Silver/Gold/Platinum/Diamond
  - `governance_score: u32` - Governance participation component (25%)
  - `education_score: u32` - Education completion component (15%)
  - `sustainability_score: u32` - Green project component (15%)
  - `participation_score: u32` - General activity component (25%)
  - `endorsement_score: u32` - Peer endorsement component (10%)
  - `honesty_score: u32` - Honesty/integrity component (10%)
  - `last_updated: BlockNumber` - Last SRS recalculation
  - `public_display: bool` - Privacy setting
  - `anonymous_hash: Option<[u8; 32]>` - Hash for anonymous display

### ParticipationHistory
- **Type**: `StorageMap<AccountId, BoundedVec<ParticipationRecord, 1000>>`
- **Purpose**: Historical activity log (last 1000 activities per account)
- **Data** (per record):
  - `activity_type: ActivityType` - Type of activity (vote, proposal, PoUW, education, etc.)
  - `block_number: BlockNumber` - When activity occurred
  - `value: u32` - Optional weighted value (e.g., PoUW quality score)

### UserProposals
- **Type**: `StorageMap<AccountId, ProposalStats>`
- **Purpose**: Track proposal submission/approval rates for honesty score
- **Data**:
  - `total: u32` - Total proposals submitted
  - `approved: u32` - Proposals that passed voting
  - `rejected: u32` - Proposals that failed

### PeerEndorsements
- **Type**: `StorageMap<AccountId, u32>`
- **Purpose**: Count of endorsements received
- **Max**: 100 endorsements count toward score (prevents gaming)

### LastEndorsement
- **Type**: `StorageMap<(AccountId, AccountId), BlockNumber>`
- **Purpose**: Prevent endorsement spam (1 endorsement per pair per month)

### CommunityProposals
- **Type**: `StorageMap<u32, CommunityProposal>`
- **Purpose**: Community-driven funding proposals
- **Data**:
  - `proposer: AccountId` - Who submitted proposal
  - `beneficiary: AccountId` - Who receives funds
  - `amount: Balance` - DALLA requested
  - `deposit: Balance` - Proposer's deposit (10% of amount)
  - `proposal_type: CommunityProposalType` - Category
  - `title: BoundedVec<u8, 128>` - Proposal title
  - `description: BoundedVec<u8, 1024>` - Proposal details
  - `status: ProposalStatus` - Active/Approved/Rejected/Executed
  - `votes_for: u32` - SRS-weighted votes for
  - `votes_against: u32` - SRS-weighted votes against
  - `total_votes: u32` - Number of unique voters
  - `voting_deadline: BlockNumber` - When voting ends (7 days)

### ProposalVotes
- **Type**: `StorageDoubleMap<u32, AccountId, Vote>`
- **Purpose**: Track individual votes on proposals
- **Data**:
  - `approve: bool` - Vote direction
  - `weight: u32` - SRS weight at time of vote

### SanctionedAccounts
- **Type**: `StorageMap<AccountId, SanctionStatus>`
- **Purpose**: Blocked accounts (ethics violations, illegal activity)
- **Data**:
  - `active: bool` - Sanction status
  - `reason: BoundedVec<u8, 128>` - Reason for sanction
  - `sanctioned_at: u32` - Block number of sanction

### FeeExemptionUsage
- **Type**: `StorageMap<AccountId, FeeExemptionData>`
- **Purpose**: Track monthly fee exemption usage
- **Data**:
  - `used_this_month: Balance` - DALLA fees waived this month
  - `last_reset_block: BlockNumber` - When monthly counter reset

### EducationModules
- **Type**: `StorageMap<u32, EducationModule>`
- **Purpose**: Available learn-to-earn courses
- **Data**:
  - `title: BoundedVec<u8, 128>` - Course title
  - `content_hash: [u8; 32]` - DAG content hash
  - `reward: Balance` - DALLA reward for completion
  - `completions: u32` - Total completions
  - `max_completions: Option<u32>` - Capacity limit

### ModuleCompletions
- **Type**: `StorageDoubleMap<AccountId, u32, CompletionData>`
- **Purpose**: Track education progress
- **Data**:
  - `completed_at: BlockNumber` - Completion timestamp
  - `score: u8` - Quiz score (0-100)
  - `reward_claimed: bool` - Whether reward paid out

### GreenProjects
- **Type**: `StorageMap<u32, GreenProject>`
- **Purpose**: Registered sustainability initiatives
- **Data**:
  - `organizer: AccountId` - Project owner
  - `title: BoundedVec<u8, 128>` - Project title
  - `project_type: GreenProjectType` - Solar/Recycling/Conservation/etc.
  - `verified: bool` - Government verification status
  - `multiplier: u8` - Sustainability score multiplier (1x-5x)
  - `participants: BoundedVec<AccountId, 1000>` - Contributors

## Extrinsics

### record_participation
**Purpose**: Record activity for SRS calculation (called by other pallets or self)

**Parameters**:
- `account: AccountId` - Account to record activity for
- `activity_code: u8` - Activity type code (see ActivityType enum)

**Activity Types**:
- `0`: ProposalSubmission (governance)
- `1`: ProposalVoting (governance)
- `2`: ProposalApproved (governance)
- `3`: PoUWContribution (staking)
- `4`: EducationCompletion (community)
- `5`: GreenProjectParticipation (community)
- `6`: PeerEndorsement (community)
- `7`: ReferralSuccess (community)

**Checks**:
- Caller is account owner or root (cross-pallet calls)
- Account is BelizeID verified (KYC L0+)

**Effects**:
- Adds `ParticipationRecord` to history
- Updates proposal stats (if applicable)
- Triggers SRS recalculation (`update_srs_internal`)
- Emits `ParticipationRecorded` event

**Returns**: `Ok(())` or Error

### update_srs
**Purpose**: Recalculate SRS from participation history (permissionless, anyone can call)

**Parameters**:
- `account: AccountId` - Account to update SRS for

**Checks**:
- Account is BelizeID verified

**Effects**:
- Calculates 6-factor weighted sum:
  - **Governance (25%)**: Votes, proposals, referendum participation
  - **Education (15%)**: Completed modules, quiz scores
  - **Sustainability (15%)**: Green project participation, verified contributions
  - **Participation (25%)**: PoUW contributions, general activity
  - **Endorsements (10%)**: Peer endorsements received (max 100 count)
  - **Honesty (10%)**: Proposal approval rate, no sanctions
- Applies time decay (older activities count less)
- Applies 6-month sustained participation bonus (+10%)
- Assigns tier based on score
- Updates `SocialResponsibilityScores` storage
- Emits `SRSUpdated` event

**Scoring Formula**:
```
governance_score = (votes + proposals * 10 + approvals * 20) / max_governance * 2500
education_score = (completions * avg_quiz_score) / max_education * 1500
sustainability_score = (green_participations * multiplier) / max_green * 1500
participation_score = (pouw_contributions * quality) / max_participation * 2500
endorsement_score = min(endorsements, 100) / 100 * 1000
honesty_score = (approved / total_proposals) * 1000 (if no sanctions)

total_srs = sum(all_scores) * time_decay_factor * sustained_bonus
```

**Returns**: `Ok(())` or Error

### endorse_peer
**Purpose**: Endorse another verified citizen (requires Silver+ tier)

**Parameters**:
- `endorsee: AccountId` - Account being endorsed
- `endorsement_code: u8` - Endorsement type code

**Endorsement Types**:
- `0`: CommunityService
- `1`: Honesty
- `2`: Leadership
- `3`: Innovation
- `4`: Mentorship

**Checks**:
- Both endorser and endorsee are BelizeID verified
- Endorser cannot endorse self
- Endorser has Silver+ tier SRS
- Last endorsement of this pair was >30 days ago (432,000 blocks)

**Effects**:
- Increments `PeerEndorsements` counter for endorsee
- Records `LastEndorsement` timestamp
- Triggers SRS update for endorsee
- Emits `PeerEndorsed` event

**Returns**: `Ok(())` or Error

### set_srs_privacy
**Purpose**: Toggle public/anonymous SRS display

**Parameters**:
- `public: bool` - Whether to make SRS publicly visible

**Checks**:
- Account is BelizeID verified

**Effects**:
- Updates `SRSData.public_display` flag
- If anonymous: Generates `blake2_256(account + score)` hash
- If public: Clears anonymous hash
- Emits `SRSPrivacyUpdated` event

**Use Case**: Sensitive businesses may want anonymous SRS to prevent discrimination

**Returns**: `Ok(())` or Error

### submit_community_proposal
**Purpose**: Submit proposal for community treasury funding

**Parameters**:
- `beneficiary: AccountId` - Who receives funds
- `amount: Balance` - DALLA requested
- `proposal_type: u8` - Proposal category code
- `title: Vec<u8>` - Proposal title (max 128 bytes)
- `description: Vec<u8>` - Proposal details (max 1024 bytes)

**Proposal Types**:
- `0`: LocalProject (schools, infrastructure)
- `1`: EducationModule (new courses)
- `2`: GreenInitiative (sustainability)
- `3`: CulturalPreservation (language, traditions)
- `4`: DisasterRelief (emergency funding)
- `5`: CommunityBounty (development tasks)

**Checks**:
- Proposer is BelizeID verified
- Proposer not sanctioned
- Title/description within size limits
- Deposit requirement:
  - Bronze: 10% of amount (500 DALLA for 5K request)
  - Silver: 8% (400 DALLA)
  - Gold: 6% (300 DALLA)
  - Platinum: 4% (200 DALLA)
  - Diamond: 2% (100 DALLA)
- Proposer has sufficient DALLA for deposit

**Effects**:
- Reserves deposit from proposer
- Creates `CommunityProposal` entry
- Sets voting deadline to +7 days (604,800 blocks)
- Emits `ProposalSubmitted` event

**Returns**: Proposal ID

### vote_on_community_proposal
**Purpose**: Cast SRS-weighted vote on proposal

**Parameters**:
- `proposal_id: u32` - Proposal to vote on
- `approve: bool` - Vote direction

**Checks**:
- Voter is BelizeID verified
- Proposal is active
- Voting period not ended
- Voter hasn't already voted on this proposal

**Effects**:
- Retrieves voter's SRS score as vote weight
- Records `Vote` in storage
- Updates proposal vote counters:
  - If approve: `votes_for += weight`
  - If reject: `votes_against += weight`
- Emits `ProposalVoted` event

**Vote Weight Example**:
```
Alice: SRS 8,000 (Platinum tier) → weight = 8,000
Bob: SRS 3,000 (Silver tier) → weight = 3,000
Charlie: SRS 1,000 (Bronze tier) → weight = 1,000

Total votes FOR: Alice + Bob = 11,000 weighted votes
Total votes AGAINST: Charlie = 1,000 weighted votes
Result: APPROVED (simple majority of weighted votes)
```

**Returns**: `Ok(())` or Error

### finalize_community_proposal
**Purpose**: Execute proposal after voting period ends (permissionless)

**Parameters**:
- `proposal_id: u32` - Proposal to finalize

**Checks**:
- Proposal is active
- Voting period has ended (current block > deadline)

**Effects**:
- Calculates approval: `votes_for > votes_against` (simple majority)
- **If APPROVED**:
  - Transfers `amount` from Community Treasury to beneficiary
  - Unreserves proposer's deposit
  - Marks proposal as `Approved`
  - Emits `ProposalExecuted`, `DepositReturned` events
- **If REJECTED**:
  - Slashes proposer's deposit (goes to Treasury)
  - Marks proposal as `Rejected`
  - Emits `ProposalRejected` event
- Emits `ProposalFinalized` event

**Returns**: `Ok(())` or Error

### sanction_account
**Purpose**: Block account from community participation (governance-only)

**Parameters**:
- `account: AccountId` - Account to sanction
- `reason: BoundedVec<u8, 128>` - Reason for sanction

**Checks**:
- Caller has `GovernanceOrigin` permission (council/root)

**Effects**:
- Creates `SanctionStatus` entry
- Sets honesty score to 0 for sanctioned account
- Blocks account from:
  - Submitting proposals
  - Voting on proposals
  - Receiving endorsements
  - Claiming education rewards
- Emits `AccountSanctioned` event

**Use Cases**:
- Spam proposals
- Fraudulent green projects
- Ethics violations
- Illegal activity

**Returns**: `Ok(())` or Error

### lift_sanction
**Purpose**: Remove sanction from account (governance-only)

**Parameters**:
- `account: AccountId` - Account to restore

**Checks**:
- Caller has `GovernanceOrigin` permission

**Effects**:
- Removes `SanctionStatus` entry
- Account can participate again
- SRS recalculation recommended
- Emits `SanctionLifted` event

**Returns**: `Ok(())` or Error

## Events

### ParticipationRecorded
```rust
ParticipationRecorded {
    account: AccountId,
    activity_type: u8,
    block_number: BlockNumber,
}
```

### SRSUpdated
```rust
SRSUpdated {
    account: AccountId,
    old_score: u32,
    new_score: u32,
    old_tier: SRSTier,
    new_tier: SRSTier,
}
```

### PeerEndorsed
```rust
PeerEndorsed {
    endorser: AccountId,
    endorsee: AccountId,
    endorsement_type: u8,
}
```

### SRSPrivacyUpdated
```rust
SRSPrivacyUpdated {
    account: AccountId,
    public_display: bool,
}
```

### ProposalSubmitted
```rust
ProposalSubmitted {
    proposal_id: u32,
    proposer: AccountId,
    amount: Balance,
    proposal_type: u8,
}
```

### ProposalVoted
```rust
ProposalVoted {
    proposal_id: u32,
    voter: AccountId,
    approve: bool,
    weight: u32,
}
```

### ProposalFinalized
```rust
ProposalFinalized {
    proposal_id: u32,
    approved: bool,
    votes_for: u32,
    votes_against: u32,
}
```

### ProposalExecuted
```rust
ProposalExecuted {
    proposal_id: u32,
    beneficiary: AccountId,
    amount: Balance,
}
```

### AccountSanctioned
```rust
AccountSanctioned {
    account: AccountId,
    reason: BoundedVec<u8, 128>,
}
```

## Errors

- `NotVerified` - Account not BelizeID verified (KYC L0+ required)
- `InsufficientSRS` - SRS too low for operation (e.g., endorsing requires Silver+)
- `CannotEndorseSelf` - Cannot endorse own account
- `EndorsementTooFrequent` - Must wait 30 days between endorsements of same pair
- `ProposalNotFound` - Proposal ID doesn't exist
- `InvalidProposalStatus` - Proposal not in expected state
- `VotingPeriodEnded` - Deadline passed
- `AlreadyVoted` - Already cast vote on this proposal
- `AccountSanctioned` - Sanctioned accounts cannot participate
- `InvalidActivityType` - Activity code not recognized
- `InvalidEndorsementType` - Endorsement code not recognized
- `ParticipationHistoryFull` - 1000 activity limit reached (oldest pruned)
- `InsufficientBalance` - Not enough DALLA for deposit

## Integration with Other Pallets

### Identity Pallet (KYC Verification)
```rust
type BelizeKyc: pallet_belize_identity::BelizeKyc<AccountId, BlockNumber>
```
- **Purpose**: Verify all participants are KYC verified
- **Method**: `is_kyc_verified(&who, level, block) -> bool`
- **Usage**: Called before all participation operations

### Economy Pallet (DALLA Payments)
```rust
type Currency: Currency<AccountId> + ReservableCurrency<AccountId>
```
- **Purpose**: Handle deposits, rewards, fee exemptions
- **Methods**: `reserve()`, `unreserve()`, `slash_reserved()`, `transfer()`
- **Usage**: Proposal deposits, education rewards, fee waivers

### Governance Pallet (Voting Weight Export)
```rust
// Exported trait implementation
impl<T: Config> CommunityRank<T::AccountId> for Pallet<T> {
    fn get_voting_weight(who: &T::AccountId) -> u32 {
        Self::get_srs(who).map(|srs| srs.score).unwrap_or(1)
    }
}
```
- **Purpose**: Provide SRS-based voting weights to governance
- **Integration**: Governance pallet queries Community for vote weight
- **Effect**: Higher SRS = more influence on national referendums

### Staking Pallet (PoUW Participation)
- **Integration**: Staking pallet calls `record_participation` for PoUW contributions
- **Activity**: `ActivityType::PoUWContribution` with quality score
- **Effect**: Increases participation score component of SRS

## Runtime Configuration

```rust
impl pallet_community::Config for Runtime {
    type WeightInfo = ();
    type Currency = Balances;
    type BelizeKyc = Identity;
    type ProposalDepositPercentage = ConstU32<10>; // 10% of request amount
    type FeeExemptionMonthlyLimit = ConstU32<100>; // 100 DALLA/month
    type EducationRewardAmount = ConstU128<50_000_000_000_000>; // 50 DALLA
    type ReferralRewardAmount = ConstU128<100_000_000_000_000>; // 100 DALLA
    type GreenProjectReward = ConstU128<200_000_000_000_000>; // 200 DALLA
    type GovernanceOrigin = EnsureRootOrHalfCouncil;
    type CommunityTreasuryAccount = CommunityTreasuryPalletId;
}
```

## Testing

### Unit Tests
```bash
cargo test -p pallet-community
```

Tests cover:
- SRS calculation (6-factor weighted sum)
- Tier assignment (Bronze-Diamond thresholds)
- Endorsement frequency limits (1/month per pair)
- Proposal voting (SRS-weighted)
- Deposit slashing (rejected proposals)
- Sanctioning effects (blocked from participation)

### Integration Tests
```bash
./run_integration_tests.sh community
```

Tests cross-pallet interactions:
- Identity → Community (KYC verification)
- Economy → Community (deposits, rewards)
- Governance → Community (voting weight export)
- Staking → Community (PoUW participation recording)

## Usage Example

### Scenario: Citizen Builds SRS and Submits Community Proposal

**Step 1: Initial Verification**
```rust
// Alice completes KYC Level 1 (BelizeID + phone)
// Identity pallet creates verified account
// Community pallet initializes default SRS (score: 0, tier: Bronze)
```

**Step 2: Participate in Governance**
```rust
// Alice votes on national referendum
Governance::vote(referendum_id, Vote::Aye);

// Governance pallet calls Community::record_participation
record_participation(
    account: alice,
    activity_code: 1  // ProposalVoting
);
// Effect: SRS increases to ~500 (governance component +500)
```

**Step 3: Complete Education Module**
```rust
// Alice completes "Blockchain Basics" course
complete_education_module(
    origin: alice,
    module_id: 1,
    quiz_score: 95
);

// Effect: 
// - SRS increases to ~1,200 (education component +700)
// - Alice receives 50 DALLA reward
```

**Step 4: Join Green Project**
```rust
// Alice participates in beach cleanup (verified by government)
participate_in_green_project(
    origin: alice,
    project_id: 5  // "Placencia Beach Cleanup"
);

// Effect: SRS increases to ~2,000 (sustainability component +800)
// Tier: Bronze → Silver ✅
```

**Step 5: Receive Peer Endorsement**
```rust
// Bob (Gold tier) endorses Alice for community service
endorse_peer(
    origin: bob,
    endorsee: alice,
    endorsement_code: 0  // CommunityService
);

// Effect: SRS increases to ~2,200 (endorsement component +200)
```

**Step 6: Submit Community Proposal**
```rust
// Alice wants to build solar panels for local school
submit_community_proposal(
    origin: alice,
    beneficiary: school_account,
    amount: 50_000 * 10^12,  // 50K DALLA (~$25K USD)
    proposal_type: 2,  // GreenInitiative
    title: "Solar Panels for Placencia Elementary".as_bytes().to_vec(),
    description: "Install 100kW solar system to reduce school electricity costs by 80%...".as_bytes().to_vec()
);

// Deposit: 50K * 8% = 4K DALLA (Silver tier discount)
// Voting deadline: Block 604,800 (7 days)
// Emits: ProposalSubmitted { proposal_id: 42, ... }
```

**Step 7: Community Votes** (7-day voting period)
```rust
// High-SRS voters support proposal
vote_on_community_proposal(origin: carlos, proposal_id: 42, approve: true);
// Carlos: SRS 9,500 (Diamond) → +9,500 votes FOR

vote_on_community_proposal(origin: diana, proposal_id: 42, approve: true);
// Diana: SRS 6,000 (Gold) → +6,000 votes FOR

vote_on_community_proposal(origin: eric, proposal_id: 42, approve: false);
// Eric: SRS 1,500 (Bronze) → +1,500 votes AGAINST

// Final tally: 15,500 FOR vs 1,500 AGAINST = APPROVED ✅
```

**Step 8: Proposal Execution**
```rust
// After 7 days, anyone finalizes proposal
finalize_community_proposal(
    origin: anyone,
    proposal_id: 42
);

// Effects:
// - 50K DALLA transferred from Community Treasury → school_account
// - Alice's 4K deposit unreserved (returned)
// - Proposal marked as Executed
// - Alice's SRS increases to ~2,500 (approved proposal bonus)
// - School builds solar panels, saves 80% on electricity
```

**Alice's SRS Journey**:
```
Initial: 0 (Bronze)
After governance: 500 (Bronze)
After education: 1,200 (Bronze)
After green project: 2,000 (Bronze)
After endorsement: 2,200 (Silver) ✅
After proposal approval: 2,500 (Silver)
```

**Benefits Earned**:
- Fee exemptions: 100 DALLA/month (saves ~5 DALLA/month on transactions)
- Voting weight: 1.5x (Silver tier)
- Endorsement rights: Can endorse 1 peer/month
- Lower proposal deposits: 8% instead of 10%
- Education rewards: 50 DALLA
- Community respect: Reputation as engaged citizen

## Future Enhancements

### Phase 2: Zero-Fee Protocol (PLANNED)
- **Feature**: Automatic fee waivers for verified citizens based on tier
- **Status**: Storage structures defined, integration with Economy pallet pending
- **Benefit**: Active citizens save on transaction costs
- **Timeline**: Q2 2026

### Phase 3: Advanced Analytics
- **Feature**: SRS leaderboards, district rankings, time-series analysis
- **Status**: Planned
- **Benefit**: Gamification, competition between districts
- **Timeline**: Q3 2026

### Phase 4: Cross-Chain Reputation
- **Feature**: Export SRS to Ethereum/Polkadot for global reputation
- **Status**: Research phase
- **Benefit**: Belize citizens have verifiable reputation on other chains
- **Timeline**: Q1 2027

### Phase 5: AI Ethics Filter
- **Feature**: Machine learning model to detect harmful proposals automatically
- **Status**: Planned (requires Nawal integration)
- **Benefit**: Reduce governance burden, faster screening
- **Timeline**: Q2 2027

### Phase 6: NFT Achievements
- **Feature**: Mint NFTs for SRS milestones (e.g., "Diamond Tier Citizen")
- **Status**: Planned
- **Benefit**: Tradeable reputation tokens, bragging rights
- **Timeline**: Q3 2027

## References

- [Social Credit Systems](https://en.wikipedia.org/wiki/Social_Credit_System) - Inspiration (Belize version is voluntary, privacy-respecting)
- [DAO Governance](https://arxiv.org/abs/2201.06752) - Decentralized decision-making
- [Quadratic Voting](https://vitalik.ca/general/2019/12/07/quadratic.html) - Alternative voting weight model
- [Governance Pallet](../governance/README.md) - National referendum system
- [Identity Pallet](../identity/README.md) - KYC verification system
- [Staking Pallet](../staking/README.md) - PoUW participation recording

## License
This pallet is part of BelizeChain and is licensed under GPL-3.0.
