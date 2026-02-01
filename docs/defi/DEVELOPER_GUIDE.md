# DeFi Developer Guide

## Quick Start

### Prerequisites

```bash
# Install Rust and ink! toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update
rustup component add rust-src
rustup target add wasm32-unknown-unknown
cargo install cargo-contract --force

# Verify installation
cargo contract --version  # Should show 4.0+
```

### Deploy Your First Contract

1. **Build the contract:**
```bash
cd contracts/defi/tokens
cargo contract build --release
```

2. **Deploy to local node:**
```bash
# Start local node in another terminal
../../target/release/belizechain-node --dev --tmp

# Deploy contract
cargo contract instantiate \
  --constructor new \
  --args "BelizeChain USD" "bBZD" 12 1000000000000 \
  --suri //Alice \
  --url ws://localhost:9944
```

3. **Interact with contract:**
```bash
# Transfer tokens
cargo contract call \
  --contract <CONTRACT_ADDRESS> \
  --message transfer \
  --args <RECIPIENT> 1000000 \
  --suri //Alice
```

## Contract Integration Examples

### Example 1: Swap Tokens via AMM

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';

async function swapTokens() {
  // Connect to node
  const wsProvider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider: wsProvider });
  
  // Load contract ABI
  const ammAbi = require('./contracts/defi/amm/metadata.json');
  const ammContract = new ContractPromise(
    api,
    ammAbi,
    'AMM_CONTRACT_ADDRESS'
  );
  
  // Execute swap: 1000 DALLA -> bBZD
  const { gasRequired, storageDeposit, result } = await ammContract.query.swap(
    aliceAddress,
    {
      value: 0,
      gasLimit: -1,
    },
    0,              // amount0_out (0 = buying token1)
    100_000_000,    // amount1_out (0.1 bBZD)
    bobAddress,     // recipient
    100             // max slippage 1%
  );
  
  // Sign and send transaction
  const tx = ammContract.tx.swap(
    { gasLimit: gasRequired },
    0,
    100_000_000,
    bobAddress,
    100
  );
  
  await tx.signAndSend(alice, (result) => {
    if (result.status.isInBlock) {
      console.log('Swap confirmed in block', result.status.asInBlock.toHex());
    }
  });
}
```

### Example 2: Provide Liquidity

```typescript
async function provideLiquidity() {
  const ammContract = new ContractPromise(api, ammAbi, ammAddress);
  
  // Approve AMM to spend tokens
  await token0Contract.tx.approve(
    { gasLimit: -1 },
    ammAddress,
    1_000_000_000_000  // 1000 DALLA
  ).signAndSend(alice);
  
  await token1Contract.tx.approve(
    { gasLimit: -1 },
    ammAddress,
    1_000_000_000_000  // 1000 bBZD
  ).signAndSend(alice);
  
  // Add liquidity
  await ammContract.tx.addLiquidity(
    { gasLimit: -1 },
    1_000_000_000_000,  // amount0
    1_000_000_000_000,  // amount1
    aliceAddress        // LP tokens recipient
  ).signAndSend(alice);
  
  console.log('Liquidity added! LP tokens minted.');
}
```

### Example 3: Borrow Against Collateral

```typescript
async function borrowWithCollateral() {
  const lendingContract = new ContractPromise(api, lendingAbi, lendingAddress);
  
  // Step 1: Supply collateral (DALLA)
  await lendingContract.tx.supply(
    { gasLimit: -1 },
    dallaTokenAddress,
    10_000_000_000_000  // 10,000 DALLA
  ).signAndSend(alice);
  
  console.log('Collateral supplied');
  
  // Step 2: Borrow stablecoin (bBZD)
  // Can borrow up to 66% of collateral value (150% collateralization)
  await lendingContract.tx.borrow(
    { gasLimit: -1 },
    bBZDTokenAddress,
    5_000_000_000_000   // 5,000 bBZD
  ).signAndSend(alice);
  
  console.log('Borrowed bBZD');
  
  // Step 3: Check health factor
  const { output } = await lendingContract.query.calculateHealthFactor(
    aliceAddress,
    { gasLimit: -1 }
  );
  
  console.log('Health Factor:', output.toHuman());
  // Should be >1.0 (e.g., 1.5 = 150%)
}
```

### Example 4: Bridge Assets from Ethereum

```typescript
async function bridgeFromEthereum() {
  const bridgeContract = new ContractPromise(api, bridgeAbi, bridgeAddress);
  
  // Validator signatures from off-chain relayer
  const signatures = await fetchValidatorSignatures(
    ethereumTxHash,
    userAddress,
    amount
  );
  
  // Deposit bridged tokens
  await bridgeContract.tx.deposit(
    { gasLimit: -1 },
    aliceAddress,                      // recipient
    [0x12, 0x34, ...],                // ERC-20 address (20 bytes)
    1_000_000_000_000,                // amount
    signatures                         // validator signatures
  ).signAndSend(validatorAccount);
  
  console.log('Tokens bridged from Ethereum!');
}
```

## Frontend Integration with React

### Setup

```bash
npm install @polkadot/api @polkadot/api-contract @polkadot/extension-dapp
```

### React Hook for Contract Interaction

```typescript
import { useState, useEffect } from 'react';
import { web3FromSource } from '@polkadot/extension-dapp';
import { ContractPromise } from '@polkadot/api-contract';

export function useContract(api, abi, address) {
  const [contract, setContract] = useState(null);
  
  useEffect(() => {
    if (api && abi && address) {
      setContract(new ContractPromise(api, abi, address));
    }
  }, [api, abi, address]);
  
  const call = async (method, args, signer) => {
    const injector = await web3FromSource(signer.meta.source);
    
    const { gasRequired } = await contract.query[method](
      signer.address,
      { gasLimit: -1 },
      ...args
    );
    
    return contract.tx[method](
      { gasLimit: gasRequired },
      ...args
    ).signAndSend(signer.address, { signer: injector.signer });
  };
  
  return { contract, call };
}
```

### Example Component

```typescript
import React, { useState } from 'react';
import { useContract } from './hooks/useContract';

export function SwapComponent({ api, account }) {
  const [amount, setAmount] = useState('');
  const { contract, call } = useContract(api, ammAbi, ammAddress);
  
  const handleSwap = async () => {
    try {
      await call('swap', [
        0,                    // amount0_out
        parseAmount(amount),  // amount1_out
        account.address,      // recipient
        100                   // 1% slippage
      ], account);
      
      alert('Swap successful!');
    } catch (error) {
      console.error('Swap failed:', error);
    }
  };
  
  return (
    <div>
      <input
        type="number"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        placeholder="Amount to swap"
      />
      <button onClick={handleSwap}>Swap</button>
    </div>
  );
}
```

## API Reference

### PSP22 Token

```rust
// Transfer tokens
fn transfer(to: AccountId, value: u128) -> Result<()>

// Approve spender
fn approve(spender: AccountId, value: u128) -> Result<()>

// Transfer from
fn transfer_from(from: AccountId, to: AccountId, value: u128) -> Result<()>

// Mint (owner only)
fn mint(to: AccountId, value: u128) -> Result<()>

// Burn
fn burn(value: u128) -> Result<()>

// Queries
fn balance_of(owner: AccountId) -> u128
fn allowance(owner: AccountId, spender: AccountId) -> u128
fn total_supply() -> u128
```

### AMM Pool

```rust
// Add liquidity
fn add_liquidity(
    amount0: u128,
    amount1: u128,
    to: AccountId
) -> Result<u128>  // Returns LP tokens minted

// Remove liquidity
fn remove_liquidity(
    liquidity: u128,
    to: AccountId
) -> Result<(u128, u128)>  // Returns (amount0, amount1)

// Swap tokens
fn swap(
    amount0_out: u128,
    amount1_out: u128,
    to: AccountId,
    max_slippage_bps: u128
) -> Result<()>

// Queries
fn get_reserves() -> (u128, u128, u64)  // (reserve0, reserve1, timestamp)
fn get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128>
fn balance_of(account: AccountId) -> u128  // LP tokens
```

### Lending Protocol

```rust
// Supply collateral
fn supply(asset: AccountId, amount: u128) -> Result<()>

// Withdraw collateral
fn withdraw(asset: AccountId, amount: u128) -> Result<()>

// Borrow assets
fn borrow(asset: AccountId, amount: u128) -> Result<()>

// Repay borrowed assets
fn repay(asset: AccountId, amount: u128) -> Result<()>

// Liquidate position
fn liquidate(
    borrower: AccountId,
    borrowed_asset: AccountId,
    collateral_asset: AccountId,
    repay_amount: u128
) -> Result<()>

// Queries
fn calculate_health_factor(user: AccountId) -> Result<u128>
fn get_user_position(user: AccountId, asset: AccountId) -> Position
fn get_market_info(asset: AccountId) -> Result<Market>
```

### Ethereum Bridge

```rust
// Deposit from Ethereum
fn deposit(
    user: AccountId,
    erc20_address: [u8; 20],
    amount: u128,
    signatures: Vec<[u8; 65]>
) -> Result<()>

// Withdraw to Ethereum
fn withdraw(
    psp22_address: AccountId,
    amount: u128,
    eth_recipient: [u8; 20]
) -> Result<()>

// Admin functions (owner only)
fn whitelist_asset(erc20_address: [u8; 20], psp22_address: AccountId) -> Result<()>
fn add_validator(validator: AccountId) -> Result<()>

// Queries
fn get_tvl() -> u128
fn get_nonce(user: AccountId) -> u128
fn is_validator(account: AccountId) -> bool
```

## Economic Models

### AMM Fee Structure
- **Total swap fee**: 0.3% (30 basis points)
- **LP providers**: 0.25% (83.3% of fees)
- **Protocol treasury**: 0.05% (16.7% of fees)

**Example**: Swap 1000 DALLA
- Fee: 3 DALLA
- To LPs: 2.5 DALLA
- To treasury: 0.5 DALLA

### Lending Interest Rates

**Formula**: 
```
if utilization <= 80%:
  rate = base_rate + (utilization / optimal_utilization) * slope1
else:
  rate = rate_at_kink + ((utilization - optimal_utilization) / (100 - optimal_utilization)) * slope2
```

**Parameters**:
- Base rate: 2% APY
- Optimal utilization: 80%
- Slope 1: 8% (from 2% to 10%)
- Slope 2: 40% (from 10% to 50%)

**Example calculations**:
- 50% utilization → 6% APY
- 80% utilization → 10% APY
- 90% utilization → 30% APY

### Bridge Fees
- **Bridge fee**: 0.1% of transfer amount
- **Minimum transfer**: 100 DALLA
- **Daily limit**: 1,000,000 DALLA per user
- **Confirmation time**: 
  - Ethereum → BelizeChain: ~3 minutes (12 blocks)
  - BelizeChain → Ethereum: ~12 seconds (2 blocks)

## Security Best Practices

### For Developers

1. **Always check return values**:
```rust
let result = contract.call()?;  // ✅ Handle errors
// NOT: contract.call().unwrap()  // ❌ Can panic
```

2. **Use safe math operations**:
```rust
let sum = a.checked_add(b).ok_or(Error::Overflow)?;  // ✅
// NOT: let sum = a + b;  // ❌ Can overflow
```

3. **Validate inputs**:
```rust
if amount == 0 {
    return Err(Error::InvalidAmount);
}
```

4. **Implement reentrancy guards**:
```rust
if self.locked {
    return Err(Error::Reentrant);
}
self.locked = true;
// ... perform operations
self.locked = false;
```

### For Users

1. **Set slippage tolerance**: Use 0.5-2% for normal conditions
2. **Check health factor**: Keep >1.5 for safe borrowing
3. **Use hardware wallets**: For large amounts (Ledger/Trezor)
4. **Verify contract addresses**: Always double-check before signing
5. **Start small**: Test with small amounts first

## Troubleshooting

### Common Errors

**Error**: `InsufficientBalance`
**Solution**: Ensure you have enough tokens before calling

**Error**: `InsufficientAllowance`
**Solution**: Call `approve()` before `transfer_from()`

**Error**: `ContractPaused`
**Solution**: Wait for contract to be unpaused (emergency situation)

**Error**: `HealthFactorTooLow`
**Solution**: Add more collateral or repay some debt

**Error**: `SlippageExceeded`
**Solution**: Increase slippage tolerance or wait for better prices

## Support

- **Documentation**: https://docs.belizechain.org/defi
- **Discord**: https://discord.gg/belizechain
- **Telegram**: https://t.me/belizechain_dev
- **Email**: dev@belizechain.org
- **GitHub**: https://github.com/BelizeChain/belizechain

## Resources

- [ink! Documentation](https://use.ink/)
- [Polkadot.js API](https://polkadot.js.org/docs/)
- [Substrate Docs](https://docs.substrate.io/)
- [PSP Standards](https://github.com/w3f/PSPs)
