# BelizeID: Sovereign Identity Pallet

## Overview

The **Identity** pallet (`pallet-belize-identity`) implements BelizeID, a sovereign digital identity system designed specifically for the nation of Belize. It provides privacy-preserving, blockchain-anchored identity verification with strong alignment to Belizean institutions (Social Security Board, Immigration Department) while supporting global standards (ICAO Doc 9303 passports, Decentralized Identifiers).

**Key Features**:
- **Privacy-First Architecture**: On-chain salted hashes + credential anchors, zero plaintext PII
- **Multi-Tier KYC System**: 4 levels (L0-L3) from anonymous to full biometric verification
- **Government Integration**: Direct support for SSN (SSB 9-digit) and Passport (ICAO Doc 9303)
- **Issuer Attestations**: Immigration, Social Security Board, approved biometric providers
- **Annual Validity**: 52-week validity + 26-week grace period, revocation/suspension by governance
- **Multi-Account Support**: Link multiple blockchain accounts to single identity
- **DID Export**: Minimal decentralized identifier format `did:belize:<identity_hash>`
- **Oracle Integration**: Provides KYC data to other pallets via `IdentityOracleProvider` trait
- **Compliance**: OFAC/UN sanctions checking, FSC oversight integration

---

## Architecture

### 1. Privacy Model

BelizeID is designed with **privacy-by-default**:

```
┌──────────────────────────────────────┐
│   ON-CHAIN (Blockchain Storage)      │
├──────────────────────────────────────┤
│  ✓ Salted hash of credentials        │
│  ✓ Credential anchors (CID pointers) │
│  ✓ Attestation metadata (issuer, ts) │
│  ✓ KYC level (L0/L1/L2/L3)            │
│  ✗ NO plaintext PII (SSN/Passport)    │
└──────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│   OFF-CHAIN (Pakit DAG Storage)      │
├──────────────────────────────────────┤
│  ✓ Encrypted PII documents (DAG)     │
│  ✓ Biometric templates (secure TEE)  │
│  ✓ Full credential history (DAG)     │
└──────────────────────────────────────┘
```

**Verification Flow**:
1. Citizen provides credentials to issuer (off-chain)
2. Issuer validates (government database, biometric match)
3. Issuer generates salted hash + credential anchor (CID from Pakit DAG)
4. Attestation posted on-chain (hash + anchor only)
5. Other pallets query KYC level via Oracle trait (no raw PII exposed)

### 2. KYC Hierarchy (4 Tiers)

| Level | Name | Requirements | Capabilities | Validity |
|-------|------|--------------|--------------|----------|
| **L0** | Anonymous | None | Read-only, observer | N/A |
| **L1** | SSN Verified | Social Security Number (SSB) | Basic financial ops, voting | 52 weeks |
| **L2** | Full Identity | SSN + Passport (ICAO Doc 9303) | Full financial, land registry, governance | 52 weeks |
| **L3** | Biometric+ | SSN + Passport + Biometrics | Validators, bridge operators, treasury | 52 weeks |

**KYC States** (time-based):
- `Active`: Within 52-week validity
- `Grace`: 52-78 weeks (26-week grace period)
- `Expired`: Beyond 78 weeks (requires re-attestation)
- `Revoked`: Suspended by governance/councils (immediate)

### 3. Issuer Ecosystem

**Three Issuer Types** (each specializes in one attribute):

1. **SSN Issuer** (Social Security Board - SSB):
   - Issues SSN attestations (9-digit format: `XXX-XX-XXXX`)
   - Verifies against national database
   - Grants KYC Level 1 (if first attestation)

2. **Passport Issuer** (Immigration Department):
   - Issues Passport attestations (ICAO Doc 9303 compliant)
   - Verifies against national passport registry
   - Grants KYC Level 2 (if SSN also present)

3. **Biometric Issuer** (Approved Providers):
   - Issues biometric attestations (fingerprint + facial recognition)
   - Stores templates in secure TEEs (Pakit DAG encrypted storage)
   - Grants KYC Level 3 (if SSN + Passport also present)

**Issuer Requirements**:
- Governance approval (added via `add_issuer()` extrinsic)
- Bond deposit (configurable, slashable for false attestations)
- Flagging system for suspicious activity
- On-chain performance monitoring

---

## Storage Items

### Identity Records

| Storage | Type | Description |
|---------|------|-------------|
| `Identities` | `StorageMap<AccountId, IdentityRecord>` | Main identity records by blockchain account |
| `AccountToIdentity` | `StorageMap<AccountId, T::Hash>` | Reverse lookup: account → identity hash |
| `IdentityAccounts` | `StorageMap<T::Hash, BoundedVec<AccountId>>` | Multi-account support: identity → linked accounts |
| `KycLevel` | `StorageMap<AccountId, KycLevel>` | Current KYC tier (L0/L1/L2/L3) |
| `Attestations` | `StorageDoubleMap<AccountId, AttributeType, Attestation>` | SSN/Passport/Biometric attestations |

**IdentityRecord Structure**:
```rust
pub struct IdentityRecord<T: Config> {
    pub name: BoundedVec<u8, T::MaxNameLen>,           // Display name
    pub did_anchor: BoundedVec<u8, T::MaxAnchorLen>,   // CID to DID document
    pub ssn_hash: Option<T::Hash>,                     // Salted hash of SSN
    pub passport_hash: Option<T::Hash>,                // Salted hash of Passport
    pub biometric_anchor: Option<BoundedVec<u8, T::MaxAnchorLen>>, // CID to biometric data
    pub created: BlockNumberFor<T>,                    // Registration timestamp
    pub last_updated: BlockNumberFor<T>,               // Last modification
}
```

**Attestation Structure**:
```rust
pub struct Attestation<T: Config> {
    pub issuer: T::AccountId,                          // Who issued (SSB/Immigration/Biometric)
    pub credential_anchor: BoundedVec<u8, T::MaxAnchorLen>, // CID to encrypted credential
    pub issued_at: BlockNumberFor<T>,                  // Issuance block
    pub expires_at: BlockNumberFor<T>,                 // Expiry (annual renewal)
    pub status: AttestationStatus,                     // Active/Suspended/Revoked
    pub standard_version: u32,                         // Compliance version (future-proofing)
}
```

### Configuration & Governance

| Storage | Type | Description |
|---------|------|-------------|
| `Issuers` | `StorageDoubleMap<AttributeType, AccountId, bool>` | Approved issuers per attribute type |
| `StandardVersions` | `StorageMap<AttributeType, u32>` | Current standard version (SSN/Passport/Biometric) |
| `OperationFee` | `StorageValue<BalanceOf<T>>` | Fee for identity operations (spam prevention) |
| `IssuerBondAmount` | `StorageValue<BalanceOf<T>>` | Required bond for issuers (slashable) |
| `IssuerBonds` | `StorageDoubleMap<AttributeType, AccountId, BalanceOf<T>>` | Deposited bonds by issuer |
| `FlaggedIssuers` | `StorageDoubleMap<AttributeType, AccountId, bool>` | Issuers under investigation |
| `Paused` | `StorageValue<bool>` | Emergency pause flag (governance control) |

### Anti-Spam & Rate Limiting

| Storage | Type | Description |
|---------|------|-------------|
| `RateLimits` | `StorageMap<AttributeType, (u32, BlockNumberFor<T>)>` | Max operations per time window |
| `RateCounters` | `StorageDoubleMap<AttributeType, AccountId, RateCounter>` | Per-issuer operation tracking |

### History & Auditability

| Storage | Type | Description |
|---------|------|-------------|
| `IdentityHistory` | `StorageMap<T::Hash, BoundedVec<HistoryEvent>>` | Full audit trail per identity |

---

## Extrinsics (Public Functions)

### Citizen Operations

#### `register_identity(origin, name)`
**Purpose**: Create new BelizeID identity (initial L0 KYC)

**Parameters**:
- `origin`: Signed origin (account creating identity)
- `name`: Display name (bounded, UTF-8)

**Requirements**:
- Account must not have existing identity
- Pays operation fee (spam prevention)
- System not paused

**Outcome**:
- Creates `IdentityRecord` with L0 KYC
- Generates identity hash (unique identifier)
- Emits `IdentityRegistered(account, identity_hash, name)`

**Example**:
```rust
let name = b"Juan Martinez".to_vec();
Identity::register_identity(Origin::signed(account), name.try_into().unwrap())?;
```

---

#### `link_account(origin, new_account)`
**Purpose**: Link additional blockchain account to existing identity

**Parameters**:
- `origin`: Signed origin (must have existing identity)
- `new_account`: New account to link

**Requirements**:
- Origin must have identity
- `new_account` must not have existing identity
- Total linked accounts < `MaxAccountsPerIdentity`

**Outcome**:
- Links `new_account` to origin's identity hash
- Updates `AccountToIdentity` and `IdentityAccounts` mappings
- Emits `AccountLinked(identity_hash, new_account)`

**Use Case**: Separate hot wallet (transactions) from cold storage (staking)

---

#### `update_did_doc(origin, cid)`
**Purpose**: Update DID document anchor (Pakit CID)

**Parameters**:
- `origin`: Signed origin (identity owner)
- `cid`: Content ID from Pakit (encrypted DID document)

**Requirements**:
- Origin must have identity
- CID must be valid Pakit reference

**Outcome**:
- Updates `did_anchor` in `IdentityRecord`
- Emits `DidDocUpdated(identity_hash, cid)`

---

### Issuer Operations

#### `issue_ssn(origin, account, ssn_hash, credential_anchor)`
**Purpose**: Issue SSN attestation (Social Security Board)

**Parameters**:
- `origin`: Signed origin (must be approved SSN issuer)
- `account`: Citizen's blockchain account
- `ssn_hash`: Salted hash of SSN (9-digit format)
- `credential_anchor`: CID to encrypted SSN document (Pakit DAG)

**Requirements**:
- Origin is approved SSN issuer (not flagged)
- Account has registered identity
- Issuer has deposited bond
- Within rate limits
- System not paused

**Outcome**:
- Creates SSN `Attestation` (52-week validity)
- Stores `ssn_hash` in `IdentityRecord`
- Upgrades KYC to L1 (if first attestation)
- Emits `SsnIssued(account, issuer, expires_at)`

**KYC Impact**: L0 → L1 (if previously unverified)

---

#### `issue_passport(origin, account, passport_hash, credential_anchor, country_code)`
**Purpose**: Issue Passport attestation (Immigration Department)

**Parameters**:
- `origin`: Signed origin (must be approved Passport issuer)
- `account`: Citizen's blockchain account
- `passport_hash`: Salted hash of Passport ID (ICAO Doc 9303)
- `credential_anchor`: CID to encrypted passport scan (Pakit DAG)
- `country_code`: Issuing country (e.g., `BZ` for Belize)

**Requirements**:
- Origin is approved Passport issuer
- Account has registered identity
- Issuer has deposited bond
- Within rate limits

**Outcome**:
- Creates Passport `Attestation`
- Stores `passport_hash` in `IdentityRecord`
- Upgrades KYC to L2 (if SSN also present)
- Emits `PassportIssued(account, issuer, country_code, expires_at)`

**KYC Impact**: L1 → L2 (if SSN verified), or L0 → L1 (if no SSN)

---

#### `issue_biometrics(origin, account, biometric_anchor)`
**Purpose**: Issue Biometric attestation (Approved Providers)

**Parameters**:
- `origin`: Signed origin (approved Biometric issuer)
- `account`: Citizen's blockchain account
- `biometric_anchor`: CID to encrypted biometric templates (Pakit DAG TEE)

**Requirements**:
- Origin is approved Biometric issuer
- Account has registered identity
- Issuer has deposited bond

**Outcome**:
- Creates Biometric `Attestation`
- Stores `biometric_anchor` in `IdentityRecord`
- Upgrades KYC to L3 (if SSN + Passport also present)
- Emits `BiometricsIssued(account, issuer, expires_at)`

**KYC Impact**: L2 → L3 (if SSN + Passport verified)

---

#### `issuer_deposit_bond(origin, attr)`
**Purpose**: Deposit bond to become active issuer

**Parameters**:
- `origin`: Signed origin (approved issuer)
- `attr`: Attribute type (0=SSN, 1=Passport, 2=Biometrics)

**Requirements**:
- Origin is approved issuer for `attr`
- Sufficient balance for bond amount

**Outcome**:
- Reserves bond from issuer's balance
- Enables issuer to issue attestations
- Emits `IssuerBondDeposited(issuer, attr, amount)`

---

#### `issuer_withdraw_bond(origin, attr)`
**Purpose**: Withdraw bond when leaving issuer role

**Parameters**:
- `origin`: Signed origin (issuer)
- `attr`: Attribute type

**Requirements**:
- Issuer has deposited bond
- Issuer is not flagged
- Governance approval (or time delay)

**Outcome**:
- Unreserves bond to issuer's balance
- Disables issuer attestation capability
- Emits `IssuerBondWithdrawn(issuer, attr, amount)`

---

### Governance Operations

#### `add_issuer(origin, attr, issuer)`
**Purpose**: Approve new issuer for attribute type

**Parameters**:
- `origin`: `T::GovernanceOrigin` (root or district council)
- `attr`: Attribute type (0/1/2)
- `issuer`: Account to approve

**Requirements**:
- Governance origin authority
- Issuer not already approved

**Outcome**:
- Adds issuer to `Issuers` map
- Emits `IssuerAdded(attr, issuer)`

**Example**: Adding Social Security Board as SSN issuer

---

#### `remove_issuer(origin, attr, issuer)`
**Purpose**: Revoke issuer approval

**Parameters**:
- `origin`: `T::GovernanceOrigin`
- `attr`: Attribute type
- `issuer`: Issuer to remove

**Requirements**:
- Governance authority
- Issuer currently approved

**Outcome**:
- Removes from `Issuers`
- Flags for investigation
- Emits `IssuerRemoved(attr, issuer)`

---

#### `revoke(origin, account, attr)`
**Purpose**: Permanently revoke attestation

**Parameters**:
- `origin`: `T::GovernanceOrigin` or `T::CouncilOrigin`
- `account`: Account to revoke
- `attr`: Attribute to revoke (0/1/2)

**Requirements**:
- Governance or council authority
- Attestation exists and is active

**Outcome**:
- Sets attestation status to `Revoked`
- Downgrades KYC level
- Emits `AttestationRevoked(account, attr, by)`

**Use Case**: Court order, identity fraud, sanctions enforcement

---

#### `suspend(origin, account, attr)`
**Purpose**: Temporarily suspend attestation (investigation)

**Parameters**:
- `origin`: `T::GovernanceOrigin` or `T::CouncilOrigin`
- `account`: Account to suspend
- `attr`: Attribute to suspend

**Requirements**:
- Governance/council authority
- Attestation exists

**Outcome**:
- Sets status to `Suspended`
- KYC level queries return `None` (treated as unverified)
- Emits `AttestationSuspended(account, attr, by)`

**Use Case**: Pending investigation, temporary restrictions

---

#### `flag_issuer(origin, attr, issuer, flagged)`
**Purpose**: Flag/unflag issuer for investigation

**Parameters**:
- `origin`: `T::GovernanceOrigin`
- `attr`: Attribute type
- `issuer`: Issuer account
- `flagged`: `true` to flag, `false` to clear

**Outcome**:
- Updates `FlaggedIssuers`
- Prevents new attestations (if flagged)
- Emits `IssuerFlagged(attr, issuer, flagged)`

---

#### `slash_issuer_bond(origin, attr, issuer, amount)`
**Purpose**: Punish issuer for false attestations

**Parameters**:
- `origin`: `T::GovernanceOrigin`
- `attr`: Attribute type
- `issuer`: Issuer to slash
- `amount`: Amount to slash (≤ deposited bond)

**Requirements**:
- Governance authority
- Issuer has deposited bond ≥ `amount`

**Outcome**:
- Transfers slashed amount to treasury
- Reduces issuer bond
- Emits `IssuerBondSlashed(issuer, attr, amount)`

**Use Case**: Fraudulent attestations, negligence

---

#### `set_standard_version(origin, attr, version)`
**Purpose**: Update standard version for attribute (future-proofing)

**Parameters**:
- `origin`: `T::GovernanceOrigin`
- `attr`: Attribute type
- `version`: New version number

**Outcome**:
- Updates `StandardVersions`
- Emits `StandardVersionUpdated(attr, old_version, new_version)`

**Use Case**: New SSN format, updated biometric standards

---

#### `set_operation_fee(origin, fee)`
**Purpose**: Update identity operation fee (spam prevention)

**Outcome**: Updates `OperationFee`, emits event

---

#### `set_issuer_bond_amount(origin, amount)`
**Purpose**: Update required bond for issuers

**Outcome**: Updates `IssuerBondAmount`, emits event

---

#### `set_rate_limits(origin, attr, max_ops, window)`
**Purpose**: Set rate limits for issuer operations

**Parameters**:
- `attr`: Attribute type
- `max_ops`: Max operations allowed
- `window`: Time window (blocks)

**Outcome**: Updates `RateLimits`, emits event

---

#### `set_pause(origin, paused)`
**Purpose**: Emergency pause all identity operations

**Parameters**:
- `paused`: `true` to pause, `false` to resume

**Outcome**: Updates `Paused`, emits `SystemPaused(paused)`

---

#### `report_bad_attestation(origin, account, attr, reason)`
**Purpose**: Community reporting of suspicious attestations

**Parameters**:
- `origin`: Signed origin (any account)
- `account`: Account with suspicious attestation
- `attr`: Attribute type
- `reason`: Bounded reason string

**Requirements**:
- Pays reporting fee (anti-spam)
- Account has attestation

**Outcome**:
- Creates audit record
- Flags issuer for investigation (if reports > threshold)
- Emits `AttestationReported(account, attr, reporter, reason)`

---

## Helper Functions (View/Query)

### `get_verified_kyc_level(account) -> Option<u8>`
**Purpose**: Get current KYC level for account (0-3)

**Returns**:
- `Some(0)`: L0 (registered, no attestations)
- `Some(1)`: L1 (SSN verified)
- `Some(2)`: L2 (SSN + Passport)
- `Some(3)`: L3 (SSN + Passport + Biometrics)
- `None`: No identity registered

**Oracle Integration**: Used by other pallets via `IdentityOracleProvider` trait

---

### `meets_kyc_requirement_level(account, required_level) -> bool`
**Purpose**: Check if account meets minimum KYC level

**Example**:
```rust
// Validator requires L3 KYC
if !Identity::meets_kyc_requirement_level(&account, 3) {
    return Err(Error::<T>::InsufficientKyc.into());
}
```

---

### `is_account_sanctioned(account) -> bool`
**Purpose**: Check OFAC/UN sanctions status

**Integration**: Oracle pallet provides external sanctions data

**Returns**: `true` if account is sanctioned (blocks operations)

---

### `kyc_state(account, level, now) -> KycState`
**Purpose**: Get time-based KYC state

**Returns**:
- `Active`: Within 52-week validity
- `Grace`: 52-78 weeks (grace period)
- `Expired`: Beyond 78 weeks
- `Revoked`: Governance/council revocation

---

### `did_of(account) -> Option<Did>`
**Purpose**: Export minimal DID (Decentralized Identifier)

**Format**: `did:belize:<identity_hash>`

**Example**:
```
did:belize:0x8f3a2b1c9e4d5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0
```

---

## Events

| Event | When Emitted |
|-------|-------------|
| `IdentityRegistered(AccountId, T::Hash, Vec<u8>)` | New identity created |
| `AccountLinked(T::Hash, AccountId)` | Additional account linked to identity |
| `DidDocUpdated(T::Hash, Vec<u8>)` | DID document updated |
| `SsnIssued(AccountId, AccountId, BlockNumber)` | SSN attestation issued |
| `PassportIssued(AccountId, AccountId, BoundedVec<u8>, BlockNumber)` | Passport attestation issued |
| `BiometricsIssued(AccountId, AccountId, BlockNumber)` | Biometric attestation issued |
| `AttestationRevoked(AccountId, u8, AccountId)` | Attestation permanently revoked |
| `AttestationSuspended(AccountId, u8, AccountId)` | Attestation temporarily suspended |
| `IssuerAdded(u8, AccountId)` | New issuer approved |
| `IssuerRemoved(u8, AccountId)` | Issuer approval revoked |
| `IssuerBondDeposited(AccountId, u8, Balance)` | Issuer deposited bond |
| `IssuerBondWithdrawn(AccountId, u8, Balance)` | Issuer withdrew bond |
| `IssuerFlagged(u8, AccountId, bool)` | Issuer flagged for investigation |
| `IssuerBondSlashed(AccountId, u8, Balance)` | Issuer bond slashed for fraud |
| `SystemPaused(bool)` | Emergency pause activated/deactivated |
| `AttestationReported(AccountId, u8, AccountId, Vec<u8>)` | Community reported suspicious attestation |

---

## Errors

| Error | Cause |
|-------|-------|
| `IdentityAlreadyExists` | Account already has identity |
| `IdentityNotFound` | Account does not have identity |
| `InvalidIssuer` | Origin is not approved issuer for attribute |
| `InsufficientBond` | Issuer bond below required amount |
| `RateLimitExceeded` | Issuer exceeded operation rate limit |
| `SystemPaused` | Operations paused by governance |
| `MaxAccountsReached` | Cannot link more accounts to identity |
| `InvalidAttribute` | Attribute type out of bounds (0-2) |
| `AttestationNotFound` | No attestation exists for account/attribute |
| `AttestationAlreadyRevoked` | Cannot revoke already-revoked attestation |
| `IssuerFlagged` | Issuer is flagged for investigation |
| `DuplicateIssuer` | Issuer already approved for attribute |
| `InvalidCredentialAnchor` | Pakit CID malformed or inaccessible |

---

## Oracle Integration

**Exported Trait**: `IdentityOracleProvider<AccountId>`

```rust
pub trait IdentityOracleProvider<AccountId> {
    /// Get current KYC level (0-3) for account
    fn get_kyc_level(who: &AccountId) -> Option<u8>;
    
    /// Check if account meets minimum KYC requirement
    fn meets_kyc_requirement(who: &AccountId, required_level: u8) -> bool;
    
    /// Check if account is sanctioned (OFAC/UN compliance)
    fn is_sanctioned(who: &AccountId) -> bool;
}
```

**Used By**:
- **Economy Pallet**: KYC L2 required for bBZD minting requests
- **Governance Pallet**: KYC L1 required for voting
- **Staking Pallet**: KYC L3 required for validators
- **Interoperability Pallet**: KYC L3 required for bridge operators
- **LandLedger Pallet**: KYC L2 required for property transfers
- **Payroll Pallet**: KYC L1 required for payroll recipients
- **Oracle Pallet**: Provides external sanctions data

---

## Configuration Parameters

### `Config` Trait

```rust
pub trait Config: frame_system::Config {
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    #[pallet::constant]
    type ValidityPeriod: Get<BlockNumberFor<Self>>;          // 52 weeks
    
    #[pallet::constant]
    type GracePeriod: Get<BlockNumberFor<Self>>;             // 26 weeks
    
    #[pallet::constant]
    type MaxNameLen: Get<u32>;                               // 64 bytes
    
    #[pallet::constant]
    type MaxAnchorLen: Get<u32>;                             // 128 bytes (CID length)
    
    #[pallet::constant]
    type MaxAccountsPerIdentity: Get<u32>;                   // 10 accounts
    
    #[pallet::constant]
    type MaxHistoryLength: Get<u32>;                         // 100 events
    
    #[pallet::constant]
    type ReportFee: Get<BalanceOf<Self>>;                    // Anti-spam fee
}
```

**Recommended Values**:
- `ValidityPeriod`: `52 weeks * 7 days * 24 hours * 600 blocks/hour = 5,241,600 blocks`
- `GracePeriod`: `26 weeks * 7 days * 24 hours * 600 blocks/hour = 2,620,800 blocks`
- `MaxNameLen`: `64` (UTF-8 display name)
- `MaxAnchorLen`: `128` (Pakit CID format)
- `MaxAccountsPerIdentity`: `10` (hot/cold wallets, specialized accounts)
- `MaxHistoryLength`: `100` (full audit trail per identity)
- `ReportFee`: `10 DALLA` (spam prevention for `report_bad_attestation`)

---

## Typical Workflows

### 1. Citizen Onboarding (L0 → L3)

```rust
// Step 1: Citizen registers identity (L0 KYC)
let name = b"Maria Gonzalez".to_vec();
Identity::register_identity(Origin::signed(citizen_account), name.try_into().unwrap())?;
// Result: IdentityRegistered event, L0 KYC level

// Step 2: Visit Social Security Board (off-chain)
// - Provide SSN (9-digit)
// - SSB verifies against national database
// - SSB generates salted hash + stores encrypted copy in Pakit DAG

// Step 3: SSB issues SSN attestation (L1 KYC)
let ssn_hash = hash_with_salt(b"123-45-6789", salt);
let ssn_cid = pakit_dag_upload_encrypted(b"123-45-6789", ssb_key);
Identity::issue_ssn(
    Origin::signed(ssb_issuer_account),
    citizen_account,
    ssn_hash,
    ssn_cid.try_into().unwrap()
)?;
// Result: SsnIssued event, L1 KYC level

// Step 4: Visit Immigration Department (off-chain)
// - Provide passport (ICAO Doc 9303)
// - Immigration verifies against passport registry
// - Generates hash + Pakit DAG CID

// Step 5: Immigration issues Passport attestation (L2 KYC)
let passport_hash = hash_with_salt(b"BZ1234567", salt);
let passport_cid = pakit_dag_upload_encrypted(passport_scan, immigration_key);
Identity::issue_passport(
    Origin::signed(immigration_issuer_account),
    citizen_account,
    passport_hash,
    passport_cid.try_into().unwrap(),
    b"BZ".to_vec().try_into().unwrap()
)?;
// Result: PassportIssued event, L2 KYC level (SSN + Passport)

// Step 6: Visit biometric provider (off-chain)
// - Fingerprint scan + facial recognition
// - Templates stored in Pakit DAG TEE (secure enclave)

// Step 7: Biometric provider issues attestation (L3 KYC)
let biometric_cid = pakit_dag_upload_tee(biometric_templates, provider_key);
Identity::issue_biometrics(
    Origin::signed(biometric_issuer_account),
    citizen_account,
    biometric_cid.try_into().unwrap()
)?;
// Result: BiometricsIssued event, L3 KYC level (full verification)

// Step 8: Citizen can now become validator (requires L3)
Staking::bond(Origin::signed(citizen_account), 10_000_000)?; // Requires L3 KYC check
```

---

### 2. Multi-Account Identity (Hot + Cold Wallets)

```rust
// Citizen has identity on primary account
let primary_account = /* ... */;

// Create new account for cold storage (staking)
let cold_storage_account = /* ... */;

// Link cold storage to existing identity
Identity::link_account(
    Origin::signed(primary_account),
    cold_storage_account
)?;
// Result: AccountLinked event, both accounts share same identity hash

// Now both accounts have same KYC level
assert_eq!(
    Identity::get_verified_kyc_level(&primary_account),
    Identity::get_verified_kyc_level(&cold_storage_account)
);
// Both return Some(3) if L3 verified
```

---

### 3. Issuer Lifecycle

```rust
// Governance approves new SSN issuer (Social Security Board)
Identity::add_issuer(
    Origin::root(),
    0, // AttributeType::Ssn
    ssb_account
)?;
// Result: IssuerAdded event

// SSB deposits bond to become active
Identity::issuer_deposit_bond(
    Origin::signed(ssb_account),
    0 // SSN
)?;
// Result: IssuerBondDeposited event, can now issue attestations

// SSB issues attestations (within rate limits)
for citizen in citizens.iter() {
    Identity::issue_ssn(Origin::signed(ssb_account), citizen, hash, cid)?;
}

// Later: SSB wants to withdraw from issuer role
Identity::issuer_withdraw_bond(
    Origin::signed(ssb_account),
    0
)?;
// Result: IssuerBondWithdrawn event, bond returned
```

---

### 4. Governance Revocation (Court Order)

```rust
// Court orders identity revocation due to fraud
Identity::revoke(
    Origin::root(),
    fraudulent_account,
    0 // Revoke SSN
)?;
// Result: AttestationRevoked event, KYC downgraded to L0

// Account can no longer participate in financial operations
let result = Economy::transfer_bbzd(
    Origin::signed(fraudulent_account),
    recipient,
    1000
);
assert_eq!(result, Err(Error::<T>::InsufficientKyc));
```

---

### 5. Oracle Integration (Other Pallets)

```rust
// In Economy pallet: Check KYC before bBZD mint request
impl<T: Config> Pallet<T> {
    pub fn request_bbzd_mint(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Requires L2 KYC (SSN + Passport)
        ensure!(
            T::IdentityProvider::meets_kyc_requirement(&who, 2),
            Error::<T>::InsufficientKyc
        );
        
        // Check sanctions
        ensure!(
            !T::IdentityProvider::is_sanctioned(&who),
            Error::<T>::AccountSanctioned
        );
        
        // Proceed with mint request...
        Ok(())
    }
}
```

---

## Security Considerations

### 1. **Privacy Protection**
- ✅ **Zero Plaintext PII**: No SSN/Passport/Biometric data stored on-chain
- ✅ **Salted Hashes**: Rainbow table attacks prevented
- ✅ **Credential Anchors**: CIDs reference encrypted off-chain data (Pakit DAG)
- ✅ **Minimal Disclosure**: Oracle queries return only KYC level (0-3), not raw credentials

### 2. **Issuer Security**
- ✅ **Bond Slashing**: Fraudulent attestations punished financially
- ✅ **Flagging System**: Investigation without immediate revocation
- ✅ **Rate Limiting**: Prevents bulk false attestations
- ✅ **Governance Oversight**: Add/remove issuers via democratic process

### 3. **Time-Based Validity**
- ✅ **Annual Renewal**: Prevents stale identities (52-week validity)
- ✅ **Grace Period**: 26-week buffer for renewal (avoid service disruption)
- ✅ **Expiry Enforcement**: Expired attestations return `None` in KYC queries

### 4. **Sanctions Compliance**
- ✅ **OFAC/UN Integration**: Oracle provides external sanctions data
- ✅ **Real-Time Checking**: All operations check sanctions before execution
- ✅ **FSC Oversight**: Financial Services Commission monitoring

### 5. **Emergency Controls**
- ✅ **Pause Mechanism**: Governance can halt all operations (system-wide emergency)
- ✅ **Revocation**: Immediate suspension of attestations (court orders)
- ✅ **Suspension**: Temporary pause for investigation

---

## Testing

### Unit Tests (src/tests.rs)

**Coverage**:
- Identity registration (L0 KYC)
- Multi-account linking (max accounts enforcement)
- SSN attestation issuance (L0 → L1 upgrade)
- Passport attestation (L1 → L2 upgrade)
- Biometric attestation (L2 → L3 upgrade)
- KYC level queries (Oracle integration)
- Revocation workflows (governance authority)
- Suspension workflows (council authority)
- Issuer bond management (deposit/withdraw/slash)
- Rate limiting (issuer operations)
- Validity periods (active/grace/expired states)
- Multi-signature governance (add/remove issuers)

**Run Tests**:
```bash
cargo test -p pallet-belize-identity
```

---

## Integration with BelizeChain

### Runtime Configuration (belizechain/runtime/src/lib.rs)

```rust
impl pallet_belize_identity::Config for Runtime {
    type Currency = Balances;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type CouncilOrigin = EnsureOneOf<EnsureRoot<AccountId>, pallet_belize_governance::EnsureCouncilMember<Runtime>>;
    type ValidityPeriod = ConstU32<5_241_600>;      // 52 weeks
    type GracePeriod = ConstU32<2_620_800>;         // 26 weeks
    type MaxNameLen = ConstU32<64>;
    type MaxAnchorLen = ConstU32<128>;
    type MaxAccountsPerIdentity = ConstU32<10>;
    type MaxHistoryLength = ConstU32<100>;
    type ReportFee = ConstU128<10_000_000_000_000>; // 10 DALLA
}

// Oracle provider for other pallets
pub struct IdentityOracleProviderImpl;
impl pallet_belize_economy::IdentityProvider<AccountId> for IdentityOracleProviderImpl {
    fn get_kyc_level(who: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(who)
    }
    
    fn meets_kyc_requirement(who: &AccountId, level: u8) -> bool {
        Identity::meets_kyc_requirement_level(who, level)
    }
    
    fn is_sanctioned(who: &AccountId) -> bool {
        Identity::is_account_sanctioned(who)
    }
}
```

---

## Future Enhancements

1. **Zero-Knowledge Proofs**: Prove KYC level without revealing attestation details
2. **Credential Revocation Lists**: Off-chain CRL for faster revocation checks
3. **Attribute-Based Credentials**: Fine-grained permissions (e.g., "over 18", "resident of Belize District")
4. **Delegation**: Temporary identity delegation (e.g., lawyer acting on behalf of client)
5. **Cross-Chain Identity**: Bridge BelizeID to other blockchains (Polkadot, Ethereum)
6. **Biometric Liveness**: Require periodic biometric re-verification (prevent impersonation)
7. **Multi-Issuer Consensus**: Require 2-of-3 issuer agreement for high-risk attestations
8. **Privacy-Preserving Analytics**: Aggregate KYC statistics without individual exposure

---

## References

- **ICAO Doc 9303**: [Machine Readable Travel Documents (Passports)](https://www.icao.int/publications/pages/publication.aspx?docnum=9303)
- **W3C DID Specification**: [Decentralized Identifiers](https://www.w3.org/TR/did-core/)
- **Polkadot SDK Documentation**: [Substrate Pallet Development](https://docs.substrate.io/reference/how-to-guides/pallet-design/)
- **Pakit DAG Storage Layer**: `pakit/README.md` (DAG-based encrypted credential storage)
- **Oracle Pallet**: `belizechain/pallets/oracle/README.md` (sanctions data integration)

---

## Contact & Support

For questions regarding BelizeID implementation or sovereign identity policy:
- **Technical**: BelizeChain Core Developer Team
- **Policy**: Ministry of Economic Development, Belize
- **Issuers**: Social Security Board (SSN), Immigration Department (Passport)

**Audit Status**: ✅ Complete (January 2026)  
**Clippy Warnings**: 0  
**Test Coverage**: Comprehensive unit tests  
**Integration**: Oracle trait exported to 6+ pallets
