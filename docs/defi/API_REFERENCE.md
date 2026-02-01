# DeFi API Reference

## Table of Contents
1. [PSP22 Token Contract](#psp22-token-contract)
2. [AMM Pool Contract](#amm-pool-contract)
3. [Lending Protocol Contract](#lending-protocol-contract)
4. [Ethereum Bridge Contract](#ethereum-bridge-contract)
5. [Error Codes](#error-codes)
6. [Events Reference](#events-reference)

---

## PSP22 Token Contract

Standard fungible token implementation with extensions for pausability, minting, and burning.

### Constructor

```rust
#[ink(constructor)]
pub fn new(
    name: String,
    symbol: String,
    decimals: u8,
    initial_supply: u128
) -> Self
```

**Parameters:**
- `name`: Token name (e.g., "BelizeChain USD")
- `symbol`: Token symbol (e.g., "bBZD")
- `decimals`: Number of decimals (typically 12 for DALLA)
- `initial_supply`: Initial token supply minted to deployer

**Example:**
```typescript
const token = await deploy('PSP22Token', [
  'BelizeChain USD',
  'bBZD',
  12,
  1000000000000000000  // 1 million tokens
]);
```

### Messages (Functions)

#### transfer
```rust
#[ink(message)]
pub fn transfer(&mut self, to: AccountId, value: u128) -> Result<()>
```
Transfer tokens from caller to recipient.

**Parameters:**
- `to`: Recipient address
- `value`: Amount to transfer (in smallest unit)

**Returns:** `Ok(())` or error

**Gas:** ~100k

**Example:**
```typescript
await token.transfer(bobAddress, 1000000000000);  // 1 DALLA
```

#### approve
```rust
#[ink(message)]
pub fn approve(&mut self, spender: AccountId, value: u128) -> Result<()>
```
Allow spender to transfer tokens on behalf of caller.

**Parameters:**
- `spender`: Address allowed to spend
- `value`: Maximum amount spender can transfer

**Returns:** `Ok(())` or error

**Gas:** ~80k

**Example:**
```typescript
await token.approve(ammAddress, 1000000000000000000);  // 1M DALLA
```

#### transfer_from
```rust
#[ink(message)]
pub fn transfer_from(
    &mut self,
    from: AccountId,
    to: AccountId,
    value: u128
) -> Result<()>
```
Transfer tokens using allowance.

**Parameters:**
- `from`: Token owner address
- `to`: Recipient address
- `value`: Amount to transfer

**Returns:** `Ok(())` or error

**Gas:** ~120k

**Example:**
```typescript
await token.transferFrom(aliceAddress, bobAddress, 1000000000000);
```

#### mint
```rust
#[ink(message)]
pub fn mint(&mut self, to: AccountId, value: u128) -> Result<()>
```
Create new tokens (owner only).

**Parameters:**
- `to`: Recipient of new tokens
- `value`: Amount to mint

**Returns:** `Ok(())` or error

**Gas:** ~90k

**Errors:**
- `Unauthorized`: Caller is not owner

#### burn
```rust
#[ink(message)]
pub fn burn(&mut self, value: u128) -> Result<()>
```
Destroy tokens from caller's balance.

**Parameters:**
- `value`: Amount to burn

**Returns:** `Ok(())` or error

**Gas:** ~85k

**Errors:**
- `InsufficientBalance`: Not enough tokens to burn

#### pause / unpause
```rust
#[ink(message)]
pub fn pause(&mut self) -> Result<()>

#[ink(message)]
pub fn unpause(&mut self) -> Result<()>
```
Emergency stop mechanism (owner only).

**Gas:** ~50k each

### Query Messages

#### balance_of
```rust
#[ink(message)]
pub fn balance_of(&self, owner: AccountId) -> u128
```
Get token balance of account.

**Gas:** ~10k

#### allowance
```rust
#[ink(message)]
pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128
```
Get approved spending amount.

**Gas:** ~10k

#### total_supply
```rust
#[ink(message)]
pub fn total_supply(&self) -> u128
```
Get total token supply.

**Gas:** ~5k

---

## AMM Pool Contract

Automated market maker using constant product formula (x * y = k).

### Constructor

```rust
#[ink(constructor)]
pub fn new(token0: AccountId, token1: AccountId) -> Self
```

**Parameters:**
- `token0`: First token address (typically DALLA)
- `token1`: Second token address (typically bBZD)

### Messages

#### add_liquidity
```rust
#[ink(message)]
pub fn add_liquidity(
    &mut self,
    amount0: u128,
    amount1: u128,
    to: AccountId
) -> Result<u128>
```
Add tokens to pool and receive LP tokens.

**Parameters:**
- `amount0`: Token0 amount to add
- `amount1`: Token1 amount to add
- `to`: LP token recipient

**Returns:** LP tokens minted

**Gas:** ~500k

**Formula (first liquidity):**
```
liquidity = sqrt(amount0 * amount1)
```

**Formula (subsequent liquidity):**
```
liquidity = min(
  (amount0 * total_supply) / reserve0,
  (amount1 * total_supply) / reserve1
)
```

**Example:**
```typescript
// First liquidity provision
const lpTokens = await amm.addLiquidity(
  1000_000_000_000_000,  // 1000 DALLA
  1000_000_000_000_000,  // 1000 bBZD
  aliceAddress
);
console.log('LP tokens received:', lpTokens);
```

#### remove_liquidity
```rust
#[ink(message)]
pub fn remove_liquidity(
    &mut self,
    liquidity: u128,
    to: AccountId
) -> Result<(u128, u128)>
```
Burn LP tokens and receive underlying assets.

**Parameters:**
- `liquidity`: LP tokens to burn
- `to`: Token recipient

**Returns:** `(amount0, amount1)` withdrawn

**Gas:** ~450k

**Formula:**
```
amount0 = (liquidity * reserve0) / total_supply
amount1 = (liquidity * reserve1) / total_supply
```

**Example:**
```typescript
const [amount0, amount1] = await amm.removeLiquidity(
  50_000_000_000_000,  // 50 LP tokens
  aliceAddress
);
```

#### swap
```rust
#[ink(message)]
pub fn swap(
    &mut self,
    amount0_out: u128,
    amount1_out: u128,
    to: AccountId,
    max_slippage_bps: u128
) -> Result<()>
```
Exchange tokens using constant product.

**Parameters:**
- `amount0_out`: Token0 to receive (0 if buying token1)
- `amount1_out`: Token1 to receive (0 if buying token0)
- `to`: Token recipient
- `max_slippage_bps`: Maximum slippage in basis points (e.g., 100 = 1%)

**Gas:** ~500k

**Fee:** 0.3% of input amount
- 0.25% to liquidity providers
- 0.05% to protocol treasury

**Formula:**
```
amount_in_with_fee = amount_in * 997 / 1000
amount_out = (reserve_out * amount_in_with_fee) / (reserve_in + amount_in_with_fee)

// Constant product check
(reserve0 - amount0_out) * (reserve1 - amount1_out) >= reserve0 * reserve1
```

**Example:**
```typescript
// Swap 100 DALLA for bBZD
await amm.swap(
  0,                    // Not receiving token0
  98_000_000_000_000,  // Expecting ~98 bBZD
  aliceAddress,
  100                   // 1% slippage tolerance
);
```

#### get_amount_out
```rust
#[ink(message)]
pub fn get_amount_out(
    &self,
    amount_in: u128,
    reserve_in: u128,
    reserve_out: u128
) -> Result<u128>
```
Calculate output amount for given input.

**Gas:** ~15k

**Example:**
```typescript
const amountOut = await amm.getAmountOut(
  100_000_000_000_000,  // 100 DALLA in
  reserve0,              // DALLA reserve
  reserve1               // bBZD reserve
);
console.log('Will receive:', amountOut, 'bBZD');
```

### Query Messages

#### get_reserves
```rust
#[ink(message)]
pub fn get_reserves(&self) -> (u128, u128, u64)
```
Get current pool reserves and last update timestamp.

**Returns:** `(reserve0, reserve1, block_timestamp)`

**Gas:** ~10k

---

## Lending Protocol Contract

Over-collateralized lending with dynamic interest rates.

### Constructor

```rust
#[ink(constructor)]
pub fn new(oracle: AccountId) -> Self
```

**Parameters:**
- `oracle`: Merchant verification oracle address

### Messages

#### add_market
```rust
#[ink(message)]
pub fn add_market(
    &mut self,
    asset: AccountId,
    collateral_factor_bps: u128,
    reserve_factor_bps: u128
) -> Result<()>
```
Add new lending market (owner only).

**Parameters:**
- `asset`: Token address
- `collateral_factor_bps`: Borrowing power (5000-8000 = 50-80%)
- `reserve_factor_bps`: Protocol fee (1000 = 10%)

**Gas:** ~150k

**Example:**
```typescript
// Add DALLA market with 66% LTV
await lending.addMarket(
  dallaAddress,
  6666,   // 66.66% collateral factor
  1000    // 10% reserve factor
);
```

#### supply
```rust
#[ink(message)]
pub fn supply(&mut self, asset: AccountId, amount: u128) -> Result<()>
```
Deposit assets to earn interest.

**Parameters:**
- `asset`: Token to deposit
- `amount`: Deposit amount

**Gas:** ~800k

**Interest earned:** Based on utilization rate

**Example:**
```typescript
await lending.supply(dallaAddress, 10_000_000_000_000_000);  // 10k DALLA
```

#### withdraw
```rust
#[ink(message)]
pub fn withdraw(&mut self, asset: AccountId, amount: u128) -> Result<()>
```
Withdraw deposited assets.

**Parameters:**
- `asset`: Token to withdraw
- `amount`: Withdrawal amount

**Gas:** ~750k

**Errors:**
- `InsufficientLiquidity`: Pool doesn't have enough tokens
- `HealthFactorTooLow`: Would make position unhealthy

#### borrow
```rust
#[ink(message)]
pub fn borrow(&mut self, asset: AccountId, amount: u128) -> Result<()>
```
Borrow assets against collateral.

**Parameters:**
- `asset`: Token to borrow
- `amount`: Borrow amount

**Gas:** ~900k

**Requirements:**
- Health factor must remain >1.0 after borrow
- Must have sufficient collateral

**Collateralization:**
- Minimum: 150% (collateral_factor = 66.66%)

**Example:**
```typescript
// Borrow 5k DALLA against other assets
await lending.borrow(dallaAddress, 5_000_000_000_000_000);
```
- Liquidation threshold: 120%

**Interest rate:** Dynamic based on utilization

**Example:**
```typescript
// Borrow 5k bBZD against 10k DALLA collateral
await lending.borrow(bBZDAddress, 5_000_000_000_000_000);
```

#### repay
```rust
#[ink(message)]
pub fn repay(&mut self, asset: AccountId, amount: u128) -> Result<()>
```
Repay borrowed amount (with interest).

**Parameters:**
- `asset`: Borrowed token
- `amount`: Repayment amount

**Gas:** ~850k

#### liquidate
```rust
#[ink(message)]
pub fn liquidate(
    &mut self,
    borrower: AccountId,
    borrowed_asset: AccountId,
    collateral_asset: AccountId,
    repay_amount: u128
) -> Result<()>
```
Liquidate unhealthy position.

**Parameters:**
- `borrower`: Position to liquidate
- `borrowed_asset`: Asset being repaid
- `collateral_asset`: Collateral to seize
- `repay_amount`: Amount to repay

**Gas:** ~1.2M

**Requirements:**
- Borrower health factor <0.8 (120% collateralization)
- Liquidator must have tokens to repay

**Reward:** 10% liquidation penalty to liquidator

**Example:**
```typescript
// Liquidate unhealthy position
await lending.liquidate(
  bobAddress,             // Borrower
  bBZDAddress,            // Borrowed asset
  dallaAddress,           // Collateral
  1_000_000_000_000_000  // Repay 1k bBZD
);
// Receives ~1.1k DALLA (1k + 10% penalty)
```

### Query Messages

#### calculate_health_factor
```rust
#[ink(message)]
pub fn calculate_health_factor(&self, user: AccountId) -> Result<u128>
```
Calculate position health (1.0 = 100%, represented as 1_000_000_000_000).

**Formula:**
```
health_factor = (collateral_value * collateral_factor) / borrowed_value
```

**Interpretation:**
- `>1.5` (150%): Very safe
- `1.0-1.5`: Safe
- `0.8-1.0` (80-100%): Warning
- `<0.8` (80%): Liquidatable

**Gas:** ~50k

**Example:**
```typescript
const healthFactor = await lending.calculateHealthFactor(aliceAddress);
console.log('Health:', healthFactor / 1_000_000_000_000);  // 1.5 = 150%
```

#### get_user_position
```rust
#[ink(message)]
pub fn get_user_position(
    &self,
    user: AccountId,
    asset: AccountId
) -> (u128, u128, u128)
```
Get user's position for specific asset.

**Returns:** `(supplied, borrowed, borrow_index)`

**Gas:** ~20k

#### get_market_info
```rust
#[ink(message)]
pub fn get_market_info(&self, asset: AccountId) -> Result<Market>
```
Get market statistics.

**Returns:**
```rust
struct Market {
    total_supply: u128,
    total_borrows: u128,
    collateral_factor_bps: u128,
    reserve_factor_bps: u128,
    borrow_index: u128,
    last_update: u64,
}
```

**Gas:** ~25k

---

## Ethereum Bridge Contract

Cross-chain asset bridge with multi-sig security.

### Constructor

```rust
#[ink(constructor)]
pub fn new() -> Self
```

### Messages

#### whitelist_asset
```rust
#[ink(message)]
pub fn whitelist_asset(
    &mut self,
    erc20_address: [u8; 20],
    psp22_address: AccountId
) -> Result<()>
```
Add supported token pair (owner only).

**Parameters:**
- `erc20_address`: Ethereum ERC-20 address (20 bytes)
- `psp22_address`: BelizeChain PSP22 address

**Gas:** ~100k

**Example:**
```typescript
await bridge.whitelistAsset(
  '0x1234567890123456789012345678901234567890',  // USDC on Ethereum
  bBZDAddress                                      // bBZD on BelizeChain
);
```

#### add_validator / remove_validator
```rust
#[ink(message)]
pub fn add_validator(&mut self, validator: AccountId) -> Result<()>

#[ink(message)]
pub fn remove_validator(&mut self, validator: AccountId) -> Result<()>
```
Manage multi-sig validator set (owner only).

**Gas:** ~80k each

**Example:**
```typescript
await bridge.addValidator(validatorAddress);
```

#### deposit
```rust
#[ink(message)]
pub fn deposit(
    &mut self,
    user: AccountId,
    erc20_address: [u8; 20],
    amount: u128,
    signatures: Vec<[u8; 65]>
) -> Result<()>
```
Mint PSP22 tokens after Ethereum deposit (validator call).

**Parameters:**
- `user`: Recipient on BelizeChain
- `erc20_address`: Source ERC-20 address
- `amount`: Amount bridged
- `signatures`: Multi-sig signatures (requires 3 of 5)

**Gas:** ~1.2M

**Fee:** 0.1% to treasury

**Requirements:**
- Asset must be whitelisted
- 3 of 5 validator signatures required
- User's nonce must match
- Daily limit not exceeded (1M DALLA)

**Example:**
```typescript
// Called by validator after detecting Ethereum lock event
await bridge.deposit(
  aliceAddress,
  ethUSDCAddress,
  1_000_000_000_000_000,  // 1000 USDC
  [sig1, sig2, sig3]       // 3 validator signatures
);
```

#### withdraw
```rust
#[ink(message)]
pub fn withdraw(
    &mut self,
    psp22_address: AccountId,
    amount: u128,
    eth_recipient: [u8; 20]
) -> Result<()>
```
Burn PSP22 tokens to unlock on Ethereum (user call).

**Parameters:**
- `psp22_address`: Token to bridge
- `amount`: Amount to bridge
- `eth_recipient`: Ethereum recipient address

**Gas:** ~900k

**Fee:** 0.1% to treasury

**Process:**
1. User calls withdraw (burns PSP22)
2. Validators detect burn event
3. Validators unlock ERC-20 on Ethereum (3 of 5 multi-sig)

**Example:**
```typescript
await bridge.withdraw(
  bBZDAddress,
  1_000_000_000_000_000,                          // 1000 bBZD
  '0x1234567890123456789012345678901234567890'   // ETH address
);
```

### Query Messages

#### get_tvl
```rust
#[ink(message)]
pub fn get_tvl(&self) -> u128
```
Get total value locked in bridge.

**Gas:** ~10k

#### get_nonce
```rust
#[ink(message)]
pub fn get_nonce(&self, user: AccountId) -> u128
```
Get user's current nonce (for replay protection).

**Gas:** ~10k

#### get_daily_transfers
```rust
#[ink(message)]
pub fn get_daily_transfers(&self, user: AccountId) -> u128
```
Get user's transfer volume today.

**Gas:** ~15k

#### is_validator
```rust
#[ink(message)]
pub fn is_validator(&self, account: AccountId) -> bool
```
Check if account is validator.

**Gas:** ~10k

---

## Error Codes

### Common Errors (All Contracts)

| Error | Code | Description |
|-------|------|-------------|
| `InsufficientBalance` | 1000 | Not enough tokens in account |
| `InsufficientAllowance` | 1001 | Approval amount too low |
| `Unauthorized` | 1002 | Caller lacks permission |
| `ContractPaused` | 1003 | Contract is paused |
| `Overflow` | 1004 | Arithmetic overflow |
| `Underflow` | 1005 | Arithmetic underflow |
| `InvalidAmount` | 1006 | Amount is zero or invalid |

### AMM Errors

| Error | Code | Description |
|-------|------|-------------|
| `InsufficientLiquidity` | 2000 | Pool has insufficient reserves |
| `InsufficientInputAmount` | 2001 | Input too small |
| `InsufficientOutputAmount` | 2002 | Output too small |
| `SlippageExceeded` | 2003 | Price moved beyond tolerance |
| `InvalidKValue` | 2004 | Constant product not preserved |

### Lending Errors

| Error | Code | Description |
|-------|------|-------------|
| `HealthFactorTooLow` | 3000 | Position would be unhealthy |
| `PositionNotLiquidatable` | 3001 | Health factor too high |
| `MarketNotFound` | 3002 | Asset market doesn't exist |
| `InsufficientCollateral` | 3003 | Not enough collateral |

### Bridge Errors

| Error | Code | Description |
|-------|------|-------------|
| `AssetNotWhitelisted` | 4000 | Token pair not supported |
| `InvalidSignatures` | 4001 | Insufficient/invalid signatures |
| `DailyLimitExceeded` | 4002 | User exceeded daily transfer limit |
| `InvalidNonce` | 4003 | Replay attack detected |

---

## Events Reference

### PSP22 Token Events

```rust
#[ink(event)]
pub struct Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    value: u128,
}

#[ink(event)]
pub struct Approval {
    owner: AccountId,
    spender: AccountId,
    value: u128,
}
```

### AMM Events

```rust
#[ink(event)]
pub struct Swap {
    sender: AccountId,
    amount0_in: u128,
    amount1_in: u128,
    amount0_out: u128,
    amount1_out: u128,
    to: AccountId,
}

#[ink(event)]
pub struct LiquidityAdded {
    provider: AccountId,
    amount0: u128,
    amount1: u128,
    liquidity: u128,
}

#[ink(event)]
pub struct LiquidityRemoved {
    provider: AccountId,
    amount0: u128,
    amount1: u128,
    liquidity: u128,
}
```

### Lending Events

```rust
#[ink(event)]
pub struct Supply {
    user: AccountId,
    asset: AccountId,
    amount: u128,
}

#[ink(event)]
pub struct Borrow {
    user: AccountId,
    asset: AccountId,
    amount: u128,
}

#[ink(event)]
pub struct Liquidation {
    liquidator: AccountId,
    borrower: AccountId,
    borrowed_asset: AccountId,
    collateral_asset: AccountId,
    repay_amount: u128,
    collateral_seized: u128,
}
```

### Bridge Events

```rust
#[ink(event)]
pub struct Deposit {
    user: AccountId,
    erc20_address: [u8; 20],
    psp22_address: AccountId,
    amount: u128,
    nonce: u128,
}

#[ink(event)]
pub struct Withdrawal {
    user: AccountId,
    psp22_address: AccountId,
    erc20_address: [u8; 20],
    amount: u128,
    eth_recipient: [u8; 20],
}
```

---

## Rate Limits & Quotas

| Resource | Limit |
|----------|-------|
| Bridge daily transfer per user | 1M DALLA |
| Maximum supply per transaction | No limit (gas constrained) |
| Maximum borrow per transaction | No limit (health factor constrained) |
| AMM minimum liquidity lock | 1000 tokens (first provider only) |
| Transaction timeout | 5 minutes (300 seconds) |

---

## Gas Optimization Tips

1. **Batch operations**: Use `multicall` for multiple transactions
2. **Approve once**: Set high allowance to avoid repeated approvals
3. **Query off-chain**: Use view functions before transactions
4. **Use events**: Subscribe to events instead of polling
5. **Estimate gas**: Always call `query` first to estimate gas needed

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
