# BelizeChain Incident Response Plan

**Version:** 1.0  
**Last Updated:** November 4, 2025  
**Status:** Active testing and hardening  
**Compliance:** FSC Security Requirements, ISO 27001, NIST Cybersecurity Framework  

---

## 1. Executive Summary

This Incident Response Plan (IRP) defines BelizeChain's procedures for detecting, responding to, and recovering from security incidents. It covers blockchain-specific incidents (consensus attacks, bridge compromises) and traditional infrastructure issues (network outages, data breaches).

**Scope:** All BelizeChain infrastructure, including:
- Blockchain runtime (13 pallets)
- 5 smart contracts (ink! WASM)
- Cross-chain bridges (Ethereum + Polkadot XCM)
- Node infrastructure (validators, RPC, archive)
- Web applications (6 UI portals)
- Supporting services (databases, monitoring, APIs)

**Objectives:**
1. **Rapid Detection:** Identify incidents within 15 minutes (critical), 1 hour (high)
2. **Swift Containment:** Stop incident spread within 1 hour (critical), 4 hours (high)
3. **Effective Recovery:** Restore services within 4 hours (critical), 24 hours (high)
4. **Lessons Learned:** Document and improve after every incident

---

## 2. Incident Classification

### 2.1 Severity Levels

**SEV-1 (CRITICAL) - Red Alert 🔴**
- **Definition:** Active exploit causing or imminently threatening significant harm
- **Response Time:** <15 minutes
- **Examples:**
  - Treasury funds actively being drained
  - Bridge double-spending exploit in progress
  - Private keys compromised (validator or admin)
  - Consensus failure (blockchain halted)
  - KYC database breach (PII exposure)
  - Ransomware encryption of critical systems
  
- **Response Team:** Full team mobilized (all hands on deck)
- **Communication:** Immediate (Slack + Phone + SMS)
- **Authority:** CEO, CTO, Security Lead have emergency powers

---

**SEV-2 (HIGH) - Orange Alert 🟠**
- **Definition:** Significant security issue requiring urgent action
- **Response Time:** <1 hour
- **Examples:**
  - Vulnerability discovered (P0/P1 from bug bounty)
  - Unauthorized access attempt (successful authentication bypass)
  - DDoS attack degrading services (50%+ capacity)
  - Bridge validator compromise (1 of 5 compromised)
  - Governance proposal attack (vote manipulation detected)
  - Data integrity issue (state corruption)
  
- **Response Team:** On-call engineer + Security Lead + relevant specialists
- **Communication:** Slack #incidents + Email
- **Authority:** Security Lead coordinates response

---

**SEV-3 (MEDIUM) - Yellow Alert 🟡**
- **Definition:** Security issue requiring investigation, moderate urgency
- **Response Time:** <4 hours
- **Examples:**
  - Suspicious activity detected (anomaly, not confirmed attack)
  - Non-critical vulnerability (P2/P3 from bug bounty)
  - Service degradation (latency spike, no exploit)
  - Failed authentication attempts (brute force, blocked)
  - Misconfiguration discovered (exposed internal API)
  
- **Response Team:** On-call engineer investigates
- **Communication:** Slack #security + Ticket
- **Authority:** On-call engineer handles, escalates if needed

---

**SEV-4 (LOW) - Green Alert 🟢**
- **Definition:** Minor issue or informational, no immediate action required
- **Response Time:** <24 hours
- **Examples:**
  - Low-severity vulnerability report
  - User-reported bug (UI issue, no security impact)
  - Monitoring alert false positive
  - Documentation error
  
- **Response Team:** Assigned engineer during business hours
- **Communication:** Ticket system
- **Authority:** Standard development process

---

### 2.2 Incident Categories

**1. Blockchain Exploits**
- Consensus attacks (51%, long-range, finality stall)
- Economic exploits (flash loans, front-running, oracle manipulation)
- Pallet vulnerabilities (unauthorized minting, treasury draining)
- Validator compromise (private key theft, collusion)

**2. Bridge Compromises**
- Double-spending attacks (cross-chain replay)
- Relayer compromise (multi-sig threshold breach)
- Smart contract exploits (reentrancy, overflow)
- Cross-chain message spoofing (fake XCM messages)

**3. Infrastructure Incidents**
- Network outages (DDoS, hardware failure, cloud provider issues)
- Database corruption (PostgreSQL, Redis)
- Service degradation (high latency, resource exhaustion)
- Configuration errors (firewall misconfiguration, DNS issues)

**4. Application Security**
- Web application attacks (SQL injection, XSS, CSRF)
- API abuse (rate limiting bypass, unauthorized access)
- Authentication bypass (session hijacking, credential stuffing)
- Data breaches (KYC/PII exposure)

**5. Governance Attacks**
- Vote manipulation (Sybil attack, vote buying)
- Proposal spam (governance DoS)
- Emergency power abuse (JaguarMode exploitation)
- Treasury proposal fraud (malicious proposals)

**6. Social Engineering**
- Phishing attacks (fake wallet, credential theft)
- Insider threats (malicious employee, compromised account)
- Validator bribery (censorship, collusion)
- Supply chain attacks (compromised dependencies)

---

## 3. Response Procedures

### 3.1 Detection & Triage (Phase 1)

**Detection Methods:**

1. **Automated Monitoring:**
   - Prometheus alerts (CPU, memory, disk, network)
   - Grafana dashboards (transaction volume, block times, finality lag)
   - Loki log alerts (error patterns, suspicious activity)
   - Custom blockchain monitors (balance changes, large transfers, governance events)

2. **Bug Bounty Reports:**
   - HackerOne/Immunefi submissions (immediate notification)
   - Security email (security@belizechain.org, monitored 24/7)
   - Community reports (Discord, Twitter, public forums)

3. **User Reports:**
   - Support tickets (Zendesk, flagged keywords)
   - Social media monitoring (Twitter alerts for "@BelizeChain + hack/exploit")
   - Validator reports (via private channels)

4. **Threat Intelligence:**
   - Blockchain security feeds (Chainalysis, Elliptic)
   - CVE databases (Substrate, Polkadot, Rust crates)
   - Peer networks (Polkadot ecosystem alerts)

**Triage Workflow:**

```
1. ALERT RECEIVED
   ├─> Automated: Prometheus/Grafana/Loki alert fires
   ├─> Manual: Bug bounty, user report, threat intel
   └─> On-call engineer notified (PagerDuty)

2. INITIAL ASSESSMENT (5 minutes)
   ├─> Review alert details, logs, metrics
   ├─> Confirm incident (real threat vs false positive)
   └─> Assign preliminary severity (SEV-1/2/3/4)

3. ESCALATION DECISION
   ├─> SEV-1: Immediately escalate to Security Lead + CTO
   ├─> SEV-2: Escalate to Security Lead
   ├─> SEV-3: On-call handles, notify Security Lead
   └─> SEV-4: Create ticket, handle during business hours

4. INCIDENT DECLARATION
   ├─> Create incident channel: #incident-YYYY-MM-DD-NNN
   ├─> Post incident summary (what, impact, status)
   ├─> Assign Incident Commander (Security Lead for SEV-1/2)
   └─> Start incident timeline (Google Doc shared with team)
```

---

### 3.2 Containment (Phase 2)

**Goal:** Stop incident spread, prevent further damage

**SEV-1 Containment (15-60 minutes):**

**Treasury Draining:**
1. Activate emergency pause (sudo call: `Economy::emergency_pause()`)
2. Freeze affected accounts (multi-sig approval required)
3. Snapshot blockchain state (for forensics)
4. Coordinate with exchanges (freeze deposits/withdrawals)
5. Public communication (Twitter: "Investigating security incident, trading paused")

**Bridge Compromise:**
1. Pause bridge contracts (Ethereum + BelizeChain side)
2. Revoke relayer keys (multi-sig transaction)
3. Halt cross-chain transfers (XCM + Ethereum)
4. Notify partner chains (Acala, Moonbeam)
5. Calculate affected funds (prepare recovery plan)

**Consensus Failure:**
1. Emergency validator coordination call (within 15 minutes)
2. Activate emergency governance (JaguarMode if needed)
3. Deploy runtime patch (if known fix available)
4. Coordinate chain restart (if necessary)
5. Monitor finality restoration

**Private Key Compromise:**
1. Rotate compromised keys immediately (new validator/admin keys)
2. Revoke old keys (multi-sig call: `Session::purge_keys()`)
3. Review transaction history (identify unauthorized actions)
4. Slash compromised validator (if applicable)
5. Force session rotation

**KYC Database Breach:**
1. Disconnect database from network (isolate)
2. Snapshot database for forensics
3. Identify affected records (query access logs)
4. Prepare breach notification (legal + FSC compliance)
5. Enable account monitoring (fraud detection)

---

**SEV-2 Containment (1-4 hours):**

**DDoS Attack:**
1. Enable Cloudflare "Under Attack" mode (aggressive filtering)
2. Increase rate limits (reduce from 1000 to 100 req/s per IP)
3. Block attacking IPs (firewall rules)
4. Scale up RPC nodes (Kubernetes HPA)
5. Monitor attack patterns (identify attack vectors)

**Validator Compromise (1 of 5):**
1. Suspend compromised validator (multi-sig call)
2. Transfer stake to new validator (if possible)
3. Rotate validator keys
4. Investigate compromise vector (how did it happen?)
5. Strengthen validator security (2FA, hardware keys)

**Governance Attack:**
1. Pause referendum voting (if in progress)
2. Invalidate fraudulent votes (governance pallet call)
3. Extend voting period (allow legitimate re-vote)
4. Implement stricter voting requirements (temporary)
5. Investigate attack source (identify attackers)

---

### 3.3 Eradication (Phase 3)

**Goal:** Remove root cause, fix vulnerabilities

**Patch Development:**
1. **Identify Root Cause:**
   - Code review (find vulnerable function)
   - Reproduce exploit (testnet simulation)
   - Assess scope (other affected components?)

2. **Develop Fix:**
   - Write patch (Rust code for pallets, ink! for contracts)
   - Test extensively (unit tests, integration tests, fuzz testing)
   - Peer review (2+ engineers approve)
   - Security review (Security Lead approves)

3. **Deploy Patch:**
   - **Runtime Upgrade (On-Chain):**
     ```bash
     # Build patched runtime
     cargo build --release
     
     # Submit runtime upgrade proposal
     polkadot-js-api tx.sudo.sudoUncheckedWeight(
       api.tx.system.setCode(wasmCode),
       weight
     )
     
     # Wait for finality (12 blocks ~1 minute)
     # Verify upgrade successful
     ```
   
   - **Smart Contract Upgrade:**
     ```bash
     # Deploy new contract version
     cargo contract upload --suri //Alice
     
     # Update contract address in UI
     # Migrate state if needed (manual process)
     ```
   
   - **Infrastructure Update:**
     ```bash
     # Rolling update (Ceiba/self-hosted)
     docker compose -f /home/wicked/Projects/infra/docker-compose.yml pull belizechain-node
     docker compose -f /home/wicked/Projects/infra/docker-compose.yml up -d belizechain-node

     # Monitor rollout
     docker compose -f /home/wicked/Projects/infra/docker-compose.yml ps
     ```

4. **Verify Fix:**
   - Re-run exploit POC (should fail)
   - Monitor for recurrence (24 hours)
   - Run regression tests (ensure no new issues)

---

### 3.4 Recovery (Phase 4)

**Goal:** Restore normal operations, compensate affected users

**Service Restoration:**

**Blockchain Resume:**
1. Verify all validators online (check finality)
2. Unpause pallets (Economy, BelizeX, Interoperability)
3. Monitor transaction throughput (gradual ramp-up)
4. Notify exchanges (resume deposits/withdrawals)
5. Public announcement (services restored)

**Bridge Resume:**
1. Re-enable relayers (multi-sig approval)
2. Process queued transactions (oldest first)
3. Verify cross-chain state consistency
4. Monitor for anomalies (24 hours)
5. Notify partner chains (Acala, Moonbeam)

**Data Recovery:**
1. Restore from latest backup (if database corrupted)
2. Replay transactions (from blockchain archive)
3. Verify data integrity (checksums, audits)
4. Re-index blockchain data (SubQuery)
5. Update UI caches (Redis flush)

**Affected User Compensation:**

**Treasury Draining:**
- Identify affected accounts (query transaction history)
- Calculate losses (balance_before - balance_after)
- Governance proposal (treasury compensation)
- Multi-sig approval (4 of 7 signers)
- Execute compensation (automated script)

**Bridge Loss:**
- Review bridge transaction logs (failed withdrawals)
- Verify legitimate users (exclude attackers)
- Insurance fund activation (5% of treasury reserved)
- Pro-rata distribution (if insufficient funds)
- Transparency report (public accounting)

**KYC Breach:**
- Mandatory breach notification (72 hours, FSC requirement)
- Credit monitoring services (12 months free)
- Identity theft insurance (offered to affected users)
- Legal hotline (24/7 support)
- Public apology (CEO statement)

---

### 3.5 Post-Incident Review (Phase 5)

**Goal:** Learn from incident, improve defenses

**Timeline:** Within 7 days of incident resolution

**Post-Incident Report (PIR) Template:**

```markdown
# Post-Incident Report: [Incident Title]

**Incident ID:** INC-2025-11-04-001
**Severity:** SEV-1 (Critical)
**Incident Commander:** [Name]
**Date:** November 4, 2025
**Duration:** 3 hours 42 minutes (detection → resolution)

## 1. Executive Summary
[2-3 paragraph overview of what happened, impact, resolution]

## 2. Timeline
| Time (UTC) | Event |
|------------|-------|
| 14:23 | Alert fired: Unusual treasury withdrawal detected |
| 14:28 | On-call engineer triaged (confirmed SEV-1) |
| 14:30 | Security Lead notified, incident channel created |
| 14:35 | Emergency pause activated (Economy pallet) |
| 14:45 | Root cause identified (integer overflow in transfer()) |
| 15:30 | Patch developed and tested |
| 16:00 | Runtime upgrade deployed |
| 16:15 | Exploit POC re-run (failed, fix confirmed) |
| 17:30 | Services restored, public announcement |
| 18:05 | Incident closed |

## 3. Root Cause Analysis
**What Happened:**
[Technical explanation of vulnerability]

**Why It Happened:**
- Direct cause: Code used wrapping_add() instead of checked_add()
- Contributing factors: Missed in code review, no fuzz testing

**Why We Didn't Catch It Earlier:**
- Audit hadn't started yet (scheduled Q1 2026)
- Test coverage for overflow cases was incomplete
- Automated analysis (Clippy) doesn't catch all overflow cases

## 4. Impact Assessment
**Users Affected:** 47 accounts
**Funds Lost:** 1.2M DALLA ($1.8M USD)
**Services Disrupted:** Economy pallet (3.5 hours), BelizeX DEX (4 hours)
**Reputational Impact:** Moderate (handled transparently, quick resolution)

## 5. Response Effectiveness
**What Went Well:**
✅ Fast detection (5 minutes from exploit to alert)
✅ Rapid escalation (2 minutes triage)
✅ Clear communication (incident channel, updates every 15 min)
✅ Quick fix deployment (1.5 hours from patch to resolution)

**What Could Be Improved:**
❌ Slower patch development than ideal (1.5 hours, target <1 hour)
❌ Runtime upgrade process manual (should be automated)
❌ Public communication delayed (30 minutes after pause)

## 6. Action Items
| Action | Owner | Due Date | Status |
|--------|-------|----------|--------|
| Add overflow fuzz tests | @alice | 2025-11-11 | Open |
| Automate runtime upgrade | @bob | 2025-11-18 | Open |
| Improve public comms SOP | @carol | 2025-11-08 | Open |
| Schedule emergency audit | @security-lead | 2025-11-05 | Complete |
| Compensate affected users | @treasury | 2025-11-15 | In Progress |

## 7. Lessons Learned
1. **Prevention:** Prioritize formal verification for critical pallets
2. **Detection:** Existing monitoring worked well, but could be faster
3. **Response:** Emergency pause mechanism saved millions (worked as designed)
4. **Recovery:** User compensation process needs pre-defined criteria
5. **Communication:** Need better template for public announcements

## 8. Related Incidents
- None (first critical incident)

## 9. Attachments
- Exploit POC code: [link]
- Patch diff: [link]
- Transaction logs: [link]
- Grafana dashboard: [link]

**Report Prepared By:** [Security Lead]
**Reviewed By:** CTO, CEO
**Distribution:** Core team, Board of Directors, FSC (summary only)
```

---

## 4. Communication Protocols

### 4.1 Internal Communication

**Incident Channels:**

**Slack Channels:**
- `#incidents` - All incidents (SEV-1 through SEV-4)
- `#incident-YYYY-MM-DD-NNN` - Per-incident war room (SEV-1/2 only)
- `#security` - Ongoing security discussions
- `#on-call` - On-call engineer coordination

**Communication Templates:**

**SEV-1 Initial Announcement:**
```
🚨 SEV-1 INCIDENT DECLARED 🚨

**Incident ID:** INC-2025-11-04-001
**Title:** Treasury Draining Exploit Active
**Detected:** 2025-11-04 14:23 UTC
**Status:** Containment in progress

**Impact:**
- Economy pallet: PAUSED
- Affected users: ~50 accounts
- Estimated loss: 1-2M DALLA

**Actions Taken:**
- Emergency pause activated (14:35 UTC)
- Root cause investigation ongoing
- Patch development started

**Incident Commander:** @security-lead
**War Room:** #incident-2025-11-04-001

**Next Update:** 15:00 UTC (every 15 minutes)

@channel - All hands on deck. Join war room if available.
```

**Status Update (Every 15 minutes for SEV-1):**
```
📊 UPDATE - 14:45 UTC

**Status:** Root cause identified
**Progress:** Patch development in progress (50% complete)
**ETA:** Services restored by 17:00 UTC

**What Changed:**
- Identified vulnerable function: Economy::transfer()
- Integer overflow via wrapping_add()
- Patch in review (checked_add() replacement)

**Next Steps:**
- Complete patch testing (15 minutes)
- Deploy runtime upgrade (30 minutes)
- Verify fix (15 minutes)

**Next Update:** 15:00 UTC
```

---

### 4.2 External Communication

**Public Communication Channels:**
- **Twitter/X:** @BelizeChain (primary, fastest)
- **Blog:** belizechain.org/blog (detailed post-mortem)
- **Discord:** Announcement channel (community updates)
- **Email:** Notify affected users directly

**Public Announcement Templates:**

**Initial Announcement (Twitter):**
```
🔒 Security Update

We're investigating a security incident affecting the Economy pallet. 
As a precaution, we've temporarily paused transfers to protect user funds.

Status: https://status.belizechain.org
Updates: Every 30 minutes

Thank you for your patience. User funds are safe. 🇧🇿
```

**Resolution Announcement (Twitter):**
```
✅ Issue Resolved

The security incident has been resolved. Services are now restored.

Summary:
• Issue: Smart contract vulnerability
• Impact: 47 users, 1.2M DALLA
• Resolution: Patched within 4 hours
• Compensation: In progress

Full report: https://belizechain.org/blog/incident-2025-11-04

Thank you for your patience and trust. 💎
```

**Blog Post (Post-Mortem):**
```markdown
# Security Incident Post-Mortem: November 4, 2025

## What Happened
On November 4, 2025 at 14:23 UTC, we detected an active exploit targeting the Economy pallet...

[Full PIR content, slightly sanitized for public]

## Affected Users
If your account was affected, you've received a direct email. Compensation will be processed within 7 days.

## What We're Doing
1. Accelerating security audit (Trail of Bits, starting immediately)
2. Implementing formal verification for all critical pallets
3. Launching bug bounty program (December 2025)

## Transparency Commitment
We believe in full transparency. This incident report will remain public indefinitely.

Questions? Email security@belizechain.org

- BelizeChain Team
```

---

### 4.3 Regulatory Communication

**FSC (Financial Services Commission) Notification:**

**Timeline:**
- **Within 1 hour (SEV-1):** Initial notification via phone
- **Within 24 hours:** Written report (preliminary)
- **Within 7 days:** Full post-incident report

**Report Template:**
```
TO: Financial Services Commission, Belize
FROM: BelizeChain Security Team
RE: Security Incident Notification - [Incident ID]
DATE: [Date]

PRELIMINARY INCIDENT REPORT

1. INCIDENT SUMMARY
   - Date/Time: [UTC timestamp]
   - Severity: [SEV-1/2/3/4]
   - Type: [Exploit/Breach/Outage]
   - Status: [Contained/Resolved/Ongoing]

2. IMPACT ASSESSMENT
   - Users Affected: [Number]
   - Financial Impact: [Amount in BZD]
   - PII Exposure: [Yes/No, details]
   - Services Disrupted: [List]

3. RESPONSE ACTIONS
   - Containment: [Actions taken, timeline]
   - Law Enforcement: [Notified? Agency, case number]
   - User Notification: [Method, timeline]

4. ROOT CAUSE (Preliminary)
   - [Brief technical explanation]

5. REMEDIATION PLAN
   - Immediate: [Actions within 24 hours]
   - Short-term: [Actions within 7 days]
   - Long-term: [Actions within 30 days]

6. CONTACT INFORMATION
   - Incident Commander: [Name, phone, email]
   - Legal Counsel: [Name, phone, email]

FULL REPORT TO FOLLOW WITHIN 7 DAYS

[Signature]
[Title]
BelizeChain
```

---

## 5. Incident Runbooks

### 5.1 Treasury Draining Exploit

**Scenario:** Attacker is actively draining treasury funds

**Detection Signs:**
- Large unusual withdrawals from treasury (>100K DALLA)
- Multiple proposals executed rapidly (<5 minute intervals)
- Multi-sig threshold bypassed (unapproved transactions)

**Response Steps:**

1. **Confirm Exploit (2 minutes)**
   ```bash
   # Check treasury balance
   polkadot-js-api query.treasury.pot
   
   # Check recent proposals
   polkadot-js-api query.treasury.proposals
   
   # Check multi-sig transactions
   polkadot-js-api query.multiSig.calls
   ```

2. **Emergency Pause (3 minutes)**
   ```bash
   # Activate emergency pause (sudo required)
   polkadot-js-api tx.sudo.sudo(
     api.tx.economy.emergencyPause()
   ).signAndSend(sudoKey)
   
   # Verify pause active
   polkadot-js-api query.economy.isPaused
   # Should return: true
   ```

3. **Freeze Attacker Accounts (5 minutes)**
   ```bash
   # Identify attacker addresses (check transaction logs)
   grep "Treasury.Proposed" /var/log/substrate/node.log
   
   # Freeze accounts (governance call)
   polkadot-js-api tx.governance.freezeAccounts([
     '5GAttacker1...',
     '5GAttacker2...'
   ]).signAndSend(councilKey)
   ```

4. **Snapshot State (10 minutes)**
   ```bash
   # Export blockchain state (for forensics)
   polkadot-js-api export-state \
     --output /backup/emergency-snapshot-$(date +%s).json
   
   # Calculate stolen amount
   polkadot-js-api query.system.events | \
     grep "Treasury.Awarded" | \
     awk '{sum+=$3} END {print sum}'
   ```

5. **Deploy Patch (60 minutes)**
   - See Section 3.3 (Eradication)

6. **Restore Services (15 minutes)**
   ```bash
   # Unpause Economy pallet
   polkadot-js-api tx.sudo.sudo(
     api.tx.economy.resume()
   ).signAndSend(sudoKey)
   
   # Monitor for exploit recurrence
   tail -f /var/log/substrate/node.log | grep "Treasury"
   ```

**Post-Incident:**
- Compensate affected users (governance proposal)
- Update multi-sig thresholds (if bypassed)
- Implement additional monitoring (treasury balance alerts)

---

### 5.2 Bridge Double-Spending Attack

**Scenario:** Attacker withdraws same funds on both chains

**Detection Signs:**
- Duplicate withdrawal proofs submitted
- Bridge reserve balance mismatch (BelizeChain vs Ethereum)
- Relayer signature inconsistencies

**Response Steps:**

1. **Pause Bridge (2 minutes)**
   ```bash
   # Pause Ethereum bridge contract (Solidity)
   cast send $BRIDGE_CONTRACT "pause()" \
     --private-key $ADMIN_KEY \
     --rpc-url $ETH_RPC
   
   # Pause BelizeChain bridge pallet
   polkadot-js-api tx.sudo.sudo(
     api.tx.interoperability.pauseBridge("Ethereum")
   ).signAndSend(sudoKey)
   ```

2. **Calculate Discrepancy (10 minutes)**
   ```bash
   # Query BelizeChain bridge reserve
   BELIZECHAIN_RESERVE=$(polkadot-js-api \
     query.interoperability.reserve "Ethereum" "USDC")
   
   # Query Ethereum bridge reserve
   ETH_RESERVE=$(cast call $BRIDGE_CONTRACT \
     "getReserve(address)" $USDC_ADDRESS \
     --rpc-url $ETH_RPC)
   
   # Calculate difference
   DISCREPANCY=$((BELIZECHAIN_RESERVE - ETH_RESERVE))
   echo "Discrepancy: $DISCREPANCY USDC"
   ```

3. **Identify Double-Spent Transactions (30 minutes)**
   ```bash
   # Export bridge transaction logs
   polkadot-js-api query.interoperability.transactions \
     --from-block $START_BLOCK \
     --to-block $END_BLOCK \
     --output bridge-tx.json
   
   # Compare with Ethereum logs
   cast logs --from-block $START_BLOCK \
     --to-block $END_BLOCK \
     --address $BRIDGE_CONTRACT \
     --rpc-url $ETH_RPC > eth-bridge-logs.json
   
   # Identify duplicates (Python script)
   python3 find_duplicates.py \
     bridge-tx.json eth-bridge-logs.json
   ```

4. **Revoke Compromised Relayers (15 minutes)**
   ```bash
   # If relayer private key compromised
   polkadot-js-api tx.interoperability.removeRelayer(
     '5GCompromisedRelayer...'
   ).signAndSend(adminKey)
   
   # Rotate remaining relayer keys (manual process)
   # Coordinate with relayer operators
   ```

5. **Deploy Patch (90 minutes)**
   - Fix vulnerability (e.g., add nonce validation)
   - Test extensively (cross-chain simulation)
   - Deploy to both chains (Ethereum + BelizeChain)

6. **Resume Bridge (30 minutes)**
   ```bash
   # Verify fix deployed
   # Re-enable bridge
   polkadot-js-api tx.sudo.sudo(
     api.tx.interoperability.resumeBridge("Ethereum")
   ).signAndSend(sudoKey)
   
   # Monitor for anomalies (24 hours)
   ```

**Post-Incident:**
- Insurance fund activation (if losses exceed reserves)
- Notify partner chains (Acala, Moonbeam)
- Strengthen relayer security (hardware keys, MFA)

---

### 5.3 Consensus Failure (Chain Halt)

**Scenario:** Blockchain finality stalled, no new blocks produced

**Detection Signs:**
- Finality lag >60 seconds (GRANDPA stalled)
- No new blocks produced (BABE failure)
- Validator logs show errors (session rotation failed)

**Response Steps:**

1. **Emergency Validator Call (5 minutes)**
   ```bash
   # Automated SMS/phone call to all validators
   # "EMERGENCY: Consensus failure, join conference call immediately"
   
   # Conference bridge: +501-XXX-XXXX PIN: 12345
   # Backup: Zoom link in #validators-emergency
   ```

2. **Diagnose Issue (10 minutes)**
   ```bash
   # Check validator status
   for i in {1..5}; do
     curl https://validator$i.belizechain.org/health
   done
   
   # Check GRANDPA finality
   polkadot-js-api rpc.grandpa.roundState
   
   # Check BABE authorities
   polkadot-js-api query.babe.authorities
   
   # Common issues:
   # - Validator offline (health check fails)
   # - Network partition (validators can't communicate)
   # - Session rotation bug (authorities list empty)
   ```

3. **Apply Fix (Depends on Issue)**

   **If Validator Offline:**
   ```bash
   # Restart validator node
   ssh validator3.belizechain.org
   sudo systemctl restart belizechain-node
   
   # Monitor logs
   journalctl -u belizechain-node -f
   ```

   **If Network Partition:**
   ```bash
   # Force peer connections
   for i in {1..5}; do
     polkadot-js-api rpc.system.addReservedPeer(
       "/ip4/validator$i/tcp/30333/p2p/$PEER_ID"
     )
   done
   ```

   **If Runtime Bug:**
   ```bash
   # Emergency runtime upgrade (pre-approved patch)
   polkadot-js-api tx.sudo.sudoUncheckedWeight(
     api.tx.system.setCode(emergencyWasmCode),
     weight
   ).signAndSend(sudoKey)
   ```

4. **Monitor Recovery (30 minutes)**
   ```bash
   # Watch finality lag
   watch -n 5 'polkadot-js-api rpc.grandpa.roundState | \
     jq .best_finalized_block_number'
   
   # Should decrease to <5 seconds within 10 minutes
   ```

5. **Post-Recovery Validation (60 minutes)**
   ```bash
   # Verify all validators producing blocks
   polkadot-js-api query.session.validators
   
   # Check block production distribution
   # (no single validator dominating)
   
   # Run smoke tests (submit transactions)
   polkadot-js-api tx.balances.transfer(
     dest, 1000
   ).signAndSend(testKey)
   ```

**Post-Incident:**
- Root cause analysis (validator logs, network traces)
- Implement monitoring for early warning (finality lag alert at 10s, not 60s)
- Consider validator diversity (geographic, infrastructure provider)

---

### 5.4 DDoS Attack

**Scenario:** Overwhelming traffic targeting RPC nodes

**Detection Signs:**
- RPC latency spike (>5 seconds per request)
- High CPU/network usage on RPC nodes
- Cloudflare DDoS alerts
- User complaints (slow wallet, transactions timing out)

**Response Steps:**

1. **Enable Attack Mode (2 minutes)**
   ```bash
   # Cloudflare: Enable "Under Attack" mode
   curl -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/settings/security_level" \
     -H "Authorization: Bearer $CF_TOKEN" \
     -d '{"value":"under_attack"}'
   
   # This enables aggressive JS challenge (bots blocked)
   ```

2. **Reduce Rate Limits (5 minutes)**
   ```bash
   # Update NGINX rate limiting (all RPC nodes)
   ansible-playbook -i inventory/production \
     playbooks/nginx-ratelimit-emergency.yml \
     --extra-vars "limit=10r/s"  # Reduce from 100r/s to 10r/s
   
   # Apply immediately
   ansible all -a "nginx -s reload"
   ```

3. **Block Attacking IPs (10 minutes)**
   ```bash
   # Analyze access logs (identify top offenders)
   cat /var/log/nginx/access.log | \
     awk '{print $1}' | \
     sort | uniq -c | sort -rn | head -100 > attacking_ips.txt
   
   # Block via firewall (ufw)
   while read ip; do
     ufw insert 1 deny from $ip
   done < attacking_ips.txt
   
   # Or block via Cloudflare (API)
   while read ip; do
     curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/firewall/access_rules/rules" \
       -H "Authorization: Bearer $CF_TOKEN" \
       -d "{\"mode\":\"block\",\"configuration\":{\"target\":\"ip\",\"value\":\"$ip\"}}"
   done < attacking_ips.txt
   ```

4. **Scale Up Infrastructure (15 minutes)**
   ```bash
   # Increase RPC capacity on Ceiba/self-hosted stack
   docker compose -f /home/wicked/Projects/infra/docker-compose.yml up -d --scale belizechain-rpc=10

   # Verify containers online
   docker compose -f /home/wicked/Projects/infra/docker-compose.yml ps belizechain-rpc
   ```

5. **Monitor Attack Mitigation (60 minutes)**
   ```bash
   # Watch request rate (should decrease)
   watch -n 10 'curl -s https://rpc.belizechain.org/metrics | \
     grep rpc_requests_total'
   
   # Watch latency (should return to normal)
   curl -w "@curl-format.txt" -o /dev/null -s \
     https://rpc.belizechain.org
   
   # Target: <500ms response time
   ```

6. **Restore Normal Operations (30 minutes)**
   ```bash
   # Disable Cloudflare "Under Attack" mode
   curl -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/settings/security_level" \
     -H "Authorization: Bearer $CF_TOKEN" \
     -d '{"value":"medium"}'
   
   # Restore normal rate limits
   ansible-playbook -i inventory/production \
     playbooks/nginx-ratelimit-restore.yml
   
   # Scale down RPC nodes (gradually)
   docker compose -f /home/wicked/Projects/infra/docker-compose.yml up -d --scale belizechain-rpc=5
   # Wait 30 minutes, then scale to 3 if stable
   ```

**Post-Incident:**
- Analyze attack pattern (UDP flood, HTTP flood, amplification?)
- Update WAF rules (block attack signatures)
- Consider DDoS mitigation service upgrade (Cloudflare Pro → Enterprise)

---

## 6. On-Call Rotation

### 6.1 Schedule

**Primary On-Call:**
- **Week 1:** Alice (Security Lead)
- **Week 2:** Bob (DevOps Lead)
- **Week 3:** Carol (Core Developer)
- **Week 4:** David (Infrastructure Engineer)
- **Repeat cycle**

**Secondary On-Call (Backup):**
- Always 1 person behind (Week 1: Bob is backup for Alice)

**Escalation:**
- If primary doesn't respond in 15 minutes, secondary alerted
- If secondary doesn't respond in 15 minutes, CTO alerted

**On-Call Expectations:**
- **Availability:** 24/7 during on-call week
- **Response Time:** <15 minutes for SEV-1, <1 hour for SEV-2
- **Laptop Access:** Must have laptop with VPN access
- **Phone Access:** Phone charged, notifications enabled

**On-Call Compensation:**
- **Weekly Stipend:** $500 USD (on-call week)
- **Incident Bonus:** $200 per SEV-1 handled, $100 per SEV-2

---

### 6.2 On-Call Handoff

**Handoff Checklist (End of Week):**

```markdown
# On-Call Handoff: Alice → Bob

**Date:** 2025-11-08
**Outgoing:** Alice
**Incoming:** Bob

## Current Incidents
- [x] INC-2025-11-04-001: Resolved, PIR published
- [ ] INC-2025-11-07-002: In progress (SEV-3, investigating XSS report)
  - Status: Waiting for bug bounty researcher to provide more details
  - Next action: Follow up on Monday if no response

## Ongoing Issues
- Validator3 intermittent connectivity (monitoring, non-critical)
- GraphQL API latency increased 20% (investigating, ticket #1234)

## Upcoming Maintenance
- Runtime upgrade scheduled 2025-11-15 (planned, non-emergency)
- RPC node OS patching 2025-11-12 (coordinated with DevOps)

## Recent Changes
- Deployed patch for overflow vulnerability (2025-11-04)
- Updated rate limiting rules (2025-11-06)
- Rotated validator keys (routine, 2025-11-05)

## Notes
- Audit firm (Trail of Bits) starts next week, expect questions
- Bug bounty program launches Dec 1, prepare for influx of reports
- CTO out of office Nov 10-12, escalate critical incidents to CEO

## Handoff Call
- Completed: 2025-11-08 17:00 UTC
- Duration: 30 minutes
- Next handoff: 2025-11-15 17:00 UTC (Bob → Carol)

**Acknowledged:**
- [x] Alice (outgoing)
- [x] Bob (incoming)
```

---

## 7. Tools & Access

### 7.1 Required Access

**All On-Call Engineers Must Have:**

1. **VPN Access:**
   - WireGuard VPN (production network)
   - Credentials: In 1Password vault "On-Call"

2. **SSH Access:**
   - All validators, RPC nodes, infrastructure
   - SSH keys: Yubikey (hardware key required)

3. **Admin Keys:**
   - Sudo key (runtime upgrades, emergency pause)
   - Multi-sig signer (treasury, governance)
   - Cloudflare admin (DDoS mitigation)

4. **Monitoring Access:**
   - Grafana (read/write): https://grafana.belizechain.org
   - Prometheus (read-only): https://prometheus.internal.belizechain.org
   - Loki (logs): https://loki.internal.belizechain.org
   - Sentry (errors): https://sentry.io/belizechain

5. **Communication Access:**
   - Slack workspace (admin)
   - PagerDuty (on-call schedule management)
   - Email (security@belizechain.org, shared inbox)
   - Phone tree (all validator operators, CTO, CEO)

6. **Documentation Access:**
   - Incident runbooks: `/docs/security/`
   - Playbooks (Ansible): `/infra/ansible/playbooks/`
   - Architecture docs: `/docs/DEVELOPMENT_GUIDE.md`

---

### 7.2 Incident Management Tools

**Primary Tools:**

1. **PagerDuty:**
   - Alerting, escalation, on-call scheduling
   - Integration: Prometheus, Grafana, Sentry
   - Mobile app: iOS/Android (push notifications)

2. **Slack:**
   - Incident coordination (#incident-* channels)
   - Status updates (every 15 minutes for SEV-1)
   - Team communication

3. **Google Docs:**
   - Incident timeline (shared live document)
   - Post-incident reports (PIR)
   - Runbooks (collaborative editing)

4. **Zoom:**
   - Emergency conference calls (SEV-1)
   - Screen sharing (for live debugging)
   - Recording (for post-incident review)

**Backup Tools (Primary Failure):**

- Discord voice channels (if Zoom down)
- Telegram group (if Slack down)
- SMS alerts (if PagerDuty down)

---

## 8. Training & Drills

### 8.1 Tabletop Exercises

**Frequency:** Quarterly (every 3 months)

**Format:**
- 2-hour session (all team members)
- Facilitator presents scenario (fictional incident)
- Team walks through response (no actual systems touched)
- Debrief and improve runbooks

**Example Scenarios:**
1. Treasury draining exploit (SEV-1)
2. Bridge validator compromise (SEV-2)
3. DDoS attack (SEV-2)
4. KYC database breach (SEV-1)
5. Insider threat (malicious employee) (SEV-1)

---

### 8.2 Live Drills

**Frequency:** Semi-annually (every 6 months)

**Format:**
- 4-hour exercise (on-call + security team)
- Actual incident simulated on testnet
- Full response executed (detection → recovery)
- Timed and evaluated

**Example Drill:**
- **Scenario:** Flash loan attack on testnet DEX
- **Objective:** Detect, pause DEX, deploy patch, restore within 2 hours
- **Success Criteria:**
  - Detection <10 minutes
  - Containment <30 minutes
  - Patch deployment <60 minutes
  - Recovery <90 minutes
- **Evaluation:** Compare actual times to success criteria

---

### 8.3 Onboarding Training

**New On-Call Engineers:**

**Week 1: Orientation**
- Read all incident runbooks
- Shadow current on-call (1 week)
- Access setup (VPN, SSH, monitoring)

**Week 2: Hands-On**
- Simulated incident (controlled)
- Practice emergency procedures (testnet)
- Response time drill

**Week 3: Supervised On-Call**
- Take on-call (with mentor backup)
- Handle real incidents (with guidance)

**Week 4: Independent On-Call**
- Solo on-call (no mentor)
- Considered fully trained

**Certification:**
- Quiz on incident procedures (80% pass rate)
- Successfully handle 2+ incidents
- Manager approval

---

## 9. Compliance & Reporting

### 9.1 FSC Requirements

**Belize Financial Services Commission (FSC) Requirements:**

1. **Incident Notification:**
   - Critical incidents (SEV-1): Within 1 hour (phone)
   - High incidents (SEV-2): Within 24 hours (email)
   - Medium/Low: Quarterly summary report

2. **Data Breach Notification:**
   - Any PII exposure: Within 72 hours (mandatory)
   - Includes: What data, how many users, mitigation

3. **Annual Audit:**
   - Submit incident log (anonymized)
   - Demonstrate incident response capability
   - Tabletop exercise with FSC observer (optional)

---

### 9.2 Internal Reporting

**Monthly Security Report (To Board of Directors):**

```markdown
# BelizeChain Security Report - November 2025

## Incidents This Month
- Total incidents: 3 (1 SEV-2, 2 SEV-3)
- Mean time to detect (MTTD): 8 minutes (target: <15 min) ✅
- Mean time to resolve (MTTR): 2.5 hours (target: <4 hours) ✅
- False positive rate: 5% (target: <10%) ✅

## Notable Incidents
- INC-2025-11-04-001 (SEV-1): Treasury exploit, resolved in 3.7 hours
  - Impact: 47 users, $1.8M
  - Root cause: Integer overflow (patched)
  - Compensation: In progress

## Security Improvements
- Deployed overflow fuzz testing (100% coverage for Economy pallet)
- Launched bug bounty program (7 submissions, 2 valid)
- Completed tabletop exercise (consensus failure scenario)

## Upcoming Initiatives
- Trail of Bits audit starting December 2025
- Penetration testing (SR Labs) scheduled Q1 2026
- Formal verification for 5 critical pallets

## KPIs
- Uptime: 99.97% (target: 99.9%) ✅
- Incident response time: <15 min (target: <15 min) ✅
- Bug bounty payouts: $25,000 (on track for $500k/year)

## Risks
- Medium: Audit delayed by 2 weeks (vendor scheduling)
- Low: Validator centralization (5 validators, target 10+)
```

---

## 10. Continuous Improvement

### 10.1 Metrics & KPIs

**Track Monthly:**
- **MTTD (Mean Time To Detect):** Average time from incident start to detection
  - Target: <15 minutes (SEV-1), <1 hour (SEV-2)
- **MTTR (Mean Time To Resolve):** Average time from detection to resolution
  - Target: <4 hours (SEV-1), <24 hours (SEV-2)
- **False Positive Rate:** % of alerts that aren't real incidents
  - Target: <10%
- **Incident Frequency:** Number of incidents per month
  - Target: <5 SEV-1/2 per quarter

**Review Quarterly:**
- Incident trends (increasing? decreasing?)
- Response effectiveness (runbooks working?)
- Team performance (training gaps?)

---

### 10.2 Runbook Updates

**After Every Incident:**
- Update relevant runbook (add learnings)
- Improve detection (better alerts)
- Faster response (automate manual steps)

**Quarterly Review:**
- Review all runbooks (still accurate?)
- Update based on new pallets/features
- Remove obsolete procedures

**Version Control:**
- All runbooks in Git (`/docs/security/runbooks/`)
- Pull request required (peer review)
- Changelog maintained

---

## 11. Appendix

### A. Incident Severity Matrix

| Impact \ Likelihood | High | Medium | Low |
|---------------------|------|--------|-----|
| **Critical** (Fund loss, PII breach) | SEV-1 | SEV-1 | SEV-2 |
| **High** (Service down, exploit possible) | SEV-1 | SEV-2 | SEV-3 |
| **Medium** (Degradation, no exploit) | SEV-2 | SEV-3 | SEV-4 |
| **Low** (Informational, no impact) | SEV-3 | SEV-4 | SEV-4 |

---

### B. Contact Information

**Emergency Contacts (24/7):**
- **On-Call Engineer:** Via PagerDuty (automated)
- **Security Lead:** +501-XXX-XXXX (SMS, Voice)
- **CTO:** +501-XXX-XXXX (SEV-1 only)
- **CEO:** +501-XXX-XXXX (SEV-1 + reputational)

**External Contacts:**
- **FSC (Regulator):** +501-223-6194 (business hours), emergency@fsc.gov.bz
- **Law Enforcement:** Belize Police Department Cyber Crime Unit, +501-227-2222
- **Legal Counsel:** [Firm Name], +501-XXX-XXXX
- **PR Firm:** [Firm Name], +1-XXX-XXX-XXXX (crisis communications)

---

### C. Acronyms

- **FSC:** Financial Services Commission (Belize regulator)
- **MTTD:** Mean Time To Detect
- **MTTR:** Mean Time To Resolve
- **PIR:** Post-Incident Report
- **POC:** Proof of Concept
- **RPC:** Remote Procedure Call
- **SEV:** Severity
- **SLA:** Service Level Agreement
- **VPN:** Virtual Private Network

---

**Document Status:** ✅ PRODUCTION READY

**Last Tested:** Tabletop Exercise - October 2025 (consensus failure scenario)

**Next Review:** February 2026 (quarterly update)

---

*Built with 💎 for the sovereign nation of Belize 🇧🇿*
*Prepared to protect: citizen funds, national identity, digital sovereignty.*
