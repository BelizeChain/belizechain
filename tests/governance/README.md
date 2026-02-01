# 🏛️ Governance Integration Tests

## Overview

Tests for BelizeChain's multi-tiered governance system including:
- **Proposals**: Creation, voting, execution
- **Treasury**: Multi-sig operations, spending proposals
- **Council**: Elections, voting, district representation  
- **Emergency Powers**: JaguarMode activation, transaction pausing

## Governance Architecture

```
┌──────────────────────────────────────────────┐
│         BelizeChain Governance               │
├──────────────────────────────────────────────┤
│ Tier 1: Citizens (10 DALLA voting power)    │
│ Tier 2: Council (12 seats, 6 districts)     │
│ Tier 3: Ministries (8 departments)          │
│ Tier 4: Foundation Board (7 members)        │
└──────────────────────────────────────────────┘
```

## Test Categories

### 1. Proposal Tests (`test_proposals.py`)
- `test_create_proposal_success` - Citizen creates proposal with minimum deposit
- `test_create_proposal_insufficient_deposit` - Proposal fails without deposit
- `test_proposal_second_by_council` - Council seconds proposal for voting
- `test_proposal_enters_voting_period` - Proposal transitions to voting
- `test_proposal_execution_after_approval` - Approved proposal executes
- `test_proposal_cancellation` - Cancel proposal before voting

### 2. Voting Tests (`test_voting.py`)
- `test_citizen_vote_with_conviction` - Conviction voting (1x-6x multiplier)
- `test_council_vote_weighted` - Council votes count more
- `test_quorum_requirements` - Proposal requires minimum turnout
- `test_voting_period_expires` - Proposal fails if no quorum
- `test_delegated_voting` - Citizens delegate votes to council
- `test_vote_tallying` - Accurate vote counting

### 3. Treasury Tests (`test_treasury.py`)
- `test_treasury_multi_sig_approval` - 4-of-7 signature requirement
- `test_treasury_spending_proposal` - Proposal to spend treasury funds
- `test_treasury_automatic_funding` - 20% inflation feeds treasury
- `test_treasury_account_types` - Different limits per account type
- `test_treasury_tip_payout` - Community tips approved by council

### 4. Council Tests (`test_council.py`)
- `test_council_election` - 12 council seats elected from 6 districts
- `test_council_term_limits` - Maximum 3 consecutive terms
- `test_council_resignation` - Council member resigns mid-term
- `test_district_representation` - 2 seats per district enforced
- `test_council_motion` - Council creates internal motion
- `test_council_voting_power` - Council votes weighted by stake

### 5. Emergency Powers Tests (`test_emergency_powers.py`)
- `test_jaguarmode_activation` - 3-of-7 board activates emergency mode
- `test_transactions_paused` - All transfers paused during JaguarMode
- `test_system_freeze` - New accounts/operations blocked
- `test_emergency_treasury_access` - Board can access emergency funds
- `test_jaguarmode_deactivation` - 4-of-7 board deactivates
- `test_transparency_report` - All emergency actions logged publicly

## Running Tests

```bash
# All governance tests
pytest tests/integration/governance/ -v

# Specific test category
pytest tests/integration/governance/test_proposals.py -v

# Single test
pytest tests/integration/governance/test_voting.py::TestVoting::test_citizen_vote_with_conviction -v
```

## Test Data

### Test Accounts
- **Alice**: Sudo/Foundation Board member
- **Bob**: Citizen voter (10 DALLA power)
- **Charlie**: Council member (District 1)
- **Dave**: Council member (District 2)
- **Eve**: Ministry representative
- **Ferdie**: Standard citizen

### Test Proposals
- **Treasury Spending**: Request 100K DALLA for infrastructure
- **Parameter Change**: Adjust voting period from 7 to 14 days
- **Emergency Proposal**: Activate disaster relief fund

## Success Criteria

- ✅ 25+ governance tests covering all tiers
- ✅ 95%+ code coverage for Governance pallet
- ✅ All multi-sig scenarios tested
- ✅ Emergency powers tested without breaking system
- ✅ Vote tallying mathematically verified
- ✅ Council elections simulate real-world scenarios

## Resources

- [BelizeChain Governance Documentation](https://docs.belizechain.org/governance/)
- [Substrate Governance Guide](https://docs.substrate.io/reference/how-to-guides/pallet-design/use-loose-coupling/)
