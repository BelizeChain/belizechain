# Voting Mechanisms

**Conviction Voting • Referendum Process • Delegation**

Comprehensive guide to voting on BelizeChain governance proposals.

---

## Voting Power Calculation

### Base Voting Power

```rust
// Simple formula
voting_power = staked_dalla × conviction_multiplier

// Example
stake = 10_000 * DALLA
conviction = 4  // 4x multiplier (28-day lock)
voting_power = 10_000 × 4 = 40_000 votes
```

### Conviction Multipliers

| Conviction | Lock Period | Multiplier | Example (10K DALLA) |
|-----------|-------------|------------|---------------------|
| **None** | 0 days | 0.1x | 1,000 votes |
| **Locked1x** | 7 days | 1x | 10,000 votes |
| **Locked2x** | 14 days | 2x | 20,000 votes |
| **Locked3x** | 21 days | 3x | 30,000 votes |
| **Locked4x** | 28 days | 4x | 40,000 votes |
| **Locked5x** | 56 days | 5x | 50,000 votes |
| **Locked6x** | 112 days | 6x | 60,000 votes |

**Rationale:** Stronger conviction (longer lock) = more voting power

---

## How to Vote

### Via Polkadot.js (Maya Wallet)

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

const api = await ApiPromise.create({ 
  provider: new WsProvider('ws://localhost:9944') 
});

// Vote Aye with 4x conviction
await api.tx.governance.vote(
  proposalId,
  {
    Standard: {
      vote: {
        aye: true,
        conviction: 'Locked4x'
      },
      balance: 10_000_000_000_000_000n  // 10K DALLA
    }
  }
).signAndSend(voterAccount, ({ status, events }) => {
  if (status.isInBlock) {
    console.log('Vote cast in block:', status.asInBlock.toHex());
    
    events.forEach(({ event }) => {
      if (event.method === 'VoteCast') {
        const [proposalId, voter, vote, power] = event.data;
        console.log(`Proposal ${proposalId}: ${power} voting power`);
      }
    });
  }
});
```

### Via Rust Runtime

```rust
// Vote from pallet or smart contract
let vote = Vote {
    aye: true,
    conviction: Conviction::Locked4x
};

Governance::vote(
    Origin::signed(voter),
    proposal_id,
    AccountVote::Standard {
        vote,
        balance: 10_000 * DALLA
    }
)?;
```

### Vote Splitting

**Vote with different amounts on same proposal:**

```javascript
// Split 15K DALLA: 10K Aye, 5K Nay (hedge your bet)
await api.tx.governance.vote(
  proposalId,
  {
    Split: {
      aye: 10_000_000_000_000_000n,  // 10K DALLA Aye
      nay: 5_000_000_000_000_000n    // 5K DALLA Nay
    }
  }
).signAndSend(voter);
```

---

## Referendum Process

### Referendum Types

#### 1. Public Referendums
**Trigger:** Anyone can propose  
**Deposit:** 1,000 DALLA  
**Voting period:** 7 days  
**Threshold:** Simple majority (>50%)

```javascript
// Submit public referendum
await api.tx.governance.propose(
  api.tx.treasury.approveProposal(proposalId),
  50_000_000_000_000_000n  // 50K DALLA treasury request
).signAndSend(proposer);
```

#### 2. Council Referendums
**Trigger:** District council majority  
**Deposit:** None (council backed)  
**Voting period:** 14 days  
**Threshold:** Simple majority

#### 3. Emergency Referendums
**Trigger:** Treasury Council (4-of-7)  
**Deposit:** None  
**Voting period:** 24 hours  
**Threshold:** 75% supermajority

### Adaptive Quorum Biasing

**Turnout-based approval threshold:**

```python
def calculate_approval_threshold(proposal_type, turnout):
    """
    Lower turnout = higher approval needed
    Higher turnout = lower approval needed
    """
    if proposal_type == "PUBLIC":
        # Simple majority at 50% turnout
        # Supermajority needed at low turnout
        return 0.5 + (0.25 * (1 - turnout))
    
    elif proposal_type == "COUNCIL":
        # Council backing = lower threshold
        return 0.4 + (0.2 * (1 - turnout))
    
    elif proposal_type == "EMERGENCY":
        # Always high threshold
        return 0.75

# Examples
turnout_10_percent = 0.10
turnout_50_percent = 0.50
turnout_90_percent = 0.90

# Public referendum thresholds
threshold_10 = 0.5 + (0.25 * 0.9)  # 72.5% needed
threshold_50 = 0.5 + (0.25 * 0.5)  # 62.5% needed
threshold_90 = 0.5 + (0.25 * 0.1)  # 52.5% needed
```

---

## Vote Delegation

### Delegate Your Voting Power

**Delegate to expert voter:**

```typescript
// Delegate 10K DALLA voting power to expert
await api.tx.governance.delegate(
  expertAddress,
  'Locked2x',  // Conviction for delegated votes
  10_000_000_000_000_000n  // 10K DALLA
).signAndSend(delegator);
```

**Expert votes on your behalf:**
- You don't need to monitor every proposal
- Expert's vote automatically includes your delegated power
- You can override by voting directly (removes delegation for that vote)

### Delegation Rewards

**Delegatees earn 0.5% of delegated votes:**

```python
# Expert with 100K DALLA delegated
delegated_power = 100_000  # DALLA
proposals_voted = 50  # in a month

reward_per_vote = delegated_power * 0.005 / 100
# = 5 DALLA per vote

monthly_earnings = reward_per_vote * proposals_voted
# = 250 DALLA/month

annual_earnings = monthly_earnings * 12
# = 3,000 DALLA/year
```

**Delegators benefit:**
- No need to research every proposal
- Experts incentivized to vote responsibly
- Can always override if disagree

---

## Voting Strategies

### Conservative Voter (Low Risk)

```javascript
// Always vote with low conviction (quick unlock)
const strategy = {
  conviction: 'Locked1x',  // 7-day lock
  participation: 'High',   // Vote on most proposals
  delegation: 'None'       // Vote yourself
};

// Pros: Maximum flexibility, quick exit
// Cons: Lower voting power per DALLA
```

### Activist Voter (High Impact)

```javascript
const strategy = {
  conviction: 'Locked6x',  // 112-day lock
  participation: 'Selective',  // Only vote on important issues
  delegation: 'None'
};

// Pros: 6x voting power, strong influence
// Cons: Capital locked for ~4 months
```

### Delegated Voter (Passive)

```javascript
const strategy = {
  conviction: 'Locked2x',  // 14-day lock
  participation: 'None',   // Delegate everything
  delegation: expertAddress
};

// Pros: Passive participation, expert decision-making
// Cons: Less direct control
```

---

## Referendum Results Tracking

### Query Voting Status

```javascript
// Get current tally for active proposal
const tally = await api.query.governance.proposals(proposalId);

if (tally.isSome) {
  const { ayes, nays, abstains, voting_ends } = tally.unwrap();
  
  console.log(`Aye power: ${ayes.toString()}`);
  console.log(`Nay power: ${nays.toString()}`);
  console.log(`Abstain power: ${abstains.toString()}`);
  
  const currentBlock = await api.query.system.number();
  const blocksRemaining = voting_ends - currentBlock;
  const hoursRemaining = (blocksRemaining * 6) / 3600;
  
  console.log(`Voting ends in ${hoursRemaining.toFixed(1)} hours`);
  
  // Calculate approval rate
  const approvalRate = ayes / (ayes + nays);
  console.log(`Current approval: ${(approvalRate * 100).toFixed(1)}%`);
}
```

### Historical Results

```sql
-- Example analytics query (off-chain database)
SELECT
  proposal_id,
  proposal_type,
  SUM(aye_power) as total_ayes,
  SUM(nay_power) as total_nays,
  SUM(aye_power + nay_power + abstain_power) as total_turnout,
  CASE
    WHEN SUM(aye_power) / (SUM(aye_power) + SUM(nay_power)) > threshold
    THEN 'APPROVED'
    ELSE 'REJECTED'
  END as result
FROM governance_votes
GROUP BY proposal_id;
```

**Historical approval rates (hypothetical):**
```
Treasury proposals (<50K DALLA): 75% approval rate
Treasury proposals (>50K DALLA): 45% approval rate
Protocol upgrades: 85% approval rate
Emergency proposals: 95% approval rate
```

---

## Governance Participation Levels

### Participation Tiers

| Tier | Monthly Votes | Avg Conviction | Annual Rewards |
|------|---------------|----------------|----------------|
| **Inactive** | 0-2 | N/A | 0 DALLA |
| **Casual** | 3-10 | 1-2x | ~10 DALLA |
| **Active** | 11-25 | 2-4x | ~30 DALLA |
| **Power User** | 26-50 | 4-6x | ~60 DALLA |
| **Delegate** | 51+ | 4-6x | ~150 DALLA |

### Engagement Analytics

```python
# Calculate user engagement score
def engagement_score(user):
    votes_cast = count_votes(user)
    avg_conviction = average_conviction(user)
    delegated_to_others = sum_delegated(user)
    
    score = (
        votes_cast * 1.0 +
        avg_conviction * 10.0 +
        delegated_to_others / 1000 * 5.0
    )
    
    return score

# Leaderboard (top 10 governance participants)
# Updated monthly, displayed in Blue Hole Portal
```

---

## Best Practices

### 1. Research Before Voting
```
✅ DO:
- Read proposal description fully
- Check proposer's track record
- Estimate impact on treasury/protocol
- Ask questions in governance forum

❌ DON'T:
- Vote blindly following others
- Ignore financial implications
- Vote without understanding conviction locks
```

### 2. Use Conviction Wisely
```
✅ DO:
- Use high conviction (5-6x) for critical issues
- Use low conviction (1-2x) for minor proposals
- Plan conviction unlocks around liquidity needs

❌ DON'T:
- Always use 6x conviction (unnecessary capital lock)
- Lock more than you can afford to freeze
- Forget about conviction unlock dates
```

### 3. Delegate Strategically
```
✅ DO:
- Delegate to experts in specific domains
- Monitor delegatee voting record
- Override delegation if you disagree strongly

❌ DON'T:
- Delegate 100% of voting power permanently
- Delegate to unknown/unproven accounts
- Forget you've delegated (and vote accidentally)
```

---

## Related Documentation

- [Democracy System](./democracy.md)
- [Governance Pallet API](../developer-guides/pallet-apis-core.md#governance-pallet)
- [Treasury Management](../economics/tokenomics.md#treasury-management)
- [Maya Wallet Voting Guide](../user-guides/maya-wallet.md#governance)
- [Blue Hole Portal Dashboard](../user-guides/blue-hole-portal.md)
