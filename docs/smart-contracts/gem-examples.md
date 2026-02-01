# GEM Smart Contract Examples

**Practical Code Examples for BelizeChain Smart Contract Development**

This guide provides complete, production-ready code examples for common smart contract use cases on the GEM platform.

---

## Table of Contents

1. [Beginner Examples](#beginner-examples)
2. [Intermediate Examples](#intermediate-examples)
3. [Advanced Examples](#advanced-examples)
4. [Integration Patterns](#integration-patterns)
5. [Testing Examples](#testing-examples)

---

## Beginner Examples

### Example 1: Query Token Balance

Simple script to check DALLA token balance.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function checkBalance() {
    // Connect to testnet
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    // DALLA token contract
    const dallaContract = '5GD4w5...NVsNB';
    
    // Check Alice's balance
    const alice = sdk.getAccount('//Alice');
    const balance = await sdk.dallaBalanceOf(dallaContract, alice.address);
    
    // Convert from planck to DALLA (12 decimals)
    const dallaAmount = parseInt(balance) / 1e12;
    
    console.log(`Alice's DALLA balance: ${dallaAmount.toFixed(2)}`);
    
    await sdk.disconnect();
}

checkBalance().catch(console.error);
```

**Expected Output:**
```
Alice's DALLA balance: 1234567.89
```

---

### Example 2: Transfer DALLA Tokens

Transfer tokens between accounts.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function transferTokens() {
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    const dallaContract = '5GD4w5...NVsNB';
    
    // Get accounts
    const alice = sdk.getAccount('//Alice');
    const bob = sdk.getAccount('//Bob');
    
    // Amount to transfer (100 DALLA)
    const amount = 100 * 1e12;
    
    console.log('Transferring 100 DALLA from Alice to Bob...');
    
    try {
        await sdk.dallaTransfer(dallaContract, alice, bob.address, amount.toString());
        console.log('✅ Transfer successful!');
        
        // Check new balance
        const bobBalance = await sdk.dallaBalanceOf(dallaContract, bob.address);
        console.log(`Bob's new balance: ${parseInt(bobBalance) / 1e12} DALLA`);
    } catch (error) {
        console.error('❌ Transfer failed:', error.message);
    }
    
    await sdk.disconnect();
}

transferTokens().catch(console.error);
```

---

### Example 3: Mint Your First NFT

Mint a BeLi cultural NFT.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function mintNFT() {
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    const beliContract = '5Ho6Ks...iFQL7';
    const alice = sdk.getAccount('//Alice');
    
    // NFT metadata (IPFS or HTTP)
    const metadata = {
        name: "Blue Hole #1",
        description: "The Great Blue Hole - Natural Wonder of Belize",
        image: "ipfs://QmBlueHoleImage...",
        attributes: [
            { trait_type: "Location", value: "Lighthouse Reef" },
            { trait_type: "Depth", value: "124 meters" },
            { trait_type: "Rarity", value: "Legendary" }
        ]
    };
    
    // Upload metadata to IPFS first
    // For this example, assume metadata already uploaded
    const metadataUri = 'ipfs://QmMetadataHash...';
    
    console.log('Minting Blue Hole NFT...');
    
    try {
        await sdk.nftMint(beliContract, alice, alice.address, metadataUri);
        console.log('✅ NFT minted successfully!');
        console.log('Metadata:', metadataUri);
    } catch (error) {
        console.error('❌ Minting failed:', error.message);
    }
    
    await sdk.disconnect();
}

mintNFT().catch(console.error);
```

---

## Intermediate Examples

### Example 4: Batch Token Transfers

Transfer tokens to multiple recipients efficiently.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function batchTransfer() {
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    const dallaContract = '5GD4w5...NVsNB';
    const alice = sdk.getAccount('//Alice');
    
    // Recipients and amounts
    const transfers = [
        { to: '5FHneW...JM694ty', amount: 100 },  // Bob: 100 DALLA
        { to: '5GNJqT...YNRY7w', amount: 250 },  // Charlie: 250 DALLA
        { to: '5FLSig...9pKdE6', amount: 500 }   // Dave: 500 DALLA
    ];
    
    console.log(`Starting batch transfer to ${transfers.length} recipients...`);
    
    for (const transfer of transfers) {
        try {
            const amount = transfer.amount * 1e12;
            await sdk.dallaTransfer(dallaContract, alice, transfer.to, amount.toString());
            console.log(`✅ Sent ${transfer.amount} DALLA to ${transfer.to.substring(0, 10)}...`);
        } catch (error) {
            console.error(`❌ Failed to send to ${transfer.to}:`, error.message);
        }
    }
    
    console.log('Batch transfer complete!');
    await sdk.disconnect();
}

batchTransfer().catch(console.error);
```

---

### Example 5: NFT Marketplace Listing

List NFT for sale with price and verification.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

class NFTMarketplace {
    constructor(nodeUrl) {
        this.sdk = new GemSDK(nodeUrl);
        this.beliContract = '5Ho6Ks...iFQL7';
        this.listings = new Map();  // In production, use database
    }

    async connect() {
        await this.sdk.connect();
    }

    async listForSale(seller, tokenId, priceInDalla) {
        // Verify ownership
        const owner = await this.sdk.nftOwnerOf(this.beliContract, tokenId);
        if (owner !== seller.address) {
            throw new Error('Not the owner of this NFT');
        }

        // Get NFT metadata
        const uri = await this.sdk.nftTokenUri(this.beliContract, tokenId);
        
        // Create listing
        const listing = {
            tokenId,
            seller: seller.address,
            price: priceInDalla,
            uri,
            timestamp: Date.now()
        };

        this.listings.set(tokenId, listing);
        
        console.log(`✅ Listed NFT #${tokenId} for ${priceInDalla} DALLA`);
        return listing;
    }

    async buyNFT(buyer, tokenId, dallaContract) {
        const listing = this.listings.get(tokenId);
        if (!listing) {
            throw new Error('NFT not listed');
        }

        // 1. Transfer DALLA from buyer to seller
        const priceInPlanck = listing.price * 1e12;
        await this.sdk.dallaTransfer(
            dallaContract,
            buyer,
            listing.seller,
            priceInPlanck.toString()
        );
        console.log(`✅ Payment of ${listing.price} DALLA sent`);

        // 2. Transfer NFT from seller to buyer
        // Note: Seller must approve marketplace first
        const sellerAccount = this.sdk.getAccount(process.env.SELLER_SEED);
        await this.sdk.nftTransfer(
            this.beliContract,
            sellerAccount,
            buyer.address,
            tokenId
        );
        console.log(`✅ NFT #${tokenId} transferred to buyer`);

        // Remove listing
        this.listings.delete(tokenId);
        
        return { success: true, tokenId, price: listing.price };
    }

    async disconnect() {
        await this.sdk.disconnect();
    }
}

// Usage
async function example() {
    const marketplace = new NFTMarketplace('wss://testnet.belizechain.org');
    await marketplace.connect();

    const alice = marketplace.sdk.getAccount('//Alice');
    const bob = marketplace.sdk.getAccount('//Bob');

    // Alice lists NFT #5 for 1000 DALLA
    await marketplace.listForSale(alice, 5, 1000);

    // Bob buys NFT #5
    const dallaContract = '5GD4w5...NVsNB';
    await marketplace.buyNFT(bob, 5, dallaContract);

    await marketplace.disconnect();
}

example().catch(console.error);
```

---

### Example 6: DAO Proposal Creation & Voting

Create and vote on governance proposals.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function daoGovernanceExample() {
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    const daoContract = '5DAOAddr...xyz';
    
    // Get accounts
    const alice = sdk.getAccount('//Alice');  // Proposer
    const bob = sdk.getAccount('//Bob');      // Voter
    const charlie = sdk.getAccount('//Charlie'); // Voter

    // 1. Alice creates proposal
    console.log('📝 Creating proposal...');
    const description = 'Allocate 50,000 DALLA to infrastructure development fund';
    
    const proposalId = await sdk.daoCreateProposal(daoContract, alice, description);
    console.log(`✅ Proposal #${proposalId} created`);

    // 2. Get proposal details
    const proposal = await sdk.daoGetProposal(daoContract, proposalId);
    console.log('\n📋 Proposal Details:');
    console.log('  Description:', proposal.description);
    console.log('  Proposer:', proposal.proposer.substring(0, 10) + '...');

    // 3. Bob votes FOR
    console.log('\n🗳️ Voting...');
    await sdk.daoVote(daoContract, bob, proposalId, true);
    console.log('✅ Bob voted FOR');

    // 4. Charlie votes AGAINST
    await sdk.daoVote(daoContract, charlie, proposalId, false);
    console.log('✅ Charlie voted AGAINST');

    // 5. Check updated vote counts
    const updatedProposal = await sdk.daoGetProposal(daoContract, proposalId);
    console.log('\n📊 Vote Results:');
    console.log(`  For: ${parseInt(updatedProposal.forVotes) / 1e12} DALLA`);
    console.log(`  Against: ${parseInt(updatedProposal.againstVotes) / 1e12} DALLA`);
    console.log(`  Status: ${updatedProposal.executed ? 'Executed' : 'Pending'}`);

    await sdk.disconnect();
}

daoGovernanceExample().catch(console.error);
```

---

## Advanced Examples

### Example 7: Cross-Contract DEX Swap

Swap DALLA for another PSP22 token through DEX.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function swapTokens() {
    const sdk = new GemSDK('wss://testnet.belizechain.org');
    await sdk.connect();

    const dallaContract = '5GD4w5...NVsNB';
    const usdtContract = '5USDT...xyz123';  // Example stablecoin
    const dexContract = '5DEXAddr...abc789';

    const alice = sdk.getAccount('//Alice');
    
    // Amount to swap: 1000 DALLA
    const swapAmount = 1000 * 1e12;
    
    console.log('Initiating DALLA -> USDT swap...');

    try {
        // 1. Approve DEX to spend DALLA
        console.log('1️⃣ Approving DEX...');
        await sdk.dallaApprove(dallaContract, alice, dexContract, swapAmount.toString());
        
        // 2. Execute swap via DEX contract
        console.log('2️⃣ Executing swap...');
        const contract = sdk.api.createType('ContractPromise', dexContract, dexAbi);
        
        await sdk.api.tx.contracts.call(
            dexContract,
            0,  // value
            -1, // gasLimit (auto)
            null, // storageDepositLimit
            contract.tx.swap(
                dallaContract,  // fromToken
                usdtContract,   // toToken
                swapAmount,     // amountIn
                950 * 1e12      // minAmountOut (5% slippage)
            )
        ).signAndSend(alice);

        console.log('✅ Swap successful!');
        
        // 3. Check new USDT balance
        const usdtBalance = await sdk.dallaBalanceOf(usdtContract, alice.address);
        console.log(`New USDT balance: ${parseInt(usdtBalance) / 1e12}`);
        
    } catch (error) {
        console.error('❌ Swap failed:', error.message);
    }

    await sdk.disconnect();
}

swapTokens().catch(console.error);
```

---

### Example 8: Automated Faucet Bot

Bot that monitors balance and auto-claims from faucet.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

class FaucetBot {
    constructor(nodeUrl, faucetContract) {
        this.sdk = new GemSDK(nodeUrl);
        this.faucetContract = faucetContract;
        this.minBalance = 100 * 1e12;  // Claim when below 100 DALLA
        this.checkInterval = 60000;     // Check every 60 seconds
    }

    async start(account) {
        await this.sdk.connect();
        console.log('🤖 Faucet bot started for', account.address.substring(0, 10) + '...');

        setInterval(async () => {
            try {
                // Check current balance
                const balance = await this.sdk.getBalance(account.address);
                const currentBalance = parseInt(balance.free);

                if (currentBalance < this.minBalance) {
                    console.log('💧 Balance low, attempting faucet claim...');
                    
                    // Check if eligible
                    const canClaim = await this.sdk.faucetCanClaim(
                        this.faucetContract,
                        account.address
                    );

                    if (canClaim) {
                        // Claim tokens
                        await this.sdk.faucetClaim(this.faucetContract, account);
                        console.log('✅ Claimed 1000 DALLA from faucet');
                    } else {
                        // Calculate wait time
                        const lastClaim = await this.sdk.faucetGetLastClaim(
                            this.faucetContract,
                            account.address
                        );
                        const currentBlock = await this.sdk.api.query.system.number();
                        const blocksRemaining = 100 - (currentBlock.toNumber() - lastClaim);
                        
                        console.log(`⏰ Must wait ${blocksRemaining} blocks (~${blocksRemaining / 10} min)`);
                    }
                } else {
                    console.log(`✅ Balance sufficient: ${currentBalance / 1e12} DALLA`);
                }
            } catch (error) {
                console.error('❌ Error:', error.message);
            }
        }, this.checkInterval);
    }

    async stop() {
        await this.sdk.disconnect();
        console.log('🛑 Faucet bot stopped');
    }
}

// Usage
async function runBot() {
    const bot = new FaucetBot(
        'wss://testnet.belizechain.org',
        '5FaucetAddr...xyz'
    );

    const account = bot.sdk.getAccount('//Alice');
    await bot.start(account);

    // Run for 1 hour then stop
    setTimeout(() => bot.stop(), 3600000);
}

runBot().catch(console.error);
```

---

## Integration Patterns

### Example 9: React Wallet Component

React component for wallet integration.

```typescript
import React, { useState, useEffect } from 'react';
import { GemSDK } from '@belizechain/gem-sdk';

interface WalletInfo {
    address: string;
    nativeBalance: number;
    dallaBalance: number;
}

const WalletConnect: React.FC = () => {
    const [sdk] = useState(new GemSDK('wss://testnet.belizechain.org'));
    const [wallet, setWallet] = useState<WalletInfo | null>(null);
    const [loading, setLoading] = useState(false);
    
    const dallaContract = '5GD4w5...NVsNB';

    const connectWallet = async (seedPhrase: string) => {
        setLoading(true);
        try {
            await sdk.connect();
            const account = sdk.getAccount(seedPhrase);
            
            // Get balances
            const native = await sdk.getBalance(account.address);
            const dalla = await sdk.dallaBalanceOf(dallaContract, account.address);
            
            setWallet({
                address: account.address,
                nativeBalance: parseInt(native.free) / 1e12,
                dallaBalance: parseInt(dalla) / 1e12
            });
        } catch (error) {
            console.error('Connection failed:', error);
        } finally {
            setLoading(false);
        }
    };

    const disconnect = async () => {
        await sdk.disconnect();
        setWallet(null);
    };

    if (!wallet) {
        return (
            <div className="wallet-connect">
                <button onClick={() => connectWallet('//Alice')} disabled={loading}>
                    {loading ? 'Connecting...' : 'Connect Wallet'}
                </button>
            </div>
        );
    }

    return (
        <div className="wallet-info">
            <p>Address: {wallet.address.substring(0, 10)}...</p>
            <p>Native Balance: {wallet.nativeBalance.toFixed(2)} DALLA</p>
            <p>Token Balance: {wallet.dallaBalance.toFixed(2)} DALLA</p>
            <button onClick={disconnect}>Disconnect</button>
        </div>
    );
};

export default WalletConnect;
```

---

### Example 10: Express API Backend

Node.js API for contract interactions.

```javascript
const express = require('express');
const { GemSDK } = require('@belizechain/gem-sdk');

const app = express();
app.use(express.json());

// Initialize SDK
const sdk = new GemSDK('wss://testnet.belizechain.org');
const dallaContract = '5GD4w5...NVsNB';

// Connect on startup
sdk.connect().then(() => console.log('Connected to BelizeChain'));

// Get balance endpoint
app.get('/api/balance/:address', async (req, res) => {
    try {
        const { address } = req.params;
        const balance = await sdk.dallaBalanceOf(dallaContract, address);
        
        res.json({
            address,
            balance: parseInt(balance) / 1e12,
            unit: 'DALLA'
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Transfer endpoint
app.post('/api/transfer', async (req, res) => {
    try {
        const { fromSeed, to, amount } = req.body;
        
        // Get sender account
        const sender = sdk.getAccount(fromSeed);
        
        // Convert to planck
        const amountPlanck = amount * 1e12;
        
        // Execute transfer
        await sdk.dallaTransfer(dallaContract, sender, to, amountPlanck.toString());
        
        res.json({
            success: true,
            from: sender.address,
            to,
            amount
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// NFT ownership endpoint
app.get('/api/nft/:tokenId', async (req, res) => {
    try {
        const beliContract = '5Ho6Ks...iFQL7';
        const { tokenId } = req.params;
        
        const owner = await sdk.nftOwnerOf(beliContract, parseInt(tokenId));
        const uri = await sdk.nftTokenUri(beliContract, parseInt(tokenId));
        
        res.json({
            tokenId: parseInt(tokenId),
            owner,
            metadataUri: uri
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => console.log(`API running on port ${PORT}`));
```

---

## Testing Examples

### Example 11: Unit Test Suite

Jest tests for contract interactions.

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

describe('DALLA Token Tests', () => {
    let sdk;
    let alice, bob;
    const dallaContract = '5GD4w5...NVsNB';

    beforeAll(async () => {
        sdk = new GemSDK('ws://localhost:9944');
        await sdk.connect();
        alice = sdk.getAccount('//Alice');
        bob = sdk.getAccount('//Bob');
    });

    afterAll(async () => {
        await sdk.disconnect();
    });

    test('should get token metadata', async () => {
        const metadata = await sdk.dallaMetadata(dallaContract);
        
        expect(metadata.name).toBe('DALLA');
        expect(metadata.symbol).toBe('DALLA');
        expect(metadata.decimals).toBe(12);
    });

    test('should transfer tokens successfully', async () => {
        const amount = 100 * 1e12;
        
        // Get initial balance
        const initialBalance = await sdk.dallaBalanceOf(dallaContract, bob.address);
        
        // Transfer
        await sdk.dallaTransfer(dallaContract, alice, bob.address, amount.toString());
        
        // Get new balance
        const newBalance = await sdk.dallaBalanceOf(dallaContract, bob.address);
        
        expect(parseInt(newBalance) - parseInt(initialBalance)).toBe(amount);
    });

    test('should fail transfer with insufficient balance', async () => {
        const hugeAmount = '9999999999999999999999';
        
        await expect(
            sdk.dallaTransfer(dallaContract, alice, bob.address, hugeAmount)
        ).rejects.toThrow();
    });
});
```

---

## Related Documentation

- [GEM Platform Overview](./gem-platform.md)
- [PSP22 Tokens](./gem-psp22-tokens.md)
- [PSP34 NFTs](./gem-psp34-nfts.md)
- [DAO Templates](./gem-dao-templates.md)
- [SDK Reference](./gem-sdk-reference.md)
- [Deployment Guide](./gem-deployment-guide.md)
- [Security Audit](./gem-security-audit.md)

---

## Additional Resources

- **Full Code Repository**: https://github.com/BelizeChain/gem-examples
- **Video Tutorials**: https://youtube.com/@belizechain
- **Developer Discord**: https://discord.gg/belizechain
- **Stack Overflow**: Tag `belizechain`

---

## Community Contributions

Submit your own examples via pull request:
1. Fork the repository
2. Add your example with complete code
3. Include documentation and tests
4. Submit PR with description

**Featured Contributors:**
- Examples 1-5: BelizeChain Core Team
- Example 6: Community contributor @dev_alice
- Example 7: Community contributor @blockchain_bob
