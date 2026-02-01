# Incentive Systems

**Staking Rewards • Tourism Cashback • Governance Participation**

Comprehensive guide to BelizeChain's economic incentive mechanisms.

---

## Staking Rewards (Proof of Useful Work)

### Validator Rewards

#### Block Production Rewards
```
Base Reward: 10 DALLA per block
Block Time: 6 seconds
Blocks per day: 14,400

Daily validator earnings (solo):
- Block rewards: 144,000 DALLA
- Transaction fees: ~5,000 DALLA
Total: ~149,000 DALLA/day

Annual (solo validator): ~54.4M DALLA
```

**Distribution among validators:**
- 21 active validators (Aura consensus)
- Rewards split proportionally by stake

```python
# Calculate individual validator rewards
total_stake = 10_000_000  # DALLA staked across all validators
validator_stake = 500_000  # Your stake

daily_rewards = 149_000  # Total daily rewards
your_share = (validator_stake / total_stake) * daily_rewards
# = 7,450 DALLA/day

annual_return = (your_share * 365) / validator_stake
# = 544% APY (early network, high rewards)
```

#### Proof of Useful Work (PoUW) Bonuses

**Federated Learning (Nawal integration):**
```rust
// Scoring formula
total_score = (quality * 0.4) + (timeliness * 0.3) + (honesty * 0.3)

// Quality: Model accuracy improvement
quality = (new_accuracy - baseline_accuracy) / baseline_accuracy * 100
// Range: 0-100

// Timeliness: Submission speed
timeliness = 100 - (submission_time / deadline * 100)
// Range: 0-100

// Honesty: Privacy verification
honesty = differential_privacy_verified ? 100 : 0
// Binary: 0 or 100
```

**Rewards:**
```
Base PoUW: 50 DALLA per training session
Quality bonus (>90% score): +100 DALLA
Timeliness bonus (top 10%): +100 DALLA
Perfect score bonus (100%): +100 DALLA
Maximum: 350 DALLA per session

Sessions per week: ~7
Weekly PoUW earnings: 350-2,450 DALLA
Annual PoUW bonus: ~127K DALLA
```

**Quantum Work (Kinich integration):**
```
Base PQW reward: 50 DALLA
Difficulty multiplier:
- Simple problems (10 qubits): 1x → 50 DALLA
- Medium problems (20 qubits): 2x → 100 DALLA
- Complex problems (30+ qubits): 4x → 200 DALLA

Weekly quantum jobs: ~10
Weekly PQW earnings: 500-2,000 DALLA
Annual PQW bonus: ~78K DALLA
```

### Nominator Rewards

**Nominate validators instead of running your own node:**

```javascript
// Nominate up to 16 validators
await api.tx.staking.nominate([
  validator1Address,
  validator2Address,
  validator3Address,
  // ... up to 16 total
]).signAndSend(nominatorAccount);
```

**Returns:**
```
Validator commission: 5-20% (set by validator)
Nominator share: 80-95% of rewards

Example:
Stake: 10,000 DALLA
Validator APY: 544%
Commission: 10%

Your rewards:
= 10,000 × 544% × (1 - 0.10)
= 48,960 DALLA/year
= 489.6% APY
```

---

## Tourism Cashback Program

### Merchant Categories

| Category | Cashback Rate | Examples |
|----------|---------------|----------|
| **Hotels** | 8% | Resorts, guesthouses, Airbnb |
| **Tours** | 7% | Cave tubing, snorkeling, ruins |
| **Restaurants** | 6% | Dining, bars, cafes |
| **Crafts** | 5% | Souvenirs, art, local products |
| **Retail** | 0% | Grocery, pharmacies |

### How It Works

**Customer Perspective:**
1. Pay at verified merchant in DALLA
2. Receive 5-8% cashback in bBZD instantly
3. Use bBZD for stable value or convert back to DALLA

```typescript
// Customer pays for hotel (tourist scenario)
const hotelStay = 500_000_000_000_000n;  // 500 DALLA

// Pay merchant
await api.tx.balances.transferKeepAlive(
  hotelAddress,
  hotelStay
).signAndSend(touristAccount);

// Economy pallet automatically applies cashback
// Cashback = 500 DALLA × 8% = 40 DALLA worth of bBZD
// @ 0.6 DALLA/bBZD exchange rate → 66.67 bBZD credited
```

**Merchant Perspective:**
```javascript
// Register as verified tourism merchant
await api.tx.economy.registerMerchant(
  'Hotels',  // Category
  2  // KYC level (Verified required)
).signAndSend(merchantAccount);

// Oracle pallet verifies merchant (NOT price feed)
await api.tx.oracle.registerMerchantVerification(
  merchantAccount,
  'Hotels',
  verificationProof
).signAndSend(verifierAccount);
```

### Cashback Economics

**Funding source:**
- 60% from Treasury (government tourism promotion)
- 40% from transaction fees (network subsidy)

**Annual budget allocation:**
```
Tourism sector GDP: $400M USD
Target on-chain: 10% → $40M
Average cashback: 6.5%
Annual cashback cost: $2.6M

In DALLA (@ $0.30):
= $2.6M / $0.30
= 8.67M DALLA/year from treasury

Monthly: ~722K DALLA
% of inflation: ~29% of monthly treasury allocation
```

### Usage Example

**Tourist spending $1,000 USD in Belize:**

```
Day 1: Hotel (3 nights) - $450
  Pay: 1,500 DALLA (@ $0.30)
  Cashback: 8% → 120 DALLA worth → 200 bBZD

Day 2: Cave tubing tour - $150
  Pay: 500 DALLA
  Cashback: 7% → 35 DALLA worth → 58 bBZD

Day 3: Restaurant dinner - $100
  Pay: 333 DALLA
  Cashback: 6% → 20 DALLA worth → 33 bBZD

Day 4: Crafts shopping - $300
  Pay: 1,000 DALLA
  Cashback: 5% → 50 DALLA worth → 83 bBZD

Total spent: $1,000 (3,333 DALLA)
Total cashback: 374 bBZD ($187 value)
Effective savings: 18.7%
```

---

## Governance Participation Rewards

### Proposal Submission

**Deposit required:** 1,000 DALLA

**Outcomes:**
- **Approved:** Deposit refunded + 100 DALLA bonus
- **Rejected:** Deposit slashed (sent to treasury)

```rust
// Submit governance proposal
let proposal_id = Governance::propose(
    Origin::signed(proposer),
    Box::new(proposal_call),
    50_000 * DALLA  // Treasury request
)?;

// Deposit locked
Balances::reserve(&proposer, 1_000 * DALLA)?;
```

### Voting Rewards

**Voting power:** 1 DALLA staked = 1 vote

**Participation bonus:**
```
Vote on proposal: 0.1 DALLA reward
Vote on referendum: 1 DALLA reward
Voting period: 7 days

Active governance participant:
- ~4 proposals/month
- ~2 referendums/month
Monthly earnings: (4 × 0.1) + (2 × 1) = 2.4 DALLA

Small, but encourages participation!
```

### Delegation Incentives

**Delegate voting power to expert:**
```javascript
// Delegate 10,000 DALLA voting power
await api.tx.governance.delegate(
  expertAddress,
  10_000_000_000_000_000n
).signAndSend(delegator);
```

**Delegate rewards:** 0.5% of votes cast
- If expert votes 100 times: 0.5 DALLA per 100 votes × delegated amount

---

## Early Adopter Bonuses

### Network Launch (First 6 Months)

**Validator bonus:** 2x rewards
- Regular: 149K DALLA/day
- Launch: 298K DALLA/day

**Staking bonus:** 1.5x APY
- Regular: 544% APY
- Launch: 816% APY

**Rationale:** Incentivize early network security

### First BNS Domain Holders

**Tier pricing discounts:**
```
Launch pricing (first 1,000 domains):
- Standard: 50 DALLA/year (50% off)
- Premium: 500 DALLA/year (50% off)
- Verified: 250 DALLA/year (50% off)

After 1,000 domains: Regular pricing applies
```

### Liquidity Provider Rewards (BelizeX)

**Bootstrap liquidity:**
```
Month 1-3: 10,000 DALLA/month LP rewards
Month 4-6: 5,000 DALLA/month
Month 7+: Dynamic based on trading volume

Distribution:
Proportional to liquidity provided × time
```

**Example:**
```python
# Provide 50K DALLA + 25K USDC liquidity
your_liquidity = 50_000  # DALLA
total_liquidity = 500_000  # Total pool

your_share = your_liquidity / total_liquidity  # 10%

monthly_rewards = 10_000  # DALLA (Month 1)
your_rewards = monthly_rewards * your_share
# = 1,000 DALLA/month

annual_bonus = 1_000 * 12  # 12,000 DALLA
annual_return = (12_000 / 50_000) * 100
# = 24% APY (on top of trading fees!)
```

---

## Penalty System (Negative Incentives)

### Validator Slashing

**Offenses:**
- **Downtime:** >1 hour offline → 1% stake slashed
- **Double signing:** Produce conflicting blocks → 10% slashed
- **Privacy violation:** Leak FL training data → 50% slashed
- **Malicious behavior:** Attack network → 100% slashed

**Slashing destination:** 100% to treasury

### Missed Proposals

**Validator penalties:**
```
Miss 5% of blocks: Warning
Miss 10% of blocks: 0.1% stake slashed
Miss 25% of blocks: Kicked from validator set
```

### KYC Violations

**Account freeze:**
- Suspicious activity flagged
- Funds locked pending investigation
- If fraud confirmed: Assets seized to treasury

---

## Total Earnings Potential

### Full Node Operator (All Capabilities)

```
Block production: 54.4M DALLA/year
PoUW (Nawal): 127K DALLA/year
PQW (Kinich): 78K DALLA/year
Governance: 30 DALLA/year
Total: ~54.6M DALLA/year

ROI on 500K DALLA stake:
= 54.6M / 500K
= 10,920% APY

Note: Shared among 21 validators, individual
      earnings depend on total network stake
```

### Passive Nominator

```
Nominator stake: 10,000 DALLA
Validator commission: 10%
Base APY: 544%

Annual earnings: 48,960 DALLA
+ Governance participation: ~30 DALLA
Total: ~49K DALLA/year (490% APY)
```

### Tourism Merchant

```
Monthly tourist transactions: $50,000
6.5% average cashback: $3,250
Cashback funded by: Treasury (not merchant cost)

Merchant benefits:
- Attract tourists with cashback
- On-chain payment settlement
- Low fees (0.012 DALLA vs 3% credit card)
```

---

## Related Documentation

- [Tokenomics Overview](./tokenomics.md)
- [Staking Guide](../validators/staking.md)
- [Governance Voting](../governance/voting.md)
- [Tourism Ecosystem](../user-guides/tourism.md)
- [Economy Pallet API](../developer-guides/pallet-apis-core.md#economy-pallet)
