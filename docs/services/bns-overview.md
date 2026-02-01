# BNS (Belize Name Service) Overview

**Decentralized Domain System for BelizeChain**

BNS (Belize Name Service) is BelizeChain's native domain name system providing .bz domains with permanent ownership, decentralized web hosting, and marketplace functionality.

---

## What is BNS?

BNS enables users to:
- **Register .bz domains** with permanent, transferable ownership
- **Resolve domains** to wallet addresses, content, and metadata
- **Host websites** using decentralized DAG storage (via Pakit)
- **Trade domains** in a built-in marketplace
- **Link external domains** (.com, .org, .net) to BelizeChain

Unlike traditional DNS where you "rent" domains annually, **BNS domains are permanently owned** once registered (similar to ENS on Ethereum).

---

## Architecture

### System Components

```
┌─────────────────────────────────────────┐
│  BNS Pallet (On-Chain Registry)        │
│  • Immutable domain ownership           │
│  • Resolution records                   │
│  • Marketplace logic                    │
│  • KYC-gated registration              │
└─────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────┐
│  Pakit DAG Storage (Content Hosting)    │
│  • MerkleDAG file storage               │
│  • Content-addressed data               │
│  • Multi-algorithm compression          │
│  • Deduplication                        │
└─────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────┐
│  Identity Pallet (KYC Integration)      │
│  • Basic KYC → Standard domains         │
│  • Level 3 KYC → Verified domains       │
│  • Sanctioned account prevention        │
└─────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────┐
│  Treasury (Fee Collection)              │
│  • 5% marketplace fees                  │
│  • Hosting subscription fees            │
│  • Domain registration revenue          │
└─────────────────────────────────────────┘
```

### Pallet Integration

BNS integrates with 4 core pallets:

1. **Identity Pallet**: KYC verification for registration
2. **Economy Pallet**: DALLA payments for domains/hosting
3. **Treasury**: Fee collection (5% marketplace + hosting fees)
4. **Pakit Storage**: DAG-based website hosting

---

## Core Features

### 1. Immutable Domain Registry

Once registered, domains are **permanently owned** and can only be transferred by the owner. No renewal fees or expiration (unlike traditional DNS).

**Key Characteristics:**
- Permanent ownership (one-time purchase)
- Transferable to other accounts
- Transfer lock period (prevents rapid trading)
- Ownership history tracking

**Example:**
```
alice.bz
  Owner: 5GrwvaEF... (Alice)
  Registered: Block #123,456
  Original Owner: 5GrwvaEF... (Alice)
  Purchase Price: 100 DALLA
  Tier: Standard
  Transfer Count: 0
```

---

### 2. Domain Resolution

Domains resolve to multiple records:

| Record Type | Purpose | Example |
|-------------|---------|---------|
| **Wallet Address** | Payment routing | `alice.bz → 5GrwvaEF...` |
| **Content Hash** | Website hosting | `alice.bz → DAG: 0x1a2b...` |
| **Avatar** | Profile image | `alice.bz → IPFS: QmAvatar...` |
| **Metadata** | JSON data | `{"bio": "Developer"}` |
| **Text Records** | Custom fields | `email: alice@example.com` |

**Resolution Query:**
```rust
// Get wallet address for payment
let wallet = BNS::resolve_wallet("alice.bz");

// Get website content hash
let content = BNS::resolve_content("alice.bz");

// Get custom text record
let email = BNS::resolve_text("alice.bz", "email");
```

---

### 3. Decentralized Web Hosting

Host static websites using Pakit DAG storage (100% sovereign, no IPFS/Arweave).

**Hosting Tiers:**

| Tier | Storage | Bandwidth | Price | Features |
|------|---------|-----------|-------|----------|
| **Free** | 100 MB | Basic | 0 DALLA/mo | DAG storage, basic CDN |
| **Basic** | 1 GB | Standard | 10 DALLA/mo | DAG + CDN caching |
| **Pro** | 10 GB | Enhanced | 50 DALLA/mo | Full CDN, compression |
| **Enterprise** | 100 GB | Premium | 200 DALLA/mo | Priority CDN, analytics |

**Monthly Billing:**
- Automatically deducted every 30 days (432,000 blocks)
- Auto-renewal option available
- Manual top-up supported
- Hosting paused if payment fails (content preserved)

---

### 4. Domain Marketplace

Built-in marketplace for buying/selling domains with automated escrow.

**Features:**
- List domains for fixed price or auction
- 5% marketplace fee (goes to treasury)
- Automatic ownership transfer on purchase
- Listing expiry (auto-delist after X blocks)
- Minimum offer support (for bidding)

**Example Transaction:**
```
alice.bz listed for 1,000 DALLA
  Buyer pays: 1,000 DALLA
  Seller receives: 950 DALLA (95%)
  Treasury receives: 50 DALLA (5% fee)
  Ownership transferred to buyer
```

---

### 5. External Domain Support

Link external domains (.com, .org, .net) to BelizeChain for hosting.

**Verification Process:**
1. Register external domain in BNS
2. Receive verification token
3. Add DNS TXT record to your domain:
   ```
   _belizechain-verification.example.com TXT "bz-verify-abc123..."
   ```
4. BNS verifies DNS record
5. Domain activated for hosting

**Benefits:**
- Use existing domain with BelizeChain hosting
- Decentralized content storage
- Sovereign data control
- Lower hosting costs than centralized providers

---

## Domain Tiers

### Standard Domain (100 DALLA)
- Basic .bz domains (4+ characters)
- Permanent ownership
- Full resolution features
- Marketplace trading enabled
- **KYC Required:** Basic (Level 1)

### Premium Domain (500 DALLA)
- Short domains (3 characters)
- Enhanced features (priority resolution)
- Marketplace priority
- Featured listings
- **KYC Required:** Basic (Level 1)

### Government Domain (50 DALLA)
- Subsidized for public sector (.gov.bz)
- Verified government entities only
- Special branding
- **KYC Required:** Government verification

### Verified Domain (1,000 DALLA)
- High-trust verified accounts
- Blue checkmark equivalent
- Business verification
- **KYC Required:** Level 3 (highest)

---

## Technical Specifications

### Storage Schema

```rust
// Domain Registry (permanent ownership)
DomainRegistry: Map<domain_name, DomainRecord>
  DomainRecord {
    owner: AccountId
    original_owner: AccountId
    registered_at: BlockNumber
    purchase_price: u128
    tier: DomainTier
    locked_until: Option<BlockNumber>
    transfer_count: u32
  }

// Resolution Records
DomainResolution: Map<domain_name, ResolutionRecords>
  ResolutionRecords {
    wallet_address: Option<AccountId>
    content_hash: Option<[u8; 32]>  // DAG block hash
    avatar: Option<[u8; 32]>
    metadata: BoundedVec<u8, 256>
    text_records: Vec<TextRecord>
  }

// Marketplace Listings
DomainListings: Map<domain_name, DomainListing>
  DomainListing {
    seller: AccountId
    price: u128
    listed_at: BlockNumber
    expires_at: BlockNumber
    min_offer: Option<u128>
  }

// Web Hosting
HostedWebsites: Map<domain_name, HostingInfo>
  HostingInfo {
    subscriber: AccountId
    tier: HostingTier
    content_hash: [u8; 32]
    activated_at: BlockNumber
    expires_at: BlockNumber
    data_size: u64
    monthly_fee: u128
    auto_renew: bool
  }
```

### Extrinsics (Functions)

| Extrinsic | Description | Fee |
|-----------|-------------|-----|
| `register_domain(name, tier)` | Register new .bz domain | Tier price |
| `update_resolution(domain, records)` | Update resolution data | ~15K gas |
| `transfer_domain(domain, to)` | Transfer ownership | ~18K gas |
| `list_for_sale(domain, price)` | List on marketplace | ~12K gas |
| `buy_domain(domain)` | Purchase listed domain | Price + 5% |
| `activate_hosting(domain, tier, content)` | Start web hosting | Tier fee |
| `renew_hosting(domain)` | Extend hosting period | Monthly fee |
| `update_content(domain, content_hash)` | Update website | ~20K gas |
| `register_external(domain, tier)` | Register external domain | Tier fee |
| `verify_external(domain)` | Verify DNS ownership | Free |

### Events

```rust
DomainRegistered { domain, owner, price, tier }
ResolutionUpdated { domain, owner }
DomainTransferred { domain, from, to }
DomainListed { domain, seller, price }
DomainSold { domain, seller, buyer, price, fee }
HostingActivated { domain, owner, tier, content_hash }
HostingRenewed { domain, owner, blocks_extended }
ContentUpdated { domain, version, content_hash }
ExternalDomainRegistered { domain, owner, tier }
ExternalDomainVerified { domain, owner }
```

---

## Security Features

### KYC Integration
- **Basic KYC (Level 1)**: Required for all domain registrations
- **Level 3 KYC**: Required for Verified domains
- **Sanctioned Check**: Prevents registration by sanctioned accounts

### Transfer Lock
Newly registered or purchased domains have a 7-day transfer lock to prevent:
- Rapid domain flipping
- Scam/fraud schemes
- Marketplace manipulation

### Marketplace Escrow
All domain sales use automated escrow:
1. Buyer payment locked in escrow
2. Ownership transferred
3. Seller receives payment (minus 5% fee)
4. No manual intervention needed

### Content Versioning
All website updates create new versions:
- Full version history preserved
- Rollback capability
- Audit trail for content changes
- Prevents accidental overwrites

---

## Use Cases

### 1. Personal Branding
```
alice.bz → Personal wallet & website
  Wallet: 5GrwvaEF... (receive payments)
  Website: Personal portfolio/blog
  Email: alice@alice.bz
  Social: twitter.com/alice
```

### 2. Business Identity
```
belizecoffee.bz → Coffee shop
  Wallet: Business payment address
  Website: Online store + menu
  Location: GPS coordinates
  Contact: info@belizecoffee.bz
```

### 3. Government Services
```
permits.gov.bz → Building permits
  Verified: ✅ Government entity
  Website: Permit application portal
  Contact: permits@gov.bz
  Status: Active
```

### 4. Land Registry
```
property-123456.bz → Land parcel
  Owner: Property owner wallet
  Content: Land title documents (Pakit)
  Metadata: Legal description, survey
  History: Transfer history on-chain
```

---

## Economic Model

### Revenue Streams

**1. Domain Registration**
- Standard: 100 DALLA/domain
- Premium: 500 DALLA/domain
- Government: 50 DALLA/domain
- Verified: 1,000 DALLA/domain

**2. Marketplace Fees**
- 5% of every domain sale
- Deposited to BelizeChain treasury
- Funds governance and development

**3. Hosting Subscriptions**
- Free: 0 DALLA/month (limited)
- Basic: 10 DALLA/month
- Pro: 50 DALLA/month
- Enterprise: 200 DALLA/month

**Total Revenue (Example Month):**
- 100 domains registered: 10,000 DALLA
- 50 domains sold (avg 500 DALLA): 1,250 DALLA (5% of 25K)
- 200 hosting subscriptions (avg 25 DALLA): 5,000 DALLA
- **Total: 16,250 DALLA/month**

---

## Governance

### Treasury Allocation
All BNS revenue goes to the BelizeChain Treasury for:
- Development funding
- Validator rewards
- Community grants
- Infrastructure costs

### Emergency Controls
Governance can:
- Pause new registrations (emergency only)
- Update pricing tiers (with proposal)
- Blacklist domains (for illegal content)
- Adjust marketplace fees (max 10%)

### Decentralized Decisions
Community governance controls:
- Premium domain pricing
- Hosting tier features
- Marketplace fee percentage
- External domain policies

---

## Comparison with Other Systems

| Feature | BNS (.bz) | ENS (.eth) | Traditional DNS |
|---------|-----------|------------|-----------------|
| **Ownership** | Permanent | Permanent | Annual rental |
| **Renewal Fees** | None | Gas fees only | Yes (yearly) |
| **Blockchain** | BelizeChain | Ethereum | Centralized |
| **Web Hosting** | ✅ Built-in | ❌ No | Third-party |
| **Marketplace** | ✅ Built-in | Third-party | Domain brokers |
| **KYC Required** | ✅ Yes | ❌ No | Sometimes |
| **Content Storage** | DAG (Pakit) | IPFS/Arweave | AWS/Cloudflare |
| **Data Sovereignty** | 100% sovereign | Decentralized | Centralized |

---

## Integration Examples

### Wallet Payment Routing
```rust
// Resolve domain to wallet address
let wallet = BNS::resolve_wallet("alice.bz")?;

// Send payment to domain instead of address
Economy::transfer(origin, "alice.bz", 100 * DALLA);
```

### Website Resolution
```javascript
// Resolve domain to content hash
const content = await bns.resolveContent("alice.bz");

// Fetch website from DAG storage
const html = await pakit.getDagBlock(content.hash);
```

### Identity Verification
```rust
// Check if domain is verified
let verified = BNS::is_verified("business.bz");

// Get domain tier
let tier = BNS::get_domain_tier("business.bz");
```

---

## Performance Metrics

### On-Chain Efficiency
- **Register Domain**: ~50,000 gas (~0.00005 DALLA)
- **Update Resolution**: ~15,000 gas (~0.000015 DALLA)
- **Transfer Domain**: ~18,000 gas (~0.000018 DALLA)
- **List for Sale**: ~12,000 gas (~0.000012 DALLA)

### Off-Chain Hosting
- **DAG Storage**: O(1) content-addressed retrieval
- **CDN Cache**: <100ms response time (Pro/Enterprise)
- **Bandwidth**: Unlimited (no throttling)
- **Uptime**: 99.9% SLA (backed by DAG redundancy)

---

## Roadmap

### Phase 1 (Current - January 2026)
- ✅ Core domain registry
- ✅ Basic web hosting
- ✅ Marketplace functionality
- ✅ External domain support

### Phase 2 (Q2 2026)
- 🔄 Subdomain delegation
- 🔄 ENS compatibility bridge
- 🔄 Mobile domain management app
- 🔄 Advanced analytics dashboard

### Phase 3 (Q3-Q4 2026)
- ⏳ DNS bridge (resolve .bz from traditional DNS)
- ⏳ Advanced marketplace (auctions, bids)
- ⏳ Domain leasing (temporary transfers)
- ⏳ Multi-language domain support (IDN)

---

## Related Documentation

- [Domain Registration Guide](./bns-domain-registry.md)
- [Marketplace Usage](./bns-marketplace.md)
- [Web Hosting Setup](./bns-ipfs-hosting.md)
- [API Reference](./bns-api-reference.md)
- [Integration Guide](./bns-integration-guide.md)
- [Pricing Details](./bns-pricing.md)

---

## Support

- **Documentation**: https://docs.belizechain.org/bns
- **Discord**: #bns channel
- **Domain Search**: https://bns.belizechain.org
- **Marketplace**: https://bns.belizechain.org/marketplace
