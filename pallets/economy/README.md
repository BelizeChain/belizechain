# 💰 Economy Pallet

## Overview

The **Economy Pallet** is BelizeChain's national economic system managing the dual-token architecture that underpins the country's sovereign blockchain infrastructure. It implements DALLA (native token) for network operations and bBZD (fiat-backed stablecoin) for price stability.

## Purpose

This pallet serves as the monetary foundation for Belize's blockchain economy, providing:

1. **Native Token (DALLA)**: Gas fees, staking rewards, governance participation
2. **Stablecoin (bBZD)**: 1:1 peg to Belize Dollar for everyday transactions
3. **Tourism Economy**: Incentive system to boost Belize's tourism sector
4. **Cross-Border Efficiency**: Optimized remittance processing
5. **National Treasury**: Multi-signature controlled government funds

## Architecture

### Dual-Token System

#### DALLA (Native Token)
- **Maximum Supply**: 501,000,000,000 DALLA (501 billion)
- **Decimals**: 6
- **Inflation**: 2% annual (minted to Treasury)
- **Uses**: Transaction fees, staking rewards, governance voting, tourism incentives

#### bBZD (Fiat-Backed Stablecoin)
- **Peg**: 1:1 to Belize Dollar (BZD)
- **Model**: USDC-style (fully backed by Central Bank reserves)
- **Minting**: Central Bank deposits BZD → issues bBZD
- **Redemption**: Users burn bBZD → Central Bank returns BZD
- **Backing**: 100% reserves held by Central Bank of Belize

### Economic Parameters

```rust
// Constants (from lib.rs)
TREASURY_ID: u64 = 1000
BLOCKS_PER_YEAR: u32 = 2_628_000  // ~12s block time
ANNUAL_INFLATION_RATE: u32 = 2    // 2% per year

// Tourism Incentive Rates (cashback in DALLA)
Hotels: 8%
Dining: 5%
Tours: 7%
Shopping: 3%
General: 5%
```

## Key Features

### 1. **bBZD Stablecoin Management**

**Central Bank Minting** (requires authorization):
```rust
mint_bbzd(origin, amount, recipient) -> DispatchResult
```
- Verifies caller is authorized minter (Central Bank)
- Checks adequate BZD reserves exist
- Issues bBZD 1:1 with deposited BZD
- Updates total supply and balance mappings

**Redemption Process**:
```rust
redeem_bbzd(origin, amount) -> DispatchResult
```
- Burns user's bBZD
- Creates redemption request (pending status)
- Requires Central Bank processing

```rust
process_redemption(origin, request_id) -> DispatchResult
```
- Central Bank approves redemption
- Releases BZD from reserves
- Marks request as completed

### 2. **Tourism Incentive System**

**Payment Processing with Cashback**:
```rust
process_tourism_payment(origin, amount, category, merchant_id) -> DispatchResult
```
- Validates merchant is verified (via Oracle pallet)
- Processes bBZD payment
- Calculates DALLA cashback (3-8% based on category)
- Mints DALLA rewards to user
- Transfers payment to merchant

**Incentive Calculation**:
```rust
fn get_tourism_incentive_rate(category: &TourismCategory) -> u32 {
    match category {
        Hotels => 800,      // 8.00%
        Dining => 500,      // 5.00%
        Tours => 700,       // 7.00%
        Shopping => 300,    // 3.00%
        General => 500,     // 5.00%
    }
}
```

### 3. **DALLA Supply Management**

**Controlled Burning**:
```rust
burn_dalla(origin, amount) -> DispatchResult
```
- Any user can burn their DALLA
- Permanently reduces circulating supply
- Cannot burn below existential deposit

**Governance Burning** (treasury control):
```rust
governance_burn(origin, amount) -> DispatchResult
```
- Requires governance approval
- Burns DALLA from treasury
- Used for supply management

### 4. **Multi-Signature Treasury**

**Account Type System**:
```rust
pub enum AccountType {
    Citizen,     // 25,000 DALLA daily limit
    Business,    // 100,000 DALLA daily limit
    Tourism,     // 100,000 DALLA + special incentives
    Government,  // Unlimited (multi-sig required)
}
```

**Treasury Operations** (from MultiSigOperation struct):
- Spend: Transfer funds from treasury
- Burn: Reduce total DALLA supply
- Mint: Issue new bBZD (Central Bank only)
- Reserve: Allocate funds for future use

**Multi-Signature Requirements**:
- **Threshold**: 4-of-7 signatures
- **Roles**: Minister of Finance, Governor, FSC Commissioner, BTB Director, Auditor General, Deputy Minister, Treasury Secretary
- **Workflow**: Create operation → Collect signatures → Execute when threshold met

### 5. **Remittance Optimization**

**Cross-Border Transfers**:
- **Fee Structure**: 0.5% (vs. traditional 5-10%)
- **Settlement**: Near-instant on-chain
- **Currencies**: DALLA ↔ bBZD seamless conversion
- **Compliance**: Integrated KYC/AML checks (via Compliance pallet)

## Data Structures

### Storage Items

```rust
// bBZD Stablecoin Management
CentralBankReserves: u128                    // Total BZD backing
TotalBbzdSupply: u128                        // Total bBZD in circulation
BBZDBalances: map AccountId => u128          // User bBZD balances
RedemptionRequests: map u64 => RedemptionRequest  // Pending redemptions
AuthorizedMinters: map AccountId => bool     // Central Bank minters

// DALLA Native Token
TreasuryBalance: Balance                     // Treasury DALLA holdings
LastInflationBlock: BlockNumber              // Inflation tracking

// Tourism Economy
TourismMerchants: map AccountId => bool      // Verified merchants
TourismIncentivesIssued: Balance             // Total DALLA issued

// Economic Metrics
CirculatingSupply: Balance                   // Total DALLA in circulation
TotalTransactions: u64                       // Transaction counter
```

### Key Types

```rust
pub struct RedemptionRequest<AccountId> {
    pub requester: AccountId,
    pub amount: u128,
    pub requested_at: BlockNumber,
    pub status: RedemptionStatus,
}

pub enum RedemptionStatus {
    Pending,      // Awaiting Central Bank processing
    Approved,     // Ready for BZD transfer
    Completed,    // BZD released
    Rejected,     // Denied by Central Bank
}

pub struct EconomicMetrics {
    pub dalla_supply: Balance,
    pub bbzd_supply: u128,
    pub treasury_balance: Balance,
    pub tourism_rewards: Balance,
    pub tx_count: u64,
}
```

## Extrinsics (Public Functions)

### Central Bank Operations

| Function | Authority | Description |
|----------|-----------|-------------|
| `mint_bbzd` | Central Bank | Issue bBZD after BZD deposit verification |
| `process_redemption` | Central Bank | Approve bBZD → BZD redemption |
| `set_minter_authorization` | Governance | Authorize/revoke minter status |
| `update_reserves` | Central Bank | Update BZD reserve amount |

### User Operations

| Function | Authority | Description |
|----------|-----------|-------------|
| `redeem_bbzd` | Any user | Request bBZD → BZD conversion |
| `burn_dalla` | Any user | Permanently destroy DALLA |
| `process_tourism_payment` | Any user | Pay merchant with bBZD, earn DALLA cashback |

### Governance Operations

| Function | Authority | Description |
|----------|-----------|-------------|
| `governance_burn` | Governance | Burn DALLA from treasury |

## Events

```rust
pub enum Event<T: Config> {
    // bBZD Events
    BBZDMinted { amount: u128, recipient: AccountId },
    BBZDRedeemed { requester: AccountId, amount: u128 },
    RedemptionProcessed { request_id: u64, approved: bool },
    
    // Tourism Events
    TourismPaymentProcessed { payer: AccountId, merchant: AccountId, amount: u128, cashback: Balance },
    
    // DALLA Events
    DALLABurned { account: AccountId, amount: Balance },
    
    // Treasury Events
    ReservesUpdated { new_amount: u128 },
    MinterAuthorized { account: AccountId, authorized: bool },
}
```

## Errors

```rust
pub enum Error<T> {
    // bBZD Errors
    InsufficientReserves,        // Not enough BZD backing
    UnauthorizedMinter,          // Caller not Central Bank
    InvalidRedemptionRequest,    // Request doesn't exist
    AlreadyProcessed,            // Request already handled
    
    // DALLA Errors
    InsufficientBalance,         // Not enough DALLA
    BelowExistentialDeposit,     // Would leave dust amount
    
    // Tourism Errors
    InvalidMerchant,             // Merchant not verified
    TourismIncentiveFailed,      // Couldn't mint DALLA reward
    
    // Treasury Errors
    TreasuryInsufficientFunds,   // Treasury balance too low
}
```

## Integration Points

### Cross-Pallet Dependencies

**Compliance Pallet** (KYC/AML):
- Verifies identity for large bBZD redemptions
- Enforces AML checks on cross-border transfers
- Provides KYC tier for transaction limits

**Oracle Pallet** (Merchant Verification):
- **IMPORTANT**: Oracle is ONLY used for merchant verification, NOT for bBZD peg
- bBZD peg is **always 1:1 BZD** (governed by Central Bank reserves)
- Validates tourism merchant status
- Provides merchant category data

**Governance Pallet** (Treasury Management):
- Multi-sig approval for treasury operations
- Governance burn authorization
- Economic parameter updates

**Staking Pallet** (PoUW Rewards):
- Issues DALLA rewards to validators
- Pays federated learning contributors

### Provider Pattern (from runtime/lib.rs)

```rust
// Economy queries Oracle for merchant verification
pub struct EconomyOracleProvider;
impl pallet_belize_economy::OracleProvider<AccountId> for EconomyOracleProvider {
    fn verify_merchant(merchant_id: &AccountId) -> bool {
        Oracle::is_verified_merchant(merchant_id)
    }
}
```

## Weight Functions

All extrinsics use benchmarked weights (see `SubstrateWeight<T>`):

```rust
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn mint_bbzd() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    
    fn process_tourism_payment() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    
    // ... (11 benchmarked functions total)
}
```

## Economic Model Deep Dive

### bBZD Stablecoin Mechanics

**Why 1:1 Peg to BZD?**
- Stability for everyday commerce
- Familiar value to Belizean citizens (1 bBZD = 1 BZD = 0.50 USD)
- No exchange rate volatility
- Fully backed by government reserves

**Minting Process**:
1. Central Bank receives BZD deposit (off-chain)
2. Verifies deposit with banking system
3. Calls `mint_bbzd(amount, recipient)` on-chain
4. Updates `CentralBankReserves` (+amount)
5. Issues bBZD to recipient

**Redemption Process**:
1. User calls `redeem_bbzd(amount)` 
2. System burns user's bBZD
3. Creates pending `RedemptionRequest`
4. Central Bank verifies request off-chain
5. Calls `process_redemption(request_id)`
6. Releases BZD from reserves to user's bank account

**Reserve Management**:
- **Requirement**: 100% backing (1 BZD per 1 bBZD)
- **Auditing**: Quarterly financial audits by FSC
- **Emergency Shutdown**: Governance can halt minting if reserves compromised

### DALLA Native Token Economics

**Supply Schedule**:
- **Genesis Supply**: 501,000,000,000 DALLA
- **Inflation**: 2% annually (minted to Treasury)
- **Deflationary Mechanisms**: Transaction fees burned, user/governance burns
- **Circulating Supply**: Tracked via `CirculatingSupply` storage

**Inflation Distribution** (every ~year = 2,628,000 blocks):
```rust
// Pseudo-code for inflation minting
inflation_amount = treasury_balance * 0.02
mint_to_treasury(inflation_amount)
```

**Fee Structure**:
- **Transaction Fee**: 0.01 DALLA base + weight-based
- **Tourism Cashback**: 3-8% DALLA (funded by inflation)
- **Cross-Border Fee**: 0.5% of transfer amount

### Tourism Incentive Economics

**Cashback Tiers** (paid in DALLA):
| Category | Rate | Example |
|----------|------|---------|
| Hotels | 8% | 100 bBZD payment → 8 DALLA cashback |
| Tours | 7% | 50 bBZD payment → 3.5 DALLA cashback |
| Dining | 5% | 25 bBZD payment → 1.25 DALLA cashback |
| Shopping | 3% | 40 bBZD payment → 1.2 DALLA cashback |
| General | 5% | 30 bBZD payment → 1.5 DALLA cashback |

**Funding Source**:
- DALLA minted from Treasury reserves (inflation-funded)
- Annual budget: ~10,000,000 DALLA (subject to governance)

**Merchant Requirements**:
- Verified by Oracle pallet (tourism license validation)
- KYC/AML compliant (via Compliance pallet)
- Active business registration

## Security Considerations

### Central Bank Safeguards
- **Minter Authorization**: Only approved accounts can mint bBZD
- **Reserve Checks**: Minting fails if reserves insufficient
- **Multi-Sig Treasury**: 4-of-7 signatures prevent single point of failure

### Economic Attacks
- **Inflation Attacks**: Fixed 2% annual rate prevents manipulation
- **Reserve Depletion**: 100% backing requirement enforced on-chain
- **Sybil Attacks**: Tourism rewards require verified merchant status
- **Front-Running**: Fixed incentive rates prevent MEV exploitation

### Compliance Integration
- **KYC Requirements**: Large bBZD redemptions (>$10,000) require KYC
- **AML Monitoring**: Cross-border transfers flagged for FSC review
- **Transaction Limits**: Account type-based daily limits

## Testing

### Test Coverage (from tests.rs)

**Unit Tests** (930 lines):
- bBZD minting/redemption workflows
- Tourism payment processing
- DALLA burning mechanics
- Multi-sig treasury operations
- Reserve management
- Error conditions

**Mock Runtime** (mock.rs, 159 lines):
- Test accounts (Alice, Bob, Charlie)
- Treasury account (ID: 1000)
- Central Bank account (authorized minter)
- Mock Oracle for merchant verification

**Key Test Scenarios**:
```rust
#[test]
fn test_mint_bbzd_with_reserves() { ... }

#[test]
fn test_redeem_bbzd_creates_request() { ... }

#[test]
fn test_tourism_payment_with_cashback() { ... }

#[test]
fn test_burn_dalla_reduces_supply() { ... }

#[test]
fn test_insufficient_reserves_fails() { ... }
```

## Usage Examples

### For Citizens

**Redeem bBZD for BZD**:
```rust
// User has 500 bBZD, wants BZD cash
Economy::redeem_bbzd(Origin::signed(alice), 500_000_000)?;
// Creates redemption request, burns bBZD
// Central Bank processes off-chain, releases BZD
```

**Earn Tourism Cashback**:
```rust
// Pay 100 bBZD at hotel, earn 8 DALLA
Economy::process_tourism_payment(
    Origin::signed(bob),
    100_000_000,  // 100 bBZD
    TourismCategory::Hotels,
    hotel_merchant_id
)?;
// Receives 8 DALLA cashback instantly
```

### For Central Bank

**Mint bBZD After Deposit**:
```rust
// User deposited 1,000 BZD, issue 1,000 bBZD
Economy::mint_bbzd(
    Origin::signed(central_bank),
    1_000_000_000,  // 1,000 bBZD (6 decimals)
    charlie
)?;
```

**Process Redemption**:
```rust
// Approve redemption after BZD transfer
Economy::process_redemption(
    Origin::signed(central_bank),
    request_id
)?;
```

### For Governance

**Burn Treasury DALLA**:
```rust
// Reduce supply by 1M DALLA
Economy::governance_burn(
    Origin::signed(governance_origin),
    1_000_000_000_000  // 1M DALLA
)?;
```

## Future Enhancements

### Planned Features (stable2512+)
- [ ] **Automated Reserve Auditing**: On-chain proof of reserves via ZK proofs
- [ ] **Dynamic Cashback Rates**: Tourism incentives based on seasonal demand
- [ ] **Cross-Chain bBZD**: Bridge bBZD to Ethereum/Polkadot
- [ ] **Merchant Loyalty Programs**: Tiered rewards for repeat customers
- [ ] **Carbon Credits**: Link DALLA to Belize's environmental initiatives

### Integration Roadmap
- **Q1 2026**: Nawal AI for fraud detection in tourism payments
- **Q2 2026**: Kinich Quantum for reserve auditing via quantum-resistant signatures
- **Q3 2026**: Pakit Storage for Central Bank document storage (audit trails)

## Performance Metrics

### Benchmark Results

| Extrinsic | Weight (ns) | DB Reads | DB Writes |
|-----------|-------------|----------|-----------|
| `mint_bbzd` | 35,000,000 | 3 | 2 |
| `redeem_bbzd` | 30,000,000 | 2 | 2 |
| `process_redemption` | 28,000,000 | 2 | 2 |
| `process_tourism_payment` | 45,000,000 | 4 | 3 |
| `burn_dalla` | 25,000,000 | 1 | 1 |

### On-Chain Storage

| Storage Item | Size | Capacity |
|--------------|------|----------|
| `BBZDBalances` | ~100 bytes/account | Millions of accounts |
| `RedemptionRequests` | ~150 bytes/request | Thousands of pending requests |
| `CentralBankReserves` | 16 bytes | Single value |
| `TotalBbzdSupply` | 16 bytes | Single value |

## References

### Documentation
- Architecture: `/docs/architecture/economic-model.md`
- Security Audit: `/docs/security/SECURITY_AUDIT_FINAL.md`
- User Guide: `/docs/user-guides/wallet-guide.md`

### Related Pallets
- Governance: `/belizechain/pallets/governance/`
- Compliance: `/belizechain/pallets/compliance/`
- Oracle: `/belizechain/pallets/oracle/`
- Staking: `/belizechain/pallets/staking/`

### External Resources
- Central Bank of Belize: https://www.centralbank.org.bz/
- Belize Tourism Board: https://www.travelbelize.org/
- Polkadot SDK Docs: https://paritytech.github.io/polkadot-sdk/

---

**Status**: ✅ Production-Ready (stable2512)  
**Last Audit**: January 2026  
**Lines of Code**: 2,140 (lib: 1,051, mock: 159, tests: 930)  
**Test Coverage**: 93% (unit tests)  
**Clippy Warnings**: 0 (pallet-specific)
