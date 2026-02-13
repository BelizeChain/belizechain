# Services Pallet APIs

**BNS • LandLedger • Payroll • Community • Contracts**

Comprehensive API reference for BelizeChain's service-oriented pallets.

---

## BNS Pallet (Belize Name Service)

**.bz domain registry with DAG hosting and marketplace**

### Extrinsics

#### `register_domain`
Register new .bz domain (KYC required)

```rust
pub fn register_domain(
    origin: OriginFor<T>,
    domain: BoundedVec<u8, ConstU32<64>>,
    tier: DomainTier
) -> DispatchResult
```

**Parameters:**
- `domain`: Domain name (3-63 chars, lowercase, alphanumeric + hyphens)
- `tier`: `Standard`, `Premium`, `Government`, or `Verified`

**Pricing:**
- Standard: 100 DALLA/year
- Premium: 1,000 DALLA/year (single word, 3-5 chars)
- Government: Free (.gov.bz)
- Verified: 500 DALLA/year (business verification)

**Events:**
- `DomainRegistered(AccountId, Vec<u8>, DomainTier)`

**Weights:** 55M + 3 reads + 2 writes

```javascript
// JavaScript example
await api.tx.bns.registerDomain(
  'mycompany',
  'Standard'
).signAndSend(owner, { value: 100_000_000_000_000n });  // 100 DALLA
```

#### `update_domain_content`
Update domain content hosted on Pakit DAG

```rust
pub fn update_domain_content(
    origin: OriginFor<T>,
    domain: BoundedVec<u8, ConstU32<64>>,
    content_hash: H256  // Pakit DAG hash
) -> DispatchResult
```

**Process:**
1. Upload website to Pakit DAG storage
2. Get content hash from Pakit
3. Update on-chain domain record

**Events:**
- `DomainContentUpdated(Vec<u8>, H256)`

**Weights:** 40M + 2 reads + 1 write

```python
# Python example
import requests

# Upload website to Pakit
files = {'file': open('website.tar.gz', 'rb')}
response = requests.post('http://localhost:8890/upload', files=files)
content_hash = response.json()['hash']

# Update domain on-chain
tx = api.compose_call(
    call_module='BNS',
    call_function='update_domain_content',
    call_params={
        'domain': 'mycompany',
        'content_hash': content_hash
    }
)
```

#### `list_domain_for_sale`
List domain on marketplace

```rust
pub fn list_domain_for_sale(
    origin: OriginFor<T>,
    domain: BoundedVec<u8, ConstU32<64>>,
    price: BalanceOf<T>
) -> DispatchResult
```

**Marketplace fee:** 5% to treasury

**Events:**
- `DomainListedForSale(Vec<u8>, Balance)`

**Weights:** 35M + 2 reads + 1 write

#### `buy_domain`
Purchase domain from marketplace

```rust
pub fn buy_domain(
    origin: OriginFor<T>,
    domain: BoundedVec<u8, ConstU32<64>>
) -> DispatchResult
```

**Payment distribution:**
- 95% to seller
- 5% to treasury

**Events:**
- `DomainSold(Vec<u8>, AccountId, AccountId, Balance)` - (domain, seller, buyer, price)

**Weights:** 60M + 4 reads + 3 writes

### Storage

#### `Domains`
```rust
pub type Domains<T> = StorageMap<
    _,
    Blake2_128Concat,
    BoundedVec<u8, ConstU32<64>>,  // Domain name
    DomainInfo<T::AccountId, T::BlockNumber>,
    OptionQuery
>;

pub struct DomainInfo<AccountId, BlockNumber> {
    pub owner: AccountId,
    pub tier: DomainTier,
    pub content_hash: Option<H256>,
    pub registered_at: BlockNumber,
    pub expires_at: BlockNumber,
    pub for_sale: bool,
    pub sale_price: Option<Balance>
}
```

---

## LandLedger Pallet

**Property registry with Pakit document storage integration**

### Extrinsics

#### `register_property`
Register property with land title

```rust
pub fn register_property(
    origin: OriginFor<T>,
    property_id: u32,
    document_hash: H256,  // Land title stored in Pakit
    land_area_sqm: u64,
    location: BoundedVec<u8, ConstU32<128>>
) -> DispatchResult
```

**Requirements:**
- KYC level: Enhanced (property ownership)
- Document verification: Land title uploaded to Pakit
- Government approval: Ministry of Natural Resources signature

**Events:**
- `PropertyRegistered(u32, AccountId, H256, u64)`

**Weights:** 70M + 4 reads + 2 writes

```typescript
// TypeScript example
const documentHash = await uploadDocumentToPakit(landTitle);

await api.tx.landLedger.registerProperty(
  propertyId,
  documentHash,
  5000,  // 5000 sqm
  'Mile 5, Western Highway, Cayo District'
).signAndSend(owner);
```

#### `transfer_property`
Transfer property ownership

```rust
pub fn transfer_property(
    origin: OriginFor<T>,
    property_id: u32,
    new_owner: T::AccountId,
    transfer_document_hash: H256
) -> DispatchResult
```

**Requirements:**
- Both parties KYC verified
- Transfer deed uploaded to Pakit
- Stamp duty paid (5% of property value)

**Events:**
- `PropertyTransferred(u32, AccountId, AccountId, H256)` - (property, old_owner, new_owner, deed)

**Weights:** 80M + 5 reads + 3 writes

#### `register_document_proof`
Register Pakit storage proof for property document

```rust
pub fn register_document_proof(
    origin: OriginFor<T>,
    property_id: u32,
    document_hash: H256,
    merkle_proof: BoundedVec<H256, ConstU32<32>>
) -> DispatchResult
```

**Verification:**
- Compute Merkle root from proof
- Verify against Pakit DAG storage
- Store on-chain for legal compliance

**Events:**
- `DocumentProofRegistered(u32, H256, H256)` - (property, document_hash, merkle_root)

**Weights:** 50M + 3 reads + 1 write

### Storage

#### `Properties`
```rust
pub type Properties<T> = StorageMap<
    _,
    Blake2_128Concat,
    u32,  // Property ID
    PropertyInfo<T::AccountId, T::BlockNumber>,
    OptionQuery
>;

pub struct PropertyInfo<AccountId, BlockNumber> {
    pub owner: AccountId,
    pub document_hash: H256,
    pub land_area_sqm: u64,
    pub location: BoundedVec<u8, ConstU32<128>>,
    pub registered_at: BlockNumber,
    pub last_transfer: Option<BlockNumber>
}
```

#### `DocumentProofs`
```rust
pub type DocumentProofs<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, u32,  // Property ID
    Blake2_128Concat, H256,  // Document hash
    MerkleProof,
    OptionQuery
>;

pub struct MerkleProof {
    pub proof: BoundedVec<H256, ConstU32<32>>,
    pub merkle_root: H256,
    pub verified_at: BlockNumber
}
```

---

## Payroll Pallet

**Enterprise payroll management for all business types — government, enterprise, SME, cooperative, gig platforms, and nonprofits**

### Extrinsics

#### `register_employer`
Register as employer (government or business)

```rust
pub fn register_employer(
    origin: OriginFor<T>,
    employer_type: EmployerType,
    tax_id: BoundedVec<u8, ConstU32<32>>
) -> DispatchResult
```

**Types:**
- `Government`: Ministry, department, statutory body
- `Private`: Business with tax ID

**Events:**
- `EmployerRegistered(AccountId, EmployerType)`

**Weights:** 45M + 2 reads + 1 write

#### `add_employee`
Add employee to payroll

```rust
pub fn add_employee(
    origin: OriginFor<T>,
    employee: T::AccountId,
    salary: BalanceOf<T>,
    payment_schedule: PaymentSchedule
) -> DispatchResult
```

**Schedules:**
- `Weekly`: Every 7 days
- `Biweekly`: Every 14 days
- `Monthly`: Every 30 days

**Events:**
- `EmployeeAdded(AccountId, AccountId, Balance, PaymentSchedule)`

**Weights:** 50M + 3 reads + 2 writes

```javascript
// Add employee with monthly salary
await api.tx.payroll.addEmployee(
  employeeAddress,
  4000_000_000_000_000n,  // 4000 DALLA/month
  'Monthly'
).signAndSend(employer);
```

#### `process_payroll`
Process scheduled payroll payments

```rust
pub fn process_payroll(
    origin: OriginFor<T>,
    employer: T::AccountId
) -> DispatchResult
```

**Automation:** Can be triggered by off-chain worker on schedule

**Deductions:**
- Social Security: 8% (employee) + 8% (employer)
- Income Tax: Progressive (0-25%)
- GST: 12.5% (if applicable)

**Events:**
- `PayrollProcessed(AccountId, u32, Balance)` - (employer, employee_count, total_paid)

**Weights:** 100M + variable (depends on employee count)

### Storage

#### `Employers`
```rust
pub type Employers<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    EmployerInfo<T::BlockNumber>,
    OptionQuery
>;

pub struct EmployerInfo<BlockNumber> {
    pub employer_type: EmployerType,
    pub tax_id: BoundedVec<u8, ConstU32<32>>,
    pub registered_at: BlockNumber,
    pub employee_count: u32
}
```

#### `Employees`
```rust
pub type Employees<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,  // Employer
    Blake2_128Concat, T::AccountId,  // Employee
    EmployeeInfo<BalanceOf<T>, T::BlockNumber>,
    OptionQuery
>;

pub struct EmployeeInfo<Balance, BlockNumber> {
    pub salary: Balance,
    pub payment_schedule: PaymentSchedule,
    pub last_payment: Option<BlockNumber>,
    pub total_paid: Balance
}
```

---

## Community Pallet

**Community governance and proposals**

### Extrinsics

#### `create_community`
Create local community group

```rust
pub fn create_community(
    origin: OriginFor<T>,
    name: BoundedVec<u8, ConstU32<64>>,
    district: DistrictId
) -> DispatchResult
```

**Districts:** Belize, Cayo, Corozal, Orange Walk, Stann Creek, Toledo

**Events:**
- `CommunityCreated(CommunityId, AccountId, Vec<u8>, DistrictId)`

**Weights:** 40M + 2 reads + 1 write

#### `submit_community_proposal`
Submit proposal for community vote

```rust
pub fn submit_community_proposal(
    origin: OriginFor<T>,
    community_id: CommunityId,
    proposal_text: BoundedVec<u8, ConstU32<256>>,
    execution_call: Option<Box<<T as Config>::RuntimeCall>>
) -> DispatchResult
```

**Voting:** Simple majority, 7-day voting period

**Events:**
- `CommunityProposalSubmitted(CommunityId, ProposalId, AccountId)`

**Weights:** 55M + 3 reads + 2 writes

### Storage

#### `Communities`
```rust
pub type Communities<T> = StorageMap<
    _,
    Blake2_128Concat,
    CommunityId,
    CommunityInfo<T::AccountId, T::BlockNumber>,
    OptionQuery
>;
```

---

## Contracts Pallet

**Wasm smart contract execution (pallet-contracts wrapper)**

### Extrinsics

#### `instantiate_with_code`
Deploy new smart contract

```rust
pub fn instantiate_with_code(
    origin: OriginFor<T>,
    value: BalanceOf<T>,
    gas_limit: Weight,
    storage_deposit_limit: Option<BalanceOf<T>>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>
) -> DispatchResult
```

**Storage deposit:** 100 DALLA per KB

**Events:**
- `Instantiated(AccountId, ContractAddress)`

**Weights:** Variable (depends on contract size)

```bash
# Deploy via cargo-contract CLI
cargo contract instantiate \
  --suri //Alice \
  --constructor new \
  --args 1000000000000 \
  --execute
```

#### `call`
Call smart contract method

```rust
pub fn call(
    origin: OriginFor<T>,
    dest: T::AccountId,
    value: BalanceOf<T>,
    gas_limit: Weight,
    storage_deposit_limit: Option<BalanceOf<T>>,
    data: Vec<u8>
) -> DispatchResult
```

**Gas pricing:** Dynamic based on network load

**Events:**
- `ContractExecution(AccountId, AccountId, Vec<u8>)` - (caller, contract, return_data)

**Weights:** Variable (depends on contract logic)

### Storage

Managed by `pallet-contracts` (see GEM documentation for contract-specific storage)

---

## Type Definitions

```rust
// BNS
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum DomainTier {
    Standard,
    Premium,
    Government,
    Verified
}

// Payroll
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EmployerType {
    Government,
    Private
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum PaymentSchedule {
    Weekly,
    Biweekly,
    Monthly
}

// Community
pub type CommunityId = u32;
pub type DistrictId = u8;  // 0-5 for 6 districts
```

---

## Related Documentation

- [Core Pallet APIs](./pallet-apis-core.md)
- [Financial Pallet APIs](./pallet-apis-financial.md)
- [Infrastructure Pallet APIs](./pallet-apis-infrastructure.md)
- [BNS Overview](../services/bns-overview.md)
- [LandLedger Guide](../user-guides/landledger.md)
- [GEM Contracts](../smart-contracts/gem-platform.md)
