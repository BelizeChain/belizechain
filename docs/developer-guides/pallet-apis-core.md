# Core Pallet APIs

**Economy • Identity • Governance • Compliance**

Comprehensive API reference for BelizeChain's foundational pallets.

---

## Economy Pallet

**Multi-currency management with DALLA (native) and bBZD (BZD-pegged stablecoin)**

### Extrinsics

#### `mint_bbzd`
Mint bBZD stablecoin (Central Bank only, 1:1 BZD backing required)

```rust
pub fn mint_bbzd(
    origin: OriginFor<T>,
    account: T::AccountId,
    amount: BalanceOf<T>,
    proof: OffChainProof
) -> DispatchResult
```

**Parameters:**
- `origin`: Must be Central Bank multi-sig account
- `account`: Recipient of bBZD
- `amount`: Amount to mint (12 decimals)
- `proof`: Off-chain proof of BZD deposit

**Events:**
- `bBZDMinted(AccountId, Balance)`

**Errors:**
- `Unauthorized`: Not Central Bank account
- `InvalidProof`: BZD deposit proof verification failed
- `ExceedsReserves`: Attempting to mint more than BZD reserves

**Weights:** 50M + 2 reads + 2 writes

```javascript
// JavaScript example
const tx = api.tx.economy.mintBbzd(
  recipientAddress,
  1000_000_000_000_000n,  // 1000 bBZD
  proof
);

await tx.signAndSend(centralBankAccount);
```

#### `apply_tourism_cashback`
Apply 5-8% cashback for tourism merchant transactions

```rust
pub fn apply_tourism_cashback(
    origin: OriginFor<T>,
    merchant: T::AccountId,
    transaction_amount: BalanceOf<T>
) -> DispatchResult
```

**Parameters:**
- `merchant`: Verified tourism merchant address
- `transaction_amount`: Original transaction in DALLA

**Returns:** Cashback in bBZD based on merchant category

**Events:**
- `TourismCashbackApplied(AccountId, AccountId, Balance, u8)` - (customer, merchant, cashback, rate%)

**Weights:** 35M + 3 reads + 1 write

```rust
// Rust example
let result = Economy::apply_tourism_cashback(
    Origin::signed(customer),
    merchant_account,
    10_000 * DALLA
)?;
```

#### `register_merchant`
Register business as merchant with category and KYC level

```rust
pub fn register_merchant(
    origin: OriginFor<T>,
    category: MerchantCategory,
    kyc_level: u8
) -> DispatchResult
```

**Categories:**
- `Hotels` (8% cashback)
- `Restaurants` (6% cashback)
- `Tours` (7% cashback)
- `Crafts` (5% cashback)
- `Retail` (0% cashback)

**Events:**
- `MerchantRegistered(AccountId, MerchantCategory)`

**Weights:** 30M + 2 reads + 1 write

### Storage

#### `Merchants`
```rust
pub type Merchants<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    MerchantInfo<T::AccountId>,
    OptionQuery
>;

pub struct MerchantInfo<AccountId> {
    pub category: MerchantCategory,
    pub kyc_level: u8,
    pub registered_at: BlockNumber,
    pub total_transactions: u64,
    pub total_volume: Balance,
    pub verified: bool
}
```

#### `bBZDSupply`
```rust
pub type bBZDSupply<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;
```

Total circulating bBZD supply (should equal off-chain BZD reserves)

#### `CentralBankAccount`
```rust
pub type CentralBankAccount<T> = StorageValue<_, T::AccountId, OptionQuery>;
```

Multi-sig account authorized to mint bBZD (4-of-7 signatures required)

---

## Identity Pallet

**BelizeID system with SSN/Passport integration and KYC levels**

### Extrinsics

#### `register_belizeid`
Register new BelizeID with SSN or Passport

```rust
pub fn register_belizeid(
    origin: OriginFor<T>,
    identity_type: IdentityType,
    document_hash: H256,
    kyc_level: KycLevel
) -> DispatchResult
```

**Parameters:**
- `identity_type`: `SSN` or `Passport`
- `document_hash`: SHA-256 hash of document stored in Pakit
- `kyc_level`: `Basic`, `Verified`, or `Enhanced`

**Events:**
- `BelizeIDRegistered(AccountId, IdentityType, KycLevel)`

**Weights:** 45M + 3 reads + 2 writes

```typescript
// TypeScript example
const documentHash = await uploadToPakit(ssnDocument);

await api.tx.identity.registerBelizeid(
  'SSN',
  documentHash,
  'Verified'
).signAndSend(account);
```

#### `update_kyc_level`
Upgrade KYC level (government authority only)

```rust
pub fn update_kyc_level(
    origin: OriginFor<T>,
    account: T::AccountId,
    new_level: KycLevel
) -> DispatchResult
```

**Requirements:**
- Basic → Verified: In-person verification
- Verified → Enhanced: Background check + biometric scan

**Events:**
- `KycLevelUpdated(AccountId, KycLevel, KycLevel)` - (account, old, new)

**Weights:** 40M + 2 reads + 1 write

#### `verify_identity`
On-chain identity verification query (read-only via runtime call)

```rust
pub fn verify_identity(
    account: T::AccountId
) -> Result<IdentityInfo<T::AccountId>, DispatchError>
```

**Returns:**
```rust
pub struct IdentityInfo<AccountId> {
    pub identity_type: IdentityType,
    pub kyc_level: KycLevel,
    pub document_hash: H256,
    pub verified_at: BlockNumber,
    pub verified_by: AccountId,
    pub expiry: BlockNumber
}
```

### Storage

#### `Identities`
```rust
pub type Identities<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    IdentityInfo<T::AccountId>,
    OptionQuery
>;
```

#### `SSNStandardVersion`
```rust
pub type SSNStandardVersion<T> = StorageValue<_, u32, ValueQuery>;
```

Current SSN format version (v1 = 9-digit, v2 = 12-digit with checksum)

### Events

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    BelizeIDRegistered(T::AccountId, IdentityType, KycLevel),
    KycLevelUpdated(T::AccountId, KycLevel, KycLevel),
    IdentityRevoked(T::AccountId, RevocationReason),
    DocumentUpdated(T::AccountId, H256)
}
```

---

## Governance Pallet

**District council democracy with proposal lifecycle**

### Extrinsics

#### `propose`
Submit governance proposal

```rust
pub fn propose(
    origin: OriginFor<T>,
    proposal: Box<<T as Config>::Proposal>,
    value: BalanceOf<T>
) -> DispatchResult
```

**Parameters:**
- `proposal`: Encoded runtime call to execute if approved
- `value`: Treasury amount requested (0 for non-financial proposals)

**Deposit:** 1000 DALLA (refunded if approved, slashed if rejected)

**Events:**
- `ProposalSubmitted(ProposalIndex, AccountId, Balance)`

**Weights:** 60M + 3 reads + 2 writes

```javascript
// Propose treasury spending
const proposal = api.tx.treasury.approveProposal(
  treasuryProposalId
);

await api.tx.governance.propose(
  proposal,
  50000_000_000_000_000n  // 50K DALLA
).signAndSend(proposer);
```

#### `vote`
Vote on active proposal

```rust
pub fn vote(
    origin: OriginFor<T>,
    proposal_id: ProposalIndex,
    vote: Vote
) -> DispatchResult
```

**Vote types:**
- `Aye`: Approve
- `Nay`: Reject
- `Abstain`: Count toward quorum but neutral

**Voting power:** 1 DALLA staked = 1 vote

**Events:**
- `VoteCast(ProposalIndex, AccountId, Vote, Balance)` - (proposal, voter, vote, voting_power)

**Weights:** 35M + 4 reads + 2 writes

#### `execute_proposal`
Execute approved proposal (automatic after voting period)

```rust
pub fn execute_proposal(
    origin: OriginFor<T>,
    proposal_id: ProposalIndex
) -> DispatchResult
```

**Requirements:**
- Voting period ended
- Quorum reached (>50% of staked DALLA participated)
- Majority vote Aye

**Events:**
- `ProposalExecuted(ProposalIndex, DispatchResult)`

**Weights:** 100M + 5 reads + 3 writes + proposal weight

### Storage

#### `Proposals`
```rust
pub type Proposals<T> = StorageMap<
    _,
    Blake2_128Concat,
    ProposalIndex,
    ProposalInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>,
    OptionQuery
>;

pub struct ProposalInfo<AccountId, BlockNumber, Balance> {
    pub proposer: AccountId,
    pub proposal_hash: H256,
    pub value: Balance,
    pub voting_ends: BlockNumber,
    pub ayes: Balance,
    pub nays: Balance,
    pub abstains: Balance,
    pub executed: bool
}
```

#### `DistrictCouncils`
```rust
pub type DistrictCouncils<T> = StorageMap<
    _,
    Blake2_128Concat,
    DistrictId,
    Vec<T::AccountId>,
    ValueQuery
>;
```

6 districts: Belize, Cayo, Corozal, Orange Walk, Stann Creek, Toledo

---

## Compliance Pallet

**KYC/AML enforcement with FSC oversight**

### Extrinsics

#### `submit_kyc_application`
Submit KYC application for review

```rust
pub fn submit_kyc_application(
    origin: OriginFor<T>,
    documents: BoundedVec<H256, ConstU32<10>>,
    requested_level: KycLevel
) -> DispatchResult
```

**Documents required:**
- **Basic**: Photo ID
- **Verified**: Photo ID + Proof of address
- **Enhanced**: Photo ID + Proof of address + Biometric scan + Background check

**Events:**
- `KycApplicationSubmitted(AccountId, KycLevel, u32)` - (applicant, level, application_id)

**Weights:** 55M + 2 reads + 2 writes

#### `approve_kyc`
Approve KYC application (FSC officer only)

```rust
pub fn approve_kyc(
    origin: OriginFor<T>,
    application_id: u32
) -> DispatchResult
```

**Requirements:**
- Must be authorized FSC officer
- All documents verified
- No AML red flags

**Events:**
- `KycApproved(u32, AccountId, KycLevel)` - (application_id, applicant, level)

**Weights:** 50M + 4 reads + 3 writes

#### `flag_suspicious_activity`
Flag account for AML review

```rust
pub fn flag_suspicious_activity(
    origin: OriginFor<T>,
    account: T::AccountId,
    reason: SuspicionReason,
    evidence: BoundedVec<H256, ConstU32<5>>
) -> DispatchResult
```

**Triggers:**
- Large transactions (>100K DALLA)
- Rapid velocity (>50 tx/hour)
- High-risk jurisdictions
- Matching watchlist

**Events:**
- `SuspiciousActivityFlagged(AccountId, SuspicionReason)`

**Weights:** 45M + 3 reads + 2 writes

### Storage

#### `KycApplications`
```rust
pub type KycApplications<T> = StorageMap<
    _,
    Blake2_128Concat,
    u32,
    KycApplication<T::AccountId, T::BlockNumber>,
    OptionQuery
>;

pub struct KycApplication<AccountId, BlockNumber> {
    pub applicant: AccountId,
    pub requested_level: KycLevel,
    pub documents: BoundedVec<H256, ConstU32<10>>,
    pub submitted_at: BlockNumber,
    pub status: ApplicationStatus,
    pub reviewer: Option<AccountId>
}
```

#### `AmlWatchlist`
```rust
pub type AmlWatchlist<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    WatchlistEntry<T::BlockNumber>,
    OptionQuery
>;
```

#### `FscOfficers`
```rust
pub type FscOfficers<T> = StorageValue<_, BoundedVec<T::AccountId, ConstU32<50>>, ValueQuery>;
```

Authorized Financial Services Commission officers

### Events

```rust
#[pallet::event]
pub enum Event<T: Config> {
    KycApplicationSubmitted(T::AccountId, KycLevel, u32),
    KycApproved(u32, T::AccountId, KycLevel),
    KycRejected(u32, T::AccountId, RejectionReason),
    SuspiciousActivityFlagged(T::AccountId, SuspicionReason),
    AccountFrozen(T::AccountId, FreezeReason),
    AccountUnfrozen(T::AccountId)
}
```

---

## Type Definitions

```rust
// Economy
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MerchantCategory {
    Hotels,
    Restaurants,
    Tours,
    Crafts,
    Retail
}

// Identity
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum IdentityType {
    SSN,
    Passport
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum KycLevel {
    None,
    Basic,
    Verified,
    Enhanced
}

// Governance
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Vote {
    Aye,
    Nay,
    Abstain
}

// Compliance
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum SuspicionReason {
    LargeTransaction,
    HighVelocity,
    HighRiskJurisdiction,
    WatchlistMatch,
    StructuringPattern
}
```

---

## Error Handling

```rust
#[pallet::error]
pub enum Error<T> {
    // Economy
    Unauthorized,
    InvalidProof,
    ExceedsReserves,
    MerchantNotFound,
    InsufficientBalance,
    
    // Identity
    IdentityAlreadyExists,
    InvalidDocument,
    KycExpired,
    InsufficientKycLevel,
    
    // Governance
    ProposalNotFound,
    VotingPeriodEnded,
    QuorumNotReached,
    ProposalAlreadyExecuted,
    InsufficientDeposit,
    
    // Compliance
    ApplicationNotFound,
    NotAuthorizedOfficer,
    DocumentVerificationFailed,
    AccountFrozen,
    AmlCheckFailed
}
```

---

## Related Documentation

- [Multi-Repo Overview](../architecture/multi-repo-overview.md)
- [Economics Documentation](../economics/tokenomics.md)
- [Governance System](../governance/democracy.md)
- [Compliance Guide](../security/compliance.md)
