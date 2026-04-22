# 🆘 BelizeChain Disaster Recovery Guide

**Version**: Rolling (testnet)  
**Updated**: November 4, 2025  
**Criticality**: ESSENTIAL - Review quarterly  

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Backup Procedures](#backup-procedures)
3. [Recovery Scenarios](#recovery-scenarios)
4. [Emergency Protocols](#emergency-protocols)
5. [Testing & Validation](#testing--validation)

---

## Overview

Disaster recovery planning is MANDATORY for all validators. This guide covers backup procedures, recovery scenarios, and emergency protocols to minimize downtime and protect staked funds.

### Recovery Time Objectives (RTO)

| Scenario | Target RTO | Maximum Acceptable Downtime |
|----------|-----------|----------------------------|
| **Node crash** | 5 minutes | 15 minutes |
| **Hardware failure** | 30 minutes | 2 hours |
| **Data corruption** | 1 hour | 4 hours |
| **Datacenter outage** | 2 hours | 6 hours |
| **Key compromise** | 4 hours | 24 hours |

### Recovery Point Objectives (RPO)

| Data Type | Backup Frequency | Maximum Data Loss |
|-----------|------------------|-------------------|
| **Chain state** | Continuous | None (re-sync) |
| **Session keys** | On generation | None (critical) |
| **Configuration** | Daily | 24 hours |
| **Monitoring data** | Hourly | 1 hour |

---

## Backup Procedures

### 1. Critical Keys Backup

**NEVER store unencrypted keys on any internet-connected device.**

#### Stash Key (Cold Storage)

```bash
# Stash key should NEVER be on validator server
# Store on hardware wallet (Ledger/Trezor) or paper wallet

# Paper wallet backup procedure:
# 1. Write down 12-word mnemonic on PAPER (pen/pencil)
# 2. Store in fireproof safe (primary location)
# 3. Create duplicate copy (secondary location)
# 4. NEVER photograph or type mnemonic

# Hardware wallet backup:
# 1. Initialize Ledger/Trezor
# 2. Write down recovery phrase (included recovery card)
# 3. Store recovery card in safe
# 4. Test recovery on separate device
```

#### Controller Key (Hot Wallet)

```bash
# Export controller key (encrypted)
cd ~/belizechain-backup
mkdir -p keys
chmod 700 keys

# Option 1: Export from Polkadot.js extension
# 1. Open extension → Click account → Export Account
# 2. Enter password → Download JSON file
# 3. Move to backup directory:
mv ~/Downloads/controller-account.json ~/belizechain-backup/keys/

# Option 2: Create encrypted backup of mnemonic
cat > ~/belizechain-backup/keys/controller-mnemonic.txt << EOF
Controller Account Mnemonic:
[PASTE YOUR 12 WORDS HERE]
Created: $(date)
Purpose: Manages stash account, sets session keys
EOF

# Encrypt with GPG
gpg --symmetric --cipher-algo AES256 ~/belizechain-backup/keys/controller-mnemonic.txt

# Verify encryption
gpg -d ~/belizechain-backup/keys/controller-mnemonic.txt.gpg

# Delete plaintext (CRITICAL)
shred -u ~/belizechain-backup/keys/controller-mnemonic.txt

# Store encrypted .gpg file on:
# 1. USB drive (store in safe)
# 2. Cloud storage (encrypted)
# 3. Password manager (as secure note)
```

#### Session Keys

```bash
# Session keys are stored in node database
# Backup is NOT required (can regenerate)
# But recommended for faster recovery

# Backup session keys (generated from author_rotateKeys)
cd ~/belizechain-backup/keys

cat > session-keys.txt << EOF
Session Keys (Hex):
0x1234567890abcdef... [PASTE YOUR SESSION KEYS HERE]

Generated: $(date)
Set On-Chain: Yes/No
Validator: YourValidatorName
EOF

# Encrypt
gpg --symmetric --cipher-algo AES256 session-keys.txt

# Delete plaintext
shred -u session-keys.txt
```

#### Node P2P Key

```bash
# Backup libp2p node key
cd ~/belizechain-backup/keys

# Copy node key (CRITICAL: This identifies your node)
sudo cp /data/belizechain/chains/belizechain_testnet/network/secret_ed25519 \
       ~/belizechain-backup/keys/node-key

# Encrypt
gpg --symmetric --cipher-algo AES256 ~/belizechain-backup/keys/node-key

# Delete plaintext
shred -u ~/belizechain-backup/keys/node-key

# Store encrypted file securely
```

### 2. Configuration Backup

```bash
# Create backup script
cat > ~/belizechain-backup/backup-config.sh << 'EOF'
#!/bin/bash

BACKUP_DIR=~/belizechain-backup/configs
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="belizechain-config-$DATE.tar.gz"

mkdir -p $BACKUP_DIR

# Backup configurations
tar czf $BACKUP_DIR/$BACKUP_FILE \
    ~/belizechain-config/ \
    /etc/systemd/system/belizechain-*.service \
    /etc/prometheus/ \
    /etc/alertmanager/ \
    /etc/grafana/ \
    ~/.bashrc \
    ~/.profile

echo "Config backup saved: $BACKUP_FILE"

# Keep only last 30 backups
cd $BACKUP_DIR
ls -t | tail -n +31 | xargs -r rm

# Encrypt backup
gpg --symmetric --cipher-algo AES256 $BACKUP_FILE

# Upload to cloud (optional)
# rclone copy $BACKUP_FILE.gpg remote:belizechain-backups/
EOF

chmod +x ~/belizechain-backup/backup-config.sh

# Add to cron (daily at 2 AM)
crontab -e
# Add line:
0 2 * * * /home/belizechain/belizechain-backup/backup-config.sh
```

### 3. Chain State Backup (Optional)

**Note**: Chain state can always be re-synced from network. Only backup for FAST recovery.

```bash
# Backup chain database (LARGE - ~100-500 GB)
# Only recommended for archive nodes or critical validators

# Stop node
sudo systemctl stop belizechain-validator

# Backup database
tar czf /backup/belizechain-db-$(date +%Y%m%d).tar.gz \
    /data/belizechain/chains/belizechain_testnet/db/

# Restart node
sudo systemctl start belizechain-validator

# Alternative: Use LVM snapshots for zero-downtime backup
# (Requires LVM setup - see Advanced Configuration)
```

### 4. Monitoring Data Backup

```bash
# Backup Prometheus data (metrics history)
sudo systemctl stop prometheus
sudo tar czf /backup/prometheus-data-$(date +%Y%m%d).tar.gz \
    /var/lib/prometheus/
sudo systemctl start prometheus

# Backup Grafana dashboards
sudo tar czf /backup/grafana-data-$(date +%Y%m%d).tar.gz \
    /var/lib/grafana/
```

---

## Recovery Scenarios

### Scenario 1: Node Crash (Process Died)

**Symptoms**: systemd shows "failed" status, node not responding

**Recovery**:
```bash
# 1. Check logs for crash reason
sudo journalctl -u belizechain-validator -n 100

# 2. Common fixes:
# - Out of memory: Increase RAM or add swap
# - Disk full: Clear old logs, increase storage
# - Corrupted database: See Scenario 3

# 3. Restart node
sudo systemctl start belizechain-validator

# 4. Verify recovery
sudo systemctl status belizechain-validator
curl http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' | jq

# Recovery Time: ~5 minutes
```

### Scenario 2: Hardware Failure (Server Dead)

**Symptoms**: Server unresponsive, SSH timeout, datacenter alert

**Recovery** (Hot Failover):
```bash
# Assumption: You have a backup server with node already synced

# On backup server:

# 1. Stop current node (if running)
sudo systemctl stop belizechain-validator

# 2. Copy session keys from primary (if different node)
# (Only if you have backup - see Keys Backup section)
# If not, you'll need to generate new keys and set them on-chain

# 3. Update systemd service with correct chain spec
sudo nano /etc/systemd/system/belizechain-validator.service
# Ensure: --chain points to correct spec
# Ensure: --name is YOUR validator name

# 4. Start node
sudo systemctl start belizechain-validator

# 5. Wait for sync (if behind)
# Check: curl http://localhost:9944 -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}'

# 6. Verify block production resumes
# Check Polkadot.js Apps → Network → Explorer

# Recovery Time: 30 minutes - 2 hours (depending on sync)
```

**Recovery** (Cold Start - No Backup Server):
```bash
# 1. Provision new server (see TESTNET_DEPLOYMENT.md)
# 2. Install node binary
# 3. Restore configuration from backup
cd ~/belizechain-backup/configs
gpg -d belizechain-config-YYYYMMDD-HHMMSS.tar.gz.gpg | tar xz -C /

# 4. Start node and wait for sync (~2-6 hours)
sudo systemctl start belizechain-validator

# 5. Node will automatically resume validation once synced

# Recovery Time: 2-6 hours (full sync required)
```

### Scenario 3: Data Corruption (Database Errors)

**Symptoms**: Errors like "failed to import block", "database corruption", "invalid trie"

**Recovery**:
```bash
# 1. Stop node
sudo systemctl stop belizechain-validator

# 2. Clear corrupted database
rm -rf /data/belizechain/chains/belizechain_testnet/db/full

# 3. Start node (will re-sync from network)
sudo systemctl start belizechain-validator

# 4. Monitor sync progress
sudo journalctl -u belizechain-validator -f

# Alternative: Restore from backup (if you have one)
# sudo tar xzf /backup/belizechain-db-YYYYMMDD.tar.gz -C /data/belizechain/

# Recovery Time: 2-6 hours (full re-sync)
```

### Scenario 4: Datacenter Outage (Geographic Failure)

**Symptoms**: Entire datacenter unreachable, regional network failure

**Recovery** (Geographic Failover):
```bash
# Assumption: You have a validator node in a different geographic region

# 1. SSH into backup region server
ssh belizechain@backup-region-ip

# 2. Verify chain sync status
curl http://localhost:9944 -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' | jq

# 3. If behind, wait for sync
# If synced, validation should already be happening

# 4. Update monitoring alerts to point to new IP
# Update Prometheus targets
sudo nano /etc/prometheus/prometheus.yml

# 5. Update DNS (if using domain name)
# Point validator.yourdomain.com to backup-region-ip

# Recovery Time: <5 minutes (if already synced)
#                2-6 hours (if needs sync)
```

### Scenario 5: Key Compromise (Security Incident)

**Symptoms**: Unauthorized transactions, suspicious activity, stolen device

**CRITICAL ACTIONS**:

```bash
# 1. IMMEDIATELY chill validator (stop validation)
# Via Polkadot.js Apps:
# - Connect to any RPC node
# - Go to Developer → Extrinsics
# - Select: staking.chill()
# - Sign with controller account
# - Submit transaction

# 2. Transfer all funds from compromised accounts
# To new, secure stash account

# 3. Rotate all keys
# a. Generate new stash account (hardware wallet)
# b. Generate new controller account
# c. Generate new session keys:
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys"}' \
     http://localhost:9944

# 4. Bond funds with new accounts
# Via Polkadot.js Apps → Staking

# 5. Set new session keys
# Developer → Extrinsics → session.setKeys()

# 6. Start validating with new keys
# Staking → Account Actions → Validate

# 7. Report incident
# Email: security@belizechain.org
# Include: Timeline, compromised accounts, suspected cause

# Recovery Time: 4-24 hours (depending on incident severity)
```

---

## Emergency Protocols

### Emergency Contact List

Keep this list up-to-date and accessible (printed copy + secure note):

```
PRIMARY CONTACT:
Name: [Your Name]
Phone: [Your Phone]
Email: [Your Email]

BACKUP OPERATOR:
Name: [Backup Name]
Phone: [Backup Phone]
Email: [Backup Email]

BELIZECHAIN SUPPORT:
Emergency: emergency@belizechain.org
Validators: validators@belizechain.org
Discord: https://discord.gg/belizechain
Telegram: @BelizeChainValidators

DATACENTER SUPPORT:
Provider: [Provider Name]
Support Phone: [24/7 Number]
Ticket Portal: [URL]
Account ID: [Your Account ID]

HARDWARE WALLET SUPPORT:
Ledger: https://support.ledger.com
Trezor: https://trezor.io/support

EMERGENCY SEED PHRASE LOCATION:
Primary: [Safe deposit box, bank vault, etc.]
Backup: [Secondary secure location]
```

### Network Emergency Procedures

In case of network-wide emergency (e.g., consensus failure, security vulnerability):

1. **Monitor Discord #announcements channel** (24/7)
2. **Subscribe to emergency mailing list**: emergency@belizechain.org
3. **Follow emergency upgrade procedures** (if issued)
4. **Do NOT panic-sell DALLA** (could cause further instability)

### Slashing Incident Response

If you are slashed:

```bash
# 1. Identify slash reason
# Check Polkadot.js Apps → Network → Staking → Slashes

# 2. Stop node IMMEDIATELY (prevent further slashing)
sudo systemctl stop belizechain-validator

# 3. Investigate root cause:
# - Double-signing: Multiple nodes with same session keys?
# - Unresponsiveness: Downtime >10% of era?
# - Equivocation: Forked chain, wrong chain spec?

# 4. Fix underlying issue

# 5. File appeal (if unjustified)
# Via governance: Treasury → Tip Request
# Provide evidence: logs, metrics, timeline

# 6. Re-stake (if still have funds) or exit validator program
```

---

## Testing & Validation

### Quarterly DR Drill (MANDATORY)

Perform these tests every 3 months:

#### Test 1: Key Recovery
```bash
# 1. Decrypt backup keys
gpg -d ~/belizechain-backup/keys/controller-mnemonic.txt.gpg

# 2. Import to Polkadot.js extension (use TEST network)
# 3. Verify account matches expected address
# 4. Delete imported account after test
```

#### Test 2: Configuration Restore
```bash
# 1. Provision new test VM
# 2. Copy encrypted config backup
# 3. Decrypt and restore
# 4. Start node and verify sync
# 5. Destroy test VM after validation
```

#### Test 3: Failover Drill
```bash
# 1. Stop primary validator node
sudo systemctl stop belizechain-validator

# 2. Switch to backup node
# 3. Time how long until block production resumes
# 4. Verify no slash occurred
# 5. Switch back to primary node
```

### Recovery Validation Checklist

After any recovery:

- [ ] Node is synced (check block height vs. explorer)
- [ ] Peer count >25
- [ ] Session keys set correctly (verify on-chain)
- [ ] Blocks being produced (check Polkadot.js Apps)
- [ ] Monitoring alerts active (test with dummy alert)
- [ ] No slash penalties occurred (check staking page)
- [ ] Firewall rules restored
- [ ] Systemd services enabled (start on boot)

---

## Advanced: Hot Failover Setup

For mission-critical validators, setup automated failover:

```bash
# Primary node: Run with --reserved-only
# Backup node: Run as validator with same session keys
# Keepalived: Monitors primary, activates backup on failure

# See: docs/advanced/HOT_FAILOVER_SETUP.md (coming soon)
```

---

## Next Steps

- ✅ **Test your backups** (quarterly drill)
- ✅ **Document custom procedures** (unique to your setup)
- ✅ **Train backup operator** (if you have one)
- ✅ **Review security practices**: [../security/VALIDATOR_SECURITY.md](../security/VALIDATOR_SECURITY.md)

---

## Support

- **Emergency**: emergency@belizechain.org (24/7 mainnet)
- **Discord**: https://discord.gg/belizechain (#validators)
- **Forum**: https://forum.belizechain.org/c/operations

---

**Remember: The best disaster recovery plan is one you've TESTED.** 🛡️

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
