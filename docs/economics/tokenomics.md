# Tokenomics & Economic Model

**DALLA Native Token • bBZD Stablecoin • Treasury Management**

Complete economic design for BelizeChain's dual-currency system.

---

## Currency Overview

### DALLA (Native Token)
**Symbol:** DALLA  
**Decimals:** 12  
**Total Supply:** Inflationary (see model below)  
**Purpose:** Network fees, staking, governance, rewards

### bBZD (Belize Dollar Stablecoin)
**Symbol:** bBZD  
**Decimals:** 12  
**Peg:** 1:1 with Belize Dollar (BZD)  
**Backing:** 100% BZD reserves held by Central Bank  
**Stability:** Central Bank governance (NOT Oracle-based)

---

## DALLA Inflation Model

### Issuance Formula
```
Annual Inflation Rate = Base + Staking Bonus + Network Activity

Base: 3% per year
Staking Bonus: 0-2% (if staking >50% of supply)
Network Activity: 0-1% (based on transaction volume)

Maximum: 6% per year
Minimum: 3% per year
```

### Inflation Distribution
```
Validators: 60% (block rewards + transaction fees)
Treasury: 25% (governance spending)
Development Fund: 10% (ecosystem grants)
Community Rewards: 5% (PoUW bonuses)
```

### Monthly Token Release
```python
# Calculate monthly DALLA inflation
total_supply = 100_000_000  # Initial 100M DALLA

annual_inflation_rate = 0.03  # 3% base
monthly_rate = (1 + annual_inflation_rate) ** (1/12) - 1

monthly_new_dalla = total_supply * monthly_rate
# ≈ 246,600 DALLA/month at launch

# Distribution
validators = monthly_new_dalla * 0.60  # 147,960 DALLA
treasury = monthly_new_dalla * 0.25    # 61,650 DALLA
dev_fund = monthly_new_dalla * 0.10    # 24,660 DALLA
community = monthly_new_dalla * 0.05   # 12,330 DALLA
```

---

## bBZD Stablecoin Mechanism

### Minting Process
1. **Deposit BZD** to Central Bank (off-chain)
2. **Verification** by Central Bank auditors
3. **Multi-sig approval** (4-of-7 signatures)
4. **Mint bBZD** 1:1 ratio on-chain

```rust
// Central Bank mints bBZD (belizechain/pallets/economy/src/lib.rs)
pub fn mint_bbzd(
    origin: OriginFor<T>,
    account: T::AccountId,
    amount: BalanceOf<T>,
    proof: OffChainProof
) -> DispatchResult {
    ensure!(
        origin == CentralBankAccount::<T>::get(),
        Error::<T>::Unauthorized
    );
    
    // Verify off-chain BZD deposit
    Self::verify_deposit_proof(&proof)?;
    
    // Mint bBZD
    T::Currency::deposit_creating(&account, amount);
    
    Self::deposit_event(Event::bBZDMinted(account, amount));
}
```

### Redemption Process
1. **Burn bBZD** on-chain
2. **Request withdrawal** from Central Bank
3. **Receive BZD** off-chain (bank transfer)

**Redemption time:** 1-3 business days

### Stability Guarantee
- **NO price Oracle** - fixed 1:1 peg
- **100% backed** by BZD reserves
- **Quarterly audits** published on-chain
- **Emergency freeze** if reserves threatened

```javascript
// Check bBZD supply vs reserves
const supply = await api.query.economy.bBZDSupply();
const reserves = await api.query.economy.centralBankReserves();

console.log(`bBZD Supply: ${supply.toString()}`);
console.log(`BZD Reserves: ${reserves.toString()}`);
console.log(`Backing ratio: ${(reserves / supply * 100).toFixed(2)}%`);
// Should always be ≥100%
```

---

## Treasury Management

### Treasury Sources
```
Transaction Fees: 80% (20% burned)
Inflation: 25% of annual issuance
BNS Marketplace: 5% commission
Slashing: 100% of slashed stakes
```

### Treasury Balance (Est. Year 1)
```
Monthly Inflows:
- Transaction fees: ~50,000 DALLA
- Inflation allocation: ~61,650 DALLA
- BNS sales: ~5,000 DALLA
- Slashing: ~2,000 DALLA
Total: ~118,650 DALLA/month

Annual Treasury Growth: ~1.42M DALLA
```

### Multi-Sig Governance
**Treasury Council:** 7 members (4-of-7 required)

**Members:**
- Prime Minister (or designate)
- Minister of Finance
- Financial Services Commission Chair
- Central Bank Governor
- 3 District Representatives (elected)

**Spending Approval:**
1. Proposal submitted via Governance pallet
2. Public referendum (7-day voting)
3. If passed, Treasury Council signs (4-of-7)
4. Funds transferred automatically

```typescript
// Propose treasury spending
const proposal = api.tx.treasury.approveProposal(proposalId);

await api.tx.governance.propose(
  proposal,
  50_000_000_000_000_000n  // 50K DALLA
).signAndSend(proposer);

// After referendum passes, Treasury Council signs
const multiSigCall = api.tx.multisig.asMulti(
  4,  // Threshold
  treasuryCouncilMembers,
  null,
  proposal,
  false,
  10_000_000_000  // Max weight
);
```

---

## Fee Structure

### Transaction Fees
```
Base Fee: 0.01 DALLA
Per Byte: 0.0001 DALLA
Priority Fee: 0-10 DALLA (optional, for faster inclusion)

Examples:
- Simple transfer: ~0.012 DALLA ($0.0036 @ $0.30/DALLA)
- Smart contract call: ~0.05 DALLA ($0.015)
- Complex governance proposal: ~0.2 DALLA ($0.06)
```

### Fee Distribution
```
Validator: 80%
Treasury: 15%
Burn: 5%
```

### Dynamic Fee Adjustment
```python
# Fees adjust based on network congestion
current_load = pending_transactions / max_transactions
target_load = 0.5  # 50% capacity

if current_load > target_load:
    fee_multiplier *= 1.1  # Increase 10%
elif current_load < target_load:
    fee_multiplier *= 0.9  # Decrease 10%

fee_multiplier = clamp(fee_multiplier, 0.5, 5.0)
# Fees can 5x in congestion, 0.5x in quiet times
```

---

## Exchange Rate Discovery

### DALLA/USD Market
**Target:** $0.25 - $0.35 USD per DALLA  
**Mechanism:** Free market (BelizeX DEX + external exchanges)  

**Launch liquidity:**
- 1M DALLA + 300K USDC pool
- Initial price: $0.30 per DALLA

### bBZD/USD Fixed
**Rate:** 1 bBZD = 2 BZD = $1 USD (fixed by BZD peg)  
**Stability:** Central Bank maintains BZD-USD peg at 2:1

### DALLA/bBZD Market
**Expected rate:** 0.6 - 0.7 DALLA per bBZD  
**Volatility:** Moderate (driven by DALLA demand)

```javascript
// Get current exchange rates on BelizeX
const dallaUsdcPair = await api.query.belizex.tradingPairs(0);
const { reserve_a, reserve_b } = dallaUsdcPair.unwrap();

const dallaPrice = reserve_b / reserve_a;  // USDC per DALLA
console.log(`1 DALLA = $${dallaPrice.toFixed(4)} USD`);

const bBZDPrice = 0.50;  // Fixed: 1 bBZD = $0.50 (2 BZD)
const dallaBbzdRate = dallaPrice / bBZDPrice;
console.log(`1 DALLA = ${dallaBbzdRate.toFixed(4)} bBZD`);
```

---

## Supply Projections

### 10-Year DALLA Supply Forecast
```
Year | Supply (M) | Inflation | New Tokens (M)
-----|------------|-----------|----------------
2026 | 100.00     | 3.0%      | 3.00
2027 | 103.00     | 3.2%      | 3.30
2028 | 106.30     | 3.5%      | 3.72
2029 | 110.02     | 3.8%      | 4.18
2030 | 114.20     | 4.0%      | 4.57
2031 | 118.77     | 4.2%      | 4.99
2032 | 123.76     | 4.5%      | 5.57
2033 | 129.33     | 4.8%      | 6.21
2034 | 135.54     | 5.0%      | 6.78
2035 | 142.32     | 5.2%      | 7.40
```

**Note:** Inflation rate increases with network adoption (more staking, more activity)

### bBZD Supply (Demand-Driven)
```
Projection based on Belize financial sector:

Year | bBZD Supply | % of BZD M2
-----|-------------|-------------
2026 | 10M         | 1%
2027 | 50M         | 5%
2028 | 150M        | 15%
2029 | 300M        | 30%
2030 | 500M        | 50%

Total BZD M2: ~$1B ($2B BZD)
```

---

## Token Utility

### DALLA Use Cases
1. **Transaction fees** (all blockchain operations)
2. **Staking** (validator bonds, minimum 1,000 DALLA)
3. **Governance** (voting power: 1 DALLA = 1 vote)
4. **Smart contract gas** (GEM platform)
5. **BNS domain registration** (100-1,000 DALLA/year)
6. **Collateral** (DeFi lending on BelizeX)
7. **Tourism spending** (earn 5-8% cashback in bBZD)

### bBZD Use Cases
1. **Stable payments** (no volatility risk)
2. **Merchant acceptance** (prefer stable currency)
3. **Cashback redemption** (tourism rewards paid in bBZD)
4. **Cross-border remittances** (stable BZD peg)
5. **Government payments** (salaries, benefits in bBZD)
6. **Savings** (hold stable value on-chain)

---

## Related Documentation

- [Staking Rewards](./staking-rewards.md)
- [Tourism Cashback](./tourism-cashback.md)
- [Treasury Governance](../governance/treasury.md)
- [BelizeX Trading](../user-guides/belizex-trading.md)
- [Economy Pallet API](../developer-guides/pallet-apis-core.md#economy-pallet)
