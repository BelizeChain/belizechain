# 🔐 BelizeChain Validator Requirements

**Version**: 1.0.0  
**Updated**: November 4, 2025  
**Network**: Testnet → Mainnet  

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Technical Requirements](#technical-requirements)
3. [Economic Requirements](#economic-requirements)
4. [Operational Requirements](#operational-requirements)
5. [Security Requirements](#security-requirements)
6. [Compliance Requirements](#compliance-requirements)
7. [Performance Standards](#performance-standards)
8. [Application Process](#application-process)

---

## Overview

BelizeChain validators are critical infrastructure providers for the network. As a sovereign blockchain for Belize, validators must meet stringent technical, economic, and operational standards.

### Validator Responsibilities

✅ **Block Production**: Produce blocks when selected by consensus  
✅ **Finalization**: Vote on block finality (GRANDPA)  
✅ **Governance**: Participate in on-chain governance  
✅ **Network Health**: Maintain uptime and peer connectivity  
✅ **Security**: Protect session keys and validator infrastructure  
✅ **Compliance**: Adhere to Belizean regulations (if applicable)  

### Validator Benefits

💰 **Staking Rewards**: ~12-18% APY (dynamic based on stake)  
💎 **Transaction Fees**: Share of network transaction fees  
🏛️ **Governance Rights**: Vote on protocol upgrades  
🌍 **Sovereignty**: Support Belize's digital infrastructure  
📈 **Growth Potential**: Early validator privileges  

---

## Technical Requirements

### Hardware Specifications

#### Minimum Requirements (Testnet)

```yaml
CPU: 4 cores (2.0 GHz+)
RAM: 16 GB
Storage: 250 GB NVMe SSD
Network: 50 Mbps (symmetrical)
Uptime: 95%+ (testnet allowance)
```

#### Recommended Requirements (Mainnet)

```yaml
CPU: 8 cores (3.0 GHz+)
RAM: 32 GB
Storage: 1 TB NVMe SSD (RAID 1)
Network: 100 Mbps (symmetrical)
Uptime: 99.5%+ (mandatory)
Redundancy: Hot failover node (optional but recommended)
```

### Software Requirements

```bash
# Operating System
Ubuntu 22.04 LTS (preferred)
Debian 11+ (supported)
RHEL 8+ (supported)

# Node Software
BelizeChain Node v1.0.0+
Rust 1.75+
Docker (optional)

# Monitoring
Prometheus + Grafana
Systemd (for service management)

# Security
Firewall (ufw/iptables)
SSH key-based authentication
Fail2ban (recommended)
```

### Network Configuration

```yaml
Ports Required:
  - 30333 (P2P) - MUST be publicly accessible
  - 9944 (RPC) - Internal only (localhost)
  - 9615 (Prometheus) - Internal network only

Firewall Rules:
  - Allow inbound 30333 from 0.0.0.0/0
  - Block 9944, 9615 from public internet
  - Rate limiting on P2P port (DDoS protection)

DNS (Recommended):
  - A record: validator.yourdomain.com → Public IP
  - PTR record: Reverse DNS setup
```

---

## Economic Requirements

### Staking Requirements

| Network | Minimum Bond | Recommended Bond | Maximum Nominators |
|---------|--------------|------------------|-------------------|
| **Testnet** | 10,000 DALLA | 50,000 DALLA | 256 |
| **Mainnet** | 100,000 DALLA | 500,000 DALLA | 256 |

### Tokenomics

- **Total Supply**: 1,000,000,000 DALLA (1 billion)
- **Validator Rewards**: 10% annual inflation → validator pool
- **Commission**: Validators set 0-100% commission on rewards
- **Slashing**: Up to 100% of stake for severe misbehavior

### Cost Estimates

#### Monthly Operational Costs

```yaml
Testnet:
  Server: $100-200 (VPS/Dedicated)
  Network: $50-100 (bandwidth)
  Monitoring: $20-50 (tools)
  Total: $170-350/month

Mainnet:
  Primary Server: $300-500 (bare metal)
  Backup Server: $300-500 (optional)
  Network: $100-200 (enterprise)
  Monitoring: $50-100 (premium tools)
  Security: $100-200 (audits, DDoS protection)
  Total: $850-1,500/month
```

#### Break-Even Analysis (Mainnet)

```
Scenario: 100,000 DALLA stake, 15% APY, 10% commission

Annual Rewards: 100,000 × 0.15 = 15,000 DALLA
Commission (10%): 1,500 DALLA
Nominator Rewards: 13,500 DALLA

Monthly Costs: $1,000
Required DALLA Price: $8 USD (to cover costs)

Current Price: ~$12 USD (example)
Monthly Profit: $500+ (after costs)
```

---

## Operational Requirements

### Uptime Standards

| Metric | Testnet | Mainnet |
|--------|---------|---------|
| **Minimum Uptime** | 95% | 99.5% |
| **Max Downtime/Month** | 36 hours | 3.6 hours |
| **Block Miss Tolerance** | 5% | 0.5% |
| **Slash Threshold** | None (testnet) | 10% unresponsiveness |

### Monitoring Requirements

All validators MUST implement:

1. **Node Health Monitoring**
   - Block production rate
   - Peer connectivity (min 25 peers)
   - Disk usage alerts (<80% full)
   - Memory usage alerts (<90% full)

2. **Network Monitoring**
   - Chain synchronization status
   - Finality lag (<10 blocks)
   - Fork detection

3. **Alert System**
   - Email/SMS for critical issues
   - Pagerduty or similar (mainnet)
   - Response time: <15 minutes (mainnet)

### Backup & Disaster Recovery

**Required**:
- Daily database backups (chain state)
- Session key backups (encrypted, offline)
- Configuration backups (chain spec, systemd files)
- Recovery runbook documented

**Recommended**:
- Hot failover node (automatic switchover)
- Geographic redundancy (different datacenter)
- Backup internet connection

---

## Security Requirements

### Key Management

1. **Session Keys**
   - Generate on validator node (NEVER export)
   - Rotate after security incidents
   - Store backup offline (encrypted USB)

2. **Stash/Controller Keys**
   - Stash: Hardware wallet (Ledger/Trezor) REQUIRED
   - Controller: Hot wallet for management
   - NEVER use stash key online

3. **Node Key (P2P)**
   - Generate unique key per node
   - Backup securely
   - Rotate annually

### Infrastructure Security

```yaml
Required:
  - SSH key authentication (password auth disabled)
  - Firewall (only required ports open)
  - Automatic security updates
  - Fail2ban (brute force protection)
  - Non-root user for node process

Recommended:
  - VPN for remote access
  - DDoS protection (Cloudflare, AWS Shield)
  - Intrusion detection (Snort, Suricata)
  - Regular security audits
  - Hardware security modules (HSM) for mainnet
```

### Incident Response

Validators must:
- Monitor security advisories (Discord #security channel)
- Apply critical patches within 24 hours
- Report security incidents within 1 hour
- Participate in emergency network upgrades

---

## Compliance Requirements

### Legal Compliance (Mainnet Validators)

**Belizean Validators** (preferred):
- Must register with Belize Financial Services Commission (FSC)
- Business license required
- Tax compliance (GST registration if applicable)
- KYC submission to BelizeChain Foundation

**International Validators**:
- Entity registration in home jurisdiction
- KYC/AML compliance documentation
- Tax treaty documentation (if applicable)
- No restrictions under Belize sanctions law

### Data Sovereignty

- Validators MUST NOT censor transactions
- Validators MUST NOT discriminate against users
- Validators SHOULD host nodes in jurisdictions with strong privacy laws
- Validators MUST comply with Belizean data protection laws

### Reporting Requirements

- Quarterly performance reports (mainnet)
- Annual compliance attestation
- Security incident reports (immediate)
- Governance participation records

---

## Performance Standards

### Block Production

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| **Block Production Rate** | 100% | <99% | <95% |
| **Missed Blocks/Era** | 0 | 1-5 | >5 |
| **Finality Vote Participation** | 100% | <99% | <95% |

### Network Performance

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| **Peer Count** | 50+ | 25-49 | <25 |
| **Sync Speed** | Real-time | <10 blocks lag | >50 blocks lag |
| **Response Time** | <100ms | 100-500ms | >500ms |

### Governance Participation

- **Minimum**: Vote on 50% of referenda
- **Recommended**: Vote on 80%+ of referenda
- **Incentivized**: Bonus rewards for consistent participation

---

## Application Process

### Step 1: Self-Assessment

Use our [Validator Readiness Checklist](VALIDATOR_CHECKLIST.md):
- ☐ Hardware meets requirements
- ☐ Budget covers operational costs
- ☐ Technical skills verified
- ☐ Security practices documented
- ☐ Legal compliance confirmed

### Step 2: Testnet Registration

1. **Setup testnet validator** (see [TESTNET_DEPLOYMENT.md](../deployment/TESTNET_DEPLOYMENT.md))
2. **Run for 30+ days** with >95% uptime
3. **Submit performance report**:
   ```
   Validator Name: YourName
   Testnet Address: 5Abc...xyz
   Uptime: 98.5%
   Missed Blocks: 12/14,400
   Peer Count Avg: 45
   ```

### Step 3: KYC Submission

Submit to `validators@belizechain.org`:
- Government-issued ID (passport/driver's license)
- Proof of address (utility bill <3 months old)
- Entity documentation (if registering as business)
- Background check authorization

### Step 4: Technical Interview

- 30-minute video call with BelizeChain technical team
- Topics: Substrate knowledge, security practices, incident response
- Live demo of validator setup

### Step 5: Mainnet Onboarding

Upon approval:
- Receive **Mainnet Validator Badge** (NFT)
- Invited to **Validator Discord Channel**
- Listed on **Validator Registry** (belizechain.org/validators)
- Technical support package activated

### Timeline

- **Testnet Phase**: 30 days minimum
- **KYC Processing**: 5-10 business days
- **Technical Interview**: Scheduled within 7 days
- **Mainnet Approval**: 2-5 days post-interview
- **Total**: ~45-60 days

---

## Validator Tiers

### Tier 1: Genesis Validators (21 slots)
- Selected before mainnet launch
- Guaranteed slot in active set (Year 1)
- 5% commission cap (Year 1)
- Founding validator NFT
- **Status**: Applications open

### Tier 2: Early Validators (79 slots)
- Onboarded within 6 months of mainnet
- Priority selection for active set
- Early validator badge
- **Status**: Applications open

### Tier 3: Community Validators (Unlimited)
- Standard validator program
- Compete for active set slots
- Full rewards and benefits
- **Status**: Always open

---

## Incentive Programs

### Testnet Bug Bounty

Find critical issues, earn rewards:
- **Critical**: 50,000 DALLA (consensus, security)
- **High**: 10,000 DALLA (performance, data loss)
- **Medium**: 2,500 DALLA (UX, minor bugs)

Submit via: `security@belizechain.org`

### Performance Bonuses (Mainnet)

- **Perfect Uptime** (30 days): 1,000 DALLA bonus
- **Governance Leader** (most votes): 5,000 DALLA/quarter
- **Community Support** (forum/Discord help): 500 DALLA/month

### Referral Program

- Refer qualified validators: 5% of their Year 1 commission
- Max 10 referrals per validator

---

## Disqualification Criteria

Validators may be removed for:
- **Double-signing**: Producing two blocks at same height (auto-slash)
- **Persistent downtime**: <95% uptime over 30 days (mainnet)
- **Compliance failure**: KYC fraud, legal violations
- **Malicious behavior**: Censorship, collusion, attacks
- **Non-participation**: <50% governance votes over 90 days

**Testnet**: Warning → probation → removal  
**Mainnet**: Slashing + removal (immediate for critical issues)

---

## Support Resources

### Documentation
- [Testnet Deployment Guide](../deployment/TESTNET_DEPLOYMENT.md)
- [Monitoring Setup](../monitoring/PROMETHEUS_SETUP.md)
- [Security Best Practices](../security/VALIDATOR_SECURITY.md)
- [Disaster Recovery](../operations/DISASTER_RECOVERY.md)

### Community
- **Discord**: https://discord.gg/belizechain (#validators channel)
- **Forum**: https://forum.belizechain.org/c/validators
- **Telegram**: @BelizeChainValidators

### Technical Support
- **Email**: validators@belizechain.org
- **Emergency**: +501-XXX-XXXX (24/7 hotline for mainnet)
- **Office Hours**: Weekly Wednesday 10am EST (Discord voice)

---

## Next Steps

1. ✅ **Review requirements** (this document)
2. ✅ **Setup testnet node**: [TESTNET_DEPLOYMENT.md](../deployment/TESTNET_DEPLOYMENT.md)
3. ✅ **Learn incentive structure**: [INCENTIVES.md](INCENTIVES.md)
4. ✅ **Complete onboarding**: [ONBOARDING.md](ONBOARDING.md)
5. ✅ **Join validator community**: [Discord](https://discord.gg/belizechain)

---

**Questions?** Join our [validator Discord channel](https://discord.gg/belizechain) or email `validators@belizechain.org`

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
