# BelizeX Decentralized Exchange (DEX) Pallet

## Overview

BelizeX is BelizeChain's native Automated Market Maker (AMM) decentralized exchange, enabling permissionless trading of DALLA, bBZD, and other digital assets. Unlike traditional order book exchanges, BelizeX uses constant product formulas (x*y=k) to enable instant swaps without requiring matching buy/sell orders.

**Core Innovation**: Tourism-optimized trading with reduced fees, real-time Oracle price verification for bBZD peg protection, and native support for Belize's dual-currency economy (DALLA crypto + bBZD stablecoin).

### Key Features

- **Automated Market Maker (AMM)**: Constant product formula (x*y=k) for decentralized liquidity pools
- **Dual-Currency Support**: Native DALLA/bBZD pairs + TourismDALLA + WUSDC
- **Tourism Incentives**: Reduced trading fees (50% discount) for verified tourism merchants
- **Liquidity Provision**: Earn LP tokens representing pool ownership shares
- **Real-Time Oracle Integration**: bBZD/USD price verification to prevent peg manipulation
- **Cross-Border Remittance**: Optimized WUSDC/bBZD pairs for international money transfers
- **Volume-Based Discounts**: 4-tier system (0-3) with lower fees for high-volume traders
- **KYC-Gated Trading**: All traders require KYC Level 1+ (prevent wash trading, compliance)
- **Protocol Revenue**: Treasury collects configurable portion of trading fees
- **Emergency Pause**: Global circuit breaker for security incidents

### Business Use Cases

1. **Tourism Merchants**: Hotel accepts DALLA, swaps to bBZD for payroll (0.15% fee instead of 0.3%)
2. **Cross-Border Workers**: Swap WUSDC to bBZD (lower fees than traditional banks)
3. **Liquidity Providers**: Earn fees proportional to pool share (passive income)
4. **Government Operations**: Swap DALLA rewards to bBZD for stable accounting
5. **DeFi Integration**: Foundation for lending, derivatives, yield farming

## Architecture

### Trading Flow
```
User Request → KYC Check → Fee Calculation → Oracle Verification (bBZD/WUSDC only) 
→ AMM Swap (x*y=k) → Fee Distribution (Treasury + LPs) → LP Token Update → Event Emission
```

### Components

1. **Liquidity Pools**: Hold paired asset reserves (e.g., DALLA/bBZD)
2. **LP Tokens**: Represent proportional ownership of pool (tradeable, redeemable)
3. **Fee Router**: Distributes fees between Treasury (protocol) and LPs (incentive)
4. **Oracle Price Guard**: Prevents bBZD trades beyond ±500 bps of 1:1 BZD peg
5. **Volume Tracker**: Monitors account trading volume for tier upgrades
6. **Tourism Verification**: Integrates with Oracle for merchant status checks

### Constant Product Formula
```
x * y = k (before swap)
x' * y' = k (after swap, where k remains constant)

Fee = input_amount * fee_rate / 10000
Tourism discount = fee * tourism_discount_rate / 10000
Final fee = base_fee - tourism_discount - volume_discount

Output = (input_after_fee * y) / (x + input_after_fee)
```

### Asset Types
```rust
pub enum AssetId {
    DALLA,          // Native token (12 decimals)
    BBZD,           // BZD-pegged stablecoin (12 decimals, 1:1 peg)
    TourismDALLA,   // Tourism-incentivized DALLA variant
    WUSDC,          // Wrapped USDC (cross-border remittance)
}
```

## Storage

### TradingPairs
- **Type**: `StorageMap<(u8, u8), TradingPair>`
- **Key**: `(base_asset_id, quote_asset_id)` as u8 tuple
- **Purpose**: Registry of active trading pairs with reserve balances
- **Data**:
  - `base_asset: AssetId` - First asset in pair
  - `quote_asset: AssetId` - Second asset in pair
  - `base_reserve: u128` - Total base asset in pool
  - `quote_reserve: u128` - Total quote asset in pool
  - `total_lp_tokens: u128` - Total LP tokens issued
  - `fee_rate: u32` - Trading fee in basis points (default 30 = 0.3%)
  - `active: bool` - Whether pair is enabled for trading

### LiquidityProviders
- **Type**: `StorageMap<AccountId, LiquidityProvider>`
- **Purpose**: Track LP positions and rewards for each account
- **Data**:
  - `provider: AccountId` - LP account
  - `total_value_locked: u128` - Total value deposited across all pairs
  - `rewards_earned: u128` - Total fees claimed (historical tracking)
  - `is_tourism_provider: bool` - Eligible for enhanced rewards (future)

### TourismTraders
- **Type**: `StorageMap<AccountId, bool>`
- **Purpose**: Whitelist of accounts eligible for tourism fee discounts
- **Access**: Updated by `TourismOrigin` (governance/council)

### LiquidityPosition
- **Type**: Embedded in `LiquidityProvider` struct
- **Purpose**: Per-pair LP position details
- **Data**:
  - `base_amount: u128` - Base asset deposited
  - `quote_amount: u128` - Quote asset deposited
  - `lp_tokens: u128` - LP tokens owned
  - `total_value_locked: u128` - Current USD value (if Oracle available)
  - `rewards_earned: u128` - Fees claimed from this position

### GlobalPaused
- **Type**: `StorageValue<bool>`
- **Purpose**: Emergency circuit breaker flag
- **Default**: `false`

### DexStatistics
- **Type**: Derived from storage (computed on-the-fly)
- **Purpose**: Global metrics for analytics
- **Data**:
  - `total_trades: u64` - All-time trade count
  - `total_volume_dalla: u128` - All-time DALLA volume
  - `total_fees_collected: u128` - All-time fees to Treasury
  - `active_lps: u32` - Current unique liquidity providers

## Extrinsics

### create_trading_pair
**Purpose**: Create new trading pair (governance-only)

**Parameters**:
- `base_asset: u8` - First asset (e.g., 0 = DALLA)
- `quote_asset: u8` - Second asset (e.g., 1 = bBZD)
- `fee_rate: u32` - Trading fee in basis points

**Checks**:
- Caller has `PairListingOrigin` permission (governance/sudo)
- Pair doesn't already exist
- Both assets are valid `AssetId` variants

**Effects**:
- Creates `TradingPair` storage entry
- Emits `TradingPairCreated` event

**Returns**: `Ok(())` or Error

### add_liquidity
**Purpose**: Deposit assets to pool, receive LP tokens

**Parameters**:
- `base_asset: u8` - First asset ID
- `quote_asset: u8` - Second asset ID
- `base_amount: Balance` - Amount of base asset
- `quote_amount: Balance` - Amount of quote asset
- `min_lp_tokens: u128` - Slippage protection (minimum LP tokens expected)

**Checks**:
- KYC Level 1+ verified (`Kyc::is_kyc_ok(&who)`)
- Global pause not active
- Pair exists and is active
- User has sufficient balances for both assets
- Amounts meet minimum liquidity threshold

**Effects**:
- Locks `base_amount + quote_amount` from user (using `LIQUIDITY_LOCK_ID`)
- Mints LP tokens (simplified formula: `sqrt(base + quote)`)
- Updates pool reserves
- Creates/updates `LiquidityProvider` entry
- Emits `LiquidityAdded` event

**Formula**:
```rust
lp_tokens = (base_amount + quote_amount).integer_sqrt()
// Production: More complex formula considering existing pool ratio
```

**Returns**: LP tokens minted (via event)

### remove_liquidity (PLANNED - not yet implemented)
**Purpose**: Redeem LP tokens for proportional pool assets

**Parameters**:
- `base_asset: u8` - First asset ID
- `quote_asset: u8` - Second asset ID
- `lp_tokens: u128` - Amount of LP tokens to burn
- `min_base: Balance` - Minimum base asset expected
- `min_quote: Balance` - Minimum quote asset expected

**Checks**:
- User owns sufficient LP tokens
- Pair exists and is active
- Slippage protection met

**Effects**:
- Burns LP tokens
- Unlocks proportional base/quote assets
- Updates pool reserves
- Updates `LiquidityProvider` entry

**Formula**:
```rust
user_share = lp_tokens / total_lp_supply
base_return = base_reserve * user_share
quote_return = quote_reserve * user_share
```

### execute_trade
**Purpose**: Swap assets using AMM formula (exact input)

**Parameters**:
- `base_asset: u8` - Input asset ID
- `quote_asset: u8` - Output asset ID
- `amount_in: Balance` - Exact amount of input asset
- `min_amount_out: u128` - Minimum output amount (slippage protection)
- `is_tourism_trade: bool` - Claim tourism discount (requires verification)

**Checks**:
- KYC Level 1+ verified
- Global pause not active
- Pair exists and is active
- If `is_tourism_trade == true`: Account must be in `TourismTraders` registry
- User has sufficient input balance
- Output meets `min_amount_out` requirement
- **For WUSDC/bBZD trades**: Oracle deviation check (±500 bps max from 1:1 peg)

**Effects**:
- Reserves input tokens from user
- Calculates effective fee rate:
  - Base fee: `amount_in * fee_rate / 10000` (e.g., 0.3%)
  - Tourism discount: `base_fee * 50 / 100` (50% reduction)
  - Volume tier discount: 0-25% based on `Oracle::get_trading_volume_tier(&who)`
- Splits fee:
  - `treasury_fee = fee * ProtocolFeeToTreasuryBps / 10000` (default 20%)
  - `lp_fee = fee - treasury_fee` (default 80%)
- Executes AMM swap: `output = (input_after_fee * quote_reserve) / (base_reserve + input_after_fee)`
- Transfers tokens, updates reserves
- Emits `TokensTraded` event

**Oracle Verification** (WUSDC/bBZD only):
```rust
oracle_rate = Oracle::get_crypto_exchange_rate(WUSDC, BBZD) // Returns USD/BZD * 1e6
implied_rate = (amount_out * 1e6) / amount_in
deviation_bps = |implied_rate - oracle_rate| * 10000 / oracle_rate

if deviation_bps > MaxOracleDeviationBps (500) → Reject trade
if oracle_rate unavailable → Reject trade (emit OracleRateUnavailable)
```

**Returns**: Output amount (via event)

### Helper Functions (Internal)

#### get_effective_fee_rate
```rust
fn get_effective_fee_rate(account: &AccountId, is_tourism: bool) -> u32
```
Calculates final fee rate after all discounts:
1. Base: `TradingFeeRate` (30 bps)
2. Tourism: `-50%` if verified merchant
3. Volume tier: `-0%/-10%/-15%/-25%` for tiers 0/1/2/3
4. Returns: Effective rate in basis points

#### is_verified_tourism_merchant
```rust
fn is_verified_tourism_merchant(account: &AccountId) -> bool
```
Checks if account is eligible for tourism discounts:
- First checks local `TourismTraders` storage
- Then queries `Oracle::is_tourism_merchant(account)`
- Returns: `true` if either source confirms

#### get_amount_out
```rust
fn get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128>
```
Constant product formula for swap calculation:
```rust
k = reserve_in * reserve_out
new_reserve_in = reserve_in + amount_in
new_reserve_out = k / new_reserve_in
amount_out = reserve_out - new_reserve_out
```

## Events

### TradingPairCreated
```rust
TradingPairCreated {
    base_asset: u8,
    quote_asset: u8,
}
```
Emitted when new trading pair is created by governance.

### LiquidityAdded
```rust
LiquidityAdded {
    provider: AccountId,
    pair: (u8, u8),
    base_amount: u128,
    quote_amount: u128,
    lp_tokens: u128,
}
```
Emitted when user deposits liquidity to pool.

### TokensTraded
```rust
TokensTraded {
    trader: AccountId,
    pair: (u8, u8),
    amount_in: u128,
    amount_out: u128,
    fee_paid: u128,
    is_tourism_trade: bool,
}
```
Emitted on successful swap.

### OracleGuardRejected
```rust
OracleGuardRejected {
    pair: (u8, u8),
    implied_rate: u128,
    oracle_rate: u128,
    deviation_bps: u128,
    max_allowed_bps: u32,
}
```
Emitted when trade is rejected due to excessive Oracle deviation (bBZD peg protection).

### OracleRateUnavailable
```rust
OracleRateUnavailable {
    pair: (u8, u8),
}
```
Emitted when Oracle price feed is stale/missing (trade rejected).

## Errors

- `PairAlreadyExists` - Trading pair already created
- `PairNotFound` - Trading pair doesn't exist
- `PairNotActive` - Trading pair is paused
- `InsufficientLiquidity` - Pool has insufficient reserves for swap
- `InsufficientLpTokens` - User doesn't own enough LP tokens
- `SlippageExceeded` - Output below minimum or Oracle deviation too high
- `InsufficientBalance` - User doesn't have enough tokens
- `KycRequired` - Trader not KYC Level 1+ verified
- `Unauthorized` - Caller lacks required permission (tourism discount without verification)
- `Paused` - Global emergency pause active
- `OracleRateUnavailable` - Required Oracle price feed missing/stale
- `InvalidAsset` - Asset ID doesn't match enum variants

## Integration with Other Pallets

### Identity Pallet (KYC Verification)
```rust
type Kyc: KycCheck<AccountId>
```
- **Purpose**: Verify all traders are KYC Level 1+ (prevent wash trading, sanctions compliance)
- **Method**: `is_kyc_ok(&account) -> bool`
- **Usage**: Called before every `add_liquidity` and `execute_trade`

### Oracle Pallet (Price Verification & Merchant Data)
```rust
type Oracle: BelizeXOracleProvider<AccountId>
```
- **Purpose**: 
  - Verify bBZD/WUSDC trades don't exceed ±500 bps from Oracle rate
  - Identify tourism merchants (50% fee discount)
  - Track trading volume tiers (0-25% fee discounts)
- **Methods**:
  - `get_crypto_exchange_rate(base: u8, quote: u8) -> Option<u128>` - Returns rate * 1e6
  - `is_tourism_merchant(account: &AccountId) -> bool` - Merchant verification
  - `get_trading_volume_tier(account: &AccountId) -> u8` - Volume tier (0-3)
- **Integration**: Called during `execute_trade` for fee calculation and peg protection

### Economy Pallet (DALLA/bBZD Balances)
```rust
type Currency: Currency<AccountId> + ReservableCurrency<AccountId> + LockableCurrency<AccountId>
```
- **Purpose**: Handle token transfers, locks, reserves
- **Methods**: `free_balance()`, `transfer()`, `set_lock()`, `remove_lock()`
- **Usage**: All swap/liquidity operations

### Treasury Account
```rust
type Treasury: Get<AccountId>
```
- **Purpose**: Receive protocol share of trading fees
- **Fee Split**: Configurable via `ProtocolFeeToTreasuryBps` (default 20%)
- **Integration**: Automatic transfer during swaps

## Runtime Configuration

```rust
impl pallet_belizex::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type TourismOrigin = EnsureTourismCouncil; // Or EnsureRoot
    type PairListingOrigin = EnsureRootOrHalfCouncil;
    type Treasury = TreasuryPalletId;
    type TradingFeeRate = ConstU32<30>; // 0.3% (30 basis points)
    type TourismDiscountRate = ConstU32<50>; // 50% discount (final 0.15%)
    type MinLiquidityAmount = ConstU128<10_000_000_000_000>; // 10 DALLA minimum
    type WeightInfo = ();
    type Kyc = Identity;
    type ProtocolFeeToTreasuryBps = ConstU32<2000>; // 20% to treasury, 80% to LPs
    type Oracle = Oracle;
    type MaxOracleDeviationBps = ConstU32<500>; // ±5% max deviation for bBZD/WUSDC
}
```

## Testing

### Unit Tests
```bash
cargo test -p pallet-belizex
```

Tests cover:
- AMM formula correctness (constant product)
- Fee calculation (base, tourism, volume discounts)
- Liquidity provision (deposit, LP token minting)
- Slippage protection
- Oracle deviation checks for bBZD trades
- KYC enforcement
- Emergency pause functionality

### Integration Tests
```bash
./run_integration_tests.sh belizex
```

Tests cross-pallet interactions:
- Identity → BelizeX (KYC verification before trades)
- Oracle → BelizeX (price feeds, merchant verification, volume tiers)
- Economy → BelizeX (DALLA/bBZD/WUSDC balances)
- Treasury → BelizeX (fee collection)

## Usage Example

### Scenario: Tourism Hotel Swaps DALLA to bBZD for Payroll

**Context**: Blue Horizon Hotel receives 100,000 DALLA from tourist bookings, needs to swap to bBZD for employee payroll (stable wages).

**Setup** (one-time, governance):

1. **Create DALLA/bBZD Pair**:
```rust
create_trading_pair(
    origin: EnsureRoot,
    base_asset: 0, // DALLA
    quote_asset: 1, // bBZD
    fee_rate: 30   // 0.3%
)
// Emits: TradingPairCreated { base_asset: 0, quote_asset: 1 }
```

2. **Add Tourism Merchant Whitelist**:
```rust
// Called by TourismOrigin (tourism council)
TourismTraders::insert(hotel_account, true);
```

**Liquidity Provision** (liquidity provider):

3. **LP Deposits Initial Liquidity**:
```rust
add_liquidity(
    origin: lp_account,
    base_asset: 0,  // DALLA
    quote_asset: 1, // bBZD
    base_amount: 1_000_000 * 10^12,  // 1M DALLA
    quote_amount: 1_000_000 * 10^12, // 1M bBZD (1:1 initial ratio)
    min_lp_tokens: 990_000 * 10^12   // 1% slippage tolerance
)
// Returns: ~1.414M LP tokens (sqrt(1M + 1M) * 10^12)
// Emits: LiquidityAdded { lp_tokens: ~1.414M... }
```

**Trade Execution** (hotel):

4. **Hotel Swaps DALLA for bBZD**:
```rust
execute_trade(
    origin: hotel_account,
    base_asset: 0,  // DALLA
    quote_asset: 1, // bBZD
    amount_in: 100_000 * 10^12,  // 100K DALLA
    min_amount_out: 99_000 * 10^12, // Accept 99K bBZD minimum (1% slippage)
    is_tourism_trade: true  // Claim 50% fee discount
)
```

**Fee Calculation**:
```
Base fee (0.3%): 100,000 * 0.003 = 300 DALLA
Tourism discount (50%): 300 * 0.5 = 150 DALLA savings
Final fee: 150 DALLA (0.15% effective)

Protocol share (20%): 150 * 0.2 = 30 DALLA → Treasury
LP share (80%): 150 * 0.8 = 120 DALLA → Pool (LPs earn passively)

Swap amount: 100,000 - 150 = 99,850 DALLA
```

**AMM Calculation**:
```
Before: DALLA reserve = 1,000,000, bBZD reserve = 1,000,000
k = 1,000,000 * 1,000,000 = 1e12

After adding 99,850 DALLA:
new_dalla_reserve = 1,099,850
new_bbzd_reserve = 1e12 / 1,099,850 = ~909,243 bBZD
output = 1,000,000 - 909,243 = ~90,757 bBZD

Price impact: (100,000 - 90,757) / 100,000 = ~9.2% (large trade)
```

**Oracle Verification** (automatic):
```rust
// Oracle checks bBZD/USD exchange rate
oracle_rate = 1.0 * 1e6 (1:1 peg by Central Bank)
implied_rate = (90,757 / 99,850) * 1e6 = ~909,091
deviation = |1,000,000 - 909,091| / 1,000,000 = 9.1%

if deviation > 5% → REJECT trade (emit OracleGuardRejected)
// In this example, trade would be rejected due to high price impact
// Hotel should split into smaller trades or wait for more liquidity
```

**Improved Strategy** (split trade):
```rust
// Split into 10 smaller trades of 10K DALLA each
for i in 0..10 {
    execute_trade(
        amount_in: 10_000 * 10^12,
        min_amount_out: 9_800 * 10^12,
        is_tourism_trade: true
    )
}
// Results in ~99,500 bBZD total (better price, lower slippage)
```

**Result**: 
- Hotel receives ~90,757 bBZD (single trade) OR ~99,500 bBZD (split trades)
- Hotel saved 50% on fees (tourism merchant status)
- LPs earned 120 DALLA in fees per 100K trade
- Treasury received 30 DALLA protocol revenue per trade

## Future Enhancements

### Phase 2: Limit Orders
- **Feature**: Off-chain order book with on-chain settlement
- **Status**: `OpenOrders` storage defined, execution logic pending
- **Benefit**: Better price discovery, reduced slippage for large trades
- **Timeline**: Q2 2026

### Phase 3: Concentrated Liquidity
- **Feature**: Uniswap v3-style range orders
- **Status**: Research phase
- **Benefit**: 10-100x capital efficiency for LPs
- **Timeline**: Q4 2026

### Phase 4: Advanced Routing
- **Feature**: Multi-hop path optimization (A→B→C auto-routing)
- **Status**: Basic multi-hop implemented, gas optimization needed
- **Benefit**: Better execution prices for exotic pairs
- **Timeline**: Q3 2026

### Phase 5: Governance LP Incentives
- **Feature**: Additional DALLA rewards for strategic pairs (e.g., DALLA/bBZD)
- **Status**: Planned
- **Benefit**: Deeper liquidity for critical pairs
- **Timeline**: Q3 2026

### Phase 6: Cross-Chain DEX
- **Feature**: Trade assets from Ethereum, Polkadot via Interoperability pallet
- **Status**: Planned (requires bridge upgrades)
- **Benefit**: Access to global liquidity
- **Timeline**: Q1 2027

## References

- [Uniswap v2 Whitepaper](https://uniswap.org/whitepaper.pdf) - AMM design inspiration
- [Constant Product Market Maker](https://arxiv.org/abs/2003.10001) - Mathematical foundation
- [Economy Pallet](../economy/README.md) - DALLA/bBZD token implementation
- [Oracle Pallet](../oracle/README.md) - Price feed integration
- [Identity Pallet](../identity/README.md) - KYC verification system
- [Curve Finance](https://curve.fi/) - Stablecoin-optimized AMM (future consideration)
- [Balancer](https://balancer.fi/) - Multi-asset pool design (future consideration)

## License
This pallet is part of BelizeChain and is licensed under GPL-3.0.
