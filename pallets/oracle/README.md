# BelizeChain Oracle: External Data Integration Layer

## Overview

The **Oracle** pallet (`pallet-belize-oracle`) serves as BelizeChain's trusted external data integration layer, providing **merchant verification**, **sanctions compliance**, **KYC data bridging**, **land registry verification**, **IoT device management**, and **fallback exchange rate management** (NOTE: bBZD peg is 1:1 BZD, NOT oracle-based). This pallet acts as the **single source of truth** for all external data consumed by other pallets.

**Key Features**:
- **Merchant Verification**: Tourism merchant categorization (5-8% DALLA spending rewards)
- **Sanctions Compliance**: OFAC/UN sanctions list integration (FSC oversight)
- **KYC Data Bridge**: External KYC verification (supplements Identity pallet)
- **Land Registry Integration**: Property ownership verification (external database cross-check)
- **IoT Device Management**: Register, verify, and consume IoT sensor data (agriculture, environment)
- **Exchange Rate Fallback**: Manual admin rates for DEX quoting (bBZD peg is ALWAYS 1:1 BZD, not oracle-dependent)
- **Multi-Operator Consensus**: 3+ oracle operators required for price feed consensus
- **Data Staleness Prevention**: Configurable max age for oracle data (50,000 blocks default)
- **Operator Reputation**: Performance tracking and reward distribution

**CRITICAL**: The Oracle pallet does **NOT** determine the bBZD peg rate. The bBZD stablecoin is **always 1:1 BZD** (backed by Central Bank reserves). Oracle exchange rates are ONLY used for:
- DEX quoting (DALLA/bBZD liquidity pool pricing)
- Cross-border remittance fee estimates
- Tourism cashback calculations (DALLA → bBZD conversion)

---

## Architecture

### 1. Oracle Operator Model

```
┌──────────────────────────────────────────────────────────┐
│         Oracle Operator Network (3+ operators)           │
└──────────────────────────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
  ┌──────────┐     ┌──────────┐     ┌──────────┐
  │ Operator │     │ Operator │     │ Operator │
  │    #1    │     │    #2    │     │    #3    │
  └──────────┘     └──────────┘     └──────────┘
        │                │                │
        └────────────────┼────────────────┘
                         ▼
           ┌──────────────────────────┐
           │  Price Feed Aggregation  │
           │  (Median of submissions) │
           └──────────────────────────┘
                         │
                         ▼
           ┌──────────────────────────┐
           │   PriceFeeds Storage     │
           │  (Consensus price only)  │
           └──────────────────────────┘
```

**Consensus Requirements**:
- **Minimum Operators**: 3 (configurable via `MinConsensusOperators`)
- **Aggregation Method**: Median price (prevents outlier manipulation)
- **Staleness Check**: Data older than 50,000 blocks (configurable) rejected
- **Reputation**: Operators earn rewards for accurate, timely submissions

### 2. Data Flow (Merchant Verification Example)

```
1. OFF-CHAIN: Business applies for tourism merchant status
   ↓
2. VERIFICATION: Belize Tourism Board reviews application
   ↓
3. ON-CHAIN: Oracle operator submits merchant verification
   ┌──────────────────────────────────────────────────┐
   │  verify_merchant(merchant_account, category)     │
   │  - MerchantCategory::Restaurant (6% rewards)     │
   │  - Certification: BTB-CERT-2024-00456            │
   └──────────────────────────────────────────────────┘
   ↓
4. STORAGE: MerchantCategories[merchant] = MerchantInfo
   ↓
5. CONSUMPTION: Economy pallet queries merchant category
   ┌──────────────────────────────────────────────────┐
   │  Oracle::get_merchant_category(&merchant)        │
   │  Returns: Some(MerchantCategory::Restaurant)     │
   └──────────────────────────────────────────────────┘
   ↓
6. REWARD: Customer pays 100 DALLA at restaurant
   - 6 DALLA cashback (6% reward for restaurants)
   - Redeemable for bBZD at 1:1 BZD peg
```

### 3. Sanctions Compliance Workflow

```
┌──────────────────────────────────────────────────────────┐
│  External Sanctions Lists (OFAC, UN, EU, FSC)            │
│  - Updated daily by Oracle operators                     │
│  - Cross-checked against national database               │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
           ┌──────────────────────────┐
           │  Oracle::add_sanctioned_ │
           │  entity(account, reason) │
           └──────────────────────────┘
                         │
                         ▼
           ┌──────────────────────────┐
           │  SanctionedEntities[]    │
           │  Storage (on-chain)      │
           └──────────────────────────┘
                         │
                         ▼
   ┌─────────────────────────────────────────────┐
   │  ALL PALLETS: Check sanctions before ops    │
   │  - Economy: Block bBZD minting              │
   │  - LandLedger: Block property transfers     │
   │  - Interoperability: Block bridge ops       │
   │  - Staking: Block validator registration    │
   └─────────────────────────────────────────────┘
```

---

## Storage Items

### Oracle Operator Management

| Storage | Type | Description |
|---------|------|-------------|
| `OracleOperators` | `StorageMap<AccountId, bool>` | Authorized oracle operators (admin-approved) |
| `OperatorStats` | `StorageMap<AccountId, OracleOperatorStats>` | Performance tracking (submissions, accuracy, rewards) |

**OracleOperatorStats Structure**:
```rust
pub struct OracleOperatorStats<BlockNumber> {
    pub total_submissions: u64,        // Total data submissions
    pub accurate_submissions: u64,     // Submissions within consensus range
    pub last_submission: BlockNumber,  // Most recent activity
    pub total_rewards: u128,           // Lifetime rewards earned (DALLA)
    pub reputation_score: u32,         // 0-100 reputation
}
```

### Price Feed Data (Fallback for DEX Quoting ONLY)

**IMPORTANT**: Oracle price feeds are **NOT** used for bBZD peg (always 1:1 BZD). Only for:
- DEX liquidity pool pricing (DALLA/bBZD pairs)
- Cross-border fee estimates
- Tourism cashback conversions

| Storage | Type | Description |
|---------|------|-------------|
| `PriceSubmissions` | `StorageDoubleMap<CurrencyPair, AccountId, (u128, BlockNumber)>` | Individual operator price submissions |
| `PriceFeeds` | `StorageMap<CurrencyPair, PriceFeedData>` | Aggregated consensus prices (median) |
| `ManualExchangeRates` | `StorageMap<CurrencyPair, (u128, BlockNumber)>` | Admin-set fallback rates (6 decimal precision, 1e6 scale) |

**CurrencyPair Structure**:
```rust
pub struct CurrencyPair {
    pub base: Currency,   // BZD, USD, EUR, CAD, MXN
    pub quote: Currency,
}
```

**PriceFeedData Structure**:
```rust
pub struct PriceFeedData<BlockNumber> {
    pub pair: CurrencyPair,
    pub price: u128,                   // Scaled by 1e6 (6 decimals)
    pub num_submissions: u32,          // Number of operators contributing
    pub last_update: BlockNumber,
    pub confidence: u8,                // 0-100 (based on submission variance)
}
```

### Merchant Verification (Tourism Incentives)

| Storage | Type | Description |
|---------|------|-------------|
| `MerchantCategories` | `StorageMap<AccountId, MerchantInfo>` | Verified tourism merchants (6 categories) |

**MerchantInfo Structure**:
```rust
pub struct MerchantInfo<AccountId, BlockNumber> {
    pub category: MerchantCategory,       // Restaurant, Hotel, Tour, etc.
    pub certified_by: AccountId,          // Oracle operator who verified
    pub certification_number: BoundedVec<u8, 128>, // BTB cert (e.g., "BTB-CERT-2024-00456")
    pub verified_at: BlockNumber,
    pub reward_rate: u8,                  // 5-8% DALLA cashback
    pub expires_at: Option<BlockNumber>,  // Certification expiry (annual renewal)
}
```

**MerchantCategory Enum** (6 types):
- **Restaurant**: 6% DALLA cashback
- **Hotel**: 7% DALLA cashback
- **TourOperator**: 8% DALLA cashback
- **Transportation**: 5% DALLA cashback
- **Retail**: 5% DALLA cashback
- **Entertainment**: 6% DALLA cashback

### Sanctions Compliance

| Storage | Type | Description |
|---------|------|-------------|
| `SanctionedEntities` | `StorageMap<AccountId, SanctionInfo>` | OFAC/UN sanctioned accounts |

**SanctionInfo Structure**:
```rust
pub struct SanctionInfo<BlockNumber> {
    pub reason: BoundedVec<u8, 256>,      // "OFAC SDN List", "UN 1267 Committee", etc.
    pub sanctioned_at: BlockNumber,
    pub sanctioned_by: AccountId,         // Oracle operator who added
    pub expires_at: Option<BlockNumber>,  // Expiration (if temporary)
    pub level: SanctionLevel,             // Warning, Restricted, Blocked
}
```

**SanctionLevel Enum**:
- **Warning**: Monitored, no restrictions yet
- **Restricted**: Limited operations (max 1000 bBZD/day)
- **Blocked**: All financial operations prohibited

### KYC Data Bridge (Supplements Identity Pallet)

| Storage | Type | Description |
|---------|------|-------------|
| `IdentityVerifications` | `StorageMap<AccountId, IdentityInfo>` | External KYC verification data |

**IdentityInfo Structure**:
```rust
pub struct IdentityInfo<AccountId, BlockNumber> {
    pub kyc_level: KycLevel,              // L0, L1, L2, L3 (matches Identity pallet)
    pub verified_by: AccountId,           // Oracle operator
    pub verification_date: BlockNumber,
    pub transaction_limit: u128,          // Daily limit (bBZD)
    pub daily_limit: u128,                // Per-transaction limit (bBZD)
    pub expires_at: BlockNumber,          // Annual renewal
}
```

**KycLevel Enum** (matches Identity pallet):
- **L0**: No verification (read-only)
- **L1**: SSN verified (basic financial ops)
- **L2**: SSN + Passport (full financial ops)
- **L3**: Biometric + all (validators, bridge operators)

### Land Registry Integration

| Storage | Type | Description |
|---------|------|-------------|
| `LandRegistryData` | `StorageMap<PropertyId, LandOwnershipInfo>` | External land registry cross-check |

**LandOwnershipInfo Structure**:
```rust
pub struct LandOwnershipInfo<AccountId, BlockNumber> {
    pub property_id: PropertyId,          // u32 internal ID
    pub owner: AccountId,                 // Current owner (cross-checked)
    pub title_number: BoundedVec<u8, 64>, // Physical title (e.g., "BZ-BELIZE-CITY-2024-00123")
    pub verified_at: BlockNumber,
    pub verified_by: AccountId,           // Oracle operator
    pub last_transfer: Option<BlockNumber>, // Last ownership change
}
```

### IoT Device Management (Agriculture, Environment)

| Storage | Type | Description |
|---------|------|-------------|
| `IoTDevices` | `StorageMap<[u8; 32], IoTDevice>` | Registered IoT devices (sensors, weather stations) |
| `IoTDataSubmissions` | `StorageDoubleMap<[u8; 32], BlockNumber, IoTDataPoint>` | Time-series sensor data |

**IoTDevice Structure**:
```rust
pub struct IoTDevice<AccountId, BlockNumber> {
    pub device_id: [u8; 32],              // Unique device hash
    pub owner: AccountId,                 // Device owner
    pub device_type: IoTDeviceType,       // WeatherStation, SoilSensor, etc.
    pub location: (i64, i64),             // GPS coordinates
    pub verified: bool,                   // Oracle operator verified
    pub registered_at: BlockNumber,
    pub last_data_submission: Option<BlockNumber>,
}
```

**IoTDeviceType Enum** (7 types):
- **WeatherStation**: Temperature, humidity, rainfall
- **SoilSensor**: Soil moisture, pH, nutrients
- **WaterQuality**: River/ocean water quality monitoring
- **AirQuality**: PM2.5, CO2, pollutant monitoring
- **CropMonitor**: Plant health, growth rate
- **LivestockTracker**: Animal health, location
- **EnergyMeter**: Solar/wind energy production

**IoTDataPoint Structure**:
```rust
pub struct IoTDataPoint<BlockNumber> {
    pub device_id: [u8; 32],
    pub timestamp: BlockNumber,
    pub data_type: BoundedVec<u8, 32>,    // "temperature", "humidity", "soil_moisture"
    pub value: i128,                      // Sensor reading (scaled appropriately)
    pub unit: BoundedVec<u8, 16>,         // "celsius", "percent", "ppm"
}
```

---

## Extrinsics (Public Functions)

### Operator Management (Admin Only)

#### `add_operator(origin, operator)`
**Purpose**: Authorize new oracle operator

**Parameters**:
- `origin`: `T::OracleAdminOrigin` (root or council)
- `operator`: Account to authorize

**Requirements**:
- Admin origin authority
- Operator not already authorized

**Outcome**:
- Adds to `OracleOperators`
- Initializes `OperatorStats` (zero values)
- Emits `OperatorAdded(operator)`

**Example**:
```rust
// Government authorizes new oracle operator
Oracle::add_operator(
    Origin::root(),
    operator_account
)?;
// Result: Operator can now submit data
```

---

#### `remove_operator(origin, operator)`
**Purpose**: Revoke oracle operator authorization (fraud, inactivity)

**Parameters**:
- `origin`: `T::OracleAdminOrigin`
- `operator`: Operator to revoke

**Outcome**:
- Removes from `OracleOperators`
- Operator stats preserved (for historical audit)
- Emits `OperatorRemoved(operator)`

---

### Price Feed Operations (Fallback for DEX Quoting ONLY)

#### `submit_price(origin, pair, price)`
**Purpose**: Oracle operator submits exchange rate (NOT for bBZD peg)

**Parameters**:
- `origin`: Signed origin (must be authorized operator)
- `pair`: `CurrencyPair` (e.g., DALLA/USD)
- `price`: Exchange rate (scaled by 1e6, 6 decimal precision)

**Requirements**:
- Origin is authorized operator
- Price is reasonable (not 100x deviation from last price)

**Process**:
1. Store in `PriceSubmissions[pair][operator]`
2. If 3+ operators submitted: Calculate median price
3. Update `PriceFeeds[pair]` with consensus price
4. Increment operator stats (reputation)

**Outcome**:
- Price stored in `PriceSubmissions`
- If consensus reached: `PriceFeeds` updated
- Emits `PriceSubmitted(operator, pair, price)`

**Example**:
```rust
// Operator submits DALLA/USD rate
let pair = CurrencyPair::new(Currency::DALLA, Currency::USD);
let price = 500_000; // 0.50 USD per DALLA (1e6 scale)

Oracle::submit_price(
    Origin::signed(operator_account),
    pair,
    price
)?;
// Result: Price submitted, consensus calculated if 3+ operators
```

---

#### `update_exchange_rate(origin, pair, rate)`
**Purpose**: Admin manually sets exchange rate (fallback for DEX)

**Parameters**:
- `origin`: `T::OracleAdminOrigin`
- `pair`: `CurrencyPair`
- `rate`: Exchange rate (scaled by 1e6)

**Requirements**:
- Admin origin authority

**Outcome**:
- Updates `ManualExchangeRates[pair]`
- Emits `ExchangeRateUpdated(pair, rate)`

**Use Case**: Oracle operators offline, DEX needs fallback rate for quoting

---

### Merchant Verification (Tourism Incentives)

#### `verify_merchant(origin, merchant, category, certification_number, reward_rate, expires_at)`
**Purpose**: Verify tourism merchant for DALLA spending rewards

**Parameters**:
- `origin`: Signed origin (authorized operator)
- `merchant`: Merchant account
- `category`: `MerchantCategory` (Restaurant, Hotel, TourOperator, etc.)
- `certification_number`: Belize Tourism Board certification (e.g., "BTB-CERT-2024-00456")
- `reward_rate`: Reward percentage (5-8%)
- `expires_at`: Optional expiration block (annual renewal)

**Requirements**:
- Origin is authorized operator
- Reward rate within bounds (5-8%)

**Outcome**:
- Adds to `MerchantCategories`
- Emits `MerchantVerified(merchant, category, reward_rate)`

**Example**:
```rust
// Verify restaurant for 6% DALLA cashback
Oracle::verify_merchant(
    Origin::signed(operator_account),
    restaurant_account,
    MerchantCategory::Restaurant,
    b"BTB-CERT-2024-00789".to_vec().try_into().unwrap(),
    6, // 6% cashback
    Some(current_block + 5_256_000) // 1 year expiry
)?;
// Result: Restaurant eligible for 6% DALLA rewards
```

---

#### `revoke_merchant_verification(origin, merchant)`
**Purpose**: Remove merchant verification (fraud, non-compliance)

**Parameters**:
- `origin`: `T::OracleAdminOrigin` OR authorized operator
- `merchant`: Merchant to revoke

**Outcome**:
- Removes from `MerchantCategories`
- Emits `MerchantVerificationRevoked(merchant)`

---

### Sanctions Compliance

#### `add_sanctioned_entity(origin, account, reason, level, expires_at)`
**Purpose**: Add account to OFAC/UN sanctions list

**Parameters**:
- `origin`: Signed origin (authorized operator)
- `account`: Account to sanction
- `reason`: Sanction reason (max 256 bytes, e.g., "OFAC SDN List - Drug Trafficking")
- `level`: `SanctionLevel` (Warning, Restricted, Blocked)
- `expires_at`: Optional expiration block (if temporary)

**Requirements**:
- Origin is authorized operator

**Outcome**:
- Adds to `SanctionedEntities`
- **Immediate effect**: All pallets check sanctions before operations
- Emits `EntitySanctioned(account, reason, level)`

**Example**:
```rust
// Add account to sanctions list (OFAC)
Oracle::add_sanctioned_entity(
    Origin::signed(operator_account),
    suspicious_account,
    b"OFAC SDN List - Specially Designated National".to_vec().try_into().unwrap(),
    SanctionLevel::Blocked,
    None // Permanent
)?;
// Result:
// - Account BLOCKED from all financial operations
// - Economy pallet rejects bBZD minting
// - LandLedger blocks property transfers
// - Interoperability blocks bridge operations
```

---

#### `remove_sanction(origin, account)`
**Purpose**: Remove account from sanctions list (cleared by OFAC/UN)

**Parameters**:
- `origin`: `T::OracleAdminOrigin` OR authorized operator
- `account`: Account to unsanction

**Outcome**:
- Removes from `SanctionedEntities`
- Emits `SanctionRemoved(account)`

---

### KYC Data Bridge (Supplements Identity Pallet)

#### `verify_identity(origin, account, kyc_level, transaction_limit, daily_limit, expires_at)`
**Purpose**: Provide external KYC verification (supplements Identity pallet)

**Parameters**:
- `origin`: Signed origin (authorized operator)
- `account`: Account to verify
- `kyc_level`: `KycLevel` (L0, L1, L2, L3)
- `transaction_limit`: Per-transaction limit (bBZD, smallest unit)
- `daily_limit`: Daily limit (bBZD)
- `expires_at`: Expiration block (annual renewal)

**Requirements**:
- Origin is authorized operator

**Outcome**:
- Adds to `IdentityVerifications`
- Emits `IdentityVerified(account, kyc_level, transaction_limit)`

**Example**:
```rust
// Verify account with KYC L2 (SSN + Passport)
Oracle::verify_identity(
    Origin::signed(operator_account),
    citizen_account,
    KycLevel::L2,
    10_000 * 10u128.pow(12), // 10K bBZD per transaction
    50_000 * 10u128.pow(12), // 50K bBZD daily
    current_block + 5_256_000 // 1 year expiry
)?;
// Result: Account can perform full financial operations
```

---

### Land Registry Integration

#### `register_land(origin, property_id, owner, title_number)`
**Purpose**: Cross-check property ownership with external land registry

**Parameters**:
- `origin`: Signed origin (authorized operator)
- `property_id`: u32 internal property ID (from LandLedger pallet)
- `owner`: Current owner (cross-checked against national database)
- `title_number`: Physical title number (e.g., "BZ-BELIZE-CITY-2024-00123")

**Requirements**:
- Origin is authorized operator
- Title number verified against Lands and Surveys Department database

**Outcome**:
- Adds to `LandRegistryData`
- Emits `LandOwnershipRegistered(property_id, owner, title_number)`

**Example**:
```rust
// Oracle operator confirms property ownership
Oracle::register_land(
    Origin::signed(operator_account),
    42, // property_id from LandLedger
    owner_account,
    b"BZ-CAYO-2026-00789".to_vec().try_into().unwrap()
)?;
// Result: LandLedger pallet can verify ownership via Oracle
```

---

### IoT Device Management

#### `register_iot_device(origin, device_id, device_type, location)`
**Purpose**: Register IoT sensor device (agriculture, environment)

**Parameters**:
- `origin`: Signed origin (device owner)
- `device_id`: Unique device hash (32 bytes, e.g., SHA-256 of serial number)
- `device_type`: `IoTDeviceType` (WeatherStation, SoilSensor, etc.)
- `location`: GPS coordinates `(latitude_i64, longitude_i64)`

**Requirements**:
- Device ID unique (not already registered)
- Location within Belize territory

**Outcome**:
- Adds to `IoTDevices` with `verified: false`
- Emits `IoTDeviceRegistered(device_id, owner, device_type)`

**Example**:
```rust
// Farmer registers soil moisture sensor
let device_id = sha256(b"SENSOR-SN-123456789"); // 32-byte hash
let location = (17_250_000, -88_770_000); // Cayo District

Oracle::register_iot_device(
    Origin::signed(farmer_account),
    device_id,
    IoTDeviceType::SoilSensor,
    location
)?;
// Result: Device registered, pending verification
```

---

#### `verify_iot_device(origin, device_id, verified)`
**Purpose**: Oracle operator verifies IoT device (physical inspection)

**Parameters**:
- `origin`: Signed origin (authorized operator)
- `device_id`: Device hash
- `verified`: `true` to verify, `false` to revoke

**Requirements**:
- Origin is authorized operator
- Device exists

**Outcome**:
- Sets `verified` flag in `IoTDevices`
- Emits `IoTDeviceVerified(device_id, verified)`

---

#### `submit_iot_data(origin, device_id, data_type, value, unit)`
**Purpose**: Submit sensor data from IoT device

**Parameters**:
- `origin`: Signed origin (device owner OR automated submission)
- `device_id`: Device hash
- `data_type`: Data type (e.g., "temperature", "humidity", "soil_moisture")
- `value`: Sensor reading (i128, scaled appropriately)
- `unit`: Measurement unit (e.g., "celsius", "percent", "ppm")

**Requirements**:
- Origin owns device
- Device is verified (`verified: true`)

**Outcome**:
- Adds to `IoTDataSubmissions[device_id][block]`
- Updates `last_data_submission` in `IoTDevices`
- Emits `IoTDataSubmitted(device_id, data_type, value, unit)`

**Example**:
```rust
// Soil sensor submits moisture reading
Oracle::submit_iot_data(
    Origin::signed(farmer_account),
    device_id,
    b"soil_moisture".to_vec().try_into().unwrap(),
    65_000, // 65% moisture (scaled by 1000)
    b"percent".to_vec().try_into().unwrap()
)?;
// Result: Data stored, available for analytics
```

---

### Operator Rewards

#### `claim_oracle_rewards(origin)`
**Purpose**: Oracle operator claims accumulated rewards

**Parameters**:
- `origin`: Signed origin (operator account)

**Requirements**:
- Origin is authorized operator
- Has unclaimed rewards (based on stats)

**Outcome**:
- Calculates rewards based on `OperatorStats.accurate_submissions`
- Transfers DALLA rewards from treasury
- Emits `RewardsClaimed(operator, amount)`

**Reward Formula**:
```rust
// Base reward: 1 DALLA per accurate submission
// Bonus: +50% for 90%+ accuracy rate
let base_reward = stats.accurate_submissions * 1_000_000_000_000; // 1 DALLA
let accuracy_rate = (stats.accurate_submissions * 100) / stats.total_submissions;
let bonus = if accuracy_rate >= 90 {
    base_reward / 2 // 50% bonus
} else {
    0
};
let total_reward = base_reward + bonus;
```

---

## Helper Functions (View/Query)

### `is_sanctioned(account) -> bool`
**Purpose**: Check if account is on sanctions list

**Returns**: `true` if sanctioned, `false` otherwise

**Used By**: ALL pallets before financial operations

---

### `get_exchange_rate(pair) -> Option<u128>`
**Purpose**: Get exchange rate for currency pair (fallback for DEX)

**Returns**: Consensus price or manual rate (6 decimal precision, 1e6 scale)

**Priority**:
1. Manual rate (if set)
2. Consensus price (if recent < 50,000 blocks)
3. `None` (if stale)

**Example**:
```rust
let pair = CurrencyPair::new(Currency::DALLA, Currency::USD);
let rate = Oracle::get_exchange_rate(pair); // Returns: Some(500_000) = 0.50 USD per DALLA
```

---

### `get_merchant_category(merchant) -> Option<MerchantCategory>`
**Purpose**: Get merchant category for tourism rewards

**Returns**: Category or `None` if not verified

---

### `get_merchant_reward_rate(merchant) -> Option<u8>`
**Purpose**: Get merchant reward percentage (5-8%)

**Returns**: Reward rate or `None`

---

### `get_kyc_level(account) -> Option<KycLevel>`
**Purpose**: Get KYC level from Oracle (supplements Identity pallet)

**Returns**: KYC level (L0-L3) or `None`

---

### `meets_kyc_requirement(account, required_level) -> bool`
**Purpose**: Check if account meets minimum KYC level

**Returns**: `true` if meets requirement

---

### `get_transaction_limit(account) -> u128`
**Purpose**: Get per-transaction limit (bBZD)

**Returns**: Limit or 0 if not verified

---

### `get_daily_limit(account) -> u128`
**Purpose**: Get daily transaction limit (bBZD)

**Returns**: Limit or 0

---

### `verify_land_owner(property_id, account) -> bool`
**Purpose**: Cross-check property ownership (external database)

**Returns**: `true` if owner matches

**Used By**: LandLedger pallet during transfers

---

### `get_iot_device(device_id) -> Option<IoTDevice>`
**Purpose**: Get IoT device details

**Returns**: Device info or `None`

---

### `is_device_verified(device_id) -> bool`
**Purpose**: Check if IoT device is verified

**Returns**: `true` if verified by operator

---

### `get_operator_stats(operator) -> Option<OracleOperatorStats>`
**Purpose**: Get operator performance statistics

**Returns**: Stats or `None` if not operator

---

## Events

| Event | When Emitted |
|-------|-------------|
| `OperatorAdded(AccountId)` | New operator authorized |
| `OperatorRemoved(AccountId)` | Operator revoked |
| `PriceSubmitted(AccountId, CurrencyPair, u128)` | Operator submits price |
| `PriceFeedUpdated(CurrencyPair, u128, u32)` | Consensus price updated (price, num_operators) |
| `ExchangeRateUpdated(CurrencyPair, u128)` | Admin sets manual rate |
| `MerchantVerified(AccountId, MerchantCategory, u8)` | Merchant verified (category, reward_rate) |
| `MerchantVerificationRevoked(AccountId)` | Merchant verification removed |
| `EntitySanctioned(AccountId, Vec<u8>, SanctionLevel)` | Account sanctioned (reason, level) |
| `SanctionRemoved(AccountId)` | Sanction lifted |
| `IdentityVerified(AccountId, KycLevel, u128)` | KYC verification added (level, limit) |
| `LandOwnershipRegistered(u32, AccountId, Vec<u8>)` | Property ownership verified (property_id, owner, title) |
| `IoTDeviceRegistered([u8; 32], AccountId, IoTDeviceType)` | IoT device registered |
| `IoTDeviceVerified([u8; 32], bool)` | Device verification status changed |
| `IoTDataSubmitted([u8; 32], Vec<u8>, i128, Vec<u8>)` | Sensor data submitted (device_id, data_type, value, unit) |
| `RewardsClaimed(AccountId, u128)` | Operator claimed rewards |

---

## Errors

| Error | Cause |
|-------|-------|
| `NotAuthorizedOperator` | Origin is not authorized operator |
| `OperatorAlreadyExists` | Operator already authorized |
| `OperatorNotFound` | Operator does not exist |
| `InvalidPrice` | Price submission out of reasonable bounds |
| `InsufficientConsensus` | Less than 3 operators submitted price |
| `StaleData` | Oracle data older than max staleness (50,000 blocks) |
| `InvalidRewardRate` | Merchant reward rate not 5-8% |
| `MerchantNotFound` | Merchant not verified |
| `EntityAlreadySanctioned` | Account already on sanctions list |
| `SanctionNotFound` | Sanction does not exist |
| `InvalidKycLevel` | KYC level out of bounds |
| `DeviceAlreadyRegistered` | IoT device ID already exists |
| `DeviceNotFound` | Device does not exist |
| `DeviceNotVerified` | Device not verified by operator |
| `InvalidLocation` | GPS coordinates outside Belize |
| `NoRewardsToClaim` | Operator has zero unclaimed rewards |

---

## Integration with Other Pallets

### Economy Pallet

**Used For**:
- Merchant category lookup → tourism cashback calculation
- Sanctions check → block bBZD minting for sanctioned accounts

**Example**:
```rust
// Economy pallet checks merchant category
let category = Oracle::get_merchant_category(&merchant);
let reward_rate = match category {
    Some(MerchantCategory::Restaurant) => 6,  // 6%
    Some(MerchantCategory::Hotel) => 7,       // 7%
    Some(MerchantCategory::TourOperator) => 8, // 8%
    _ => 0, // Not a verified merchant
};
```

---

### Identity Pallet

**Used For**:
- Sanctions check → verify account not on OFAC/UN list
- KYC data bridge → external verification supplement

**Example**:
```rust
// Identity pallet checks sanctions
if Oracle::is_sanctioned(&account) {
    return Err(Error::<T>::AccountSanctioned.into());
}
```

---

### LandLedger Pallet

**Used For**:
- Property ownership verification → cross-check with Lands and Surveys Department
- KYC check → ensure both parties have KYC L2
- Sanctions check → block transfers for sanctioned accounts

**Example**:
```rust
// LandLedger verifies property ownership
if !Oracle::verify_land_owner(property_id, &account) {
    return Err(Error::<T>::InvalidOwnership.into());
}
```

---

### Interoperability Pallet

**Used For**:
- Sanctions check → block bridge operations for sanctioned accounts
- KYC check → bridge operators require KYC L3

---

### Staking Pallet

**Used For**:
- Sanctions check → block validator registration for sanctioned accounts

---

## Configuration Parameters

### `Config` Trait

```rust
pub trait Config: frame_system::Config {
    type OracleAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    #[pallet::constant]
    type MaxOperators: Get<u32>;                    // 10 operators max
    
    #[pallet::constant]
    type MaxDataStaleness: Get<BlockNumberFor<Self>>; // 50,000 blocks (~1 week)
    
    #[pallet::constant]
    type MinConsensusOperators: Get<u32>;           // 3 operators minimum
    
    type WeightInfo: WeightInfo;
}
```

**Recommended Values**:
- `MaxOperators`: `10` (reasonable for consensus)
- `MaxDataStaleness`: `50_000` (blocks, ~1 week at 6s/block)
- `MinConsensusOperators`: `3` (minimum for median consensus)

---

## Testing

### Unit Tests (src/tests.rs)

**Coverage**:
- Operator management (add/remove)
- Price submission and consensus (3+ operators)
- Merchant verification (all 6 categories)
- Sanctions enforcement (3 levels)
- KYC data bridge (L0-L3)
- Land ownership verification
- IoT device registration and verification
- IoT data submission
- Operator reward calculation
- Data staleness checks

**Run Tests**:
```bash
cargo test -p pallet-belize-oracle
```

---

## Integration with BelizeChain

### Runtime Configuration (belizechain/runtime/src/lib.rs)

```rust
impl pallet_belize_oracle::Config for Runtime {
    type OracleAdminOrigin = EnsureRoot<AccountId>;
    type MaxOperators = ConstU32<10>;
    type MaxDataStaleness = ConstU32<50_000>;      // ~1 week
    type MinConsensusOperators = ConstU32<3>;
    type WeightInfo = ();
}
```

---

## Future Enhancements

1. **Weather Oracle**: Real-time weather data for agriculture insurance
2. **Crop Price Oracle**: Market prices for citrus, sugar, seafood (farmer subsidies)
3. **Carbon Credit Oracle**: Verify carbon sequestration for conservation payments
4. **Tourism Occupancy Oracle**: Hotel/resort occupancy rates for tax assessment
5. **Health Data Oracle**: Anonymized health statistics for national healthcare planning
6. **Education Oracle**: Student performance data for school funding allocation
7. **Disaster Oracle**: Hurricane/flood damage assessment for emergency relief
8. **Wildlife Oracle**: Species population monitoring for conservation

---

## References

- **OFAC Sanctions**: [U.S. Treasury SDN List](https://sanctionssearch.ofac.treas.gov/)
- **UN Sanctions**: [UN Security Council Sanctions](https://www.un.org/securitycouncil/sanctions)
- **Belize Tourism Board**: [BTB Certification](https://www.travelbelize.org/)
- **Polkadot SDK Documentation**: [Substrate Pallet Development](https://docs.substrate.io/)
- **Identity Pallet**: `belizechain/pallets/identity/README.md` (KYC integration)
- **LandLedger Pallet**: `belizechain/pallets/landledger/README.md` (property verification)

---

## Contact & Support

For questions regarding BelizeChain Oracle implementation or data integration:
- **Technical**: BelizeChain Core Developer Team
- **Policy**: Financial Services Commission (sanctions), Belize Tourism Board (merchant verification)
- **Oracle Operators**: oracle-support@belizechain.org

**Audit Status**: ✅ Complete (January 2026)  
**Clippy Warnings**: 0  
**Test Coverage**: Comprehensive unit tests  
**Operators**: TBD (mainnet launch)  
**Integration**: 6+ pallets (Economy, Identity, LandLedger, Interoperability, Staking, Governance)
