# BelizeChain Security Best Practices

**Version:** 1.0  
**Last Updated:** November 4, 2025  
**Status:** Mandatory for All Contributors  
**Authority:** CTO + Security Lead  

---

## 1. Executive Summary

This document defines mandatory security practices for all BelizeChain development, deployment, and operations. These practices apply to:
- Core blockchain developers (Rust pallets)
- Smart contract developers (ink! WASM)
- Frontend developers (TypeScript UI portals)
- DevOps engineers (infrastructure)
- Third-party contributors (community PRs)

**Enforcement:**
- ✅ All PRs MUST pass security checks (automated CI/CD gates)
- ✅ Security Lead approval required for sensitive changes (runtime, multi-sig, cryptography)
- ✅ Quarterly security training (mandatory for all team members)
- ❌ Security violations = immediate PR rejection + incident review

---

## 2. Secure Coding Guidelines

### 2.1 Rust/Substrate Pallet Development

**✅ DO: Use Checked Arithmetic**
```rust
// ✅ CORRECT: Use checked_add/checked_sub to prevent overflow
let new_balance = current_balance.checked_add(amount)
    .ok_or(Error::<T>::ArithmeticOverflow)?;

// ❌ WRONG: Wrapping arithmetic silently overflows
let new_balance = current_balance.wrapping_add(amount);  // ⚠️ DANGEROUS

// ❌ WRONG: Unchecked operators panic in debug, wrap in release
let new_balance = current_balance + amount;  // ⚠️ INCONSISTENT BEHAVIOR
```

**✅ DO: Validate All Inputs**
```rust
// ✅ CORRECT: Comprehensive input validation
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::transfer())]
pub fn transfer(
    origin: OriginFor<T>,
    dest: AccountIdLookupOf<T>,
    #[pallet::compact] value: T::Balance,
) -> DispatchResult {
    let sender = ensure_signed(origin)?;
    let receiver = T::Lookup::lookup(dest)?;
    
    // Validate: sender != receiver
    ensure!(sender != receiver, Error::<T>::SelfTransfer);
    
    // Validate: value > 0
    ensure!(!value.is_zero(), Error::<T>::ZeroAmount);
    
    // Validate: sender has sufficient balance
    let sender_balance = Self::balance(&sender);
    ensure!(sender_balance >= value, Error::<T>::InsufficientBalance);
    
    // Validate: receiver balance won't overflow
    let receiver_balance = Self::balance(&receiver);
    receiver_balance.checked_add(&value)
        .ok_or(Error::<T>::BalanceOverflow)?;
    
    // Execute transfer (validation complete)
    Self::do_transfer(&sender, &receiver, value)?;
    Ok(())
}
```

**✅ DO: Use BoundedVec for Storage**
```rust
// ✅ CORRECT: BoundedVec with reasonable limit
#[pallet::storage]
pub type DistrictMembers<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    DistrictId,
    BoundedVec<T::AccountId, ConstU32<1000>>,  // Max 1000 members per district
    ValueQuery,
>;

// ❌ WRONG: Unbounded Vec (DoS vulnerability)
pub type DistrictMembers<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    DistrictId,
    Vec<T::AccountId>,  // ⚠️ UNLIMITED SIZE
    ValueQuery,
>;
```

**✅ DO: Implement Proper Weight Functions**
```rust
// ✅ CORRECT: Accurate weight calculation
impl<T: Config> WeightInfo for SubstrateWeight<T> {
    fn transfer() -> Weight {
        Weight::from_parts(50_000_000, 0)  // 50ms computational
            .saturating_add(T::DbWeight::get().reads(2))  // Sender + receiver balance
            .saturating_add(T::DbWeight::get().writes(2))  // Update both balances
    }
    
    fn transfer_batch(n: u32) -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(Weight::from_parts(10_000_000, 0).saturating_mul(n as u64))  // 10ms per transfer
            .saturating_add(T::DbWeight::get().reads(1 + n as u64))  // Sender + n receivers
            .saturating_add(T::DbWeight::get().writes(1 + n as u64))
    }
}

// ❌ WRONG: Arbitrary weights
fn transfer() -> Weight {
    Weight::from_parts(1_000_000, 0)  // ⚠️ TOO LOW (DoS vulnerability)
}
```

**✅ DO: Emit Events for State Changes**
```rust
// ✅ CORRECT: Emit detailed events
Self::do_transfer(&sender, &receiver, value)?;

Self::deposit_event(Event::Transfer {
    from: sender.clone(),
    to: receiver.clone(),
    amount: value,
    timestamp: <frame_system::Pallet<T>>::block_number(),
});
```

**✅ DO: Handle Errors Explicitly**
```rust
// ✅ CORRECT: Comprehensive error handling
#[pallet::error]
pub enum Error<T> {
    /// Transfer amount is zero
    ZeroAmount,
    /// Sender has insufficient balance
    InsufficientBalance,
    /// Transfer to self not allowed
    SelfTransfer,
    /// Arithmetic overflow
    ArithmeticOverflow,
    /// Balance would overflow
    BalanceOverflow,
    /// Account is frozen
    AccountFrozen,
    /// Daily limit exceeded
    DailyLimitExceeded,
}

// ❌ WRONG: Generic errors
pub enum Error<T> {
    TransferFailed,  // ⚠️ TOO VAGUE
}
```

---

### 2.2 Smart Contract Development (ink!)

**✅ DO: Use PSP Standards**
```rust
// ✅ CORRECT: Implement PSP22 (ERC-20 equivalent)
use ink::prelude::string::String;
use ink::storage::Mapping;

#[ink(storage)]
pub struct Token {
    total_supply: Balance,
    balances: Mapping<AccountId, Balance>,
    allowances: Mapping<(AccountId, AccountId), Balance>,
    name: String,
    symbol: String,
    decimals: u8,
}

#[ink(message)]
pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
    let from = self.env().caller();
    self.transfer_from_to(&from, &to, value)?;
    Ok(())
}
```

**✅ DO: Prevent Reentrancy**
```rust
// ✅ CORRECT: Checks-Effects-Interactions pattern
#[ink(storage)]
pub struct LendingPool {
    balances: Mapping<AccountId, Balance>,
    is_locked: bool,  // Reentrancy guard
}

#[ink(message)]
pub fn withdraw(&mut self, amount: Balance) -> Result<(), Error> {
    // Guard
    if self.is_locked {
        return Err(Error::Reentrant);
    }
    self.is_locked = true;
    
    let caller = self.env().caller();
    
    // Checks
    let balance = self.balances.get(&caller).unwrap_or(0);
    if balance < amount {
        self.is_locked = false;
        return Err(Error::InsufficientBalance);
    }
    
    // Effects (update state BEFORE external call)
    self.balances.insert(&caller, &(balance - amount));
    
    // Interactions (external call LAST)
    if self.env().transfer(caller, amount).is_err() {
        // Revert state on failure
        self.balances.insert(&caller, &balance);
        self.is_locked = false;
        return Err(Error::TransferFailed);
    }
    
    self.is_locked = false;
    Ok(())
}
```

**✅ DO: Use Access Control**
```rust
// ✅ CORRECT: Role-based access control
use ink::storage::Mapping;

#[ink(storage)]
pub struct AccessControlled {
    admin: AccountId,
    operators: Mapping<AccountId, bool>,
}

impl AccessControlled {
    fn only_admin(&self) -> Result<(), Error> {
        if self.env().caller() != self.admin {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
    
    fn only_operator(&self) -> Result<(), Error> {
        let caller = self.env().caller();
        if caller != self.admin && !self.operators.get(&caller).unwrap_or(false) {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}

#[ink(message)]
pub fn pause(&mut self) -> Result<(), Error> {
    self.only_admin()?;  // ✅ Access control enforced
    self.paused = true;
    Ok(())
}
```

---

### 2.3 TypeScript/JavaScript Development

**✅ DO: Validate User Input**
```typescript
// ✅ CORRECT: Input sanitization
import DOMPurify from 'dompurify';
import { z } from 'zod';

const TransferSchema = z.object({
  to: z.string().regex(/^5[A-Za-z0-9]{47}$/),  // Valid SS58 address
  amount: z.number().positive().finite(),
  memo: z.string().max(256).optional(),
});

function handleTransfer(input: unknown) {
  // Validate input structure
  const result = TransferSchema.safeParse(input);
  if (!result.success) {
    throw new Error(`Invalid input: ${result.error.message}`);
  }
  
  const { to, amount, memo } = result.data;
  
  // Sanitize memo (prevent XSS)
  const sanitizedMemo = memo ? DOMPurify.sanitize(memo) : undefined;
  
  // Execute transfer
  return api.tx.balances.transfer(to, amount).signAndSend(...);
}

// ❌ WRONG: No validation
function handleTransfer(to: string, amount: number, memo?: string) {
  return api.tx.balances.transfer(to, amount).signAndSend(...);  // ⚠️ UNSAFE
}
```

**✅ DO: Use Secure Storage**
```typescript
// ✅ CORRECT: Encrypt sensitive data
import { encrypt, decrypt } from '@metamask/browser-passworder';

async function storePrivateKey(privateKey: string, password: string) {
  // Encrypt before storing
  const encrypted = await encrypt(password, privateKey);
  localStorage.setItem('encrypted_key', JSON.stringify(encrypted));
}

async function retrievePrivateKey(password: string): Promise<string> {
  const encrypted = localStorage.getItem('encrypted_key');
  if (!encrypted) {
    throw new Error('No key found');
  }
  
  // Decrypt when retrieving
  const privateKey = await decrypt(password, JSON.parse(encrypted));
  return privateKey;
}

// ❌ WRONG: Store plaintext
localStorage.setItem('private_key', privateKey);  // ⚠️ NEVER DO THIS
```

**✅ DO: Implement Rate Limiting (Client-Side)**
```typescript
// ✅ CORRECT: Rate limiting for API calls
class RateLimiter {
  private requests: number[] = [];
  private maxRequests: number;
  private windowMs: number;
  
  constructor(maxRequests: number, windowMs: number) {
    this.maxRequests = maxRequests;
    this.windowMs = windowMs;
  }
  
  async limit<T>(fn: () => Promise<T>): Promise<T> {
    const now = Date.now();
    
    // Remove old requests
    this.requests = this.requests.filter(time => now - time < this.windowMs);
    
    // Check limit
    if (this.requests.length >= this.maxRequests) {
      throw new Error('Rate limit exceeded');
    }
    
    // Execute and record
    this.requests.push(now);
    return fn();
  }
}

const limiter = new RateLimiter(10, 60000);  // 10 requests per minute

// Usage
const balance = await limiter.limit(() => api.query.system.account(address));
```

---

## 3. Key Management

### 3.1 Private Key Storage

**✅ DO: Use Hardware Wallets (Production)**
- **Validators:** Ledger Nano X (mandatory)
- **Treasury Multi-Sig:** Yubikey + Ledger (required 4 of 7)
- **Admin Keys:** Trezor Model T (cold storage)

**✅ DO: Encrypt Keys at Rest**
```bash
# ✅ CORRECT: Encrypt keystore with strong password
subkey generate \
  --scheme sr25519 \
  --password-interactive \
  --output-type json > keystore.json

# Encrypt keystore file
gpg --symmetric --cipher-algo AES256 keystore.json
rm keystore.json  # Delete plaintext

# Store encrypted file securely
aws s3 cp keystore.json.gpg s3://belizechain-keys-backup/ --sse aws:kms
```

**❌ DON'T: Store Keys in Code**
```typescript
// ❌ WRONG: Hardcoded private key
const PRIVATE_KEY = '0x1234...';  // ⚠️ NEVER DO THIS

// ❌ WRONG: Private key in environment variable (plain text)
const key = process.env.PRIVATE_KEY;  // ⚠️ STILL DANGEROUS

// ✅ CORRECT: Use secure key management service
const key = await aws.secretsManager.getSecretValue({ SecretId: 'validator-key' });
```

---

### 3.2 Multi-Signature Procedures

**Treasury Operations (4-of-7 Multi-Sig):**

**Step 1: Proposal Creation**
```bash
# Proposer creates transaction
polkadot-js-api tx.treasury.proposeSpend(
  beneficiary,
  amount
)

# Get call hash
CALL_HASH=$(polkadot-js-api \
  encode-call-data \
  '{"module":"treasury","call":"proposeSpend","args":{"beneficiary":"...","amount":"..."}}' \
  | sha256sum)

echo "Call hash: $CALL_HASH"
```

**Step 2: Multi-Sig Approval (4 Signatures Required)**
```bash
# Signer 1: Approve
polkadot-js-api tx.multiSig.approveAsMulti(
  threshold=4,
  otherSignatories=[signer2, signer3, signer4, signer5, signer6, signer7],
  callHash=$CALL_HASH
).signAndSend(signer1)

# Signer 2, 3: Approve (same process)
# ...

# Signer 4: Approve and execute (threshold reached)
polkadot-js-api tx.multiSig.asMulti(
  threshold=4,
  otherSignatories=[...],
  call=treasuryProposalCall,
  maxWeight=defaultWeight
).signAndSend(signer4)

# ✅ Transaction executed (4 of 7 signatures verified)
```

**Emergency Multi-Sig Rotation (Compromised Signer):**
1. Emergency meeting (within 1 hour)
2. New multi-sig address created (excluding compromised signer)
3. Treasury funds migrated (requires 4-of-7 approval)
4. Update all pallet configurations (new multi-sig address)
5. Notify FSC (regulatory requirement)

---

### 3.3 Validator Key Rotation

**Scheduled Rotation (Every 90 Days):**

```bash
# Generate new session keys
NEW_KEYS=$(curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys"}' \
  http://localhost:9933 | jq -r .result)

# Submit key rotation transaction
polkadot-js-api tx.session.setKeys(
  newKeys=$NEW_KEYS,
  proof=0x00
).signAndSend(validatorController)

# Wait for session change (4 hours)
# Old keys automatically deactivated

# Backup old keys (archive for 12 months)
mv ~/.local/share/belizechain/chains/dev/keystore/OLDKEYS \
   ~/validator-key-archive/backup-$(date +%Y%m%d).json

# Verify new keys active
polkadot-js-api query.session.nextKeys(validatorStash)
```

---

## 4. Deployment Security

### 4.1 Runtime Upgrade Checklist

**Pre-Upgrade (1 Week Before):**
- [ ] Runtime compiled with `--release` flag
- [ ] WASM blob size <2MB (verify with `ls -lh runtime.wasm`)
- [ ] All pallets pass `cargo test --release`
- [ ] Zero Clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Benchmarks run successfully (if weight changes)
- [ ] Storage migrations tested on fork of mainnet
- [ ] Security review by 2+ senior devs (PR approval required)
- [ ] Governance proposal created (7-day voting period)

**Upgrade Day:**
```bash
# Build runtime
cd runtime
cargo build --release

# Extract WASM blob
WASM_BLOB=../target/release/wbuild/belizechain-runtime/belizechain_runtime.wasm

# Verify WASM hash (compare with governance proposal)
sha256sum $WASM_BLOB

# Submit upgrade transaction (via governance or sudo)
polkadot-js-api tx.sudo.sudoUncheckedWeight(
  api.tx.system.setCode(wasmBlob),
  weight={refTime: 1_000_000_000, proofSize: 200_000}
).signAndSend(sudoKey)

# Wait for block finalization (2 minutes)
# Monitor logs for errors
journalctl -u belizechain-node -f | grep ERROR
```

**Post-Upgrade (1 Hour After):**
- [ ] Verify runtime version updated (`api.rpc.state.getRuntimeVersion()`)
- [ ] Submit test transactions (transfer, governance vote, DEX swap)
- [ ] Monitor for errors (Grafana dashboard, Sentry)
- [ ] Check finality lag (<10 blocks)
- [ ] Announce upgrade complete (Twitter, Discord)

---

### 4.2 Node Deployment

**Validator Node Security Hardening:**

```bash
# 1. Firewall configuration (only allow essential ports)
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 30333/tcp  # P2P (libp2p)
sudo ufw allow 22/tcp     # SSH (restrict to bastion host only)
sudo ufw enable

# 2. Disable unnecessary services
sudo systemctl disable bluetooth
sudo systemctl disable cups
sudo systemctl stop snapd

# 3. SSH hardening
sudo vi /etc/ssh/sshd_config
# PermitRootLogin no
# PasswordAuthentication no
# PubkeyAuthentication yes
# AllowUsers validator-admin
sudo systemctl restart sshd

# 4. Enable automatic security updates
sudo apt install unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades

# 5. Install fail2ban (brute force protection)
sudo apt install fail2ban
sudo systemctl enable fail2ban

# 6. Run node as non-root user
sudo useradd -m -s /bin/bash belizechain
sudo -u belizechain ./belizechain-node \
  --validator \
  --name ValidatorBelize \
  --base-path /var/lib/belizechain

# 7. Enable systemd service (auto-restart)
sudo vi /etc/systemd/system/belizechain-node.service
# [Unit]
# Description=BelizeChain Validator
# After=network.target
#
# [Service]
# User=belizechain
# ExecStart=/usr/local/bin/belizechain-node --validator
# Restart=always
# RestartSec=10
#
# [Install]
# WantedBy=multi-user.target

sudo systemctl enable belizechain-node
sudo systemctl start belizechain-node
```

---

### 4.3 Smart Contract Deployment

**Pre-Deployment Checklist:**
- [ ] Contract compiled with optimizations (`cargo contract build --release`)
- [ ] Contract size <1MB (check `metadata.json`)
- [ ] Unit tests pass (`cargo test`)
- [ ] Integration tests pass (deploy to testnet)
- [ ] Fuzz testing (AFL, Honggfuzz, 24+ hours)
- [ ] Security audit (OpenZeppelin, Trail of Bits)
- [ ] Gas optimization (minimize storage operations)

**Deployment Process:**
```bash
# 1. Deploy to testnet (Rococo/Canvas)
cargo contract upload --suri //Alice \
  --url wss://canvas-rpc.polkadot.io

# 2. Instantiate contract (testnet)
cargo contract instantiate \
  --constructor new \
  --args 1000000 "BelizeCoin" "BLC" 18 \
  --suri //Alice \
  --url wss://canvas-rpc.polkadot.io

# 3. Test contract functions (testnet)
cargo contract call \
  --contract $CONTRACT_ADDRESS \
  --message transfer \
  --args $RECIPIENT 100 \
  --suri //Alice \
  --url wss://canvas-rpc.polkadot.io

# 4. Monitor for 7 days (testnet)
# - Check for unexpected errors
# - Review transaction logs
# - Verify state transitions

# 5. Deploy to mainnet (if testnet successful)
cargo contract upload --suri //Deployer \
  --url wss://rpc.belizechain.org

# 6. Verify contract code (block explorer)
# Upload source code + metadata to Kijka Explorer
```

---

## 5. Operational Security

### 5.1 Access Control

**Role-Based Access Control (RBAC):**

| Role | Access | Tools |
|------|--------|-------|
| **Validator Operator** | Validator nodes (SSH), monitoring dashboards (read-only) | WireGuard VPN, SSH keys, Grafana viewer |
| **Core Developer** | Git repo, testnet nodes, CI/CD (write) | GitHub, GitLab CI, Kubernetes (dev namespace) |
| **DevOps Engineer** | All infrastructure, production nodes (read/write) | Kubernetes (all namespaces), AWS console, Terraform |
| **Security Lead** | All systems, incident response, audit logs | Grafana admin, PagerDuty, Cloudflare, vault access |
| **Treasury Signer** | Multi-sig wallet (4-of-7) | Ledger Nano X, Polkadot.js apps |

**Access Review (Quarterly):**
- Remove inactive users (no activity >90 days)
- Rotate shared credentials (VPN, SSH CA)
- Audit privileged actions (sudo commands, AWS root usage)

---

### 5.2 Incident Handling

**Immediate Actions (SEV-1 Incident):**
1. **Notify on-call engineer** (PagerDuty, <15 minutes response)
2. **Create incident channel** (Slack #incident-YYYY-MM-DD-NNN)
3. **Activate emergency procedures** (pause pallets if needed)
4. **Document timeline** (Google Doc, shared with team)
5. **Notify stakeholders** (CTO, CEO, FSC if required)

**Communication Template (Public Announcement):**
```
🔒 Security Update

We're investigating a security incident affecting [pallet/service].
As a precaution, we've temporarily paused [functionality].

Status: https://status.belizechain.org
ETA: Services restored by [TIME]
Updates: Every 30 minutes on Twitter

User funds are safe. Thank you for your patience. 🇧🇿
```

---

### 5.3 Backup & Recovery

**Automated Backups:**

```bash
#!/bin/bash
# backup.sh - Run daily via cron

# Blockchain state snapshot
belizechain-node export-state \
  --chain dev \
  --pruning archive \
  > /backup/state-$(date +%Y%m%d).json

# Compress
gzip /backup/state-$(date +%Y%m%d).json

# Upload to S3 (encrypted)
aws s3 cp /backup/state-$(date +%Y%m%d).json.gz \
  s3://belizechain-backups/state/ \
  --sse aws:kms \
  --storage-class GLACIER  # Long-term archive

# PostgreSQL database (SubQuery indexed data)
pg_dump -U postgres belizechain_indexed \
  | gzip > /backup/db-$(date +%Y%m%d).sql.gz

aws s3 cp /backup/db-$(date +%Y%m%d).sql.gz \
  s3://belizechain-backups/db/ \
  --sse aws:kms

# Cleanup old local backups (keep 7 days)
find /backup -name "state-*.json.gz" -mtime +7 -delete
find /backup -name "db-*.sql.gz" -mtime +7 -delete

# Verify backup integrity
sha256sum /backup/state-$(date +%Y%m%d).json.gz > /backup/checksums.txt
```

**Recovery Testing (Quarterly):**
- Restore blockchain state from backup (isolated node)
- Verify block height matches production
- Replay last 1000 transactions (validate state consistency)

---

## 6. Compliance & Auditing

### 6.1 Audit Logging

**Log All Privileged Actions:**

```rust
// Rust pallet example
#[pallet::call_index(10)]
#[pallet::weight(T::WeightInfo::emergency_pause())]
pub fn emergency_pause(origin: OriginFor<T>) -> DispatchResult {
    ensure_root(origin.clone())?;  // Sudo only
    
    let caller = ensure_signed(origin)?;
    
    // Log privileged action (immutable audit trail)
    log::warn!(
        target: "runtime::economy",
        "🚨 EMERGENCY PAUSE activated by {:?} at block {:?}",
        caller,
        <frame_system::Pallet<T>>::block_number()
    );
    
    IsPaused::<T>::put(true);
    Self::deposit_event(Event::EmergencyPauseActivated { by: caller });
    Ok(())
}
```

**Centralized Audit Log (Loki):**
```logql
# Query all privileged actions
{job="substrate"} |= "EMERGENCY" or "SUDO" or "MULTI_SIG"
| json
| line_format "{{.timestamp}} | {{.caller}} | {{.action}}"
```

---

### 6.2 Compliance Documentation

**FSC Annual Report (Required):**
- Security incident log (anonymized)
- Penetration testing results (executive summary)
- Third-party audit reports (Trail of Bits, OpenZeppelin)
- Access control review (user list, role changes)
- Backup & recovery testing (quarterly results)

**Data Protection (GDPR-Equivalent for Belize):**
- PII encryption at rest (AES-256)
- PII encryption in transit (TLS 1.3)
- Data retention policy (KYC: 7 years, logs: 1 year)
- Right to erasure (delete user data on request)
- Breach notification (72 hours to FSC + affected users)

---

## 7. Training & Awareness

### 7.1 Security Training (Mandatory)

**Onboarding (Week 1):**
- Security best practices overview (this document)
- Threat modeling for blockchain applications
- Incident response procedures
- Key management protocols

**Quarterly Training (All Team):**
- Latest vulnerabilities (CVE reviews)
- Recent incidents (post-mortem analysis)
- Tabletop exercises (simulate SEV-1 incidents)
- Security tool updates (new Clippy rules, audit tools)

**Annual Certification (Engineers):**
- OWASP Top 10 (Web Application Security)
- SANS Blockchain Security (if available)
- Internal quiz (80% pass rate required)

---

### 7.2 Security Champions Program

**Nominate Security Champions (1 per team):**
- Core Blockchain: Alice
- Smart Contracts: Bob
- Frontend: Carol
- DevOps: David

**Responsibilities:**
- Weekly security review (PRs from team)
- Monthly threat modeling sessions
- Coordinate with Security Lead
- Triage security reports (bug bounty, CVEs)

---

## 8. Appendix

### A. Security Checklist (Pre-PR)

**Before Submitting Pull Request:**
- [ ] Run `cargo clippy -- -D warnings` (zero warnings)
- [ ] Run `cargo test --all` (all tests pass)
- [ ] Run `cargo audit` (no high/critical CVEs)
- [ ] Add unit tests (80%+ coverage target)
- [ ] Update documentation (inline comments, README)
- [ ] Security self-review (checklist below)

**Security Self-Review:**
- [ ] No hardcoded secrets (API keys, private keys)
- [ ] Input validation (all user-provided data)
- [ ] Arithmetic operations use `checked_*` functions
- [ ] Storage items use `BoundedVec` (not `Vec`)
- [ ] Weight functions accurate (benchmarked)
- [ ] Events emitted for state changes
- [ ] Error messages descriptive (not generic)
- [ ] Access control enforced (`ensure_signed`, `ensure_root`)

---

### B. Security Tools

**Recommended Tools:**

**Rust/Substrate:**
- `cargo-audit` - CVE scanning
- `cargo-clippy` - Linting
- `cargo-fuzz` - Fuzz testing (AFL, Honggfuzz)
- `kani` - Formal verification (Rust verifier)

**Smart Contracts:**
- `cargo-contract` - Build, deploy, call contracts
- `Slither` - Static analysis (Solidity, can adapt for ink!)
- `Mythril` - Symbolic execution

**Web/API:**
- `npm audit` - JavaScript dependency scanning
- `Snyk` - Dependency vulnerability scanning
- `OWASP ZAP` - Web application security testing
- `Burp Suite` - Manual penetration testing

**Infrastructure:**
- `Trivy` - Docker image scanning
- `Kubesec` - Kubernetes manifest analysis
- `Terraform Security` - IaC security scanning

---

**Document Status:** ✅ MANDATORY FOR ALL CONTRIBUTORS

**Enforcement:** Security Lead + CTO approval required for exceptions

**Next Review:** Quarterly (February 2026)

---

*Built with 💎 for the sovereign nation of Belize 🇧🇿*  
*Security is everyone's responsibility. Report vulnerabilities: security@belizechain.org*
