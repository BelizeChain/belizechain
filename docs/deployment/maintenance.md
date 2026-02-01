# 🔧 Maintenance Guide

**Ongoing maintenance tasks for BelizeChain nodes**

---

## 📋 Maintenance Schedule

| Task | Frequency | Time Required |
|------|-----------|---------------|
| **Monitor logs** | Daily | 5 minutes |
| **Check disk space** | Daily | 2 minutes |
| **Review alerts** | Daily | 5 minutes |
| **Update node software** | Monthly | 30 minutes |
| **Database cleanup** | Monthly | 15 minutes |
| **Security patches** | Weekly | 10 minutes |
| **Backup verification** | Weekly | 15 minutes |
| **Performance review** | Monthly | 30 minutes |
| **Disaster recovery drill** | Quarterly | 2 hours |

**Total**: ~30 minutes/day, 2 hours/quarter

---

## 📅 Daily Maintenance

### **1. Monitor Node Health** (5 minutes)

```bash
# Check service status
systemctl status belizechain

# Should show: "active (running)"
```

**Check sync status**:

```bash
curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
  | jq '.result'
```

**Expected output**:
```json
{
  "peers": 50,
  "isSyncing": false,
  "shouldHavePeers": true
}
```

**Red flags**:
- `peers < 5`: Network connectivity issue
- `isSyncing: true`: Node behind (check logs)
- Service not running: Critical failure!

---

### **2. Check Disk Space** (2 minutes)

```bash
# Check disk usage
df -h /var/lib/belizechain

# Should have > 20% free space
```

**Automated alert**:

```bash
# Add to crontab (daily check at 9am)
crontab -e
```

```bash
0 9 * * * /home/belizechain/scripts/check-disk.sh
```

**check-disk.sh**:

```bash
#!/bin/bash
THRESHOLD=80
USAGE=$(df -h /var/lib/belizechain | tail -1 | awk '{print $5}' | sed 's/%//')

if [ $USAGE -gt $THRESHOLD ]; then
  echo "ALERT: Disk usage at ${USAGE}%" | mail -s "BelizeChain Disk Alert" admin@example.com
fi
```

---

### **3. Review Logs** (5 minutes)

```bash
# Check for errors
journalctl -u belizechain --since "1 hour ago" | grep -i error

# Check for warnings
journalctl -u belizechain --since "1 hour ago" | grep -i warn
```

**Common warnings** (safe to ignore):
- `Timeout importing block`: Temporary network issue
- `Unable to author block`: Not elected as block producer this slot
- `Sync is slow`: Catching up from restart

**Critical errors** (requires action):
- `Database corruption`: Restore from backup
- `Out of memory`: Increase RAM or reduce cache
- `Peer not connecting`: Check firewall/network

---

## 🔄 Weekly Maintenance

### **1. Security Updates** (10 minutes)

```bash
# Update system packages
sudo apt update
sudo apt upgrade -y

# Check for security updates specifically
sudo unattended-upgrades --dry-run

# Restart if kernel updated
if [ -f /var/run/reboot-required ]; then
  echo "Reboot required"
  # Schedule maintenance window for reboot
fi
```

---

### **2. Backup Verification** (15 minutes)

```bash
# Verify backups exist
ls -lh /backup/belizechain-*

# Test restore (on separate machine!)
tar -xzf /backup/belizechain-$(date -d yesterday +%Y%m%d).tar.gz -C /tmp/restore-test
# Verify files intact

# Verify offsite backup
rclone ls remote:belizechain-backups/
```

---

### **3. Performance Check** (10 minutes)

```bash
# Check CPU usage
top -bn1 | grep belizechain

# Check memory usage
ps aux | grep belizechain | awk '{print $4}'

# Check network I/O
iftop -t -s 10 -n -i eth0 2>&1 | grep -A 10 "Total send"
```

**Baseline expectations**:
- **CPU**: 30-60% average (spikes to 100% during block production)
- **Memory**: 4-8 GB (depends on cache size)
- **Network**: 10-50 Mbps steady

---

## 📆 Monthly Maintenance

### **1. Update Node Software** (30 minutes)

**Before updating**:
- [ ] Check [release notes](https://github.com/BelizeChain/belizechain/releases)
- [ ] Backup database (see backup section)
- [ ] Schedule maintenance window (low-traffic time)

```bash
# Download new version
VERSION="1.1.0"
wget https://github.com/BelizeChain/belizechain/releases/download/v${VERSION}/belizechain-node-linux-x86_64.tar.gz

# Verify checksum
sha256sum belizechain-node-linux-x86_64.tar.gz
# Compare with release page

# Stop node
sudo systemctl stop belizechain

# Backup current binary
sudo cp /usr/local/bin/belizechain-node /usr/local/bin/belizechain-node.bak

# Install new version
tar -xzf belizechain-node-linux-x86_64.tar.gz
sudo mv belizechain-node /usr/local/bin/
sudo chmod +x /usr/local/bin/belizechain-node

# Verify version
/usr/local/bin/belizechain-node --version

# Start node
sudo systemctl start belizechain

# Monitor startup
journalctl -u belizechain -f
```

**Rollback** (if issues):

```bash
sudo systemctl stop belizechain
sudo cp /usr/local/bin/belizechain-node.bak /usr/local/bin/belizechain-node
sudo systemctl start belizechain
```

---

### **2. Database Cleanup** (15 minutes)

```bash
# Stop node
sudo systemctl stop belizechain

# Compact database (RocksDB)
belizechain-node database compact \
  --base-path /var/lib/belizechain \
  --chain mainnet

# Verify database integrity
belizechain-node database check \
  --base-path /var/lib/belizechain \
  --chain mainnet

# Start node
sudo systemctl start belizechain
```

**Expected improvements**:
- 10-20% disk space reduction
- 5-10% faster queries

---

### **3. Certificate Renewal** (5 minutes, if using SSL)

```bash
# Renew Let's Encrypt certificate
sudo certbot renew

# Reload nginx
sudo systemctl reload nginx

# Verify expiry
sudo certbot certificates
```

---

## 💾 Backup Strategy

### **Full Backup** (Weekly, Sunday 2 AM)

```bash
#!/bin/bash
# /home/belizechain/scripts/backup-full.sh

DATE=$(date +%Y%m%d)
BACKUP_DIR="/backup"
DATA_DIR="/var/lib/belizechain"

# Stop node (for consistent backup)
systemctl stop belizechain

# Create backup
tar -czf ${BACKUP_DIR}/belizechain-full-${DATE}.tar.gz \
  ${DATA_DIR}/chains/belizechain/db \
  ${DATA_DIR}/chains/belizechain/keystore \
  /etc/belizechain/config.toml

# Start node
systemctl start belizechain

# Upload to offsite storage
rclone copy ${BACKUP_DIR}/belizechain-full-${DATE}.tar.gz \
  remote:belizechain-backups/

# Remove old backups (keep 4 weeks)
find ${BACKUP_DIR} -name "belizechain-full-*.tar.gz" -mtime +28 -delete

# Log
echo "$(date): Full backup completed" >> /var/log/belizechain-backup.log
```

**Add to crontab**:

```bash
0 2 * * 0 /home/belizechain/scripts/backup-full.sh
```

---

### **Incremental Backup** (Daily, 3 AM)

```bash
#!/bin/bash
# /home/belizechain/scripts/backup-incremental.sh

DATE=$(date +%Y%m%d)
BACKUP_DIR="/backup"

# Backup only keystore (critical, small)
tar -czf ${BACKUP_DIR}/belizechain-keys-${DATE}.tar.gz \
  /var/lib/belizechain/chains/belizechain/keystore

# Upload
rclone copy ${BACKUP_DIR}/belizechain-keys-${DATE}.tar.gz \
  remote:belizechain-backups/keys/

# Keep 90 days of key backups
find ${BACKUP_DIR} -name "belizechain-keys-*.tar.gz" -mtime +90 -delete
```

---

### **Restore from Backup**

```bash
# Stop node
sudo systemctl stop belizechain

# Restore data
cd /var/lib/belizechain
sudo tar -xzf /backup/belizechain-full-20251014.tar.gz

# Fix permissions
sudo chown -R belizechain:belizechain /var/lib/belizechain

# Start node
sudo systemctl start belizechain

# Monitor sync
journalctl -u belizechain -f
```

---

## 🚨 Emergency Procedures

### **Node Won't Start**

**Step 1: Check logs**

```bash
journalctl -u belizechain -n 100 --no-pager
```

**Common causes**:
- Database corruption → Restore from backup
- Port conflict → Check `netstat -tlnp | grep 30333`
- Permission issues → `chown -R belizechain:belizechain /var/lib/belizechain`

---

**Step 2: Verify binary**

```bash
/usr/local/bin/belizechain-node --version
# Should print version without errors
```

---

**Step 3: Try safe mode** (without validator)

```bash
# Temporarily disable validator
sudo nano /etc/systemd/system/belizechain.service
# Remove --validator flag

sudo systemctl daemon-reload
sudo systemctl start belizechain
```

If starts successfully → Issue with validator setup (check keys).

---

### **Database Corruption**

**Symptoms**:
- `Database error` in logs
- Node crashes on startup
- `Bad block` errors

**Solution 1: Restore from backup** (Fastest)

```bash
sudo systemctl stop belizechain
cd /var/lib/belizechain
sudo rm -rf chains/belizechain/db
sudo tar -xzf /backup/belizechain-full-YYYYMMDD.tar.gz
sudo chown -R belizechain:belizechain /var/lib/belizechain
sudo systemctl start belizechain
```

---

**Solution 2: Resync from scratch** (Slower, 6-12 hours)

```bash
sudo systemctl stop belizechain
sudo rm -rf /var/lib/belizechain/chains/belizechain/db
sudo systemctl start belizechain
# Node will resync from genesis
```

---

### **Slashing Event**

**If you're slashed**:

1. **Identify cause**:
   - Offline: Improve uptime
   - Equivocation: Check for duplicate validators!

2. **Fix issue**:
   ```bash
   # If duplicate validator found
   ssh other-server
   sudo systemctl stop belizechain
   # Remove validator keys from duplicate
   ```

3. **Check remaining stake**:
   ```bash
   curl -s http://localhost:9933 -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"query.staking.ledger","params":["YOUR_ADDRESS"]}' \
     | jq '.result.active'
   ```

4. **Bond additional stake** (if needed):
   ```bash
   polkadot-js-api \
     --ws wss://mainnet.belizechain.org \
     --seed "your seed" \
     tx.staking.bondExtra \
     AMOUNT
   ```

---

### **Out of Disk Space**

**Immediate action**:

```bash
# Check disk usage
df -h /var/lib/belizechain

# Stop node
sudo systemctl stop belizechain

# Delete old logs
sudo journalctl --vacuum-time=7d

# Prune database (if full node)
belizechain-node database prune \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --pruning 256

# Start node
sudo systemctl start belizechain
```

**Long-term solution**: Upgrade disk or switch to pruned mode.

---

## 📊 Performance Optimization

### **Database Tuning**

**RocksDB optimization** (`/etc/belizechain/config.toml`):

```toml
[database]
backend = "rocksdb"
cache_size = 4096  # Increase if you have RAM

# Advanced RocksDB settings
[database.rocksdb]
max_open_files = 10000
keep_log_file_num = 10
max_background_jobs = 4
```

---

### **Network Optimization**

**Increase peer connections** (for RPC nodes):

```toml
[network]
max_peers = 100  # Default: 50

# More bandwidth for syncing
max_parallel_downloads = 10  # Default: 5
```

---

### **Memory Optimization**

```bash
# Check memory usage
free -h

# If using swap, increase cache
sudo nano /etc/belizechain/config.toml
```

```toml
[database]
cache_size = 8192  # Increase if you have 16+ GB RAM

[state_cache]
size = 4096
```

---

## 🔍 Health Checks

### **Automated Health Check Script**

```bash
#!/bin/bash
# /home/belizechain/scripts/health-check.sh

# Check service running
if ! systemctl is-active --quiet belizechain; then
  echo "CRITICAL: Service not running"
  systemctl start belizechain
  exit 1
fi

# Check peer count
PEERS=$(curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
  | jq '.result.peers')

if [ "$PEERS" -lt 5 ]; then
  echo "WARNING: Low peer count ($PEERS)"
fi

# Check sync status
IS_SYNCING=$(curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
  | jq '.result.isSyncing')

if [ "$IS_SYNCING" = "true" ]; then
  echo "WARNING: Node is syncing"
fi

# Check disk space
DISK_USAGE=$(df -h /var/lib/belizechain | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 80 ]; then
  echo "WARNING: Disk usage at ${DISK_USAGE}%"
fi

echo "OK: All health checks passed"
```

**Run every 5 minutes**:

```bash
*/5 * * * * /home/belizechain/scripts/health-check.sh >> /var/log/belizechain-health.log 2>&1
```

---

## ✅ Maintenance Checklist

### **Daily**:
- [ ] Check node status (`systemctl status belizechain`)
- [ ] Check peer count (should be > 5)
- [ ] Check disk space (should have > 20% free)
- [ ] Review error logs (`journalctl -u belizechain | grep -i error`)

### **Weekly**:
- [ ] Apply security updates (`apt upgrade`)
- [ ] Verify backups exist
- [ ] Review performance metrics (CPU, memory, network)

### **Monthly**:
- [ ] Update node software (if new release)
- [ ] Compact database
- [ ] Review and analyze logs for patterns
- [ ] Test backup restore (on separate machine)
- [ ] Review validator performance (if applicable)

### **Quarterly**:
- [ ] Disaster recovery drill (full restore test)
- [ ] Security audit (check firewall, SSH, access logs)
- [ ] Capacity planning (project disk/RAM needs)
- [ ] Review and update documentation

---

## 🚀 Next Steps

1. **[Monitoring →](monitoring.md)**  
   Set up comprehensive monitoring

2. **[Security →](security.md)**  
   Harden your node security

3. **[Validator Setup →](validator-setup.md)**  
   Become a validator (if applicable)

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) for maintenance support! 🔧🇧🇿
