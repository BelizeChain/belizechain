# BelizeChain Governance Documentation

This directory contains the official governance system documentation for BelizeChain, Belize's sovereign blockchain infrastructure.

## Overview

The BelizeChain Governance Pallet implements a sophisticated multi-tiered democratic system combining:
- **Direct Democracy**: District-based council elections
- **Representative Democracy**: Vote delegation with transparency
- **Departmental Governance**: 8 government ministries with specialized workflows
- **Foundation Board**: 7 expert roles with term limits
- **Treasury Management**: Multi-signature spending controls
- **Emergency Powers**: National security provisions

## Documentation Structure

### High-Level Documents

#### [GOVERNANCE_STATUS.md](./GOVERNANCE_STATUS.md) *(BROKEN — file does not exist; flagged 2026-09-10 docs audit. Executive summary as of 2026-09-10: governance pallets implemented and exercised on testnet, but "87.5% complete / Phase 7" claims are UNVERIFIED — no dated per-phase ledger exists.)*
**Current development status and metrics**
- Overall progress tracker (87.5% complete - Phase 7)
- Test coverage summary (137/137 tests passing)
- Feature implementation checklist
- Code metrics and statistics
- Production readiness assessment

#### [GOVERNANCE_IMPLEMENTATION_PLAN.md](./GOVERNANCE_IMPLEMENTATION_PLAN.md)
**Complete 8-phase development roadmap**
- Phase-by-phase feature breakdown
- Technical specifications
- Timeline and dependencies
- Design decisions and rationale
- Integration requirements

### Phase Completion Reports (in root directory)

Detailed technical reports for each completed phase:
- `GOVERNANCE_PHASE2_COMPLIANCE_INTEGRATION.md` - KYC/AML integration
- `GOVERNANCE_PHASE3_DEPARTMENTAL.md` - 8 ministry system
- `GOVERNANCE_PHASE4_BOARD_SYSTEM.md` - Foundation board with 7 roles
- `GOVERNANCE_PHASE5_EXECUTION.md` - Treasury and action execution
- `GOVERNANCE_PHASE6_ELECTIONS.md` - District-based elections (6 districts, 12 seats)
- `GOVERNANCE_PHASE7_ADVANCED.md` - Vote delegation, amendments, rewards, priority queues

## Quick Start

### For Developers

```bash
# Navigate to pallet
cd belizechain/pallets/governance

# Run tests
cargo test

# Check compilation
cargo check -p pallet-belize-governance

# View implementation
cat src/lib.rs
```

### For Governance Participants

**Citizens** (KYC Level 2+):
- Submit proposals (250 DALLA deposit)
- Vote on proposals with conviction multipliers
- Delegate voting power to trusted representatives
- Claim participation rewards (10 DALLA per vote)
- Run for district council seats

**Council Members** (Elected):
- 12 seats across 6 districts (proportional allocation)
- 2-year terms with re-election
- Enhanced voting weight
- Monthly rewards (500 DALLA)
- Emergency override powers

**Department Managers** (Appointed):
- Submit department-specific proposals
- Manage ministry budgets
- Coordinate cross-department initiatives
- Treasury allocation authority

**Board Members** (7 Roles):
- Founder, Technical Steward, FSC Rep, BTB Delegate
- Citizen Delegate, Security Auditor, Culture & Ethics Advisor
- 1-2 year terms with rotation
- Specialized governance functions

## Key Features

### 1. Multi-Tiered Democracy
- **District Elections**: 6 Belize districts elect 12 council members
- **Board System**: 7 specialized roles with term limits
- **Department Governance**: 8 ministries with dedicated workflows
- **Vote Delegation**: Representative democracy option

### 2. Compliance-First Design
- KYC/AML checks via pallet-belize-compliance
- ParticipationTier requirements (Standard tier minimum)
- Restricted account handling
- Financial Services Commission (FSC) oversight

### 3. Treasury Management
- Multi-signature spending controls
- Department-specific budget allocations
- Emergency fund mechanisms
- Transparent execution tracking

### 4. Participation Incentives
- **Vote Rewards**: 10 DALLA per vote
- **Proposal Rewards**: 100 DALLA per proposal
- **Council Rewards**: 500 DALLA per month
- Total rewards tracked for treasury management

### 5. Advanced Features
- Vote delegation with expiry (max 100 per delegate)
- Proposal amendments (before voting starts)
- Priority queue (4 levels: Low/Normal/High/Critical)
- Voting power calculation (base + delegations)

## Technical Specifications

### Substrate Framework
- **Version**: FRAME v42
- **Runtime**: Custom BelizeChain runtime
- **Dependencies**: pallet-balances, pallet-belize-compliance
- **Storage**: BoundedVec for MaxEncodedLen compliance

### Extrinsics (23 total)
```rust
// Core Governance (0-5)
submit_proposal, cast_vote, finalize_proposal
update_community_rank, update_pouw_contribution, council_override

// Departmental (6-8)
set_department_manager, submit_department_proposal, approve_cross_department

// Board System (9-12)
add_board_member, remove_board_member, nominate_for_delegate, vote_for_delegate

// Execution (13)
execute_proposal

// Elections (14-17)
start_district_election, register_candidate, vote_in_district_election, finalize_district_election

// Advanced (18-22)
delegate_vote, revoke_delegation, amend_proposal, claim_participation_reward, set_proposal_priority
```

### Districts & Seats
| District | Population | Seats |
|----------|-----------|-------|
| Belize | ~100,000 | 3 |
| Cayo | ~90,000 | 2 |
| Corozal | ~45,000 | 1 |
| Orange Walk | ~50,000 | 2 |
| Stann Creek | ~35,000 | 2 |
| Toledo | ~35,000 | 2 |
| **Total** | **~355,000** | **12** |

### Departments (8 Ministries)
1. **Finance** - Treasury, budgets, economic policy
2. **Education** - Schools, training, scholarships
3. **Health** - Hospitals, clinics, public health
4. **Works** - Infrastructure, utilities, construction
5. **Justice** - Legal system, courts, law enforcement
6. **Tourism** - BTB coordination, tourism development
7. **Agriculture** - Farming, fishing, rural development
8. **Defense** - Security, emergency response

## Integration Points

### With Other Pallets
- **pallet-belize-economy**: DALLA/bBZD token transfers, treasury funding
- **pallet-belize-compliance**: KYC/AML verification, tier checking
- **pallet-belize-identity**: BelizeID integration, identity verification
- **pallet-belize-landledger**: Land-related proposals and governance
- **pallet-belize-staking**: PoUW contribution tracking, validator governance

### With External Systems
- **Financial Services Commission (FSC)**: Regulatory compliance
- **Belize Tourism Board (BTB)**: Tourism governance
- **District Governments**: Local representation
- **Multi-sig Wallets**: Treasury security

## Usage Examples

### Submit a Proposal
```rust
BelizeGovernance::submit_proposal(
    origin,
    b"Infrastructure Upgrade".to_vec(),
    b"Renovate Belize City port facilities".to_vec(),
    0, // Constitutional proposal
    0, // Simple majority
    false // Not emergency
);
```

### Vote with Delegation
```rust
// Delegate voting power
BelizeGovernance::delegate_vote(
    origin,
    delegate_account,
    Some(block_number + 5_256_000) // 1 year
);

// Vote counts as 1 + delegated votes
BelizeGovernance::cast_vote(
    origin,
    proposal_id,
    1, // Vote yes
    3  // 3x conviction
);
```

### Department Proposal
```rust
// Finance manager submits budget proposal
BelizeGovernance::submit_department_proposal(
    origin,
    0, // Finance department
    b"Q1 Budget Allocation".to_vec(),
    b"Allocate 5M DALLA for infrastructure".to_vec(),
    0, // Standard proposal
    0, // Simple majority
    false
);
```

### District Election
```rust
// Root starts election
BelizeGovernance::start_district_election(
    RuntimeOrigin::root(),
    0, // Belize district
    3, // 3 seats
    block + 100_000, // Registration ends
    block + 200_000, // Voting starts
    block + 300_000  // Voting ends
);

// Candidate registers
BelizeGovernance::register_candidate(
    origin,
    0, // Belize district
    b"My platform: better infrastructure and education".to_vec()
);

// Citizens vote
BelizeGovernance::vote_in_district_election(
    origin,
    0, // Belize district
    candidate_account
);

// Root finalizes (assigns seats to top N)
BelizeGovernance::finalize_district_election(
    RuntimeOrigin::root(),
    0 // Belize district
);
```

## Development Status

### Completed (7/8 Phases)
✅ Phase 1: Testing Infrastructure (32 tests)  
✅ Phase 2: Compliance Integration (36 tests)  
✅ Phase 3: Departmental Governance (52 tests)  
✅ Phase 4: Foundation Board System (68 tests)  
✅ Phase 5: Execution Layer (89 tests)  
✅ Phase 6: Council Election System (109 tests)  
✅ Phase 7: Advanced Features (137 tests)  

### In Progress
🔄 Phase 8: Documentation & Polish  
- Comprehensive Rust documentation
- Integration guides
- Performance optimization
- Security audit preparation

## Security Considerations

### Access Control
- Root-only operations: department managers, board member management, elections
- Signed operations: proposals, voting, delegation
- Compliance checks: All participation requires KYC Level 2+

### Economic Security
- Proposal deposits prevent spam (250 DALLA)
- Conviction multipliers lock tokens during voting
- Multi-sig treasury for large expenditures
- Reward caps prevent treasury depletion

### Governance Security
- Max 100 delegators per delegate (prevents centralization)
- One amendment per proposal (prevents spam)
- Time-locked amendments (before voting only)
- Emergency override requires council supermajority

### Data Integrity
- BoundedVec for all collections (prevents storage bloat)
- MaxEncodedLen compliance for all types
- Atomic execution (all-or-nothing proposal actions)
- Transparent event emission

## Performance Characteristics

### Storage Complexity
- Proposals: O(1) lookup by ID
- Votes: O(1) lookup by (proposal, voter)
- Council: O(n) iteration (n = council size, max ~20)
- Delegations: O(m) iteration (m = delegators, max 100)

### Computation Weights
- submit_proposal: 15M weight + 2 reads + 2 writes
- cast_vote: 10M weight + 3 reads + 1 write
- finalize_proposal: 25M weight + 5 reads + 3 writes
- delegate_vote: 25M weight + 2 reads + 2 writes
- execute_proposal: Variable (depends on action type)

### Network Impact
- Proposal submission: ~5KB on-chain data
- Vote: ~200 bytes on-chain data
- Delegation: ~300 bytes on-chain data
- Events: ~100-500 bytes per event

## Testing

### Test Coverage
```bash
$ cargo test -p pallet-belize-governance

test result: ok. 137 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.41s
```

### Test Categories
- **Core Governance** (32 tests): Proposals, voting, finalization
- **Compliance** (4 tests): KYC/AML enforcement
- **Departmental** (16 tests): Ministry workflows, cross-department
- **Board System** (16 tests): Roles, terms, elections
- **Execution** (21 tests): Actions, treasury, parameters
- **Elections** (20 tests): District voting, seat assignment
- **Advanced** (28 tests): Delegation, amendments, rewards, priority

## Future Enhancements

### Phase 8+ Roadmap
- [ ] Quadratic voting experiments
- [ ] Conviction-weighted delegations
- [ ] Automated reward vesting
- [ ] DAO treasury investment strategies
- [ ] Cross-chain governance (Polkadot)
- [ ] Mobile governance app integration
- [ ] Real-time analytics dashboard

## Resources

### Documentation
- [Implementation Plan](./GOVERNANCE_IMPLEMENTATION_PLAN.md)
- [Current Status](./GOVERNANCE_STATUS.md)
- [Phase Completion Reports](../../) - Root directory

### Code
- [Pallet Source](../../belizechain/pallets/governance/src/lib.rs)
- [Tests](../../belizechain/pallets/governance/src/)
- [Mock Runtime](../../belizechain/pallets/governance/src/mock.rs)

### Related Documents
- [BelizeChain Whitepaper](../whitepaper.md)
- [Architecture Overview](../architecture.md)
- [Compliance Pallet](../../belizechain/pallets/compliance/)
- [Economy Pallet](../../belizechain/pallets/economy/)

## Support

### For Technical Issues
- Review test suite for usage examples
- Check phase completion reports for feature details
- Consult implementation plan for design rationale

### For Governance Questions
- See usage examples above
- Review district/department structures
- Check participation requirements

---

**Status**: 87.5% Complete (Phase 7/8)  
**Last Updated**: October 6, 2025  
**Maintainer**: BelizeChain Development Team  
**License**: See repository root
