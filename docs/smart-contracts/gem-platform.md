# GEM Smart Contracts Platform

**The Gem** is BelizeChain's smart contract execution environment powered by ink! 4.0 and `pallet-contracts`.

## Overview

The Gem enables developers to deploy WebAssembly (Wasm) smart contracts that can interact with BelizeChain's entire ecosystem:

- **💰 Economy Integration**: Direct access to DALLA/bBZD token transfers, balances, and treasury operations
- **🆔 Identity Verification**: Query BelizeID credentials (SSN, Passport, KYC status) for compliance
- **🤖 AI Capabilities**: Access Nawal federated learning predictions and genome-evolved models
- **⚛️ Quantum Computing**: Execute quantum workloads through Kinich (Azure Quantum + IBM Quantum)
- **📦 Sovereign Storage**: Store data via Pakit DAG storage with on-chain proof verification
- **🏛️ Governance Participation**: Create proposals, vote, and manage district council operations
- **🌐 Domain Services**: Register and manage .bz domains through BNS (Belize Name Service)

## Architecture

### Execution Environment

```rust
// pallet-contracts configuration in BelizeChain runtime
impl pallet_contracts::Config for Runtime {
    type Currency = Balances;
    type CallStack = [Frame<Self>; 5];
    type WeightPrice = pallet_transaction_payment::Pallet<Self>;
    type Schedule = Schedule;
    type CallFilter = Nothing;
    type DepositPerByte = DepositPerByte;
    type DepositPerItem = DepositPerItem;
    // ... additional config
}
```

**Key Features:**
- **Gas Metering**: Precise execution cost tracking with configurable limits
- **Storage Rent**: Pay-per-byte storage model with deposit mechanism
- **Sandboxed Execution**: Isolated Wasm VM with no direct system access
- **Upgradability**: Proxy pattern support for contract upgrades
- **Cross-Contract Calls**: Call other contracts and runtime pallets via chain extensions

### Supported Standards

#### PSP22 - Fungible Tokens
Standard for fungible tokens (similar to ERC-20):
- `transfer()` - Send tokens between accounts
- `approve()` - Approve spending allowance
- `transfer_from()` - Third-party transfers
- `total_supply()` - Query total token supply
- `balance_of()` - Check account balance

**Implementation**: [dalla_token/](https://github.com/BelizeChain/gem/tree/main/dalla_token)

#### PSP34 - Non-Fungible Tokens
Standard for NFTs (similar to ERC-721):
- `mint()` - Create new NFT with metadata
- `transfer()` - Transfer NFT ownership
- `owner_of()` - Query NFT owner
- `total_supply()` - Count all NFTs
- `collection_id()` - Get collection ID

**Implementation**: [beli_nft/](https://github.com/BelizeChain/gem/tree/main/beli_nft)

### Chain Extensions

GEM contracts can call BelizeChain pallets directly through chain extensions:

```rust
// Example: Query BelizeID from contract
#[ink::chain_extension]
pub trait BelizeChainExtension {
    #[ink(extension = 1001)]
    fn get_belizeid(account: AccountId) -> Option<BelizeID>;
    
    #[ink(extension = 1002)]
    fn verify_kyc(account: AccountId) -> bool;
    
    #[ink(extension = 1003)]
    fn get_dalla_balance(account: AccountId) -> Balance;
    
    #[ink(extension = 1004)]
    fn register_bns_domain(domain: Vec<u8>) -> Result<(), BNSError>;
}
```

## Development Tools

### ink! CLI
```bash
# Install ink! CLI
cargo install cargo-contract --locked

# Create new contract
cargo contract new my_contract

# Build contract (produces .wasm + metadata.json)
cargo contract build

# Run unit tests
cargo test

# Deploy to BelizeChain testnet
cargo contract upload --suri //Alice --url ws://localhost:9944

# Instantiate contract
cargo contract instantiate --suri //Alice --args "1000000000000000" \
    --constructor new --url ws://localhost:9944
```

### SDK Integration
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

const sdk = new GemSDK('wss://testnet.belizechain.org');
await sdk.connect();

// Deploy contract
const { address } = await sdk.deployContract(
    wasmCode,
    metadata,
    constructorArgs,
    signerKeyring
);

// Call contract method
const result = await sdk.callContract(
    contractAddress,
    'transfer',
    [recipientAddress, amount],
    signerKeyring
);
```

## Gas Economics

### Gas Costs (Testnet Estimates)

| Operation | Gas Cost | DALLA Equivalent |
|-----------|----------|------------------|
| PSP22 Transfer | ~15,000 | 0.000015 DALLA |
| PSP34 Mint | ~45,000 | 0.000045 DALLA |
| Storage Write (1 KB) | ~8,000 | 0.000008 DALLA |
| Cross-Contract Call | ~12,000 + callee cost | Variable |
| DAO Vote Cast | ~18,000 | 0.000018 DALLA |

**Storage Deposits:**
- **Per Byte**: 100 DALLA (refundable on deletion)
- **Per Item**: 1,000 DALLA (refundable)

## Security Model

### Contract Sandboxing
- **No Direct I/O**: Contracts cannot access filesystem or network
- **Gas Limits**: Every instruction has measured gas cost
- **Stack Depth**: Limited to 5 nested contract calls
- **Memory Bounds**: Maximum 16 MB per contract instance

### Best Practices
1. **Reentrancy Protection**: Use check-effects-interaction pattern
2. **Integer Overflow**: ink! 4.0 provides built-in overflow checks
3. **Access Control**: Implement role-based permissions (see [simple_dao](https://github.com/BelizeChain/gem/tree/main/simple_dao))
4. **Upgradeability**: Use proxy pattern for future upgrades
5. **Audits**: Submit production contracts to BelizeChain security bounty program

## Production Contracts

### DALLA Token (PSP22)
- **Address**: `5GD4w5...NVsNB` (truncated for security)
- **Supply**: 1,000,000,000 DALLA (matches on-chain native token)
- **Decimals**: 12
- **Status**: ✅ Live on testnet
- **Repository**: [dalla_token/](https://github.com/BelizeChain/gem/tree/main/dalla_token)

### BeLi NFT Collection (PSP34)
- **Address**: `5Ho6Ks...iFQL7` (truncated for security)
- **Collection**: Belizean cultural artifacts and landmarks
- **Metadata**: IPFS-hosted JSON with Pakit backup
- **Status**: ✅ Live on testnet
- **Repository**: [beli_nft/](https://github.com/BelizeChain/gem/tree/main/beli_nft)

### Simple DAO Template
- **Status**: 🟡 Built, awaiting deployment
- **Features**: Proposal creation, weighted voting, timelock execution
- **Integration**: Direct access to treasury via Economy pallet chain extension
- **Repository**: [simple_dao/](https://github.com/BelizeChain/gem/tree/main/simple_dao)

### Testnet Faucet
- **Status**: 🟡 Built, awaiting deployment
- **Drip Amount**: 1,000 DALLA per claim
- **Cooldown**: 100 blocks (~10 minutes)
- **Anti-Abuse**: Per-account cooldown tracking
- **Repository**: [faucet/](https://github.com/BelizeChain/gem/tree/main/faucet)

## Getting Started

### Prerequisites
- Rust 1.75+ with `wasm32-unknown-unknown` target
- cargo-contract 3.2+
- Node.js 18+ (for SDK)
- Polkadot.js browser extension (for testing)

### Quick Start
```bash
# 1. Clone GEM repository
git clone https://github.com/BelizeChain/gem.git
cd gem

# 2. Build example contracts
cd dalla_token
cargo contract build

# 3. Install SDK
cd ../sdk
npm install

# 4. Run examples
node examples/01-deploy-token.js
```

For detailed tutorials, see:
- [GEM Quick Start Guide](gem-quick-start.md)
- [Deploying Your First Contract](gem-deployment-guide.md)
- [Building PSP22 Tokens](gem-psp22-tokens.md)
- [Creating NFT Collections](gem-psp34-nfts.md)
- [DAO Governance Patterns](gem-dao-templates.md)

## Resources

- **GitHub**: [github.com/BelizeChain/gem](https://github.com/BelizeChain/gem)
- **SDK Documentation**: [GEM SDK Reference](gem-sdk-reference.md)
- **API Reference**: [Complete Contract APIs](gem-api-reference.md)
- **Security Audits**: [Audit Results](gem-security-audit.md)
- **Code Examples**: [GEM Examples](gem-examples.md)
- **Discord**: [discord.belizechain.org](https://discord.belizechain.org)
- **Forum**: [forum.belizechain.org/c/smart-contracts](https://forum.belizechain.org/c/smart-contracts)
