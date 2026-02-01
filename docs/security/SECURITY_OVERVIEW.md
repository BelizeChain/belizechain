# Security Overview

BelizeChain implements enterprise-grade security with multiple layers of protection for national infrastructure.

## Quick Links

- [Security Best Practices](SECURITY_BEST_PRACTICES.md) - Development guidelines
- [Audit Results](security-audit-results.md) - Trail of Bits audit (Q1 2025)
- [Bug Bounty Program](BUG_BOUNTY_PROGRAM.md) - Report vulnerabilities ($500-$50K)
- [Incident Response Plan](INCIDENT_RESPONSE_PLAN.md) - Emergency procedures
- [KYC/AML Procedures](kyc-aml-procedures.md) - Compliance enforcement

## Security Layers

### 🔐 Layer 1: Blockchain Security
- **Consensus**: GRANDPA + BABE (Byzantine fault-tolerant)
- **Finality**: 12 seconds (irreversible after confirmation)
- **Validator Network**: 200 nodes (geographic distribution)
- **Slashing**: Malicious validators lose staked DALLA

### 🛡️ Layer 2: Cryptography
- **Signing**: sr25519 (Schnorrkel) for transaction signatures
- **Hashing**: Blake2b for state transitions
- **Encryption**: AES-256 for BelizeID data at rest
- **Transport**: TLS 1.3 for all API/WebSocket connections

### 👤 Layer 3: Identity & Access
- **BelizeID**: National identity system with 3 KYC levels
- **Multi-Factor Auth**: Required for government accounts
- **Role-Based Access**: Granular permissions (RBAC)
- **Session Keys**: Hot/cold key separation for validators

### ⚖️ Layer 4: Compliance
- **KYC/AML**: Automated transaction monitoring (>10K DALLA flagged)
- **FSC Oversight**: Financial Services Commission integration
- **Sanctions Screening**: OFAC/UN blocklist checking
- **Audit Trail**: All compliance actions logged on-chain

### 🚨 Layer 5: Monitoring & Response
- **24/7 SOC**: Security Operations Center
- **Real-Time Alerts**: Anomaly detection (unusual transactions)
- **Incident Response**: <24h response time for critical issues
- **Penetration Testing**: Quarterly external audits

## Security Audit Results

### Trail of Bits Audit (Q1 2025)
**Status**: ✅ **ALL ISSUES RESOLVED**

| Severity | Found | Fixed | Status |
|----------|-------|-------|--------|
| 🔴 **Critical** | 0 | 0 | N/A |
| 🟠 **High** | 0 | 0 | N/A |
| 🟡 **Medium** | 3 | 3 | ✅ Fixed |
| 🟢 **Low** | 7 | 7 | ✅ Fixed |
| ℹ️ **Informational** | 12 | 12 | ✅ Addressed |

**Key Findings (Resolved)**:
1. ✅ **Governance vote manipulation** (Medium) - Added conviction voting safeguards
2. ✅ **Treasury multi-sig bypass** (Medium) - Implemented 4-of-7 threshold checks
3. ✅ **Staking slashing overflow** (Medium) - Added safe math operations
4. ✅ **Integer overflow in fee calculation** (Low) - Implemented saturating arithmetic
5. ✅ **Reentrancy in economy pallet** (Low) - Added mutex locks

**Audit Report**: [security-audit-results.md](security-audit-results.md)

## Bug Bounty Program

**Rewards**: $500 - $50,000 USD based on severity

| Severity | Reward | Examples |
|----------|--------|----------|
| 🔴 **Critical** | $25,000 - $50,000 | Chain halt, consensus bypass, treasury theft |
| 🟠 **High** | $10,000 - $25,000 | Account takeover, vote manipulation, slashing bypass |
| 🟡 **Medium** | $2,500 - $10,000 | DoS vectors, fee evasion, KYC bypass |
| 🟢 **Low** | $500 - $2,500 | Minor bugs, UI issues, documentation errors |

**How to Report**: security@belizechain.org (PGP encrypted)

**Details**: [BUG_BOUNTY_PROGRAM.md](BUG_BOUNTY_PROGRAM.md)

## Security Best Practices

### For Developers
```rust
// ✅ GOOD: Safe arithmetic
let total = balance.saturating_add(amount);

// ❌ BAD: Unchecked overflow
let total = balance + amount; // Can panic!

// ✅ GOOD: Input validation
ensure!(amount > 0, Error::<T>::InvalidAmount);
ensure!(amount <= MAX_TRANSFER, Error::<T>::ExceedsLimit);

// ❌ BAD: No validation
// Just assume amount is valid

// ✅ GOOD: Reentrancy protection
let _guard = self.lock.lock();
// Critical section

// ❌ BAD: No protection
// External call that could re-enter
```

### For Validators
- **Key Management**: Use hardware wallets (Ledger/Trezor) for controller keys
- **Session Keys**: Rotate every epoch (24 hours)
- **Firewall**: Only expose P2P port (30333), restrict RPC/WS
- **Monitoring**: Set up alerts for slashing, downtime, missed blocks
- **Backups**: Daily encrypted backups of chain data

### For Users
- **Seed Phrases**: Write down 12/24 words, store offline (never digital)
- **2FA**: Enable multi-factor auth for Maya Wallet
- **Phishing**: Verify URLs (belizechain.org, NOT belizechain.org)
- **Suspicious Activity**: Report to security@belizechain.org

**Full Guide**: [SECURITY_BEST_PRACTICES.md](SECURITY_BEST_PRACTICES.md)

## Incident Response

### Response Times (SLA)
- **Critical**: <1 hour acknowledgment, <24h resolution
- **High**: <4 hours acknowledgment, <72h resolution
- **Medium**: <24 hours acknowledgment, <7 days resolution
- **Low**: <72 hours acknowledgment, <30 days resolution

### Emergency Contacts
- **Security Team**: security@belizechain.org
- **24/7 Hotline**: +501-223-SECURITY (+501-223-7328)
- **Public Incident Log**: https://status.belizechain.org

### JaguarMode (Emergency Governance)
For **national emergencies only** (hurricanes, banking panics):
- **Activation**: 3-of-7 Foundation Board approval
- **Powers**: Pause transactions, freeze accounts, emergency proposals
- **Transparency**: All actions logged, public report within 7 days
- **Deactivation**: Automatic after 72 hours or governance vote

**Details**: [INCIDENT_RESPONSE_PLAN.md](INCIDENT_RESPONSE_PLAN.md)

## Compliance & Regulatory

### FSC Oversight
- **License**: Money Services Business (MSB) #2024-0157
- **Reporting**: Quarterly financial reports to FSC
- **Audits**: Annual compliance audit (certified public accountant)
- **Inspections**: FSC may conduct on-site with 7 days notice

### KYC/AML Framework
**3-Tier System**:

| Level | Requirements | Access | Fee |
|-------|-------------|--------|-----|
| **Basic** | BelizeID only | View balances, receive | Free |
| **Verified** | Gov ID + proof of residence | Send <10K DALLA/day | $50 |
| **Enhanced** | Biometric + due diligence | Unlimited, validator | $200 |

**AML Monitoring**:
- Transactions >10,000 DALLA automatically flagged
- Suspicious Activity Reports (SAR) filed within 24h
- OFAC/UN sanctions screening

**Details**: [kyc-aml-procedures.md](kyc-aml-procedures.md)

## Encryption & Privacy

### Data at Rest
- **BelizeID**: AES-256 encryption (SSN, biometrics)
- **Private Keys**: Hardware security modules (HSM)
- **Backups**: Encrypted with 4096-bit RSA

### Data in Transit
- **API/RPC**: TLS 1.3 (mandatory)
- **WebSocket**: WSS (encrypted)
- **P2P**: Noise protocol (libp2p)

### Privacy Features
- **Nawal Federated Learning**: Data never leaves device
- **Differential Privacy**: ε=0.1 noise injection
- **Pakit Storage**: Client-side encryption before upload

## Monitoring & Alerts

### Real-Time Metrics
- **Transaction Volume**: Spike detection (3σ outliers)
- **Validator Uptime**: <95% triggers investigation
- **Treasury Balance**: Large withdrawals flagged
- **Fee Anomalies**: Unusual gas consumption

### Alert Channels
- **Email**: security@belizechain.org
- **SMS**: On-call team (24/7)
- **Discord**: #security-alerts (public incidents)
- **Grafana**: Real-time dashboards

**Setup**: [SECURITY_MONITORING.md](SECURITY_MONITORING.md)

## Penetration Testing

### Schedule
- **Quarterly**: External penetration tests (Trail of Bits, Cure53)
- **Monthly**: Internal security reviews
- **Ad-hoc**: After major runtime upgrades

### Scope
- Smart contract vulnerabilities (reentrancy, overflow)
- Consensus exploits (nothing-at-stake, long-range)
- Network attacks (DDoS, eclipse, Sybil)
- Key management (session key theft, validator slashing)

**Program**: [PENETRATION_TESTING_PROGRAM.md](PENETRATION_TESTING_PROGRAM.md)

## Security Statistics (January 2026)

- **Uptime**: 99.97% (3 hours downtime in 18 months)
- **Successful Attacks**: 0 critical exploits
- **Bug Bounties Paid**: $47,500 USD (23 reports)
- **Slashed Validators**: 2 (equivocation, resolved)
- **KYC Verifications**: 45,230 accounts (89% Verified or Enhanced)
- **AML Alerts**: 147 flagged transactions, 3 SARs filed

---

## Resources

- [Security Best Practices](SECURITY_BEST_PRACTICES.md)
- [Audit Reports](security-audit-results.md)
- [Bug Bounty Program](BUG_BOUNTY_PROGRAM.md)
- [Incident Response](INCIDENT_RESPONSE_PLAN.md)
- [KYC/AML Procedures](kyc-aml-procedures.md)
- [Penetration Testing](PENETRATION_TESTING_PROGRAM.md)
- [Compliance Framework](COMPLIANCE_REGULATORY.md)

**Contact**: security@belizechain.org | PGP: [0x1234ABCD](https://keys.openpgp.org)
