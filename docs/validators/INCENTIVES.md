# 💰 BelizeChain Validator Incentive Structure

**Version**: 1.0.0  
**Updated**: November 4, 2025  
**Effective Date**: Mainnet Launch  

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Reward Components](#reward-components)
3. [Inflation & Distribution](#inflation--distribution)
4. [Commission Structure](#commission-structure)
5. [Slashing Penalties](#slashing-penalties)
6. [Performance Bonuses](#performance-bonuses)
7. [Nominator Mechanics](#nominator-mechanics)
8. [Reward Calculation Examples](#reward-calculation-examples)

---

## Overview

BelizeChain implements a sophisticated incentive system to reward validators and nominators while ensuring network security and performance.

### Key Principles

✅ **Inflationary Rewards**: New DALLA tokens minted annually  
✅ **Transaction Fees**: Distributed to validators/nominators  
✅ **Performance-Based**: Higher uptime = higher rewards  
✅ **Slashing**: Penalties for misbehavior  
✅ **Governance Rights**: Validators have privileged voting power  

---

## Reward Components

### 1. Staking Rewards (Primary)

**Source**: Inflationary DALLA minting  
**Annual Rate**: 10% of total supply → validator pool  
**Distribution**: Based on stake + performance  

```
Total Supply: 1,000,000,000 DALLA
Annual Inflation: 100,000,000 DALLA
Validator Pool: 100,000,000 DALLA/year
```

### 2. Transaction Fees

**Source**: Network transaction fees  
**Burn Rate**: 80% burned (deflationary pressure)  
**Validator Share**: 20% distributed to validators  

```
Example:
Daily Transactions: 1,000,000
Avg Fee: 0.01 DALLA
Daily Fees: 10,000 DALLA

Burned: 8,000 DALLA (80%)
Validator Pool: 2,000 DALLA (20%)
```

### 3. Governance Rewards (Future)

**Source**: Treasury allocations  
**Criteria**: Active governance participation  
**Amount**: 1,000-5,000 DALLA per quarter  

---

## Inflation & Distribution

### Annual Inflation Schedule

| Year | Inflation Rate | New DALLA | Validator Pool | Notes |
|------|---------------|-----------|----------------|-------|
| **1** | 10% | 100M | 100M | Genesis year |
| **2** | 9% | 99M | 99M | -1% reduction |
| **3** | 8% | 96.5M | 96.5M | -1% reduction |
| **4** | 7% | 93.8M | 93.8M | -1% reduction |
| **5+** | 5% | Variable | Variable | Stabilized |

### Distribution Model

```
Validator Pool (100M DALLA/year)
│
├─ Active Validators (21-100 slots)
│  ├─ Validator Commission (0-100%)
│  └─ Nominator Rewards (remaining)
│
└─ Era-based distribution (every 24 hours)
```

### Validator Selection

- **Active Set Size**: 21 validators initially, expanding to 100
- **Selection Criteria**: Total stake (self + nominators)
- **Waiting Pool**: Validators not in active set earn NO rewards

---

## Commission Structure

### How Commission Works

Validators set a commission rate (0-100%) that determines their share of rewards BEFORE distribution to nominators.

```
Example: 10% Commission

Total Era Rewards: 10,000 DALLA
Validator Commission: 1,000 DALLA (10%)
Nominator Pool: 9,000 DALLA (90%)
```

### Commission Limits

| Validator Tier | Max Commission (Year 1) | Max Commission (Year 2+) |
|----------------|------------------------|--------------------------|
| **Genesis** | 5% | No limit |
| **Early** | 10% | No limit |
| **Community** | No limit | No limit |

### Changing Commission

- **Cooldown**: 24 hours (1 era) before new rate applies
- **Notification**: Nominators alerted via on-chain event
- **Unstaking**: Nominators can unstake if unhappy

---

## Slashing Penalties

### Slash Events

| Offense | Severity | Slash Amount | Additional Penalty |
|---------|----------|--------------|-------------------|
| **Double-signing** | Critical | 100% of stake | Immediate ejection |
| **Unresponsiveness** | High | 10% of stake | After 10% blocks missed/era |
| **Invalid block** | Medium | 5% of stake | Per invalid block |
| **Equivocation** | Critical | 100% of stake | Permanent ban |

### Slash Distribution

- **Validator**: 100% of slashed amount lost
- **Nominators**: Slashed proportionally to stake
- **Treasury**: Slashed funds sent to treasury
- **Reporters**: 10% of slash as bounty (for reporting)

### Protection Mechanisms

1. **Nominator Pools**: Max 256 nominators per validator
2. **Chill**: Auto-disable validator after slash
3. **Governance Override**: Council can reduce/waive slashes
4. **Insurance**: Validators can offer nominator insurance (optional)

---

## Performance Bonuses

### Uptime Bonuses (Monthly)

| Uptime | Bonus Multiplier | Example (10K base reward) |
|--------|------------------|---------------------------|
| **100%** | 1.10x | 11,000 DALLA (+10%) |
| **99.9%** | 1.05x | 10,500 DALLA (+5%) |
| **99.5%** | 1.00x | 10,000 DALLA (base) |
| **99.0%** | 0.95x | 9,500 DALLA (-5%) |
| **<99%** | 0.90x | 9,000 DALLA (-10%) |

### Governance Participation

Validators earn bonus rewards for active governance:

- **Vote on >90% proposals**: +500 DALLA/quarter
- **Propose passed motions**: +1,000 DALLA per proposal
- **Serve on council**: +2,000 DALLA/month

### Community Contributions

- **Forum moderation**: 500 DALLA/month
- **Technical documentation**: 1,000-5,000 DALLA per guide
- **Bug reports**: 500-50,000 DALLA (see bug bounty)
- **Validator onboarding**: 5% of referred validator's Year 1 commission

---

## Nominator Mechanics

### How Nominating Works

1. **Choose validators** (up to 16)
2. **Bond DALLA** (minimum 100 DALLA)
3. **Earn proportional rewards** after validator commission
4. **Unbond with 28-day delay**

### Nominator Rewards Formula

```rust
nominator_reward = (validator_era_reward - validator_commission) 
                   × (nominator_stake / total_validator_stake)
```

### Example Calculation

```
Validator Stats:
- Self-stake: 100,000 DALLA
- Nominator stakes: 400,000 DALLA (total)
- Total stake: 500,000 DALLA
- Commission: 10%

Era Reward: 10,000 DALLA

Step 1: Calculate commission
Commission: 10,000 × 0.10 = 1,000 DALLA (to validator)
Nominator Pool: 10,000 - 1,000 = 9,000 DALLA

Step 2: Calculate nominator share
Your stake: 50,000 DALLA
Your share: (50,000 / 500,000) × 9,000 = 900 DALLA

Step 3: Calculate validator self-stake reward
Validator self-stake share: (100,000 / 500,000) × 9,000 = 1,800 DALLA
Validator total: 1,000 (commission) + 1,800 (stake) = 2,800 DALLA
```

### Risk Considerations

⚠️ **Slashing Risk**: If validator misbehaves, nominators are slashed proportionally  
⚠️ **Oversubscription**: Validators with >256 nominators reject new nominations  
⚠️ **Waiting Pool**: Validators not in active set earn ZERO rewards  
⚠️ **Unbonding Period**: 28 days to unlock funds (prevents chain attacks)  

---

## Reward Calculation Examples

### Scenario 1: Small Validator

```yaml
Validator Profile:
  Self-stake: 50,000 DALLA
  Nominator stakes: 150,000 DALLA
  Total stake: 200,000 DALLA
  Commission: 5%
  Uptime: 100%

Network Stats:
  Total staked: 200,000,000 DALLA (20% of supply)
  Annual rewards: 100,000,000 DALLA
  Daily rewards: 273,972 DALLA
  Validator's share: 200,000 / 200,000,000 = 0.1%

Daily Calculation:
  Base reward: 273,972 × 0.001 = 273.97 DALLA
  Uptime bonus: 273.97 × 1.10 = 301.37 DALLA
  Commission: 301.37 × 0.05 = 15.07 DALLA
  Nominator pool: 301.37 - 15.07 = 286.30 DALLA
  
  Validator total: 15.07 + (286.30 × 0.25) = 86.65 DALLA/day
  Nominator (10K stake): 286.30 × (10,000/200,000) = 14.32 DALLA/day

Annual Returns:
  Validator: 31,627 DALLA (~63% APY on 50K stake)
  Nominator: 5,227 DALLA (~52% APY on 10K stake)
```

### Scenario 2: Large Validator

```yaml
Validator Profile:
  Self-stake: 500,000 DALLA
  Nominator stakes: 2,000,000 DALLA
  Total stake: 2,500,000 DALLA
  Commission: 15%
  Uptime: 99.5%

Network Stats:
  Total staked: 200,000,000 DALLA
  Daily rewards: 273,972 DALLA
  Validator's share: 2,500,000 / 200,000,000 = 1.25%

Daily Calculation:
  Base reward: 273,972 × 0.0125 = 3,424.65 DALLA
  Uptime bonus: 3,424.65 × 1.00 = 3,424.65 DALLA (no bonus)
  Commission: 3,424.65 × 0.15 = 513.70 DALLA
  Nominator pool: 3,424.65 - 513.70 = 2,910.95 DALLA
  
  Validator total: 513.70 + (2,910.95 × 0.20) = 1,095.89 DALLA/day
  Nominator (50K stake): 2,910.95 × (50,000/2,500,000) = 58.22 DALLA/day

Annual Returns:
  Validator: 400,000 DALLA (~80% APY on 500K stake)
  Nominator: 21,250 DALLA (~42.5% APY on 50K stake)
```

### Scenario 3: Slashing Event

```yaml
Before Slash:
  Validator stake: 100,000 DALLA
  Nominator stake: 400,000 DALLA
  Total: 500,000 DALLA

Slash Event: Double-signing (100% slash)
  Validator lost: 100,000 DALLA (100%)
  Nominator lost: 400,000 DALLA (100%)
  Total slashed: 500,000 DALLA → Treasury

After Slash:
  Validator stake: 0 DALLA
  Nominator stake: 0 DALLA
  Validator status: Chilled (removed from active set)
  Nominators: Must rebond to new validator
```

---

## Optimizing Returns

### For Validators

1. **Maintain 100% uptime** (+10% bonus)
2. **Keep commission competitive** (5-10% recommended)
3. **Build reputation** (attracts more nominators)
4. **Participate in governance** (bonus rewards)
5. **Avoid oversubscription** (>256 nominators rejected)

### For Nominators

1. **Diversify across validators** (up to 16)
2. **Choose low-commission validators** (more rewards)
3. **Monitor validator performance** (uptime, slashes)
4. **Avoid oversubscribed validators** (diluted rewards)
5. **Stay informed** (Discord, forum updates)

---

## Economic Security

### Attack Cost Analysis

To control 51% of network:

```
Scenario: 100 validators, 200M DALLA staked

51% Attack Cost:
  Required stake: 102,000,000 DALLA
  
  At $10/DALLA: $1,020,000,000 (over $1 billion)
  At $50/DALLA: $5,100,000,000 (over $5 billion)

Risk vs. Reward:
  If attack succeeds: Network value crashes, attacker loses all
  If attack fails: Slashed immediately, loses 100% of stake
  
Conclusion: Economically irrational to attack
```

### Long-Term Sustainability

```yaml
Year 1:
  Inflation: 10%
  Burned fees: 8,000 DALLA/day (2.9M/year)
  Net inflation: 97.1M DALLA (~9.7%)

Year 5:
  Inflation: 5%
  Burned fees: 20,000 DALLA/day (7.3M/year)
  Net inflation: Variable (target ~3-5%)

Goal: Balance inflation with fee burn for stable supply
```

---

## Governance Integration

### Validator Privileges

- **Council Seats**: Validators can run for council (7 seats)
- **Technical Committee**: Validators can apply (3 seats)
- **Emergency Calls**: Validators can trigger fast-track votes
- **Veto Power**: Council can veto harmful proposals

### Treasury Proposals

Validators can submit proposals for:
- Network upgrades
- Marketing campaigns
- Community events
- Infrastructure grants

**Success Rate**: ~40% of proposals pass (historical)

---

## Tax Implications

⚠️ **Disclaimer**: This is NOT tax advice. Consult a tax professional.

### Belizean Validators

- **Business Income**: Validator rewards taxed as business income
- **GST**: May apply if revenue >BZD$75,000/year
- **Deductions**: Hardware, electricity, hosting costs
- **Rate**: 0-25% (progressive, depends on income)

### International Validators

- **Varies by Jurisdiction**: Consult local tax authority
- **Staking Rewards**: Often treated as income (not capital gains)
- **Slashing Losses**: May be deductible as business losses
- **Record-Keeping**: Keep detailed logs for audit purposes

---

## Next Steps

1. ✅ **Understand rewards**: [VALIDATOR_REQUIREMENTS.md](VALIDATOR_REQUIREMENTS.md)
2. ✅ **Calculate returns**: Use examples above
3. ✅ **Setup testnet node**: [TESTNET_DEPLOYMENT.md](../deployment/TESTNET_DEPLOYMENT.md)
4. ✅ **Complete onboarding**: [ONBOARDING.md](ONBOARDING.md)
5. ✅ **Start earning**: Join active set on mainnet

---

## Support

- **Discord**: https://discord.gg/belizechain (#validators)
- **Forum**: https://forum.belizechain.org/c/validators
- **Email**: validators@belizechain.org
- **Reward Calculator**: https://belizechain.org/calculator

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
