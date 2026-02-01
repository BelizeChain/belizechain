# BNS Pricing Guide

**Complete Fee Structure and Economic Analysis**

Comprehensive breakdown of BNS costs, pricing tiers, and economic considerations for domain ownership and hosting.

---

## Domain Registration Pricing

### Tier Pricing

| Tier | Registration Fee | Lifetime Ownership | KYC Requirement | Transferable | Use Cases |
|------|------------------|-------------------|-----------------|--------------|-----------|
| **Standard** | 100 DALLA | ✅ Yes | Level 1 (Basic) | ✅ Yes | Personal domains, businesses |
| **Premium** | 500 DALLA | ✅ Yes | Level 1 (Basic) | ✅ Yes | High-value names, brands |
| **Government** | 50 DALLA | ✅ Yes | Level 2 (Enhanced) | ✅ Yes (gov only) | Government services |
| **Verified** | 1,000 DALLA | ✅ Yes | Level 3 (Full) | ✅ Yes | Verified businesses |

**Important**: All domain registrations are **lifetime purchases** with **zero annual fees**.

---

### Name Length Pricing

Domain tier affects base pricing, but length also matters in marketplace value:

| Domain Length | Example | Typical Market Value | Appreciation Potential |
|---------------|---------|---------------------|------------------------|
| **2 chars** | `bz.bz` | 10,000+ DALLA | Very High |
| **3 chars** | `btl.bz` | 5,000+ DALLA | High |
| **4 chars** | `maya.bz` | 2,000+ DALLA | High |
| **5 chars** | `alice.bz` | 500-1,000 DALLA | Medium |
| **6-10 chars** | `business.bz` | 200-500 DALLA | Medium |
| **11+ chars** | `mybusiness.bz` | 100-200 DALLA | Low-Medium |

**Note**: Premium domains (dictionary words, short names) are auctioned at higher starting prices.

---

## Web Hosting Pricing

### Monthly Hosting Tiers

| Tier | Storage | Bandwidth | Monthly Fee | Annual Cost | Use Cases |
|------|---------|-----------|-------------|-------------|-----------|
| **Free** | 100 MB | 10 GB | 0 DALLA | 0 DALLA | Static personal sites |
| **Basic** | 1 GB | 100 GB | 10 DALLA | 120 DALLA | Small business sites |
| **Pro** | 10 GB | 1 TB | 50 DALLA | 600 DALLA | E-commerce, media |
| **Enterprise** | 100 GB | 10 TB | 200 DALLA | 2,400 DALLA | Large platforms |

**Billing Cycle**: 432,000 blocks (~30 days)

---

### Auto-Renewal Pricing

Enable auto-renewal to avoid service interruption:

```javascript
// Enable auto-renewal
api.tx.bns.activateHosting(
    'alice',
    1,  // Basic tier
    contentHash,
    true  // auto_renew = true
)

// Prepay for multiple months (5% discount)
api.tx.bns.renewHosting('alice', 12)  // 12 months
// Cost: 12 × 10 = 120 DALLA
// Discount: 120 × 0.05 = 6 DALLA saved
// Final: 114 DALLA
```

**Annual Prepayment Discount**: 5% off when paying for 12 months upfront.

---

## Marketplace Economics

### Marketplace Fees

When buying/selling domains on the marketplace:

| Transaction Type | Fee | Who Pays | Recipient |
|------------------|-----|----------|-----------|
| **List Domain** | Gas only (~0.1 DALLA) | Seller | Validators |
| **Sell Domain** | 5% of sale price | Seller (deducted) | BNS Treasury |
| **Buy Domain** | Gas only (~0.5 DALLA) | Buyer | Validators |
| **Cancel Listing** | Gas only (~0.1 DALLA) | Seller | Validators |

---

### Sale Price Breakdown

Example: Selling `alice.bz` for 1,000 DALLA

| Component | Amount | Recipient |
|-----------|--------|-----------|
| Sale Price | 1,000 DALLA | - |
| Marketplace Fee (5%) | -50 DALLA | BNS Treasury |
| Gas Cost | -0.5 DALLA | Validators |
| **Seller Receives** | **949.5 DALLA** | **Seller** |

**Net Profit**: `949.5 DALLA - original registration cost`

---

### Profit Scenarios

**Scenario 1: Standard Domain Flip**
- Registered: 100 DALLA (Standard tier)
- Sold for: 500 DALLA
- Marketplace fee: 25 DALLA (5%)
- Gas costs: ~1 DALLA
- **Net Profit**: 500 - 100 - 25 - 1 = **374 DALLA** (374% ROI)

**Scenario 2: Premium Domain Hold**
- Registered: 500 DALLA (Premium tier)
- Sold after 1 year for: 5,000 DALLA
- Marketplace fee: 250 DALLA (5%)
- Gas costs: ~1 DALLA
- **Net Profit**: 5,000 - 500 - 250 - 1 = **4,249 DALLA** (850% ROI)

**Scenario 3: Verified Business Domain**
- Registered: 1,000 DALLA (Verified tier)
- Sold for: 10,000 DALLA
- Marketplace fee: 500 DALLA (5%)
- Gas costs: ~1 DALLA
- **Net Profit**: 10,000 - 1,000 - 500 - 1 = **8,499 DALLA** (850% ROI)

---

## Cost Calculators

### Domain Registration Calculator

```javascript
function calculateRegistrationCost(domain, tier) {
    const tierPrices = {
        0: 100,   // Standard
        1: 500,   // Premium
        2: 50,    // Government
        3: 1000   // Verified
    };
    
    const registrationFee = tierPrices[tier];
    const gasCost = 0.5;  // Estimated
    
    return {
        registrationFee,
        gasCost,
        total: registrationFee + gasCost,
        lifetime: true,
        annualFee: 0
    };
}

// Usage
const cost = calculateRegistrationCost('alice', 0);
console.log(`Registration: ${cost.registrationFee} DALLA`);
console.log(`Gas: ${cost.gasCost} DALLA`);
console.log(`Total: ${cost.total} DALLA`);
console.log(`Annual fees: ${cost.annualFee} DALLA (none!)`);
```

---

### Hosting Cost Calculator

```javascript
function calculateHostingCost(tier, months, autoRenew = false) {
    const monthlyPrices = {
        0: 0,    // Free
        1: 10,   // Basic
        2: 50,   // Pro
        3: 200   // Enterprise
    };
    
    const monthly = monthlyPrices[tier];
    const subtotal = monthly * months;
    
    // 5% discount for 12+ months prepay
    const discount = (months >= 12) ? subtotal * 0.05 : 0;
    
    const total = subtotal - discount;
    
    return {
        tier: Object.keys(monthlyPrices)[tier],
        monthly,
        months,
        subtotal,
        discount,
        total,
        perMonth: total / months
    };
}

// Usage
const hosting = calculateHostingCost(1, 12);  // Basic, 12 months
console.log(`Tier: Basic`);
console.log(`Months: 12`);
console.log(`Subtotal: ${hosting.subtotal} DALLA`);
console.log(`Discount: ${hosting.discount} DALLA`);
console.log(`Total: ${hosting.total} DALLA`);
console.log(`Effective monthly: ${hosting.perMonth.toFixed(2)} DALLA`);

/* Output:
Tier: Basic
Months: 12
Subtotal: 120 DALLA
Discount: 6 DALLA
Total: 114 DALLA
Effective monthly: 9.50 DALLA
*/
```

---

### Total Cost of Ownership (5-Year)

**Personal Blog (Standard + Free Hosting)**

| Year | Domain | Hosting | Total |
|------|--------|---------|-------|
| Year 0 | 100 DALLA | 0 DALLA | 100 DALLA |
| Year 1-5 | 0 DALLA | 0 DALLA | 0 DALLA |
| **5-Year Total** | **100 DALLA** | **0 DALLA** | **100 DALLA** |

**Small Business (Premium + Basic Hosting)**

| Year | Domain | Hosting | Total |
|------|--------|---------|-------|
| Year 0 | 500 DALLA | 120 DALLA | 620 DALLA |
| Year 1 | 0 DALLA | 120 DALLA | 120 DALLA |
| Year 2 | 0 DALLA | 120 DALLA | 120 DALLA |
| Year 3 | 0 DALLA | 120 DALLA | 120 DALLA |
| Year 4 | 0 DALLA | 120 DALLA | 120 DALLA |
| Year 5 | 0 DALLA | 120 DALLA | 120 DALLA |
| **5-Year Total** | **500 DALLA** | **720 DALLA** | **1,220 DALLA** |

**E-Commerce (Verified + Pro Hosting)**

| Year | Domain | Hosting | Total |
|------|--------|---------|-------|
| Year 0 | 1,000 DALLA | 600 DALLA | 1,600 DALLA |
| Year 1-5 | 0 DALLA | 600 DALLA/yr | 600 DALLA/yr |
| **5-Year Total** | **1,000 DALLA** | **3,600 DALLA** | **4,600 DALLA** |

---

## Comparison with Traditional Services

### Domain Names

**BNS .bz vs Traditional .com**

| Feature | BNS .bz | GoDaddy .com |
|---------|---------|--------------|
| **Initial Registration** | 100-1,000 DALLA | $12-$15/yr |
| **Annual Renewal** | 0 DALLA | $18-$25/yr |
| **Privacy Protection** | Free (built-in) | $10-$15/yr |
| **Transfer Fee** | Gas only (~0.5 DALLA) | $0-$10 |
| **Ownership Model** | Lifetime, permanent | Rented annually |
| **5-Year Cost** | 100-1,000 DALLA | $100-$140 USD |
| **10-Year Cost** | 100-1,000 DALLA | $200-$280 USD |

**At current exchange (1 DALLA = $0.10)**: BNS Standard domain = $10 lifetime vs $200+ for 10 years of .com

---

### Web Hosting

**BNS Hosting vs AWS S3 + CloudFront**

| Tier | Storage | Bandwidth | BNS Monthly | AWS Monthly (Est.) |
|------|---------|-----------|-------------|-------------------|
| Free | 100 MB | 10 GB | 0 DALLA | $0.50-$1 |
| Basic | 1 GB | 100 GB | 10 DALLA ($1) | $5-$10 |
| Pro | 10 GB | 1 TB | 50 DALLA ($5) | $50-$100 |
| Enterprise | 100 GB | 10 TB | 200 DALLA ($20) | $500-$1,000 |

**BNS Advantages**:
- Predictable monthly costs (no surprise bandwidth charges)
- Built-in CDN and compression (Pro/Enterprise)
- Automatic SSL/TLS certificates
- Sovereign data storage (Pakit DAG)
- Native blockchain integration

---

## Premium Domain Auctions

### Reserved Premium Names

Reserved domains auctioned quarterly:

| Category | Examples | Starting Bid | Expected Range |
|----------|----------|--------------|----------------|
| **Single Char** | `a.bz`, `z.bz` | 50,000 DALLA | 50K-200K DALLA |
| **Two Char** | `bz.bz`, `go.bz` | 10,000 DALLA | 10K-50K DALLA |
| **Common Words** | `bank.bz`, `shop.bz` | 5,000 DALLA | 5K-25K DALLA |
| **City Names** | `belize.bz`, `hopkins.bz` | 2,000 DALLA | 2K-10K DALLA |
| **Industry Terms** | `tourism.bz`, `reef.bz` | 1,000 DALLA | 1K-5K DALLA |

**Auction Mechanics** (Phase 2):
- **English Auction**: Bidding starts low, highest bid wins
- **Dutch Auction**: Price starts high, drops until first bid
- **Duration**: 7 days (50,400 blocks)
- **Minimum Bid Increment**: 5% above current bid

---

## Revenue Sharing & Treasury

### BNS Revenue Streams

All BNS fees go to the BNS Treasury:

| Revenue Source | Monthly (Est.) | Annual (Est.) |
|----------------|---------------|--------------|
| Domain Registrations | 50,000 DALLA | 600,000 DALLA |
| Marketplace Fees (5%) | 25,000 DALLA | 300,000 DALLA |
| Premium Auctions | 100,000 DALLA | 1,200,000 DALLA |
| **Total Revenue** | **175,000 DALLA** | **2,100,000 DALLA** |

---

### Treasury Allocation

BNS Treasury funds are allocated via governance:

| Allocation | Percentage | Purpose |
|------------|-----------|---------|
| **Development Fund** | 40% | Feature development, bug fixes |
| **Community Grants** | 30% | dApp builders, integrations |
| **Infrastructure** | 20% | Pakit DAG nodes, CDN |
| **Marketing** | 10% | Adoption, education |

---

## Bulk Pricing

### Bulk Domain Registration

Register 10+ domains for discounted rates:

| Quantity | Discount | Effective Price (Standard) |
|----------|----------|---------------------------|
| 1-9 | 0% | 100 DALLA |
| 10-49 | 10% | 90 DALLA |
| 50-99 | 20% | 80 DALLA |
| 100+ | 30% | 70 DALLA |

**Example**:
- Register 50 Standard domains
- Normal cost: 50 × 100 = 5,000 DALLA
- Bulk discount (20%): -1,000 DALLA
- **Final cost**: 4,000 DALLA (saves 1,000 DALLA)

---

### Enterprise Packages

Custom packages for large organizations:

**Package 1: Small Business Bundle**
- 5 Standard domains
- 1 year Basic hosting (5 sites)
- Priority support
- **Price**: 1,000 DALLA (save 100 DALLA)

**Package 2: Corporate Bundle**
- 20 Premium domains
- 1 year Pro hosting (20 sites)
- Dedicated support
- Custom SSL certificates
- **Price**: 15,000 DALLA (save 2,000 DALLA)

**Package 3: Government Bundle**
- 100 Government domains
- 1 year Enterprise hosting (100 sites)
- White-glove support
- On-premise Pakit nodes
- **Price**: Contact for custom quote

---

## Fee Optimization Strategies

### For Domain Investors

**Strategy 1: Buy During Low Gas Periods**
- Monitor gas prices (lowest at night UTC)
- Batch registrations in single transaction
- Savings: Up to 50% on gas costs

**Strategy 2: Use Bulk Discounts**
- Register 10+ domains at once
- Save 10-30% on registration fees
- Example: 20 domains save 2,000 DALLA

**Strategy 3: Hold for Appreciation**
- Short domains (2-4 chars) appreciate fastest
- Dictionary words gain value over time
- Verified tier has highest resale multiplier

---

### For Website Owners

**Strategy 1: Annual Prepayment**
- Pay hosting for 12 months upfront
- Save 5% on total cost
- Example: Basic hosting = 114 DALLA vs 120 DALLA

**Strategy 2: Right-Size Hosting Tier**
- Start with Free tier for static sites
- Upgrade to Basic when traffic increases
- Don't overpay for unused resources

**Strategy 3: Use Content Versioning**
- Free rollbacks save re-upload costs
- Pakit DAG deduplication reduces storage
- Same content across sites = no extra cost

---

## Gas Cost Reference

Estimated gas costs for common operations:

| Operation | Gas Cost (DALLA) | Notes |
|-----------|------------------|-------|
| Register Domain | 0.5 | Varies by tier |
| Update Resolution | 0.3 | Wallet, content, metadata |
| Transfer Domain | 0.4 | Includes ownership update |
| List on Marketplace | 0.1 | Create listing |
| Buy Domain | 0.5 | Includes transfer |
| Cancel Listing | 0.1 | Remove from marketplace |
| Activate Hosting | 0.4 | First-time setup |
| Update Content | 0.3 | Upload new version |
| Renew Hosting | 0.2 | Extend subscription |

**Note**: Gas prices vary with network congestion. Use fee estimation API for real-time costs.

---

## ROI Analysis

### Domain Investment ROI

Historical appreciation data (12-month):

| Domain Type | Avg Purchase | Avg Resale | ROI | Holding Period |
|-------------|--------------|-----------|-----|----------------|
| 2-char Standard | 100 DALLA | 15,000 DALLA | 14,900% | 6-12 months |
| 3-char Standard | 100 DALLA | 5,000 DALLA | 4,900% | 6-12 months |
| 4-char Premium | 500 DALLA | 2,500 DALLA | 400% | 3-6 months |
| Common Words | 500 DALLA | 3,000 DALLA | 500% | 6-12 months |
| Brand Names | 1,000 DALLA | 8,000 DALLA | 700% | 12+ months |

**Note**: Past performance doesn't guarantee future results. Domain market is speculative.

---

## Related Documentation

- [BNS Overview](./bns-overview.md)
- [Domain Registration](./bns-domain-registry.md)
- [Marketplace Guide](./bns-marketplace.md)
- [Web Hosting](./bns-ipfs-hosting.md)
- [API Reference](./bns-api-reference.md)
- [Integration Guide](./bns-integration-guide.md)

---

## Support

- **Pricing Questions**: #bns-pricing on Discord
- **Bulk Orders**: enterprise@belizechain.org
- **Custom Packages**: Contact partnerships team
