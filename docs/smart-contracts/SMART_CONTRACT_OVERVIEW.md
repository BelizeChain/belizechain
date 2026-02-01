# Smart Contracts Overview

BelizeChain supports WebAssembly (Wasm) smart contracts via the **GEM Platform** (Government & Enterprise Marketplace).

## Quick Links

- [GEM Platform Overview](gem-platform.md) - Platform introduction
- [Deployment Guide](gem-deployment-guide.md) - Deploy your first contract
- [PSP22 Tokens](gem-psp22-tokens.md) - Fungible token standard
- [PSP34 NFTs](gem-psp34-nfts.md) - Non-fungible token standard
- [DAO Templates](gem-dao-templates.md) - Governance contracts
- [SDK Reference](gem-sdk-reference.md) - JavaScript/TypeScript SDK

## GEM Platform Features

### 🦀 ink! 4.0 Support
- **Language**: Rust with ink! smart contract framework
- **Runtime**: WebAssembly (Wasm) execution
- **Standards**: PSP22 (tokens), PSP34 (NFTs), PSP37 (multi-token)
- **Size Limit**: 512KB per contract

### 💰 Token Standards
- **PSP22**: ERC-20 equivalent (fungible tokens)
- **PSP34**: ERC-721 equivalent (NFTs)
- **Native Integration**: Works with DALLA and bBZD

### 🏛️ DAO Tooling
- Member management
- Proposal creation and voting
- Treasury management
- Execution mechanisms

### 🧪 Testnet Faucet
- 1,000 DALLA per claim
- Daily limit per account
- Free for developers

## Getting Started

### 1. Install cargo-contract
```bash
cargo install cargo-contract --force --locked
```

### 2. Create a New Contract
```bash
cargo contract new my_token
cd my_token
```

### 3. Write Your Contract
```rust
#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod my_token {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct MyToken {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
    }

    impl MyToken {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            
            Self {
                total_supply,
                balances,
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or(0)
        }
    }
}
```

### 4. Build & Deploy
```bash
# Build contract
cargo contract build --release

# Deploy to testnet
cargo contract instantiate \
  --constructor new \
  --args 1000000000000 \
  --suri //Alice \
  --url wss://testnet-rpc.belizechain.org

# Interact with contract
cargo contract call \
  --contract <CONTRACT_ADDRESS> \
  --message total_supply \
  --suri //Alice \
  --url wss://testnet-rpc.belizechain.org
```

## Available Templates

### PSP22 Token
```bash
# Clone PSP22 template
git clone https://github.com/BelizeChain/gem.git
cd gem/dalla_token

# Deploy
cargo contract build --release
cargo contract instantiate \
  --constructor new \
  --args 1000000000000 "MyToken" "MTK" 12
```

**Features**:
- Transfer, approve, transfer_from
- Mint and burn (optional)
- 12 decimal precision
- Compatible with DALLA/bBZD

**Guide**: [gem-psp22-tokens.md](gem-psp22-tokens.md)

### PSP34 NFT
```bash
cd gem/beli_nft

# Deploy NFT collection
cargo contract instantiate \
  --constructor new \
  --args "BelizeNFT" "BNFT"
```

**Features**:
- Mint, transfer, burn
- Metadata support (IPFS URIs)
- Enumeration (total_supply, owner_of)
- Royalty support (optional)

**Guide**: [gem-psp34-nfts.md](gem-psp34-nfts.md)

### DAO
```bash
cd gem/simple_dao

# Deploy DAO
cargo contract instantiate \
  --constructor new \
  --args 5 3600 50 # 5 members, 1hr voting, 50% quorum
```

**Features**:
- Member management (add/remove)
- Proposal submission
- Voting (yes/no/abstain)
- Automatic execution

**Guide**: [gem-dao-templates.md](gem-dao-templates.md)

## JavaScript SDK

### Installation
```bash
npm install @belizechain/gem-sdk
```

### Usage
```typescript
import { GemSDK } from '@belizechain/gem-sdk';

// Connect to testnet
const sdk = new GemSDK({
  rpc: 'wss://testnet-rpc.belizechain.org',
  signer: '//Alice'
});

// Deploy PSP22 token
const token = await sdk.deployPSP22({
  name: 'MyToken',
  symbol: 'MTK',
  decimals: 12,
  totalSupply: '1000000000000'
});

// Transfer tokens
await token.transfer(recipient, '1000000000'); // 1 MTK

// Check balance
const balance = await token.balanceOf(address);
console.log(`Balance: ${balance}`);
```

**Full Reference**: [gem-sdk-reference.md](gem-sdk-reference.md)

## Security Audit

GEM platform underwent security audit by Trail of Bits (Q3 2025):
- ✅ **0 Critical** vulnerabilities
- ✅ **1 High** (reentrancy - fixed)
- ✅ **3 Medium** (all fixed)
- ✅ **5 Low** (all addressed)

**Report**: [gem-security-audit.md](gem-security-audit.md)

## Deployment Costs

| Operation | Gas (DALLA) | USD |
|-----------|------------|-----|
| Deploy PSP22 | ~0.25 | ~$0.0375 |
| Deploy PSP34 | ~0.30 | ~$0.045 |
| Deploy DAO | ~0.35 | ~$0.0525 |
| Token Transfer | ~0.00015 | ~$0.000023 |
| NFT Mint | ~0.0003 | ~$0.000045 |

## Examples

### Tokenized Real Estate
```rust
// Represent property as PSP34 NFT
#[ink(message)]
pub fn mint_property(
    &mut self,
    property_id: String,
    ipfs_hash: String, // Property docs on Pakit
    owner: AccountId
) -> Result<()> {
    self.mint(owner, property_id.clone())?;
    self.set_metadata(property_id, ipfs_hash)?;
    Ok(())
}
```

### Tourism Vouchers
```rust
// PSP22 token for tourism discounts
#[ink(message)]
pub fn redeem_voucher(
    &mut self,
    merchant: AccountId,
    amount: Balance
) -> Result<()> {
    ensure!(self.is_verified_merchant(merchant), Error::NotVerified);
    self.burn(Self::env().caller(), amount)?;
    self.credit_merchant(merchant, amount)?;
    Ok(())
}
```

**More Examples**: [gem-examples.md](gem-examples.md)

## Resources

- **Testnet Faucet**: https://faucet.belizechain.org
- **Contract Explorer**: https://explorer.belizechain.org/contracts
- **GitHub**: https://github.com/BelizeChain/gem
- **Discord**: #smart-contracts channel
- **Documentation**: https://docs.belizechain.org/smart-contracts

## Integration with BelizeChain Pallets

Smart contracts can interact with native pallets:

### Economy Pallet
```rust
// Check DALLA balance
let balance = self.env().balance();

// Transfer native DALLA
self.env().transfer(recipient, amount)?;
```

### Identity Pallet
```rust
// Verify KYC status (requires cross-contract call)
let kyc_level = self.query_kyc_level(account)?;
ensure!(kyc_level >= 2, Error::KYCRequired);
```

### Governance Pallet
```rust
// Submit proposal from contract
self.env().emit_event(ProposalSubmitted {
    proposer: self.env().caller(),
    proposal_id,
});
```

## Statistics (January 2026)

- **Deployed Contracts**: 1,247
- **PSP22 Tokens**: 342
- **PSP34 NFTs**: 189
- **DAOs**: 67
- **Daily Transactions**: ~8,500 contract calls
- **Total Value Locked**: 450K DALLA (~$67,500 USD)

---

**See Also**:
- [GEM Platform](gem-platform.md)
- [Deployment Guide](gem-deployment-guide.md)
- [PSP22 Tokens](gem-psp22-tokens.md)
- [PSP34 NFTs](gem-psp34-nfts.md)
- [DAO Templates](gem-dao-templates.md)
- [SDK Reference](gem-sdk-reference.md)
