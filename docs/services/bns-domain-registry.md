# BNS Domain Registry Guide

**How to Register and Manage .bz Domains**

Complete guide to registering, configuring, and transferring domains on the Belize Name Service.

---

## Registration Process

### Prerequisites

1. **BelizeChain Account**: Active wallet with DALLA tokens
2. **KYC Verification**: Minimum Level 1 (basic verification)
3. **Sufficient Balance**: Tier price + gas fees

**Balance Requirements:**
- Standard domain (4+ chars): 100 DALLA + gas
- Premium domain (3 chars): 500 DALLA + gas
- Government domain: 50 DALLA + gas (requires verification)
- Verified domain: 1,000 DALLA + gas (requires Level 3 KYC)

---

## Step-by-Step Registration

### 1. Check Domain Availability

Before registering, verify the domain is available:

**Via UI:**
```
1. Visit https://bns.belizechain.org
2. Enter desired domain name
3. Click "Check Availability"
4. View results and pricing
```

**Via CLI:**
```bash
# Using Polkadot.js CLI
polkadot-js-api query.bns.domainRegistry alice.bz
# If returns null → Available
# If returns data → Already registered
```

**Via SDK:**
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

const sdk = new GemSDK('wss://rpc.belizechain.org');
await sdk.connect();

// Check if domain exists
const domain = await sdk.api.query.bns.domainRegistry('alice.bz');
if (domain.isNone) {
    console.log('✅ Available!');
} else {
    console.log('❌ Already registered');
}
```

---

### 2. Verify KYC Requirements

**Domain Tier → KYC Level Mapping:**

| Domain Tier | KYC Level Required | Verification Steps |
|-------------|-------------------|-------------------|
| Standard | Level 1 (Basic) | Email + phone verification |
| Premium | Level 1 (Basic) | Email + phone verification |
| Government | Government ID | .gov.bz email verification |
| Verified | Level 3 (Enhanced) | Full identity documents + address proof |

**Check Your KYC Status:**
```javascript
// Via SDK
const kycLevel = await sdk.api.query.identity.identityOf(alice.address);
console.log('KYC Level:', kycLevel.level);

// Level 0: Not verified
// Level 1: Basic (Standard/Premium domains)
// Level 2: Intermediate
// Level 3: Enhanced (Verified domains)
```

**Complete KYC:**
1. Navigate to Maya Wallet → BelizeID
2. Complete verification steps for your desired level
3. Wait for approval (typically 1-3 business days)
4. Return to domain registration

---

### 3. Register Domain

**Via Web UI (Recommended for Beginners):**

```
1. Visit https://bns.belizechain.org/register
2. Enter domain name (without .bz)
3. Select domain tier:
   - Standard (100 DALLA)
   - Premium (500 DALLA)
   - Verified (1,000 DALLA)
4. Review pricing and gas estimate
5. Connect wallet (Maya Wallet or Polkadot.js)
6. Click "Register Domain"
7. Sign transaction
8. Wait for confirmation (~12 seconds)
9. Domain is yours! 🎉
```

**Via Polkadot.js Apps:**

```
1. Go to https://polkadot.js.org/apps
2. Connect to BelizeChain RPC
3. Navigate to Developer → Extrinsics
4. Select account
5. Choose extrinsic: bns → register_domain
6. Enter parameters:
   - name: "alice" (without .bz)
   - tier: 0 (Standard), 1 (Premium), 2 (Government), 3 (Verified)
7. Submit transaction
8. Sign and wait for block confirmation
```

**Via SDK (For Developers):**

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function registerDomain() {
    const sdk = new GemSDK('wss://rpc.belizechain.org');
    await sdk.connect();

    const alice = sdk.getAccount(process.env.SEED_PHRASE);
    
    // Register Standard domain (100 DALLA)
    await sdk.api.tx.bns.registerDomain(
        'alice',  // Domain name (without .bz)
        0         // Tier: 0=Standard, 1=Premium, 2=Government, 3=Verified
    ).signAndSend(alice, ({ status, events }) => {
        if (status.isInBlock) {
            console.log('✅ Domain registered in block:', status.asInBlock.toHex());
            
            // Find DomainRegistered event
            events.forEach(({ event }) => {
                if (event.section === 'bns' && event.method === 'DomainRegistered') {
                    const [domain, owner, price, tier] = event.data;
                    console.log('Domain:', domain.toHuman());
                    console.log('Owner:', owner.toHuman());
                    console.log('Price:', price.toHuman());
                    console.log('Tier:', tier.toHuman());
                }
            });
        }
    });
}

registerDomain().catch(console.error);
```

---

### 4. Configure Domain Resolution

After registration, set up domain resolution records:

**Set Wallet Address:**
```javascript
// Point domain to your wallet for payments
await sdk.api.tx.bns.updateResolution(
    'alice',  // Domain name
    {
        walletAddress: alice.address,  // Payment destination
        contentHash: null,              // No website yet
        avatar: null,                   // No avatar
        metadata: '{}',                 // Empty metadata
        textRecords: []                 // No text records
    }
).signAndSend(alice);

// Now payments to "alice.bz" route to your wallet!
```

**Set Custom Text Records:**
```javascript
// Add email, social media, etc.
await sdk.api.tx.bns.updateResolution(
    'alice',
    {
        walletAddress: alice.address,
        contentHash: null,
        avatar: null,
        metadata: JSON.stringify({
            bio: 'Software Developer',
            location: 'Belize City'
        }),
        textRecords: [
            { key: 'email', value: 'alice@example.com' },
            { key: 'twitter', value: '@alice_dev' },
            { key: 'github', value: 'github.com/alice' },
            { key: 'url', value: 'https://alice.example.com' }
        ]
    }
).signAndSend(alice);
```

---

## Domain Management

### Query Domain Information

**Get Domain Details:**
```javascript
// Via SDK
const domain = await sdk.api.query.bns.domainRegistry('alice');

if (domain.isSome) {
    const record = domain.unwrap();
    console.log('Owner:', record.owner.toHuman());
    console.log('Registered At:', record.registeredAt.toHuman());
    console.log('Purchase Price:', record.purchasePrice.toHuman());
    console.log('Tier:', record.tier.toHuman());
    console.log('Transfer Count:', record.transferCount.toNumber());
}
```

**Get Resolution Records:**
```javascript
const resolution = await sdk.api.query.bns.domainResolution('alice');

if (resolution.isSome) {
    const data = resolution.unwrap();
    console.log('Wallet:', data.walletAddress.toHuman());
    console.log('Content Hash:', data.contentHash.toHuman());
    console.log('Avatar:', data.avatar.toHuman());
    console.log('Metadata:', JSON.parse(data.metadata.toString()));
    console.log('Text Records:', data.textRecords.toHuman());
}
```

---

### Transfer Domain Ownership

**Transfer Lock Period:**
- New domains: 7-day transfer lock (50,400 blocks)
- Recently purchased: 7-day lock from purchase date
- Purpose: Prevent rapid flipping and scams

**Check Transfer Lock:**
```javascript
const domain = await sdk.api.query.bns.domainRegistry('alice');
const record = domain.unwrap();

if (record.lockedUntil.isSome) {
    const lockBlock = record.lockedUntil.unwrap().toNumber();
    const currentBlock = await sdk.api.query.system.number();
    const blocksRemaining = lockBlock - currentBlock.toNumber();
    
    if (blocksRemaining > 0) {
        const hoursRemaining = (blocksRemaining * 6) / 3600;
        console.log(`🔒 Transfer locked for ${hoursRemaining.toFixed(1)} hours`);
    } else {
        console.log('✅ Transfer unlocked');
    }
}
```

**Execute Transfer:**
```javascript
// Transfer domain to Bob
await sdk.api.tx.bns.transferDomain(
    'alice',          // Domain name
    bob.address       // New owner
).signAndSend(alice);

// Event emitted:
// DomainTransferred { domain: 'alice', from: Alice, to: Bob }
```

---

### Update Resolution Records

**Change Wallet Address:**
```javascript
// Update payment destination
await sdk.api.tx.bns.updateResolution(
    'alice',
    {
        walletAddress: newAddress,  // New wallet
        // ... keep other fields unchanged
    }
).signAndSend(alice);
```

**Update Avatar:**
```javascript
// Set profile image (DAG hash or IPFS)
const avatarHash = '0x1a2b3c4d...';  // 32-byte hash
await sdk.api.tx.bns.updateResolution(
    'alice',
    {
        avatar: avatarHash,
        // ... keep other fields
    }
).signAndSend(alice);
```

**Update Metadata:**
```javascript
// Update JSON metadata
const newMetadata = {
    bio: 'Updated bio',
    location: 'San Pedro',
    occupation: 'Blockchain Developer',
    website: 'https://alice.bz'
};

await sdk.api.tx.bns.updateResolution(
    'alice',
    {
        metadata: JSON.stringify(newMetadata),
        // ... keep other fields
    }
).signAndSend(alice);
```

---

## Subdomain Creation

**Coming in Phase 2 (Q2 2026)**

Subdomain delegation will allow domain owners to create subdomains:
- `blog.alice.bz` (owned by alice.bz owner)
- `shop.alice.bz` (delegated to another account)
- `api.alice.bz` (programmatic access)

**Planned Features:**
- Owner-controlled subdomain creation
- Delegated subdomain management
- Separate resolution records per subdomain
- Marketplace for subdomain sales

---

## Domain Naming Rules

### Valid Characters
- **Allowed**: lowercase letters (a-z), numbers (0-9), hyphens (-)
- **Not Allowed**: uppercase, spaces, special characters (@, #, $, etc.)
- **Cannot**: Start or end with hyphen
- **Cannot**: Contain consecutive hyphens (--)

**Valid Examples:**
```
✅ alice.bz
✅ alice-smith.bz
✅ alice123.bz
✅ my-cool-domain.bz
```

**Invalid Examples:**
```
❌ Alice.bz (uppercase)
❌ alice smith.bz (space)
❌ alice@smith.bz (special char)
❌ -alice.bz (starts with hyphen)
❌ alice-.bz (ends with hyphen)
❌ alice--smith.bz (consecutive hyphens)
```

---

### Length Requirements

| Tier | Minimum Length | Maximum Length |
|------|----------------|----------------|
| Standard | 4 characters | 63 characters |
| Premium | 3 characters | 63 characters |
| Government | 4 characters | 63 characters |
| Verified | 3 characters | 63 characters |

**Short Domain Pricing:**
- 3 characters: Premium tier only (500 DALLA)
- 4+ characters: Standard tier (100 DALLA)
- 1-2 characters: Reserved for governance/future use

---

## Reserved Domains

The following domains are reserved and cannot be registered:

**System Reserved:**
- `root.bz`, `system.bz`, `admin.bz`, `www.bz`
- `api.bz`, `rpc.bz`, `ws.bz`, `node.bz`

**Government Reserved:**
- `gov.bz`, `parliament.bz`, `pm.bz`, `police.bz`
- `immigration.bz`, `treasury.bz`, `centralbank.bz`

**Premium Reserved (Auction Only):**
- `belize.bz`, `btl.bz`, `bank.bz`, `hotel.bz`
- Common words (dictionary.bz, beach.bz, reef.bz)

**How to Bid:**
Premium reserved domains will be auctioned through governance proposals in Phase 2.

---

## Bulk Registration

Register multiple domains in one transaction:

```javascript
// Batch registration (saves on gas fees)
const domains = ['alice', 'alice-dev', 'alice-portfolio'];

// Prepare batch call
const calls = domains.map(name => 
    sdk.api.tx.bns.registerDomain(name, 0)  // All Standard tier
);

// Execute batch
await sdk.api.tx.utility.batch(calls).signAndSend(alice, ({ status }) => {
    if (status.isInBlock) {
        console.log(`✅ Registered ${domains.length} domains!`);
    }
});

// Gas savings: ~30% compared to individual transactions
```

---

## Common Errors

### Error: `DomainAlreadyExists`
**Cause:** Domain is already registered by another user  
**Solution:** Choose a different domain name

### Error: `DomainTooShort`
**Cause:** Domain name less than minimum length for tier  
**Solution:** Use longer name or upgrade to Premium tier (3 chars)

### Error: `InsufficientBalance`
**Cause:** Not enough DALLA tokens for registration  
**Solution:** Add funds to your wallet

### Error: `KYCRequired`
**Cause:** Account lacks required KYC verification  
**Solution:** Complete KYC in Maya Wallet → BelizeID

### Error: `MaxDomainsReached`
**Cause:** Account owns maximum allowed domains (100)  
**Solution:** Transfer or sell existing domains first

### Error: `InvalidDomainCharacters`
**Cause:** Domain contains uppercase or special characters  
**Solution:** Use only lowercase letters, numbers, and hyphens

---

## Best Practices

### Choosing a Good Domain

**✅ Do:**
- Keep it short and memorable
- Use your brand name or personal name
- Check trademark conflicts
- Consider future uses (resale, subdomains)

**❌ Don't:**
- Register trademarked names (risk blacklist)
- Use offensive or illegal terms
- Squat on high-value names (premium tier exists)
- Register domains you won't use (ties up namespace)

---

### Security Tips

**Protect Your Domain:**
1. **Secure Your Wallet**: Use hardware wallet for domain owner account
2. **Enable Transfer Lock**: Don't unlock unless actively selling
3. **Monitor Changes**: Subscribe to domain events
4. **Backup Seed Phrase**: Losing seed = losing domain forever
5. **Use Multi-Sig**: For high-value domains, use multi-signature account

**Avoid Scams:**
- Never share your seed phrase
- Verify marketplace URLs (only use official marketplace)
- Check domain ownership before purchase
- Use escrow for large transactions

---

## Related Documentation

- [BNS Overview](./bns-overview.md)
- [Domain Marketplace](./bns-marketplace.md)
- [Web Hosting Setup](./bns-ipfs-hosting.md)
- [Pricing Details](./bns-pricing.md)
- [API Reference](./bns-api-reference.md)
- [Integration Guide](./bns-integration-guide.md)

---

## Support

- **Domain Search**: https://bns.belizechain.org
- **Discord**: #bns-support channel
- **Troubleshooting**: https://docs.belizechain.org/bns/troubleshooting
- **Report Issues**: security@belizechain.org
