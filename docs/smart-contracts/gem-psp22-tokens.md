# PSP22 Fungible Tokens - DALLA Token Standard

PSP22 is BelizeChain's fungible token standard, equivalent to Ethereum's ERC-20. The DALLA token contract serves as the reference implementation.

## Standard Overview

PSP22 defines a common interface for fungible tokens on ink! smart contracts:

```rust
pub trait PSP22 {
    fn total_supply(&self) -> u128;
    fn balance_of(&self, owner: AccountId) -> u128;
    fn allowance(&self, owner: AccountId, spender: AccountId) -> u128;
    fn transfer(&mut self, to: AccountId, value: u128) -> Result<()>;
    fn approve(&mut self, spender: AccountId, value: u128) -> Result<()>;
    fn transfer_from(&mut self, from: AccountId, to: AccountId, value: u128) -> Result<()>;
}
```

## DALLA Token Implementation

### Contract Details

- **Symbol**: DALLA
- **Decimals**: 12 (matching native DALLA)
- **Max Supply**: 100,000,000 DALLA (100 million)
- **Initial Supply**: 21,000,000 DALLA (configurable at deployment)
- **Address**: `5GD4w5...NVsNB` (testnet)

### Key Features

#### 1. Standard Transfers
```rust
// Transfer tokens from caller to recipient
pub fn transfer(&mut self, to: AccountId, value: u128) -> Result<()>
```

**Example Usage:**
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

// Transfer 100 DALLA to Bob
await sdk.dallaTransfer(
    contractAddress,
    alice,
    bob.address,
    100_000_000_000_000 // 100 DALLA (12 decimals)
);
```

**Gas Cost**: ~15,000 units (~0.000015 DALLA)

#### 2. Allowance System
```rust
// Approve spender to transfer up to 'value' tokens
pub fn approve(&mut self, spender: AccountId, value: u128) -> Result<()>

// Transfer tokens from owner using approved allowance
pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: u128) -> Result<()>
```

**Example:**
```javascript
// Alice approves DEX to spend 1000 DALLA
await sdk.dallaApprove(
    contractAddress,
    alice,
    dexAddress,
    1000_000_000_000_000 // 1000 DALLA
);

// DEX transfers DALLA from Alice to Bob
await sdk.dallaTransferFrom(
    contractAddress,
    dex,
    alice.address,
    bob.address,
    500_000_000_000_000 // 500 DALLA
);
```

#### 3. Minting & Burning (Owner Only)
```rust
// Mint new tokens (restricted to contract owner)
pub fn mint(&mut self, to: AccountId, value: u128) -> Result<()>

// Burn tokens from account (restricted to contract owner)
pub fn burn(&mut self, from: AccountId, value: u128) -> Result<()>
```

**Security**: Only the contract owner can mint/burn, preventing unauthorized inflation.

### Events

All PSP22 operations emit events for off-chain indexing:

```rust
#[ink(event)]
pub struct Transfer {
    #[ink(topic)]
    from: Option<AccountId>,  // None for minting
    #[ink(topic)]
    to: Option<AccountId>,    // None for burning
    value: u128,
}

#[ink(event)]
pub struct Approval {
    #[ink(topic)]
    owner: AccountId,
    #[ink(topic)]
    spender: AccountId,
    value: u128,
}
```

**Querying Events:**
```javascript
// Get all transfers involving Alice
const transfers = await sdk.getPSP22Transfers(contractAddress, alice.address);

// Get approval events
const approvals = await sdk.getPSP22Approvals(contractAddress, alice.address);
```

## Error Handling

PSP22 contracts return typed errors:

```rust
pub enum Error {
    InsufficientBalance,
    InsufficientAllowance,
    InvalidRecipient,
    UnauthorizedAccess,
    ExceedsMaxSupply,
    Overflow,
}
```

**Handling Errors:**
```javascript
try {
    await sdk.dallaTransfer(address, alice, bob.address, amount);
} catch (error) {
    if (error.code === 'InsufficientBalance') {
        console.log('Not enough DALLA balance');
    } else if (error.code === 'InvalidRecipient') {
        console.log('Cannot send to zero address');
    }
}
```

## Integration Patterns

### 1. Token Wallet
```rust
#[ink::contract]
mod wallet {
    use dalla_token::DallaToken;
    
    #[ink(storage)]
    pub struct Wallet {
        dalla_contract: AccountId,
    }
    
    impl Wallet {
        #[ink(message)]
        pub fn send_dalla(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let dalla: DallaToken = ink::env::call::FromAccountId::from_account_id(
                self.dalla_contract
            );
            dalla.transfer(to, amount)
        }
    }
}
```

### 2. DEX Integration
```rust
#[ink(message)]
pub fn swap_dalla_for_bbzd(&mut self, dalla_amount: u128) -> Result<u128> {
    // 1. Transfer DALLA from user to DEX
    let dalla: DallaToken = self.get_dalla_contract();
    dalla.transfer_from(
        self.env().caller(),
        self.env().account_id(),
        dalla_amount
    )?;
    
    // 2. Calculate bBZD to give (1:1 ratio for testnet)
    let bbzd_amount = dalla_amount;
    
    // 3. Transfer bBZD to user
    let bbzd: BBZDToken = self.get_bbzd_contract();
    bbzd.transfer(self.env().caller(), bbzd_amount)?;
    
    Ok(bbzd_amount)
}
```

### 3. Staking Contract
```rust
#[ink(message)]
pub fn stake(&mut self, amount: u128) -> Result<()> {
    let caller = self.env().caller();
    
    // Transfer DALLA from user to staking contract
    let dalla: DallaToken = self.get_dalla_contract();
    dalla.transfer_from(caller, self.env().account_id(), amount)?;
    
    // Update stake record
    let current_stake = self.stakes.get(caller).unwrap_or(0);
    self.stakes.insert(caller, &(current_stake + amount));
    
    Ok(())
}
```

## Deployment Guide

### 1. Build Contract
```bash
cd gem/dalla_token
cargo contract build --release
```

Output:
- `target/ink/dalla_token.wasm` (contract bytecode)
- `target/ink/dalla_token.json` (metadata)

### 2. Deploy via CLI
```bash
cargo contract upload --suri //Alice \
    --url wss://<current-public-testnet-rpc>

cargo contract instantiate \
    --suri //Alice \
    --constructor new \
    --args 21000000000000000000 \ # 21M DALLA initial supply
    --url wss://<current-public-testnet-rpc>
```

### 3. Deploy via SDK
```javascript
const fs = require('fs');
const { GemSDK } = require('@belizechain/gem-sdk');

const sdk = new GemSDK('wss://<current-public-testnet-rpc>');
await sdk.connect();

const wasm = fs.readFileSync('dalla_token.wasm');
const metadata = JSON.parse(fs.readFileSync('dalla_token.json'));

const { address } = await sdk.deployContract(
    wasm,
    metadata,
    {
        constructor: 'new',
        args: ['21000000000000000000'], // 21M DALLA
    },
    alice
);

console.log(`DALLA token deployed at: ${address}`);
```

## Security Considerations

### 1. Reentrancy Protection
PSP22 transfers complete before external calls:
```rust
pub fn transfer(&mut self, to: AccountId, value: u128) -> Result<()> {
    // 1. Check balance
    let from = self.env().caller();
    let from_balance = self.balance_of_impl(&from);
    ensure!(from_balance >= value, Error::InsufficientBalance);
    
    // 2. Update state BEFORE external calls
    self.balances.insert(&from, &(from_balance - value));
    let to_balance = self.balance_of_impl(&to);
    self.balances.insert(&to, &(to_balance + value));
    
    // 3. Emit event (external observers notified AFTER state change)
    self.env().emit_event(Transfer { from: Some(from), to: Some(to), value });
    
    Ok(())
}
```

### 2. Overflow Protection
ink! 4.0 provides automatic overflow checks:
```rust
// This will panic on overflow in debug mode, saturate in release
let new_balance = old_balance + amount;

// Explicit checked arithmetic
let new_balance = old_balance.checked_add(amount)
    .ok_or(Error::Overflow)?;
```

### 3. Access Control
```rust
fn ensure_owner(&self) -> Result<()> {
    let caller = self.env().caller();
    if caller != self.owner {
        return Err(Error::UnauthorizedAccess);
    }
    Ok(())
}

#[ink(message)]
pub fn mint(&mut self, to: AccountId, value: u128) -> Result<()> {
    self.ensure_owner()?;
    // ... minting logic
}
```

## Gas Optimization Tips

1. **Batch Transfers**: Use `transfer_from` with a loop for multiple recipients
2. **Minimize Storage**: Pack related data into single storage slots
3. **Event Optimization**: Only index necessary fields (max 3 topics)
4. **Lazy Deletion**: Mark as deleted instead of removing from storage

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn transfer_works() {
        let mut dalla = DallaToken::new(1000);
        let accounts = ink::env::test::default_accounts::<Environment>();
        
        // Transfer 100 DALLA to Bob
        assert_eq!(dalla.transfer(accounts.bob, 100), Ok(()));
        
        // Verify balances
        assert_eq!(dalla.balance_of(accounts.alice), 900);
        assert_eq!(dalla.balance_of(accounts.bob), 100);
    }
    
    #[ink::test]
    fn approval_works() {
        let mut dalla = DallaToken::new(1000);
        let accounts = ink::env::test::default_accounts::<Environment>();
        
        // Alice approves Bob to spend 500
        assert_eq!(dalla.approve(accounts.bob, 500), Ok(()));
        assert_eq!(dalla.allowance(accounts.alice, accounts.bob), 500);
        
        // Bob transfers 200 from Alice to Charlie
        ink::env::test::set_caller::<Environment>(accounts.bob);
        assert_eq!(dalla.transfer_from(accounts.alice, accounts.charlie, 200), Ok(()));
        
        // Verify balances and remaining allowance
        assert_eq!(dalla.balance_of(accounts.alice), 800);
        assert_eq!(dalla.balance_of(accounts.charlie), 200);
        assert_eq!(dalla.allowance(accounts.alice, accounts.bob), 300);
    }
}
```

Run tests:
```bash
cargo test
```

## Resources

- **Contract Source**: [github.com/BelizeChain/gem/tree/main/dalla_token](https://github.com/BelizeChain/gem/tree/main/dalla_token)
- **PSP22 Standard**: [github.com/w3f/PSPs/blob/master/PSPs/psp-22.md](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md)
- **SDK Examples**: [GEM SDK Reference](gem-sdk-reference.md)
- **Live Contract**: `5GD4w5...NVsNB` on testnet
