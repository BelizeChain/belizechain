# BNS Domain Marketplace

**Buy, Sell, and Trade .bz Domains**

Complete guide to the built-in BNS marketplace for domain trading with automated escrow and 5% treasury fees.

---

## Marketplace Overview

The BNS marketplace is a decentralized platform for trading .bz domains with:
- **Automated Escrow**: Safe, trustless transactions
- **Fixed Price Listings**: Set your price and wait for buyers
- **Auction Support**: Coming in Phase 2 (Q2 2026)
- **5% Treasury Fee**: Supports BelizeChain development
- **Instant Transfers**: Ownership changes on purchase

---

## Listing Domains for Sale

### Prerequisites

Before listing a domain:
- ✅ You must own the domain
- ✅ Domain transfer lock must be expired
- ✅ Domain must not be hosting an active website (pause hosting first)

### List Domain (Fixed Price)

**Via Web UI:**
```
1. Visit https://bns.belizechain.org/marketplace
2. Click "My Domains"
3. Select domain to sell
4. Click "List for Sale"
5. Enter asking price (in DALLA)
6. Set listing duration (7-90 days)
7. Optional: Set minimum offer for negotiations
8. Review marketplace fee (5%)
9. Click "Create Listing"
10. Sign transaction
```

**Via SDK:**
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function listDomain() {
    const sdk = new GemSDK('wss://rpc.belizechain.org');
    await sdk.connect();

    const alice = sdk.getAccount(process.env.SEED_PHRASE);
    
    // List "alice.bz" for 1,000 DALLA
    const price = 1000 * 1e12;  // Convert to planck
    const duration = 30 * 24 * 600;  // 30 days in blocks (6s blocks)
    
    await sdk.api.tx.bns.listForSale(
        'alice',      // Domain name (without .bz)
        price,        // Asking price
        duration,     // Listing duration (blocks)
        null          // Minimum offer (null = no offers)
    ).signAndSend(alice, ({ status, events }) => {
        if (status.isInBlock) {
            console.log('✅ Domain listed for sale!');
            
            // Find DomainListed event
            events.forEach(({ event }) => {
                if (event.method === 'DomainListed') {
                    const [domain, seller, price] = event.data;
                    console.log('Domain:', domain.toHuman());
                    console.log('Price:', (price.toNumber() / 1e12).toFixed(2), 'DALLA');
                }
            });
        }
    });
}

listDomain().catch(console.error);
```

---

### Listing Parameters

**Price:**
- Minimum: 10 DALLA (prevents spam)
- Maximum: No limit
- Precision: 12 decimals (DALLA standard)

**Duration:**
- Minimum: 7 days (100,800 blocks)
- Maximum: 90 days (1,296,000 blocks)
- Auto-delist: Listing expires automatically after duration

**Minimum Offer:**
- Optional: Set lowest acceptable offer
- Use Case: Enable negotiations below asking price
- Example: Asking 1,000 DALLA, min offer 800 DALLA

---

## Buying Domains

### Browse Marketplace

**Via Web UI:**
```
1. Visit https://bns.belizechain.org/marketplace
2. Filter by:
   - Price range
   - Domain length
   - Domain tier (Standard/Premium/Verified)
   - Recently listed
3. Search for specific names
4. Sort by price, age, popularity
```

**Via SDK (Query Listings):**
```javascript
// Get all active listings
const listings = await sdk.api.query.bns.domainListings.entries();

const activeListings = listings
    .map(([key, listing]) => {
        const domain = key.args[0].toHuman();
        const data = listing.unwrap();
        return {
            domain,
            seller: data.seller.toHuman(),
            price: data.price.toNumber() / 1e12,
            listedAt: data.listedAt.toNumber(),
            expiresAt: data.expiresAt.toNumber()
        };
    })
    .filter(listing => {
        const currentBlock = sdk.api.query.system.number();
        return listing.expiresAt > currentBlock;  // Not expired
    });

console.log('Active Listings:', activeListings);
```

---

### Purchase Domain

**Via Web UI:**
```
1. Find domain on marketplace
2. Click domain to view details
3. Review:
   - Current owner
   - Asking price
   - Marketplace fee (5%)
   - Total cost
4. Click "Buy Now"
5. Confirm payment
6. Sign transaction
7. Ownership transferred instantly! 🎉
```

**Via SDK:**
```javascript
async function buyDomain() {
    const sdk = new GemSDK('wss://rpc.belizechain.org');
    await sdk.connect();

    const bob = sdk.getAccount(process.env.BUYER_SEED);
    
    // Buy "alice.bz" from marketplace
    await sdk.api.tx.bns.buyDomain(
        'alice'  // Domain name
    ).signAndSend(bob, ({ status, events }) => {
        if (status.isInBlock) {
            console.log('✅ Domain purchased!');
            
            // Find DomainSold event
            events.forEach(({ event }) => {
                if (event.method === 'DomainSold') {
                    const [domain, seller, buyer, price, fee] = event.data;
                    console.log('Domain:', domain.toHuman());
                    console.log('Seller:', seller.toHuman());
                    console.log('Buyer:', buyer.toHuman());
                    console.log('Price:', (price.toNumber() / 1e12).toFixed(2), 'DALLA');
                    console.log('Marketplace Fee:', (fee.toNumber() / 1e12).toFixed(2), 'DALLA');
                }
            });
        }
    });
}

buyDomain().catch(console.error);
```

---

### Transaction Flow

```
┌─────────────────────────────────────┐
│  1. Buyer initiates purchase        │
│     buyDomain("alice")               │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  2. Marketplace validates:           │
│     ✓ Domain is listed               │
│     ✓ Listing not expired            │
│     ✓ Buyer has sufficient balance   │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  3. Payment processing:              │
│     Total: 1,000 DALLA               │
│     To Seller: 950 DALLA (95%)       │
│     To Treasury: 50 DALLA (5% fee)   │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  4. Ownership transfer:              │
│     Domain owner: Alice → Bob        │
│     Transfer count: +1               │
│     Transfer lock: 7 days            │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  5. Listing removed:                 │
│     Domain delisted from marketplace │
│     Event: DomainSold emitted        │
└─────────────────────────────────────┘
```

---

## Marketplace Economics

### Fee Structure

**5% Marketplace Fee:**
- Applied to seller's proceeds
- Deposited to BelizeChain Treasury
- Funds governance and development

**Example Transaction:**
```
Asking Price: 1,000 DALLA
─────────────────────────
Buyer Pays:     1,000 DALLA
Seller Receives:  950 DALLA (95%)
Treasury Fee:      50 DALLA (5%)
─────────────────────────
Total: 1,000 DALLA
```

**Gas Costs:**
- List domain: ~12,000 gas (~0.000012 DALLA)
- Buy domain: ~25,000 gas (~0.000025 DALLA)
- Delist domain: ~10,000 gas (~0.00001 DALLA)

---

### Price Discovery

**Recent Sales:**
```javascript
// Query recent domain sales
const events = await sdk.api.query.system.events();

const sales = events
    .filter(record => {
        const { event } = record;
        return event.section === 'bns' && event.method === 'DomainSold';
    })
    .map(record => {
        const [domain, seller, buyer, price, fee] = record.event.data;
        return {
            domain: domain.toHuman(),
            price: price.toNumber() / 1e12,
            timestamp: record.block.toNumber()
        };
    });

console.log('Recent Sales:', sales);
```

**Price Trends:**
- 3-letter domains: 500-2,000 DALLA
- 4-letter domains: 200-800 DALLA
- 5+ letter domains: 100-500 DALLA
- Premium/verified: 1,000-10,000 DALLA

---

## Managing Listings

### Update Listing Price

```javascript
// Cancel existing listing
await sdk.api.tx.bns.delistDomain('alice').signAndSend(alice);

// Create new listing with updated price
await sdk.api.tx.bns.listForSale(
    'alice',
    1200 * 1e12,  // New price: 1,200 DALLA
    30 * 24 * 600
).signAndSend(alice);
```

---

### Cancel Listing

**Via Web UI:**
```
1. Go to "My Domains"
2. Find listed domain
3. Click "Cancel Listing"
4. Confirm transaction
```

**Via SDK:**
```javascript
// Delist domain
await sdk.api.tx.bns.delistDomain(
    'alice'  // Domain name
).signAndSend(alice);

// Domain removed from marketplace
// No penalty for delisting
```

---

### Auto-Expiry

Listings automatically expire after the specified duration:
- No manual action needed
- Domain remains yours
- Can relist anytime
- No fees for expiration

**Check Expiry:**
```javascript
const listing = await sdk.api.query.bns.domainListings('alice');
if (listing.isSome) {
    const data = listing.unwrap();
    const currentBlock = await sdk.api.query.system.number();
    const blocksRemaining = data.expiresAt.toNumber() - currentBlock.toNumber();
    const daysRemaining = (blocksRemaining * 6) / 86400;
    
    console.log(`Listing expires in ${daysRemaining.toFixed(1)} days`);
}
```

---

## Advanced Features (Phase 2)

### Auction System (Coming Q2 2026)

**English Auctions:**
- Set starting bid and reserve price
- Bidders compete in ascending price auction
- Highest bid wins after auction period
- 5% fee on final sale price

**Dutch Auctions:**
- Start at high price, decrease over time
- First bid at current price wins
- Faster price discovery
- No bidding wars

### Offer System (Coming Q2 2026)

**Make Offers:**
- Buyers propose prices below asking
- Sellers accept/reject/counter
- Negotiation built into marketplace
- 5% fee on accepted offers

**Example:**
```
Domain: alice.bz (listed at 1,000 DALLA)
Buyer offers: 850 DALLA
Seller counters: 925 DALLA
Buyer accepts
Sale: 925 DALLA (87.875 to seller, 46.25 fee)
```

---

## Marketplace Analytics

### View Market Stats

```javascript
// Total domains listed
const totalListings = await sdk.api.query.bns.domainListings.entries();
console.log('Active Listings:', totalListings.length);

// Total marketplace revenue
const totalRevenue = await sdk.api.query.bns.totalMarketplaceRevenue();
console.log('Total Revenue:', (totalRevenue.toNumber() / 1e12).toFixed(2), 'DALLA');

// Average sale price (calculate from events)
const avgPrice = sales.reduce((sum, s) => sum + s.price, 0) / sales.length;
console.log('Average Sale Price:', avgPrice.toFixed(2), 'DALLA');
```

---

### Popular Domains

**Most Traded:**
```javascript
// Query domains by transfer count
const domains = await sdk.api.query.bns.domainRegistry.entries();

const popular = domains
    .map(([key, record]) => ({
        domain: key.args[0].toHuman(),
        transferCount: record.unwrap().transferCount.toNumber()
    }))
    .sort((a, b) => b.transferCount - a.transferCount)
    .slice(0, 10);

console.log('Top 10 Most Traded:', popular);
```

---

## Best Practices

### For Sellers

**✅ Do:**
- Research comparable domain prices
- Set realistic asking price (check recent sales)
- Write clear listing description
- Maintain domain resolution (keeps value)
- Respond to offers promptly (Phase 2)

**❌ Don't:**
- Price gouge (deters buyers)
- List trademarked names (risk blacklist)
- Change ownership during active listing
- Forget to delist before making changes

---

### For Buyers

**✅ Do:**
- Verify domain ownership before purchase
- Check domain history (transfer count)
- Review resolution records (any issues?)
- Calculate total cost (price + fees + gas)
- Use official marketplace only

**❌ Don't:**
- Rush into purchases (research first)
- Buy domains with legal issues
- Use third-party escrow (built-in is safer)
- Forget about 7-day transfer lock

---

## Security Considerations

### Escrow Safety

**Built-In Protections:**
- Atomic transactions (payment + transfer or full revert)
- No manual escrow needed
- No counterparty risk
- Instant settlement

**Scam Prevention:**
- Verify seller owns domain
- Check listing on-chain (not just website)
- Use official marketplace URLs only
- Never send DALLA directly to seller

---

### Fraud Protection

**Red Flags:**
- Domain listed well below market value
- Seller pressuring for quick sale
- External payment requests
- Unverified marketplace websites

**Safe Practices:**
- Only use https://bns.belizechain.org
- Verify transactions on-chain
- Check domain ownership history
- Report suspicious listings

---

## Marketplace Governance

### Treasury Fee Allocation

5% marketplace fees fund:
- Validator rewards (40%)
- Development (30%)
- Marketing/adoption (20%)
- Emergency reserve (10%)

**Monthly Revenue (Example):**
```
50 domains sold @ avg 500 DALLA = 25,000 DALLA
Treasury fee (5%): 1,250 DALLA

Allocation:
- Validators: 500 DALLA
- Development: 375 DALLA
- Marketing: 250 DALLA
- Reserve: 125 DALLA
```

---

### Community Proposals

Governance can adjust:
- Marketplace fee (currently 5%, max 10%)
- Listing duration limits
- Minimum/maximum prices
- Reserved domain auctions

**Propose Changes:**
1. Submit governance proposal
2. Community discussion period
3. Token holder vote
4. Implementation if approved

---

## API Integration

### Marketplace API (Developers)

```javascript
// Get all listings (paginated)
async function getAllListings(pageSize = 100) {
    const listings = await sdk.api.query.bns.domainListings.entries();
    
    return listings.map(([key, listing]) => {
        const domain = key.args[0].toString();
        const data = listing.unwrap();
        
        return {
            domain,
            seller: data.seller.toString(),
            price: data.price.toString(),
            listedAt: data.listedAt.toNumber(),
            expiresAt: data.expiresAt.toNumber(),
            minOffer: data.minOffer.isSome ? data.minOffer.unwrap().toString() : null
        };
    });
}

// Filter by price range
function filterByPrice(listings, minPrice, maxPrice) {
    return listings.filter(l => {
        const price = parseInt(l.price) / 1e12;
        return price >= minPrice && price <= maxPrice;
    });
}

// Filter by expiry (show only active)
async function filterActive(listings) {
    const currentBlock = await sdk.api.query.system.number();
    const current = currentBlock.toNumber();
    
    return listings.filter(l => l.expiresAt > current);
}
```

---

## Related Documentation

- [BNS Overview](./bns-overview.md)
- [Domain Registration](./bns-domain-registry.md)
- [Web Hosting](./bns-ipfs-hosting.md)
- [Pricing Details](./bns-pricing.md)
- [API Reference](./bns-api-reference.md)
- [Integration Guide](./bns-integration-guide.md)

---

## Support

- **Marketplace**: https://bns.belizechain.org/marketplace
- **Discord**: #marketplace-support
- **Report Fraud**: security@belizechain.org
- **FAQs**: https://docs.belizechain.org/bns/marketplace-faq
