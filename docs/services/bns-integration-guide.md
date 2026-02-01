# BNS Integration Guide

**Integrate .bz Domains Into Your dApp**

Complete guide for developers integrating BNS functionality into applications, wallets, and services.

---

## Integration Overview

BNS can be integrated for:
- **Payment Routing**: Send DALLA to domains instead of addresses
- **Identity Resolution**: Display domain names instead of addresses
- **Website Hosting**: Decentralized content delivery
- **Domain Marketplace**: Domain trading platforms
- **Profile Systems**: User profiles linked to domains

---

## Quick Start

### Installation

```bash
# Install Polkadot.js API
npm install @polkadot/api @polkadot/api-contract

# Or with TypeScript support
npm install @polkadot/api @polkadot/api-contract @polkadot/types
```

### Connect to BelizeChain

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function connect() {
    const provider = new WsProvider('wss://rpc.belizechain.org');
    const api = await ApiPromise.create({ provider });
    
    await api.isReady;
    console.log('Connected to BelizeChain');
    
    return api;
}
```

---

## Use Case 1: Payment Routing

### Resolve Domain to Wallet

```javascript
/**
 * Resolve .bz domain to wallet address
 * @param {string} domain - Domain name (with or without .bz)
 * @returns {Promise<string|null>} Wallet address or null
 */
async function resolveWallet(api, domain) {
    // Normalize domain (remove .bz if present)
    const domainName = domain.replace('.bz', '');
    
    // Query resolution records
    const resolution = await api.query.bns.domainResolution(domainName);
    
    if (resolution.isNone) {
        return null;  // Domain not found or no wallet set
    }
    
    const data = resolution.unwrap();
    
    if (data.walletAddress.isNone) {
        return null;  // No wallet address configured
    }
    
    return data.walletAddress.unwrap().toString();
}

// Usage
const wallet = await resolveWallet(api, 'alice.bz');
if (wallet) {
    console.log('alice.bz → ', wallet);
    // Now send payment to this address
} else {
    console.log('Domain has no wallet configured');
}
```

---

### Send Payment to Domain

```javascript
/**
 * Send DALLA payment to domain
 * @param {KeyringPair} sender - Sender account
 * @param {string} domain - Recipient domain
 * @param {string} amount - Amount in DALLA
 */
async function sendToDomain(api, sender, domain, amount) {
    // Resolve domain to wallet
    const recipient = await resolveWallet(api, domain);
    
    if (!recipient) {
        throw new Error(`Domain ${domain} has no wallet configured`);
    }
    
    // Convert DALLA to planck
    const amountPlanck = amount * 1e12;
    
    // Send transaction
    await api.tx.balances.transfer(
        recipient,
        amountPlanck
    ).signAndSend(sender, ({ status, events }) => {
        if (status.isInBlock) {
            console.log(`✅ Sent ${amount} DALLA to ${domain}`);
            console.log(`   Recipient: ${recipient}`);
        }
    });
}

// Usage
await sendToDomain(api, alice, 'bob.bz', 100);
```

---

## Use Case 2: Identity Display

### Get Domain for Address

```javascript
/**
 * Reverse lookup: find primary domain for address
 * @param {string} address - Account address
 * @returns {Promise<string|null>} Primary domain or null
 */
async function getPrimaryDomain(api, address) {
    // Get all domains owned by account
    const domains = await api.query.bns.accountDomains(address);
    
    if (domains.length === 0) {
        return null;
    }
    
    // Return first domain (in future, support "primary" flag)
    return domains[0].toString() + '.bz';
}

// Usage
const domain = await getPrimaryDomain(api, '5GrwvaEF...');
console.log(domain);  // Output: alice.bz
```

---

### Display Name Component (React)

```tsx
import React, { useState, useEffect } from 'react';
import { ApiPromise } from '@polkadot/api';

interface DisplayNameProps {
    api: ApiPromise;
    address: string;
    showAddress?: boolean;
}

const DisplayName: React.FC<DisplayNameProps> = ({ 
    api, 
    address, 
    showAddress = false 
}) => {
    const [domain, setDomain] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        async function loadDomain() {
            try {
                const domains = await api.query.bns.accountDomains(address);
                if (domains.length > 0) {
                    setDomain(domains[0].toString() + '.bz');
                }
            } catch (error) {
                console.error('Failed to load domain:', error);
            } finally {
                setLoading(false);
            }
        }

        loadDomain();
    }, [api, address]);

    if (loading) {
        return <span>Loading...</span>;
    }

    if (domain) {
        return (
            <div>
                <strong>{domain}</strong>
                {showAddress && <span className="text-muted"> ({address.substring(0, 10)}...)</span>}
            </div>
        );
    }

    return <span>{address.substring(0, 10)}...</span>;
};

export default DisplayName;
```

---

## Use Case 3: Domain Marketplace Integration

### List Active Domains for Sale

```javascript
/**
 * Get all active marketplace listings
 * @returns {Promise<Array>} Array of listings
 */
async function getMarketplaceListings(api) {
    // Get all listings
    const entries = await api.query.bns.domainListings.entries();
    
    // Get current block for expiry check
    const currentBlock = await api.query.system.number();
    const current = currentBlock.toNumber();
    
    // Filter and format
    const listings = entries
        .map(([key, listing]) => {
            const domain = key.args[0].toString();
            const data = listing.unwrap();
            
            return {
                domain: domain + '.bz',
                seller: data.seller.toString(),
                price: data.price.toNumber() / 1e12,  // Convert to DALLA
                listedAt: data.listedAt.toNumber(),
                expiresAt: data.expiresAt.toNumber(),
                minOffer: data.minOffer.isSome ? data.minOffer.unwrap().toNumber() / 1e12 : null,
                active: data.expiresAt.toNumber() > current
            };
        })
        .filter(l => l.active);  // Only show active listings
    
    return listings;
}

// Usage
const listings = await getMarketplaceListings(api);
console.log(`Found ${listings.length} active listings`);

listings.forEach(listing => {
    console.log(`${listing.domain} - ${listing.price} DALLA by ${listing.seller.substring(0, 10)}...`);
});
```

---

### Buy Domain from Marketplace

```javascript
/**
 * Purchase domain from marketplace
 * @param {KeyringPair} buyer - Buyer account
 * @param {string} domain - Domain to purchase
 */
async function buyDomain(api, buyer, domain) {
    // Normalize domain name
    const domainName = domain.replace('.bz', '');
    
    // Get listing to check price
    const listing = await api.query.bns.domainListings(domainName);
    
    if (listing.isNone) {
        throw new Error(`${domain} is not listed for sale`);
    }
    
    const data = listing.unwrap();
    const price = data.price.toNumber() / 1e12;
    const marketplaceFee = price * 0.05;
    const total = price;
    
    console.log(`Buying ${domain} for ${price} DALLA (${marketplaceFee} DALLA marketplace fee)`);
    
    // Execute purchase
    await api.tx.bns.buyDomain(domainName).signAndSend(buyer, ({ status, events }) => {
        if (status.isInBlock) {
            console.log('✅ Domain purchased!');
            
            // Find DomainSold event
            events.forEach(({ event }) => {
                if (event.section === 'bns' && event.method === 'DomainSold') {
                    const [domain, seller, buyer, price, fee] = event.data;
                    console.log(`Domain: ${domain.toString()}.bz`);
                    console.log(`Paid: ${price.toNumber() / 1e12} DALLA`);
                    console.log(`Fee: ${fee.toNumber() / 1e12} DALLA`);
                }
            });
        }
    });
}

// Usage
await buyDomain(api, bob, 'alice.bz');
```

---

## Use Case 4: Website Hosting Integration

### Resolve Domain to Website Content

```javascript
/**
 * Resolve domain to website content hash
 * @param {string} domain - Domain name
 * @returns {Promise<string|null>} DAG content hash
 */
async function resolveContent(api, domain) {
    const domainName = domain.replace('.bz', '');
    
    // Get hosting info
    const hosting = await api.query.bns.hostedWebsites(domainName);
    
    if (hosting.isNone) {
        return null;  // No hosting active
    }
    
    const data = hosting.unwrap();
    
    // Check if hosting expired
    const currentBlock = await api.query.system.number();
    if (data.expiresAt.toNumber() < currentBlock.toNumber()) {
        return null;  // Hosting expired
    }
    
    return data.contentHash.toHex();
}

// Usage
const contentHash = await resolveContent(api, 'alice.bz');
if (contentHash) {
    console.log('alice.bz → DAG:', contentHash);
    // Fetch content from Pakit using this hash
} else {
    console.log('No website hosted');
}
```

---

### Web Gateway Implementation

```javascript
const express = require('express');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { getDagBlock } = require('@belizechain/pakit-client');

const app = express();

// Connect to BelizeChain
let api;
(async () => {
    const provider = new WsProvider('wss://rpc.belizechain.org');
    api = await ApiPromise.create({ provider });
})();

// Gateway route: https://gateway.example.com/:domain/*
app.get('/:domain/*', async (req, res) => {
    try {
        const domain = req.params.domain;
        const path = req.params[0] || 'index.html';
        
        // Resolve domain to content hash
        const domainName = domain.replace('.bz', '');
        const hosting = await api.query.bns.hostedWebsites(domainName);
        
        if (hosting.isNone) {
            return res.status(404).send('Domain not hosted');
        }
        
        const data = hosting.unwrap();
        const contentHash = data.contentHash.toHex();
        
        // Fetch from Pakit DAG
        const content = await getDagBlock(contentHash, path);
        
        // Determine content type
        const ext = path.split('.').pop();
        const contentType = {
            'html': 'text/html',
            'css': 'text/css',
            'js': 'application/javascript',
            'json': 'application/json',
            'png': 'image/png',
            'jpg': 'image/jpeg',
            'svg': 'image/svg+xml'
        }[ext] || 'application/octet-stream';
        
        res.setHeader('Content-Type', contentType);
        res.send(content);
        
    } catch (error) {
        console.error('Gateway error:', error);
        res.status(500).send('Internal server error');
    }
});

app.listen(3000, () => {
    console.log('BNS Gateway running on port 3000');
});
```

---

## Use Case 5: Profile System

### Get Complete Profile

```javascript
/**
 * Get full profile for domain
 * @param {string} domain - Domain name
 * @returns {Promise<Object>} Profile data
 */
async function getProfile(api, domain) {
    const domainName = domain.replace('.bz', '');
    
    // Get domain record
    const domainRecord = await api.query.bns.domainRegistry(domainName);
    if (domainRecord.isNone) {
        throw new Error('Domain not found');
    }
    
    // Get resolution data
    const resolution = await api.query.bns.domainResolution(domainName);
    
    const record = domainRecord.unwrap();
    const profile = {
        domain: domain,
        owner: record.owner.toString(),
        registeredAt: record.registeredAt.toNumber(),
        tier: record.tier.toString(),
        transferCount: record.transferCount.toNumber()
    };
    
    if (resolution.isSome) {
        const data = resolution.unwrap();
        
        profile.wallet = data.walletAddress.isSome ? 
            data.walletAddress.unwrap().toString() : null;
        
        profile.contentHash = data.contentHash.isSome ? 
            data.contentHash.unwrap().toHex() : null;
        
        profile.avatar = data.avatar.isSome ? 
            data.avatar.unwrap().toHex() : null;
        
        try {
            profile.metadata = JSON.parse(data.metadata.toString());
        } catch {
            profile.metadata = {};
        }
        
        profile.textRecords = data.textRecords.map(record => ({
            key: record.key.toString(),
            value: record.value.toString()
        }));
    }
    
    return profile;
}

// Usage
const profile = await getProfile(api, 'alice.bz');
console.log(profile);

/* Output:
{
    domain: 'alice.bz',
    owner: '5GrwvaEF...',
    registeredAt: 123456,
    tier: 'Standard',
    transferCount: 0,
    wallet: '5GrwvaEF...',
    contentHash: '0x1a2b3c...',
    avatar: '0xfedcba...',
    metadata: {
        bio: 'Software Developer',
        location: 'Belize City'
    },
    textRecords: [
        { key: 'email', value: 'alice@example.com' },
        { key: 'twitter', value: '@alice_dev' }
    ]
}
*/
```

---

## Event Subscription

### Listen for Domain Registrations

```javascript
/**
 * Subscribe to new domain registrations
 * @param {Function} callback - Called for each new domain
 */
function subscribeToRegistrations(api, callback) {
    api.query.system.events((events) => {
        events.forEach((record) => {
            const { event } = record;
            
            if (event.section === 'bns' && event.method === 'DomainRegistered') {
                const [domain, owner, price, tier] = event.data;
                
                callback({
                    domain: domain.toString() + '.bz',
                    owner: owner.toString(),
                    price: price.toNumber() / 1e12,
                    tier: tier.toNumber(),
                    block: record.block.toNumber()
                });
            }
        });
    });
}

// Usage
subscribeToRegistrations(api, (registration) => {
    console.log('New domain registered:', registration.domain);
    console.log('Owner:', registration.owner);
    console.log('Price:', registration.price, 'DALLA');
});
```

---

### Monitor Domain Transfers

```javascript
/**
 * Track when specific domain changes ownership
 * @param {string} domain - Domain to monitor
 * @param {Function} callback - Called on transfer
 */
function watchDomain(api, domain, callback) {
    const domainName = domain.replace('.bz', '');
    
    api.query.system.events((events) => {
        events.forEach((record) => {
            const { event } = record;
            
            if (event.section === 'bns' && event.method === 'DomainTransferred') {
                const [transferredDomain, from, to] = event.data;
                
                if (transferredDomain.toString() === domainName) {
                    callback({
                        domain: domain,
                        from: from.toString(),
                        to: to.toString(),
                        block: record.block.toNumber()
                    });
                }
            }
        });
    });
}

// Usage
watchDomain(api, 'alice.bz', (transfer) => {
    console.log(`alice.bz transferred from ${transfer.from} to ${transfer.to}`);
});
```

---

## Best Practices

### Caching

Cache resolution data to reduce RPC calls:

```javascript
const cache = new Map();
const CACHE_TTL = 60000;  // 60 seconds

async function resolveWalletCached(api, domain) {
    const cacheKey = `wallet:${domain}`;
    const cached = cache.get(cacheKey);
    
    if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
        return cached.value;
    }
    
    const wallet = await resolveWallet(api, domain);
    
    cache.set(cacheKey, {
        value: wallet,
        timestamp: Date.now()
    });
    
    return wallet;
}
```

---

### Error Handling

Always handle domain resolution failures gracefully:

```javascript
async function safeSendToDomain(api, sender, domain, amount) {
    try {
        const wallet = await resolveWallet(api, domain);
        
        if (!wallet) {
            console.error(`Domain ${domain} has no wallet configured`);
            // Fallback: prompt user for manual address
            return null;
        }
        
        await sendPayment(api, sender, wallet, amount);
        return wallet;
        
    } catch (error) {
        console.error(`Failed to send to ${domain}:`, error);
        // Log error, show user-friendly message
        return null;
    }
}
```

---

### TypeScript Types

```typescript
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';

interface DomainRecord {
    owner: string;
    originalOwner: string;
    registeredAt: number;
    purchasePrice: number;
    tier: string;
    lockedUntil: number | null;
    transferCount: number;
}

interface ResolutionRecords {
    walletAddress: string | null;
    contentHash: string | null;
    avatar: string | null;
    metadata: Record<string, any>;
    textRecords: TextRecord[];
}

interface TextRecord {
    key: string;
    value: string;
}

interface MarketplaceListing {
    domain: string;
    seller: string;
    price: number;
    listedAt: number;
    expiresAt: number;
    minOffer: number | null;
    active: boolean;
}
```

---

## Related Documentation

- [BNS Overview](./bns-overview.md)
- [Domain Registration](./bns-domain-registry.md)
- [Marketplace](./bns-marketplace.md)
- [Web Hosting](./bns-ipfs-hosting.md)
- [API Reference](./bns-api-reference.md)
- [Pricing Details](./bns-pricing.md)

---

## Support

- **Developer Discord**: #dev-support channel
- **GitHub Examples**: https://github.com/BelizeChain/bns-examples
- **API Documentation**: https://docs.belizechain.org/api
