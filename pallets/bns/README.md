# BNS (Belize Name Service) Pallet

## Overview

BNS (Belize Name Service) is BelizeChain's decentralized domain name system, enabling users to register permanent `.bz` domains for wallet resolution, decentralized web hosting, and digital identity. Inspired by Ethereum Name Service (ENS), BNS provides **immutable ownership**, **marketplace trading**, and **sovereign DAG storage** for hosted websites.

**Core Innovation**: Domains are permanent NFTs (never expire), hosted websites use BelizeChain's sovereign Pakit DAG storage (zero IPFS/Arweave dependencies), and all operations require KYC compliance.

### Key Features

- **Immutable Domain Registry**: `.bz` domains with permanent ownership (ENS-like)
- **Domain Resolution**: Map domains to wallet addresses, content hashes, avatars, metadata
- **Domain Marketplace**: Buy/sell domains with 5% Treasury fee
- **Decentralized Web Hosting**: Host websites via Pakit DAG storage (renewable subscriptions)
- **External Domain Support**: Register `.com`, `.org`, `.net`, etc. (DNS integration planned)
- **Multi-Tier Pricing**: Standard (5 DALLA), Premium (50 DALLA), Government (free), Verified (100 DALLA)
- **KYC-Gated Registration**: All domains require KYC Level 1+ (prevents domain squatting abuse)
- **Content Versioning**: Track website content history (up to 10 versions)
- **Auto-Renewal**: Optional subscription auto-payment for hosting
- **Emergency Controls**: Governance can revoke domains (sanctions, abuse)

### Business Use Cases

1. **Digital Identity**: `alice.bz` → wallet address, profile, avatar
2. **Business Websites**: `resort.bz` → hosted website (Pakit DAG), payment address
3. **Government Portals**: `health.gov.bz` → official ministry website
4. **Domain Speculation**: Buy premium domains (e.g., `casino.bz`), sell on marketplace for profit
5. **Decentralized Applications**: `dex.bz` → smart contract interface
6. **Personal Branding**: `artist.bz` → portfolio, NFT gallery, donations address

## Architecture

### Domain Lifecycle
```
Registration → KYC Check → Price Calculation → Payment → Ownership NFT 
→ (Optional) Marketplace Listing → (Optional) Web Hosting Activation 
→ (Optional) Content Updates → Transfer/Hold Forever
```

### Components

1. **Domain Registry**: Permanent ownership records (never expire)
2. **Resolution System**: Maps domains to addresses/hashes/metadata
3. **Marketplace**: Peer-to-peer domain trading (5% Treasury fee)
4. **Hosting Service**: Pakit DAG storage with renewable subscriptions
5. **Content Versioning**: Historical content snapshots (up to 10 versions)
6. **DNS Integration** (FUTURE): External domain `.com`/`.org` registration

### Domain Tiers & Pricing

| Tier | Price | Requirements | Use Case |
|------|-------|-------------|----------|
| Standard | 5 DALLA | KYC L1+, 3+ characters | Personal domains |
| Premium | 50 DALLA | KYC L2+, premium keywords | Business domains |
| Government | 0 DALLA | KYC L3+, government identity | Official ministries |
| Verified | 100 DALLA | KYC L3+, biometric | High-trust businesses |

**Premium Keywords**: `casino`, `bank`, `insurance`, `lottery`, `gold`, `luxury`, `hotel`, `resort` (configurable by governance)

### Hosting Tiers

| Tier | Monthly Fee | Storage | Bandwidth | Use Case |
|------|-------------|---------|-----------|----------|
| Free | 0 DALLA | 100 MB | 1 GB | Personal sites |
| Basic | 10 DALLA | 1 GB | 10 GB | Small business |
| Pro | 50 DALLA | 10 GB | 100 GB | E-commerce |
| Enterprise | 250 DALLA | 100 GB | 1 TB | High-traffic portals |

## Storage

### DomainRegistry
- **Type**: `StorageMap<BoundedVec<u8, MaxDomainLength>, DomainRecord>`
- **Key**: Domain name (e.g., `alice.bz`)
- **Purpose**: Immutable ownership records
- **Data**:
  - `owner: AccountId` - Current owner
  - `original_owner: AccountId` - First registrant (historical tracking)
  - `registered_at: BlockNumber` - Registration timestamp
  - `purchase_price: u128` - Original registration cost
  - `tier: DomainTier` - Standard/Premium/Government/Verified
  - `locked_until: Option<BlockNumber>` - Transfer lock (optional)
  - `transfer_count: u32` - Number of ownership transfers

### AccountDomains
- **Type**: `StorageMap<AccountId, BoundedVec<BoundedVec<u8>, MaxDomainsPerAccount>>`
- **Purpose**: Reverse lookup (AccountId → list of owned domains)
- **Max Domains**: 100 per account (configurable)

### DomainResolution
- **Type**: `StorageMap<BoundedVec<u8>, ResolutionRecords>`
- **Purpose**: Domain resolution data (what the domain points to)
- **Data**:
  - `wallet_address: Option<AccountId>` - Payment/transfer address
  - `content_hash: Option<[u8; 32]>` - DAG content root hash (for websites)
  - `avatar: Option<[u8; 32]>` - Profile picture hash
  - `metadata: BoundedVec<u8, 256>` - JSON metadata (name, bio, social links)
  - `text_records: BoundedVec<TextRecord, MaxTextRecords>` - Custom key-value pairs

### DomainListings
- **Type**: `StorageMap<BoundedVec<u8>, DomainListing>`
- **Purpose**: Active marketplace listings
- **Data**:
  - `seller: AccountId` - Current owner selling domain
  - `price: u128` - Sale price in DALLA
  - `listed_at: BlockNumber` - Listing timestamp
  - `expires_at: BlockNumber` - Listing expiry
  - `min_offer: Option<u128>` - Minimum acceptable offer (optional)

### HostedWebsites
- **Type**: `StorageMap<BoundedVec<u8>, HostingInfo>`
- **Purpose**: Active website hosting subscriptions
- **Data**:
  - `subscriber: AccountId` - Domain owner paying for hosting
  - `tier: HostingTier` - Free/Basic/Pro/Enterprise
  - `content_hash: [u8; 32]` - Current DAG content root
  - `activated_at: BlockNumber` - Hosting start
  - `last_payment_at: BlockNumber` - Last renewal payment
  - `expires_at: BlockNumber` - Subscription expiry
  - `data_size: u64` - Current website size in bytes
  - `monthly_fee: u128` - Recurring fee per month
  - `auto_renew: bool` - Auto-renewal enabled

### ContentHistory
- **Type**: `StorageMap<BoundedVec<u8>, BoundedVec<ContentVersion, 10>>`
- **Purpose**: Historical website versions (up to 10)
- **Data** (per version):
  - `content_hash: [u8; 32]` - DAG content root
  - `uploaded_at: BlockNumber` - Version timestamp
  - `size_bytes: u64` - Version size
  - `description: BoundedVec<u8, 128>` - Change notes

### TotalDomains
- **Type**: `StorageValue<u32>`
- **Purpose**: Total registered domains (analytics)

### CurrentContentVersion
- **Type**: `StorageMap<BoundedVec<u8>, u32>`
- **Purpose**: Current version number for content history

## Extrinsics

### register_domain
**Purpose**: Register new permanent `.bz` domain

**Parameters**:
- `domain_name: Vec<u8>` - Domain name (e.g., `"alice.bz"`)
- `tier: u8` - Tier code (0=Standard, 1=Premium, 2=Government, 3=Verified)

**Checks**:
- Domain name validation:
  - Length: 3-64 characters
  - Characters: `a-z`, `0-9`, `-` (no special characters)
  - No leading/trailing hyphens
  - `.bz` suffix required
- Domain doesn't already exist
- KYC verification:
  - Standard/Premium: KYC Level 1+ (basic verification)
  - Government: KYC Level 2+ (government employee)
  - Verified: KYC Level 3+ (biometric)
- Sanctions check: `Identity::is_sanctioned(&who) == false`
- Max domains per account: < 100 (configurable)
- Price calculation:
  - **Standard**: 5 DALLA
  - **Premium**: 50 DALLA (if name in premium keywords list)
  - **Government**: 0 DALLA (free for ministries)
  - **Verified**: 100 DALLA (high-trust businesses)

**Effects**:
- Creates `DomainRecord` in storage
- Adds domain to `AccountDomains` for owner
- Collects registration fee (transfers to Treasury)
- Increments `TotalDomains` counter
- Emits `DomainRegistered` event

**Returns**: `Ok(())` or Error

### set_resolution
**Purpose**: Update resolution records for owned domain

**Parameters**:
- `domain_name: Vec<u8>` - Domain to update
- `wallet_address: Option<AccountId>` - Payment address
- `content_hash: Option<[u8; 32]>` - Website DAG root hash
- `metadata: Vec<u8>` - JSON metadata (max 256 bytes)

**Checks**:
- Domain exists
- Caller is domain owner

**Effects**:
- Updates `DomainResolution` storage
- Emits `ResolutionUpdated` event

**Example Metadata**:
```json
{
  "name": "Alice Smith",
  "bio": "Tourism entrepreneur in Belize",
  "twitter": "@alice_belize",
  "website": "https://alice.bz",
  "email": "alice@alice.bz"
}
```

**Returns**: `Ok(())` or Error

### transfer_domain
**Purpose**: Transfer domain ownership to another account

**Parameters**:
- `domain_name: Vec<u8>` - Domain to transfer
- `new_owner: AccountId` - Recipient account

**Checks**:
- Domain exists
- Caller is current owner
- Domain not locked (optional lock mechanism)
- Recipient hasn't reached max domains (100)

**Effects**:
- Updates `DomainRecord.owner` to new owner
- Increments `transfer_count`
- Updates `AccountDomains` for both parties
- Emits `DomainTransferred` event

**Returns**: `Ok(())` or Error

### list_domain
**Purpose**: List domain for sale on marketplace

**Parameters**:
- `domain_name: Vec<u8>` - Domain to sell
- `price: u128` - Sale price in DALLA
- `min_offer: Option<u128>` - Minimum acceptable offer
- `duration_blocks: BlockNumber` - Listing duration (e.g., 30 days)

**Checks**:
- Domain exists
- Caller is owner
- Not already listed

**Effects**:
- Creates `DomainListing` storage entry
- Emits `DomainListed` event

**Returns**: `Ok(())` or Error

### buy_domain
**Purpose**: Purchase domain from marketplace

**Parameters**:
- `domain_name: Vec<u8>` - Domain to buy
- `max_price: u128` - Maximum willing to pay (slippage protection)

**Checks**:
- Listing exists and not expired
- Buyer has sufficient DALLA balance
- Price <= `max_price`
- Buyer hasn't reached max domains (100)

**Effects**:
- Transfers `price * 0.95` to seller (95%)
- Transfers `price * 0.05` to Treasury (5% marketplace fee)
- Updates domain ownership
- Removes listing
- Emits `DomainSold` event

**Fee Breakdown**:
```
Sale price: 1000 DALLA
Seller receives: 950 DALLA (95%)
Treasury receives: 50 DALLA (5%)
```

**Returns**: `Ok(())` or Error

### unlist_domain
**Purpose**: Remove domain from marketplace

**Parameters**:
- `domain_name: Vec<u8>` - Domain to unlist

**Checks**:
- Listing exists
- Caller is seller

**Effects**:
- Removes `DomainListing` entry
- Emits `DomainUnlisted` event (implicit)

**Returns**: `Ok(())` or Error

### activate_hosting
**Purpose**: Enable web hosting for domain (DAG storage)

**Parameters**:
- `domain_name: Vec<u8>` - Domain to host
- `tier: u8` - Hosting tier (0=Free, 1=Basic, 2=Pro, 3=Enterprise)
- `content_hash: [u8; 32]` - Initial DAG content root hash
- `auto_renew: bool` - Enable automatic monthly renewal

**Checks**:
- Caller owns domain
- Hosting not already active
- User has sufficient DALLA for first month's fee

**Effects**:
- Collects first month's fee (if non-free tier)
- Creates `HostingInfo` entry
- Sets expiry to 1 month (432,000 blocks)
- Emits `HostingActivated` event

**Monthly Fees**:
- Free: 0 DALLA
- Basic: 10 DALLA
- Pro: 50 DALLA
- Enterprise: 250 DALLA

**Returns**: `Ok(())` or Error

### renew_hosting
**Purpose**: Extend hosting subscription

**Parameters**:
- `domain_name: Vec<u8>` - Domain to renew
- `months: u32` - Number of months to extend

**Checks**:
- Hosting active
- Caller is subscriber
- User has sufficient DALLA for renewal

**Effects**:
- Collects `monthly_fee * months` in DALLA
- Extends `expires_at` by `432,000 * months` blocks
- Updates `last_payment_at` to current block
- Emits `HostingRenewed` event

**Returns**: `Ok(())` or Error

### deactivate_hosting
**Purpose**: Cancel web hosting

**Parameters**:
- `domain_name: Vec<u8>` - Domain to deactivate

**Checks**:
- Hosting active
- Caller is subscriber

**Effects**:
- Removes `HostingInfo` entry
- Website becomes inaccessible (content remains in Pakit DAG)
- Emits `HostingDeactivated` event (implicit)

**Returns**: `Ok(())` or Error

### update_hosting_content
**Purpose**: Upload new version of hosted website

**Parameters**:
- `domain_name: Vec<u8>` - Domain to update
- `new_content_hash: [u8; 32]` - New DAG content root hash
- `description: Vec<u8>` - Change notes (max 128 bytes)
- `size_bytes: u64` - New content size

**Checks**:
- Hosting active and not expired
- Caller is subscriber
- Size within tier limits

**Effects**:
- Saves current version to `ContentHistory` (max 10 versions)
- Updates `HostingInfo.content_hash` to new hash
- Increments `CurrentContentVersion`
- Emits `ContentUpdated` event

**Returns**: `Ok(())` or Error

## Events

### DomainRegistered
```rust
DomainRegistered {
    domain: BoundedVec<u8>,
    owner: AccountId,
    price: u128,
    tier: u8,
}
```

### ResolutionUpdated
```rust
ResolutionUpdated {
    domain: BoundedVec<u8>,
    owner: AccountId,
}
```

### DomainTransferred
```rust
DomainTransferred {
    domain: BoundedVec<u8>,
    from: AccountId,
    to: AccountId,
}
```

### DomainListed
```rust
DomainListed {
    domain: BoundedVec<u8>,
    seller: AccountId,
    price: u128,
}
```

### DomainSold
```rust
DomainSold {
    domain: BoundedVec<u8>,
    seller: AccountId,
    buyer: AccountId,
    price: u128,
    treasury_fee: u128,
}
```

### HostingActivated
```rust
HostingActivated {
    domain: BoundedVec<u8>,
    owner: AccountId,
    tier: u8,
    content_hash: [u8; 32],
}
```

### HostingRenewed
```rust
HostingRenewed {
    domain: BoundedVec<u8>,
    owner: AccountId,
    blocks_extended: BlockNumber,
}
```

### ContentUpdated
```rust
ContentUpdated {
    domain: BoundedVec<u8>,
    new_content_hash: [u8; 32],
    version: u32,
}
```

## Errors

- `DomainAlreadyExists` - Domain name already registered
- `DomainNotFound` - Domain doesn't exist
- `NotDomainOwner` - Caller doesn't own domain
- `InvalidDomainName` - Name violates format rules
- `DomainLocked` - Transfer locked until expiry
- `MaxDomainsReached` - Account has 100 domains (max)
- `KycRequired` - Caller not KYC Level 1+ verified
- `VerifiedKycRequired` - Verified tier needs Level 3 KYC
- `AccountSanctioned` - Caller is sanctioned (blocked)
- `InvalidTier` - Tier code not 0-3
- `InvalidMetadata` - Metadata exceeds 256 bytes
- `NotListedForSale` - Domain not on marketplace
- `HostingAlreadyActive` - Domain already has hosting
- `HostingNotActive` - Domain doesn't have hosting
- `HostingExpired` - Subscription expired (renew required)
- `InsufficientBalance` - Not enough DALLA for fee
- `ArithmeticOverflow` - Math overflow in calculations
- `ContentHistoryFull` - 10 versions reached (max)

## Integration with Other Pallets

### Identity Pallet (KYC Verification)
```rust
type Identity: BnsIdentityProvider<AccountId>
```
- **Purpose**: Verify KYC levels and sanctions status
- **Methods**:
  - `can_register_domain(&who) -> bool` - KYC Level 1+ check
  - `can_register_verified(&who) -> bool` - KYC Level 3+ check (biometric)
  - `is_sanctioned(&who) -> bool` - Sanctions check
- **Usage**: Called before every domain registration

### Economy Pallet (DALLA Payments)
```rust
type Currency: Currency<AccountId> + ReservableCurrency<AccountId>
```
- **Purpose**: Handle registration fees, marketplace payments, hosting subscriptions
- **Methods**: `transfer()`, `reserve()`, `unreserve()`
- **Usage**: All payment operations

### Pakit Storage (DAG Backend)
- **Integration**: External (not trait-based)
- **Purpose**: Store hosted website content (sovereign DAG storage)
- **Content Hash**: BNS stores DAG root hash, Pakit nodes serve content
- **Access**: `https://alice.bz` → Pakit resolves via BNS pallet query → serves DAG content

### Treasury
```rust
type Treasury: Get<AccountId>
```
- **Purpose**: Receive registration fees and marketplace commissions
- **Fees**:
  - Domain registration: 100% to Treasury
  - Marketplace sales: 5% to Treasury
  - Hosting subscriptions: 100% to Treasury (pays Pakit validators)

## Runtime Configuration

```rust
impl pallet_bns::Config for Runtime {
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type Treasury = TreasuryPalletId;
    type MaxDomainsPerAccount = ConstU32<100>;
    type MaxDomainLength = ConstU32<64>;
    type MaxTextRecords = ConstU32<20>;
    type MinDomainLength = ConstU32<3>;
    type WeightInfo = ();
    type Identity = Identity;
    type GovernanceOrigin = EnsureRootOrHalfCouncil;
}
```

## Testing

### Unit Tests
```bash
cargo test -p pallet-bns
```

Tests cover:
- Domain name validation
- KYC tier enforcement
- Price calculation (Standard/Premium/Government/Verified)
- Marketplace fee calculation (5% Treasury split)
- Hosting tier limits
- Content versioning (max 10 versions)
- Transfer ownership
- Sanctions blocking

### Integration Tests
```bash
./run_integration_tests.sh bns
```

Tests cross-pallet interactions:
- Identity → BNS (KYC verification, sanctions check)
- Economy → BNS (DALLA payments for domains/hosting)
- Treasury → BNS (fee collection)
- Pakit → BNS (DAG content storage and retrieval)

## Usage Example

### Scenario: Tourism Business Registers Domain and Hosts Website

**Context**: "Blue Horizon Resort" wants to register `bluehorizon.bz` for their booking website.

**Step 1: Domain Registration**
```rust
register_domain(
    origin: resort_account,
    domain_name: "bluehorizon.bz".as_bytes().to_vec(),
    tier: 1  // Premium tier (50 DALLA)
)
```

**Price Calculation**:
```
Tier: Premium ("resort" keyword)
Base price: 50 DALLA
KYC check: Level 2 ✅ (business verified)
Sanctions check: Not sanctioned ✅
Payment: 50 DALLA → Treasury
```

**Result**: Domain registered, resort owns `bluehorizon.bz` permanently

**Step 2: Set Resolution Records**
```rust
set_resolution(
    origin: resort_account,
    domain_name: "bluehorizon.bz".as_bytes().to_vec(),
    wallet_address: Some(resort_payment_account),
    content_hash: None,  // Will set later after uploading website
    metadata: r#"{
        "name": "Blue Horizon Resort",
        "email": "info@bluehorizon.bz",
        "phone": "+501-555-0123",
        "address": "Placencia, Belize"
    }"#.as_bytes().to_vec()
)
```

**Result**: Domain now resolves to payment address and metadata

**Step 3: Upload Website to Pakit DAG**
```bash
# Off-chain: Upload website files to Pakit DAG
pakit upload website/ --output-hash
# Returns: 0x1234abcd... (DAG root hash)
```

**Step 4: Activate Hosting**
```rust
activate_hosting(
    origin: resort_account,
    domain_name: "bluehorizon.bz".as_bytes().to_vec(),
    tier: 2,  // Pro tier (50 DALLA/month)
    content_hash: [0x12, 0x34, 0xab, 0xcd, ...],  // DAG root hash
    auto_renew: true
)
```

**Hosting Costs**:
```
Tier: Pro
Monthly fee: 50 DALLA
Storage: 10 GB
Bandwidth: 100 GB
First payment: 50 DALLA → Treasury (funds Pakit validators)
Auto-renewal: Enabled (charges 50 DALLA every 432,000 blocks)
```

**Result**: Website live at `https://bluehorizon.bz` (served by Pakit nodes)

**Step 5: Update Website Content**
```bash
# Off-chain: Make changes, upload new version
pakit upload website-v2/ --output-hash
# Returns: 0x5678efgh...
```

```rust
update_hosting_content(
    origin: resort_account,
    domain_name: "bluehorizon.bz".as_bytes().to_vec(),
    new_content_hash: [0x56, 0x78, 0xef, 0xgh, ...],
    description: "Added winter promotion banner".as_bytes().to_vec(),
    size_bytes: 15_000_000  // 15 MB
)
```

**Result**: Website updated, old version saved in `ContentHistory`

**Step 6: List Domain for Sale** (future)
```rust
list_domain(
    origin: resort_account,
    domain_name: "bluehorizon.bz".as_bytes().to_vec(),
    price: 500_000 * 10^12,  // 500K DALLA (10x original price)
    min_offer: Some(300_000 * 10^12),  // Accept offers >= 300K DALLA
    duration_blocks: 2_592_000  // 60 days (6s blocks)
)
```

**Marketplace Listing**:
```
Sale price: 500,000 DALLA
Seller receives: 475,000 DALLA (95%)
Treasury receives: 25,000 DALLA (5%)
Min offer: 300,000 DALLA
Expires: Block 2,592,000 from listing
```

**Total Costs Summary**:
- Initial registration: 50 DALLA (one-time)
- Hosting (12 months): 600 DALLA (50/month * 12)
- Future sale: +475K DALLA (95% of 500K)
- **Net profit**: +474,350 DALLA (if domain appreciates)

## Future Enhancements

### Phase 2: DNS Integration
- **Feature**: Register external domains (`.com`, `.org`, `.net`)
- **Status**: Planned
- **Benefit**: Unified domain management (traditional + blockchain)
- **Timeline**: Q3 2026

### Phase 3: Subdomain System
- **Feature**: Allow domain owners to create subdomains (e.g., `shop.bluehorizon.bz`)
- **Status**: Planned
- **Benefit**: Hierarchical namespace, additional revenue for domain owners
- **Timeline**: Q4 2026

### Phase 4: Advanced Marketplace
- **Feature**: Auctions, offers, escrow, domain leasing
- **Status**: Planned
- **Benefit**: More liquidity, better price discovery
- **Timeline**: Q1 2027

### Phase 5: ENS/Unstoppable Bridge
- **Feature**: Cross-chain domain resolution (ENS ↔ BNS)
- **Status**: Research phase
- **Benefit**: Interoperability with Ethereum ecosystem
- **Timeline**: Q2 2027

### Phase 6: Decentralized Email
- **Feature**: Email service for BNS domains (e.g., `alice@alice.bz`)
- **Status**: Planned
- **Benefit**: Complete decentralized digital identity
- **Timeline**: Q3 2027

## References

- [Ethereum Name Service (ENS)](https://ens.domains/) - Domain system inspiration
- [Unstoppable Domains](https://unstoppabledomains.com/) - NFT domain model
- [Handshake](https://handshake.org/) - Decentralized DNS root
- [Pakit Storage](../pakit/README.md) - DAG storage backend
- [Identity Pallet](../identity/README.md) - KYC verification system
- [Economy Pallet](../economy/README.md) - DALLA payment system
- [ICANN Domain Guidelines](https://www.icann.org/) - Traditional DNS standards

## License
This pallet is part of BelizeChain and is licensed under GPL-3.0.
