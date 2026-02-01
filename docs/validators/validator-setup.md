# Validator Setup & Operations

**Hardware Requirements • Node Installation • Key Management • PoUW Participation**

Complete guide to running a BelizeChain validator node.

---

## Validator Types

BelizeChain supports **four validator types** based on capabilities:

| Type | Requirements | Rewards | Use Case |
|------|--------------|---------|----------|
| **Standard** | 10K DALLA stake + KYC Verified | Base 144K DALLA/day | Basic block production |
| **Nawal** | Standard + GPU (8GB+ VRAM) | Base + 50-350 DALLA/session | Federated learning participation |
| **Kinich** | Standard + Azure Quantum account | Base + 50-200 DALLA/job | Quantum work execution |
| **Full** | Standard + GPU + Azure Quantum | All rewards combined | Maximum earnings (54.6M DALLA/year) |

---

## Hardware Requirements

### Standard Validator (Minimum)

```
CPU: 8 cores (x86_64, 3.0+ GHz)
RAM: 32 GB
Storage: 500 GB NVMe SSD
Network: 100 Mbps symmetric, <50ms latency to validators
OS: Ubuntu 22.04 LTS
```

### Nawal Validator (GPU Required)

```
GPU: NVIDIA RTX 3090/4090 or A100 (8+ GB VRAM, CUDA 12.0+)
CPU: 16 cores
RAM: 64 GB
Storage: 1 TB NVMe SSD
```

### Full Validator (Recommended for Max Earnings)

```
GPU: NVIDIA A100 (40GB VRAM)
CPU: 32 cores (AMD EPYC or Intel Xeon)
RAM: 128 GB
Storage: 2 TB NVMe SSD (RAID 1 recommended)
Network: 1 Gbps symmetric, redundant ISPs
```

---

## Node Installation

### 1. System Preparation

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y \
    build-essential \
    clang \
    curl \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler

# Create dedicated user
sudo useradd -m -s /bin/bash belizechain
sudo usermod -aG sudo belizechain

# Switch to belizechain user
sudo su - belizechain
```

### 2. Install Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

rustup default stable
rustup update stable
rustup target add wasm32-unknown-unknown
```

### 3. Build BelizeChain Node

```bash
# Clone repository
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain/

# Build release binary (~10 minutes)
cargo build --release

# Verify binary
./target/release/belizechain-node --version
# Expected: belizechain-node 4.0.0-stable2512
```

### 4. Install as Systemd Service

```bash
# Copy binary
sudo cp target/release/belizechain-node /usr/local/bin/

# Create data directory
sudo mkdir -p /var/lib/belizechain
sudo chown belizechain:belizechain /var/lib/belizechain

# Create systemd service
sudo tee /etc/systemd/system/belizechain.service > /dev/null <<EOF
[Unit]
Description=BelizeChain Validator Node
After=network.target

[Service]
Type=simple
User=belizechain
WorkingDirectory=/var/lib/belizechain
ExecStart=/usr/local/bin/belizechain-node \\
    --validator \\
    --name "YOUR_VALIDATOR_NAME" \\
    --base-path /var/lib/belizechain \\
    --chain mainnet \\
    --port 30333 \\
    --rpc-port 9933 \\
    --prometheus-port 9615 \\
    --prometheus-external \\
    --rpc-cors all \\
    --rpc-methods Safe

Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable belizechain
sudo systemctl start belizechain

# Check status
sudo systemctl status belizechain

# View logs
sudo journalctl -u belizechain -f
```

---

## Key Management

### 1. Generate Session Keys

```bash
# Generate keys via RPC
curl -H "Content-Type: application/json" -d '{
  "id":1,
  "jsonrpc":"2.0",
  "method":"author_rotateKeys",
  "params":[]
}' http://localhost:9933

# Response:
# {
#   "jsonrpc": "2.0",
#   "result": "0xabc123def456...",  # THIS IS YOUR SESSION KEY
#   "id": 1
# }

# Save this key securely!
```

### 2. Create Controller & Stash Accounts

**Using Polkadot.js Apps:**

1. Navigate to https://polkadot.js.org/apps/?rpc=wss://rpc.belizechain.org
2. Accounts → Add account
3. Create **Stash account** (holds funds, keep offline)
   - Save 12/24-word seed phrase securely
   - Transfer 10,000+ DALLA (minimum stake)
4. Create **Controller account** (manages validator, can be online)
   - Transfer 100 DALLA (for transaction fees)

**Using `subkey` (command line):**

```bash
# Install subkey
cargo install --force subkey --git https://github.com/paritytech/polkadot-sdk

# Generate stash account
subkey generate --scheme sr25519
# Secret phrase: /* KEEP THIS ULTRA SECURE */
# Public key (hex): 0xabc...
# Account ID: 5GrwvaEF...
# SS58 Address: 5GrwvaEF...

# Generate controller account
subkey generate --scheme sr25519
# Secret phrase: /* ALSO KEEP SECURE */
# Public key (hex): 0xdef...
# Account ID: 5FHneW...
# SS58 Address: 5FHneW...
```

---

## Register as Validator

### 1. Complete KYC

```javascript
// Apply for Verified KYC (required for validators)
await api.tx.compliance.submitKycApplication(
  'Verified',
  [idDocumentHash, addressProofHash, selfieHash]
).signAndSend(controllerAccount);

// Wait 1-3 business days for FSC approval
```

### 2. Bond Tokens

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function bondTokens() {
  const provider = new WsProvider('wss://rpc.belizechain.org');
  const api = await ApiPromise.create({ provider });
  
  // Bond 10,000 DALLA (minimum)
  const bondAmount = 10_000 * 1e12;  // 12 decimals
  
  const tx = api.tx.staking.bond(
    bondAmount,
    { Staked: null }  // Rewards compounded automatically
  );
  
  await tx.signAndSend(stashAccount, ({ status }) => {
    if (status.isInBlock) {
      console.log('✅ Bonded 10,000 DALLA');
    }
  });
}
```

### 3. Set Session Keys

```javascript
// Set the session key from author_rotateKeys
const sessionKey = '0xabc123def456...';  // From earlier

await api.tx.session.setKeys(sessionKey, '0x')
  .signAndSend(controllerAccount);
```

### 4. Register Validator

```javascript
// Declare validator intent
await api.tx.staking.validate({
  commission: 10_000_000,  // 10% commission (in perbill: 10_000_000 / 1_000_000_000)
  blocked: false
}).signAndSend(controllerAccount);

console.log('✅ Validator registered!');
console.log('Wait for next era (~6 hours) to start validating');
```

---

## Nawal Federated Learning (Optional)

### Setup GPU Node

```bash
# Install NVIDIA drivers
sudo ubuntu-drivers install nvidia-driver-550
nvidia-smi  # Verify GPU detected

# Install CUDA 12.0
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-ubuntu2204.pin
sudo mv cuda-ubuntu2204.pin /etc/apt/preferences.d/cuda-repository-pin-600
wget https://developer.download.nvidia.com/compute/cuda/12.0.0/local_installers/cuda-repo-ubuntu2204-12-0-local_12.0.0-525.60.13-1_amd64.deb
sudo dpkg -i cuda-repo-ubuntu2204-12-0-local_12.0.0-525.60.13-1_amd64.deb
sudo apt update
sudo apt install cuda

# Setup Python environment
cd ~/belizechain/nawal/
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Start Nawal client
python -m nawal.client.fl_client \
    --node-id $(cat /var/lib/belizechain/node-id) \
    --server-address 51.120.45.67:8080 \
    --gpu-device 0
```

**Earnings:** 50-350 DALLA per training session (7 sessions/week = 127K DALLA/year)

---

## Kinich Quantum Work (Optional)

### Setup Azure Quantum

```bash
# Install Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login to Azure
az login

# Create quantum workspace
az quantum workspace create \
    --resource-group belizechain-validators \
    --workspace-name validator-quantum \
    --location westus \
    --storage-account validatorquantumstorage

# Setup Kinich client
cd ~/belizechain/kinich/
source .venv/bin/activate
pip install -r requirements.txt

# Configure credentials
export AZURE_QUANTUM_WORKSPACE_ID="..."
export AZURE_QUANTUM_LOCATION="westus"
export AZURE_SUBSCRIPTION_ID="..."

# Start Kinich node
python -m kinich.core.quantum_node \
    --blockchain-rpc wss://rpc.belizechain.org \
    --validator-account 5GrwvaEF... \
    --backends ionq,quantinuum
```

**Earnings:** 50-200 DALLA per quantum job (10 jobs/week = 78K DALLA/year)

---

## Monitoring

### Prometheus Metrics

```bash
# Install Prometheus
sudo apt install prometheus

# Configure scrape target (/etc/prometheus/prometheus.yml)
scrape_configs:
  - job_name: 'belizechain'
    static_configs:
      - targets: ['localhost:9615']

# Restart Prometheus
sudo systemctl restart prometheus

# Access metrics: http://localhost:9090
```

**Key metrics to monitor:**
- `substrate_block_height` (should increase every 6 seconds)
- `substrate_finality_lag` (<6 seconds healthy)
- `substrate_peers_count` (>50 peers)
- `substrate_transactions_in_pool` (<1000)

### Grafana Dashboard

```bash
# Install Grafana
sudo apt install grafana

# Import BelizeChain dashboard
# ID: belizechain-validator-dashboard.json
```

---

## Troubleshooting

### Node Not Producing Blocks

```bash
# Check if in active validator set
curl -H "Content-Type: application/json" -d '{
  "id":1,
  "jsonrpc":"2.0",
  "method":"state_call",
  "params":["SessionApi_validators", "0x"]
}' http://localhost:9933

# If not in set, check:
# 1. Bonded amount >= 10,000 DALLA
# 2. Session keys set correctly
# 3. KYC level >= Verified
# 4. Wait for next era (query: staking.currentEra)
```

### High Finality Lag

```bash
# Increase peer connections
belizechain-node --validator --out-peers 50 --in-peers 50

# Check network latency
ping -c 10 other-validator-ip
# Should be <100ms
```

---

## Security Best Practices

1. **Hardware security module (HSM):** Use YubiHSM 2 or Ledger for session keys
2. **Firewall:** Only allow P2P port (30333), block RPC/WS externally
3. **DDoS protection:** Use Cloudflare or AWS Shield
4. **Backups:** Weekly snapshots to Azure Blob Storage
5. **Monitoring:** Set up alerts for >1 minute downtime
6. **Updates:** Subscribe to GitHub releases, upgrade within 24 hours

---

## Related Documentation

- [Staking Pallet API](../developer-guides/pallet-apis-financial.md#staking-pallet)
- [Staking Rewards](../economics/staking-rewards.md)
- [Azure Deployment](../deployment/azure-kubernetes-deployment.md)
- [Security Hardening](../security/security-audit-results.md)
