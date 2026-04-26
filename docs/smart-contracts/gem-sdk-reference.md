# GEM SDK Reference

**JavaScript/TypeScript SDK for BelizeChain Smart Contracts**

The GEM SDK provides a developer-friendly interface for interacting with BelizeChain's smart contract platform, including PSP22 tokens (DALLA), PSP34 NFTs (BeLi), DAO governance, and testnet faucet functionality.

Replace the public-testnet endpoint placeholders below with the current operator-published RPC and faucet URLs for the active network.

---

## Installation

### NPM
```bash
npm install @belizechain/gem-sdk @polkadot/api @polkadot/api-contract
```

### Yarn
```bash
yarn add @belizechain/gem-sdk @polkadot/api @polkadot/api-contract
```

### Peer Dependencies
- `@polkadot/api` >= 10.9.1
- `@polkadot/api-contract` >= 10.9.1
- Node.js >= 18.0.0

---

## Quick Start

### Connect to BelizeChain
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

// Create SDK instance (defaults to ws://localhost:9944)
const sdk = new GemSDK();

// Or specify custom node
const sdkTestnet = new GemSDK('wss://<current-public-testnet-rpc>');

// Connect to node
await sdk.connect();

// Get chain info
const chain = await sdk.api.rpc.system.chain();
console.log('Connected to:', chain.toString());

// Disconnect when done
await sdk.disconnect();
```

### Account Management
```javascript
// Development accounts (local devnet only)
const alice = sdk.getAccount('//Alice');
const bob = sdk.getAccount('//Bob');

// Production: Load from seed phrase
const user = sdk.getAccount('your twelve word seed phrase here');

// Get account balance
const balance = await sdk.getBalance(alice.address);
console.log('Free balance:', (parseInt(balance.free) / 1e12).toFixed(2), 'DALLA');
```

---

## API Reference

### Constructor

#### `new GemSDK(nodeUrl?: string)`

Create a new SDK instance.

**Parameters:**
- `nodeUrl` (optional): WebSocket URL
  - Default: `ws://localhost:9944`
    - Testnet: `wss://<current-public-testnet-rpc>`
  - Mainnet: `wss://rpc.belizechain.org`

**Example:**
```typescript
const sdk = new GemSDK('ws://localhost:9944');
```

---

### Connection Methods

#### `connect(): Promise<ApiPromise>`

Connect to BelizeChain node and initialize API.

**Returns:** Polkadot.js ApiPromise instance

**Example:**
```javascript
await sdk.connect();
console.log('Connected to BelizeChain');
```

**Throws:**
- `Error` if connection fails

---

#### `disconnect(): Promise<void>`

Disconnect from node and cleanup resources.

**Example:**
```javascript
await sdk.disconnect();
```

---

### Account Methods

#### `getAccount(uri: string): KeyringPair`

Get account keypair from URI or seed phrase.

**Parameters:**
- `uri`: Account URI (`//Alice`, `//Bob`), seed phrase, or private key

**Returns:** Polkadot.js KeyringPair

**Example:**
```javascript
const alice = sdk.getAccount('//Alice');
const user = sdk.getAccount('your seed phrase here');
```

---

#### `getBalance(address: string): Promise<BalanceInfo>`

Get account balance details.

**Parameters:**
- `address`: Account address (ss58 format)

**Returns:**
```typescript
interface BalanceInfo {
    free: string;      // Available balance (planck units)
    reserved: string;  // Reserved balance
    frozen: string;    // Frozen balance
}
```

**Example:**
```javascript
const balance = await sdk.getBalance(alice.address);
const dallaBalance = parseInt(balance.free) / 1e12;
console.log(`Balance: ${dallaBalance} DALLA`);
```

---

## PSP22 Token Methods (DALLA)

### Transfer Tokens

#### `dallaTransfer(contractAddress, signer, to, amount, gasLimit?): Promise<void>`

Transfer DALLA tokens to another account.

**Parameters:**
- `contractAddress`: DALLA contract address
- `signer`: Sender's KeyringPair
- `to`: Recipient address
- `amount`: Amount in planck units (1 DALLA = 10^12 planck)
- `gasLimit` (optional): Custom gas limit (default: auto-estimated)

**Gas Cost:** ~15,000 units (~0.000015 DALLA)

**Example:**
```javascript
const dallaContract = '5GD4w5...NVsNB';

// Transfer 100 DALLA
await sdk.dallaTransfer(
    dallaContract,
    alice,
    bob.address,
    100000000000000  // 100 * 10^12
);

// Transfer 0.5 DALLA
await sdk.dallaTransfer(
    dallaContract,
    alice,
    bob.address,
    500000000000  // 0.5 * 10^12
);
```

---

### Query Balance

#### `dallaBalanceOf(contractAddress, account): Promise<string>`

Get account's token balance.

**Parameters:**
- `contractAddress`: DALLA contract address
- `account`: Account address to query

**Returns:** Balance in planck units (string)

**Example:**
```javascript
const balance = await sdk.dallaBalanceOf(dallaContract, alice.address);
const dallaAmount = parseInt(balance) / 1e12;
console.log(`${dallaAmount} DALLA`);
```

---

### Get Token Metadata

#### `dallaMetadata(contractAddress): Promise<TokenMetadata>`

Get token metadata (name, symbol, decimals, supply).

**Returns:**
```typescript
interface TokenMetadata {
    name: string;        // "DALLA"
    symbol: string;      // "DALLA"
    decimals: number;    // 12
    totalSupply: string; // Total supply in planck
}
```

**Example:**
```javascript
const metadata = await sdk.dallaMetadata(dallaContract);
console.log('Token:', metadata.name);
console.log('Supply:', parseInt(metadata.totalSupply) / 1e12, 'DALLA');
```

---

### Approve Allowance

#### `dallaApprove(contractAddress, signer, spender, amount): Promise<void>`

Approve another account to spend tokens.

**Parameters:**
- `contractAddress`: DALLA contract address
- `signer`: Owner's KeyringPair
- `spender`: Spender address to approve
- `amount`: Allowance amount in planck units

**Example:**
```javascript
// Approve DEX to spend 1000 DALLA
await sdk.dallaApprove(
    dallaContract,
    alice,
    dexContractAddress,
    1000000000000000  // 1000 DALLA
);
```

---

### Transfer From Allowance

#### `dallaTransferFrom(contractAddress, signer, from, to, amount): Promise<void>`

Transfer tokens from an approved allowance.

**Parameters:**
- `contractAddress`: DALLA contract address
- `signer`: Spender's KeyringPair (approved account)
- `from`: Token owner address
- `to`: Recipient address
- `amount`: Amount in planck units

**Example:**
```javascript
// DEX transfers tokens from user
await sdk.dallaTransferFrom(
    dallaContract,
    dexAccount,      // DEX is the approved spender
    alice.address,   // From Alice's balance
    bob.address,     // To Bob
    500000000000000  // 500 DALLA
);
```

---

## PSP34 NFT Methods (BeLi)

### Mint NFT

#### `nftMint(contractAddress, signer, to, uri): Promise<void>`

Mint new NFT to recipient.

**Parameters:**
- `contractAddress`: BeliNFT contract address
- `signer`: Minter's KeyringPair (must have minting role)
- `to`: Recipient address
- `uri`: Token metadata URI (IPFS or HTTP)

**Gas Cost:** ~45,000 units (~0.000045 DALLA)

**Example:**
```javascript
const beliNftContract = '5Ho6Ks...iFQL7';

// Mint Blue Hole NFT
await sdk.nftMint(
    beliNftContract,
    alice,
    bob.address,
    'ipfs://QmYourBlueHoleImageHash'
);

// Mint with HTTP metadata
await sdk.nftMint(
    beliNftContract,
    alice,
    bob.address,
    'https://nft.belizechain.org/metadata/1.json'
);
```

---

### Transfer NFT

#### `nftTransfer(contractAddress, signer, to, tokenId): Promise<void>`

Transfer NFT to another account.

**Parameters:**
- `contractAddress`: BeliNFT contract address
- `signer`: Current owner's KeyringPair
- `to`: Recipient address
- `tokenId`: NFT token ID (number)

**Gas Cost:** ~18,000 units (~0.000018 DALLA)

**Example:**
```javascript
// Transfer NFT #5 to Bob
await sdk.nftTransfer(
    beliNftContract,
    alice,
    bob.address,
    5
);
```

---

### Query NFT Owner

#### `nftOwnerOf(contractAddress, tokenId): Promise<string>`

Get current owner of NFT.

**Parameters:**
- `contractAddress`: BeliNFT contract address
- `tokenId`: NFT token ID

**Returns:** Owner address

**Example:**
```javascript
const owner = await sdk.nftOwnerOf(beliNftContract, 5);
console.log('NFT #5 owned by:', owner);
```

---

### Get NFT Metadata

#### `nftTokenUri(contractAddress, tokenId): Promise<string>`

Get NFT metadata URI.

**Parameters:**
- `contractAddress`: BeliNFT contract address
- `tokenId`: NFT token ID

**Returns:** Metadata URI (IPFS or HTTP)

**Example:**
```javascript
const uri = await sdk.nftTokenUri(beliNftContract, 5);
console.log('Metadata:', uri);

// Fetch metadata JSON
const response = await fetch(uri.replace('ipfs://', 'https://ipfs.io/ipfs/'));
const metadata = await response.json();
console.log('Name:', metadata.name);
console.log('Description:', metadata.description);
console.log('Image:', metadata.image);
```

---

## DAO Governance Methods

### Create Proposal

#### `daoCreateProposal(contractAddress, signer, description): Promise<number>`

Create new governance proposal.

**Parameters:**
- `contractAddress`: DAO contract address
- `signer`: Proposer's KeyringPair (must hold 10,000+ DALLA)
- `description`: Proposal description text

**Returns:** Proposal ID (number)

**Gas Cost:** ~80,000 units (~0.00008 DALLA)

**Example:**
```javascript
const daoContract = '5DAOAddr...xyz';

const proposalId = await sdk.daoCreateProposal(
    daoContract,
    alice,
    'Allocate 50,000 DALLA to infrastructure development fund'
);

console.log('Created proposal #', proposalId);
```

---

### Vote on Proposal

#### `daoVote(contractAddress, signer, proposalId, support): Promise<void>`

Cast vote on proposal.

**Parameters:**
- `contractAddress`: DAO contract address
- `signer`: Voter's KeyringPair
- `proposalId`: Proposal ID to vote on
- `support`: Vote decision (true = for, false = against)

**Voting Weight:** Based on DALLA holdings at proposal creation block

**Example:**
```javascript
// Vote FOR proposal #0
await sdk.daoVote(daoContract, alice, 0, true);

// Vote AGAINST proposal #1
await sdk.daoVote(daoContract, bob, 1, false);
```

---

### Get Proposal Details

#### `daoGetProposal(contractAddress, proposalId): Promise<ProposalInfo>`

Get proposal information.

**Parameters:**
- `contractAddress`: DAO contract address
- `proposalId`: Proposal ID

**Returns:**
```typescript
interface ProposalInfo {
    description: string;
    proposer: string;
    forVotes: string;      // In DALLA units
    againstVotes: string;
    executed: boolean;
    canceled: boolean;
}
```

**Example:**
```javascript
const proposal = await sdk.daoGetProposal(daoContract, 0);
console.log('Description:', proposal.description);
console.log('For:', parseInt(proposal.forVotes) / 1e12, 'DALLA');
console.log('Against:', parseInt(proposal.againstVotes) / 1e12, 'DALLA');
console.log('Status:', proposal.executed ? 'Executed' : 'Pending');
```

---

## Faucet Methods

### Claim Tokens

#### `faucetClaim(contractAddress, signer): Promise<void>`

Claim testnet DALLA tokens from faucet.

**Parameters:**
- `contractAddress`: Faucet contract address
- `signer`: Claimer's KeyringPair

**Drip Amount:** 1,000 DALLA per claim  
**Cooldown:** 100 blocks (~10 minutes)  
**Daily Limit:** 10,000 DALLA per account

**Example:**
```javascript
const faucetContract = '5FaucetAddr...xyz';

// Claim 1000 DALLA
await sdk.faucetClaim(faucetContract, alice);
console.log('Claimed 1000 DALLA from faucet');
```

---

### Check Eligibility

#### `faucetCanClaim(contractAddress, account): Promise<boolean>`

Check if account can claim from faucet.

**Parameters:**
- `contractAddress`: Faucet contract address
- `account`: Account address to check

**Returns:** true if eligible, false if on cooldown

**Example:**
```javascript
const canClaim = await sdk.faucetCanClaim(faucetContract, alice.address);

if (canClaim) {
    await sdk.faucetClaim(faucetContract, alice);
} else {
    console.log('Please wait for cooldown period');
}
```

---

### Get Last Claim Time

#### `faucetGetLastClaim(contractAddress, account): Promise<number>`

Get block number of last claim.

**Parameters:**
- `contractAddress`: Faucet contract address
- `account`: Account address

**Returns:** Block number of last claim (0 if never claimed)

**Example:**
```javascript
const lastClaim = await sdk.faucetGetLastClaim(faucetContract, alice.address);
const currentBlock = await sdk.api.query.system.number();

const blocksSinceLastClaim = currentBlock.toNumber() - lastClaim;
const canClaimIn = Math.max(0, 100 - blocksSinceLastClaim);

console.log(`Can claim in ${canClaimIn} blocks (~${canClaimIn / 10} minutes)`);
```

---

## TypeScript Support

The SDK includes complete TypeScript definitions for all methods and types.

### Type Definitions

```typescript
import { GemSDK, TokenMetadata, BalanceInfo, ProposalInfo } from '@belizechain/gem-sdk';
import { KeyringPair } from '@polkadot/keyring/types';
import { ApiPromise } from '@polkadot/api';

// Create typed SDK instance
const sdk: GemSDK = new GemSDK('ws://localhost:9944');

// Account management
const alice: KeyringPair = sdk.getAccount('//Alice');

// Balance query
const balance: BalanceInfo = await sdk.getBalance(alice.address);

// Token metadata
const metadata: TokenMetadata = await sdk.dallaMetadata(contractAddress);

// Proposal info
const proposal: ProposalInfo = await sdk.daoGetProposal(daoContract, 0);
```

### Custom Types

```typescript
interface TokenMetadata {
    name: string;
    symbol: string;
    decimals: number;
    totalSupply: string;
}

interface BalanceInfo {
    free: string;
    reserved: string;
    frozen: string;
}

interface ProposalInfo {
    description: string;
    proposer: string;
    forVotes: string;
    againstVotes: string;
    executed: boolean;
    canceled: boolean;
}
```

---

## Error Handling

All SDK methods throw errors on failure. Use try/catch blocks for proper error handling.

```javascript
try {
    await sdk.dallaTransfer(contractAddress, alice, bob.address, amount);
    console.log('Transfer successful');
} catch (error) {
    if (error.message.includes('InsufficientBalance')) {
        console.error('Not enough DALLA tokens');
    } else if (error.message.includes('ExtrinsicFailed')) {
        console.error('Transaction failed:', error.message);
    } else {
        console.error('Unexpected error:', error);
    }
}
```

### Common Errors

- `InsufficientBalance`: Not enough tokens/gas
- `ExtrinsicFailed`: Transaction reverted
- `TokenNotFound`: Invalid token ID (PSP34)
- `NotOwner`: Caller doesn't own NFT
- `CooldownNotExpired`: Faucet claim too soon
- `DailyLimitExceeded`: Faucet daily limit reached

---

## Examples

### Complete Wallet Service

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

class WalletService {
    constructor(nodeUrl) {
        this.sdk = new GemSDK(nodeUrl);
        this.dallaContract = '5GD4w5...NVsNB';
    }

    async connect() {
        await this.sdk.connect();
    }

    async getWalletInfo(address) {
        const nativeBalance = await this.sdk.getBalance(address);
        const dallaBalance = await this.sdk.dallaBalanceOf(this.dallaContract, address);

        return {
            address,
            nativeBalance: parseInt(nativeBalance.free) / 1e12,
            dallaBalance: parseInt(dallaBalance) / 1e12
        };
    }

    async sendTokens(fromAccount, toAddress, amount) {
        const amountPlanck = amount * 1e12;
        await this.sdk.dallaTransfer(
            this.dallaContract,
            fromAccount,
            toAddress,
            amountPlanck.toString()
        );
    }

    async disconnect() {
        await this.sdk.disconnect();
    }
}

// Usage
const wallet = new WalletService('wss://<current-public-testnet-rpc>');
await wallet.connect();

const alice = wallet.sdk.getAccount('//Alice');
const info = await wallet.getWalletInfo(alice.address);
console.log('Wallet:', info);

await wallet.sendTokens(alice, bob.address, 100);
await wallet.disconnect();
```

### NFT Marketplace Integration

```javascript
async function listNftForSale(nftContract, tokenId, price) {
    const sdk = new GemSDK('wss://<current-public-testnet-rpc>');
    await sdk.connect();

    const seller = sdk.getAccount(process.env.SELLER_SEED);

    // Verify ownership
    const owner = await sdk.nftOwnerOf(nftContract, tokenId);
    if (owner !== seller.address) {
        throw new Error('Not NFT owner');
    }

    // Get metadata
    const uri = await sdk.nftTokenUri(nftContract, tokenId);
    
    // Store listing in marketplace
    const listing = {
        tokenId,
        seller: seller.address,
        price,
        uri,
        timestamp: Date.now()
    };

    // ... marketplace logic ...

    await sdk.disconnect();
    return listing;
}
```

---

## Testing

Run SDK test suite:

```bash
# Start local devnet
./target/release/belizechain-node --dev --tmp

# In another terminal
cd gem/sdk
npm test
```

---

## Related Documentation

- [GEM Platform Overview](./gem-platform.md)
- [PSP22 Tokens](./gem-psp22-tokens.md)
- [PSP34 NFTs](./gem-psp34-nfts.md)
- [DAO Governance](./gem-dao-templates.md)
- [Deployment Guide](./gem-deployment-guide.md)
- [Code Examples](./gem-examples.md)

---

## Support

- **Documentation**: https://docs.belizechain.org
- **Discord**: https://discord.gg/belizechain
- **GitHub**: https://github.com/BelizeChain/gem-sdk
