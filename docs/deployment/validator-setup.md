# 🏛️ Validator Setup Guide

**Complete guide to becoming a BelizeChain validator**

> **Status:** Mainnet is not launched yet. The planned public endpoint
> `wss://mainnet.belizechain.org` does not resolve yet (verified 2026-10-04).
> Substitute `<operator-provided-rpc-url>` in the commands below with the RPC
> URL for the network you are targeting.

---

## 📋 Prerequisites

Before becoming a validator:

- [ ] [Node installed](installation.md) and fully synced
- [ ] [Configuration](configuration.md) optimized
- [ ] Minimum stake: **1,000 DALLA** (`pallet_belize_staking::MIN_VALIDATOR_STAKE`)
- [ ] **BelizeID KYC Level 3 (Enhanced)** — `join_validators` rejects anything lower
- [ ] Not on the sanctions list
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
# SS58 Address: r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur
#
# NOTE: BelizeChain sets SS58Prefix = 1981, so addresses render as `r1...`
# rather than the `5...` form used by default-substrate chains. The same key is
# `5GrwvaEF...` under prefix 42 — compare accounts by public key bytes, never by
# the rendered string.

# 2. Insert keys into node. The runtime's SessionKeys are BABE + GRANDPA only
#    (`impl_opaque_keys!` in runtime/src/lib.rs) — there is no ImOnline or
#    AuthorityDiscovery key to insert.
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain <path-to-testnet-spec.json> \
  --scheme Sr25519 \
  --suri "innocent perfect bus suspect..." \
  --key-type babe

belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain <path-to-testnet-spec.json> \
  --scheme Ed25519 \
  --suri "innocent perfect bus suspect..." \
  --key-type gran

# 3. Restart node as validator
sudo systemctl stop belizechain
# Edit service file to add --validator flag
sudo systemctl start belizechain

# 4. Join the validator set (requires KYC Level 3)
# See "Join Validators" section below

# 5. Set session keys
# See "Set Session Keys" section below
```

**Done!** You're now a validator candidate.

---

## 📚 Complete Setup

### **Step 1: Understand Validator Economics** (5 min read)

> **No fixed reward or slashing rate is published.** Mainnet has not launched, and
> the runtime fixes neither a per-day reward nor a per-offence slash percentage.
> A definite DALLA-per-day figure, or a "0.1% per hour offline" rate, is not
> sourced from this codebase. The mechanics below are what the pallet enforces.

**How rewards work**:

- Rewards are distributed **per epoch** by `Staking.distribute_rewards`, paying
  out from the epoch reward pool tracked in `EpochRewards`.
- PoUW participants earn additional **domain bonuses** on top of the base reward,
  claimed with `Staking.claim_pouw_with_domain_bonus`. `LastClaimedEpoch` stops a
  double claim. Fixed-point multipliers (10,000 = 1.0x):

  | Domain | Multiplier | Priority |
  |---|---|---|
  | AgriTech | 1.5x (15,000) | Critical |
  | Marine | 1.4x (14,000) | High |
  | Education | 1.3x (13,000) | High |
  | Tech | 1.1x (11,000) | Medium |
  | General | 1.0x (10,000) | Baseline |

- Federated-learning work is recorded by `submit_model_delta` with a
  `quality_score`; quantum work by `record_quantum_contribution`.

**How slashing works**:

- There is **no hard-coded slash rate**. `Staking.report_validator_offense` takes
  a `slash_percent: u32` (root-only) and `slash_validator` applies
  `Perbill::from_percent(slash_percent.min(100))` — the percentage is a governance
  decision, capped at 100%, applied to the validator's stake.
- Stake that is still unbonding remains slashable until it is withdrawn.

**Costs**: server, bandwidth and electricity are operator costs and are not
modelled on-chain.

---

### **Step 2: Generate Session Keys** (10 minutes)

Session keys are used for block production and finalization.

**Method A: On Server** (Recommended)

```bash
# Generate all keys at once
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys"}' \
  http://localhost:9944

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

BelizeChain's `SessionKeys` are **BABE + GRANDPA only** (see `impl_opaque_keys!`
in `runtime/src/lib.rs`). There is no ImOnline or AuthorityDiscovery key type —
generating `imon`/`audi` keys and inserting them has no effect on this chain.

```bash
# BABE key (block production)
belizechain-node key generate --scheme Sr25519 --output-type json > babe.json

# GRANDPA key (finalization)
belizechain-node key generate --scheme Ed25519 --output-type json > grandpa.json
```

**Insert keys into node**:

```bash
# BABE
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain <path-to-testnet-spec.json> \
  --scheme Sr25519 \
  --suri "$(cat babe.json | jq -r '.secretPhrase')" \
  --key-type babe

# GRANDPA
belizechain-node key insert \
  --base-path /var/lib/belizechain \
  --chain <path-to-testnet-spec.json> \
  --scheme Ed25519 \
  --suri "$(cat grandpa.json | jq -r '.secretPhrase')" \
  --key-type gran
```

**Verify keys inserted**:

```bash
# Check keystore
ls -la /var/lib/belizechain/chains/belizechain/keystore/
# Should see 2 files (babe, gran)

# Verify via RPC
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_hasKey", "params": ["0xYOUR_PUBLIC_KEY", "babe"]}' \
  http://localhost:9944
# Should return: {"result": true}
```

---

### **Step 3: Configure Node as Validator** (5 minutes)

Edit systemd service `/etc/systemd/system/belizechain.service`:

```ini
[Service]
ExecStart=/usr/local/bin/belizechain-node \
  --base-path /var/lib/belizechain \
  --chain /data/chain/testnet-spec.json \
  --name "MyValidator" \
  --validator \
  --offchain-worker always \
  --port 30333 \
  --rpc-port 9944 \
  --prometheus-port 9615
```

**Important flags**:
- `--validator`: Enable validator mode
- `--offchain-worker always`: Required for Proof of Useful Work
- `--name "MyValidator"`: Shown in telemetry and governance

> **Port note**: `--rpc-port 9944` serves **both** HTTP and WebSocket JSON-RPC. The
> legacy separate `--ws-port` / 9933 listener is not part of the current SDK, so
> do not pass `--ws-port` and do not expect anything on 9933.
>
> **Telemetry note**: no BelizeChain telemetry endpoint is configured. Do not
> point `--telemetry-url` at `telemetry.polkadot.io` — it is a third-party
> service and will not show this chain's nodes.

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
- Stake as validator (minimum **1,000 DALLA**)
- Pay transaction fees
- Maintain a buffer for slashing protection (recommended +5%)

**Transfer funds**:

**Option A: Using Wallet**

1. Open [Maya Wallet](../user-guides/maya-wallet.md)
2. Send 1,050 DALLA to your validator address
3. Wait 1 block confirmation (~6 seconds)

**Option B: Using Command Line**

```bash
# Install Polkadot.js tools
npm install -g @polkadot/api-cli

# Transfer (replace with your addresses)
polkadot-js-api \
  --ws <operator-provided-rpc-url> \
  tx.balances.transferKeepAlive \
  r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur \
  1050000000000000  # 1,050 DALLA (12 decimals)
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
  http://localhost:9944
```

---

### **Step 5: Join Validators** (5 minutes)

> There is **no `staking.bond` / `staking.validate` and no controller account or
> commission** on BelizeChain. Staking is PoUW-specific: a single
> `Staking.join_validators` call stakes your DALLA *and* enters the validator
> set. The account must hold **BelizeID KYC Level 3** and must not be sanctioned.

**Using Polkadot.js Apps**:

1. Go to [https://polkadot.js.org/apps](https://polkadot.js.org/apps)
2. Connect to BelizeChain:
   - Settings → Custom endpoint
   - `<operator-provided-rpc-url>`
3. Go to **Developer → Extrinsics**
4. Select **staking → joinValidators**
5. Fill in:
   - **stake**: 1,000,000,000,000,000 (1,000 DALLA, 12 decimals)
   - **computeCapacity**: your advertised compute units
   - **location**: a short region label, e.g. `Belize-City` (max 64 bytes)
6. Sign and submit

---

**Using CLI**:

```bash
# Join with 1,000 DALLA
polkadot-js-api \
  --ws <operator-provided-rpc-url> \
  --seed "your validator seed phrase" \
  tx.staking.joinValidators \
  1000000000000000 \
  1000 \
  Belize-City
```

**Verify you joined**:

```bash
# Your validator record: stake, status, compute capacity, location
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method": "query.staking.validators",
    "params": ["r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur"]
  }' \
  http://localhost:9944

# Number of registered validators
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.staking.validatorCount"}' \
  http://localhost:9944
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
  --ws <operator-provided-rpc-url> \
  --seed "your validator seed phrase" \
  tx.session.setKeys \
  0xYOUR_SESSION_KEYS_FROM_STEP_2 \
  0x00  # proof (always 0x00)
```

---

### **Step 7: Session Keys Must Be Registered** (2 minutes)

There is **no `staking.validate`** and no commission to set. Joining is what puts
you in the set. What you *do* still have to do is register session keys, because
`pallet_session` drops validators with no `NextKeys` entry when it builds the
queued validator set.

If you already ran Step 6, nothing further is needed. If you joined **before**
setting keys, you will be in the set but will not author blocks until you call
`session.setKeys` — the node logs a warning at join time in that case.

**Verify**:

```bash
# Does the chain have keys for this account?
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_hasSessionKeys","params":["0xYOUR_KEYS"]}' \
  http://localhost:9944
```

---

### **Step 8: Wait for the Next Epoch**

The validator set is rotated at **epoch boundaries**, not on an "era". BelizeChain
runs BABE with `BabeEpochDuration = 14_400` slots (~24 hours at 6-second blocks)
by default; the testnet build uses `testnet-fast-epoch` (300 slots, ~30 minutes)
so rotations can be observed quickly.

**Check validator status**:

```bash
# Active authorities for the current epoch
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method": "query.session.validators"
  }' \
  http://localhost:9944 | jq '.result'

# Registered (waiting) validators
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method": "query.staking.validatorCount"
  }' \
  http://localhost:9944
```
**Validator states**:
- **Registered**: Present in `Staking.Validators`, awaiting the next epoch rotation
- **Active**: Listed in `Session.Validators` for the current epoch (authoring blocks)
- **Left**: Removed from the set; stake is unbonding until withdrawn

---

## 🎯 Validator Operations

### **Check Validator Status**

```bash
# Am I an active authority this epoch?
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.session.validators"}' \
  | jq '.result | map(select(. == "r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur"))'

# My validator record (stake, status, compute capacity, location)
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.staking.validators","params":["r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur"]}' \
  | jq '.result'
```

---

### **Increase Stake**

There is **no `staking.bondExtra`**. Stake is set once at join time, so changing it
means leaving and rejoining:

```bash
# 1. Leave the set (starts the unbonding period)
polkadot-js-api --ws <operator-provided-rpc-url> --seed "your seed" \
  tx.staking.leaveValidators

# 2. Wait for UnbondingPeriod to elapse — 14_400 blocks, ~24h at 6s blocks

# 3. Withdraw the old stake
polkadot-js-api --ws <operator-provided-rpc-url> --seed "your seed" \
  tx.staking.withdrawUnbonded

# 4. Rejoin with the larger amount
polkadot-js-api --ws <operator-provided-rpc-url> --seed "your seed" \
  tx.staking.joinValidators 5000000000000000 1000 Belize-City
```

---

### **Decrease Stake / Exit the Set**

There is **no `staking.unbond`**. Leaving is a single call, and the stake is
returned after the unbonding period:

```bash
# 1. Leave — removed from the active set immediately
polkadot-js-api --ws <operator-provided-rpc-url> --seed "your seed" \
  tx.staking.leaveValidators

# 2. Wait 14_400 blocks (~24h at 6s blocks)

# 3. Withdraw the unbonded stake
polkadot-js-api --ws <operator-provided-rpc-url> --seed "your seed" \
  tx.staking.withdrawUnbonded
```

> The unbonding period is `UnbondingPeriod = ConstU32<14_400>` on
> `pallet_belize_staking` in `runtime/src/lib.rs` — **not** 28 days. (The 28-day
> `ValidatorUnbondingPeriod` on the *Consensus* pallet is a different mechanism.)

---

### **Claim PoUW Domain Bonus**

```bash
# Claim the domain-bonus portion of epoch rewards (once per epoch)
polkadot-js-api --ws <operator-provided-rpc-url> --seed "your seed" \
  tx.staking.claimPouwWithDomainBonus
```

---

### **Stop Validating**

`leaveValidators` above is the "stop validating" action — there is no separate
`staking.chill`. Your stake stops earning and is returned after the unbonding
period.

---

## 🏆 Proof of Useful Work (PoUW)

BelizeChain validators earn extra rewards for contributing to federated learning.

### **Setup Federated Learning Client** (30 minutes)

```bash
# Install Python dependencies. Nawal is a sibling repo, not a directory inside
# the node repo.
cd ../nawal-ai
pip install -r requirements.txt

# Configure client
cp config.dev.yaml config.validator.yaml
nano config.validator.yaml
```

**config.validator.yaml**:

```yaml
server:
  url: "https://testnet.belizechain.org/api/nawal"
  port: 8080

client:
  validator_address: "r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur"  # Your validator address
  stake: 1000  # Your staked amount (DALLA)

model:
  batch_size: 32
  epochs: 5

privacy:
  dp_epsilon: 1.0
  dp_delta: 0.00001
```

**Start FL client**:

```bash
# No systemd unit ships in either repo — write your own, or run the client under
# your existing process supervisor. A minimal unit looks like:
#
#   [Unit]
#   Description=BelizeChain FL client
#   After=network-online.target
#   [Service]
#   WorkingDirectory=/opt/nawal-ai
#   ExecStart=/opt/nawal-ai/.venv/bin/python -m nawal.client
#   Restart=always
#   [Install]
#   WantedBy=multi-user.target

sudo systemctl enable --now belizechain-fl

# Monitor FL logs
journalctl -u belizechain-fl -f
```

---

### **PoUW Rewards Calculation**

`Staking::calculate_validator_reward` combines five weighted components, each a
share of the epoch `base_reward`. The weights are set in the pallet (Phase 2.2):

| Component | Weight | Source field |
|---|---|---|
| Quality | **25%** | `quality_score` (0-100) |
| Timeliness | **20%** | `timeliness_score` (0-100) |
| Honesty | **20%** | `honesty_score` (0-100) |
| Quantum contribution | **30%** | `quantum_score` |
| Stake / uptime bonus | **5%** | proportional to stake, clamped at 1.0x |

```
reward = 0.25 × (quality_score/100)    × base_reward
       + 0.20 × (timeliness_score/100) × base_reward
       + 0.20 × (honesty_score/100)    × base_reward
       + 0.30 × (quantum_score/100)    × base_reward
       + 0.05 × stake_ratio            × base_reward
```

> Earlier revisions of this guide gave the weights as 40% / 30% / 30% with no
> quantum term. Those figures are stale — the pallet comment records them as
> having been reduced. Do not use them for reward modelling.
>
> `base_reward` comes from the epoch pool, so there is no fixed DALLA-per-day
> figure to quote. Derive it from `EpochRewards` for the epoch you care about.

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
1. Stake below the 1,000 DALLA minimum (the `join_validators` call would have failed)
2. KYC below Level 3, or the account is on the sanctions list
3. Node offline, not synced, or not in `--validator` mode
4. Session keys not registered — you are in `Staking.Validators` but absent from
   `Session.Validators`, because `pallet_session` drops validators with no
   `NextKeys` entry
5. The epoch has not rotated yet

**Solutions**:

```bash
# Check node status
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
  | jq '.result.isSyncing'
# Should be false

# Am I registered at all?
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.staking.validatorCount"}' \
  | jq '.result'

# Am I an authority this epoch?
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.session.validators"}' \
  | jq '.result'

# Verify session keys
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_hasSessionKeys","params":["0xYOUR_KEYS"]}' \
  | jq '.result'
# Should be true
```

---

### **Issue: Getting slashed**

> There are **no fixed slash rates** in the runtime. `report_validator_offense`
> takes the percentage as an argument and `slash_validator` caps it at 100%.
> Anything quoted as "0.1% per hour offline" or "5% for equivocation" is not
> sourced from this codebase — read the actual offence record instead.

**How to check what happened**:

```bash
# ValidatorSlashed events carry the validator and amount
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.system.events"}' \
  | jq '.result | map(select(.event.section == "staking"))'
```

**Double signing**: **NEVER run validator keys on 2+ nodes simultaneously!**
Stake that is still unbonding remains slashable until withdrawn.

**Prevention**:
- Only one node with validator keys
- Test with separate keys before switching
- High-availability setup, redundant internet, UPS power backup
- Monitoring with auto-restart

---

### **Issue: Low PoUW rewards**

```bash
# Check FL client status
systemctl status belizechain-fl

# Check FL logs
journalctl -u belizechain-fl -n 100

# Check quality score
curl -s https://testnet.belizechain.org/api/nawal/validators/r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur/score

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
- [ ] BelizeID KYC Level 3 verified
- [ ] Minimum 1,000 DALLA staked via `staking.joinValidators`
- [ ] Stake buffer for slashing protection (+5% recommended)

**Security**:
- [ ] Firewall configured (only 30333 open)
- [ ] Sentry node architecture (recommended)
- [ ] Session keys backed up (3+ locations)
- [ ] Server hardened (see [security.md](security.md))

**Monitoring**:
- [ ] Prometheus scraping metrics
- [ ] Grafana dashboard configured
- [ ] Alerts set up (offline, missed blocks, disk space)

**PoUW** (optional, earns domain bonuses):
- [ ] FL client installed and running
- [ ] FL client configured with validator address
- [ ] FL submissions successful (`submit_model_delta`)

---

## 📊 Validator Economics

> **No reward or ROI figures are published.** The numbers that previously
> appeared here (45,000 DALLA/month base, 13,500 DALLA PoUW bonus, "$0.01 per
> DALLA", 36.6% annual ROI) were not derived from the runtime or from any
> published token price, and the 10,000 DALLA stake they assumed is twice the
> real 1,000 DALLA minimum.

What can be stated from the code:

| Item | Value | Source |
|---|---|---|
| Minimum stake | 1,000 DALLA | `pallet_belize_staking::MIN_VALIDATOR_STAKE` |
| Unbonding period | 14,400 blocks (~24h at 6s) | `UnbondingPeriod` in `runtime/src/lib.rs` |
| Reward pool | per epoch, read from storage | `EpochRewards` |
| Reward split | 25% quality, 20% timeliness, 20% honesty, 30% quantum, 5% stake bonus | `calculate_validator_reward` |
| Slash amount | governance-supplied, capped at 100% | `report_validator_offense` |
| Domain bonuses | AgriTech 1.5x … General 1.0x | domain multiplier function |

To model your own returns, read the live epoch pool:

```bash
curl -s http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"query.staking.epochRewards"}' \
  | jq '.result'
```

---

## 🚀 Next Steps

1. **[Maintenance →](maintenance.md)**  
   Learn ongoing validator maintenance

2. **[Monitoring →](monitoring.md)**  
   Set up comprehensive monitoring

3. **[Security →](security.md)**  
   Harden your validator security
