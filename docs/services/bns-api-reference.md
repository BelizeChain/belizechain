# BNS API Reference

**Complete Pallet Extrinsics, Queries, and Events**

Technical reference for the BNS (Belize Name Service) pallet API on BelizeChain.

---

## Extrinsics (Functions)

### register_domain

Register a new .bz domain with permanent ownership.

**Signature:**
```rust
fn register_domain(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    tier: u8
) -> DispatchResult
```

**Parameters:**
- `origin`: Signed transaction from registrant
- `domain_name`: Domain name without .bz extension (e.g., "alice")
- `tier`: Domain tier (0=Standard, 1=Premium, 2=Government, 3=Verified)

**Requirements:**
- KYC Level 1 (basic) for Standard/Premium/Government
- KYC Level 3 (enhanced) for Verified
- Domain not already registered
- Account not sanctioned
- Sufficient DALLA balance (tier price + gas)
- Account owns < 100 domains

**Gas Cost:** ~50,000 units (~0.00005 DALLA)

**Events Emitted:**
- `DomainRegistered { domain, owner, price, tier }`

**Errors:**
- `DomainAlreadyExists`: Domain is already registered
- `DomainTooShort`: Name shorter than minimum (4 chars for Standard, 3 for Premium/Verified)
- `DomainTooLong`: Name exceeds 63 characters
- `InvalidDomainCharacters`: Contains invalid characters
- `KycRequired`: Account lacks basic KYC
- `VerifiedKycRequired`: Verified domain requires Level 3 KYC
- `AccountSanctioned`: Account is on sanctions list
- `MaxDomainsReached`: Account owns 100+ domains
- `InsufficientBalance`: Not enough DALLA for tier price

**Example:**
```javascript
// Register Standard domain (100 DALLA)
await api.tx.bns.registerDomain('alice', 0).signAndSend(alice);

// Register Premium domain (500 DALLA)
await api.tx.bns.registerDomain('bob', 1).signAndSend(bob);

// Register Verified domain (1,000 DALLA, requires Level 3 KYC)
await api.tx.bns.registerDomain('business', 3).signAndSend(business);
```

---

### set_resolution

Update domain resolution records (wallet, content, metadata).

**Signature:**
```rust
fn set_resolution(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    wallet_address: Option<AccountId>,
    content_hash: Option<[u8; 32]>,
    metadata: Vec<u8>
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to update
- `wallet_address`: Wallet for payment routing (optional)
- `content_hash`: DAG content hash for website (optional)
- `metadata`: JSON metadata (max 256 bytes)

**Requirements:**
- Caller must own domain

**Gas Cost:** ~15,000 units (~0.000015 DALLA)

**Events Emitted:**
- `ResolutionUpdated { domain, owner }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `InvalidMetadata`: Metadata exceeds 256 bytes

**Example:**
```javascript
// Set wallet address
await api.tx.bns.setResolution(
    'alice',
    alice.address,  // Wallet
    null,           // No content
    '{}'            // Empty metadata
).signAndSend(alice);

// Set website content
await api.tx.bns.setResolution(
    'alice',
    alice.address,
    '0x1a2b3c4d...',  // DAG hash
    '{"bio": "Developer"}'
).signAndSend(alice);
```

---

### transfer_domain

Transfer domain ownership to another account.

**Signature:**
```rust
fn transfer_domain(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    new_owner: AccountId
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to transfer
- `new_owner`: Recipient account

**Requirements:**
- Caller must own domain
- Domain transfer lock must be expired

**Gas Cost:** ~18,000 units (~0.000018 DALLA)

**Events Emitted:**
- `DomainTransferred { domain, from, to }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `DomainLocked`: Transfer lock still active (7 days after registration/purchase)
- `MaxDomainsReached`: Recipient already owns 100 domains

**Example:**
```javascript
// Transfer to Bob
await api.tx.bns.transferDomain('alice', bob.address).signAndSend(alice);
```

---

### list_domain

List domain for sale on marketplace.

**Signature:**
```rust
fn list_domain(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    price: u128,
    min_offer: Option<u128>,
    duration_blocks: BlockNumber
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to list
- `price`: Asking price in planck (1 DALLA = 10^12 planck)
- `min_offer`: Minimum acceptable offer (optional, for Phase 2 bidding)
- `duration_blocks`: Listing duration in blocks (max 1,296,000 = 90 days)

**Requirements:**
- Caller must own domain
- Domain not already listed
- Price >= 10 DALLA (prevents spam)
- Duration 7-90 days

**Gas Cost:** ~12,000 units (~0.000012 DALLA)

**Events Emitted:**
- `DomainListed { domain, seller, price }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `DomainAlreadyListed`: Domain already on marketplace
- `InvalidPrice`: Price below minimum (10 DALLA)

**Example:**
```javascript
// List for 1,000 DALLA for 30 days
const price = 1000 * 1e12;
const duration = 30 * 24 * 600;  // 30 days in blocks

await api.tx.bns.listDomain(
    'alice',
    price,
    null,      // No minimum offer
    duration
).signAndSend(alice);
```

---

### buy_domain

Purchase domain from marketplace.

**Signature:**
```rust
fn buy_domain(
    origin: OriginFor<T>,
    domain_name: Vec<u8>
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to purchase

**Requirements:**
- Domain must be listed
- Listing not expired
- Buyer has sufficient balance (price + gas)

**Gas Cost:** ~25,000 units (~0.000025 DALLA)

**Payment Flow:**
- Seller receives: price * 0.95 (95%)
- Treasury receives: price * 0.05 (5% marketplace fee)

**Events Emitted:**
- `DomainSold { domain, seller, buyer, price, marketplace_fee }`
- `DomainTransferred { domain, from: seller, to: buyer }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `DomainNotListed`: Domain not on marketplace
- `ListingExpired`: Listing duration passed
- `InsufficientBalance`: Buyer lacks funds

**Example:**
```javascript
// Buy alice.bz from marketplace
await api.tx.bns.buyDomain('alice').signAndSend(bob);

// Payment: 1,000 DALLA
// Seller gets: 950 DALLA
// Treasury gets: 50 DALLA
// Buyer receives domain ownership
```

---

### delist_domain

Remove domain from marketplace.

**Signature:**
```rust
fn delist_domain(
    origin: OriginFor<T>,
    domain_name: Vec<u8>
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to delist

**Requirements:**
- Caller must be seller
- Domain must be listed

**Gas Cost:** ~10,000 units (~0.00001 DALLA)

**Events Emitted:**
- `DomainDelisted { domain, seller }`

**Errors:**
- `DomainNotListed`: Domain not on marketplace
- `NotSeller`: Caller didn't list the domain

**Example:**
```javascript
// Remove from marketplace
await api.tx.bns.delistDomain('alice').signAndSend(alice);
```

---

### activate_hosting

Activate website hosting for domain.

**Signature:**
```rust
fn activate_hosting(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    tier: u8,
    content_hash: [u8; 32],
    auto_renew: bool
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to host
- `tier`: Hosting tier (0=Free, 1=Basic, 2=Pro, 3=Enterprise)
- `content_hash`: DAG root hash from Pakit upload
- `auto_renew`: Enable automatic monthly renewals

**Requirements:**
- Caller must own domain
- Sufficient balance for monthly fee
- Content hash must be valid DAG block

**Gas Cost:** ~20,000 units (~0.00002 DALLA)

**Monthly Fees:**
- Free: 0 DALLA
- Basic: 10 DALLA
- Pro: 50 DALLA
- Enterprise: 200 DALLA

**Events Emitted:**
- `HostingActivated { domain, owner, tier, content_hash }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `InsufficientBalance`: Can't pay monthly fee
- `InvalidContentHash`: DAG block not found

**Example:**
```javascript
// Activate Basic hosting (10 DALLA/month)
const contentHash = '0x1a2b3c4d5e6f...';  // From pakit upload
await api.tx.bns.activateHosting(
    'alice',
    1,            // Basic tier
    contentHash,
    true          // Auto-renew
).signAndSend(alice);
```

---

### update_content

Update hosted website content.

**Signature:**
```rust
fn update_content(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    content_hash: [u8; 32]
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to update
- `content_hash`: New DAG root hash

**Requirements:**
- Caller must own domain
- Hosting must be active

**Gas Cost:** ~20,000 units (~0.00002 DALLA)

**Events Emitted:**
- `ContentUpdated { domain, version, content_hash }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `HostingNotActive`: No active hosting subscription

**Example:**
```javascript
// Update website (creates version 2)
const newContentHash = '0xfedcba...';
await api.tx.bns.updateContent('alice', newContentHash).signAndSend(alice);
```

---

### rollback_content

Rollback website to previous version.

**Signature:**
```rust
fn rollback_content(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    version: u32
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to rollback
- `version`: Target version number

**Requirements:**
- Caller must own domain
- Version must exist in history
- Hosting must be active

**Gas Cost:** ~18,000 units (~0.000018 DALLA)

**Events Emitted:**
- `ContentRolledBack { domain, version, content_hash }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `VersionNotFound`: Version doesn't exist

**Example:**
```javascript
// Rollback to version 1
await api.tx.bns.rollbackContent('alice', 1).signAndSend(alice);
```

---

### renew_hosting

Manually renew hosting subscription.

**Signature:**
```rust
fn renew_hosting(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    months: u32
) -> DispatchResult
```

**Parameters:**
- `domain_name`: Domain to renew
- `months`: Number of months to extend (1-12)

**Requirements:**
- Caller must own domain
- Hosting must be active
- Sufficient balance for payment (monthly_fee * months)

**Gas Cost:** ~15,000 units (~0.000015 DALLA)

**Events Emitted:**
- `HostingRenewed { domain, owner, blocks_extended }`
- `HostingFeeCollected { payer, amount }`

**Errors:**
- `DomainNotFound`: Domain does not exist
- `NotDomainOwner`: Caller doesn't own domain
- `HostingNotActive`: No active subscription
- `InsufficientBalance`: Can't pay renewal fee

**Example:**
```javascript
// Renew for 3 months
await api.tx.bns.renewHosting('alice', 3).signAndSend(alice);

// Deducts: monthlyFee * 3
// Extends expiry by: 432,000 * 3 blocks
```

---

## Storage Queries

### domainRegistry

Get domain ownership record.

**Query:**
```rust
domainRegistry(domain: BoundedVec<u8>) -> Option<DomainRecord>
```

**Returns:**
```rust
DomainRecord {
    owner: AccountId,
    original_owner: AccountId,
    registered_at: BlockNumber,
    purchase_price: u128,
    tier: DomainTier,
    locked_until: Option<BlockNumber>,
    transfer_count: u32
}
```

**Example:**
```javascript
const domain = await api.query.bns.domainRegistry('alice');
if (domain.isSome) {
    const record = domain.unwrap();
    console.log('Owner:', record.owner.toHuman());
    console.log('Registered:', record.registeredAt.toNumber());
    console.log('Price:', record.purchasePrice.toNumber() / 1e12, 'DALLA');
}
```

---

### domainResolution

Get domain resolution records.

**Query:**
```rust
domainResolution(domain: BoundedVec<u8>) -> Option<ResolutionRecords>
```

**Returns:**
```rust
ResolutionRecords {
    wallet_address: Option<AccountId>,
    content_hash: Option<[u8; 32]>,
    avatar: Option<[u8; 32]>,
    metadata: BoundedVec<u8, 256>,
    text_records: Vec<TextRecord>
}
```

**Example:**
```javascript
const resolution = await api.query.bns.domainResolution('alice');
if (resolution.isSome) {
    const data = resolution.unwrap();
    console.log('Wallet:', data.walletAddress.toHuman());
    console.log('Content Hash:', data.contentHash.toHex());
    console.log('Metadata:', JSON.parse(data.metadata.toString()));
}
```

---

### accountDomains

Get all domains owned by account.

**Query:**
```rust
accountDomains(account: AccountId) -> Vec<BoundedVec<u8>>
```

**Example:**
```javascript
const domains = await api.query.bns.accountDomains(alice.address);
console.log('Alice owns:', domains.toHuman());
// Output: ['alice', 'alice-dev', 'alice-portfolio']
```

---

### domainListings

Get marketplace listing for domain.

**Query:**
```rust
domainListings(domain: BoundedVec<u8>) -> Option<DomainListing>
```

**Returns:**
```rust
DomainListing {
    seller: AccountId,
    price: u128,
    listed_at: BlockNumber,
    expires_at: BlockNumber,
    min_offer: Option<u128>
}
```

**Example:**
```javascript
const listing = await api.query.bns.domainListings('alice');
if (listing.isSome) {
    const data = listing.unwrap();
    console.log('Price:', data.price.toNumber() / 1e12, 'DALLA');
    console.log('Seller:', data.seller.toHuman());
}
```

---

### hostedWebsites

Get hosting subscription details.

**Query:**
```rust
hostedWebsites(domain: BoundedVec<u8>) -> Option<HostingInfo>
```

**Returns:**
```rust
HostingInfo {
    subscriber: AccountId,
    tier: HostingTier,
    content_hash: [u8; 32],
    activated_at: BlockNumber,
    last_payment_at: BlockNumber,
    expires_at: BlockNumber,
    data_size: u64,
    monthly_fee: u128,
    auto_renew: bool
}
```

**Example:**
```javascript
const hosting = await api.query.bns.hostedWebsites('alice');
if (hosting.isSome) {
    const data = hosting.unwrap();
    console.log('Tier:', data.tier.toString());
    console.log('Expires:', data.expiresAt.toNumber());
    console.log('Monthly Fee:', data.monthlyFee.toNumber() / 1e12, 'DALLA');
}
```

---

### contentHistory

Get content version history.

**Query:**
```rust
contentHistory(domain: BoundedVec<u8>, version: u32) -> Option<ContentVersion>
```

**Returns:**
```rust
ContentVersion {
    content_hash: [u8; 32],
    uploaded_at: BlockNumber,
    description: BoundedVec<u8, 128>,
    size_bytes: u64
}
```

**Example:**
```javascript
// Get version 1
const version = await api.query.bns.contentHistory('alice', 1);
if (version.isSome) {
    const data = version.unwrap();
    console.log('Hash:', data.contentHash.toHex());
    console.log('Size:', data.sizeBytes.toNumber(), 'bytes');
}
```

---

### totalDomains

Get total registered domains count.

**Query:**
```rust
totalDomains() -> u64
```

**Example:**
```javascript
const total = await api.query.bns.totalDomains();
console.log('Total .bz domains:', total.toNumber());
```

---

### totalMarketplaceRevenue

Get cumulative marketplace fees collected.

**Query:**
```rust
totalMarketplaceRevenue() -> u128
```

**Example:**
```javascript
const revenue = await api.query.bns.totalMarketplaceRevenue();
console.log('Marketplace Revenue:', (revenue.toNumber() / 1e12).toFixed(2), 'DALLA');
```

---

## Events

### DomainRegistered
```rust
DomainRegistered {
    domain: BoundedVec<u8>,
    owner: AccountId,
    price: u128,
    tier: u8
}
```

### ResolutionUpdated
```rust
ResolutionUpdated {
    domain: BoundedVec<u8>,
    owner: AccountId
}
```

### DomainTransferred
```rust
DomainTransferred {
    domain: BoundedVec<u8>,
    from: AccountId,
    to: AccountId
}
```

### DomainListed
```rust
DomainListed {
    domain: BoundedVec<u8>,
    seller: AccountId,
    price: u128
}
```

### DomainSold
```rust
DomainSold {
    domain: BoundedVec<u8>,
    seller: AccountId,
    buyer: AccountId,
    price: u128,
    marketplace_fee: u128
}
```

### HostingActivated
```rust
HostingActivated {
    domain: BoundedVec<u8>,
    owner: AccountId,
    tier: u8,
    content_hash: [u8; 32]
}
```

### ContentUpdated
```rust
ContentUpdated {
    domain: BoundedVec<u8>,
    version: u32,
    content_hash: [u8; 32]
}
```

---

## Related Documentation

- [BNS Overview](./bns-overview.md)
- [Domain Registration](./bns-domain-registry.md)
- [Marketplace](./bns-marketplace.md)
- [Web Hosting](./bns-ipfs-hosting.md)
- [Integration Guide](./bns-integration-guide.md)
- [Pricing Details](./bns-pricing.md)
