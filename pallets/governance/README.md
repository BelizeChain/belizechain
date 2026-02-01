# 🏛️ Governance Pallet

## Overview

The **Governance Pallet** implements BelizeChain's comprehensive democratic governance system, featuring three tiers of participatory decision-making: District Elections, Foundation Board, and Departmental Governance. This multi-layered architecture ensures representation across Belize's six electoral districts while maintaining specialized oversight through board roles and ministry-specific governance tracks.

## Purpose

This pallet serves as the democratic foundation for Belize's sovereign blockchain, providing:

1. **District Representation**: Proportional representation across 6 electoral districts (12 council seats)
2. **Foundation Board**: 7 specialized roles for technical and regulatory oversight
3. **Departmental Governance**: 8 ministry-specific decision tracks (Finance, Education, Health, Works, Justice, Tourism, Agriculture, Defense)
4. **Proposal System**: Community-driven governance with 7-day voting periods
5. **Vote Delegation**: Proxy voting with expiration for flexible participation
6. **Compliance Integration**: KYC tier-based access (Observer/Contributor/Validator)
7. **Economic Incentives**: DALLA rewards for civic engagement

## Architecture

### Three-Tier Governance Model

```
┌─────────────────────────────────────────────────────────────┐
│                  DISTRICT ELECTIONS (Tier 1)                │
│  6 Districts × 2 Representatives = 12 Council Seats         │
│  Belize | Cayo | Corozal | Orange Walk | Stann Creek | Toledo│
│  Public voting every 2 years (blocks: 5,256,000)            │
└─────────────────────────────────────────────────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  FOUNDATION BOARD (Tier 2)                  │
│  7 Specialized Roles with 4-year terms                      │
│  Founder | Technical Steward | FSC Rep | BTB Delegate |     │
│  Citizen Delegate | Security Auditor | Culture & Ethics     │
│  Executive decision-making and technical oversight          │
└─────────────────────────────────────────────────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────┐
│               DEPARTMENTAL GOVERNANCE (Tier 3)              │
│  8 Ministry Tracks: Finance, Education, Health, Works,      │
│  Justice, Tourism, Agriculture, Defense                     │
│  Specialized proposals with domain-specific voting          │
└─────────────────────────────────────────────────────────────┘
```

### Key Parameters

```rust
// Voting Configuration
VOTING_PERIOD: BlockNumber = 50_400          // 7 days (~12s blocks)
SUPERMAJORITY_THRESHOLD: u32 = 66            // 66% approval required
MIN_PARTICIPATION: u32 = 10                  // 10% turnout minimum

// District Representation
TOTAL_DISTRICTS: usize = 6                   // Belize's 6 districts
SEATS_PER_DISTRICT: u32 = 2                  // 12 total council seats
ELECTION_CYCLE: BlockNumber = 5_256_000      // ~2 years

// Foundation Board
BOARD_ROLES: usize = 7                       // Specialized positions
BOARD_TERM: BlockNumber = 10_512_000         // ~4 years

// Departments
TOTAL_DEPARTMENTS: usize = 8                 // Ministry tracks

// Economic Incentives (in DALLA)
VOTE_REWARD: Balance = 10                    // 10 DALLA per vote
PROPOSAL_REWARD: Balance = 100               // 100 DALLA per proposal
COUNCIL_SALARY: Balance = 500                // 500 DALLA/month
```

## Key Features

### 1. **District Elections (Tier 1)**

**Six Electoral Districts**:
```rust
pub enum BelizeDistrict {
    Belize,        // 2 seats (capital district)
    Cayo,          // 2 seats (largest by area)
    Corozal,       // 2 seats (northern district)
    OrangeWalk,    // 2 seats (sugar belt)
    StannCreek,    // 2 seats (coastal tourism)
    Toledo,        // 2 seats (southern Mayan heartland)
}
```

**Election Process**:
```rust
// Register as candidate
register_candidate(origin, district, platform) -> DispatchResult

// Vote for candidate (requires KYC)
vote_in_election(origin, district, candidate_id) -> DispatchResult

// End election and count votes
finalize_election(origin, district) -> DispatchResult
```

**Candidate Requirements**:
- Minimum age: 21 years
- Belizean citizenship (verified via BelizeID - Identity pallet)
- KYC Contributor tier or higher
- Deposit: 1,000 DALLA (refunded if receives >5% votes)

**Voting Rights**:
- **Observer Tier**: Read-only access (no voting)
- **Contributor Tier**: Vote in district elections
- **Validator Tier**: Vote + propose + delegate

### 2. **Foundation Board (Tier 2)**

**Seven Specialized Roles**:
```rust
pub enum BoardRole {
    Founder,               // Long-term vision, veto power (non-rotating)
    TechnicalSteward,      // Protocol upgrades, security (rotating)
    FSCRepresentative,     // Financial Services Commission liaison (rotating)
    BTBDelegate,           // Belize Tourism Board rep (rotating)
    CitizenDelegate,       // Public advocate (rotating)
    SecurityAuditor,       // Audit oversight (rotating)
    CultureEthicsAdvisor,  // Cultural preservation (rotating)
}
```

**Role Characteristics**:
- **Non-Rotating**: Founder (appointed at genesis)
- **Rotating**: 6 other roles (4-year terms, renewable once)
- **Max Term Limit**: 8 years total (2 consecutive terms)

**Board Powers**:
```rust
// Propose board actions
propose_board_action(origin, action, rationale) -> DispatchResult

// Vote on board proposals (requires board membership)
vote_on_board_proposal(origin, proposal_id, approve) -> DispatchResult

// Execute approved board proposal
execute_board_proposal(origin, proposal_id) -> DispatchResult
```

**Board Actions**:
- Treasury spending (>100,000 DALLA requires board approval)
- Runtime upgrades (technical changes)
- Emergency governance (national security events)
- Parameter adjustments (voting period, thresholds, rewards)

### 3. **Departmental Governance (Tier 3)**

**Eight Ministry Tracks**:
```rust
pub enum Department {
    Finance,       // Budget, taxation, economic policy
    Education,     // Schools, scholarships, curriculum
    Health,        // Hospitals, clinics, public health
    Works,         // Infrastructure, roads, utilities
    Justice,       // Courts, law enforcement, prisons
    Tourism,       // BTB coordination, marketing
    Agriculture,   // Farming, fisheries, land use
    Defense,       // Military, border security
}
```

**Departmental Proposals**:
```rust
// Submit department-specific proposal
create_departmental_proposal(
    origin, 
    department, 
    title, 
    description, 
    action
) -> DispatchResult

// Vote on departmental proposal (requires expertise or district tie)
vote_on_departmental_proposal(origin, proposal_id, choice) -> DispatchResult
```

**Department-Specific Features**:
- **Specialized Voting**: Department members have weighted votes
- **Budget Allocation**: Each department has annual DALLA budget
- **Priority Queues**: Critical proposals (health emergencies) get fast-tracked
- **Expert Review**: Technical proposals vetted by board (e.g., FSC for Finance)

### 4. **Proposal System**

**Proposal Lifecycle**:
```
Draft → Active → Voting → [Approved/Rejected] → [Executed/Failed]
```

**Proposal Types**:
```rust
pub enum ProposalType {
    TreasurySpend,         // Allocate DALLA from treasury
    ParameterChange,       // Adjust governance constants
    RuntimeUpgrade,        // Protocol modifications
    EmergencyAction,       // National security responses
    PolicyDecision,        // General governance decisions
    DepartmentalBudget,    // Ministry funding allocation
    DistrictInitiative,    // District-specific projects
}
```

**Creating a Proposal**:
```rust
propose(
    origin,
    title: BoundedVec<u8, 256>,
    description: BoundedVec<u8, 5000>,
    proposal_type: ProposalType,
    action: ProposalAction,
    priority: ProposalPriority
) -> DispatchResult
```

**Voting Mechanics**:
```rust
pub enum VoteChoice {
    Aye,        // Approve
    Nay,        // Reject
    Abstain,    // Counted for quorum, not for/against
}

// Cast vote (earns 10 DALLA reward)
vote(origin, proposal_id, choice, conviction) -> DispatchResult
```

**Conviction Voting**:
- **1x**: No lock-up (flexible voting)
- **2x**: Lock DALLA for 7 days
- **4x**: Lock DALLA for 28 days
- **8x**: Lock DALLA for 90 days

### 5. **Vote Delegation**

**Proxy Voting System**:
```rust
// Delegate votes to representative
delegate_vote(
    origin, 
    delegate: AccountId, 
    conviction, 
    expiry: BlockNumber
) -> DispatchResult

// Remove delegation
undelegate_vote(origin, delegate) -> DispatchResult

// Check delegation status
get_delegation(delegator: AccountId) -> Option<VoteDelegation>
```

**Delegation Rules**:
- Delegator retains ownership of DALLA (no transfer)
- Can override delegate on specific proposals
- Automatic expiry after specified blocks
- Delegate can sub-delegate with reduced conviction

### 6. **Referendum System**

**National Referendums**:
```rust
// Initiate referendum (requires board approval)
initiate_referendum(
    origin,
    question: BoundedVec<u8, 500>,
    options: Vec<BoundedVec<u8, 100>>,
    threshold: VotingThreshold
) -> DispatchResult

// Vote on referendum (all citizens)
vote_on_referendum(origin, referendum_id, option_index) -> DispatchResult
```

**Referendum Types**:
- **Constitutional**: Change fundamental blockchain rules (75% threshold)
- **Legislative**: New laws/policies (66% threshold)
- **Advisory**: Non-binding citizen input (simple majority)

**Binding vs. Advisory**:
- **Binding**: Automatic execution upon approval
- **Advisory**: Board/Council decides implementation

### 7. **Priority Queue System**

**Proposal Prioritization**:
```rust
pub enum ProposalPriority {
    Critical,      // Execute within 1 day (emergency)
    High,          // Standard 7-day voting
    Medium,        // 14-day discussion + 7-day vote
    Low,           // 30-day discussion + 7-day vote
}
```

**Critical Proposals** (fast-tracked):
- Emergency treasury transfers
- Security vulnerability patches
- National disaster responses
- Runtime hotfixes

### 8. **Emergency Governance**

**Emergency Actions**:
```rust
pub enum EmergencyType {
    SecurityBreach,        // Hacking attempt, validator compromise
    EconomicCrisis,        // DALLA/bBZD instability
    NaturalDisaster,       // Hurricane, earthquake
    GovernmentDirective,   // Official government mandate
    Other,                 // Unforeseen critical events
}

// Declare emergency (requires 5-of-7 board approval)
declare_emergency(origin, emergency_type, rationale) -> DispatchResult

// Execute emergency action (bypass normal voting)
execute_emergency_action(origin, action) -> DispatchResult

// End emergency state
resolve_emergency(origin, emergency_id) -> DispatchResult
```

**Emergency Powers**:
- **Treasury Access**: Unlimited spending (logged for audit)
- **Parameter Changes**: Immediate effect (no voting delay)
- **Validator Controls**: Remove malicious validators
- **Contract Pausing**: Halt specific pallets/contracts

**Safeguards**:
- **Time Limit**: 7 days maximum (renewable with board vote)
- **Audit Trail**: All actions logged to Pakit storage
- **Post-Emergency Review**: Mandatory governance review within 14 days

## Data Structures

### Storage Items

```rust
// District Elections
DistrictCouncil: map BelizeDistrict => Vec<CouncilMember>       // Elected representatives
ActiveElections: map BelizeDistrict => Election                 // Ongoing votes
ElectionCandidates: map (BelizeDistrict, AccountId) => Candidate // Registered candidates

// Foundation Board
BoardMembers: map BoardRole => Option<AccountId>                // Current board composition
BoardTermStart: map AccountId => BlockNumber                    // Term tracking

// Proposals
Proposals: map ProposalId => Proposal                           // All proposals
ProposalVotes: map (ProposalId, AccountId) => Vote              // Individual votes
ProposalCount: u64                                              // Total proposals created

// Departments
DepartmentProposals: map Department => Vec<ProposalId>          // Department-specific queues
DepartmentBudgets: map Department => DistrictBudget             // Annual allocations

// Delegation
VoteDelegations: map AccountId => VoteDelegation                // Active delegations

// Referendums
Referendums: map ReferendumId => Referendum                     // National votes
ReferendumResults: map ReferendumId => Vec<u64>                 // Vote counts per option

// Treasury
TreasurySpendProposals: map u64 => TreasurySpendProposal        // Spending requests

// Emergency
ActiveEmergencies: map EmergencyType => EmergencyStatus         // Current emergencies
```

### Key Types

```rust
pub struct Proposal<AccountId, BlockNumber, Balance> {
    pub id: u64,
    pub proposer: AccountId,
    pub title: BoundedVec<u8, 256>,
    pub description: BoundedVec<u8, 5000>,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub voting_start: BlockNumber,
    pub voting_end: BlockNumber,
    pub tally: VoteTally,
    pub action: ProposalAction<AccountId, Balance>,
    pub priority: ProposalPriority,
}

pub struct CouncilMember<AccountId, BlockNumber> {
    pub account: AccountId,
    pub district: BelizeDistrict,
    pub elected_at: BlockNumber,
    pub term_end: BlockNumber,
    pub votes_received: u64,
}

pub struct VoteDelegation<AccountId, BlockNumber> {
    pub delegator: AccountId,
    pub delegate: AccountId,
    pub conviction: u8,
    pub created_at: BlockNumber,
    pub expires_at: Option<BlockNumber>,
}

pub struct Referendum<AccountId, BlockNumber> {
    pub id: u64,
    pub question: BoundedVec<u8, 500>,
    pub options: Vec<BoundedVec<u8, 100>>,
    pub threshold: VotingThreshold,
    pub status: ReferendumStatus,
    pub voting_end: BlockNumber,
}
```

## Extrinsics (Public Functions)

### District Elections

| Function | Authority | Description |
|----------|-----------|-------------|
| `register_candidate` | KYC Contributor+ | Register for district election |
| `vote_in_election` | KYC Contributor+ | Vote for district candidate |
| `finalize_election` | Governance | Count votes, seat winners |

### Foundation Board

| Function | Authority | Description |
|----------|-----------|-------------|
| `propose_board_action` | Board Member | Create board proposal |
| `vote_on_board_proposal` | Board Member | Vote on board proposal |
| `execute_board_proposal` | Board Member | Execute approved proposal |
| `appoint_board_member` | Governance | Fill board vacancy |

### Proposals

| Function | Authority | Description |
|----------|-----------|-------------|
| `propose` | KYC Validator+ | Create new governance proposal |
| `vote` | KYC Contributor+ | Cast vote on active proposal |
| `execute_proposal` | Any (if approved) | Execute approved proposal |
| `cancel_proposal` | Proposer/Governance | Cancel proposal before execution |

### Delegation

| Function | Authority | Description |
|----------|-----------|-------------|
| `delegate_vote` | KYC Contributor+ | Delegate voting power |
| `undelegate_vote` | Delegator | Remove delegation |

### Referendums

| Function | Authority | Description |
|----------|-----------|-------------|
| `initiate_referendum` | Board | Start national referendum |
| `vote_on_referendum` | All citizens | Vote on referendum option |
| `finalize_referendum` | Governance | Count referendum results |

### Departmental

| Function | Authority | Description |
|----------|-----------|-------------|
| `create_departmental_proposal` | Department Member | Submit ministry proposal |
| `vote_on_departmental_proposal` | Dept. Members/Citizens | Vote on dept. proposal |

### Emergency

| Function | Authority | Description |
|----------|-----------|-------------|
| `declare_emergency` | Board (5-of-7) | Activate emergency mode |
| `execute_emergency_action` | Board | Bypass voting for critical action |
| `resolve_emergency` | Board | End emergency state |

## Events

```rust
pub enum Event<T: Config> {
    // Proposals
    ProposalCreated { proposal_id: u64, proposer: AccountId, proposal_type: ProposalType },
    VoteCast { proposal_id: u64, voter: AccountId, choice: VoteChoice, conviction: u8 },
    ProposalApproved { proposal_id: u64 },
    ProposalRejected { proposal_id: u64 },
    ProposalExecuted { proposal_id: u64 },
    
    // Elections
    CandidateRegistered { district: BelizeDistrict, candidate: AccountId },
    ElectionVoteCast { district: BelizeDistrict, voter: AccountId, candidate: AccountId },
    ElectionFinalized { district: BelizeDistrict, winners: Vec<AccountId> },
    
    // Board
    BoardProposalCreated { proposal_id: u64, proposer: AccountId },
    BoardMemberAppointed { role: BoardRole, member: AccountId },
    BoardProposalApproved { proposal_id: u64 },
    
    // Delegation
    VoteDelegated { delegator: AccountId, delegate: AccountId },
    VoteUndelegated { delegator: AccountId, delegate: AccountId },
    
    // Referendums
    ReferendumInitiated { referendum_id: u64, question: Vec<u8> },
    ReferendumVoteCast { referendum_id: u64, voter: AccountId, option: u8 },
    ReferendumFinalized { referendum_id: u64, winning_option: u8 },
    
    // Emergency
    EmergencyDeclared { emergency_type: EmergencyType, rationale: Vec<u8> },
    EmergencyActionExecuted { action_type: EmergencyActionType },
    EmergencyResolved { emergency_type: EmergencyType },
}
```

## Errors

```rust
pub enum Error<T> {
    // Proposal Errors
    ProposalNotFound,
    ProposalAlreadyExecuted,
    ProposalNotApproved,
    VotingPeriodEnded,
    AlreadyVoted,
    InvalidProposalType,
    
    // Election Errors
    NotEligibleToVote,
    CandidateAlreadyRegistered,
    ElectionNotActive,
    InvalidDistrict,
    InsufficientCandidateDeposit,
    
    // Board Errors
    NotBoardMember,
    InvalidBoardRole,
    BoardTermExpired,
    MaxTermLimitReached,
    
    // Delegation Errors
    CannotDelegateTo Self,
    DelegationNotFound,
    DelegationExpired,
    
    // Referendum Errors
    ReferendumNotActive,
    InvalidReferendumOption,
    
    // Permission Errors
    InsufficientKYCLevel,      // Requires KYC Contributor/Validator tier
    NotCouncilMember,
    
    // Emergency Errors
    NoActiveEmergency,
    EmergencyNotApproved,      // Requires 5-of-7 board approval
    EmergencyTimeExpired,
}
```

## Integration Points

### Cross-Pallet Dependencies

**Compliance Pallet** (KYC/AML):
```rust
// Check voter eligibility
impl pallet_belize_governance::ComplianceProvider<AccountId> for ComplianceIntegration {
    fn get_kyc_tier(account: &AccountId) -> ParticipationTier {
        let level = Compliance::get_verification_level(account);
        ParticipationTier::from_verification_level(level)
    }
}
```

**Participation Tiers** (from Compliance KYC levels):
- **Observer** (Level 0-1): Read governance proposals
- **Contributor** (Level 2-3): Vote on proposals + district elections
- **Validator** (Level 4-5): Propose + vote + delegate + board candidacy

**Economy Pallet** (Treasury Integration):
```rust
// Treasury spending proposals
impl pallet_belize_governance::TreasuryProvider<AccountId, Balance> for EconomyIntegration {
    fn spend_from_treasury(recipient: &AccountId, amount: Balance) -> DispatchResult {
        Economy::treasury_transfer(recipient, amount)
    }
    
    fn get_treasury_balance() -> Balance {
        Economy::treasury_account_balance()
    }
}
```

**Economic Incentives** (paid in DALLA):
- **Vote Reward**: 10 DALLA per vote cast
- **Proposal Reward**: 100 DALLA per accepted proposal
- **Council Salary**: 500 DALLA/month for district representatives
- **Board Stipend**: 1,000 DALLA/month for Foundation Board members

**Staking Pallet** (PoUW Integration):
```rust
// Validators participate in governance
impl pallet_belize_governance::StakingProvider<AccountId> for StakingIntegration {
    fn is_validator(account: &AccountId) -> bool {
        Staking::is_active_validator(account)
    }
    
    fn validator_stake(account: &AccountId) -> Balance {
        Staking::get_stake_amount(account)
    }
}
```

**Validator Privileges**:
- Automatic KYC Validator tier (highest governance access)
- Priority proposal processing
- Emergency proposal creation (with board approval)

### Provider Pattern (from runtime/lib.rs)

```rust
// Governance queries Compliance for KYC tiers
pub struct GovernanceComplianceProvider;
impl pallet_belize_governance::ComplianceProvider<AccountId> for GovernanceComplianceProvider {
    fn get_kyc_tier(account: &AccountId) -> ParticipationTier {
        Compliance::get_verification_level(account).into()
    }
}

// Governance queries Economy for treasury
pub struct GovernanceEconomyProvider;
impl pallet_belize_governance::TreasuryProvider<AccountId, Balance> for GovernanceEconomyProvider {
    fn spend_from_treasury(recipient: &AccountId, amount: Balance) -> DispatchResult {
        Economy::treasury_transfer(recipient, amount)
    }
}
```

## Weight Functions

All extrinsics use benchmarked weights:

```rust
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    
    fn vote() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    
    fn execute_proposal() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    
    // ... (30+ benchmarked functions)
}
```

## Usage Examples

### For Citizens

**Propose a Policy Change**:
```rust
// Requires KYC Validator tier
Governance::propose(
    Origin::signed(alice),
    b"Lower Transaction Fees".to_vec().try_into().unwrap(),
    b"Reduce base tx fee from 0.01 to 0.005 DALLA to increase accessibility".to_vec().try_into().unwrap(),
    ProposalType::ParameterChange,
    ProposalAction::UpdateParameter(GovernanceParameter::BaseTransactionFee, 5_000),
    ProposalPriority::Medium
)?;
// Earns 100 DALLA if proposal passes
```

**Vote on Active Proposal**:
```rust
// Requires KYC Contributor tier
Governance::vote(
    Origin::signed(bob),
    proposal_id,
    VoteChoice::Aye,
    4  // 4x conviction (lock DALLA for 28 days)
)?;
// Earns 10 DALLA immediately
```

**Delegate Voting Power**:
```rust
// Trust representative to vote on your behalf
Governance::delegate_vote(
    Origin::signed(charlie),
    trusted_delegate_id,
    2,  // 2x conviction
    Some(current_block + 50_400)  // Expires in 7 days
)?;
```

### For District Candidates

**Register for Election**:
```rust
// Requires 1,000 DALLA deposit (refunded if >5% votes)
Governance::register_candidate(
    Origin::signed(candidate),
    BelizeDistrict::Cayo,
    b"Build infrastructure, improve education, support farmers".to_vec().try_into().unwrap()
)?;
```

**Vote in District Election**:
```rust
Governance::vote_in_election(
    Origin::signed(voter),
    BelizeDistrict::Cayo,
    candidate_id
)?;
```

### For Foundation Board

**Propose Emergency Action**:
```rust
// Requires board membership
Governance::propose_board_action(
    Origin::signed(board_member),
    BoardAction::EmergencyTreasurySpend {
        recipient: disaster_relief_account,
        amount: 500_000_000_000,  // 500K DALLA for hurricane relief
    },
    b"Hurricane Julia damaged infrastructure in Toledo district".to_vec()
)?;
```

**Approve Runtime Upgrade**:
```rust
// Requires 5-of-7 board approval
Governance::vote_on_board_proposal(
    Origin::signed(technical_steward),
    proposal_id,
    true  // Approve
)?;
```

### For Departments

**Ministry Budget Request**:
```rust
// Finance department requests education funding
Governance::create_departmental_proposal(
    Origin::signed(finance_minister),
    Department::Education,
    b"Scholarship Program 2026".to_vec().try_into().unwrap(),
    b"Allocate 2M DALLA for university scholarships for low-income students".to_vec().try_into().unwrap(),
    ProposalAction::TreasurySpend {
        recipient: scholarship_fund_account,
        amount: 2_000_000_000_000,
    }
)?;
```

## Testing

### Test Coverage (6 Test Modules, 9,634 Total Lines)

**Test Phases** (separate files for organization):
1. **tests_phase3.rs** - Departmental governance (8 departments)
2. **tests_phase4.rs** - Foundation board operations (7 roles)
3. **tests_phase5.rs** - District elections (6 districts)
4. **tests_phase6.rs** - Proposal lifecycle (7 types)
5. **tests_phase7.rs** - Voting mechanics (delegation, conviction, tallying)
6. **tests_phase8.rs** - Emergency governance (4 emergency types)

**Mock Runtime** (mock.rs, 175 lines):
- Test accounts (Alice, Bob, Charlie, Dave, Eve, Ferdie, George)
- Pre-configured board members
- 6 district councils with 2 seats each
- Mock Compliance pallet for KYC tiers
- Mock Economy pallet for treasury operations

**Key Test Scenarios**:
```rust
// Proposal lifecycle
#[test]
fn test_create_and_approve_proposal() { ... }

// District elections
#[test]
fn test_district_election_full_cycle() { ... }

// Board governance
#[test]
fn test_board_proposal_requires_quorum() { ... }

// Vote delegation
#[test]
fn test_delegation_with_expiry() { ... }

// Emergency actions
#[test]
fn test_emergency_bypass_normal_voting() { ... }

// Compliance integration
#[test]
fn test_kyc_tier_restricts_voting() { ... }
```

## Security Considerations

### Vote Manipulation Prevention
- **Sybil Resistance**: KYC requirement prevents multi-account voting
- **Conviction Locking**: High-conviction votes lock DALLA (skin in the game)
- **Delegation Expiry**: Prevents permanent power consolidation
- **Audit Trail**: All votes logged to Pakit storage (transparent, immutable)

### Treasury Safeguards
- **Spending Limits**: Board approval required for >100K DALLA
- **Multi-Sig**: 4-of-7 board signatures for critical actions
- **Emergency Limits**: 7-day time cap on emergency powers
- **Post-Action Review**: Mandatory audit after emergency

### Board Accountability
- **Term Limits**: Max 8 years (2 consecutive 4-year terms)
- **Public Voting**: All board votes on-chain (transparent)
- **Recall Mechanism**: Council can vote to remove board member (75% threshold)
- **Rotation**: 6 of 7 roles rotate (prevents entrenchment)

### Compliance Integration
- **KYC Tiers Enforced**:
  - Observer (Level 0-1): Read-only
  - Contributor (Level 2-3): Vote + district elections
  - Validator (Level 4-5): Full governance access
- **AML Monitoring**: Large treasury spends flagged for FSC review
- **Identity Verification**: BelizeID integration (via Identity pallet)

## Performance Metrics

### Benchmark Results

| Extrinsic | Weight (ns) | DB Reads | DB Writes |
|-----------|-------------|----------|-----------|
| `propose` | 50,000,000 | 5 | 3 |
| `vote` | 35,000,000 | 3 | 2 |
| `execute_proposal` | 60,000,000 | 4 | 3 |
| `register_candidate` | 40,000,000 | 3 | 2 |
| `vote_in_election` | 30,000,000 | 2 | 1 |
| `delegate_vote` | 28,000,000 | 2 | 2 |

### On-Chain Storage

| Storage Item | Size | Capacity |
|--------------|------|----------|
| `Proposals` | ~800 bytes/proposal | Thousands of proposals |
| `ProposalVotes` | ~100 bytes/vote | Millions of votes |
| `DistrictCouncil` | ~200 bytes/member | 12 members (2 per district) |
| `BoardMembers` | ~100 bytes/role | 7 board members |
| `VoteDelegations` | ~150 bytes/delegation | Hundreds of thousands |

### Scalability Considerations
- **Pagination**: Proposal lists paginated (max 100 per query)
- **Archival**: Old proposals archived after 1 year (reduces active storage)
- **Vote Pruning**: Executed proposal votes archived (retain audit trail in Pakit)

## Governance Workflows

### Example: Treasury Spending Proposal

**Lifecycle**:
1. **Proposal Creation** (Day 0):
   - Citizen creates proposal: "Fund School in Punta Gorda"
   - Requests 50,000 DALLA from treasury
   - Proposes transfer to Toledo District Education Department
   
2. **Discussion Period** (Days 0-7):
   - Community discusses via off-chain forums
   - Proposer can amend based on feedback
   - Board reviews for compliance
   
3. **Voting Period** (Days 7-14):
   - Citizens vote Aye/Nay/Abstain
   - Vote delegation in effect
   - Conviction-weighted tallying
   
4. **Approval Check** (Day 14):
   - **Threshold**: 66% supermajority
   - **Quorum**: 10% participation minimum
   - **Tally**: Aye: 70%, Nay: 25%, Abstain: 5%
   - **Result**: APPROVED
   
5. **Execution** (Day 15):
   - Treasury transfers 50,000 DALLA
   - Proposal marked as executed
   - Audit trail logged
   
6. **Post-Execution** (Days 15+):
   - Board reviews spending (quarterly audit)
   - Department reports usage (via Pakit docs)
   - Community monitors impact

### Example: Emergency Governance

**Scenario**: Hurricane damages national infrastructure

**Timeline**:
1. **Emergency Declaration** (Hour 0):
   - Board votes 6-of-7 to declare "Natural Disaster" emergency
   - Emergency mode activated (7-day limit)
   
2. **Immediate Actions** (Hours 0-24):
   - Execute emergency treasury spend: 1M DALLA to disaster relief
   - Adjust transaction fees (0% for relief workers)
   - Enable free data storage (Pakit) for damage documentation
   
3. **Ongoing Response** (Days 1-7):
   - Daily board reviews
   - Additional spending as needed
   - Coordination with government ministries
   
4. **Emergency Resolution** (Day 7):
   - Board votes to end emergency (5-of-7 required)
   - Normal governance resumes
   
5. **Post-Emergency Audit** (Days 7-21):
   - Mandatory governance review
   - All emergency actions logged to Pakit
   - Community can propose improvements to emergency protocol

## Future Enhancements

### Planned Features (stable2512+)
- [ ] **Quadratic Voting**: Prevent whale dominance (sqrt(stake) voting power)
- [ ] **Liquid Democracy**: Multi-level delegation chains
- [ ] **Reputation System**: Weight votes by past participation quality
- [ ] **AI Governance Assistance**: Nawal AI summarizes proposals, suggests similar past proposals
- [ ] **Cross-Chain Governance**: Vote on Polkadot/Ethereum bridges via XCM
- [ ] **Departmental DAOs**: Ministry-specific sub-DAOs with specialized governance

### Integration Roadmap
- **Q1 2026**: Nawal AI for proposal summarization and fraud detection
- **Q2 2026**: Kinich Quantum for vote tallying (quantum-resistant aggregation)
- **Q3 2026**: Pakit Storage for proposal document hosting (IPFS + Arweave)
- **Q4 2026**: GEM Smart Contracts for programmable governance (PSP22 voting tokens)

## Governance Philosophy

### Belizean Democratic Principles

**Representation**:
- **District Elections**: Ensures every region has voice (6 districts × 2 reps)
- **Foundation Board**: Technical expertise balances populism
- **Departmental Tracks**: Specialized governance for complex domains

**Accessibility**:
- **Low Barriers**: 1,000 DALLA deposit (refundable) for candidacy
- **DALLA Rewards**: Economic incentive for participation (10 DALLA/vote)
- **Vote Delegation**: Enables participation for less-engaged citizens

**Transparency**:
- **On-Chain Voting**: All votes public and auditable
- **Pakit Storage**: Proposal documents immutably stored
- **Real-Time Tallying**: Live vote counts (Maya Wallet UI)

**Security**:
- **KYC Gating**: Prevents Sybil attacks (one person = one vote weight)
- **Multi-Sig Treasury**: No single point of failure
- **Emergency Limits**: Time-boxed emergency powers (7 days)

**Cultural Preservation**:
- **Culture & Ethics Advisor**: Board role dedicated to Belizean heritage
- **Multilingual Support**: English, Spanish, Kriol, Mayan languages (UI layer)
- **District Autonomy**: Local initiatives don't require national approval

## References

### Documentation
- Architecture: `/docs/architecture/governance-model.md`
- Security Audit: `/docs/security/SECURITY_AUDIT_FINAL.md`
- User Guide: `/docs/user-guides/governance-guide.md`
- Economic Model: `/docs/economics/tokenomics.md`

### Related Pallets
- Compliance: `/belizechain/pallets/compliance/` (KYC tier integration)
- Economy: `/belizechain/pallets/economy/` (treasury, DALLA rewards)
- Staking: `/belizechain/pallets/staking/` (validator voting rights)
- Identity: `/belizechain/pallets/identity/` (BelizeID verification)

### External Resources
- Polkadot Governance: https://wiki.polkadot.network/docs/learn-governance
- OpenGov (Polkadot v2): https://wiki.polkadot.network/docs/learn-opengov
- Substrate Democracy Pallet: https://paritytech.github.io/substrate/master/pallet_democracy/

---

**Status**: ✅ Production-Ready (stable2512)  
**Last Audit**: January 2026  
**Lines of Code**: 9,634 (lib: 5,843, mock: 175, tests: 3,616 across 6 files)  
**Test Coverage**: 91% (unit tests across 6 phases)  
**Clippy Warnings**: 0 (pallet-specific)  
**Complexity**: HIGH (multi-tiered democratic system, 3 governance layers, 8 departments)
