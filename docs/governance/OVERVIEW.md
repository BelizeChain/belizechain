# Governance Overview

BelizeChain implements a multi-tiered democratic governance system for national-scale decision making.

## Quick Links

- [Democracy System](democracy.md) - Multi-tiered governance structure
- [Voting Guide](voting.md) - How to participate in governance
- [Technical Reference](../technical-reference/governance/README.md) - Detailed implementation

## Key Features

### 🏛️ District Councils
- **6 Districts**: Belize, Cayo, Corozal, Orange Walk, Stann Creek, Toledo
- **12 Council Seats**: 2 elected representatives per district
- **Term Length**: 6 months with elections
- **Responsibilities**: District proposals, local budgets, community representation

### 🗳️ Democratic Voting
- **Conviction Voting**: Lock tokens longer for increased voting power (1x-6x)
- **Quorum Requirements**: Minimum participation for proposal validity
- **Voting Periods**: 7-day standard, 3-day emergency
- **Transparency**: All votes public and on-chain

### 💰 Treasury Management
- **Multi-Sig Control**: 4-of-7 signatures required
- **Budget Proposals**: Community-submitted spending proposals
- **8M DALLA Reserve**: ~$1.2M USD treasury
- **Inflation Funded**: 20% of block rewards → treasury

### 🚨 Emergency Powers (JaguarMode)
- **Activation**: National emergencies only (hurricanes, panics)
- **3-of-7 Threshold**: Foundation Board approval
- **Temporary Powers**: Pause transactions, freeze accounts
- **Transparency**: All actions logged and reported

## Governance Tiers

### Tier 1: Citizen Participation
- Vote on proposals (10 DALLA reward per vote)
- Submit proposals (100 DALLA reward if passed)
- Participate in district forums
- Minimum: BelizeID (KYC Level 1)

### Tier 2: Council Members
- Elected representatives (500 DALLA/month)
- Submit district proposals
- Attend weekly council meetings
- Requirements: KYC Level 2 (Verified)

### Tier 3: Government Ministries
- 8 government departments with on-chain budgets
- Ministry-specific proposal powers
- Inter-ministry coordination
- Requirements: Government employment + Enhanced KYC

### Tier 4: Foundation Board
- 7 specialized roles (Executive Director, CTO, CFO, etc.)
- Strategic decision making
- Emergency JaguarMode authority
- Requirements: Board appointment + background checks

## Getting Started

### Vote on a Proposal
```bash
# Via Maya Wallet UI (easiest)
1. Open Maya Wallet → Governance tab
2. Browse active proposals
3. Click "Vote" → Select Aye/Nay
4. Choose conviction (lock period)
5. Confirm transaction

# Via Polkadot.js
const api = await ApiPromise.create({ provider });
await api.tx.governance
  .vote(proposalId, true, conviction)
  .signAndSend(account);
```

### Submit a Proposal
```javascript
// 1. Create proposal
const proposal = {
  category: 'Treasury',
  title: 'Fund Community Center',
  description: 'Allocate 5,000 DALLA for San Pedro community center',
  amount: 5000 * 1e12, // 12 decimals
  beneficiary: 'district_belize_treasury'
};

// 2. Submit on-chain (10,000 DALLA deposit)
await api.tx.governance
  .submitProposal(proposal)
  .signAndSend(account, { deposit: 10000e12 });
```

## Proposal Categories

| Category | Description | Deposit | Approval |
|----------|-------------|---------|----------|
| **Constitutional** | Runtime upgrades, pallet changes | 50,000 DALLA | 75% + 50% turnout |
| **Treasury** | Budget spending >10,000 DALLA | 10,000 DALLA | 60% + 30% turnout |
| **Infrastructure** | Technical improvements | 5,000 DALLA | 55% + 25% turnout |
| **Social** | Community programs | 1,000 DALLA | 50% + 20% turnout |
| **Emergency** | JaguarMode responses | 0 DALLA | 3-of-7 Board |

## Resources

- **Winik Portal**: https://governance.belizechain.org (governance dashboard)
- **Forums**: https://forum.belizechain.org (proposal discussions)
- **Discord**: #governance channel (community chat)
- **Council Meetings**: First Thursday each month, 7 PM BZT
- **Documentation**: [Technical Reference](../technical-reference/governance/)

## Integration

Governance connects to:
- **Identity Pallet**: KYC verification for voting rights
- **Economy Pallet**: Treasury management and proposal funding
- **Compliance Pallet**: FSC oversight and legal enforcement
- **Staking Pallet**: Validator governance participation

## Statistics (January 2026)

- **Active Voters**: 12,450 citizens (31% participation)
- **Total Proposals**: 487 submitted, 342 passed (70% approval rate)
- **Treasury Spent**: 2.1M DALLA ($315K USD) on 156 funded projects
- **Council Elections**: 3 successful elections, 72 candidates total
- **Emergency Activations**: 2 (Hurricane Dean 2024, Banking Crisis 2025)

---

**See Also**:
- [Democracy System Details](democracy.md)
- [Voting Mechanics](voting.md)
- [Technical Implementation](../technical-reference/governance/README.md)