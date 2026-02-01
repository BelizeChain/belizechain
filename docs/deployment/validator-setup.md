# 🏛️ Validator Setup Guide

**Complete guide to becoming a BelizeChain validator**

---

## 📋 Prerequisites

Before becoming a validator:

- [ ] [Node installed](installation.md) and fully synced
- [ ] [Configuration](configuration.md) optimized
- [ ] Minimum stake: **10,000 DALLA**
- [ ] Server requirements met (see [requirements](requirements.md))
- [ ] Understanding of validator responsibilities

**⚠️ Warning**: Validators can be slashed for misbehavior! Read carefully.

---

## ⚡ Quick Setup (15 minutes)

```bash
# 1. Generate validator keys
belizechain-node key generate --scheme Sr25519

# Output (example):
# Secret phrase: innocent perfect bus suspect awake slim vocal ...
# Public key (hex): 0x1234...
# Account ID: 5GrwvaEF...
# SS58 Address: 5GrwvaEF...

# 2. Insert keys into node
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --scheme Sr25519 \
  --suri "innocent perfect bus suspect..." \
  --key-type babe

belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --scheme Ed25519 \
  --suri "innocent perfect bus suspect..." \
  --key-type gran

# 3. Restart node as validator
sudo systemctl stop belizechain
# Edit service file to add --validator flag
sudo systemctl start belizechain

# 4. Bond stake (use wallet or polkadot.js)
# See "Bond Stake" section below

# 5. Set session keys
# See "Set Session Keys" section below
```

**Done!** You're now a validator candidate.

---

## 📚 Complete Setup

### **Step 1: Understand Validator Economics** (5 min read)

**Rewards**:
- **Base rewards**: 1,500 DALLA per day (if elected)
- **Useful Work bonus**: Up to 500 DALLA per day (federated learning)
- **Total potential**: ~730,000 DALLA per year

**Costs**:
- Server: $120-400/month
- Stake: 10,000 DALLA minimum
- Electricity/bandwidth: ~$50/month

**Risks**:
- **Offline slashing**: 0.1% per hour offline (max 1% per day)
- **Equivocation slashing**: 5% of stake (double signing)
- **Inactivity kick**: After 48 hours offline

**Break-even calculation**:
```
Monthly rewards: ~61,666 DALLA
Monthly costs: ~$250 USD
Break-even: ~$0.004 per DALLA
```

---

### **Step 2: Generate Session Keys** (10 minutes)

Session keys are used for block production and finalization.

**Method A: On Server** (Recommended)

```bash
# Generate all keys at once
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys"}' \
  http://localhost:9933

# Output (your session keys):
# {
#   "jsonrpc": "2.0",
#   "result": "0xabcd1234...",  # Save this!
#   "id": 1
# }
```

**Save the result!** You'll need it to set session keys.

---

**Method B: Generate Manually** (Advanced)

```bash
# BABE key (block production)
belizechain-node key generate --scheme Sr25519 --output-type json > babe.json

# GRANDPA key (finalization)
belizechain-node key generate --scheme Ed25519 --output-type json > grandpa.json

# ImOnline key (heartbeat)
belizechain-node key generate --scheme Sr25519 --output-type json > imonline.json

# Authority Discovery key
belizechain-node key generate --scheme Sr25519 --output-type json > authority_discovery.json
```

**Insert keys into node**:

```bash
# BABE
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --scheme Sr25519 \
  --suri "$(cat babe.json | jq -r '.secretPhrase')" \
  --key-type babe

# GRANDPA
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --scheme Ed25519 \
  --suri "$(cat grandpa.json | jq -r '.secretPhrase')" \
  --key-type gran

# ImOnline
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --scheme Sr25519 \
  --suri "$(cat imonline.json | jq -r '.secretPhrase')" \
  --key-type imon

# Authority Discovery
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --scheme Sr25519 \
  --suri "$(cat authority_discovery.json | jq -r '.secretPhrase')" \
  --key-type audi
```

**Verify keys inserted**:

```bash
# Check keystore
ls -la /var/lib/belizechain/chains/belizechain/keystore/
# Should see 4 files

# Verify via RPC
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_hasKey", "params": ["0xYOUR_PUBLIC_KEY", "babe"]}' \
  http://localhost:9933
# Should return: {"result": true}
```

---

### **Step 3: Configure Node as Validator** (5 minutes)

Edit systemd service `/etc/systemd/system/belizechain.service`:

```ini
[Service]
ExecStart=/usr/local/bin/belizechain-node \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --name "MyValidator" \
  --validator \
  --offchain-worker always \
  --port 30333 \
  --rpc-port 9933 \
  --ws-port 9944 \
  --prometheus-port 9615 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0'
```

**Important flags**:
- `--validator`: Enable validator mode
- `--offchain-worker always`: Required for Proof of Useful Work
- `--name "MyValidator"`: Shown in telemetry and governance

**Restart node**:

```bash
sudo systemctl daemon-reload
sudo systemctl restart belizechain

# Verify validator mode
journalctl -u belizechain -f | grep -i "validator"
# Should see: "Running in --validator mode"
```

---

### **Step 4: Fund Validator Account** (10 minutes)

You need DALLA to:
- Bond as validator (minimum 10,000 DALLA)
- Pay transaction fees (~10 DALLA)
- Maintain balance for slashing protection (recommended +5%)

**Transfer funds**:

**Option A: Using Wallet**

1. Open [Maya Wallet](../user-guides/using-wallet.md)
2. Send 10,500 DALLA to your validator address
3. Wait 1 block confirmation (~6 seconds)

**Option B: Using Command Line**

```bash
# Install Polkadot.js tools
npm install -g @polkadot/api-cli

# Transfer (replace with your addresses)
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  tx.balances.transfer \
  5GrwvaEF... \
  10500000000000000  # 10,500 DALLA (12 decimals)
```

**Verify balance**:

```bash
curl -H "Content-Type: application/json" \
  -d '{
    "id":1, 
    "jsonrpc":"2.0", 
    "method": "state_getStorage",
    "params": ["0xYOUR_STORAGE_KEY"]
  }' \
  http://localhost:9933
```

---

### **Step 5: Bond Stake** (5 minutes)

**Using Polkadot.js Apps**:

1. Go to [https://polkadot.js.org/apps](https://polkadot.js.org/apps)
2. Connect to BelizeChain:
   - Settings → Custom endpoint
   - `wss://mainnet.belizechain.org`
3. Go to **Network → Staking → Account actions**
4. Click **+ Stash**
5. Fill in:
   - **Stash account**: Your validator account
   - **Controller account**: Same or separate account
   - **Value bonded**: 10,000 DALLA (or more)
   - **Payment destination**: Staked (compound rewards)
6. Click **Bond** and sign transaction

---

**Using CLI**:

```bash
# Bond 10,000 DALLA
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your validator seed phrase" \
  tx.staking.bond \
  5GrwvaEF...  # controller account (can be same) \
  10000000000000000  # 10,000 DALLA (12 decimals) \
  Staked  # rewards destination
```

**Verify bonded**:

```bash
# Check bonded amount
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method": "query.staking.bonded",
    "params": ["5GrwvaEF..."]
  }' \
  http://localhost:9933
```

---

### **Step 6: Set Session Keys** (5 minutes)

Tell the chain your validator's session keys.

**Using Polkadot.js Apps**:

1. Network → Staking → Account actions
2. Click **Set Session Key** (next to your validator)
3. Paste your session keys (from Step 2)
4. Sign and submit transaction

---

**Using CLI**:

```bash
# Set session keys
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your validator seed phrase" \
  tx.session.setKeys \
  0xYOUR_SESSION_KEYS_FROM_STEP_2 \
  0x00  # proof (always 0x00)
```

---

### **Step 7: Validate** (2 minutes)

Signal your intention to validate.

**Using Polkadot.js Apps**:

1. Network → Staking → Account actions
2. Click **Validate** (next to your validator)
3. Set **Commission**: 0-100% (suggestion: 5-10%)
4. Sign and submit transaction

---

**Using CLI**:

```bash
# Start validating with 5% commission
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your validator seed phrase" \
  tx.staking.validate \
  '{"commission": 50000000}'  # 5% (in per billion: 5% = 50,000,000)
```

---

### **Step 8: Wait for Election** (Up to 24 hours)

Validators are elected every **era** (24 hours).

**Check validator status**:

```bash
# Check if in active set
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method": "query.session.validators"
  }' \
  http://localhost:9933 | jq '.result'

# Check if in waiting set
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method": "query.staking.validators"
  }' \
  http://localhost:9933
```

**Validator states**:
- **Waiting**: Registered, not yet elected
- **Active**: Currently validating (earning rewards!)
- **Inactive**: Was validating, no longer elected

---

## 🎯 Validator Operations

### **Check Validator Status**

```bash
# Am I in the active validator set?
curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.session.validators"}' \
  | jq '.result | map(select(. == "5GrwvaEF..."))'

# My current stake
curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.staking.ledger","params":["5GrwvaEF..."]}' \
  | jq '.result'
```

---

### **Increase Stake**

```bash
# Bond additional 5,000 DALLA
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your seed" \
  tx.staking.bondExtra \
  5000000000000000  # 5,000 DALLA
```

---

### **Decrease Stake** (Unbond)

```bash
# Unbond 1,000 DALLA
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your seed" \
  tx.staking.unbond \
  1000000000000000

# Wait 28 days (unbonding period)

# Withdraw unbonded funds
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your seed" \
  tx.staking.withdrawUnbonded \
  0  # slashing spans
```

---

### **Change Commission**

```bash
# Change to 10% commission
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your seed" \
  tx.staking.validate \
  '{"commission": 100000000}'  # 10% in per billion
```

---

### **Stop Validating**

```bash
# Stop validating (chill)
polkadot-js-api \
  --ws wss://mainnet.belizechain.org \
  --seed "your seed" \
  tx.staking.chill

# Your stake remains bonded but you stop validating
```

---

## 🏆 Proof of Useful Work (PoUW)

BelizeChain validators earn extra rewards for contributing to federated learning.

### **Setup Federated Learning Client** (30 minutes)

```bash
# Install Python dependencies
cd /home/wicked/BelizeChain/belizechain/nawal
pip install -r requirements.txt

# Configure client
cp config.dev.yaml config.validator.yaml
nano config.validator.yaml
```

**config.validator.yaml**:

```yaml
server:
  url: "https://fl.belizechain.org"
  port: 8080

client:
  validator_address: "5GrwvaEF..."  # Your validator address
  stake: 10000  # Your bonded stake
  
model:
  batch_size: 32
  epochs: 5
  
privacy:
  dp_epsilon: 1.0
  dp_delta: 0.00001
```

**Start FL client**:

```bash
# Run as systemd service
sudo cp /home/wicked/BelizeChain/belizechain/infra/systemd/belizechain-fl.service \
  /etc/systemd/system/

sudo systemctl enable belizechain-fl
sudo systemctl start belizechain-fl

# Monitor FL logs
journalctl -u belizechain-fl -f
```

---

### **PoUW Rewards Calculation**

Rewards are based on:
- **Quality** (40%): Model improvement accuracy
- **Timeliness** (30%): Submission before deadline
- **Honesty** (30%): Privacy compliance verification

**Example**:
```
Base validator reward: 1,500 DALLA/day
Quality score: 85%
Timeliness score: 100% (submitted early)
Honesty score: 95%

PoUW bonus = 500 × (0.40×0.85 + 0.30×1.00 + 0.30×0.95)
            = 500 × (0.34 + 0.30 + 0.285)
            = 500 × 0.925
            = 462.5 DALLA/day

Total daily reward = 1,500 + 462.5 = 1,962.5 DALLA/day
```

---

## 🛡️ Validator Security

### **Sentry Node Architecture**

**Problem**: Direct exposure to internet increases attack surface.

**Solution**: Use sentry nodes as proxy.

```
Internet <-> Sentry Node 1 <-+
Internet <-> Sentry Node 2 <-+-> Validator (private network)
Internet <-> Sentry Node 3 <-+
```

**Validator configuration** (`config.toml`):

```toml
[network]
# Only connect to sentries
reserved_peers = [
  "/ip4/10.0.1.2/tcp/30333/p2p/12D3KooW...",  # Sentry 1
  "/ip4/10.0.1.3/tcp/30333/p2p/12D3KooW...",  # Sentry 2
  "/ip4/10.0.1.4/tcp/30333/p2p/12D3KooW..."   # Sentry 3
]
reserved_only = true

# No public address
public_addresses = []
```

**Sentry configuration**:

```toml
[network]
# Connect to validator
reserved_peers = [
  "/ip4/10.0.1.1/tcp/30333/p2p/12D3KooW..."  # Validator
]

# Accept public connections
listen_addresses = ["/ip4/0.0.0.0/tcp/30333"]
```

---

### **Key Management**

**⚠️ CRITICAL**: Backup session keys!

```bash
# Backup keystore
sudo tar -czf validator-keys-backup-$(date +%Y%m%d).tar.gz \
  /var/lib/belizechain/chains/belizechain/keystore/

# Encrypt backup
gpg --symmetric --cipher-algo AES256 \
  validator-keys-backup-*.tar.gz

# Store in 3+ locations:
# 1. Encrypted USB drive (offline)
# 2. Encrypted cloud storage
# 3. Hardware security module (HSM)
```

---

### **Monitoring & Alerts**

Set up alerts for:
- Node offline
- Low peer count (< 5)
- Missed blocks
- Low disk space (< 20%)
- High CPU/memory usage

See [Monitoring Guide](monitoring.md) for setup.

---

## 🐛 Troubleshooting

### **Issue: Not getting elected**

**Possible causes**:
1. Stake too low (need top 300 validators)
2. Commission too high
3. Node offline or not synced
4. Session keys not set correctly

**Solutions**:

```bash
# Check stake ranking
curl -s https://api.belizechain.org/validators | jq '.[].stake' | sort -n

# Check node status
curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
  | jq '.result.isSyncing'
# Should be false

# Verify session keys
curl -s http://localhost:9933 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_hasSessionKeys","params":["0xYOUR_KEYS"]}' \
  | jq '.result'
# Should be true
```

---

### **Issue: Getting slashed**

**Offline slash**:
- 0.1% per hour offline
- Maximum 1% per day
- Kicks validator after 48 hours

**Prevention**:
- High-availability setup (99.9% uptime)
- Redundant internet connections
- UPS power backup
- Monitoring with auto-restart

---

**Equivocation slash** (double signing):
- 5% of stake slashed
- **NEVER run validator keys on 2+ nodes simultaneously!**

**Prevention**:
- Only one node with validator keys
- Test with separate keys before switching
- Use session rotation carefully

---

### **Issue: Low PoUW rewards**

```bash
# Check FL client status
systemctl status belizechain-fl

# Check FL logs
journalctl -u belizechain-fl -n 100

# Check quality score
curl -s https://fl.belizechain.org/api/validators/5GrwvaEF.../score

# Common issues:
# - FL client not running
# - Low-quality model (increase epochs, tune hyperparameters)
# - Missed submissions (check internet connectivity)
```

---

## ✅ Validator Checklist

**Pre-launch**:
- [ ] Node synced to latest block
- [ ] Validator mode enabled (`--validator` flag)
- [ ] Session keys generated and set
- [ ] Minimum 10,000 DALLA bonded
- [ ] Commission rate configured
- [ ] Telemetry showing in [telemetry.polkadot.io](https://telemetry.polkadot.io)

**Security**:
- [ ] Firewall configured (only 30333 open)
- [ ] Sentry node architecture (recommended)
- [ ] Session keys backed up (3+ locations)
- [ ] Server hardened (see [security.md](security.md))

**Monitoring**:
- [ ] Prometheus scraping metrics
- [ ] Grafana dashboard configured
- [ ] Alerts set up (offline, missed blocks, disk space)

**PoUW** (optional, +30% rewards):
- [ ] FL client installed and running
- [ ] FL client configured with validator address
- [ ] FL submissions successful

---

## 📊 Validator Economics

**Monthly breakdown** (10,000 DALLA stake):

| Item | Amount |
|------|--------|
| Base rewards | 45,000 DALLA |
| PoUW bonus (if 90% score) | 13,500 DALLA |
| **Total rewards** | **58,500 DALLA** |
| Server costs | -$280 USD |
| **Net income** | **58,500 DALLA - $280 USD** |

**ROI calculation** (at $0.01 per DALLA):
```
Monthly rewards: 58,500 DALLA = $585 USD
Monthly costs: $280 USD
Monthly profit: $305 USD
Annual profit: $3,660 USD
ROI on 10K stake: 36.6% per year
```

---

## 🚀 Next Steps

1. **[Maintenance →](maintenance.md)**  
   Learn ongoing validator maintenance

2. **[Monitoring →](monitoring.md)**  
   Set up comprehensive monitoring

3. **[Security →](security.md)**  
   Harden your validator security

---

**Questions?** Join [Validator Discord](https://discord.gg/belizechain-validators) for support! 🏛️🇧🇿
