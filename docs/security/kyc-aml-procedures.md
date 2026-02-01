# KYC/AML Procedures

**Financial Services Commission Oversight • Identity Verification • Compliance Monitoring**

Comprehensive guide to BelizeChain's regulatory compliance framework.

---

## KYC Levels

### Level 0: None (Anonymous)
**Capabilities:**
- Read blockchain data
- Query public information
- No transactions allowed

**Use case:** Public observers, researchers

### Level 1: Basic
**Requirements:**
- Email verification
- Phone number (SMS)
- Basic identity declaration

**Capabilities:**
- Receive DALLA/bBZD (up to 1,000/month)
- Small purchases (<100 DALLA/transaction)
- Cannot send funds

**Verification time:** Instant (automated)

```javascript
// Apply for Basic KYC
await api.tx.compliance.submitKycApplication(
  'Basic',
  [emailProofHash, phoneProofHash],  // Document hashes
).signAndSend(applicant);
```

### Level 2: Verified
**Requirements:**
- Government-issued photo ID (Passport or BelizeID)
- Proof of address (utility bill, bank statement <3 months)
- Selfie with ID
- In-person verification OR video call

**Capabilities:**
- Full send/receive (up to 10,000 DALLA/day)
- Staking (minimum 1,000 DALLA)
- BNS domain registration
- Merchant registration
- BelizeX trading (<50,000 DALLA/day)

**Verification time:** 1-3 business days

```typescript
// Apply for Verified KYC
const idHash = await uploadDocumentToPakit(passport);
const addressProofHash = await uploadDocumentToPakit(utilityBill);
const selfieHash = await uploadDocumentToPakit(selfieWithId);

await api.tx.compliance.submitKycApplication(
  'Verified',
  [idHash, addressProofHash, selfieHash]
).signAndSend(applicant);
```

### Level 3: Enhanced
**Requirements:**
- All Verified requirements
- Background check (criminal record, credit history)
- Biometric scan (fingerprint or facial recognition)
- Source of funds declaration
- Tax compliance certificate

**Capabilities:**
- Unlimited transactions
- Validator operations
- Large BelizeX trades (>50,000 DALLA/day)
- Property registration (LandLedger)
- Government contracts
- Cross-border transfers (>$10,000 equivalent)

**Verification time:** 5-10 business days

**Cost:** 500 DALLA application fee (one-time)

---

## AML Monitoring

### Automated Triggers

#### 1. Large Transaction Threshold
```python
# Trigger AML review for large transactions
LARGE_TX_THRESHOLD = 100_000 * DALLA  # $30,000 @ $0.30/DALLA

def check_large_transaction(tx):
    if tx.amount > LARGE_TX_THRESHOLD:
        compliance_system.flag_for_review(
            account=tx.sender,
            reason="LARGE_TRANSACTION",
            amount=tx.amount,
            recipient=tx.recipient
        )
        
        # Require Enhanced KYC
        if get_kyc_level(tx.sender) < KycLevel.Enhanced:
            raise ComplianceError("Enhanced KYC required for transactions >100K DALLA")
```

#### 2. High Velocity Pattern
```python
# Detect rapid transaction velocity (possible structuring)
def check_transaction_velocity(account):
    recent_txs = get_transactions_last_24h(account)
    
    if len(recent_txs) > 50:
        # >50 transactions in 24h
        compliance_system.flag_for_review(
            account=account,
            reason="HIGH_VELOCITY",
            tx_count=len(recent_txs)
        )
        
        # Temporary hold on account
        freeze_account(account, duration="48h", reason="AML_REVIEW")
```

#### 3. Structuring Detection
```python
# Detect structuring (breaking large amounts into smaller txs)
def detect_structuring(account):
    txs_24h = get_transactions_last_24h(account)
    
    # Check for multiple just-under-threshold transactions
    near_threshold_txs = [
        tx for tx in txs_24h
        if 8_000 * DALLA < tx.amount < 10_000 * DALLA
    ]
    
    if len(near_threshold_txs) >= 5:
        # 5+ transactions just under 10K threshold
        compliance_system.flag_for_review(
            account=account,
            reason="STRUCTURING_PATTERN",
            suspicious_txs=near_threshold_txs
        )
        
        # Immediate freeze
        freeze_account(account, duration="indefinite", reason="SUSPECTED_STRUCTURING")
```

#### 4. High-Risk Jurisdiction
```rust
// Check recipient against high-risk country list
const HIGH_RISK_JURISDICTIONS: &[&str] = &[
    "DPRK", "Iran", "Syria", "Cuba", "Venezuela"
    // FATF blacklist + graylist countries
];

pub fn check_cross_border_transfer(
    sender: &AccountId,
    recipient: &ExternalAddress,
    amount: Balance
) -> Result<(), ComplianceError> {
    let jurisdiction = resolve_jurisdiction(recipient)?;
    
    if HIGH_RISK_JURISDICTIONS.contains(&jurisdiction.as_str()) {
        compliance_system.flag_for_review(
            sender,
            "HIGH_RISK_JURISDICTION",
            recipient,
            jurisdiction
        );
        
        // Require FSC approval
        return Err(ComplianceError::FSCApprovalRequired);
    }
    
    Ok(())
}
```

### Manual Review Process

**FSC Officers** review flagged accounts:

```javascript
// FSC officer reviews flagged account
const flaggedAccounts = await api.query.compliance.flaggedAccounts.entries();

for (const [key, flagData] of flaggedAccounts) {
  const account = key.args[0].toString();
  const { reason, flagged_at, evidence } = flagData.unwrap();
  
  console.log(`Account: ${account}`);
  console.log(`Reason: ${reason}`);
  console.log(`Flagged: ${new Date(flagged_at * 1000).toISOString()}`);
  
  // FSC officer investigates and approves/rejects
  if (investigation_clears_account(account)) {
    await api.tx.compliance.clearFlag(account).signAndSend(fscOfficer);
  } else {
    await api.tx.compliance.freezeAccount(
      account,
      'PERMANENT',
      'AML violation confirmed'
    ).signAndSend(fscOfficer);
  }
}
```

---

## FSC Oversight

### Financial Services Commission Authority

**Powers:**
1. Approve/reject KYC applications
2. Freeze/unfreeze accounts
3. Request transaction history
4. Impose fines (sent to treasury)
5. Revoke licenses (merchant, validator)
6. Emergency protocol changes (security only)

**Limitations:**
- Cannot access private keys
- Cannot reverse transactions (blockchain immutability)
- Subject to judicial review (court oversight)

### FSC Officer Roles

```rust
// Define FSC officer permissions
pub enum FscRole {
    KycReviewer,      // Approve/reject KYC applications
    AmlAnalyst,       // Review flagged transactions
    Supervisor,       // Approve account freezes
    Director,         // Emergency powers
}

pub struct FscOfficer {
    pub account: AccountId,
    pub role: FscRole,
    pub appointed_date: BlockNumber,
    pub approvals_granted: u32,
    pub freezes_enacted: u32
}
```

**Governance:**
- FSC officers appointed by Minister of Finance
- 4-year terms
- Removable by parliamentary vote
- Quarterly public reports on enforcement actions

---

## Data Privacy (GDPR-like)

### Personal Data Handling

**On-chain data (public):**
- Transaction amounts
- Account addresses
- Timestamps
- Block numbers

**Off-chain data (private):**
- Full names
- Addresses
- Government ID numbers
- Biometric data
- Financial history

**Storage:**
```
On-chain: Hashes only (SHA-256)
Off-chain: Encrypted database (AES-256)
  - Access: FSC officers only
  - Logging: All access logged
  - Retention: 7 years (compliance requirement)
```

### Right to be Forgotten

**Process:**
1. User requests account deletion
2. Off-chain PII deleted (after 7-year retention)
3. On-chain account marked as "deleted"
4. Transaction history remains (blockchain immutability)

```javascript
// Request data deletion (GDPR Article 17)
await api.tx.identity.requestDeletion().signAndSend(user);

// FSC processes after 7 years
await api.tx.compliance.deletePii(
  userAccount,
  deletionProof  // Proof of 7-year retention completion
).signAndSend(fscOfficer);
```

**Limitations:**
- Transaction history cannot be deleted (blockchain immutability)
- Amounts/addresses remain public forever
- Only identity linkage is removed

---

## Compliance Reporting

### Quarterly Reports (Public)

**Published on-chain:**
```javascript
// Quarterly compliance report
const report = {
  quarter: "Q1 2026",
  kyc_applications: {
    basic: 5420,
    verified: 1832,
    enhanced: 156
  },
  kyc_approvals: {
    basic: 5398,  // 99.6% approval
    verified: 1654,  // 90.3% approval
    enhanced: 89  // 57.1% approval (strict)
  },
  aml_flags: {
    large_transactions: 42,
    high_velocity: 18,
    structuring: 7,
    high_risk_jurisdiction: 3
  },
  enforcement_actions: {
    warnings: 25,
    temporary_freezes: 12,
    permanent_freezes: 2,
    fines_imposed: 50_000 * DALLA
  }
};

await api.tx.compliance.publishQuarterlyReport(
  JSON.stringify(report)
).signAndSend(fscDirector);
```

### Annual Audit

**Independent auditor:**
- PwC Belize (2026 auditor)
- Full compliance review
- Technology assessment
- Recommendations published

**Audit scope:**
- KYC/AML effectiveness
- Data privacy compliance
- FSC operations
- Technology security

---

## Penalties & Enforcement

### User Violations

| Violation | Penalty | Duration |
|-----------|---------|----------|
| **False KYC information** | Account freeze + 1,000 DALLA fine | Permanent |
| **Structuring transactions** | Account freeze + 10% funds seized | Permanent |
| **High-risk jurisdiction transfer (unauthorized)** | Funds frozen + investigation | Until cleared |
| **Failure to update KYC (annual renewal)** | Transaction limits reduced | Until updated |

### Merchant Violations

| Violation | Penalty | Duration |
|-----------|---------|----------|
| **Accept payments without KYC** | License revoked + 10,000 DALLA fine | Permanent |
| **Money laundering facilitation** | License revoked + funds seized | Permanent |
| **False verification claims** | License suspended | 6 months |

### Validator Violations

| Violation | Penalty | Duration |
|-----------|---------|----------|
| **Privacy breach (leak FL data)** | 50% stake slashed + validator ban | Permanent |
| **Malicious behavior** | 100% stake slashed | Permanent |
| **Downtime (>25% blocks)** | Kicked from validator set | Until fixed |

---

## Integration with Other Pallets

### Identity Pallet
```rust
// KYC check before identity registration
pub fn register_belizeid(
    origin: OriginFor<T>,
    identity_type: IdentityType
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // Require Verified KYC for BelizeID
    let kyc_level = Compliance::get_kyc_level(&who)?;
    ensure!(
        kyc_level >= KycLevel::Verified,
        Error::<T>::InsufficientKycLevel
    );
    
    // Proceed with registration
    Identity::do_register(who, identity_type)?;
}
```

### Economy Pallet
```rust
// KYC check for merchant registration
pub fn register_merchant(
    origin: OriginFor<T>,
    category: MerchantCategory
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // Require Verified KYC minimum
    ensure!(
        Compliance::is_kyc_verified(&who),
        Error::<T>::KycRequired
    );
    
    // Enhanced KYC for high-volume merchants
    if category == MerchantCategory::Retail {
        ensure!(
            Compliance::get_kyc_level(&who) >= KycLevel::Enhanced,
            Error::<T>::EnhancedKycRequired
        );
    }
    
    Economy::do_register_merchant(who, category)?;
}
```

---

## Related Documentation

- [Compliance Pallet API](../developer-guides/pallet-apis-core.md#compliance-pallet)
- [Identity Pallet](../developer-guides/pallet-apis-core.md#identity-pallet)
- [Security Audit Results](./security-audit-results.md)
- [FSC Guidelines](https://fsc.gov.bz) (external)
- [FATF Recommendations](https://www.fatf-gafi.org) (external)
