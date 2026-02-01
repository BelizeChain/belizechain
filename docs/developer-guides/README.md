# 💻 Developer Guides

**Build applications on BelizeChain**

---

## 🎯 Who This is For

- **Frontend developers** building dApps with Polkadot.js
- **Rust developers** creating custom pallets
- **Backend developers** integrating BelizeChain APIs
- **Full-stack developers** building complete solutions

**Prerequisites**:
- Basic programming knowledge (JavaScript, TypeScript, or Rust)
- Familiarity with blockchain concepts (or read [Getting Started](../getting-started/what-is-belizechain.md) first)
- Understanding of APIs and web development

---

## 📚 Developer Documentation

### 🚀 [Setup Development Environment](./setup-environment.md)
**Get your machine ready for BelizeChain development**
- Install dependencies (Node.js, Rust, tools)
- Clone BelizeChain repository
- Build blockchain locally
- Setup testing environment
- Configure IDEs (VS Code, IntelliJ)

**Time**: 1-2 hours  
**Difficulty**: ⭐⭐ Intermediate

---

### 🏗️ [Build Your First DApp](./build-dapp.md)
**Create a decentralized application on BelizeChain**
- Connect to BelizeChain from JavaScript
- Read blockchain data
- Send transactions
- Subscribe to events
- Handle wallet connections
- Deploy your dApp

**Time**: 2-3 hours  
**Difficulty**: ⭐⭐ Intermediate

**Example projects**:
- Token transfer app
- Voting dashboard
- Transaction explorer
- Balance checker

---

### 🔧 [Smart Pallets Guide](./smart-pallets.md)
**Work with BelizeChain's 9 pallets**
- Economy Pallet (DALLA, bBZD, treasury)
- Governance Pallet (proposals, voting)
- Identity Pallet (DID, attestations)
- Staking Pallet (validators, rewards)
- Compliance Pallet (KYC/AML)
- And 4 more...

**Time**: 1 hour per pallet  
**Difficulty**: ⭐⭐⭐ Advanced

---

### 📖 [API Reference](./api-reference.md)
**Complete API documentation**
- RPC endpoints
- WebSocket subscriptions
- Polkadot.js API methods
- Query storage
- Submit extrinsics
- Error handling
- Code examples

**Time**: Reference as needed  
**Difficulty**: All levels

---

### 🧪 [Testing Guide](./testing.md)
**Test your BelizeChain applications**
- Unit testing pallets
- Integration testing
- End-to-end testing
- Mock data
- Testnet usage
- CI/CD setup

**Time**: 2 hours  
**Difficulty**: ⭐⭐⭐ Advanced

---

### 🦀 [Build Custom Pallets](./custom-pallets.md)
**Create your own Substrate pallets**
- Pallet structure
- Storage design
- Extrinsics (transactions)
- Events and errors
- Weights and benchmarking
- Runtime integration

**Time**: 4-6 hours  
**Difficulty**: ⭐⭐⭐⭐ Expert

---

## 🎓 Learning Path

### 1. Beginner Developer (New to Blockchain)
**Goal**: Build simple dApp that reads blockchain data

1. Read [What is BelizeChain?](../getting-started/what-is-belizechain.md)
2. [Setup Development Environment](./setup-environment.md)
3. [Build Your First DApp](./build-dapp.md)
   - Follow "Read-Only App" tutorial
   - Display account balances
   - Show transaction history
4. [API Reference](./api-reference.md) - Bookmark for lookups

**Time**: 1 week  
**Outcome**: Working dApp that displays blockchain data

---

### 2. Intermediate Developer (Some Blockchain Experience)
**Goal**: Build interactive dApp that sends transactions

1. Complete Beginner Path ↑
2. [Build Your First DApp](./build-dapp.md)
   - Follow "Interactive App" tutorial
   - Implement wallet connections
   - Send transactions
   - Handle errors
3. [Smart Pallets Guide](./smart-pallets.md)
   - Study Economy Pallet
   - Study Governance Pallet
4. Build project:
   - Voting dashboard
   - Payment processor
   - Treasury tracker

**Time**: 2-3 weeks  
**Outcome**: Full-featured dApp with transactions

---

### 3. Advanced Developer (Ready for Blockchain Backend)
**Goal**: Build custom pallet or complex application

1. Complete Intermediate Path ↑
2. [Testing Guide](./testing.md)
   - Setup test environment
   - Write unit tests
   - Integration tests
3. [Build Custom Pallets](./custom-pallets.md) (if Rust developer)
   - Create simple pallet
   - Deploy to local chain
   - Test thoroughly
4. Build advanced project:
   - DeFi application
   - NFT marketplace
   - DAO governance tool
   - Custom blockchain feature

**Time**: 1-2 months  
**Outcome**: Production-ready application or pallet

---

## 🛠️ Development Tools

### Required Tools
- **Node.js** 18+ (JavaScript runtime)
- **npm/yarn** (Package managers)
- **Polkadot.js** (BelizeChain JavaScript API)
- **Git** (Version control)

### Optional But Recommended
- **Rust** (for pallet development)
- **Docker** (for local blockchain)
- **VS Code** (recommended IDE)
  - Polkadot extension
  - Rust Analyzer extension
- **Postman** (API testing)

### BelizeChain-Specific
- **Maya Wallet** (for testing)
- **Blue Hole Explorer** (blockchain explorer)
- **Testnet Faucet** (free test tokens)

---

## 🌐 Quick Start Projects

### Project 1: Balance Checker
**Simple read-only app**

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

async function getBalance(address: string) {
  const provider = new WsProvider('wss://rpc.belizechain.org');
  const api = await ApiPromise.create({ provider });
  
  const { data: { free } } = await api.query.system.account(address);
  console.log(`Balance: ${free.toString()} DALLA`);
  
  await api.disconnect();
}

getBalance('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
```

**Time**: 30 minutes  
**[Full Tutorial](./build-dapp.md#balance-checker)**

---

### Project 2: Transaction Sender
**Interactive app with wallet connection**

```typescript
import { web3Enable, web3Accounts, web3FromAddress } from '@polkadot/extension-dapp';

async function sendTransaction(to: string, amount: string) {
  // Enable wallet
  await web3Enable('My DApp');
  const accounts = await web3Accounts();
  const sender = accounts[0];
  
  // Create transfer
  const injector = await web3FromAddress(sender.address);
  const transfer = api.tx.balances.transfer(to, amount);
  
  // Sign and send
  await transfer.signAndSend(sender.address, { signer: injector.signer });
}
```

**Time**: 1 hour  
**[Full Tutorial](./build-dapp.md#transaction-sender)**

---

### Project 3: Governance Dashboard
**Display and vote on proposals**

```typescript
async function getProposals() {
  const proposals = await api.query.governance.proposals.entries();
  
  proposals.forEach(([key, proposal]) => {
    console.log({
      id: key.args[0].toNumber(),
      title: proposal.title.toString(),
      status: proposal.status.toString(),
      votesFor: proposal.votesFor.toNumber(),
      votesAgainst: proposal.votesAgainst.toNumber(),
    });
  });
}
```

**Time**: 2 hours  
**[Full Tutorial](./build-dapp.md#governance-dashboard)**

---

## 📖 API Resources

### RPC Endpoints
- **Mainnet**: `wss://rpc.belizechain.org`
- **Testnet**: `wss://testnet-rpc.belizechain.org`
- **Local**: `ws://localhost:9944`

### Block Explorers
- **Mainnet**: https://explorer.belizechain.org
- **Testnet**: https://testnet.explorer.belizechain.org

### Faucets
- **Testnet Faucet**: https://faucet.belizechain.org (free test tokens)

### Documentation
- **Polkadot.js Docs**: https://polkadot.js.org/docs/
- **Substrate Docs**: https://docs.substrate.io/
- **BelizeChain Specs**: [Technical Reference](../technical-reference/README.md)

---

## 💡 Best Practices

### Security
- ✅ Never commit private keys
- ✅ Use environment variables for secrets
- ✅ Validate all user input
- ✅ Handle errors gracefully
- ✅ Test on testnet first

### Performance
- ✅ Cache blockchain data when possible
- ✅ Use WebSocket subscriptions for live updates
- ✅ Batch multiple calls
- ✅ Implement pagination for large datasets
- ✅ Optimize bundle sizes

### Code Quality
- ✅ Write tests for critical functions
- ✅ Use TypeScript for type safety
- ✅ Comment complex logic
- ✅ Follow Substrate/Polkadot naming conventions
- ✅ Keep functions small and focused

---

## 🆘 Getting Help

### Developer Support
- **Discord**: #developers channel - discord.gg/belizechain
- **Forum**: forum.belizechain.org/developers
- **GitHub**: github.com/belizechain/belizechain
- **Email**: dev@belizechain.org

### Report Bugs
- **GitHub Issues**: github.com/belizechain/belizechain/issues
- **Security Issues**: security@belizechain.org (private)

### Community Resources
- **Stack Overflow**: Tag `belizechain`
- **Reddit**: r/belizechain
- **Twitter**: @belizechain

---

## 🎉 Showcase Your Project

Built something cool? Share it!

1. **Submit to Gallery**: dev@belizechain.org
2. **Get Featured**: Best projects featured on official site
3. **Win Grants**: Apply for BelizeChain Developer Grants
4. **Join Hackathons**: Quarterly hackathons with prizes

---

## 📊 Developer Stats

**BelizeChain Stats**:
- **Block Time**: 6 seconds
- **Finality**: 1 block (6 seconds)
- **TPS**: ~1000 transactions/second
- **Fee**: ~$0.01 USD average
- **Languages**: Rust (pallets), JavaScript/TypeScript (dApps)
- **API**: Polkadot.js compatible

---

## 🚀 Ready to Build?

Choose your path:
- **New to blockchain?** → [Setup Environment](./setup-environment.md)
- **Want to code now?** → [Build Your First DApp](./build-dapp.md)
- **Need API docs?** → [API Reference](./api-reference.md)
- **Building pallets?** → [Custom Pallets Guide](./custom-pallets.md)

**Let's build the future of Belize together!** 🇧🇿💻
