# BelizeChain Land Ledger: Blockchain-Based Property Registry

## Overview

The **Land Ledger** pallet (`pallet-belize-landledger`) implements a sovereign blockchain-based property registry system for all Belizean land. It provides immutable ownership records, transparent property transfers, government verification workflows, tourism property investment tracking, and environmental compliance monitoring—all integrated with Belize's physical land offices.

**Key Features**:
- **Digital Land Titles**: Blockchain-anchored ownership records (permanent, immutable)
- **Government Integration**: Direct verification by Land Registry Office, surveyors, environmental authorities
- **Property Transfers**: KYC-enforced transfers with transfer tax (10% default, governance-adjustable)
- **Tourism Tracking**: Flag properties for tourism investment incentives
- **Environmental Compliance**: Monitor developments for environmental clearances
- **GPS Coordinates**: Latitude/longitude storage for property boundaries
- **Encumbrances**: On-chain liens, mortgages, easements, and restrictions
- **Zoning Management**: Residential, commercial, agricultural, industrial, conservation classifications
- **Document Storage**: Pakit DAG integration for title deeds, surveys, environmental reports
- **Oracle Integration**: External verification via Oracle pallet (KYC, sanctions, land registry data)

---

## Architecture

### 1. Property Registry Model

```
┌──────────────────────────────────────────────────────────┐
│         Physical Land (Belize Territory)                 │
│  - GPS coordinates (lat, lon)                            │
│  - Area in square meters                                 │
│  - Boundaries surveyed by licensed surveyor              │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼ (Registered on-chain)
┌──────────────────────────────────────────────────────────┐
│         PropertyRecord (On-Chain Storage)                │
│  ✓ Property ID (unique u32)                              │
│  ✓ Owner (AccountId)                                     │
│  ✓ Title number (64 bytes, e.g., "BZ-LAND-2024-00123")  │
│  ✓ GPS coordinates (i64, i64)                            │
│  ✓ Area (u32 sqm)                                        │
│  ✓ Property type (Residential/Commercial/etc.)           │
│  ✓ Assessed value (bBZD)                                 │
│  ✓ Registration timestamp                                │
│  ✓ Government verified: bool                             │
│  ✓ Surveyed: bool                                        │
│  ✓ Environmental clearance: bool                         │
│  ✓ Tourism property: bool                                │
│  ✓ Zoning type (Residential/Industrial/etc.)             │
│  ✓ Encumbrances (up to 10: mortgages, liens, etc.)      │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼ (Supporting documents)
┌──────────────────────────────────────────────────────────┐
│      Pakit DAG Storage (Off-Chain Documents)             │
│  - Title deed scan (PDF)                                 │
│  - Survey maps (images, CAD files)                       │
│  - Environmental impact assessments                      │
│  - Historical transfer records                           │
│  - Encumbrance documentation                             │
└──────────────────────────────────────────────────────────┘
```

### 2. Property Lifecycle

```
1. REGISTRATION (Citizen/Business)
   ↓
   - Pay registration deposit (100 DALLA)
   - Provide title number, GPS coords, description
   - Specify property type, area, assessed value
   ↓
   PropertyRecord created (government_verified: false)

2. GOVERNMENT VERIFICATION (Land Registry Office)
   ↓
   - Verify title number against physical registry
   - Confirm ownership documentation
   - Cross-check with national land database
   ↓
   government_verified: true

3. SURVEYOR VERIFICATION (Licensed Surveyor)
   ↓
   - Confirm GPS coordinates
   - Verify property boundaries
   - Measure area (sqm)
   - Upload survey map to Pakit DAG
   ↓
   surveyed: true

4. ENVIRONMENTAL CLEARANCE (Optional for Developments)
   ↓
   - Environmental authority reviews development plans
   - Issues clearance for construction/modification
   ↓
   environmental_clearance: true

5. TRANSFER (Owner → New Owner)
   ↓
   - KYC Level 2 required (both parties)
   - Sanctions check (Oracle)
   - Transfer tax deducted (10% of assessed value → treasury)
   - Government re-verification required
   ↓
   Ownership updated, TransferRecord logged
```

### 3. Property Types (7 Categories)

| Type | Description | Common Use Cases |
|------|-------------|------------------|
| **Residential** | Single-family homes, apartments, condos | Citizen housing, expat residences |
| **Commercial** | Stores, offices, restaurants | Tourism businesses, retail shops |
| **Agricultural** | Farms, plantations, livestock ranches | Sugar cane, citrus, livestock |
| **Industrial** | Factories, warehouses, processing plants | Food processing, manufacturing |
| **Tourism** | Hotels, resorts, eco-lodges | Ambergris Caye, Placencia resorts |
| **Conservation** | Protected nature reserves, wildlife sanctuaries | Rainforest conservation, marine reserves |
| **Government** | Public land, infrastructure, military bases | Roads, schools, government buildings |

### 4. Zoning Types (12 Categories)

| Zoning | Purpose | Development Restrictions |
|--------|---------|-------------------------|
| **Residential** | Housing only | Max 3 stories (urban), 2 stories (rural) |
| **Commercial** | Business operations | Requires business license + environmental clearance |
| **Industrial** | Manufacturing, processing | Heavy environmental compliance |
| **Agricultural** | Farming, livestock | Minimum 5-acre lots (citrus), 10-acre (livestock) |
| **Tourism** | Hotels, resorts | Coastal setback rules (100m from high-tide mark) |
| **MixedUse** | Residential + Commercial | Ground floor commercial, upper floors residential |
| **Conservation** | Protected areas | NO development (preserve ecosystems) |
| **Urban** | City centers | High-density allowed, utilities required |
| **Suburban** | Low-density residential | Min 0.25-acre lots |
| **Rural** | Agricultural + low-density | Min 1-acre lots |
| **Coastal** | Beachfront, waterfront | Special coastal zone regulations |
| **Special** | Special economic zones | Government-defined rules (e.g., tech parks) |

### 5. Encumbrances (Liens & Restrictions)

Properties can have **up to 10 encumbrances** (on-chain):

| Encumbrance Type | Description | Example |
|------------------|-------------|---------|
| **Mortgage** | Loan secured by property | Bank of Belize mortgage (25-year term) |
| **Lien** | Legal claim for debt repayment | Contractor lien for unpaid work |
| **Easement** | Right to use part of property | Utility company power line easement |
| **Covenant** | Restrictive condition | "No commercial use" covenant |
| **Lease** | Long-term rental agreement | 99-year land lease |
| **Judgment** | Court-ordered claim | Debt repayment judgment |
| **Restriction** | Government limitation | "No subdivision" restriction |

**Encumbrance Structure**:
```rust
pub struct Encumbrance<AccountId> {
    pub encumbrance_type: EncumbranceType,  // Mortgage/Lien/etc.
    pub holder: AccountId,                  // Who holds the claim
    pub amount: Option<u128>,               // Amount owed (if financial)
    pub description: BoundedVec<u8, 256>,   // Details
    pub expires_at: Option<u64>,            // Expiration timestamp
    pub document_hash: Option<[u8; 32]>,    // Pakit DAG document CID
}
```

---

## Storage Items

### Property Records

| Storage | Type | Description |
|---------|------|-------------|
| `Properties` | `StorageMap<PropertyId, PropertyRecord>` | All registered properties (indexed by ID) |
| `PropertyOwners` | `StorageMap<AccountId, Vec<PropertyId>>` | Properties owned by account |
| `NextPropertyId` | `StorageValue<PropertyId>` | Auto-incrementing property ID counter |
| `PropertyCount` | `StorageValue<u32>` | Total number of registered properties |

### Transfer History

| Storage | Type | Description |
|---------|------|-------------|
| `TransferHistory` | `StorageDoubleMap<PropertyId, u32, TransferRecord>` | Transfer history per property (indexed by transfer #) |
| `TransferCount` | `StorageMap<PropertyId, u32>` | Number of transfers for each property |

### Authorized Personnel

| Storage | Type | Description |
|---------|------|-------------|
| `AuthorizedSurveyors` | `StorageMap<AccountId, bool>` | Licensed surveyors (can mark `surveyed: true`) |
| `RegistrationDeposits` | `StorageMap<PropertyId, (AccountId, Balance)>` | Reserved deposits for property registration |

---

## Extrinsics (Public Functions)

### Property Owner Operations

#### `register_property(origin, title_number, description, coordinates, area_sqm, property_type, assessed_value, zoning, document_hash)`
**Purpose**: Register new property on blockchain (initial L0 verification)

**Parameters**:
- `origin`: Signed origin (property owner)
- `title_number`: Physical title number (e.g., "BZ-LAND-2024-00123", max 64 bytes)
- `description`: Property description (max 256 bytes)
- `coordinates`: GPS coordinates `(latitude_i64, longitude_i64)` (scaled by 1e6)
- `area_sqm`: Property area in square meters (u32)
- `property_type`: `Residential`, `Commercial`, `Agricultural`, `Industrial`, `Tourism`, `Conservation`, `Government`
- `assessed_value`: Estimated value in bBZD (smallest unit, 12 decimals)
- `zoning`: Zoning classification (12 types)
- `document_hash`: Optional Pakit DAG CID for title deed scan

**Requirements**:
- Pays registration deposit (100 DALLA, refunded after government verification)
- Title number unique (not already registered)
- GPS coordinates valid (Belize territory: 15.8°N-18.5°N, 87.5°W-89.2°W)
- Area > 0 sqm

**Outcome**:
- Creates `PropertyRecord` with `government_verified: false`
- Reserves deposit from origin's balance
- Assigns unique `PropertyId` (auto-increment)
- Adds property to `PropertyOwners[origin]`
- Emits `PropertyRegistered(property_id, owner, title_number)`

**Example**:
```rust
// Register residential property in Belize City
let title_number = b"BZ-BELIZE-CITY-2024-00456".to_vec();
let description = b"2-bedroom house, King Street, Belize City".to_vec();
let coordinates = (17_500_000, -88_190_000); // 17.5°N, 88.19°W (scaled by 1e6)
let area_sqm = 250; // 250 sqm lot
let assessed_value = 150_000 * 10u128.pow(12); // 150K bBZD

LandLedger::register_property(
    Origin::signed(owner_account),
    title_number.try_into().unwrap(),
    description.try_into().unwrap(),
    coordinates,
    area_sqm,
    PropertyType::Residential,
    assessed_value,
    ZoningType::Residential,
    Some(pakit_dag_cid) // Title deed scan
)?;
// Result: PropertyRegistered event, property_id = 42
```

---

#### `transfer_property(origin, property_id, new_owner, transfer_type, sale_price)`
**Purpose**: Transfer property ownership (with KYC enforcement + transfer tax)

**Parameters**:
- `origin`: Signed origin (current property owner)
- `property_id`: Property ID to transfer
- `new_owner`: Recipient account
- `transfer_type`: `Sale`, `Gift`, `Inheritance`, `Foreclosure`, `Government`
- `sale_price`: Transaction price (if `Sale`, used for transfer tax calculation)

**Requirements**:
- Origin owns the property
- **KYC Level 2 required** (both origin and new_owner via Oracle pallet)
- Neither party is sanctioned (Oracle sanctions check)
- Property is government verified (`government_verified: true`)
- If `Sale`: transfer tax paid (10% of `sale_price` by default → treasury)
- No blocking encumbrances (e.g., unpaid mortgage)

**Transfer Tax Calculation**:
```rust
// 10% transfer tax (1000 basis points default, governance-adjustable)
let transfer_tax = (sale_price * T::TransferTaxRate::get() as u128) / 10_000;
// Deducted from new_owner's balance and sent to treasury
```

**Outcome**:
- Updates `owner` field in `PropertyRecord`
- Creates `TransferRecord` with timestamp, parties, price, tax paid
- Transfers deposit refund to origin (if not yet refunded)
- Sets `government_verified: false` (requires re-verification)
- Emits `PropertyTransferred(property_id, from, to, sale_price, transfer_tax)`

**Example**:
```rust
// Sell property for 200K bBZD
let sale_price = 200_000 * 10u128.pow(12);

LandLedger::transfer_property(
    Origin::signed(seller_account),
    42, // property_id
    buyer_account,
    TransferType::Sale,
    Some(sale_price)
)?;
// Result:
// - Transfer tax: 20,000 bBZD (10%) → treasury
// - Ownership updated: seller → buyer
// - PropertyTransferred event emitted
```

---

#### `add_encumbrance(origin, property_id, encumbrance_type, holder, amount, description, expires_at, document_hash)`
**Purpose**: Add lien, mortgage, easement, or restriction to property

**Parameters**:
- `origin`: Signed origin (property owner OR government for court-ordered liens)
- `property_id`: Property ID
- `encumbrance_type`: `Mortgage`, `Lien`, `Easement`, `Covenant`, `Lease`, `Judgment`, `Restriction`
- `holder`: Account holding the claim (e.g., bank for mortgage)
- `amount`: Optional financial amount (e.g., mortgage principal)
- `description`: Details (max 256 bytes)
- `expires_at`: Optional expiration timestamp (Unix timestamp)
- `document_hash`: Optional Pakit DAG CID for legal document

**Requirements**:
- Origin owns property OR is government (for court judgments)
- Maximum 10 encumbrances per property

**Outcome**:
- Adds `Encumbrance` to `PropertyRecord.encumbrances` vector
- Emits `EncumbranceAdded(property_id, encumbrance_type, holder)`

**Example**:
```rust
// Add bank mortgage (25-year term)
let expires_at = current_timestamp + (25 * 365 * 24 * 60 * 60); // 25 years

LandLedger::add_encumbrance(
    Origin::signed(owner_account),
    42, // property_id
    EncumbranceType::Mortgage,
    bank_account,
    Some(100_000 * 10u128.pow(12)), // 100K bBZD mortgage
    b"Bank of Belize 25-year mortgage".to_vec().try_into().unwrap(),
    Some(expires_at),
    Some(pakit_dag_cid) // Mortgage agreement document
)?;
// Result: EncumbranceAdded event, property now has mortgage lien
```

---

#### `remove_encumbrance(origin, property_id, encumbrance_index)`
**Purpose**: Clear paid-off mortgage, expired lien, or canceled restriction

**Parameters**:
- `origin`: Signed origin (property owner OR encumbrance holder OR government)
- `property_id`: Property ID
- `encumbrance_index`: Index in `encumbrances` vector (0-9)

**Requirements**:
- Origin is owner, encumbrance holder, or government authority
- Encumbrance exists at index

**Outcome**:
- Removes encumbrance from vector
- Emits `EncumbranceRemoved(property_id, encumbrance_type)`

**Example**:
```rust
// Bank releases mortgage after full payment
LandLedger::remove_encumbrance(
    Origin::signed(bank_account),
    42, // property_id
    0   // First encumbrance (mortgage)
)?;
// Result: Mortgage removed, property clear
```

---

#### `update_property_value(origin, property_id, new_assessed_value)`
**Purpose**: Update assessed value for property tax/insurance purposes

**Parameters**:
- `origin`: Signed origin (property owner OR government appraiser)
- `property_id`: Property ID
- `new_assessed_value`: New valuation in bBZD

**Requirements**:
- Origin owns property OR is government authority

**Outcome**:
- Updates `assessed_value` in `PropertyRecord`
- Emits `PropertyValueUpdated(property_id, old_value, new_value)`

---

#### `mark_tourism_property(origin, property_id, is_tourism)`
**Purpose**: Flag property for tourism investment incentives

**Parameters**:
- `origin`: Signed origin (property owner)
- `property_id`: Property ID
- `is_tourism`: `true` to mark as tourism property, `false` to unmark

**Requirements**:
- Origin owns property
- Property type is `Tourism` or `Commercial`

**Outcome**:
- Sets `is_tourism_property` flag
- Emits `TourismPropertyMarked(property_id, is_tourism)`

**Use Case**: Tourism properties qualify for special tax incentives, faster permitting

---

### Government Operations

#### `verify_property(origin, property_id, verified)`
**Purpose**: Government verification of property registration (Land Registry Office)

**Parameters**:
- `origin`: `T::GovernanceOrigin` (Land Registry Office authority)
- `property_id`: Property ID
- `verified`: `true` to verify, `false` to revoke verification

**Requirements**:
- Government origin authority
- Property exists

**Outcome**:
- Sets `government_verified` flag
- If verified, refunds registration deposit to owner
- Emits `PropertyVerified(property_id, verified)`

**Example**:
```rust
// Land Registry Office verifies property after cross-checking title
LandLedger::verify_property(
    Origin::root(),
    42, // property_id
    true
)?;
// Result:
// - government_verified: true
// - 100 DALLA deposit refunded to owner
// - PropertyVerified event
```

---

#### `survey_property(origin, property_id, surveyed, survey_document_hash)`
**Purpose**: Licensed surveyor confirms GPS coordinates and boundaries

**Parameters**:
- `origin`: `T::SurveyorOrigin` (authorized surveyor account)
- `property_id`: Property ID
- `surveyed`: `true` to mark surveyed, `false` to revoke
- `survey_document_hash`: Pakit DAG CID for survey map/report

**Requirements**:
- Origin is authorized surveyor (in `AuthorizedSurveyors`)
- Property exists

**Outcome**:
- Sets `surveyed` flag
- Stores survey document hash (if provided)
- Emits `PropertySurveyed(property_id, surveyor, survey_hash)`

---

#### `grant_environmental_clearance(origin, property_id, cleared)`
**Purpose**: Environmental authority approves development/construction

**Parameters**:
- `origin`: `T::EnvironmentalOrigin` (Department of Environment)
- `property_id`: Property ID
- `cleared`: `true` to grant clearance, `false` to revoke

**Requirements**:
- Environmental origin authority
- Property exists

**Outcome**:
- Sets `environmental_clearance` flag
- Emits `EnvironmentalClearanceGranted(property_id, cleared)`

**Use Case**: Required for new construction, land clearing, coastal developments

---

#### `register_surveyor(origin, surveyor)`
**Purpose**: Authorize new licensed surveyor

**Parameters**:
- `origin`: `T::GovernanceOrigin` (government authority)
- `surveyor`: Account to authorize

**Outcome**:
- Adds to `AuthorizedSurveyors`
- Emits `SurveyorRegistered(surveyor)`

---

#### `revoke_surveyor(origin, surveyor)`
**Purpose**: Revoke surveyor license (fraud, negligence)

**Parameters**:
- `origin`: `T::GovernanceOrigin`
- `surveyor`: Surveyor to revoke

**Outcome**:
- Removes from `AuthorizedSurveyors`
- Emits `SurveyorRevoked(surveyor)`

---

#### `update_zoning(origin, property_id, new_zoning)`
**Purpose**: Change property zoning classification (rezoning)

**Parameters**:
- `origin`: `T::GovernanceOrigin` (city/town council)
- `property_id`: Property ID
- `new_zoning`: New zoning type (12 options)

**Requirements**:
- Government authority
- Public consultation completed (off-chain)

**Outcome**:
- Updates `zoning` in `PropertyRecord`
- Emits `ZoningUpdated(property_id, old_zoning, new_zoning)`

**Example**:
```rust
// Rezone agricultural land to residential (subdivision development)
LandLedger::update_zoning(
    Origin::root(),
    42,
    ZoningType::Residential
)?;
// Result: Zoning changed from Agricultural → Residential
```

---

## Helper Functions (View/Query)

### `get_owned_properties(owner) -> Vec<PropertyId>`
**Purpose**: Get all properties owned by account

**Returns**: Vector of property IDs

**Example**:
```rust
let properties = LandLedger::get_owned_properties(&owner_account);
// Returns: [42, 58, 103] (3 properties owned)
```

---

### `get_property_details(property_id) -> Option<PropertyRecord>`
**Purpose**: Get full property record

**Returns**: `PropertyRecord` or `None` if not found

---

### `get_transfer_history(property_id) -> Vec<TransferRecord>`
**Purpose**: Get complete transfer history for property

**Returns**: Vector of `TransferRecord` (chronological order)

**Example**:
```rust
let history = LandLedger::get_transfer_history(42);
// Returns:
// [
//   TransferRecord { from: Alice, to: Bob, price: 100K, timestamp: 2024-01-15 },
//   TransferRecord { from: Bob, to: Charlie, price: 150K, timestamp: 2025-06-20 },
//   TransferRecord { from: Charlie, to: David, price: 200K, timestamp: 2026-01-29 }
// ]
```

---

### `is_property_verified(property_id) -> bool`
**Purpose**: Check if property is government verified

**Returns**: `true` if verified, `false` otherwise

---

### `has_encumbrance(property_id, encumbrance_type) -> bool`
**Purpose**: Check if property has specific encumbrance type

**Returns**: `true` if encumbrance exists

**Example**:
```rust
let has_mortgage = LandLedger::has_encumbrance(42, EncumbranceType::Mortgage);
// Returns: true (property has mortgage)
```

---

### `get_total_properties() -> u32`
**Purpose**: Get total number of registered properties

**Returns**: Property count

---

### `get_properties_by_type(property_type) -> Vec<PropertyId>`
**Purpose**: Find all properties of specific type

**Returns**: Vector of property IDs matching type

---

## Events

| Event | When Emitted |
|-------|-------------|
| `PropertyRegistered(PropertyId, AccountId, Vec<u8>)` | New property registered (L0 verification) |
| `PropertyVerified(PropertyId, bool)` | Government verification status changed |
| `PropertySurveyed(PropertyId, AccountId, Option<[u8; 32]>)` | Surveyor confirms boundaries |
| `PropertyTransferred(PropertyId, AccountId, AccountId, u128, u128)` | Ownership transfer (from, to, price, tax) |
| `EncumbranceAdded(PropertyId, EncumbranceType, AccountId)` | Lien/mortgage/restriction added |
| `EncumbranceRemoved(PropertyId, EncumbranceType)` | Encumbrance cleared |
| `EnvironmentalClearanceGranted(PropertyId, bool)` | Environmental approval granted/revoked |
| `ZoningUpdated(PropertyId, ZoningType, ZoningType)` | Rezoning (old → new) |
| `PropertyValueUpdated(PropertyId, u128, u128)` | Assessed value changed |
| `TourismPropertyMarked(PropertyId, bool)` | Tourism property flag toggled |
| `SurveyorRegistered(AccountId)` | New surveyor authorized |
| `SurveyorRevoked(AccountId)` | Surveyor license revoked |

---

## Errors

| Error | Cause |
|-------|-------|
| `PropertyNotFound` | Property ID does not exist |
| `NotPropertyOwner` | Origin does not own property |
| `PropertyAlreadyRegistered` | Title number already exists |
| `InsufficientDeposit` | Registration deposit too low |
| `InvalidCoordinates` | GPS coordinates outside Belize territory |
| `InvalidArea` | Property area is zero or unrealistic |
| `NotGovernmentVerified` | Property not verified for transfer |
| `InsufficientKyc` | Buyer/seller lacks KYC Level 2 |
| `AccountSanctioned` | Party is on OFAC/UN sanctions list |
| `TransferTaxUnpaid` | Transfer tax payment failed |
| `MaxEncumbrancesReached` | Property has 10 encumbrances (max) |
| `EncumbranceNotFound` | Encumbrance index out of bounds |
| `UnauthorizedSurveyor` | Origin is not authorized surveyor |
| `BlockingEncumbrance` | Cannot transfer due to active lien |
| `InvalidZoning` | Zoning change violates regulations |

---

## Oracle Integration

**Exported Trait**: `LandLedgerOracleProvider<AccountId>`

```rust
pub trait LandLedgerOracleProvider<AccountId> {
    /// Verify property ownership via external land registry (u32 internal ID)
    fn verify_land_owner(property_id: u32, account: &AccountId) -> bool;
    
    /// Get KYC level for property transfers (Level 2 required)
    fn get_kyc_level(account: &AccountId) -> Option<u8>;
    
    /// Check if account is sanctioned (OFAC/UN compliance)
    fn is_sanctioned(account: &AccountId) -> bool;
}
```

**Implementation** (runtime/src/lib.rs):
```rust
pub struct LandLedgerOracleProvider;
impl pallet_belize_landledger::LandLedgerOracleProvider<AccountId> 
    for LandLedgerOracleProvider 
{
    fn verify_land_owner(property_id: u32, account: &AccountId) -> bool {
        Oracle::verify_land_owner(property_id, account)
    }
    
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(account)
    }
    
    fn is_sanctioned(account: &AccountId) -> bool {
        Identity::is_account_sanctioned(account)
    }
}
```

**Used By**: LandLedger pallet for KYC enforcement and sanctions compliance during transfers

---

## Configuration Parameters

### `Config` Trait

```rust
pub trait Config: frame_system::Config {
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    type GovernmentOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    type SurveyorOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    type EnvironmentalOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    type Oracle: LandLedgerOracleProvider<Self::AccountId>;
    
    #[pallet::constant]
    type RegistrationDeposit: Get<BalanceOf<Self>>;  // 100 DALLA
    
    #[pallet::constant]
    type TransferTaxRate: Get<u32>;                  // 1000 bps (10%)
    
    #[pallet::constant]
    type MaxDescriptionLength: Get<u32>;             // 256 bytes
    
    type WeightInfo: WeightInfo;
}
```

**Recommended Values**:
- `RegistrationDeposit`: `100 * 10^12` (100 DALLA, refundable after verification)
- `TransferTaxRate`: `1000` (10% of sale price → treasury, stamp duty equivalent)
- `MaxDescriptionLength`: `256` (UTF-8 property description)

---

## Typical Workflows

### 1. Citizen Registers Property

```rust
// Step 1: Citizen registers property (pays deposit)
let title_number = b"BZ-CAYO-2026-00789".to_vec();
let description = b"5-acre citrus farm, San Ignacio, Cayo District".to_vec();
let coordinates = (17_160_000, -89_070_000); // San Ignacio (17.16°N, 89.07°W)
let area_sqm = 20_235; // 5 acres = ~20,235 sqm

LandLedger::register_property(
    Origin::signed(farmer_account),
    title_number.try_into().unwrap(),
    description.try_into().unwrap(),
    coordinates,
    area_sqm,
    PropertyType::Agricultural,
    50_000 * 10u128.pow(12), // Assessed value: 50K bBZD
    ZoningType::Agricultural,
    Some(pakit_dag_title_deed_cid)
)?;
// Result: PropertyRegistered event, property_id = 100, 100 DALLA deposit reserved

// Step 2: Land Registry Office verifies (off-chain title check)
LandLedger::verify_property(
    Origin::root(),
    100,
    true
)?;
// Result: government_verified: true, 100 DALLA refunded to farmer

// Step 3: Licensed surveyor confirms boundaries
LandLedger::survey_property(
    Origin::signed(surveyor_account),
    100,
    true,
    Some(pakit_dag_survey_map_cid)
)?;
// Result: surveyed: true, survey map stored

// Property now fully verified (government + surveyor)
```

---

### 2. Property Sale with Transfer Tax

```rust
// Scenario: Farmer sells 5-acre citrus farm for 75K bBZD

// Step 1: Seller initiates transfer
let sale_price = 75_000 * 10u128.pow(12);

LandLedger::transfer_property(
    Origin::signed(seller_account),
    100, // property_id
    buyer_account,
    TransferType::Sale,
    Some(sale_price)
)?;
// Result:
// - Transfer tax: 7,500 bBZD (10%) deducted from buyer
// - 7,500 bBZD → treasury
// - Ownership: seller → buyer
// - government_verified: false (requires re-verification)

// Step 2: Government re-verifies new owner
LandLedger::verify_property(
    Origin::root(),
    100,
    true
)?;
// Result: government_verified: true (buyer now verified owner)
```

---

### 3. Adding Mortgage Encumbrance

```rust
// Scenario: Buyer takes 50K bBZD mortgage from bank

let mortgage_expires = current_timestamp + (20 * 365 * 24 * 60 * 60); // 20 years

LandLedger::add_encumbrance(
    Origin::signed(buyer_account),
    100, // property_id
    EncumbranceType::Mortgage,
    bank_account,
    Some(50_000 * 10u128.pow(12)), // 50K bBZD loan
    b"Belize Bank 20-year agricultural mortgage".to_vec().try_into().unwrap(),
    Some(mortgage_expires),
    Some(pakit_dag_mortgage_agreement_cid)
)?;
// Result: Mortgage added, property encumbered

// Years later: Buyer pays off mortgage
LandLedger::remove_encumbrance(
    Origin::signed(bank_account), // Bank releases lien
    100,
    0 // First encumbrance (mortgage index)
)?;
// Result: Property clear, no encumbrances
```

---

### 4. Tourism Property Development

```rust
// Scenario: Investor develops eco-lodge on coastal property

// Step 1: Register property (coastal zoning)
LandLedger::register_property(
    Origin::signed(investor_account),
    b"BZ-PLACENCIA-2026-00555".to_vec().try_into().unwrap(),
    b"2-acre beachfront lot, Placencia Peninsula".to_vec().try_into().unwrap(),
    (16_513_000, -88_366_000), // Placencia (16.513°N, 88.366°W)
    8_094, // 2 acres
    PropertyType::Tourism,
    500_000 * 10u128.pow(12), // 500K bBZD (beachfront premium)
    ZoningType::Coastal,
    Some(pakit_dag_cid)
)?;
// Result: property_id = 200

// Step 2: Mark as tourism property (tax incentives)
LandLedger::mark_tourism_property(
    Origin::signed(investor_account),
    200,
    true
)?;
// Result: is_tourism_property: true, qualifies for incentives

// Step 3: Apply for environmental clearance (coastal development)
LandLedger::grant_environmental_clearance(
    Origin::root(), // Department of Environment
    200,
    true
)?;
// Result: environmental_clearance: true, can proceed with construction
```

---

## Security Considerations

### 1. **KYC Enforcement**
- ✅ **Level 2 Required**: All property transfers require KYC L2 (SSN + Passport via Identity pallet)
- ✅ **Sanctions Compliance**: OFAC/UN checks via Oracle pallet (blocks sanctioned entities)
- ✅ **Government Verification**: Prevents fraudulent property claims

### 2. **Transfer Tax Collection**
- ✅ **Automatic Deduction**: 10% transfer tax deducted on-chain (no manual collection)
- ✅ **Treasury Integration**: All tax revenue sent to national treasury
- ✅ **Governance Control**: Transfer tax rate adjustable via governance proposal

### 3. **Encumbrance Management**
- ✅ **Max 10 Encumbrances**: Prevents storage bloat
- ✅ **Blocking Logic**: Cannot transfer property with unpaid mortgages (unless foreclosure)
- ✅ **Document Storage**: All encumbrance docs stored in Pakit DAG (immutable proof)

### 4. **Surveyor Authorization**
- ✅ **Licensed Surveyors Only**: Government-approved surveyors (revocable)
- ✅ **Fraud Detection**: Suspicious survey patterns flagged by governance
- ✅ **Survey Document Hashing**: GPS coordinates verified against survey maps (Pakit DAG)

### 5. **GPS Validation**
- ✅ **Belize Territory Check**: Coordinates must be within Belize boundaries (15.8°N-18.5°N, 87.5°W-89.2°W)
- ⚠️ **Precision**: i64 coordinates scaled by 1e6 (6 decimal places = ~11cm accuracy)
- ⚠️ **Boundary Disputes**: Off-chain resolution required (court orders → on-chain updates)

---

## Testing

### Unit Tests (src/tests.rs)

**Coverage**:
- Property registration (all property types)
- Government verification workflow
- Surveyor authorization and verification
- Property transfers (sale, gift, inheritance)
- Transfer tax calculation (10% default)
- KYC Level 2 enforcement
- Sanctions checking (Oracle integration)
- Encumbrance management (add/remove mortgages, liens)
- Zoning updates (rezoning)
- Tourism property flagging
- Environmental clearance workflow
- GPS coordinate validation (Belize territory)
- Transfer history tracking

**Run Tests**:
```bash
cargo test -p pallet-belize-landledger
```

---

## Integration with BelizeChain

### Runtime Configuration (belizechain/runtime/src/lib.rs)

```rust
impl pallet_belize_landledger::Config for Runtime {
    type Currency = Balances;
    type GovernmentOrigin = EnsureRoot<AccountId>;
    type SurveyorOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_belize_landledger::EnsureSurveyor<Runtime>
    >;
    type EnvironmentalOrigin = EnsureRoot<AccountId>;
    type Oracle = LandLedgerOracleProvider;
    type RegistrationDeposit = ConstU128<100_000_000_000_000>; // 100 DALLA
    type TransferTaxRate = ConstU32<1000>;                      // 10% (1000 bps)
    type MaxDescriptionLength = ConstU32<256>;
    type WeightInfo = ();
}

// Oracle provider for LandLedger
pub struct LandLedgerOracleProvider;
impl pallet_belize_landledger::LandLedgerOracleProvider<AccountId> 
    for LandLedgerOracleProvider 
{
    fn verify_land_owner(property_id: u32, account: &AccountId) -> bool {
        Oracle::verify_land_owner(property_id, account)
    }
    
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(account)
    }
    
    fn is_sanctioned(account: &AccountId) -> bool {
        Identity::is_account_sanctioned(account)
    }
}
```

---

## Future Enhancements

1. **Fractional Ownership**: Multiple owners per property (e.g., timeshares, co-ownership)
2. **Property Auctions**: On-chain auction mechanism for foreclosures, government sales
3. **Automated Appraisals**: Oracle-based property valuation (comparable sales, market data)
4. **Rental Registry**: Track long-term leases, tenant rights
5. **Development Permits**: On-chain building permit issuance and tracking
6. **Property Insurance**: Integration with insurance pallets (flood, hurricane coverage)
7. **3D Property Boundaries**: Store 3D coordinates for multi-story buildings, underground rights
8. **Cross-Border Land**: Support for shared border properties (Belize-Guatemala, Belize-Mexico)

---

## References

- **Belize Land Registry**: [Lands and Surveys Department](https://www.lands.gov.bz/)
- **Environmental Compliance**: [Department of the Environment](https://www.doe.gov.bz/)
- **Tourism Incentives**: [Belize Tourism Board](https://www.travelbelize.org/)
- **Polkadot SDK Documentation**: [Substrate Pallet Development](https://docs.substrate.io/)
- **Pakit DAG Storage**: `pakit/README.md` (title deeds, survey maps, encumbrance documents)
- **Oracle Pallet**: `belizechain/pallets/oracle/README.md` (land registry data, KYC, sanctions)
- **Identity Pallet**: `belizechain/pallets/identity/README.md` (KYC verification)

---

## Contact & Support

For questions regarding BelizeChain Land Ledger implementation or property registry policy:
- **Technical**: BelizeChain Core Developer Team
- **Policy**: Lands and Surveys Department, Ministry of Natural Resources
- **Legal**: Attorney General's Chambers (property law, land disputes)

**Audit Status**: ✅ Complete (January 2026)  
**Clippy Warnings**: 0  
**Test Coverage**: Comprehensive unit tests  
**Properties Registered**: TBD (mainnet launch)  
**Integration**: Oracle + Identity + Pakit DAG
