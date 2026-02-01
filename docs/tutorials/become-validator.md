# ⚡ Become a Validator Tutorial

**Secure BelizeChain and earn 15% APY rewards**

---

## 📋 Overview

Learn how to become a BelizeChain validator - set up your node, bond DALLA stake, start validating blocks, and earn rewards for securing the network.

**What you'll learn**:
- Validator requirements
- Server setup
- Node installation
- Session keys generation
- Bonding stake
- Start validating
- Monitor and maintain

**Time**: 2-3 hours initial setup  
**Investment**: 10,000+ DALLA + server costs ($50-150/month)  
**Rewards**: 15% APY on your stake  
**Level**: ⭐⭐⭐⭐ Expert

---

## ⚠️ Before You Start

### Important Considerations

**This is for technical users only!** Requirements:
- ✅ Linux server administration experience
- ✅ Command line comfortable
- ✅ Network security knowledge
- ✅ 24/7 availability commitment
- ✅ Minimum 10,000 DALLA stake

**Time commitment**:
- Setup: 2-3 hours
- Maintenance: 2-4 hours/month
- Monitoring: Daily checks (5-10 minutes)
- Upgrades: Quarterly (1-2 hours each)

**Financial commitment**:
- Stake: 10,000+ DALLA (can lose if you misbehave!)
- Server: $50-150/month
- Backup server: $50/month (recommended)

**Are you ready?** If not, consider [delegating to a validator](./delegate-stake.md) instead (easier, same rewards, no technical work).

---

## ✅ Prerequisites

### Technical Requirements

- [ ] Linux experience (Ubuntu preferred)
- [ ] SSH and terminal skills
- [ ] Basic networking knowledge
- [ ] Git and command line tools

### Server Requirements

**Minimum**:
- 4 CPU cores (2.5+ GHz)
- 8 GB RAM
- 500 GB SSD storage
- 100 Mbps internet
- Static IP address
- 99.9% uptime guarantee

**Recommended**:
- 8 CPU cores (3+ GHz)
- 16 GB RAM
- 1 TB NVMe SSD
- 1 Gbps internet
- Redundant network
- Backup server

**Providers** (tested and recommended):
- **Hetzner** (€40-80/month, Europe)
- **OVH** ($50-100/month, global)
- **Vultr** ($60-120/month, global)
- **AWS** ($80-150/month, expensive but reliable)
- **Local Belize hosting** (contact: hosting@belizechain.org)

### Financial Requirements

- [ ] Minimum 10,000 DALLA for stake
- [ ] Extra DALLA for transaction fees (~100 DALLA)
- [ ] Server budget ($50-150/month)
- [ ] Emergency fund (3 months server costs)

### Knowledge Requirements

- [ ] Read [Staking Guide](../getting-started/staking-guide.md)
- [ ] Understand [Consensus](../technical-reference/consensus.md)
- [ ] Read [Validator Economics](../technical-reference/validator-economics.md)

**Ready?** Let's begin! 🚀

---

## 🖥️ Part 1: Server Setup (30 minutes)

### Step 1: Provision Server

**Using Hetzner (example)**:

1. Go to https://hetzner.com
2. Sign up / Log in
3. **Cloud → New Server**
4. Choose:
   - **Location**: Helsinki (or nearest to Belize for backup)
   - **Image**: Ubuntu 22.04 LTS
   - **Type**: CPX41 (8 vCPU, 16GB RAM, 240GB SSD)
   - **Networking**: Enable IPv4 + IPv6
   - **SSH Keys**: Add your public key
   - **Name**: belizechain-validator-1

5. **Create & Start** (ready in ~60 seconds)
6. **Note your IP**: `95.216.X.X`

---

### Step 2: Initial Server Configuration

**SSH into your server**:

```bash
ssh root@95.216.X.X
```

**Update system**:

```bash
# Update package lists
apt update && apt upgrade -y

# Install essential tools
apt install -y curl git build-essential pkg-config \
  libssl-dev clang cmake ufw fail2ban
```

**Create validator user** (don't run as root!):

```bash
# Create user
adduser validator
usermod -aG sudo validator

# Switch to validator user
su - validator
```

---

### Step 3: Configure Firewall

**Setup UFW (Uncomplicated Firewall)**:

```bash
# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow BelizeChain P2P (substrate)
sudo ufw allow 30333/tcp

# Allow BelizeChain RPC (for monitoring, restrict later)
sudo ufw allow 9933/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status
```

**Expected output**:
```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
30333/tcp                  ALLOW       Anywhere
9933/tcp                   ALLOW       Anywhere
```

---

### Step 4: Configure Fail2Ban

**Protect against brute force attacks**:

```bash
# Enable for SSH
sudo systemctl enable fail2ban
sudo systemctl start fail2ban

# Check status
sudo fail2ban-client status sshd
```

---

## 🔧 Part 2: Install BelizeChain Node (45 minutes)

### Step 1: Install Rust

**BelizeChain is written in Rust**:

```bash
# Install rustup (Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Choose: 1) Proceed with installation (default)

# Load Rust environment
source $HOME/.cargo/env

# Verify installation
rustc --version
# Expected: rustc 1.75.0 or newer

# Install nightly toolchain
rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

---

### Step 2: Clone BelizeChain Repository

```bash
# Clone repository
cd ~
git clone https://github.com/belizechain/belizechain.git
cd belizechain

# Checkout latest stable release
git checkout tags/v1.2.0  # Check latest: github.com/belizechain/belizechain/releases
```

---

### Step 3: Build BelizeChain Node

**This takes 30-45 minutes!** ☕

```bash
# Build release binary
cargo build --release

# Binary location: target/release/belizechain
```

**Expected output** (after 30-45 minutes):
```
   Compiling belizechain v1.2.0
    Finished release [optimized] target(s) in 42m 15s
```

**Verify binary**:

```bash
./target/release/belizechain --version
# Expected: belizechain 1.2.0-abc123
```

---

### Step 4: Install Binary System-Wide

```bash
# Copy to /usr/local/bin
sudo cp target/release/belizechain /usr/local/bin/

# Verify
belizechain --version
```

---

## 🚀 Part 3: Start Syncing Node (10 minutes + sync time)

### Step 1: Create Systemd Service

**Run node as background service**:

```bash
# Create service file
sudo nano /etc/systemd/system/belizechain.service
```

**Paste this configuration**:

```ini
[Unit]
Description=BelizeChain Validator Node
After=network.target

[Service]
Type=simple
User=validator
WorkingDirectory=/home/validator
ExecStart=/usr/local/bin/belizechain \
  --base-path /home/validator/.belizechain \
  --chain belizechain \
  --name "YourValidatorName" \
  --validator \
  --telemetry-url "wss://telemetry.belizechain.org/submit/ 0" \
  --prometheus-external
Restart=always
RestartSec=10
LimitNOFILE=10000

[Install]
WantedBy=multi-user.target
```

**Replace**:
- `YourValidatorName` → Your chosen validator name (e.g., "BelizeValidator1")

**Save**: Ctrl+X, Y, Enter

---

### Step 2: Start Node Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable belizechain

# Start service
sudo systemctl start belizechain

# Check status
sudo systemctl status belizechain
```

**Expected output**:
```
● belizechain.service - BelizeChain Validator Node
   Loaded: loaded (/etc/systemd/system/belizechain.service)
   Active: active (running) since Mon 2025-10-14 10:00:00 UTC
```

---

### Step 3: Monitor Syncing Progress

**Check logs**:

```bash
# Follow logs (Ctrl+C to exit)
journalctl -f -u belizechain
```

**Expected output** (syncing):
```
2025-10-14 10:00:15 ⚙️  Syncing 125.6 bps, target=#5246789 (19 peers), best: #1234567 (0xabc...def)
2025-10-14 10:00:20 ⚙️  Syncing 126.2 bps, target=#5246798 (21 peers), best: #1235198 (0xdef...123)
2025-10-14 10:00:25 ⚙️  Syncing 127.8 bps, target=#5246812 (23 peers), best: #1235837 (0x123...abc)
```

**Syncing takes 4-8 hours for full blockchain!**

**Check sync status**:

```bash
# Check current block vs target
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933/
```

**When synced**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "peers": 25,
    "isSyncing": false,
    "shouldHavePeers": true
  },
  "id": 1
}
```

**Wait for `"isSyncing": false` before continuing!**

---

## 🔑 Part 4: Generate Session Keys (5 minutes)

### Step 1: Generate Keys

**Once synced, generate validator keys**:

```bash
# Generate session keys
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys"}' \
  http://localhost:9933/
```

**Expected output**:
```json
{
  "jsonrpc": "2.0",
  "result": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "id": 1
}
```

**⚠️ SAVE THIS HEX STRING!** You'll need it for bonding.

**This is your session key** (combines multiple keys):
- BABE key (block production)
- GRANDPA key (finality)
- ImOnline key (heartbeat)
- AuthorityDiscovery key (peer discovery)

---

### Step 2: Backup Keys

**Keys are stored in node data directory**:

```bash
# Backup keys
cp -r ~/.belizechain/chains/belizechain/keystore ~/keystore_backup

# Create encrypted archive
tar -czf keystore_backup.tar.gz keystore_backup
gpg -c keystore_backup.tar.gz  # Enter passphrase

# Download to your local machine (from your computer)
scp validator@95.216.X.X:~/keystore_backup.tar.gz.gpg ~/

# Store safely (password manager, encrypted USB, bank vault)
```

**⚠️ CRITICAL**: If you lose these keys, you lose your validator!

---

## 💰 Part 5: Bond Stake (10 minutes)

### Step 1: Access Staking Interface

**Open in browser**: https://staking.belizechain.org

**Or use Maya Wallet**:
- Open Maya Wallet
- Menu → Staking → Become Validator

---

### Step 2: Bond Your DALLA

```
╔════════════════════════════════════════════╗
║  BOND VALIDATOR STAKE                      ║
╠════════════════════════════════════════════╣
║  Your Balance: 25,000 DALLA                ║
║                                            ║
║  Amount to Bond:                           ║
║  [10,000        ] DALLA                    ║
║                                            ║
║  ⚠️  Bonded DALLA is locked and used as   ║
║      security. If you misbehave (offline,  ║
║      double-sign), you can lose it!        ║
║                                            ║
║  Reward Destination:                       ║
║  ● Stash (re-stake automatically)          ║
║  ○ Controller (claim manually)             ║
║  ○ Account: [specify address]              ║
║                                            ║
║  Controller Account:                       ║
║  [Same as stash ▼]                         ║
║                                            ║
║  Fee: 0.05 bBZD                            ║
║                                            ║
║  [Cancel] [Bond & Nominate]                ║
╚════════════════════════════════════════════╝
```

**Recommendations**:
- **Amount**: Start with 10,000-15,000 DALLA (minimum)
- **Reward Destination**: "Stash" (compound automatically)
- **Controller**: Separate account (more secure) or same for simplicity

**Enter PIN** → **Bond!** ✅

---

### Step 3: Set Session Keys

**After bonding, set your session keys**:

```
╔════════════════════════════════════════════╗
║  SET SESSION KEYS                          ║
╠════════════════════════════════════════════╣
║  You've bonded 10,000 DALLA!               ║
║  Now set your validator's session keys.    ║
║                                            ║
║  Session Keys (from your node):            ║
║  [0x1234567890abcdef...              ]     ║
║                                            ║
║  ⚠️  Paste the FULL hex string from       ║
║      Step 4 (starts with 0x)               ║
║                                            ║
║  Fee: 0.01 bBZD                            ║
║                                            ║
║  [Back] [Set Keys]                         ║
╚════════════════════════════════════════════╝
```

**Paste your session key** (from Part 4, Step 1)  
**Tap "Set Keys"** ✅

---

### Step 4: Validate

**Start validating**:

```
╔════════════════════════════════════════════╗
║  START VALIDATING                          ║
╠════════════════════════════════════════════╣
║  Bonded: 10,000 DALLA ✅                   ║
║  Session Keys: Set ✅                      ║
║                                            ║
║  Commission:                               ║
║  [10              ] %                      ║
║                                            ║
║  Commission is what you keep from rewards  ║
║  before sharing with nominators.           ║
║                                            ║
║  Recommended for new validators: 5-10%     ║
║  Market average: 10%                       ║
║                                            ║
║  Block Authorship Rewards: Full to you     ║
║                                            ║
║  Fee: 0.01 bBZD                            ║
║                                            ║
║  [Cancel] [Start Validating]               ║
╚════════════════════════════════════════════╝
```

**Set commission** (10% is standard)  
**Tap "Start Validating"** ✅

---

### Step 5: Confirmation

```
🎉 VALIDATOR ACTIVATED!

Validator: YourValidatorName
Stash: 5YourAddressXXXXXXXXXXXXXXXXXXX
Bonded: 10,000 DALLA
Commission: 10%
Status: Waiting

Next Steps:
1. Wait for next era (24 hours max)
2. Get elected by stake amount
3. Start producing blocks
4. Earn rewards!

Your node will appear in validator list shortly.

[View Validator] [Monitor] [Dashboard]
```

**Congratulations! You're a validator!** 🎉

---

## 📊 Part 6: Monitor Your Validator

### Dashboard

**Access**: https://telemetry.belizechain.org

**Your validator stats**:

```
╔════════════════════════════════════════════╗
║  VALIDATOR: YourValidatorName              ║
╠════════════════════════════════════════════╣
║  Status: 🟢 Active (producing blocks)      ║
║  Uptime: 99.8% (last 7 days)               ║
║                                            ║
║  STAKING                                   ║
║  Own Stake: 10,000 DALLA                   ║
║  Nominated: 45,000 DALLA (12 nominators)   ║
║  Total Stake: 55,000 DALLA                 ║
║  Commission: 10%                           ║
║                                            ║
║  PERFORMANCE                               ║
║  Blocks Produced: 1,247 (this era)         ║
║  Blocks Missed: 3 (0.24%)                  ║
║  Slash Events: 0 ✅                        ║
║                                            ║
║  REWARDS (This Era)                        ║
║  Block Rewards: 125.4 DALLA                ║
║  Commission: 189.2 DALLA                   ║
║  Total Earned: 314.6 DALLA                 ║
║                                            ║
║  REWARDS (All Time)                        ║
║  Total Earned: 1,547 DALLA                 ║
║  APY: 15.2% (based on current stake)       ║
║                                            ║
║  NEXT ERA                                  ║
║  Starts in: 8 hours 23 minutes             ║
║  Estimated Rewards: 320 DALLA              ║
╚════════════════════════════════════════════╝
```

---

### Check Node Health

**Daily check (5 minutes)**:

```bash
# Check service status
sudo systemctl status belizechain

# Check if syncing
curl -s -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933/ | jq

# Check peer count (should be 20-30)
curl -s -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers"}' \
  http://localhost:9933/ | jq '.result | length'

# Check recent logs
journalctl -u belizechain -n 50 --no-pager
```

**Look for**:
- ✅ Service: "active (running)"
- ✅ Syncing: false
- ✅ Peers: 20-30
- ✅ Logs: "Imported #XXXXX"
- ❌ Errors, warnings, crashes

---

### Setup Monitoring Alerts

**Option 1: Email Alerts (Simple)**

```bash
# Install alerting script
cd ~
curl -O https://raw.githubusercontent.com/belizechain/validator-tools/main/monitor.sh
chmod +x monitor.sh

# Configure email
nano monitor.sh
# Set: EMAIL="your@email.com"

# Add to crontab (runs every 5 minutes)
crontab -e
# Add: */5 * * * * /home/validator/monitor.sh
```

**Alerts you'll receive**:
- Node offline
- Not syncing
- Low peer count
- Disk space low (< 10%)
- High CPU usage (> 90%)

---

**Option 2: Advanced Monitoring (Recommended)**

**Use Prometheus + Grafana dashboard**:

Full guide: [Validator Monitoring Setup](../deployment/validator-monitoring.md)

**Benefits**:
- Real-time graphs
- Historical data
- Mobile alerts
- Compare with other validators

---

## 💰 Part 7: Earn and Claim Rewards

### How Rewards Work

**Every era (24 hours)**:

1. **Block Rewards**: Earn DALLA for each block you produce
   - ~0.1 DALLA per block
   - Depends on total stake and performance

2. **Commission**: Earn commission on nominated stake
   - If nominators stake 45K DALLA with you
   - And earn 1,890 DALLA in era
   - You earn 10% = 189 DALLA

3. **Total Rewards**: Block rewards + commission
   - Example: 125 DALLA (blocks) + 189 DALLA (commission) = 314 DALLA
   - Per era (24 hours)
   - ~9,500 DALLA per month
   - ~15% APY on your 10K stake

---

### Auto-Compound (Recommended)

**If you selected "Stash" as reward destination**:

- ✅ Rewards automatically added to stake
- ✅ Compounding effect (earn on earnings)
- ✅ No action needed
- ✅ Maximum long-term returns

**Example compounding**:
```
Start: 10,000 DALLA
Year 1: 11,500 DALLA (+15%)
Year 2: 13,225 DALLA (+15% on 11,500)
Year 3: 15,209 DALLA (+15% on 13,225)
```

---

### Manual Claim

**If you selected "Controller" as reward destination**:

```
# Check pending rewards
curl -s -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "staking_payoutStakers"}' \
  http://localhost:9933/

# Claim rewards (in Maya Wallet)
Staking → My Validator → Claim Rewards
```

**Claim every 84 eras (84 days max)** or rewards expire!

---

## 🔧 Part 8: Maintenance

### Weekly Tasks (10 minutes)

- [ ] Check dashboard (uptime, blocks produced)
- [ ] Review logs for errors
- [ ] Check disk space (`df -h`)
- [ ] Verify peer count (20-30)
- [ ] Check rewards earned

---

### Monthly Tasks (1 hour)

- [ ] Update system packages (`apt update && apt upgrade`)
- [ ] Review security logs (`sudo fail2ban-client status`)
- [ ] Backup keystore
- [ ] Check node version (update if needed)
- [ ] Review validator stats vs network average
- [ ] Optimize commission rate (if needed)

---

### Quarterly Tasks (2-3 hours)

- [ ] **Upgrade BelizeChain node** (major releases)
- [ ] Test backup server
- [ ] Review and optimize server resources
- [ ] Audit security configuration
- [ ] Review and update monitoring alerts

---

### Upgrade Node (When New Release)

**Announcements**: https://forum.belizechain.org/validators

**Upgrade process**:

```bash
# Stop node
sudo systemctl stop belizechain

# Backup current binary
sudo cp /usr/local/bin/belizechain /usr/local/bin/belizechain.backup

# Update repository
cd ~/belizechain
git fetch --all --tags
git checkout tags/v1.3.0  # New version

# Rebuild (30-45 minutes)
cargo build --release

# Install new binary
sudo cp target/release/belizechain /usr/local/bin/

# Restart node
sudo systemctl start belizechain

# Monitor logs
journalctl -f -u belizechain

# Verify version
belizechain --version
```

**Critical**: Upgrade during scheduled maintenance windows when possible!

---

## ⚠️ Troubleshooting

### Problem: Node Won't Start

**Check logs**:
```bash
journalctl -u belizechain -n 100 --no-pager
```

**Common causes**:
- Database corruption → `rm -rf ~/.belizechain/chains/*/db` (resync needed!)
- Port already in use → Check if another process uses 30333
- Insufficient disk space → `df -h` (need 100+ GB free)
- Permission issues → `chown -R validator:validator ~/.belizechain`

---

### Problem: Not Producing Blocks

**Check**:
- ✅ Node synced? (`"isSyncing": false`)
- ✅ Session keys set correctly?
- ✅ In active validator set? (Top 100 by stake)
- ✅ Waiting for next era? (up to 24 hours)

**If still not producing**:
- Rotate session keys and set again
- Increase stake (get more nominations)
- Check firewall (port 30333 open?)

---

### Problem: Getting Slashed

**Slash reasons**:
- **Offline** (miss blocks) → 0.1% slash per incident
- **Double-signing** (running two nodes with same keys) → 5% slash!
- **Wrong chain** (follow invalid fork) → 1% slash

**Prevention**:
- Monitor uptime closely
- Never run duplicate validators
- Keep node updated
- Fast internet with backup
- Have emergency plan

**If slashed**:
1. Fix issue immediately
2. Review logs to understand cause
3. Announce in Discord (#validators)
4. Implement prevention measures
5. Rebuild reputation

---

### Problem: Low Rewards

**Causes**:
- Low total stake (not competitive)
- High commission (nominators choose others)
- Missing blocks (performance issues)
- Not in active set (top 100 only)

**Solutions**:
1. **Increase stake** (bond more DALLA)
2. **Lower commission** (attract nominators)
3. **Improve uptime** (99.9%+ goal)
4. **Promote yourself**:
   - Forum: https://forum.belizechain.org/validators
   - Discord: #validators channel
   - Social media
   - Validator profile

---

## 📚 Resources

**Documentation**:
- [Staking Guide](../getting-started/staking-guide.md) - For nominators
- [Consensus Technical Reference](../technical-reference/consensus.md)
- [Validator Economics](../technical-reference/validator-economics.md)
- [Validator Monitoring](../deployment/validator-monitoring.md)

**Tools**:
- [Telemetry Dashboard](https://telemetry.belizechain.org)
- [Block Explorer](https://explorer.belizechain.org)
- [Validator Leaderboard](https://validators.belizechain.org)
- [Monitoring Scripts](https://github.com/belizechain/validator-tools)

**Community**:
- **Discord**: #validators channel
- **Forum**: https://forum.belizechain.org/validators
- **Telegram**: @BelizeChainValidators
- **Monthly Calls**: First Wednesday, 8 PM UTC

**Support**:
- **Email**: validators@belizechain.org
- **Emergency**: +501-CHAIN (24246), option 5
- **Response**: Within 24 hours (1 hour for emergencies)

---

## ✅ Validator Checklist

### Setup ✅
- [x] Provisioned server (4+ CPU, 8+ GB RAM, 500+ GB SSD)
- [x] Configured firewall and security
- [x] Installed Rust and dependencies
- [x] Built BelizeChain node
- [x] Created systemd service
- [x] Synced blockchain (4-8 hours)
- [x] Generated session keys
- [x] Backed up keystore
- [x] Bonded 10,000+ DALLA
- [x] Set session keys on-chain
- [x] Started validating

### Monitoring ✅
- [x] Setup monitoring dashboard
- [x] Configured email alerts
- [x] Daily health checks
- [x] Joined validator Discord
- [x] Subscribed to announcements

### Ongoing ✅
- [ ] Daily: Check uptime and blocks
- [ ] Weekly: Review logs and stats
- [ ] Monthly: Update system and review performance
- [ ] Quarterly: Upgrade node software
- [ ] Always: Maintain 99.9%+ uptime!

---

## 🎉 Success! You're a Validator!

**You now**:
- ✅ Secure the BelizeChain network
- ✅ Earn 15% APY on your stake
- ✅ Contribute to Belize's digital sovereignty
- ✅ Join elite group of 100 validators

**Expected earnings** (10,000 DALLA stake):
- **Per era** (24 hours): ~40-50 DALLA
- **Per month**: ~1,250 DALLA
- **Per year**: ~15,000 DALLA (15% APY)
- **With nominations** (55K total stake): ~1,700 DALLA/month!

**Welcome to the BelizeChain validator community!** 🇧🇿⚡

---

**Questions?** validators@belizechain.org  
**Need help?** Discord #validators  
**Emergency?** +501-CHAIN (24246), option 5
