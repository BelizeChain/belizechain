# 🚀 BelizeChain Validator Onboarding Guide

**Version**: Rolling (testnet)  
**Updated**: November 4, 2025  
**Estimated Time**: 2-3 hours (technical setup)  

---

## 📋 Table of Contents

1. [Onboarding Overview](#onboarding-overview)
2. [Pre-Flight Checklist](#pre-flight-checklist)
3. [Step 1: Environment Setup](#step-1-environment-setup)
4. [Step 2: Key Generation](#step-2-key-generation)
5. [Step 3: Node Deployment](#step-3-node-deployment)
6. [Step 4: Staking Setup](#step-4-staking-setup)
7. [Step 5: Session Keys](#step-5-session-keys)
8. [Step 6: Validation Start](#step-6-validation-start)
9. [Step 7: Monitoring](#step-7-monitoring)
10. [Post-Onboarding](#post-onboarding)

---

## Onboarding Overview

This guide walks you through the complete process of becoming a BelizeChain validator, from zero to producing your first block.

### Onboarding Phases

```
Phase 1: Preparation (30 min)
├─ Review requirements
├─ Provision hardware
└─ Install dependencies

Phase 2: Key Generation (15 min)
├─ Stash/Controller accounts
├─ Session keys
└─ Secure backups

Phase 3: Node Deployment (45 min)
├─ Build/download node
├─ Configure systemd
└─ Sync blockchain

Phase 4: Staking Setup (30 min)
├─ Transfer DALLA
├─ Bond funds
└─ Set session keys

Phase 5: Validation (15 min)
├─ Start validating
├─ Verify block production
└─ Monitor performance

Total: 2-3 hours (depending on sync time)
```

---

## Pre-Flight Checklist

Before starting, ensure you have:

### ✅ Technical Requirements
- [ ] Server with specs meeting [VALIDATOR_REQUIREMENTS.md](VALIDATOR_REQUIREMENTS.md)
- [ ] Static IP address (or dynamic DNS)
- [ ] SSH access configured
- [ ] Firewall rules planned

### ✅ Financial Requirements
- [ ] **Testnet**: 10,000+ DALLA (from faucet)
- [ ] **Mainnet**: 100,000+ DALLA purchased/earned
- [ ] Budget for operational costs (~$200-500/month)

### ✅ Knowledge Requirements
- [ ] Basic Linux command-line skills
- [ ] Understanding of staking mechanics
- [ ] Familiarity with blockchain concepts
- [ ] Read [INCENTIVES.md](INCENTIVES.md) and [VALIDATOR_REQUIREMENTS.md](VALIDATOR_REQUIREMENTS.md)

### ✅ Tools Required
- [ ] Hardware wallet (Mainnet: Ledger/Trezor recommended)
- [ ] Secure password manager
- [ ] 2FA authenticator app
- [ ] Terminal emulator (SSH client)

---

## Step 1: Environment Setup

### 1.1 Provision Server

**Cloud Providers** (recommended for beginners):
- **DigitalOcean**: Droplet ($80-200/month)
- **Vultr**: High Frequency ($120-240/month)
- **AWS**: EC2 t3.xlarge ($150-300/month)
- **Hetzner**: Dedicated server ($50-150/month, EU only)

**Bare Metal** (advanced users):
- **OVH**: Dedicated servers ($80-200/month)
- **Leaseweb**: Enterprise servers ($150-500/month)

### 1.2 Initial Server Configuration

```bash
# SSH into server
ssh root@your-server-ip

# Update system
apt update && apt upgrade -y

# Create validator user (NEVER run as root)
adduser belizechain
usermod -aG sudo belizechain

# Setup SSH key authentication
mkdir -p /home/belizechain/.ssh
cp ~/.ssh/authorized_keys /home/belizechain/.ssh/
chown -R belizechain:belizechain /home/belizechain/.ssh
chmod 700 /home/belizechain/.ssh
chmod 600 /home/belizechain/.ssh/authorized_keys

# Disable password authentication
nano /etc/ssh/sshd_config
# Set: PasswordAuthentication no
# Set: PermitRootLogin no
systemctl restart sshd

# Switch to validator user
su - belizechain
```

### 1.3 Install Dependencies

```bash
# Install required packages
sudo apt install -y \
    build-essential \
    git \
    clang \
    curl \
    libssl-dev \
    llvm \
    libudev-dev \
    pkg-config \
    protobuf-compiler \
    jq \
    htop \
    net-tools

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Configure Rust
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown

# Verify installation
rustc --version
cargo --version
```

### 1.4 Configure Firewall

```bash
# Install ufw
sudo apt install -y ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (CRITICAL: Do this first!)
sudo ufw allow 22/tcp

# Allow P2P (public)
sudo ufw allow 30333/tcp comment 'BelizeChain P2P'

# Enable firewall
sudo ufw enable

# Verify rules
sudo ufw status verbose
```

---

## Step 2: Key Generation

### 2.1 Stash Account (Cold Storage)

**CRITICAL**: This account holds your funds. NEVER expose private key.

**Option A: Hardware Wallet** (RECOMMENDED for mainnet)
1. Connect Ledger/Trezor
2. Install Polkadot app
3. Generate account via Polkadot.js Apps
4. Write down address: `5ABC...XYZ`

**Option B: Paper Wallet** (testnet acceptable)
```bash
# Install subkey tool
cargo install --force subkey --git https://github.com/paritytech/polkadot-sdk

# Generate stash account
subkey generate --scheme Sr25519 --network substrate

# Output:
# Secret phrase: word1 word2 word3 ... word12
# Network ID:    substrate
# Secret seed:   0x1234...abcd
# Public key:    0x5678...efgh
# Account ID:    0x5678...efgh
# SS58 Address:  5ABC...XYZ (your stash address)

# ⚠️ WRITE DOWN SECRET PHRASE ON PAPER
# ⚠️ STORE IN SECURE LOCATION (fireproof safe)
# ⚠️ NEVER ENTER ON COMPUTER AGAIN
```

### 2.2 Controller Account (Hot Wallet)

This account manages your stash. Can be on computer.

```bash
# Generate controller account
subkey generate --scheme Sr25519 --network substrate

# Output: (save to password manager)
# Secret phrase: different12 words than stash
# SS58 Address:  5DEF...ABC (your controller address)

# Import to Polkadot.js browser extension
# 1. Install extension: https://polkadot.js.org/extension/
# 2. Click "Import account from pre-existing seed"
# 3. Paste controller secret phrase
# 4. Set name: "BelizeChain Controller"
# 5. Set password (strong, unique)
```

### 2.3 Backup Keys Securely

```bash
# Create encrypted backup directory
mkdir -p ~/belizechain-backup
chmod 700 ~/belizechain-backup

# Create backup file
cat > ~/belizechain-backup/KEYS.txt << EOF
BelizeChain Validator Keys
Generated: $(date)
Network: TESTNET/MAINNET

Stash Account:
Address: 5ABC...XYZ
Purpose: Holds bonded funds (cold storage)
⚠️ NEVER USE ONLINE

Controller Account:
Address: 5DEF...ABC
Secret: different12 words than stash
Purpose: Manages stash, sets session keys

Node P2P Key:
File: /data/belizechain/node-key
Purpose: Peer-to-peer identity
EOF

# Encrypt backup
gpg -c ~/belizechain-backup/KEYS.txt
# Enter strong passphrase (store in password manager)

# Verify encryption
gpg ~/belizechain-backup/KEYS.txt.gpg

# Delete plaintext (keep only .gpg file)
shred -u ~/belizechain-backup/KEYS.txt

# Store encrypted file on USB drive + cloud backup
```

---

## Step 3: Node Deployment

### 3.1 Build Node (Option A: From Source)

```bash
# Clone repository
cd ~
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain

# Checkout appropriate branch
git checkout testnet  # or 'mainnet'

# Build (takes 20-40 minutes)
cargo build --release

# Install binary
sudo cp target/release/belizechain-node /usr/local/bin/

# Verify
belizechain-node --version
```

### 3.2 Download Binary (Option B: Faster)

```bash
# Download latest release
LATEST_TAG=$(curl -s https://api.github.com/repos/BelizeChain/belizechain/releases/latest | grep '"tag_name":' | head -n1 | cut -d '"' -f4)
wget "https://github.com/BelizeChain/belizechain/releases/download/${LATEST_TAG}/belizechain-node-linux-amd64"

# Verify checksum
wget "https://github.com/BelizeChain/belizechain/releases/download/${LATEST_TAG}/checksums.txt"
sha256sum --check checksums.txt

# Install
chmod +x belizechain-node-linux-amd64
sudo mv belizechain-node-linux-amd64 /usr/local/bin/belizechain-node
```

### 3.3 Generate Node Key

```bash
# Create data directory
sudo mkdir -p /data/belizechain
sudo chown -R belizechain:belizechain /data/belizechain

# Generate node key
belizechain-node key generate-node-key --file /data/belizechain/node-key

# Get peer ID (save this for later)
belizechain-node key inspect-node-key --file /data/belizechain/node-key
# Output: 12D3KooWYourPeerIdHere...
```

### 3.4 Download Chain Spec

```bash
# Create config directory
mkdir -p ~/belizechain-config

# Save the operator-provided raw spec
cp /path/to/operator-provided-testnet-chain-spec-raw.json \
     ~/belizechain-config/chain-spec.json

# Verify
jq '.name, .chainType' ~/belizechain-config/chain-spec.json
# Output:
# "BelizeChain Testnet"
# "Live"
```

Do not use the disabled built-in `testnet` alias or any locally generated `--chain local` spec for validator onboarding. Validators should only use the raw spec published for the active network.

### 3.5 Create Systemd Service

```bash
# Create service file
sudo nano /etc/systemd/system/belizechain-validator.service
```

Paste the following:

```ini
[Unit]
Description=BelizeChain Validator Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=belizechain
Group=belizechain
WorkingDirectory=/home/belizechain
ExecStart=/usr/local/bin/belizechain-node \
    --base-path /data/belizechain \
    --chain /home/belizechain/belizechain-config/chain-spec.json \
    --name "YourValidatorName" \
    --validator \
    --port 30333 \
    --rpc-port 9944 \
    --prometheus-port 9615 \
    --rpc-methods Safe \
     --pruning 1000

Restart=always
RestartSec=10
LimitNOFILE=65536

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/data/belizechain

[Install]
WantedBy=multi-user.target
```

### 3.6 Start Node

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable belizechain-validator

# Start service
sudo systemctl start belizechain-validator

# Check status
sudo systemctl status belizechain-validator

# Watch logs (Ctrl+C to exit)
sudo journalctl -u belizechain-validator -f
```

### 3.7 Wait for Sync

This can take 1-6 hours depending on chain size and internet speed.

```bash
# Check sync status
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
     http://localhost:9944 | jq

# Output:
# {
#   "result": {
#     "peers": 25,
#     "isSyncing": true,  # false when synced
#     "shouldHavePeers": true
#   }
# }

# Check current block vs. target
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader"}' \
     http://localhost:9944 | jq .result.number

# Compare to block explorer: https://explorer.belizechain.org
```

---

## Step 4: Staking Setup

### 4.1 Transfer DALLA to Stash

**Testnet**: Get from faucet
```bash
# Use the current operator-provided faucet or request funds from the active coordinator
# Enter stash address: 5ABC...XYZ
# Click "Request 10,000 DALLA"
```

**Mainnet**: Purchase or transfer
- Use centralized exchange (when listed)
- Receive from another wallet
- Earn through community programs

### 4.2 Bond Funds

Using Polkadot.js Apps:

1. Visit https://polkadot.js.org/apps/
2. Connect to BelizeChain:
   - Click network dropdown
   - Select "Development" → "Custom"
     - Enter: `wss://<current-testnet-rpc-url>` from the current operator packet, or use your own node
   - Click "Switch"

3. Navigate to **Network → Staking → Account actions**

4. Click **+ Stash** button

5. Fill in bonding form:
   ```
   Stash account: 5ABC...XYZ (select your stash)
   Controller account: 5DEF...ABC (select your controller)
   Value bonded: 10,000 DALLA (or more)
   Payment destination: Staked (auto-compound rewards)
   ```

6. Click **Bond** and sign transaction (with stash account)

7. Wait for confirmation (~6 seconds)

---

## Step 5: Session Keys

### 5.1 Generate Session Keys

```bash
# Method 1: RPC call
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys"}' \
     http://localhost:9944

# Output: "0x1234567890abcdef..." (your session keys, 128 hex chars)

# Save this string! You'll need it in the next step.
```

### 5.2 Set Session Keys On-Chain

Using Polkadot.js Apps:

1. Navigate to **Developer → Extrinsics**

2. Select **session → setKeys(keys, proof)**

3. Fill in:
   ```
   using the selected account: Controller (5DEF...ABC)
   keys: 0x1234567890abcdef... (paste from 5.1)
   proof: 0x00
   ```

4. Click **Submit Transaction**

5. Sign with controller account (browser extension)

6. Wait for confirmation

### 5.3 Verify Session Keys

```bash
# Query on-chain session keys
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_hasSessionKeys", "params":["0x1234...abcd"]}' \
     http://localhost:9944

# Output: true (if keys are set correctly)
```

---

## Step 6: Validation Start

### 6.1 Start Validating

Using Polkadot.js Apps:

1. Navigate to **Network → Staking → Account actions**

2. Find your stash account in the list

3. Click **Validate** button

4. Set commission:
   ```
   Reward commission percentage: 10%
   (Tip: Lower commission attracts more nominators)
   ```

5. Optionally allow nomination (leave checked)

6. Click **Validate**

7. Sign transaction with controller account

8. Wait for confirmation

### 6.2 Check Validator Status

```bash
# Get your stash address AccountId (hex format)
curl -H "Content-Type: application/json" \
     -d '{
       "id":1,
       "jsonrpc":"2.0",
       "method": "state_getStorage",
       "params": ["0x..."]
     }' \
     http://localhost:9944

# Check validator set
# Visit https://polkadot.js.org/apps/#/staking
# Look for your validator name in "Waiting" or "Active" list
```

### 6.3 Wait for Active Set

- **Era Length**: 24 hours (14,400 blocks)
- **Selection**: Based on total stake (self + nominators)
- **Waiting Period**: Up to 24 hours (next era)

You'll start producing blocks once in the active set.

---

## Step 7: Monitoring

### 7.1 Install Monitoring Tools

```bash
# Install node_exporter (system metrics)
wget https://github.com/prometheus/node_exporter/releases/download/v1.7.0/node_exporter-1.7.0.linux-amd64.tar.gz
tar xvf node_exporter-1.7.0.linux-amd64.tar.gz
sudo cp node_exporter-1.7.0.linux-amd64/node_exporter /usr/local/bin/
rm -rf node_exporter*

# Create systemd service
sudo nano /etc/systemd/system/node_exporter.service
```

Paste:
```ini
[Unit]
Description=Node Exporter
After=network.target

[Service]
Type=simple
User=belizechain
ExecStart=/usr/local/bin/node_exporter

[Install]
WantedBy=multi-user.target
```

Start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable node_exporter
sudo systemctl start node_exporter
```

### 7.2 Setup Alerting

See [../monitoring/PROMETHEUS_SETUP.md](../monitoring/PROMETHEUS_SETUP.md) for full guide.

Quick email alerts:
```bash
# Install mailutils
sudo apt install -y mailutils

# Create alert script
nano ~/check-validator.sh
```

Paste:
```bash
#!/bin/bash
HEALTH=$(curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' | jq -r .result.peers)

if [ "$HEALTH" -lt 10 ]; then
  echo "ALERT: BelizeChain validator has <10 peers!" | \
    mail -s "Validator Alert" your-email@example.com
fi
```

Make executable and add to cron:
```bash
chmod +x ~/check-validator.sh
crontab -e

# Add this line (check every 5 minutes):
*/5 * * * * /home/belizechain/check-validator.sh
```

---

## Post-Onboarding

### ✅ Day 1 Checklist

- [ ] Node synced and producing blocks
- [ ] Staking setup completed
- [ ] Session keys set and verified
- [ ] Monitoring alerts configured
- [ ] Backup keys stored securely
- [ ] Joined Discord validator channel
- [ ] Subscribed to security announcements

### 📊 Week 1 Goals

- [ ] Maintain >99% uptime
- [ ] Earn first staking rewards
- [ ] Setup Prometheus + Grafana dashboard
- [ ] Document your setup (recovery procedures)
- [ ] Introduce yourself in forum

### 🎯 Month 1 Goals

- [ ] Achieve perfect uptime (100%)
- [ ] Attract nominators (if mainnet)
- [ ] Participate in governance vote
- [ ] Setup hot failover node (recommended)
- [ ] Contribute to community (forum/Discord)

---

## Common Issues

### Issue: Node won't start
```bash
# Check logs
sudo journalctl -u belizechain-validator -n 50

# Common fixes:
# 1. Port in use
sudo lsof -i :30333

# 2. Permission error
sudo chown -R belizechain:belizechain /data/belizechain

# 3. Missing chain spec
ls -la ~/belizechain-config/chain-spec.json
```

### Issue: Not producing blocks
1. Check you're in active validator set (Polkadot.js Apps → Staking)
2. Verify session keys: `author_hasSessionKeys` RPC call
3. Check stash is bonded: Must have ≥10,000 DALLA bonded
4. Wait for next era (up to 24 hours)

### Issue: Peers < 25
```bash
# Check firewall
sudo ufw status

# Ensure port 30333 is open
sudo ufw allow 30333/tcp

# Check if port is listening
sudo netstat -tulpn | grep 30333

# Restart node
sudo systemctl restart belizechain-validator
```

---

## Next Steps

- ✅ **Monitor performance**: [../monitoring/PROMETHEUS_SETUP.md](../monitoring/PROMETHEUS_SETUP.md)
- ✅ **Setup disaster recovery**: [../operations/DISASTER_RECOVERY.md](../operations/DISASTER_RECOVERY.md)
- ✅ **Learn security best practices**: [../security/VALIDATOR_SECURITY.md](../security/VALIDATOR_SECURITY.md)
- ✅ **Join community**: [Discord](https://discord.gg/belizechain) #validators channel

---

## Support

Need help? We're here for you!

- **Discord**: https://discord.gg/belizechain (#validators channel)
- **Forum**: https://forum.belizechain.org/c/validators
- **Email**: validators@belizechain.org (response within 24 hours)
- **Emergency**: emergency@belizechain.org (mainnet critical issues)

---

**Congratulations! You're now a BelizeChain validator! 🎉**

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
