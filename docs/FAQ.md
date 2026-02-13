# Frequently Asked Questions (FAQ)

## General Questions

### What is BelizeChain?

BelizeChain is a sovereign blockchain infrastructure for the nation of Belize, featuring:
- **Dual currency system**: DALLA (native token) + bBZD (1:1 BZD-pegged stablecoin)
- **16 custom pallets**: Economy, Identity, Governance, Compliance, Staking, Mesh, and more
- **AI integration**: Nawal federated learning for privacy-preserving ML
- **Quantum computing**: Kinich layer for quantum workloads and compression
- **Sovereign storage**: Pakit DAG storage with 6.8x quantum compression
- **Multi-repo architecture**: 7 specialized repositories for modular development

### Who can use BelizeChain?

- **Citizens**: Digital identity (BelizeID), payments, staking, tourism cashback
- **Businesses**: Merchant registration, POS systems, payroll, BNS domains
- **Government**: Blue Hole Portal for treasury, compliance, analytics
- **Validators**: Earn rewards through block production and Proof of Useful Work (PoUW)
- **Developers**: Build dApps, smart contracts, integrate AI/quantum features

### Is BelizeChain a fork of Ethereum or Polkadot?

No. BelizeChain is built on **Substrate** (same framework as Polkadot) but is a **standalone sovereign chain**, not a parachain. We support cross-chain bridges to Ethereum and Polkadot for interoperability.

### What's the difference between DALLA and bBZD?

| DALLA | bBZD |
|-------|------|
| Native blockchain token | Stablecoin pegged 1:1 to Belize Dollar (BZD) |
| Variable value (market-driven) | Fixed value (1 bBZD = 1 BZD always) |
| Used for fees, staking, governance | Used for stable payments, payroll, contracts |
| 3-6% APY inflation | Central Bank backed, no inflation |
| 12 decimals | 12 decimals |
| Earn through PoUW (544% APY staking) | Mint only by Central Bank after BZD deposit |

### How do I get DALLA or bBZD?

**DALLA**:
- Testnet faucet: https://faucet.belizechain.org (1,000 DALLA/claim)
- Mainnet: Buy on exchanges, earn through staking/PoUW, receive tourism cashback

**bBZD**:
- Deposit BZD at Central Bank of Belize
- Central Bank mints 1:1 bBZD to your wallet
- Government employees receive payroll in bBZD

## Technical Questions

### What consensus mechanism does BelizeChain use?

**Hybrid consensus**:
1. **GRANDPA finality**: Byzantine fault-tolerant finalization (12s finality)
2. **BABE block production**: Blind Assignment for Blockchain Extension (6s blocks)
3. **Proof of Useful Work (PoUW)**: Validators earn rewards by:
   - **Nawal**: Federated learning contributions (350 DALLA/session)
   - **Kinich**: Quantum computation workloads (200 DALLA/job)

### What programming languages are supported?

- **Smart contracts**: Rust (ink! for WebAssembly)
- **Runtime pallets**: Rust (Substrate framework)
- **AI/ML (Nawal)**: Python (PyTorch, Flower, Opacus)
- **Quantum (Kinich)**: Python (Qiskit, Azure Quantum SDK)
- **Storage (Pakit)**: Python (DAG compression, deduplication)
- **UI**: TypeScript/JavaScript (React, Next.js)
- **SDKs**: JavaScript, TypeScript, Python, Rust

### How many transactions per second (TPS)?

- **Sustained**: 1,000 TPS (6-validator network)
- **Peak**: 2,500 TPS (burst capacity)
- **Finality**: 12 seconds (GRANDPA)
- **Block time**: 6 seconds (BABE)

Performance varies by validator count: more validators = higher decentralization but lower TPS.

### What are the hardware requirements for validators?

| Validator Type | CPU | RAM | GPU | Storage | Use Case |
|----------------|-----|-----|-----|---------|----------|
| **Standard** | 8 cores | 32 GB | - | 500 GB SSD | Block production only |
| **Nawal** | 16 cores | 64 GB | A100 40GB | 2 TB SSD | + Federated learning |
| **Kinich** | 16 cores | 64 GB | - | 1 TB SSD | + Quantum workloads |
| **Full** | 32 cores | 128 GB | A100 80GB | 4 TB SSD | All capabilities |

### How does KYC work on BelizeChain?

**3 KYC Levels**:

| Level | Requirements | Cost | Processing Time | Transaction Limits |
|-------|--------------|------|-----------------|-------------------|
| **Basic** | BelizeID only | Free | Instant | 5,000 DALLA/month |
| **Verified** | + Documents (passport, utility bill) | $50 | 2 days | 50,000 DALLA/month |
| **Enhanced** | + Biometrics (fingerprint, facial scan) | $200 | 5 days | Unlimited |

KYC enforced by FSC (Financial Services Commission) with GDPR-like privacy protections.

## Staking & Rewards

### How do I stake DALLA?

```javascript
// Via Maya Wallet UI or programmatically:
const api = await ApiPromise.create({ provider });
const keyring = new Keyring({ type: 'sr25519' });
const account = keyring.addFromUri('//YourSeed');

// Bond 1,000 DALLA
const bond = api.tx.staking.bond(
  1_000_000_000_000_000n,  // 1,000 DALLA
  { Staked: null }  // Rewards go to staked balance
);

await bond.signAndSend(account);

// Nominate validators (up to 16)
const nominate = api.tx.staking.nominate([
  '5GrwvaEF...',
  '5FHneW...'
]);

await nominate.signAndSend(account);
```

**Minimum stake**: 1,000 DALLA  
**APY**: Up to 544% (varies by validator PoUW performance)  
**Unbonding period**: 14 days

### What is Proof of Useful Work (PoUW)?

Validators earn additional rewards by performing useful computations:

**Nawal PoUW (Federated Learning)**:
- Participate in 7 training rounds/week
- Earn 50-350 DALLA/session based on:
  - **Quality** (40%): Model improvement accuracy
  - **Timeliness** (30%): Submission before deadline
  - **Honesty** (30%): No Byzantine behavior
- Weekly earnings: 1,225-2,450 DALLA

**Kinich PQW (Quantum Work)**:
- Execute 10 quantum jobs/week
- Earn 50-200 DALLA/job based on circuit complexity:
  - 10 qubits: 1x multiplier
  - 20 qubits: 2x multiplier
  - 30+ qubits: 4x multiplier
- Weekly earnings: 500-2,000 DALLA

### How do I claim staking rewards?

```javascript
// Automatic compounding if payee = Staked
// Manual claim:
const claim = api.tx.staking.payoutStakers(
  validatorAddress,
  era  // Current era - 1
);

await claim.signAndSend(account);
```

Rewards distributed every era (~24 hours).

## Governance

### How do I vote on proposals?

**3 Ways**:

1. **Via Maya Wallet**: Navigate to Governance → Active Proposals → Vote
2. **Via Blue Hole Portal**: Government dashboard → Governance tab
3. **Programmatically**:

```javascript
const vote = api.tx.governance.vote(
  proposalId,
  {
    aye: {  // or 'nay'
      conviction: 'Locked2x',  // Lock tokens for 2x voting power
      amount: 100_000_000_000_000n  // 100 DALLA
    }
  }
);

await vote.signAndSend(account);
```

**Conviction voting**: Lock tokens longer = more voting power (1x-6x).

### Who can submit governance proposals?

**Anyone with**:
- Minimum 10,000 DALLA deposit (refunded if proposal passes)
- Verified KYC level
- Valid BelizeID

**Proposal types**:
- Referendum (public vote)
- Council motion (6 district councils vote)
- Treasury spend (4-of-7 multi-sig approval)
- Emergency (sudo power, government only)

### What are the 6 district councils?

1. **Belize District** (Belize City)
2. **Cayo District** (San Ignacio, Belmopan)
3. **Orange Walk District**
4. **Corozal District**
5. **Stann Creek District** (Dangriga)
6. **Toledo District** (Punta Gorda)

Each council has 3-5 elected members serving 2-year terms.

## Smart Contracts

### What is the GEM platform?

**GEM** (Genesis Ecosystem for Modules) is BelizeChain's smart contract platform:
- **Language**: ink! 4.0 (Rust compiled to WebAssembly)
- **Standards**: PSP22 (tokens), PSP34 (NFTs)
- **Templates**: ERC-20/721 equivalents, DAOs, governance
- **Faucet**: 1,000 DALLA/claim for testnet development
- **SDK**: JavaScript, TypeScript, Python libraries

### How do I deploy a smart contract?

```bash
# 1. Install cargo-contract
cargo install cargo-contract

# 2. Create contract from template
cargo contract new my_contract --template psp22

# 3. Build
cd my_contract
cargo contract build --release

# 4. Deploy
cargo contract instantiate \
  --constructor new \
  --args "1000000000000000" \
  --suri //YourSeed \
  --url wss://rpc.testnet.belizechain.org \
  --execute
```

### What's the maximum contract size?

- **Unoptimized**: ~512 KB
- **Optimized** (with wasm-opt): ~128 KB recommended
- **Blockchain limit**: 1 MB (but avoid approaching this)

Use `wasm-opt -Oz` and enable LTO to minimize size.

## Tourism Cashback

### How does tourism cashback work?

**Eligible merchants**:
- Hotels: 8% cashback
- Tours: 7% cashback
- Restaurants: 6% cashback
- Crafts: 5% cashback
- Retail: 5% cashback

**Process**:
1. Customer pays with DALLA at verified merchant
2. Cashback automatically calculated and credited within 24h
3. Customer receives bBZD (stable value)
4. Funding: 60% treasury subsidy + 40% network fees

**Requirements**:
- Basic KYC minimum
- Merchant must be tourism-verified by BTB (Belize Tourism Board)

### Can businesses become tourism merchants?

Yes! Apply via Blue Hole Portal:
1. Enhanced KYC registration
2. Business license verification
3. Category selection (Hotels/Tours/Restaurants/etc.)
4. 3-5 business day approval
5. Receive merchant QR code for POS

## Security & Privacy

### Is my data private on BelizeChain?

**Yes**, with multiple layers:
- **Federated learning**: Training data never leaves your device (Nawal)
- **Differential privacy**: ε=0.1 privacy budget protects individual contributions
- **On-chain encryption**: BelizeID SSN/passport data encrypted (AES-256)
- **Access control**: Role-based permissions (only authorized parties see full data)
- **GDPR-like framework**: Right to access, correct, delete personal data

**Not private**:
- Transaction amounts and addresses (public blockchain)
- Property registry records (LandLedger, publicly verifiable)

### Has BelizeChain been audited?

**Yes**:
- **Trail of Bits audit** (December 2025)
  - **High severity**: 0
  - **Medium severity**: 3 (all fixed)
  - **Low severity**: 7 (all fixed)
  - **Informational**: 12
- **Penetration testing**: Network, smart contracts, web interfaces
- **Dependency audits**: Substrate 3.0, ink! 4.0, Polkadot.js (all patched)
- **Bug bounty**: $500-$50,000 based on severity

See [Security Audit Results](security/security-audit-results.md).

## Troubleshooting

### Why is my transaction failing?

**Common reasons**:
1. **Insufficient balance**: Need DALLA for transaction + fees
2. **Nonce error**: Wait for pending transactions to finalize
3. **KYC required**: Upgrade to Verified/Enhanced level
4. **Rate limiting**: Wait 60 seconds between rapid transactions

Check [Troubleshooting Guide](operations/troubleshooting.md).

### How do I report a bug?

1. **GitHub Issues**: https://github.com/belizechain/belizechain/issues
2. **Discord**: https://discord.gg/belizechain (#support channel)
3. **Email**: support@belizechain.org
4. **Bug bounty**: security@belizechain.org (for vulnerabilities)

### Where can I get help?

- **Documentation**: https://docs.belizechain.org
- **Discord**: Community support (fastest response)
- **Stack Overflow**: Tag `belizechain`
- **Email**: support@belizechain.org (1 business day response)
- **Emergency**: emergency@belizechain.org (validators only, immediate)

## Economics

### What is DALLA inflation?

- **Current**: 3-6% APY (adjusts based on staking participation)
- **Target staking rate**: 60% of total supply
- **Distribution**:
  - Validators: 70% (block production + PoUW)
  - Treasury: 20% (public goods funding)
  - Governance: 10% (proposal incentives)

### How is bBZD pegged to BZD?

**1:1 Central Bank backing** (NOT Oracle-based):
1. User deposits BZD at Central Bank
2. Central Bank verifies deposit off-chain
3. Central Bank mints 1 bBZD for every 1 BZD deposited
4. bBZD backed 100% by BZD reserves
5. Redeem bBZD for BZD at any time (1:1)

**Oracle pallet**: Only used for merchant verification, NOT for bBZD peg.

### What are the transaction fees?

- **Economy transfers**: ~0.00015 DALLA (~$0.0001 USD)
- **Smart contract deployment**: ~0.25 DALLA (~$0.15 USD)
- **Governance vote**: ~0.0005 DALLA (~$0.0003 USD)
- **Cross-chain bridge**: 0.1% (Ethereum), 0.05% (Polkadot XCM)
- **DEX swap**: 0.3% (0.25% to LPs, 0.05% to treasury)

Fees paid in DALLA, burned to reduce inflation.

## Development

### Which SDK should I use?

- **JavaScript/TypeScript**: `@polkadot/api` (most popular)
- **Python**: `substrate-interface` (data science, ML integration)
- **Rust**: `subxt` (performance-critical applications)

All support same functionality.

### How do I test my dApp?

**3 environments**:
1. **Local dev node**: `./belizechain-node --dev --tmp`
2. **Testnet**: `wss://rpc.testnet.belizechain.org` (public, faucet available)
3. **Mainnet**: `wss://rpc.belizechain.org` (production, real value)

Use testnet for integration testing before mainnet deployment.

### Can I contribute to BelizeChain?

**Yes!** See [Contributing Guide](CONTRIBUTING.md):
- Code contributions (Rust, Python, TypeScript)
- Documentation improvements
- Bug reports and feature requests
- Community support (Discord, Stack Overflow)
- Translations (Spanish, Creole, Garifuna, Q'eqchi')

All contributions licensed under GPL-3.0 (blockchain) + MIT (smart contracts).
