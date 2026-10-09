# Compliance Pallet

## Overview

The **Compliance pallet** provides comprehensive regulatory compliance enforcement for BelizeChain, ensuring all participants meet KYC/AML requirements and comply with international standards including FATF recommendations, while maintaining privacy through hash-based verification.

## Purpose

This pallet is a critical component of BelizeChain's sovereign infrastructure, enabling:
- **Regulatory Compliance**: FATF recommendations, AML/CFT, sanctions screening
- **KYC/AML Enforcement**: Multi-level verification for different operations
- **Privacy Protection**: Hash-based verification (no raw PII on-chain)
- **Risk Management**: Transaction monitoring and suspicious activity reporting
- **Audit Trails**: Complete compliance logging for regulatory oversight

## Key Features

### 1. Verification Levels
- **None (L0)**: No verification - severely restricted operations
- **Basic (L1)**: SSN verified via BelizeIdentity - basic operations
- **Standard (L2)**: SSN + Passport - full participation
- **Enhanced (L3)**: SSN + Passport + Biometrics - validator/governance roles
- **Government**: Institutional-level access

### 2. Risk Assessment
- **Low Risk**: Normal operations allowed
- **Medium Risk**: Enhanced monitoring
- **High Risk**: Restricted operations
- **Prohibited**: Account blocked

### 3. Compliance Enforcement Points
- Validator operations (join, nomination, rewards)
- Governance participation (voting, proposals, council)
- Treasury access (spending proposals, fund management)
- Optional token transfer restrictions (based on risk scores)
- Enhanced monitoring for cross-border transactions

### 4. Suspicious Activity Reporting (SAR)
- Unusual transaction volume patterns
- Rapid fund movement detection
- Structuring attempts (avoiding thresholds)
- High-risk jurisdiction transactions
- Money laundering indicators
- Terrorist financing detection
- Sanctions violation alerts

### 5. Sanctions Screening
- OFAC sanctions list integration
- UN sanctions list support
- Account-level sanctions enforcement
- Jurisdiction-based restrictions

## Architecture

### Integration with BelizeIdentity Pallet
The Compliance pallet relies on `pallet-belize-identity` for KYC level verification:
- Queries identity verification status (SSN, Passport, Biometrics)
- Uses hash-based proofs to avoid storing raw PII
- Enforces minimum KYC levels for sensitive operations
- Cross-pallet verification checks

### Storage Items
- **`AccountCompliance`**: Maps AccountId → ComplianceStatus
- **`SuspiciousActivities`**: Maps AccountId → Vec<SuspiciousActivityReport>
- **`ComplianceAuditLog`**: Vec<ComplianceAuditRecord> (fixed-size ring buffer)
- **`SanctionsList`**: Maps AccountId/Jurisdiction → SanctionEntry
- **`WhitelistedAccounts`**: Set<AccountId> (government/institutional exemptions)
- **`AuditLogIndex`**: u32 (ring buffer index)

## Extrinsics

### `verify_account(account, verification_level, risk_assessment)`
Verify an account's compliance status with specific verification and risk levels.

**Parameters:**
- `account`: Target account
- `verification_level`: Required level (None, Basic, Standard, Enhanced, Government)
- `risk_assessment`: Optional RiskLevel override

**Authority:** FSC Compliance Authority

**Events:**
- `AccountVerified(account, verification_level, timestamp)`

**Errors:**
- `UnauthorizedOperation` - Caller lacks authority
- `InsufficientKycLevel` - Account doesn't meet KYC requirements

### `update_risk_level(account, risk_level)`
Update an account's risk assessment level.

**Authority:** FSC Compliance Authority

**Events:**
- `RiskLevelUpdated(account, old_risk, new_risk)`

### `whitelist_account(account)`
Add account to whitelist (bypasses some restrictions).

**Authority:** FSC Compliance Authority

**Use Cases:**
- Government accounts
- Central Bank operations
- Institutional partners

### `restrict_account(account, reason)`
Temporarily restrict account operations pending investigation.

**Authority:** FSC Compliance Authority

**Events:**
- `AccountRestricted(account, reason, timestamp)`

### `lift_restriction(account)`
Remove temporary restrictions from account.

**Authority:** FSC Compliance Authority

### `flag_suspicious_activity(account, activity_type, description)`
Report suspicious activity for AML/CFT compliance.

**Parameters:**
- `activity_type`: Enum (UnusualVolumePattern, RapidFundMovement, Structuring, etc.)
- `description`: BoundedVec<u8, 256> - Details of suspicious activity

**Authority:** FSC Compliance Authority or designated reporters

**Events:**
- `SuspiciousActivityReported(account, activity_type, block_number)`

### `add_sanctions_entry(target, jurisdiction, reason)`
Add account or jurisdiction to sanctions list.

**Authority:** FSC Compliance Authority

**Events:**
- `SanctionsEntryAdded(target, jurisdiction, timestamp)`

### `remove_sanctions_entry(target)`
Remove sanctions entry.

**Authority:** FSC Compliance Authority

## Helper Functions

These are the helpers that actually exist. An earlier revision of this document listed
`can_vote`, `can_propose`, `can_join_council`, `can_validate`, and `can_access_treasury`;
none of those are defined anywhere in the pallet or the runtime.

### `meets_verification_level(account, required_level) -> bool`
Check whether an account's stored verification level meets a threshold. This is a plain
level comparison - it does not check expiry, restriction, or risk.

### `requires_travel_rule(amount) -> bool`
Check whether a transfer amount crosses the travel-rule reporting threshold.

### `is_sanctioned(entity_hash) -> bool`
Check the hash-based sanctions list. Note the argument is the **hashed entity identifier**
(`[u8; 32]`), not an account. Account-level sanctions are queried separately through the
`AccountSanctionsChecker` trait, which the runtime binds to the identity pallet.

### Storage getters
- `compliance_status(account) -> ComplianceStatus` - full status (level, risk, whitelist,
  restricted, last verification timestamp)
- `is_whitelisted(account) -> bool`
- `is_restricted(account) -> bool`
- `audit_records(account) -> Vec<ComplianceAuditRecord>`
- `compliance_stats() -> (total_verified, total_restricted, total_reports, total_sanctions_checks)`

### `create_audit_record(account, action_type, success, details)`
Internal function that appends to the account's audit trail and emits
`AuditRecordCreated`.

### Verification expiry
There is no `is_verification_valid` predicate. Expiry is driven by the
`sync_verification_from_identity()` extrinsic: if the account's last verification is older
than `VerificationValidityPeriod`, it downgrades the level to `None` and restricts the
account until a compliance authority re-verifies it.

### Governance participation
Voting eligibility is **not** exposed by this pallet. The runtime's
`GovernanceComplianceProvider` computes it as `KYC L1 or above && not sanctioned`; the
pallet's status fields are not consulted by Governance.

## Compliance Monitoring

### Automatic Checks
- Cross-border transaction monitoring
- Large transaction alerts (>100,000 DALLA)
- Velocity checks (frequency analysis)
- Pattern detection (structuring attempts)

### Audit Trail
- Per-account bounded vector, capped by the runtime's `MaxAuditRecords` (1000 in the
  runtime, 100 in this pallet's mock)
- Stores: AccountId, ActionType, Timestamp, BlockNumber, Success, Details
- Immutable after creation (compliance requirement)

### Integration Points
Other pallets can use `ComplianceProvider` trait:
```rust
pub trait ComplianceProvider<AccountId> {
    fn can_vote(account: &AccountId, block_number: u64) -> bool;
    fn can_propose(account: &AccountId, block_number: u64) -> bool;
    fn can_validate(account: &AccountId) -> bool;
    fn is_sanctioned(account: &AccountId) -> bool;
    fn get_verification_level(account: &AccountId) -> VerificationLevel;
}
```

## Security & Privacy

### Privacy-First Design
- **No raw PII stored**: All identity data in BelizeIdentity pallet as hashes
- **Minimal required fields**: Only verification status, not underlying data
- **Off-chain KYC**: Verification done externally, attestations stored on-chain
- **Configurable retention**: Audit logs use ring buffer to limit historical data

### Authority Model
- **FSC Compliance Authority**: Designated account with special permissions
- **Council-majority approval support**: Can be configured as council-majority approval wallet
- **Emergency powers**: Account restriction for suspected violations
- **Audit oversight**: All authority actions logged

## Regulatory Standards

### FATF Recommendations
- **Travel Rule**: Cross-border transaction monitoring
- **Customer Due Diligence**: Multi-level KYC verification
- **Suspicious Activity Reporting**: SAR filing via on-chain events
- **Record Keeping**: Complete audit trail

### AML/CFT Compliance
- Transaction monitoring and pattern analysis
- Risk-based approach (Low/Medium/High/Prohibited)
- Sanctions screening (OFAC, UN lists)
- Enhanced due diligence for high-risk accounts

### Belize-Specific Requirements
- Financial Services Commission (FSC) oversight
- Central Bank coordination (bBZD issuance requires compliance)
- Data sovereignty (all records on national infrastructure)
- Audit trail for tax authority integration

## Weight Information

### Benchmarked Operations
- `verify_account`: 25,000,000 + 2 reads + 1 write
- `flag_suspicious_activity`: 30,000,000 + 3 reads + 2 writes
- `add_sanctions_entry`: 20,000,000 + 1 read + 1 write
- `can_vote` (helper): 15,000,000 + 1 read
- `can_validate` (helper): 20,000,000 + 1 read

## Testing

### Unit Tests (`tests.rs`)
- Account verification workflows (L1→L2→L3 progression)
- Risk level updates and enforcement
- Whitelisting/restriction logic
- Suspicious activity reporting
- Sanctions list management
- Helper function validation (can_vote, can_validate, etc.)
- Audit log ring buffer behavior

### Mock Setup (`mock.rs`)
- Test runtime with BelizeIdentity integration
- FSC authority account configuration
- Sample KYC verification scenarios

## Dependencies

### Pallets
- `pallet-belize-identity` - KYC verification data
- `frame-system` - Block numbers, timestamps
- `frame-support` - Traits, storage, events

### External
- None (fully self-contained regulatory logic)

## Future Enhancements
1. **Automated risk scoring** - ML-based transaction analysis
2. **Cross-chain compliance** - Bridge transaction monitoring
3. **Real-time OFAC/UN API integration** - Live sanctions list updates
4. **Regulatory reporting export** - JSON exports for FSC/tax authority
5. **Compliance dashboards** - Read-only UI for regulators

## Files

### Core Implementation
- **`src/lib.rs`** (1,065 lines) - Main pallet logic, extrinsics, helpers
- **`src/mock.rs`** (142 lines) - Test runtime configuration
- **`src/tests.rs`** (313 lines) - Comprehensive unit tests
- **`Cargo.toml`** - Dependencies and features

### Documentation
- **`README.md`** - This file

**Total Lines:** 1,520 lines of production code + tests

---

**Maintained by:** BelizeChain Core Team  
**Last Updated:** January 29, 2026  
**Status:** Production Ready (Polkadot SDK stable2603)
