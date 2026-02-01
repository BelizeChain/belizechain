# Smart Contract Development

**ink! 4.0 • PSP22/PSP34 Standards • GEM Platform • Testing & Deployment**

Complete guide to developing and deploying smart contracts on BelizeChain.

---

## ink! Smart Contracts Overview

BelizeChain uses **ink! 4.0**, a Rust-based eDSL for writing Wasm smart contracts compatible with Substrate's `pallet-contracts`.

**Key features:**
- ✅ Rust safety guarantees (no buffer overflows, null pointers)
- ✅ PSP22 (fungible tokens) and PSP34 (NFTs) standards
- ✅ Low gas costs (WASM execution)
- ✅ Upgradeable contracts (via delegatecall pattern)
- ✅ Cross-contract calls

---

## Environment Setup

### Install ink! CLI

```bash
# Install cargo-contract (ink! toolchain)
cargo install cargo-contract --force --version 4.0.0

# Verify installation
cargo contract --version  # Expected: cargo-contract 4.0.0

# Install substrate-contracts-node (optional, for local testing)
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git
```

### Create New Contract

```bash
cd gem/  # GEM platform directory

# Generate new contract from template
cargo contract new my-contract
cd my-contract/

# Directory structure:
# my-contract/
# ├── Cargo.toml
# └── lib.rs
```

---

## PSP22 Token Contract (Fungible Tokens)

### Basic Implementation

**File: `gem/my-token/lib.rs`**

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod my_token {
    use ink::storage::Mapping;
    
    /// PSP22 token storage
    #[ink(storage)]
    pub struct MyToken {
        /// Total token supply
        total_supply: Balance,
        /// Account balances
        balances: Mapping<AccountId, Balance>,
        /// Allowances (owner → spender → amount)
        allowances: Mapping<(AccountId, AccountId), Balance>,
        /// Token metadata
        name: String,
        symbol: String,
        decimals: u8,
    }
    
    /// Events
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }
    
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }
    
    /// Errors
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
        ZeroAddressTransfer,
    }
    
    pub type Result<T> = core::result::Result<T, Error>;
    
    impl MyToken {
        /// Constructor: mint initial supply to creator
        #[ink(constructor)]
        pub fn new(
            name: String,
            symbol: String,
            decimals: u8,
            initial_supply: Balance
        ) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &initial_supply);
            
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
            
            Self {
                total_supply: initial_supply,
                balances,
                allowances: Default::default(),
                name,
                symbol,
                decimals,
            }
        }
        
        /// Get total supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }
        
        /// Get balance of account
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or(0)
        }
        
        /// Transfer tokens
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)?;
            Ok(())
        }
        
        /// Approve spender to transfer tokens
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            
            self.env().emit_event(Approval { owner, spender, value });
            Ok(())
        }
        
        /// Transfer from one account to another (requires allowance)
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowances.get((&from, &caller)).unwrap_or(0);
            
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            
            self.transfer_from_to(&from, &to, value)?;
            self.allowances.insert((&from, &caller), &(allowance - value));
            
            Ok(())
        }
        
        /// Internal transfer helper
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance
        ) -> Result<()> {
            let from_balance = self.balance_of(*from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of(*to);
            self.balances.insert(to, &(to_balance + value));
            
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            
            Ok(())
        }
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[ink::test]
        fn new_works() {
            let token = MyToken::new(
                "My Token".to_string(),
                "MTK".to_string(),
                18,
                1_000_000
            );
            
            assert_eq!(token.total_supply(), 1_000_000);
        }
        
        #[ink::test]
        fn transfer_works() {
            let mut token = MyToken::new(
                "My Token".to_string(),
                "MTK".to_string(),
                18,
                1_000_000
            );
            
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // Transfer 100 tokens to Bob
            assert!(token.transfer(accounts.bob, 100).is_ok());
            assert_eq!(token.balance_of(accounts.bob), 100);
            assert_eq!(token.balance_of(accounts.alice), 999_900);
        }
    }
}
```

### Build & Deploy

```bash
# Build contract
cd gem/my-token/
cargo contract build --release

# Output files:
# target/ink/my_token.contract  # Metadata + WASM (for deployment)
# target/ink/my_token.wasm       # Raw WASM
# target/ink/metadata.json       # ABI

# Deploy via GEM CLI
cd ../cli/
cargo run -- deploy \
    --contract ../my-token/target/ink/my_token.contract \
    --constructor new \
    --args "My Token" "MTK" 18 1000000 \
    --suri //Alice

# Expected output:
# ✅ Contract deployed at: 5GQX7z8Rq...
# Code hash: 0xabc123...
```

---

## PSP34 NFT Contract (Non-Fungible Tokens)

### Basic Implementation

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod my_nft {
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    
    pub type TokenId = u32;
    
    #[ink(storage)]
    pub struct MyNft {
        /// Token ID counter
        next_token_id: TokenId,
        /// Token owner (token_id → owner)
        owners: Mapping<TokenId, AccountId>,
        /// Token metadata (token_id → URI)
        token_uris: Mapping<TokenId, String>,
        /// Operator approvals (owner → operator → approved)
        operator_approvals: Mapping<(AccountId, AccountId), bool>,
        /// Collection metadata
        name: String,
        symbol: String,
    }
    
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        token_id: TokenId,
    }
    
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TokenNotFound,
        NotOwner,
        NotApproved,
        TokenExists,
    }
    
    pub type Result<T> = core::result::Result<T, Error>;
    
    impl MyNft {
        #[ink(constructor)]
        pub fn new(name: String, symbol: String) -> Self {
            Self {
                next_token_id: 1,
                owners: Default::default(),
                token_uris: Default::default(),
                operator_approvals: Default::default(),
                name,
                symbol,
            }
        }
        
        /// Mint new NFT
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, uri: String) -> Result<TokenId> {
            let token_id = self.next_token_id;
            self.next_token_id += 1;
            
            self.owners.insert(token_id, &to);
            self.token_uris.insert(token_id, &uri);
            
            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                token_id,
            });
            
            Ok(token_id)
        }
        
        /// Get owner of token
        #[ink(message)]
        pub fn owner_of(&self, token_id: TokenId) -> Result<AccountId> {
            self.owners.get(token_id).ok_or(Error::TokenNotFound)
        }
        
        /// Transfer NFT
        #[ink(message)]
        pub fn transfer(
            &mut self,
            to: AccountId,
            token_id: TokenId
        ) -> Result<()> {
            let caller = self.env().caller();
            let owner = self.owner_of(token_id)?;
            
            if caller != owner {
                return Err(Error::NotOwner);
            }
            
            self.owners.insert(token_id, &to);
            
            self.env().emit_event(Transfer {
                from: Some(owner),
                to: Some(to),
                token_id,
            });
            
            Ok(())
        }
        
        /// Get token metadata URI
        #[ink(message)]
        pub fn token_uri(&self, token_id: TokenId) -> Result<String> {
            self.token_uris.get(token_id).ok_or(Error::TokenNotFound)
        }
    }
}
```

---

## DAO Contract (Governance)

**See:** [gem/simple_dao/lib.rs](../../gem/simple_dao/lib.rs) for full implementation

**Key features:**
- Proposal submission (1,000 DALLA deposit)
- Voting (weighted by DALLA holdings)
- 7-day voting period
- Automatic execution after approval

---

## Testing Smart Contracts

### Unit Tests (Built-in)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[ink::test]
    fn transfer_works() {
        let mut contract = MyToken::new(/*...*/);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        // Set caller to Alice
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        
        // Transfer 100 tokens to Bob
        assert_ok!(contract.transfer(accounts.bob, 100));
        
        // Check balances
        assert_eq!(contract.balance_of(accounts.bob), 100);
    }
    
    #[ink::test]
    #[should_panic(expected = "InsufficientBalance")]
    fn transfer_fails_insufficient_balance() {
        let mut contract = MyToken::new(/*...*/);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        // Try to transfer more than balance
        contract.transfer(accounts.bob, 2_000_000).unwrap();
    }
}
```

**Run tests:**

```bash
cargo test
```

### E2E Tests (On-chain)

```rust
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::build_message;
    
    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    
    #[ink_e2e::test]
    async fn transfer_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Deploy contract
        let constructor = MyTokenRef::new(
            "My Token".to_string(),
            "MTK".to_string(),
            18,
            1_000_000
        );
        let contract_acc_id = client
            .instantiate("my_token", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        
        // Call transfer
        let transfer_msg = build_message::<MyTokenRef>(contract_acc_id.clone())
            .call(|contract| contract.transfer(bob(), 100));
        
        let result = client
            .call(&ink_e2e::alice(), transfer_msg, 0, None)
            .await
            .expect("transfer failed");
        
        // Check balance
        let balance_msg = build_message::<MyTokenRef>(contract_acc_id.clone())
            .call(|contract| contract.balance_of(bob()));
        
        let balance = client
            .call_dry_run(&ink_e2e::alice(), &balance_msg, 0, None)
            .await
            .return_value();
        
        assert_eq!(balance, 100);
        
        Ok(())
    }
}
```

**Run E2E tests:**

```bash
cargo test --features e2e-tests
```

---

## GEM Platform Integration

### Deploy via GEM CLI

```bash
cd gem/cli/

# Deploy token contract
cargo run -- deploy \
    --contract ../dalla_token/target/ink/dalla_token.contract \
    --constructor new \
    --args 1000000000000 \  # 1M DALLA (12 decimals)
    --suri //Alice

# Interact with contract
cargo run -- call \
    --contract 5GQX7z8... \
    --message transfer \
    --args 5FHneW... 100000000000000 \  # Transfer 100 DALLA
    --suri //Alice
```

### Deploy via UI (Maya Wallet)

```typescript
// Connect to Maya Wallet
import { ApiPromise, WsProvider } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';

async function deployContract() {
  const provider = new WsProvider('wss://rpc.belizechain.org');
  const api = await ApiPromise.create({ provider });
  
  // Load contract metadata
  const metadata = require('./my_token/target/ink/metadata.json');
  const wasm = require('fs').readFileSync('./my_token/target/ink/my_token.wasm');
  
  // Upload code
  const code = new CodePromise(api, metadata, wasm);
  
  const tx = code.tx.new({ gasLimit, storageDepositLimit },
    "My Token",  // name
    "MTK",       // symbol
    18,          // decimals
    1_000_000    // initial supply
  );
  
  await tx.signAndSend(alice, (result) => {
    if (result.status.isInBlock) {
      console.log(`Contract deployed at: ${result.contractAddress}`);
    }
  });
}
```

---

## Gas Optimization

### Estimate Gas

```bash
# Dry run to estimate gas
cargo contract call \
    --contract 5GQX7z8... \
    --message transfer \
    --args 5FHneW... 100 \
    --suri //Alice \
    --dry-run

# Output:
# Gas required: 157,834,000
# Storage deposit: 0
```

### Optimization Tips

```rust
// ❌ BAD: Using Vec (unbounded)
pub struct BadContract {
    pub items: Vec<Item>,  // Can grow infinitely
}

// ✅ GOOD: Using Mapping (constant lookup)
pub struct GoodContract {
    pub items: Mapping<u32, Item>,  // O(1) lookup
    pub item_count: u32,
}

// ❌ BAD: Iterating over all items
#[ink(message)]
pub fn sum_all(&self) -> u32 {
    self.items.iter().map(|item| item.value).sum()
}

// ✅ GOOD: Maintain running total
#[ink(storage)]
pub struct OptimizedContract {
    pub total: u32,  // Updated on each insert/remove
}
```

---

## Security Best Practices

### 1. Avoid Re-entrancy

```rust
// ❌ VULNERABLE: External call before state update
#[ink(message)]
pub fn withdraw(&mut self, amount: Balance) -> Result<()> {
    // External call BEFORE balance update
    self.env().transfer(self.env().caller(), amount)?;
    
    // Attacker can re-enter here!
    let balance = self.balances.get(caller).unwrap_or(0);
    self.balances.insert(caller, &(balance - amount));
}

// ✅ SAFE: State update before external call
#[ink(message)]
pub fn withdraw(&mut self, amount: Balance) -> Result<()> {
    let caller = self.env().caller();
    let balance = self.balances.get(caller).unwrap_or(0);
    
    // Update state FIRST
    self.balances.insert(caller, &(balance - amount));
    
    // Then external call
    self.env().transfer(caller, amount)?;
}
```

### 2. Integer Overflow Protection

```rust
// ✅ Use checked arithmetic
let new_balance = balance.checked_add(amount)
    .ok_or(Error::Overflow)?;

// ✅ Or saturating (for non-critical operations)
let new_count = count.saturating_add(1);
```

### 3. Access Control

```rust
#[ink(storage)]
pub struct SecureContract {
    owner: AccountId,
}

#[ink(message)]
pub fn admin_function(&mut self) -> Result<()> {
    let caller = self.env().caller();
    ensure!(caller == self.owner, Error::Unauthorized);
    
    // Admin logic here
}
```

---

## Related Documentation

- [GEM Platform Overview](../smart-contracts/platform-overview.md)
- [PSP22 Token Standard](../smart-contracts/psp22-tokens.md)
- [PSP34 NFT Standard](../smart-contracts/psp34-nfts.md)
- [DAO Templates](../smart-contracts/dao-templates.md)
- [GEM Faucet](../smart-contracts/faucet.md)
- [ink! Documentation](https://use.ink)
