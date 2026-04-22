# Deploying Smart Contracts - Complete Guide

Step-by-step guide to building, testing, and deploying ink! smart contracts on BelizeChain.

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or WSL2 (Windows)
- **Rust**: 1.75 or later
- **Node.js**: 18+ (for SDK usage)
- **Memory**: Minimum 4GB RAM
- **Disk Space**: 10GB available

### Install Rust & ink! Tools

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install cargo-contract (ink! CLI)
cargo install cargo-contract --locked --version 3.2.0

# Verify installation
cargo contract --version
# cargo-contract-contract 3.2.0-unknown-x86_64-unknown-linux-gnu
```

### Install Polkadot.js Extension

Browser extension for account management:
- **Chrome**: [Polkadot.js Extension](https://chrome.google.com/webstore/detail/polkadot%7Bjs%7D-extension/mopnmbcafieddcagagdcbnhejhlodfdd)
- **Firefox**: [Polkadot.js Extension](https://addons.mozilla.org/en-US/firefox/addon/polkadot-js-extension/)

## Development Workflow

### 1. Create New Contract

```bash
# Generate contract from template
cargo contract new my_token

cd my_token
```

**Generated Structure:**
```
my_token/
├── Cargo.toml          # Dependencies
├── lib.rs              # Contract code
└── .gitignore
```

### 2. Write Contract Code

Edit `lib.rs`:

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod my_token {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct MyToken {
        total_supply: u128,
        balances: Mapping<AccountId, u128>,
    }

    impl MyToken {
        #[ink(constructor)]
        pub fn new(total_supply: u128) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);

            Self {
                total_supply,
                balances,
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            self.balances.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: u128) -> Result<(), Error> {
            let from = self.env().caller();
            let from_balance = self.balance_of(from);
            
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of(to);
            self.balances.insert(to, &(to_balance + value));

            Ok(())
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InsufficientBalance,
    }
}
```

### 3. Build Contract

```bash
# Build in debug mode (faster)
cargo contract build

# Build in release mode (optimized, smaller WASM)
cargo contract build --release
```

**Output Files** (in `target/ink/`):
- `my_token.wasm` - Contract bytecode (~15-50 KB)
- `my_token.json` - Contract metadata (ABI)
- `my_token.contract` - Bundled package (.wasm + .json)

**Build Stats:**
```
Original wasm size: 45.2K, Optimized: 22.8K

The contract was built in RELEASE mode.

Your contract artifacts are ready. You can find them in:
/path/to/my_token/target/ink

  - my_token.contract (code + metadata)
  - my_token.wasm (the contract's code)
  - my_token.json (the contract's metadata)
```

### 4. Run Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn new_works() {
        let token = MyToken::new(1000);
        assert_eq!(token.total_supply(), 1000);
    }

    #[ink::test]
    fn transfer_works() {
        let mut token = MyToken::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

        assert_eq!(token.transfer(accounts.bob, 100), Ok(()));
        assert_eq!(token.balance_of(accounts.alice), 900);
        assert_eq!(token.balance_of(accounts.bob), 100);
    }

    #[ink::test]
    fn transfer_fails_insufficient_balance() {
        let mut token = MyToken::new(100);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

        assert_eq!(
            token.transfer(accounts.bob, 200),
            Err(Error::InsufficientBalance)
        );
    }
}
```

Run tests:
```bash
cargo test
```

**Expected Output:**
```
running 3 tests
test my_token::tests::new_works ... ok
test my_token::tests::transfer_works ... ok
test my_token::tests::transfer_fails_insufficient_balance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Deployment Methods

### Method 1: Upload via CLI

#### Step 1: Upload Code

```bash
cargo contract upload \
    --suri //Alice \
    --url wss://testnet.belizechain.org \
    --skip-confirm

# Output:
# Code hash: 0xabcd1234...
```

**Flags:**
- `--suri //Alice` - Development account (testnet only)
- `--url` - BelizeChain RPC endpoint
- `--skip-confirm` - Auto-confirm transaction

#### Step 2: Instantiate Contract

```bash
cargo contract instantiate \
    --suri //Alice \
    --constructor new \
    --args 1000000000000000000 \
    --url wss://testnet.belizechain.org \
    --skip-confirm

# Output:
# Contract address: 5GHkm9...Xy2pQ
```

**Constructor Args**: Encoded based on metadata
- `1000000000000000000` = 1M tokens with 12 decimals

### Method 2: Deploy via SDK

#### Install SDK

```bash
npm install @belizechain/gem-sdk @polkadot/api @polkadot/api-contract
```

#### Deploy Script

Create `deploy.js`:

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');
const { Keyring } = require('@polkadot/api');
const fs = require('fs');

async function deploy() {
    // Connect to BelizeChain testnet
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    // Load contract artifacts
    const wasm = fs.readFileSync('./target/ink/my_token.wasm');
    const metadata = JSON.parse(fs.readFileSync('./target/ink/my_token.json'));

    // Create deployer account
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    console.log('Deploying contract...');

    // Deploy contract
    const { address, result } = await sdk.deployContract(
        wasm,
        metadata,
        {
            constructor: 'new',
            args: ['1000000000000000000'], // 1M tokens
        },
        alice,
        {
            gasLimit: 100000000000,
            storageDepositLimit: null, // Auto-calculate
        }
    );

    console.log(`Contract deployed at: ${address}`);
    console.log(`Transaction hash: ${result.txHash}`);

    // Save deployment info
    fs.writeFileSync('deployment.json', JSON.stringify({
        address,
        network: 'testnet',
        deployer: alice.address,
        timestamp: new Date().toISOString(),
    }, null, 2));

    await sdk.disconnect();
}

deploy().catch(console.error);
```

Run deployment:
```bash
node deploy.js
```

### Method 3: Contracts UI (Web Interface)

1. **Open Contracts UI**: [https://contracts-ui.substrate.io/](https://contracts-ui.substrate.io/)
2. **Connect to BelizeChain**:
   - Click "Add New Network"
   - RPC URL: `wss://testnet.belizechain.org`
   - Save
3. **Upload Contract**:
   - Click "Upload New Contract"
   - Select `my_token.contract` file
   - Sign transaction with Polkadot.js extension
4. **Instantiate**:
   - Choose constructor: `new`
   - Enter args: `1000000000000000000`
   - Set gas limit: 100000 (auto-calculated)
   - Submit transaction

## Post-Deployment

### Verify Deployment

```bash
# Query contract storage
cargo contract call \
    --contract 5GHkm9...Xy2pQ \
    --message total_supply \
    --url wss://testnet.belizechain.org \
    --dry-run

# Output:
# Result: 1000000000000000000
```

### Interact with Contract

```javascript
const { address } = require('./deployment.json');

// Call read-only method
const totalSupply = await sdk.callContract(
    address,
    'total_supply',
    [],
    alice,
    { readOnly: true }
);

console.log(`Total supply: ${totalSupply}`);

// Execute transaction
const result = await sdk.callContract(
    address,
    'transfer',
    [bob.address, '100000000000000'], // 100 tokens
    alice
);

console.log(`Transfer tx: ${result.txHash}`);
```

## Gas & Storage Costs

### Gas Estimation

```bash
# Dry run to estimate gas
cargo contract call \
    --contract 5GHkm9...Xy2pQ \
    --message transfer \
    --args 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY 100000000000000 \
    --url wss://testnet.belizechain.org \
    --suri //Alice \
    --dry-run

# Output:
# Gas required: 15234 units
# Storage deposit: 0 DALLA (no new storage)
```

### Cost Breakdown

| Operation | Gas Cost | DALLA Cost |
|-----------|----------|------------|
| Contract Upload | ~500,000 | 0.0005 DALLA |
| Instantiation | ~100,000 | 0.0001 DALLA |
| PSP22 Transfer | ~15,000 | 0.000015 DALLA |
| PSP34 Mint | ~45,000 | 0.000045 DALLA |
| Storage (1 KB) | - | 100 DALLA deposit* |

*Refundable on deletion

## Network Endpoints

| Network | RPC URL | Explorer | Faucet |
|---------|---------|----------|--------|
| **Testnet** | `wss://testnet.belizechain.org` | [explorer.belizechain.org/testnet](https://explorer.belizechain.org/testnet) | [faucet.belizechain.org](https://faucet.belizechain.org) |
| **Local Dev** | `ws://localhost:9944` | N/A | Use `//Alice` account |

## Troubleshooting

### Build Errors

**Error**: `wasm32-unknown-unknown target not found`
```bash
rustup target add wasm32-unknown-unknown
```

**Error**: `cargo-contract not found`
```bash
cargo install cargo-contract --locked --force
```

### Deployment Errors

**Error**: `InsufficientBalance`
- Get testnet tokens: [faucet.belizechain.org](https://faucet.belizechain.org)
- Min 1000 DALLA needed for deployment

**Error**: `ContractTrapped`
- Check constructor args match metadata
- Review contract logic for panics
- Run `cargo test` to verify

**Error**: `StorageDepositLimitExhausted`
- Increase `--storage-deposit-limit`
- Optimize contract size (remove debug code)

## Best Practices

### 1. Optimize Contract Size

```bash
# Use release mode
cargo contract build --release

# Strip unnecessary code
cargo contract build --release -- -Z strip=symbols

# Expected sizes:
# - Debug: 50-100 KB
# - Release: 15-30 KB
```

### 2. Version Contracts

```toml
# Cargo.toml
[package]
name = "my_token"
version = "1.0.0"
```

Tag in git:
```bash
git tag -a <release-tag> -m "Release"
git push origin <release-tag>
```

### 3. Audit Before Mainnet

- Run `cargo clippy` for lints
- Complete test coverage (>80%)
- External audit for financial contracts
- Bug bounty program

### 4. Document Deployment

```json
{
  "contract": "MyToken",
  "version": "1.0.0",
  "network": "testnet",
  "address": "5GHkm9...Xy2pQ",
  "codeHash": "0xabcd1234...",
  "deployer": "5GrwvaEF...",
  "timestamp": "2026-01-31T10:30:00Z",
  "constructor": {
    "name": "new",
    "args": ["1000000000000000000"]
  }
}
```

## Next Steps

- **Test on Testnet**: Deploy to `testnet.belizechain.org`
- **Integrate with Maya Wallet**: Add contract to wallet UI
- **Submit to Explorer**: Register contract on [explorer.belizechain.org](https://explorer.belizechain.org)
- **Apply for Audit**: [security@belizechain.org](mailto:security@belizechain.org)
- **Mainnet Deployment**: After audit approval

## Resources

- **ink! Documentation**: [use.ink](https://use.ink)
- **cargo-contract Guide**: [github.com/paritytech/cargo-contract](https://github.com/paritytech/cargo-contract)
- **GEM SDK**: [GEM SDK Reference](gem-sdk-reference.md)
- **Example Contracts**: [github.com/BelizeChain/gem](https://github.com/BelizeChain/gem)
