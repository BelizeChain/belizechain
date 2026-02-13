# BelizeChain Bug Bounty Program

**Program Name:** BelizeChain Security Bounty  
**Launch Date:** December 2025 (Post-Audit)  
**Platform:** HackerOne (Primary) + Immunefi (Secondary)  
**Total Bounty Pool:** $500,000 (First Year)  
**Maximum Bounty:** $100,000 (Critical Vulnerabilities)  

---

## 1. Program Overview

Welcome to the BelizeChain Bug Bounty Program! We're committed to building the most secure sovereign blockchain infrastructure. This program rewards security researchers who discover and responsibly disclose vulnerabilities in BelizeChain's runtime, smart contracts, and infrastructure.

**Why This Program Matters:**
- **National Security:** BelizeChain underpins Belize's digital economy
- **Financial Sovereignty:** Protects citizen funds (DALLA/bBZD)
- **Land Registry:** Secures property ownership records
- **Identity System:** Safeguards national identity data (SSN, passports)
- **Cross-Chain Bridges:** Prevents multi-million dollar exploits

**Our Commitment:**
- ✅ Fair, transparent reward system
- ✅ Fast triage (<48 hours for critical)
- ✅ No legal action for good-faith research
- ✅ Public recognition (Hall of Fame)
- ✅ Direct communication with core team

---

## 2. Scope

### 2.1 In-Scope Assets (✅ Eligible for Bounties)

#### **Blockchain Runtime (Polkadot Substrate)**
- **13 Custom Pallets:**
  - `pallet-belize-economy` - DALLA/bBZD tokens, multi-sig treasury
  - `pallet-belize-identity` - BelizeID, SSN validation, KYC
  - `pallet-belize-governance` - District councils, referendums
  - `pallet-belize-compliance` - KYC/AML, FSC oversight
  - `pallet-belize-staking` - PoUW consensus, validator rewards
  - `pallet-belize-oracle` - Merchant verification (tourism cashback eligibility)
  - `pallet-payroll` - Enterprise payroll, departments, deductions
  - `pallet-interoperability` - Cross-chain bridges (Ethereum/Polkadot)
  - `pallet-belizex` - DEX, liquidity pools
  - `pallet-land-ledger` - Property registry
  - `pallet-belize-consensus` - Proof of Useful Work
  - `pallet-quantum` - Quantum workload orchestration
  - `pallet-community` - Community governance
  
- **System Pallets:**
  - `frame-system`, `pallet-balances`, `pallet-timestamp`
  - `pallet-aura`, `pallet-grandpa` (consensus)
  - `pallet-transaction-payment`, `pallet-sudo`

#### **Smart Contracts (ink! WASM)**
- **PSP22 Token Contract** (`contracts/defi/tokens/lib.rs`)
  - Token minting, burning, transfers
  - Allowance mechanism
  - Access control
  
- **AMM Pool Contract** (`contracts/defi/amm/lib.rs`)
  - Liquidity provision (add/remove)
  - Token swaps (constant product formula)
  - LP token management
  - Fee distribution
  
- **Lending Protocol Contract** (`contracts/defi/lending/lib.rs`)
  - Collateralized borrowing
  - Interest accrual
  - Liquidation mechanism
  - Oracle price integration
  
- **Ethereum Bridge Contract** (`contracts/defi/bridges/lib.rs`)
  - ERC-20 ↔ PSP22 transfers
  - 3-of-5 validator multi-sig
  - Merkle proof verification
  - Daily limits, emergency pause
  
- **Polkadot XCM Bridge Contract** (`contracts/defi/bridges/xcm/lib.rs`)
  - XCM v3 message handling
  - Reserve asset transfers
  - Teleport transfers
  - 5-of-9 relayer multi-sig
  - Parachain registry

#### **Node Infrastructure**
- **RPC Endpoints:**
  - `https://rpc.belizechain.org`
  - `wss://ws.belizechain.org`
  
- **P2P Networking:**
  - Libp2p peer discovery
  - Block propagation
  - Transaction gossip
  
- **Database:**
  - State root verification
  - Storage integrity
  - Snapshot/backup mechanisms

#### **Cross-Chain Infrastructure**
- **Ethereum Bridge:**
  - Validator relayers (5 nodes)
  - Multi-sig contracts (Ethereum side)
  - Withdrawal proof verification
  
- **Polkadot XCM Bridge:**
  - Relayer network (9 nodes)
  - Parachain connections (Acala, Moonbeam, Astar)
  - Asset registry

#### **Web Interfaces (UI Portals)**
- **Maya Wallet** (`ui/maya-wallet`) - Citizen wallet
- **Blue Hole Portal** (`ui/blue-hole-portal`) - Government dashboard
- **Winik Governance** (`ui/winik-governance`) - Voting interface
- **Pek Business** (`ui/pek-business`) - Merchant POS
- **Gúbida Validator** (`ui/gubida-validator`) - Validator dashboard
- **Kijka Explorer** (`ui/kijka-explorer`) - Block explorer

**Live Deployments:**
- **Mainnet:** `https://mainnet.belizechain.org`
- **Testnet:** `https://testnet.belizechain.org`
- **Staging:** `https://staging.belizechain.org`

---

### 2.2 Out-of-Scope Assets (❌ NOT Eligible for Bounties)

**Excluded Components:**
- **Nawal AI** (`nawal/`) - Separate bug bounty program (contact: ai-security@belizechain.org)
- **Kinich Quantum** (`kinich/`) - Separate program (quantum-security@belizechain.org)
- **Pakit Storage** (`pakit/`) - Separate program (storage-security@belizechain.org)
- **Third-Party Dependencies:**
  - Substrate framework (report to Parity Technologies)
  - Polkadot.js library (report to Web3 Foundation)
  - IPFS/Arweave (report to respective projects)

**Out-of-Scope Vulnerabilities:**
- **Low-Impact Issues:**
  - Cosmetic UI bugs (wrong colors, typos)
  - Documentation errors
  - Missing rate limiting (unless DoS proven)
  - SSL/TLS configuration issues (use Mozilla Observatory)
  
- **Known Issues:**
  - Public testnet faucet abuse (by design)
  - Validator centralization (5 validators in testnet)
  - Gas price volatility (expected behavior)
  
- **Theoretical Attacks:**
  - 51% attacks (requires controlling majority of validators)
  - Quantum computing attacks (CRYSTALS-Dilithium is post-quantum resistant)
  - Social engineering (out-of-scope)
  - Physical access attacks (data center security)

**Recently Patched:**
- See `/docs/security/PATCHED_VULNERABILITIES.md` for list of fixed issues

---

### 2.3 Attack Scope (What Can You Do?)

**Allowed Actions:**
- ✅ Create test accounts on testnet (unlimited)
- ✅ Deploy smart contracts on testnet
- ✅ Submit transactions (non-malicious on mainnet)
- ✅ Query RPC endpoints (rate-limited)
- ✅ Analyze on-chain data (all public)
- ✅ Reverse-engineer binaries (open-source anyway)
- ✅ Fuzz test smart contracts (local or testnet)
- ✅ Social engineering SIMULATIONS (inform us first)

**Prohibited Actions:**
- ❌ Exploit vulnerabilities on mainnet (use testnet!)
- ❌ DoS attacks on mainnet infrastructure
- ❌ Spam transactions (>100 tx/min triggers auto-ban)
- ❌ Attempt to access government KYC data
- ❌ Phishing attacks on users (no real damage)
- ❌ Physical intrusion into data centers
- ❌ Bribery of validators or core team

**Gray Area (Contact Us First):**
- 🔸 Penetration testing node infrastructure (requires authorization)
- 🔸 Testing bridge validators (could disrupt operations)
- 🔸 Load testing RPC endpoints (coordinate window)
- 🔸 Social engineering core team (requires consent)

---

## 3. Reward Tiers

### 3.1 Severity Classification

**How We Determine Severity:**

We use CVSS v3.1 (Common Vulnerability Scoring System) with blockchain-specific adjustments:

| Severity | CVSS Score | Economic Impact | Exploitability | Example |
|----------|------------|-----------------|----------------|---------|
| **Critical** | 9.0-10.0 | >$1M loss | Easy (public exploit) | Treasury draining, unlimited token minting |
| **High** | 7.0-8.9 | $100k-$1M | Medium (requires knowledge) | Validator reward manipulation, bridge double-spending |
| **Medium** | 4.0-6.9 | $10k-$100k | Hard (requires resources) | Front-running exploits, oracle manipulation |
| **Low** | 0.1-3.9 | <$10k | Very hard (unlikely) | Gas optimization, UI bugs, rate limiting bypass |

**Adjusted Factors:**
- **National Infrastructure:** +1 severity level if impacts government operations
- **PII Exposure:** +1 severity level if exposes SSN/passport data
- **Cross-Chain Risk:** +1 severity level if affects multiple chains

---

### 3.2 Reward Schedule

#### **Critical Vulnerabilities ($50,000 - $100,000)**

**Examples:**
- **$100,000:** Unlimited DALLA minting exploit
- **$100,000:** Treasury multi-sig bypass (4-of-7 threshold)
- **$90,000:** Bridge double-spending (Ethereum or Polkadot)
- **$80,000:** Consensus takeover (51% attack bypass)
- **$75,000:** KYC database extraction (all SSNs)
- **$70,000:** Validator private key exposure (all validators)
- **$60,000:** LandLedger title theft (unauthorized ownership transfer)
- **$50,000:** Smart contract reentrancy (full pool draining)

**Criteria:**
- Direct loss of user funds (>$1M at risk)
- Complete system compromise (mainnet shutdown)
- National security impact (government operations)
- PII exposure (>10,000 citizens)

**Recent Critical Bounties (Examples from Other Chains):**
- Ronin Bridge exploit: $625M stolen (NO bounty, was exploited)
- Poly Network hack: $600M stolen, returned (NO bounty)
- Wormhole bridge: $320M stolen (should have been $100k bounty)

---

#### **High Vulnerabilities ($10,000 - $50,000)**

**Examples:**
- **$50,000:** Validator reward inflation (>10x normal rewards)
- **$40,000:** Governance vote manipulation (bypass voting period)
- **$35,000:** Oracle price manipulation (flash loan attack)
- **$30,000:** Bridge relayer collusion (3-of-5 threshold bypass)
- **$25,000:** Smart contract front-running (>$100k MEV)
- **$20,000:** Identity spoofing (fake BelizeID creation)
- **$15,000:** Payroll manipulation (unauthorized salary increase)
- **$10,000:** DEX liquidity pool draining (single pool)

**Criteria:**
- Significant economic loss ($100k-$1M)
- Major system disruption (services down >1 hour)
- Unauthorized privilege escalation
- Limited PII exposure (100-10,000 citizens)

---

#### **Medium Vulnerabilities ($1,000 - $10,000)**

**Examples:**
- **$10,000:** RPC DoS attack (requires 1000+ servers)
- **$8,000:** Cross-chain message spoofing (limited impact)
- **$6,000:** Smart contract gas griefing (2x cost increase)
- **$5,000:** Oracle staleness exploit (<$10k loss)
- **$4,000:** Rate limiting bypass (partial)
- **$3,000:** Front-end XSS (no fund loss)
- **$2,000:** Transaction censorship (specific account)
- **$1,000:** Access control issue (read-only data)

**Criteria:**
- Limited economic loss ($10k-$100k)
- Moderate disruption (services degraded, not down)
- Requires significant resources (1000+ accounts)
- No PII exposure

---

#### **Low Vulnerabilities ($100 - $1,000)**

**Examples:**
- **$1,000:** Gas optimization (50%+ savings on popular function)
- **$500:** UI bug (displays wrong balance, no loss)
- **$300:** Documentation vulnerability (incorrect security guidance)
- **$200:** Cosmetic bug (broken CSS, minor UX issue)
- **$100:** Code quality issue (unused imports, dead code)

**Criteria:**
- No economic loss
- Minor inconvenience
- Informational severity

---

### 3.3 Bonus Multipliers

**Quality Bonus (up to 2x):**
- **Proof-of-Concept:** Runnable exploit code (+20%)
- **Fix Recommendation:** Code patch provided (+20%)
- **Comprehensive Report:** Detailed writeup, diagrams (+10%)
- **Video Demonstration:** Screen recording of exploit (+10%)
- **Multiple Variants:** Show 3+ ways to exploit (+20%)
- **Regression Test:** Automated test to prevent re-introduction (+20%)

**Example:**
- Base Critical Bounty: $50,000
- + POC: $10,000 (20%)
- + Fix: $10,000 (20%)
- + Report: $5,000 (10%)
- **Total: $75,000** (1.5x multiplier)

**First-to-Report Bonus:**
- If multiple researchers submit the same vulnerability, the first submission gets full bounty
- Subsequent duplicates: 10% of bounty (for independent discovery)

**Coordinated Disclosure Bonus (+10%):**
- Wait for 90-day embargo before public disclosure
- Work with us on blog post/disclosure
- Present at BelizeChain Security Conference

---

## 4. Submission Process

### 4.1 How to Submit

**Step 1: Verify Vulnerability**
- Reproduce on testnet (mandatory)
- Document steps clearly
- Capture screenshots/videos
- DO NOT exploit on mainnet

**Step 2: Prepare Report**

**Use This Template:**

```markdown
# Vulnerability Report

## Summary
[One-sentence description]
Example: "Unlimited DALLA minting via integer overflow in Economy pallet"

## Severity
[Self-assessed: Critical/High/Medium/Low]

## Affected Component
[Specific pallet/contract/endpoint]
Example: pallet-belize-economy, transfer() function

## Vulnerability Details

### Root Cause
[Technical explanation of the bug]
Example: "The transfer() function uses u128::wrapping_add() instead of checked_add(), allowing overflow"

### Attack Scenario
[Step-by-step exploitation]
1. Create account with 0 DALLA balance
2. Call transfer(victim, u128::MAX)
3. Overflow causes balance to wrap to u128::MAX
4. Drain treasury by transferring to self

### Proof-of-Concept
[Runnable code or transaction hash]
```rust
// Exploit code here
let overflow = u128::MAX;
Economy::transfer(origin, victim, overflow)?; // Wraps to 0
```

### Impact Assessment
[Economic loss estimate, affected users]
- **Economic:** $50M+ (entire treasury)
- **Users Affected:** All DALLA holders
- **Severity Justification:** Critical (CVSS 10.0)

### Recommended Fix
[Code patch or architecture change]
```rust
// Replace wrapping_add with checked_add
let new_balance = balance.checked_add(amount)
    .ok_or(Error::<T>::Overflow)?;
```

### Evidence
[Screenshots, videos, transaction hashes]
- Testnet TX: 0xabcd1234...
- Video: https://youtu.be/...
- Screenshot: [attached]

## Reporter Information
- Name: [Optional, for Hall of Fame]
- Contact: [Email, Signal, Keybase]
- HackerOne/Immunefi Username: [If applicable]
- Ethereum Address: [For bounty payout]
```

**Step 3: Submit via Platform**

**HackerOne (Primary):**
1. Visit https://hackerone.com/belizechain
2. Click "Submit Report"
3. Paste your report
4. Attach evidence files
5. Submit (confidential by default)

**Immunefi (Alternative):**
1. Visit https://immunefi.com/bounty/belizechain
2. Connect wallet (for payout)
3. Submit report
4. Track status in dashboard

**Email (Emergency Only):**
- If platforms are down, email: security@belizechain.org
- Use PGP encryption (key: https://keys.openpgp.org/search?q=security@belizechain.org)
- Subject line: "CRITICAL VULNERABILITY - [Component Name]"

---

### 4.2 Triage Process

**Our Response Timeline:**

| Severity | Initial Response | Triage Complete | Fix ETA | Bounty Payout |
|----------|------------------|-----------------|---------|---------------|
| **Critical** | 4 hours | 24 hours | 7 days | 14 days |
| **High** | 24 hours | 3 days | 14 days | 30 days |
| **Medium** | 3 days | 7 days | 30 days | 45 days |
| **Low** | 7 days | 14 days | 90 days | 60 days |

**Triage Workflow:**

**Phase 1: Initial Triage (24-48 hours)**
1. **Auto-Response:** "Report received, assigned ID #BZC-2025-001"
2. **Security Team Review:** Senior engineer assesses severity
3. **Reproducibility Check:** Attempt to reproduce on testnet
4. **Severity Confirmation:** Assign final severity (may differ from submission)
5. **Initial Feedback:** "Confirmed Critical, fix in progress" OR "Cannot reproduce, need more info"

**Phase 2: Validation (1-7 days)**
1. **Deep Dive:** Core team analyzes root cause
2. **Impact Assessment:** Economic loss calculation, affected users
3. **Fix Development:** Code patch created and tested
4. **Bounty Calculation:** Base bounty + multipliers
5. **Update Reporter:** "Fix ready, bounty approved: $50,000"

**Phase 3: Remediation (1-14 days)**
1. **Deploy Fix:** Testnet → Staging → Mainnet
2. **Verification:** Reporter tests fix on testnet
3. **Regression Test:** Automated test added to CI/CD
4. **Documentation:** Update security docs, changelog

**Phase 4: Payout (14-60 days)**
1. **KYC (if >$10,000):** Tax compliance (W-9 form for US residents)
2. **Payment Method:** Ethereum (USDC), bank transfer, or crypto
3. **Public Recognition:** Hall of Fame entry (if desired)
4. **Disclosure:** 90-day embargo, then public blog post

---

### 4.3 Communication Channels

**Primary Contact:**
- **Email:** security@belizechain.org (monitored 24/7)
- **PGP Key:** https://keys.openpgp.org/search?q=security@belizechain.org
- **Signal:** +501-XXX-XXXX (encrypted messaging)

**Platform Support:**
- **HackerOne:** Direct messaging within platform
- **Immunefi:** Discord #bug-bounty channel
- **Twitter/X:** @BelizeChainSec (public, non-sensitive only)

**Emergency Contact (Critical Only):**
- **On-Call Engineer:** security-emergency@belizechain.org
- **Phone:** +501-XXX-XXXX (voice, SMS)
- **Expected Response:** <1 hour for Critical

**Update Frequency:**
- **Critical:** Daily updates until fixed
- **High:** Every 3 days
- **Medium/Low:** Weekly updates

---

## 5. Rules of Engagement

### 5.1 Responsible Disclosure

**DO:**
- ✅ Report vulnerabilities privately (HackerOne/Immunefi)
- ✅ Give us 90 days to fix before public disclosure
- ✅ Test on testnet ONLY (never exploit mainnet)
- ✅ Provide detailed reproduction steps
- ✅ Suggest fixes (bonus points!)
- ✅ Respect user privacy (don't access real KYC data)
- ✅ Follow disclosure timeline we agree on

**DON'T:**
- ❌ Publicly disclose before 90-day embargo
- ❌ Exploit on mainnet (even to prove impact)
- ❌ Access production KYC/PII data
- ❌ Demand ransom or extort ("pay or I publish")
- ❌ Submit duplicates across platforms (pick one)
- ❌ DDoS infrastructure (we'll just block you)
- ❌ Social engineer core team (unless coordinated)

**Legal Safe Harbor:**

We will NOT pursue legal action if you:
1. Comply with rules above
2. Act in good faith (no malicious intent)
3. Report privately and wait for fix
4. Don't cause real harm (use testnet)

**We Reserve Right to Legal Action if:**
- You exploit mainnet and steal funds
- You ransom vulnerabilities
- You sell exploits to third parties
- You cause deliberate harm (data deletion, etc.)

---

### 5.2 Disclosure Timeline

**Standard 90-Day Embargo:**

| Day | Milestone |
|-----|-----------|
| **Day 0** | Vulnerability reported |
| **Day 1** | Initial triage, severity confirmed |
| **Day 7** | Fix developed and tested |
| **Day 14** | Fix deployed to mainnet |
| **Day 30** | Bounty paid to reporter |
| **Day 90** | Public disclosure (blog post, CVE) |

**Accelerated Disclosure (If Actively Exploited):**
- If we detect active exploitation on mainnet, we may disclose early
- Reporters will be notified and compensated fairly
- Emergency patch deployed immediately

**Extended Embargo (Complex Fixes):**
- For architecture changes requiring >90 days, we may request extension
- Reporter must agree (or can disclose after 90 days)
- Partial bounty paid at Day 90 (final bounty at fix deployment)

**Public Disclosure Format:**
- **Blog Post:** Technical writeup on belizechain.org/blog
- **CVE Number:** Registered with MITRE (if applicable)
- **Credit:** Reporter name + link (if desired)
- **Timeline:** Discovery → Fix → Disclosure
- **Lessons Learned:** What we're doing to prevent recurrence

---

### 5.3 Duplicate Submissions

**First-to-Report Wins:**
- If 3 researchers submit same vulnerability, first submission gets full bounty
- Timestamps based on platform submission time (HackerOne/Immunefi)

**Duplicate Rewards (Independent Discovery):**
- 2nd submission: 10% of bounty (if within 7 days of first)
- 3rd+ submissions: Hall of Fame mention only
- Must demonstrate independent discovery (no sharing)

**Related Vulnerabilities:**
- If same root cause affects multiple components, treated as ONE vulnerability
- Example: Integer overflow in Economy pallet → applies to all transfer functions
- Bounty: 1x Critical (not 5x)

**Variant Submissions:**
- If you find a NEW way to exploit SAME root cause, that's a bonus (+20%)
- Example: "You fixed checked_add(), but forgot checked_sub()" → New bounty

---

## 6. Payout Process

### 6.1 Payment Methods

**Cryptocurrency (Preferred):**
- **USDC (Ethereum):** Sent to your Ethereum address within 24 hours of approval
- **USDT (Tron):** Cheaper gas fees for smaller bounties
- **BTC:** Available upon request (3-day settlement)
- **ETH:** Available upon request

**Traditional Payment:**
- **Bank Transfer:** ACH (US), SWIFT (international) - 5-7 business days
- **PayPal:** Available for bounties <$10,000
- **Check:** Mailed to address (US only, 10-14 days)

**Tax Considerations:**
- **US Residents:** W-9 form required for bounties >$600 (1099-MISC issued)
- **Non-US Residents:** W-8BEN form required for bounties >$10,000
- **Withholding:** We withhold 30% for non-US residents (unless tax treaty)

---

### 6.2 KYC Requirements

**Bounties <$10,000:**
- No KYC required
- Provide payout address only

**Bounties $10,000-$50,000:**
- **Identity Verification:**
  - Full name
  - Email address
  - Country of residence
  - Tax ID (SSN, EIN, or foreign equivalent)

**Bounties >$50,000:**
- **Enhanced KYC:**
  - Government-issued ID (passport, driver's license)
  - Proof of address (utility bill, bank statement)
  - Video call verification (15 minutes)
  - Tax forms (W-9/W-8BEN)

**Privacy Commitment:**
- KYC data stored securely (encrypted at rest)
- Not shared with third parties (except tax authorities if required)
- Deleted after 7 years (legal retention requirement)

---

### 6.3 Bounty Splitting

**Multiple Reporters (Same Vulnerability):**
- First submission: 50% of bounty
- Second submission: 30% of bounty
- Third submission: 20% of bounty
- (Only if submitted within 48 hours of each other)

**Collaborated Research:**
- If 2+ researchers work together, split bounty as agreed
- Notify us of split percentage in submission
- Each researcher completes separate KYC (if required)

**Example:**
- Critical vulnerability: $100,000 base bounty
- Researcher A (lead): $60,000 (60%)
- Researcher B (support): $40,000 (40%)

---

## 7. Hall of Fame

### 7.1 Recognition

**Public Recognition (Optional):**
- **Blog Post:** Featured writeup on belizechain.org
- **Twitter Shoutout:** @BelizeChainSec mentions you
- **Conference Talk:** Invited to present at BelizeChain Security Summit
- **Swag:** Limited-edition t-shirt, hoodie, commemorative NFT

**Anonymous Recognition:**
- If you prefer privacy, we'll credit "Anonymous Researcher"
- Still eligible for all bounties and bonuses

**Top Researchers:**
- **Leaderboard:** https://belizechain.org/security/hall-of-fame
- **Yearly Awards:**
  - 🥇 Most Critical Vulnerabilities Found: $10,000 bonus
  - 🥈 Most High Vulnerabilities Found: $5,000 bonus
  - 🥉 Best Researcher Report Quality: $2,500 bonus

---

### 7.2 Current Hall of Fame (Examples)

**2025 Top Researchers:**

| Rank | Researcher | Vulnerabilities | Total Bounties | Notable Finds |
|------|------------|-----------------|----------------|---------------|
| 🥇 | **samczsun** | 2 Critical, 3 High | $215,000 | Treasury multi-sig bypass, bridge double-spend |
| 🥈 | **Anonymous** | 1 Critical, 5 High | $175,000 | Unlimited DALLA minting |
| 🥉 | **Trail of Bits** | 4 High, 8 Medium | $150,000 | Comprehensive audit findings |
| 4 | **OpenZeppelin** | 2 High, 10 Medium | $90,000 | Smart contract reentrancy issues |
| 5 | **guido** | 1 High, 7 Medium | $65,000 | Governance vote manipulation |

*Note: Above are example entries. Hall of Fame will be updated monthly after program launch.*

---

## 8. Program Evolution

### 8.1 Scope Expansion

**Quarterly Reviews:**
- Every 3 months, we review program scope
- Add new components (as BelizeChain grows)
- Adjust bounties based on risk

**Future Scope (Planned):**
- **Q1 2026:** Nawal AI integration (federated learning exploits)
- **Q2 2026:** Kinich Quantum (quantum algorithm manipulation)
- **Q3 2026:** Pakit Storage (IPFS/Arweave security)
- **Q4 2026:** Mobile apps (Maya Wallet iOS/Android)

**Community Requests:**
- If researchers suggest new scope areas, we'll consider
- Example: "Can we test the validator on-boarding process?" → Added to scope

---

### 8.2 Bounty Adjustments

**Yearly Review:**
- Adjust bounties based on TVL (Total Value Locked)
- As BelizeChain grows, bounties increase proportionally

**Dynamic Bounties:**
- **TVL $10M:** Base bounties (current)
- **TVL $50M:** 2x bounties (Critical → $200,000 max)
- **TVL $100M:** 3x bounties (Critical → $300,000 max)
- **TVL $1B:** 5x bounties (Critical → $500,000 max)

**Special Bounties (Limited Time):**
- During major upgrades (e.g., XCM v4 migration): 2x bounties for 30 days
- Before mainnet launch: 3x bounties for 60 days
- Post-incident: 5x bounties for related vulnerabilities (60 days)

---

## 9. Frequently Asked Questions

### Q1: Can I test on mainnet?
**A:** NO. Always use testnet. Exploiting mainnet results in disqualification and potential legal action. Testnet faucet: https://faucet.belizechain.org

### Q2: How long until I get paid?
**A:** 
- Critical: 14 days
- High: 30 days
- Medium: 45 days
- Low: 60 days
(After vulnerability confirmed)

### Q3: Can I publish my findings before 90 days?
**A:** No, we require 90-day embargo. Early disclosure disqualifies you from bounty. Exception: If we fail to fix within 90 days, you may disclose.

### Q4: What if I find something AFTER the audit?
**A:** Still eligible! Audit firms don't find everything. We encourage continuous testing.

### Q5: Can I automate vulnerability scanning?
**A:** Yes, on testnet only. Automated scanners on mainnet RPC may be rate-limited.

### Q6: What if my vulnerability is a duplicate?
**A:** If submitted within 7 days of first report, you get 10% of bounty (for independent discovery). After 7 days, no bounty.

### Q7: Can I remain anonymous?
**A:** Yes, for bounties <$10,000. For larger bounties, we need KYC for tax compliance, but can credit you anonymously publicly.

### Q8: What if BelizeChain disagrees with my severity?
**A:** We follow CVSS v3.1 scoring. If you disagree, we'll explain our reasoning. Final decision by Security Lead, but we're open to discussion.

### Q9: Can I test the federated learning integration?
**A:** Nawal AI has a separate bug bounty program. Contact: ai-security@belizechain.org

### Q10: What if I accidentally exploit mainnet?
**A:** Contact us IMMEDIATELY (security-emergency@belizechain.org). If unintentional and you cooperate, we won't take legal action. Return any stolen funds.

---

## 10. Program Metrics

### 10.1 Success Metrics (Public Transparency)

**Monthly Reporting:**
- Vulnerabilities reported: [Number]
- Vulnerabilities confirmed: [Number]
- Bounties paid: [Total USD]
- Average triage time: [Hours]
- Average fix time: [Days]

**Yearly Reporting:**
- Total bounties paid: [USD]
- Top researcher: [Name]
- Most common vulnerability type: [Category]
- Prevented exploits value: [Estimated USD]

---

### 10.2 Historical Data (Examples)

**2025 (First Year):**
- Reports submitted: 127
- Valid vulnerabilities: 43 (34% hit rate)
- Critical: 2 ($180,000 paid)
- High: 8 ($240,000 paid)
- Medium: 18 ($90,000 paid)
- Low: 15 ($15,000 paid)
- **Total Bounties Paid:** $525,000
- **Prevented Exploits:** ~$50M (estimated)
- **ROI:** 95x (every $1 spent prevented $95 in losses)

---

## 11. Legal Terms

### 11.1 Safe Harbor

BelizeChain provides legal safe harbor to security researchers who:
1. Act in good faith (no malicious intent)
2. Report vulnerabilities privately via official channels
3. Do not exploit mainnet or cause real harm
4. Comply with 90-day disclosure embargo
5. Do not access production PII/KYC data without authorization

**We Will NOT:**
- Pursue legal action against good-faith researchers
- Report you to law enforcement
- Request damages for testing (on testnet)
- Revoke bounty for following rules

**We Reserve Right to Legal Action For:**
- Mainnet exploitation with fund theft
- Ransom/extortion attempts
- Public disclosure before 90-day embargo (without justification)
- Selling exploits to third parties
- Deliberate harm (data deletion, DoS attacks)

---

### 11.2 Terms of Service

By participating in this bug bounty program, you agree to:

1. **Confidentiality:** Keep vulnerabilities confidential until public disclosure
2. **Testing Scope:** Only test in-scope assets (see Section 2.1)
3. **Good Faith:** Act in good faith, no malicious intent
4. **Compliance:** Follow all rules and responsible disclosure guidelines
5. **No Warranty:** Participate at your own risk (no guarantees)
6. **Program Changes:** We may modify program at any time (with notice)
7. **Dispute Resolution:** Final decisions by BelizeChain Security Lead (but we're reasonable!)

---

### 11.3 Disclaimer

**No Guarantee of Payment:**
- Payment subject to validation and compliance with rules
- Duplicate submissions receive reduced or no bounty
- Out-of-scope vulnerabilities receive no bounty
- We reserve right to adjust severity and bounty amount

**Program Modifications:**
- We may change scope, bounties, or rules at any time
- Changes announced 30 days in advance (via email + blog)
- Active reports under old rules honored

**Liability Waiver:**
- BelizeChain not liable for researcher actions
- You assume all risks of testing (e.g., testnet token loss)
- No employment relationship created by participation

---

## 12. Contact Information

**Bug Bounty Program Team:**
- **Email:** security@belizechain.org
- **PGP Key:** https://keys.openpgp.org/search?q=security@belizechain.org
- **Signal:** +501-XXX-XXXX (encrypted)

**Platforms:**
- **HackerOne:** https://hackerone.com/belizechain
- **Immunefi:** https://immunefi.com/bounty/belizechain

**Social Media:**
- **Twitter/X:** @BelizeChainSec
- **Discord:** discord.gg/belizechain (#bug-bounty channel)
- **Telegram:** t.me/BelizeChainSecurity

**Emergency Contact (Critical Only):**
- **Emergency Email:** security-emergency@belizechain.org
- **Phone:** +501-XXX-XXXX (24/7 on-call)

---

## 13. Appendix

### A. Example Vulnerability Report

```markdown
# Vulnerability Report: Unlimited DALLA Minting via Integer Overflow

## Summary
The Economy pallet's `transfer()` function uses `wrapping_add()` instead of `checked_add()`, allowing an attacker to mint unlimited DALLA tokens via integer overflow.

## Severity
**Critical (CVSS 10.0)**

## Affected Component
- **Pallet:** pallet-belize-economy
- **Function:** `transfer(origin, dest, amount)`
- **File:** `belizechain/pallets/economy/src/lib.rs`
- **Lines:** 234-248

## Vulnerability Details

### Root Cause
The transfer function performs balance updates using `wrapping_add()`:
```rust
let new_balance = balance.wrapping_add(amount); // VULNERABLE
```

This allows overflow: `u128::MAX + 1 = 0`, bypassing balance checks.

### Attack Scenario
1. Attacker creates account with 0 DALLA balance
2. Calls `transfer(victim, u128::MAX)` to their own account
3. Balance overflows: `0 + u128::MAX + 1 = 0` (wraps around)
4. Attacker now has u128::MAX DALLA (340 undecillion tokens)
5. Can drain entire treasury, collapse economy

### Proof-of-Concept
```rust
// Tested on testnet (account: 5GTest...)
use frame_support::assert_ok;

#[test]
fn test_overflow_exploit() {
    new_test_ext().execute_with(|| {
        let attacker = 1;
        let victim = 2;
        
        // Attacker has 0 balance
        assert_eq!(Economy::balance(attacker), 0);
        
        // Overflow attack
        assert_ok!(Economy::transfer(
            Origin::signed(attacker),
            victim,
            u128::MAX
        ));
        
        // Attacker now has u128::MAX - 1 DALLA
        assert_eq!(Economy::balance(attacker), u128::MAX - 1);
        // Victim has 1 DALLA (overflowed)
        assert_eq!(Economy::balance(victim), 1);
    });
}
```

**Testnet Transaction:** 
- Hash: `0xabcd1234...` 
- Block: #42,069
- Result: SUCCESS (exploit works)

### Impact Assessment
- **Economic Loss:** $50M+ (entire treasury drained)
- **Users Affected:** All DALLA holders (inflation → hyperinflation)
- **National Security:** Government payroll unpayable, economy collapse
- **Severity:** **CRITICAL** (CVSS 10.0)

### Recommended Fix
Replace `wrapping_add()` with `checked_add()`:

```rust
// BEFORE (vulnerable)
let new_balance = balance.wrapping_add(amount);

// AFTER (secure)
let new_balance = balance.checked_add(amount)
    .ok_or(Error::<T>::Overflow)?;
```

**Regression Test:**
```rust
#[test]
fn test_overflow_prevented() {
    new_test_ext().execute_with(|| {
        let attacker = 1;
        
        // Should fail with Overflow error
        assert_noop!(
            Economy::transfer(Origin::signed(attacker), 2, u128::MAX),
            Error::<T>::Overflow
        );
    });
}
```

## Evidence
- Video: https://youtu.be/overflow-exploit-demo
- Screenshots: [Attached: before.png, after.png, testnet-tx.png]
- Testnet Logs: [Attached: exploit.log]

## Reporter Information
- **Name:** Anonymous Researcher (for public credit)
- **Contact:** whiteh4t@protonmail.com
- **HackerOne:** @whiteh4t
- **Payout Address:** 0x1234abcd... (Ethereum USDC)

## Requested Bounty
**$100,000** (Critical + POC + Fix + Report = 1.5x multiplier)
```

---

### B. Security Tools for Researchers

**Recommended Tools:**

**Fuzzing:**
- **cargo-fuzz:** Rust fuzzing framework (https://github.com/rust-fuzz/cargo-fuzz)
- **AFL++:** American Fuzzy Lop (advanced fuzzer)
- **Honggfuzz:** Google's feedback-driven fuzzer

**Static Analysis:**
- **Clippy:** Rust linter (built-in)
- **Cargo Audit:** Dependency vulnerability scanner
- **Kani:** Rust formal verification (AWS)

**Dynamic Testing:**
- **Zombienet:** Polkadot/Substrate network simulator
- **Chopsticks:** Parachain forking tool
- **Polkadot.js:** Transaction crafting, RPC queries

**Smart Contract Tools:**
- **ink! Analyzer:** Linter for ink! contracts
- **Slither (adapted):** Smart contract analyzer
- **Echidna:** Ethereum fuzzer (adaptable to WASM)

---

### C. CVE Process

**If We Issue a CVE:**
1. **Severity:** Critical or High vulnerabilities only
2. **Timeline:** After 90-day embargo + fix deployed
3. **Format:** CVE-YYYY-NNNNN (e.g., CVE-2025-00123)
4. **Database:** MITRE CVE database, NVD
5. **Credit:** Your name (if desired)

**Recent CVE Examples (Other Projects):**
- CVE-2022-29177: Polkadot consensus vulnerability (High)
- CVE-2021-31535: Ethereum Geth DoS (Medium)
- CVE-2023-12345: Cosmos IBC replay attack (Critical)

---

## Program Launch Timeline

**Pre-Launch (November 2025):**
- Finalize audit with Trail of Bits
- Fix all P0/P1 findings
- Set up HackerOne/Immunefi accounts
- Allocate $500k bounty pool

**Launch (December 2025):**
- Announce program publicly (blog post, Twitter)
- Open submissions on HackerOne/Immunefi
- Deploy testnet with faucet
- Host AMA on Discord

**Post-Launch (Q1 2026):**
- Monthly public reports (vulnerabilities, bounties)
- Quarterly program review (scope, bounties)
- Yearly awards (top researchers)

---

**Program Status:** 🚀 LAUNCHING DECEMBER 2025

**Join the Hunt:** https://hackerone.com/belizechain

---

*Built with 💎 for the sovereign nation of Belize 🇧🇿*
*Protecting citizen funds, national identity, and digital sovereignty.*
