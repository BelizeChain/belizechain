# BelizeChain Glossary

Technical terminology and BelizeChain-specific concepts.

## A

**Account**: A cryptographic address that holds DALLA/bBZD. Types include:
- **User accounts**: ss58-encoded, controlled by private keys
- **Contract accounts**: Deployed smart contracts with code storage
- **Multi-sig accounts**: Require multiple signatures (e.g., 4-of-7 treasury)

→ See [Identity Pallet](developer-guides/pallet-apis-core.md#identity-pallet)

**AES-256**: Advanced Encryption Standard with 256-bit keys. Used for encrypting BelizeID data (SSN, passport) at rest.

→ See [Security](security/kyc-aml-procedures.md#data-encryption)

**AMM (Automated Market Maker)**: Trading mechanism using constant product formula `x * y = k`. BelizeX DEX hybrid model combines AMM with limit order book.

→ See [BelizeX DEX](operations/bridges-dex-landledger.md#belizex-dex)

**AML (Anti-Money Laundering)**: Regulatory framework monitoring transactions >10,000 DALLA, flagging suspicious patterns, reporting SARs within 24h to FSC.

→ See [KYC/AML Procedures](security/kyc-aml-procedures.md)

**API (Application Programming Interface)**: BelizeChain exposes REST, WebSocket, GraphQL, and RPC interfaces for blockchain interaction.

→ See [API Reference](technical-reference/api-reference.md)

**APY (Annual Percentage Yield)**: Staking rewards:
- **Base**: 3-6% (adjusts with staking participation)
- **With PoUW**: Up to 544% (Nawal + Kinich bonuses)

→ See [Staking Rewards](economics/staking-rewards.md)

**Quantum Backend Provider**: External quantum compute service. Kinich integrates IonQ, Quantinuum, IBM, Rigetti, and simulator backends.

→ See [Kinich Quantum](services/kinich-quantum-computing.md)

## B

**BABE (Blind Assignment for Blockchain Extension)**: Block production algorithm selecting validators via VRF (Verifiable Random Function). 6-second block times.

→ See [Consensus](architecture/consensus.md)

**BelizeID**: National digital identity linked to SSN/passport. Required for all KYC levels. Stored encrypted on-chain.

→ See [Identity Pallet](developer-guides/pallet-apis-core.md#identity-pallet)

**BelizeX**: Decentralized exchange (DEX) with hybrid AMM + limit order book. 0.3% swap fee (0.25% to LPs, 0.05% to treasury).

→ See [BelizeX DEX](operations/bridges-dex-landledger.md#belizex-dex)

**Blue Hole Portal**: Government dashboard for treasury management (4-of-7 multi-sig), KYC approvals, district analytics.

→ See [Business & Government Guide](user-guides/business-government-guides.md#blue-hole-portal)

**BNS (Belize Name Service)**: .bz domain registration on-chain. Features include Pakit content hosting, email forwarding, marketplace.

→ See [BNS Service](services/bns-overview.md)

**Block**: Container of transactions:
- **Header**: Parent hash, state root, extrinsics root, block number
- **Body**: List of extrinsics (transactions)
- **Produced every**: 6 seconds
- **Finalized**: 12 seconds (2 blocks via GRANDPA)

→ See [Architecture](architecture/overview.md)

**Bonding**: Locking DALLA for staking. Minimum 1,000 DALLA. Unbonding period 14 days.

→ See [Staking](economics/staking-rewards.md#validator-staking)

**BZD (Belize Dollar)**: Official fiat currency of Belize. 1 bBZD = 1 BZD (Central Bank backed).

## C

**CID (Content Identifier)**: Hash-based identifier for Pakit DAG storage. Compatible with IPFS CIDv1 (multibase encoding).

→ See [DAG Storage](architecture/dag-storage-pakit.md)

**Compression**: Pakit supports:
- **Classical**: zstd (2.6x), gzip (5.4x), lz4 (1.8x)
- **Quantum**: 6.8-7.8x ratio (hybrid quantum-classical)

→ See [Compression Engine](architecture/dag-compression-engine.md)

**Consensus**: Agreement mechanism:
- **GRANDPA**: Byzantine fault-tolerant finality
- **BABE**: Block production via VRF
- **PoUW**: Proof of Useful Work (Nawal + Kinich)

→ See [Consensus Pallet](developer-guides/pallet-apis-infrastructure.md#consensus-pallet)

**Conviction Voting**: Lock tokens longer = more voting power:
- **None**: 1x (no lock)
- **Locked1x**: 1x (7 days)
- **Locked2x**: 2x (14 days)
- **Locked6x**: 6x (42 days)

→ See [Voting Mechanisms](governance/voting-mechanisms.md)

**Cross-Chain**: Interoperability via bridges:
- **Ethereum**: Lock-and-mint, 150s finality, 0.1% fee
- **Polkadot**: XCM v3, 12s finality, 0.05% fee

→ See [Bridges](operations/bridges-dex-landledger.md)

## D

**DAG (Directed Acyclic Graph)**: Pakit storage structure using IPLD. Content-addressed, deduplication, quantum compression.

→ See [DAG Storage](architecture/dag-storage-pakit.md)

**DALLA**: Native blockchain token:
- **Decimals**: 12 (1 DALLA = 1,000,000,000,000 units)
- **Inflation**: 3-6% APY
- **Uses**: Transaction fees, staking, governance
- **Total supply**: ~50 million (circulating increases with inflation)

→ See [Tokenomics](economics/tokenomics.md)

**DAO (Decentralized Autonomous Organization)**: Smart contract-based governance. GEM platform provides templates.

→ See [DAO Templates](smart-contracts/dao-templates.md)

**Deduplication**: Content-addressable chunking eliminates duplicate data in Pakit. 2.3x storage savings average.

→ See [Deduplication](architecture/dag-deduplication.md)

**DEX (Decentralized Exchange)**: BelizeX trading platform for DALLA/bBZD and tokenized assets.

→ See [BelizeX DEX](operations/bridges-dex-landledger.md#belizex-dex)

**Differential Privacy (DP)**: Nawal uses DP-SGD:
- **Privacy budget**: ε = 0.1
- **Delta**: δ = 1e-5
- **Gradient clipping**: C = 1.0
- **Gaussian noise**: σ = √(2ln(1.25/δ))/ε

→ See [Nawal AI](services/nawal-federated-learning.md#differential-privacy)

**DP-SGD (Differentially Private Stochastic Gradient Descent)**: Training algorithm preserving privacy by adding calibrated noise to gradients.

→ See [Nawal AI](services/nawal-federated-learning.md#differential-privacy)

## E

**Epoch**: Time period of ~4 hours (2,400 blocks). 6 epochs per era.

**Era**: 24-hour period used for staking reward distribution. Each era has 6 epochs.

→ See [Staking](economics/staking-rewards.md)

**ERC-20**: Ethereum token standard. Equivalent on BelizeChain is PSP22.

**ERC-721**: Ethereum NFT standard. Equivalent on BelizeChain is PSP34.

→ See [Smart Contracts](developer-guides/smart-contract-development.md)

**Extrinsic**: Transaction submitted to blockchain:
- **Signed**: From user accounts (fees paid)
- **Unsigned**: From validators (no fees)
- **Inherent**: Built-in (timestamps, validator set)

→ See [Blockchain Development](developer-guides/blockchain-development.md)

## F

**Faucet**: Testnet DALLA distribution:
- **Amount**: 1,000 DALLA per claim
- **Cooldown**: 24 hours
- **URL**: https://faucet.belizechain.org

**Federated Learning**: Privacy-preserving ML training. Local data stays on device, only gradients shared.

→ See [Nawal AI](services/nawal-federated-learning.md)

**Finality**: Irreversible block confirmation. GRANDPA provides:
- **Time**: 12 seconds (2 blocks)
- **Mechanism**: Byzantine fault-tolerant voting

→ See [Consensus](architecture/consensus.md)

**FSC (Financial Services Commission)**: Belize regulator overseeing:
- KYC/AML compliance
- Quarterly financial audits
- Validator licensing
- Securities regulation

→ See [Compliance](security/kyc-aml-procedures.md)

## G

**GEM (Genesis Ecosystem for Modules)**: Smart contract platform:
- **Language**: ink! 4.0 (Rust → WebAssembly)
- **Standards**: PSP22, PSP34
- **Templates**: Tokens, NFTs, DAOs

→ See [GEM Platform](smart-contracts/gem-platform-overview.md)

**Governance**: On-chain decision-making:
- **Democracy**: Public referendums (7-day voting)
- **Councils**: 6 district councils (3-5 members each)
- **Treasury**: 4-of-7 multi-sig (DALLA reserve)

→ See [Democracy](governance/democracy.md)

**GRANDPA (GHOST-based Recursive Ancestor Deriving Prefix Agreement)**: Finality mechanism. Validators vote on chains, not blocks. 12-second finality.

→ See [Consensus](architecture/consensus.md)

**GST (Goods and Services Tax)**: 12.5% tax on goods purchases. Auto-deducted in payroll system.

→ See [Payroll](operations/payroll-system.md)

## H

**Hash**: Cryptographic fingerprint:
- **Blake2b**: Default for BelizeChain (faster than SHA-256)
- **Used for**: Transaction IDs, block hashes, Merkle roots

**Hierarchical Partition Keys (HPK)**: Not used. BelizeChain uses single partition key per account for simplicity.

## I

**Identity**: BelizeID system with 3 KYC levels:
- **Basic**: BelizeID only (free, instant)
- **Verified**: + Documents ($50, 2 days)
- **Enhanced**: + Biometrics ($200, 5 days)

→ See [Identity Pallet](developer-guides/pallet-apis-core.md#identity-pallet)

**Inflation**: DALLA supply increase:
- **Rate**: 3-6% APY
- **Distribution**: 70% validators, 20% treasury, 10% governance

→ See [Tokenomics](economics/tokenomics.md)

**ink!**: Rust-based smart contract language compiling to WebAssembly. Version 4.0 supported.

→ See [Smart Contract Development](developer-guides/smart-contract-development.md)

**IPLD (InterPlanetary Linked Data)**: DAG data model used by Pakit. Compatible with IPFS CIDv1.

→ See [DAG Storage](architecture/dag-storage-pakit.md)

## K

**Keyring**: Account manager holding private keys. Supports sr25519, ed25519, ecdsa.

**Kinich**: Quantum computing layer:
- **Backends**: IonQ, Quantinuum, IBM, Rigetti
- **Compression**: 6.8x quantum ratio
- **PQW**: Proof of Quantum Work (50-200 DALLA/job)

→ See [Kinich Quantum](services/kinich-quantum-computing.md)

**KYC (Know Your Customer)**: Identity verification. 3 levels (Basic/Verified/Enhanced) with increasing costs/limits.

→ See [KYC/AML Procedures](security/kyc-aml-procedures.md)

## L

**LandLedger**: Property registry on-chain:
- **Requirements**: Enhanced KYC + government approval
- **Transfer fee**: 5% stamp duty
- **Verification**: Merkle proof + Pakit document storage

→ See [LandLedger](operations/bridges-dex-landledger.md#landledger-property-registry)

**LP (Liquidity Provider)**: Users supplying token pairs to BelizeX AMM. Earn 0.25% of swap fees.

→ See [BelizeX DEX](operations/bridges-dex-landledger.md#belizex-dex)

## M

**Maya Wallet**: Official mobile/desktop wallet:
- **Platforms**: iOS, Android, macOS, Windows, Linux
- **Features**: Payments, staking, tourism cashback, BelizeID, governance voting

→ See [Maya Wallet Guide](user-guides/maya-wallet-guide.md)

**MerkleDAG**: Directed acyclic graph with Merkle tree properties. Enables cryptographic verification of Pakit storage.

→ See [DAG Storage](architecture/dag-storage-pakit.md)

**Multi-Sig (Multi-Signature)**: Account requiring M-of-N signatures. Treasury uses 4-of-7 (4 district representatives out of 7 total).

→ See [Democracy](governance/democracy.md#treasury-multi-sig)

## N

**Nawal**: Federated learning system:
- **Privacy**: Differential privacy (ε=0.1)
- **Model**: BelizeChainLLM (110M parameters)
- **PoUW**: 50-350 DALLA/session

→ See [Nawal AI](services/nawal-federated-learning.md)

**NFT (Non-Fungible Token)**: Unique tokens (PSP34 standard). Examples: property titles, identity documents, digital art.

→ See [PSP34 NFTs](smart-contracts/psp34-nfts.md)

**Nonce**: Transaction sequence number preventing replay attacks. Increments with each transaction from account.

**Nominator**: DALLA holder delegating stake to validators. Shares rewards proportionally.

→ See [Staking](economics/staking-rewards.md)

## O

**Oracle**: External data provider. BelizeChain Oracle pallet used ONLY for:
- **Merchant verification** (tourism cashback eligibility)
- **NOT** for bBZD peg (1:1 Central Bank backing, not oracle-based)

→ See [Oracle Pallet](developer-guides/pallet-apis-infrastructure.md#oracle-pallet)

## P

**Pakit**: Sovereign DAG storage system:
- **Structure**: Content-addressed IPLD
- **Compression**: 6.8x quantum ratio
- **Deduplication**: 2.3x storage savings
- **Encryption**: AES-256 optional

→ See [DAG Storage](architecture/dag-storage-pakit.md)

**Pallet**: Modular runtime component. BelizeChain has 15 custom pallets:
- **Core**: Economy, Identity, Governance, Compliance
- **Financial**: Staking, BelizeX, Treasury
- **Infrastructure**: Oracle, Interoperability, Consensus, Quantum
- **Services**: BNS, LandLedger, Payroll, Community, Contracts

→ See [Pallet APIs](developer-guides/pallet-apis-core.md)

**Payroll**: Enterprise payroll automation for all business types:
- **Employer Types**: Government, Enterprise, SME, Cooperative, GigPlatform, NonProfit
- **Worker Types**: FullTime, PartTime, Contractor, Freelancer, Seasonal, Intern
- **Deductions**: IncomeTax, SocialSecurity, Pension, HealthInsurance, Custom (auto-applied)
- **Departments**: Per-employer department management with per-department scheduling
- **Payments**: Salary, Bonus, Overtime, Commission, Reimbursement, Severance
- **Schedules**: Weekly, biweekly, monthly (per department)
- **Storage**: Payslips in Pakit

→ See [Payroll System](operations/payroll-system.md)

**PoUW (Proof of Useful Work)**: Validators earn rewards by:
- **Nawal**: Federated learning (50-350 DALLA/session)
- **Kinich**: Quantum computing (50-200 DALLA/job)

→ See [Staking Rewards](economics/staking-rewards.md)

**PQW (Proof of Quantum Work)**: Scoring algorithm for quantum job complexity:
- **Qubits**: 10q=20pts, 20q=60pts, 30q=100pts
- **Gates**: 100=10pts, 500=40pts, 1000=60pts
- **Depth**: 20=10pts, 50=30pts, 100=40pts
- **Rewards**: 50-200 DALLA based on total points

→ See [Kinich Quantum](services/kinich-quantum-computing.md#pqw-scoring)

**PSP22**: Polkadot Standard Proposal 22 (fungible token standard like ERC-20). Implemented in ink!.

→ See [PSP22 Tokens](smart-contracts/psp22-tokens.md)

**PSP34**: Polkadot Standard Proposal 34 (NFT standard like ERC-721). Implemented in ink!.

→ See [PSP34 NFTs](smart-contracts/psp34-nfts.md)

## Q

**Quantum Compression**: 6.8-7.8x ratio achieved through quantum-assisted algorithms. Integrated into Pakit storage.

→ See [Kinich Quantum](services/kinich-quantum-computing.md#quantum-compression)

**Quantum Computing**: Kinich layer supports:
- **IonQ**: 25 qubits, 99.5% fidelity
- **Quantinuum**: 20 qubits, 99.9% fidelity
- **IBM Quantum**: 127 qubits, 99.2% fidelity
- **Rigetti**: 32 qubits, 98.5% fidelity

→ See [Kinich Quantum](services/kinich-quantum-computing.md)

## R

**Referendum**: Public vote on governance proposals. 7-day voting period, conviction voting enabled.

→ See [Voting Mechanisms](governance/voting-mechanisms.md)

**RPC (Remote Procedure Call)**: Blockchain API methods:
- **chain_***: Block/header queries
- **state_***: Storage queries
- **author_***: Transaction submission
- **system_***: Node health/info

→ See [API Reference](technical-reference/api-reference.md)

**Runtime**: State transition function defining blockchain logic. Compiled to WebAssembly, upgradeable via governance.

→ See [Architecture](architecture/overview.md)

## S

**SAR (Suspicious Activity Report)**: AML compliance filing to FSC within 24h for transactions >10,000 DALLA with suspicious patterns.

→ See [KYC/AML Procedures](security/kyc-aml-procedures.md)

**SDK (Software Development Kit)**: Libraries for blockchain interaction:
- **JavaScript/TypeScript**: @polkadot/api
- **Python**: substrate-interface
- **Rust**: subxt

→ See [API Reference](technical-reference/api-reference.md)

**Session**: 1-hour period (600 blocks). Validators rotate roles each session.

**Session Keys**: Cryptographic keys for validator operations. Hot keys stored on validator node, separate from controller account.

→ See [Validator Setup](validators/validator-setup.md)

**Smart Contract**: Self-executing code on blockchain. Written in ink! (Rust), deployed to Contracts pallet.

→ See [Smart Contract Development](developer-guides/smart-contract-development.md)

**SS (Social Security)**: 8% employee + 8% employer deduction. Managed by SSB (Social Security Board).

→ See [Payroll](operations/payroll-system.md)

**Staking**: Locking DALLA to:
- **Secure network**: Validators produce blocks
- **Earn rewards**: Base 3-6% APY + PoUW bonuses = 544% effective

→ See [Staking Rewards](economics/staking-rewards.md)

**Substrate**: Blockchain framework by Parity Technologies. BelizeChain built on Substrate 3.0.

→ See [Architecture](architecture/overview.md)

## T

**TLS 1.3**: Transport Layer Security for encrypted connections. All BelizeChain RPC/WebSocket endpoints use TLS 1.3.

**TPS (Transactions Per Second)**: 
- **Sustained**: 1,000 TPS
- **Peak**: 2,500 TPS
- **Theoretical max**: 5,000 TPS

→ See [Performance Benchmarks](performance/benchmarks-optimization.md)

**Treasury**: DALLA reserve for public goods:
- **Funding**: 20% of inflation + 0.05% DEX fees
- **Governance**: 4-of-7 multi-sig approval
- **Current balance**: ~8 million DALLA

→ See [Tokenomics](economics/tokenomics.md)

## V

**Validator**: Node operator producing blocks and finalizing chain:
- **Requirements**: 1,000 DALLA stake minimum
- **Hardware**: 8-32 cores, 32-128 GB RAM
- **Rewards**: Block production + PoUW bonuses

→ See [Validator Setup](validators/validator-setup.md)

**VRF (Verifiable Random Function)**: Cryptographic randomness for BABE block production. Ensures fair validator selection.

## W

**Wasm (WebAssembly)**: Binary instruction format for smart contracts. ink! compiles to Wasm, executed by Contracts pallet.

→ See [Smart Contracts](developer-guides/smart-contract-development.md)

**WebSocket**: Real-time blockchain connection:
- **Mainnet**: wss://rpc.belizechain.org
- **Testnet**: Use the current operator-provided RPC URL or your own node

→ See [API Reference](technical-reference/api-reference.md)

**Weight**: Computational resource measurement. Replaces Ethereum gas. Calculated via benchmarking.

→ See [Blockchain Development](developer-guides/blockchain-development.md#weight-functions)

## X

**XCM (Cross-Consensus Messaging)**: Polkadot interoperability protocol. BelizeChain supports XCM v3 for asset transfers.

→ See [Bridges](operations/bridges-dex-landledger.md#polkadot-xcm)

## Z

**ZNE (Zero-Noise Extrapolation)**: Quantum error mitigation technique. Kinich achieves 72% error reduction.

→ See [Kinich Quantum](services/kinich-quantum-computing.md#error-mitigation)

**zstd (Zstandard)**: Compression algorithm. Pakit uses zstd level 3 for 2.6x compression ratio.

→ See [Compression Engine](architecture/dag-compression-engine.md)

## Belizean Terms

**bBZD**: Belize Dollar stablecoin (1:1 BZD peg, Central Bank backed)

**BTB (Belize Tourism Board)**: Verifies merchants for tourism cashback eligibility

**BZD (Belize Dollar)**: Official fiat currency (1 USD ≈ 2 BZD)

**DALLA**: Named after Belize Dollar historical term, native blockchain token

**FSC (Financial Services Commission)**: Primary financial regulator

**SSB (Social Security Board)**: Manages pension and benefits (8% deductions)

## See Also

- **Architecture**: [Overview](architecture/overview.md)
- **Developer Guides**: [Getting Started](developer-guides/environment-setup.md)
- **Economics**: [Tokenomics](economics/tokenomics.md)
- **FAQ**: [Frequently Asked Questions](FAQ.md)
- **Tutorials**: [Build Your First dApp](tutorials/README.md)
