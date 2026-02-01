# DAO Governance Templates - Simple DAO

BelizeChain's DAO template provides decentralized autonomous organization capabilities with proposal creation, weighted voting, and timelock execution.

## Overview

The Simple DAO contract enables:
- **Proposal Creation**: Members submit proposals for community voting
- **Weighted Voting**: Voting power based on DALLA token holdings
- **Timelock Execution**: Executed proposals have mandatory delay for security
- **Treasury Management**: Direct access to Economy pallet via chain extensions
- **Role-Based Permissions**: Admin, proposer, and voter roles

## Contract Architecture

```rust
#[ink(storage)]
pub struct SimpleDao {
    /// Minimum DALLA tokens required to create proposal
    proposal_threshold: u128,
    /// Minimum voting period in blocks
    voting_period: u32,
    /// Timelock delay in blocks before execution
    timelock_delay: u32,
    /// Quorum percentage (e.g., 40 = 40%)
    quorum_percentage: u8,
    /// Proposals storage
    proposals: Mapping<ProposalId, Proposal>,
    /// Next proposal ID
    next_proposal_id: ProposalId,
    /// Votes cast
    votes: Mapping<(ProposalId, AccountId), Vote>,
    /// DAO admin
    admin: AccountId,
}

#[derive(scale::Encode, scale::Decode, Clone)]
pub struct Proposal {
    /// Proposal ID
    id: ProposalId,
    /// Proposer account
    proposer: AccountId,
    /// Proposal title
    title: String,
    /// Proposal description
    description: String,
    /// Target contract for execution
    target: Option<AccountId>,
    /// Calldata to execute
    calldata: Vec<u8>,
    /// Block when voting starts
    start_block: u32,
    /// Block when voting ends
    end_block: u32,
    /// Votes in favor
    votes_for: u128,
    /// Votes against
    votes_against: u128,
    /// Proposal state
    state: ProposalState,
    /// Block when executable (after timelock)
    executable_at: Option<u32>,
}

pub enum ProposalState {
    Pending,
    Active,
    Succeeded,
    Defeated,
    Queued,
    Executed,
    Cancelled,
}
```

## Core Functions

### 1. Create Proposal

```rust
#[ink(message)]
pub fn propose(
    &mut self,
    title: String,
    description: String,
    target: Option<AccountId>,
    calldata: Vec<u8>,
) -> Result<ProposalId>
```

**Example:**
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

// Propose treasury allocation
const proposalId = await sdk.daoPropose(
    daoAddress,
    alice,
    {
        title: 'Fund Tourism Marketing Campaign',
        description: 'Allocate 100,000 DALLA for Q2 2026 tourism marketing',
        target: treasuryAddress,
        calldata: encodeFunctionCall('allocate', [100000000000000000])
    }
);

console.log(`Proposal #${proposalId} created`);
```

**Requirements:**
- Proposer must hold ≥ `proposal_threshold` DALLA
- Title: max 128 characters
- Description: max 1024 characters

**Gas Cost**: ~35,000 units

### 2. Cast Vote

```rust
#[ink(message)]
pub fn vote(
    &mut self,
    proposal_id: ProposalId,
    support: bool,
) -> Result<()>
```

**Example:**
```javascript
// Vote in favor
await sdk.daoVote(daoAddress, bob, proposalId, true);

// Vote against
await sdk.daoVote(daoAddress, charlie, proposalId, false);
```

**Voting Power**: Based on DALLA balance at proposal start block
**Requirements:**
- Proposal must be in Active state
- Current block within voting period
- Voter has not already voted

**Gas Cost**: ~18,000 units

### 3. Queue Proposal (After Success)

```rust
#[ink(message)]
pub fn queue(&mut self, proposal_id: ProposalId) -> Result<()>
```

**Example:**
```javascript
// Queue successful proposal for timelock
await sdk.daoQueue(daoAddress, admin, proposalId);
```

**Requirements:**
- Proposal state: Succeeded
- Quorum reached (votes_for + votes_against ≥ quorum)
- More votes_for than votes_against

**Effect**: Sets `executable_at` = current_block + `timelock_delay`

### 4. Execute Proposal

```rust
#[ink(message)]
pub fn execute(&mut self, proposal_id: ProposalId) -> Result<()>
```

**Example:**
```javascript
// Execute after timelock expires
await sdk.daoExecute(daoAddress, executor, proposalId);
```

**Requirements:**
- Proposal state: Queued
- Current block ≥ `executable_at`
- Execution succeeds

**Action**: Calls `target` contract with `calldata`, marks proposal as Executed

### 5. Cancel Proposal

```rust
#[ink(message)]
pub fn cancel(&mut self, proposal_id: ProposalId) -> Result<()>
```

**Allowed if:**
- Caller is admin, OR
- Proposer's DALLA balance < `proposal_threshold`

## Governance Parameters

### Default Configuration

```rust
impl SimpleDao {
    #[ink(constructor)]
    pub fn new() -> Self {
        Self {
            proposal_threshold: 10_000_000_000_000_000,  // 10,000 DALLA
            voting_period: 50_400,                        // ~7 days (6s blocks)
            timelock_delay: 14_400,                       // ~1 day
            quorum_percentage: 40,                        // 40% of total votes
            // ...
        }
    }
}
```

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Proposal Threshold** | 10,000 DALLA | Min tokens to create proposal |
| **Voting Period** | 50,400 blocks (~7 days) | Duration of voting |
| **Timelock Delay** | 14,400 blocks (~1 day) | Delay before execution |
| **Quorum** | 40% | Min participation for validity |

### Update Parameters (Admin Only)

```rust
#[ink(message)]
pub fn update_params(
    &mut self,
    proposal_threshold: Option<u128>,
    voting_period: Option<u32>,
    timelock_delay: Option<u32>,
    quorum_percentage: Option<u8>,
) -> Result<()>
```

## Integration Examples

### 1. Treasury Allocation DAO

```rust
// Proposal to allocate treasury funds
let calldata = scale::Encode::encode(&(
    "treasury_allocate",
    recipient,
    100_000_000_000_000_000_u128  // 100K DALLA
));

let proposal_id = dao.propose(
    "Fund Community Development".to_string(),
    "Allocate funds for developer grants".to_string(),
    Some(treasury_address),
    calldata,
)?;
```

### 2. BelizeX DEX Parameter DAO

```rust
// Proposal to adjust DEX trading fees
let calldata = scale::Encode::encode(&(
    "set_trading_fee",
    30_u16  // 0.30% fee
));

dao.propose(
    "Reduce DEX Trading Fee".to_string(),
    "Lower fee from 0.50% to 0.30% to increase volume".to_string(),
    Some(belizex_address),
    calldata,
)?;
```

### 3. Land Ledger DAO

```rust
// Proposal to approve land title transfer
let calldata = scale::Encode::encode(&(
    "approve_transfer",
    parcel_id,
    new_owner
));

dao.propose(
    "Approve Land Transfer #12345".to_string(),
    "Transfer parcel ownership per sale agreement".to_string(),
    Some(land_ledger_address),
    calldata,
)?;
```

## Events

```rust
#[ink(event)]
pub struct ProposalCreated {
    #[ink(topic)]
    id: ProposalId,
    proposer: AccountId,
    title: String,
}

#[ink(event)]
pub struct VoteCast {
    #[ink(topic)]
    proposal_id: ProposalId,
    #[ink(topic)]
    voter: AccountId,
    support: bool,
    weight: u128,
}

#[ink(event)]
pub struct ProposalQueued {
    #[ink(topic)]
    id: ProposalId,
    executable_at: u32,
}

#[ink(event)]
pub struct ProposalExecuted {
    #[ink(topic)]
    id: ProposalId,
}
```

## Security Features

### 1. Timelock Protection

```rust
// Prevents immediate execution of passed proposals
pub fn execute(&mut self, proposal_id: ProposalId) -> Result<()> {
    let proposal = self.proposals.get(proposal_id).ok_or(Error::NotFound)?;
    
    ensure!(proposal.state == ProposalState::Queued, Error::NotQueued);
    
    let current_block = self.env().block_number();
    let executable_at = proposal.executable_at.ok_or(Error::NoTimelockSet)?;
    
    ensure!(current_block >= executable_at, Error::TimelockNotExpired);
    
    // Execute...
}
```

**Purpose**: Gives community time to react to malicious proposals

### 2. Quorum Enforcement

```rust
pub fn queue(&mut self, proposal_id: ProposalId) -> Result<()> {
    let proposal = self.proposals.get(proposal_id)?;
    
    // Calculate total votes
    let total_votes = proposal.votes_for + proposal.votes_against;
    
    // Get total DALLA supply for quorum calculation
    let total_supply = self.get_dalla_total_supply()?;
    let quorum_needed = (total_supply * self.quorum_percentage as u128) / 100;
    
    ensure!(total_votes >= quorum_needed, Error::QuorumNotReached);
    ensure!(proposal.votes_for > proposal.votes_against, Error::ProposalDefeated);
    
    // Queue...
}
```

### 3. Snapshot Voting Power

```rust
// Prevent vote buying/selling during voting period
pub fn vote(&mut self, proposal_id: ProposalId, support: bool) -> Result<()> {
    let proposal = self.proposals.get(proposal_id)?;
    let voter = self.env().caller();
    
    // Get voter's DALLA balance at proposal start block
    let voting_power = self.get_dalla_balance_at_block(
        voter,
        proposal.start_block
    )?;
    
    ensure!(voting_power > 0, Error::NoVotingPower);
    
    // Record vote with snapshot weight
    // ...
}
```

## Deployment

```bash
cd gem/simple_dao
cargo contract build --release

cargo contract instantiate \
    --suri //Alice \
    --constructor new \
    --url wss://testnet.belizechain.org
```

## Advanced Patterns

### 1. Delegated Voting

```rust
#[ink(message)]
pub fn delegate(&mut self, delegatee: AccountId) -> Result<()> {
    let delegator = self.env().caller();
    self.delegations.insert(delegator, &delegatee);
    Ok(())
}

// Modify vote to include delegated power
pub fn get_voting_power(&self, account: AccountId, block: u32) -> u128 {
    let own_power = self.get_balance_at_block(account, block);
    let delegated_power = self.get_delegated_power(account, block);
    own_power + delegated_power
}
```

### 2. Multi-Choice Proposals

```rust
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

// Weighted voting with abstention
pub fn vote_multi(
    &mut self,
    proposal_id: ProposalId,
    choice: VoteChoice,
) -> Result<()> {
    match choice {
        VoteChoice::For => self.proposal.votes_for += weight,
        VoteChoice::Against => self.proposal.votes_against += weight,
        VoteChoice::Abstain => {}, // Counts toward quorum only
    }
}
```

### 3. Proposal Dependencies

```rust
// Require parent proposal to pass first
pub struct Proposal {
    // ...
    depends_on: Option<ProposalId>,
}

pub fn queue(&mut self, proposal_id: ProposalId) -> Result<()> {
    if let Some(parent_id) = proposal.depends_on {
        let parent = self.proposals.get(parent_id)?;
        ensure!(parent.state == ProposalState::Executed, Error::DependencyNotMet);
    }
    // ...
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    #[ink::test]
    fn proposal_lifecycle() {
        let mut dao = SimpleDao::new();
        
        // Create proposal
        let id = dao.propose(
            "Test Proposal".to_string(),
            "Description".to_string(),
            None,
            vec![],
        ).unwrap();
        
        // Vote
        dao.vote(id, true).unwrap();
        
        // Advance blocks past voting period
        advance_block(50_401);
        
        // Queue
        dao.queue(id).unwrap();
        
        // Advance blocks past timelock
        advance_block(14_401);
        
        // Execute
        dao.execute(id).unwrap();
        
        let proposal = dao.get_proposal(id).unwrap();
        assert_eq!(proposal.state, ProposalState::Executed);
    }
}
```

## Resources

- **Contract Source**: [github.com/BelizeChain/gem/tree/main/simple_dao](https://github.com/BelizeChain/gem/tree/main/simple_dao)
- **Governor Bravo Reference**: [Compound Governor](https://github.com/compound-finance/compound-protocol/blob/master/contracts/Governance/GovernorBravo.sol)
- **DAO Best Practices**: [OpenZeppelin Governor](https://docs.openzeppelin.com/contracts/4.x/governance)
