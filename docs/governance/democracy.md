# Democracy & District Councils

**Governance System • Proposal Lifecycle • Voting Mechanisms**

Complete guide to BelizeChain's on-chain democratic governance.

---

## Governance Structure

### Three-Tier System

```
┌─────────────────────────────────────┐
│ National Governance                 │
│ - Treasury spending (>100K DALLA)   │
│ - Protocol upgrades                 │
│ - Emergency actions                 │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ District Councils (6)               │
│ - Local infrastructure              │
│ - Community projects                │
│ - District representatives          │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ Community Groups                    │
│ - Village proposals                 │
│ - Local initiatives                 │
│ - Grassroots participation          │
└─────────────────────────────────────┘
```

---

## District Councils

### Six Districts

| District | Population | Council Size | Treasury Allocation |
|----------|-----------|--------------|---------------------|
| **Belize** | 95,000 | 7 members | 30% |
| **Cayo** | 85,000 | 7 members | 25% |
| **Orange Walk** | 50,000 | 5 members | 15% |
| **Corozal** | 40,000 | 5 members | 12% |
| **Stann Creek** | 35,000 | 5 members | 10% |
| **Toledo** | 30,000 | 5 members | 8% |

### Council Election

**Frequency:** Every 2 years (on-chain voting)

**Eligibility:**
- Citizen (BelizeID verified)
- KYC level: Verified or Enhanced
- Minimum stake: 5,000 DALLA bonded
- District residency proof

```javascript
// Run for district council
await api.tx.governance.registerCandidate(
  'Cayo',  // District
  'Infrastructure improvements and youth programs',  // Platform
  5_000_000_000_000_000n  // 5K DALLA bond
).signAndSend(candidate);

// Vote for candidates
await api.tx.governance.voteForCandidate(
  candidateAddress,
  1_000_000_000_000_000n  // 1K DALLA voting power
).signAndSend(voter);
```

**Voting mechanism:**
- Conviction voting: Lock DALLA for longer = more weight
- 1x: 7-day lock
- 2x: 14-day lock
- 4x: 28-day lock
- 6x: 56-day lock

---

## Proposal Types

### 1. Treasury Proposals

**Small (<10K DALLA):** District council approval  
**Medium (10K-100K DALLA):** District referendum  
**Large (>100K DALLA):** National referendum

```rust
// Submit treasury proposal
pub fn propose_spend(
    origin: OriginFor<T>,
    value: BalanceOf<T>,
    beneficiary: T::AccountId,
    description: BoundedVec<u8, ConstU32<256>>
) -> DispatchResult {
    // Require 5% bond
    let bond = value.saturating_mul(5) / 100;
    ensure!(
        T::Currency::free_balance(&who) >= bond,
        Error::<T>::InsufficientBond
    );
    
    T::Currency::reserve(&who, bond)?;
    
    // Route to appropriate governance level
    if value < 10_000 * DALLA {
        Self::route_to_district_council(&who, value, beneficiary)?;
    } else if value < 100_000 * DALLA {
        Self::route_to_district_referendum(&who, value, beneficiary)?;
    } else {
        Self::route_to_national_referendum(&who, value, beneficiary)?;
    }
}
```

### 2. Protocol Upgrade Proposals

**Requires:**
- 2-week voting period
- 66% supermajority
- 50% quorum (of staked DALLA)

```javascript
// Propose runtime upgrade
const newRuntimeWasm = fs.readFileSync('runtime.wasm');

await api.tx.governance.propose(
  api.tx.system.setCode(newRuntimeWasm),
  0  // No treasury funds
).signAndSend(proposer, { value: 10_000_000_000_000_000n });  // 10K DALLA bond
```

### 3. Emergency Proposals

**Fast-track for security issues:**
- 24-hour voting period
- Treasury Council veto power
- Requires 75% approval

**Use cases:**
- Security patches
- Network attacks
- Critical bugs

```rust
// Emergency proposal (Treasury Council only)
pub fn emergency_proposal(
    origin: OriginFor<T>,
    proposal: Box<<T as Config>::Proposal>
) -> DispatchResult {
    ensure!(
        TreasuryCouncil::<T>::get().contains(&who),
        Error::<T>::NotCouncilMember
    );
    
    let proposal_id = Self::do_propose(proposal, true)?;
    
    // 24-hour voting period
    VotingEnds::<T>::insert(
        proposal_id,
        current_block + (24 * 3600 / 6)  // 24 hours in blocks
    );
}
```

---

## Proposal Lifecycle

### Phase 1: Submission (Block 0)

```
Proposer submits:
1. Proposal call (encoded extrinsic)
2. Bond deposit (1,000-10,000 DALLA based on type)
3. Description (max 256 chars)

Proposal enters queue
```

### Phase 2: Voting Period (7-14 days)

```
Voters can:
- Vote Aye (approve)
- Vote Nay (reject)
- Vote Abstain (count toward quorum)

Voting power: 1 DALLA staked = 1 vote
Conviction multipliers: 1x to 6x
```

```typescript
// Vote on proposal
await api.tx.governance.vote(
  proposalId,
  {
    aye: true,
    conviction: 'Locked4x'  // 28-day lock for 4x voting power
  }
).signAndSend(voter);
```

### Phase 3: Tallying (Automated)

```python
# Tally votes
def tally_votes(proposal_id):
    proposal = get_proposal(proposal_id)
    
    total_ayes = sum(vote.power for vote in proposal.votes if vote.aye)
    total_nays = sum(vote.power for vote in proposal.votes if not vote.aye)
    total_abstain = sum(vote.power for vote in proposal.votes if vote.abstain)
    
    total_votes = total_ayes + total_nays + total_abstain
    total_staked = get_total_staked_dalla()
    
    # Check quorum (50% of staked DALLA must participate)
    quorum = total_votes / total_staked
    if quorum < 0.5:
        return "FAILED_QUORUM"
    
    # Check approval threshold
    approval_rate = total_ayes / (total_ayes + total_nays)
    
    if proposal.type == "PROTOCOL_UPGRADE":
        threshold = 0.66  # 66% supermajority
    elif proposal.type == "EMERGENCY":
        threshold = 0.75  # 75% supermajority
    else:
        threshold = 0.50  # Simple majority
    
    if approval_rate >= threshold:
        return "APPROVED"
    else:
        return "REJECTED"
```

### Phase 4: Execution (Block N+1 after voting ends)

```rust
// Automatic execution if approved
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_finalize(n: T::BlockNumber) {
        // Check for proposals with voting ended
        for (proposal_id, voting_end) in VotingEnds::<T>::iter() {
            if n > voting_end {
                let result = Self::tally_proposal(proposal_id);
                
                if result == TallyResult::Approved {
                    // Execute proposal
                    if let Some(proposal) = Proposals::<T>::get(proposal_id) {
                        let _ = proposal.call.dispatch(
                            frame_system::RawOrigin::Root.into()
                        );
                        
                        // Refund bond + bonus
                        Self::refund_bond(&proposal.proposer, proposal.bond);
                        T::Currency::deposit_creating(&proposal.proposer, 100 * DALLA);
                    }
                } else {
                    // Slash bond
                    Self::slash_bond(proposal_id);
                }
                
                // Clean up
                VotingEnds::<T>::remove(proposal_id);
            }
        }
    }
}
```

---

## Multi-Sig Treasury

### 4-of-7 Signature Scheme

**Council members:**
1. Prime Minister (or designate)
2. Minister of Finance
3. Financial Services Commission Chair
4. Central Bank Governor
5. Belize District Representative
6. Cayo District Representative
7. Stann Creek District Representative

### Approval Process

```typescript
// Step 1: First signatory initiates multi-sig
const threshold = 4;
const otherSignatories = [member2, member3, member4, member5, member6, member7];

const { data: multiSig } = await api.tx.treasury.approveProposal(proposalId);

await api.tx.multisig.asMulti(
  threshold,
  otherSignatories.sort(),
  null,
  multiSig,
  false,  // Not final
  1_000_000_000  // Max weight
).signAndSend(member1);

// Step 2-4: Additional signatories approve
await api.tx.multisig.asMulti(
  threshold,
  otherSignatories.sort(),
  multiSigTimepoint,
  multiSig,
  false,
  1_000_000_000
).signAndSend(member2);

// ... member3, member4 ...

// Step 5: Final (4th) signature executes
await api.tx.multisig.asMulti(
  threshold,
  otherSignatories.sort(),
  multiSigTimepoint,
  multiSig,
  true,  // Final signature
  1_000_000_000
).signAndSend(member4);

// Treasury transfer executes automatically
```

---

## Voting Analytics

### Participation Metrics

```sql
-- Query voting participation (example analytics)
SELECT
  proposal_id,
  COUNT(DISTINCT voter) as voter_count,
  SUM(voting_power) as total_voting_power,
  SUM(CASE WHEN vote = 'Aye' THEN voting_power ELSE 0 END) as aye_power,
  SUM(CASE WHEN vote = 'Nay' THEN voting_power ELSE 0 END) as nay_power
FROM votes
GROUP BY proposal_id;
```

**Historical participation:**
```
Month 1 (Jan 2026): 12% turnout
Month 3: 28% turnout
Month 6: 45% turnout
Month 12: 62% turnout (target: 60%+)
```

### Whale Protection

**Quadratic voting considered:**
- Linear: 1 DALLA = 1 vote (current)
- Quadratic: √DALLA = votes (future upgrade?)

**Example:**
```python
# Linear voting (current)
voter_1_stake = 1_000_000  # 1M DALLA
voter_2_stake = 10_000     # 10K DALLA

voter_1_power = voter_1_stake  # 1M votes
voter_2_power = voter_2_stake  # 10K votes
# Ratio: 100:1

# Quadratic voting (potential future)
voter_1_power_quad = math.sqrt(voter_1_stake)  # 1,000 votes
voter_2_power_quad = math.sqrt(voter_2_stake)  # 100 votes
# Ratio: 10:1 (more equitable)
```

---

## Governance Calendar

### Monthly Cycle

```
Week 1: Proposal submission period
Week 2: Debate & amendments
Week 3-4: Voting period
Week 5: Execution & next cycle starts
```

### Annual Events

```
January: Budget proposals for fiscal year
April: Q1 treasury report
July: Mid-year governance review
October: Council elections (even years)
December: Year-end audit publication
```

---

## Related Documentation

- [Voting Mechanisms](./voting.md)
- [Treasury Management](../economics/tokenomics.md#treasury-management)
- [Governance Pallet API](../developer-guides/pallet-apis-core.md#governance-pallet)
- [Community Pallet](../developer-guides/pallet-apis-services.md#community-pallet)
- [Blue Hole Portal Guide](../user-guides/blue-hole-portal.md) (Government dashboard)
