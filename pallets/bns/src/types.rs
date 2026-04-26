//! BNS Type Definitions

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;

// ===== DOMAIN TIER =====

/// Domain tier pricing and features
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum DomainTier {
    /// Standard domain (100 DALLA)
    Standard,
    /// Premium domain (500 DALLA) - enhanced features
    Premium,
    /// Government domain (50 DALLA) - subsidized for public sector
    Government,
    /// Verified domain (1000 DALLA) - requires Level 3 KYC
    Verified,
}

impl DomainTier {
    /// Convert to u8 for events
    pub fn to_u8(&self) -> u8 {
        match self {
            DomainTier::Standard => 0,
            DomainTier::Premium => 1,
            DomainTier::Government => 2,
            DomainTier::Verified => 3,
        }
    }
}

// ===== HOSTING TIER =====

/// Web hosting tier (monthly fees)
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum HostingTier {
    /// Free tier - 100MB storage, basic DAG
    Free,
    /// Basic tier - 1GB storage, DAG with CDN (10 DALLA/month)
    Basic,
    /// Pro tier - 10GB storage, full CDN (50 DALLA/month)
    Pro,
    /// Enterprise tier - 100GB storage, premium CDN (200 DALLA/month)
    Enterprise,
}

impl HostingTier {
    /// Convert to u8 for events
    pub fn to_u8(&self) -> u8 {
        match self {
            HostingTier::Free => 0,
            HostingTier::Basic => 1,
            HostingTier::Pro => 2,
            HostingTier::Enterprise => 3,
        }
    }
}

// ===== DOMAIN RECORD =====

/// Core domain ownership record (immutable once registered)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct DomainRecord<AccountId, BlockNumber> {
    /// Current owner (can be transferred)
    pub owner: AccountId,
    /// Original registrant (permanent record)
    pub original_owner: AccountId,
    /// Block number when registered
    pub registered_at: BlockNumber,
    /// Original purchase price (historical data)
    pub purchase_price: u128,
    /// Domain tier
    pub tier: DomainTier,
    /// Transfer lock expiry (prevents rapid trading)
    pub locked_until: Option<BlockNumber>,
    /// Number of transfers (ownership history counter)
    pub transfer_count: u32,
}

// ===== RESOLUTION RECORDS =====

/// Domain resolution data (wallet, content, metadata)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxTextRecords))]
pub struct ResolutionRecords<AccountId, MaxTextRecords: Get<u32>> {
    /// Wallet address (for payment routing)
    pub wallet_address: Option<AccountId>,
    /// Content hash (DAG block hash for web hosting)
    pub content_hash: Option<[u8; 32]>,
    /// Avatar/profile image hash
    pub avatar: Option<[u8; 32]>,
    /// General metadata (JSON encoded)
    pub metadata: BoundedVec<u8, frame_support::traits::ConstU32<256>>,
    /// Custom text records (email, url, twitter, etc.)
    pub text_records: BoundedVec<TextRecord, MaxTextRecords>,
}

/// Custom text record for arbitrary data
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct TextRecord {
    /// Record key (e.g., "email", "twitter", "github")
    pub key: BoundedVec<u8, frame_support::traits::ConstU32<32>>,
    /// Record value
    pub value: BoundedVec<u8, frame_support::traits::ConstU32<128>>,
}

// ===== MARKETPLACE =====

/// Domain listing for sale
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct DomainListing<AccountId, BlockNumber> {
    /// Seller account
    pub seller: AccountId,
    /// Asking price
    pub price: u128,
    /// Listed at block number
    pub listed_at: BlockNumber,
    /// Expiry block (auto-delist)
    pub expires_at: BlockNumber,
    /// Minimum acceptable offer (for bidding)
    pub min_offer: Option<u128>,
}

// ===== WEB HOSTING =====

/// Active web hosting subscription
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct HostingInfo<AccountId, BlockNumber> {
    /// Account paying for hosting
    pub subscriber: AccountId,
    /// Hosting tier
    pub tier: HostingTier,
    /// DAG content root hash (manifest block)
    pub content_hash: [u8; 32],
    /// Hosting activated at block
    pub activated_at: BlockNumber,
    /// Last payment block
    pub last_payment_at: BlockNumber,
    /// Hosting expires at block (subscription end)
    pub expires_at: BlockNumber,
    /// Total data stored (bytes)
    pub data_size: u64,
    /// Monthly fee amount
    pub monthly_fee: u128,
    /// Auto-renewal enabled
    pub auto_renew: bool,
}

// ===== EXTERNAL DOMAINS =====

/// External domain (.com, .org, .net, etc.) registration
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T, MaxDomainLength))]
pub struct ExternalDomainInfo<AccountId, BlockNumber, MaxDomainLength: Get<u32>> {
    /// Domain owner
    pub owner: AccountId,
    /// BNS domain to link to (e.g., mysite.bz)
    pub linked_bns_domain: BoundedVec<u8, MaxDomainLength>,
    /// Hosting tier
    pub tier: HostingTier,
    /// Verification token (for DNS TXT record)
    pub verification_token: [u8; 32],
    /// Verification status
    pub verified: bool,
    /// Registered at block
    pub registered_at: BlockNumber,
    /// Monthly fee
    pub monthly_fee: u128,
    /// Expires at block
    pub expires_at: BlockNumber,
}

/// Domain verification status
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct VerificationStatus<BlockNumber> {
    /// Verification token
    pub token: [u8; 32],
    /// Verification attempts
    pub attempts: u8,
    /// Last verification attempt block
    pub last_attempt: BlockNumber,
    /// Verified successfully
    pub verified: bool,
}

// ===== PROVIDER TRAITS =====

/// Identity provider trait for KYC checks
pub trait BnsIdentityProvider<AccountId> {
    /// Check if account can register domains (basic KYC)
    fn can_register_domain(account: &AccountId) -> bool;

    /// Check if account can register verified domains (Level 3 KYC)
    fn can_register_verified(account: &AccountId) -> bool;

    /// Check if account is sanctioned
    fn is_sanctioned(account: &AccountId) -> bool;
}

// Import Get trait for BoundedVec usage
use frame_support::traits::Get;

// ===== CONTENT VERSIONING =====

/// Content version record for rollback capability
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct ContentVersion<BlockNumber> {
    /// IPFS content hash
    pub content_hash: [u8; 32],
    /// When this version was uploaded
    pub uploaded_at: BlockNumber,
    /// Optional description/commit message
    pub description: BoundedVec<u8, frame_support::traits::ConstU32<128>>,
    /// File size in bytes (for bandwidth tracking)
    pub size_bytes: u64,
}

// ===== SSL/TLS CERTIFICATES =====

/// SSL/TLS certificate information
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SSLCertInfo<BlockNumber> {
    /// SHA-256 hash of certificate (for verification)
    pub cert_hash: [u8; 32],
    /// When certificate was issued
    pub issued_at: BlockNumber,
    /// When certificate expires (typically +90 days)
    pub expires_at: BlockNumber,
    /// Certificate serial number (for revocation check)
    pub serial_number: BoundedVec<u8, frame_support::traits::ConstU32<64>>,
    /// Issuer (e.g., "Let's Encrypt Authority X3")
    pub issuer: BoundedVec<u8, frame_support::traits::ConstU32<128>>,
}
