# 🚀 Installation Guide

**Step-by-step instructions for installing BelizeChain nodes**

---

## 📋 Prerequisites

Before installing, ensure you have:
- [ ] [System requirements](requirements.md) met
- [ ] Ubuntu 22.04 LTS installed
- [ ] Root or sudo access
- [ ] Stable internet connection

**Time Required**: 30-60 minutes

---

## ⚡ Quick Install (Recommended)

```bash
# Download and run installation script
curl -sSf https://get.belizechain.org | bash

# Follow prompts:
# - Node type: validator / full / archive
# - Network: mainnet / testnet
# - Enable monitoring: yes / no

# Start node
systemctl start belizechain
systemctl enable belizechain

# Check status
systemctl status belizechain
```

**Done!** Your node is now syncing.

---

## 📦 Manual Installation

### **Step 1: Update System** (5 minutes)

```bash
# Update package list
sudo apt update

# Upgrade packages
sudo apt upgrade -y

# Install dependencies
sudo apt install -y \
  curl \
  git \
  build-essential \
  clang \
  libssl-dev \
  llvm \
  libudev-dev \
  pkg-config \
  protobuf-compiler
```

---

### **Step 2: Create User** (2 minutes)

```bash
# Create belizechain user (runs node)
sudo useradd -m -s /bin/bash belizechain

# Create data directory
sudo mkdir -p /var/lib/belizechain
sudo chown belizechain:belizechain /var/lib/belizechain
```

---

### **Step 3: Download Binary** (3 minutes)

**Option A: Download Pre-built Binary** (Fastest)

```bash
# Download latest release
VERSION="1.0.0"
wget https://github.com/BelizeChain/belizechain/releases/download/v${VERSION}/belizechain-node-linux-x86_64.tar.gz

# Extract
tar -xzf belizechain-node-linux-x86_64.tar.gz

# Install
sudo mv belizechain-node /usr/local/bin/
sudo chmod +x /usr/local/bin/belizechain-node

# Verify
belizechain-node --version
```

**Option B: Build from Source** (1-2 hours)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install WASM target
rustup target add wasm32-unknown-unknown

# Clone repository
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain

# Checkout latest stable
git checkout tags/v1.0.0

# Build (takes 30-60 minutes)
cargo build --release

# Install
sudo cp target/release/belizechain-node /usr/local/bin/
```

---

### **Step 4: Configure Node** (5 minutes)

Create configuration file `/etc/belizechain/config.toml`:

```bash
sudo mkdir -p /etc/belizechain
sudo nano /etc/belizechain/config.toml
```

**Basic Configuration**:

```toml
[node]
name = "MyBelizeNode"  # Change this!
chain = "mainnet"  # or "testnet"
base_path = "/var/lib/belizechain"

[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/30333"]
public_addresses = []  # Auto-detect

[rpc]
port = 9933
ws_port = 9944
max_connections = 100

[telemetry]
enabled = true
url = "wss://telemetry.polkadot.io/submit/"

[pruning]
mode = "archive"  # or "constrained" for pruned mode
```

**For Validators** (add these lines):

```toml
[validator]
enabled = true

[offchain_worker]
enabled = true
```

---

### **Step 5: Create Systemd Service** (3 minutes)

Create service file `/etc/systemd/system/belizechain.service`:

```bash
sudo nano /etc/systemd/system/belizechain.service
```

```ini
[Unit]
Description=BelizeChain Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=belizechain
Group=belizechain

ExecStart=/usr/local/bin/belizechain-node \
  --base-path /var/lib/belizechain \
  --chain mainnet \
  --name "MyBelizeNode" \
  --port 30333 \
  --rpc-port 9933 \
  --ws-port 9944 \
  --rpc-cors all \
  --ws-max-connections 100 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --prometheus-port 9615

Restart=always
RestartSec=10
LimitNOFILE=65536

StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

**For Validators**, add these flags to ExecStart:

```ini
  --validator \
  --offchain-worker always
```

---

### **Step 6: Configure Firewall** (3 minutes)

```bash
# Install UFW
sudo apt install ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow P2P
sudo ufw allow 30333/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status
```

---

### **Step 7: Start Node** (2 minutes)

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
● belizechain.service - BelizeChain Node
     Loaded: loaded (/etc/systemd/system/belizechain.service; enabled)
     Active: active (running) since Mon 2025-10-14 10:00:00 UTC
```

---

### **Step 8: Monitor Sync Progress** (Ongoing)

```bash
# Follow logs
journalctl -u belizechain -f

# Check sync status
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933
```

**Expected output** (after sync complete):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "peers": 50,
    "isSyncing": false,
    "shouldHavePeers": true
  },
  "id": 1
}
```

**Sync time**:
- **Pruned node**: 2-4 hours
- **Full node**: 6-12 hours
- **Archive node**: 24-48 hours

---

## 🐳 Docker Installation

### **Docker Compose** (Easiest)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  belizechain:
    image: belizechain/node:latest
    container_name: belizechain-node
    restart: unless-stopped
    ports:
      - "30333:30333"  # P2P
      - "9933:9933"    # HTTP RPC (local only)
      - "9944:9944"    # WebSocket RPC (local only)
    volumes:
      - belizechain-data:/data
    command:
      - "--base-path=/data"
      - "--chain=mainnet"
      - "--name=MyBelizeNode"
      - "--port=30333"
      - "--rpc-port=9933"
      - "--ws-port=9944"
      - "--rpc-cors=all"
      - "--telemetry-url=wss://telemetry.polkadot.io/submit/ 0"
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "3"

volumes:
  belizechain-data:
```

**Start node**:

```bash
# Start
docker-compose up -d

# Check logs
docker-compose logs -f

# Stop
docker-compose down
```

---

## ☸️ Kubernetes Deployment

`belizechain-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: belizechain-node
spec:
  replicas: 1
  selector:
    matchLabels:
      app: belizechain
  template:
    metadata:
      labels:
        app: belizechain
    spec:
      containers:
      - name: belizechain
        image: belizechain/node:latest
        ports:
        - containerPort: 30333
          name: p2p
        - containerPort: 9933
          name: http-rpc
        - containerPort: 9944
          name: ws-rpc
        volumeMounts:
        - name: data
          mountPath: /data
        command:
          - "/usr/local/bin/belizechain-node"
          - "--base-path=/data"
          - "--chain=mainnet"
          - "--name=K8sNode"
        resources:
          requests:
            memory: "8Gi"
            cpu: "4"
          limits:
            memory: "16Gi"
            cpu: "8"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: belizechain-pvc
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: belizechain-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 500Gi
```

**Deploy**:

```bash
kubectl apply -f belizechain-deployment.yaml

# Check status
kubectl get pods
kubectl logs -f deployment/belizechain-node
```

---

## 🔧 Post-Installation

### **Verify Installation**

```bash
# Check node version
belizechain-node --version

# Check service status
systemctl status belizechain

# Check listening ports
sudo netstat -tlnp | grep belizechain

# Check peer count
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933 | jq '.result.peers'
```

### **Backup Important Data**

```bash
# Backup node key (critical for validators!)
sudo cp /var/lib/belizechain/chains/belizechain/network/secret_ed25519 \
  ~/node-key-backup.key

# Backup to secure location
# Store offline in multiple locations!
```

---

## 🐛 Troubleshooting

### **Issue 1: Service won't start**

```bash
# Check journal for errors
journalctl -u belizechain -n 50

# Common causes:
# - Port 30333 already in use
# - Insufficient permissions
# - Missing dependencies
```

**Solution**:
```bash
# Check port
sudo lsof -i :30333

# Fix permissions
sudo chown -R belizechain:belizechain /var/lib/belizechain

# Reinstall dependencies
sudo apt install --reinstall libssl-dev
```

---

### **Issue 2: No peers connecting**

```bash
# Check firewall
sudo ufw status
sudo ufw allow 30333/tcp

# Check if port is open (from external)
# Use: https://www.yougetsignal.com/tools/open-ports/

# Check public IP is correct
curl ifconfig.me
```

---

### **Issue 3: Sync is slow**

```bash
# Check disk I/O
iostat -x 1

# If > 80% utilization, disk is bottleneck
# Solution: Upgrade to faster SSD

# Check network
speedtest-cli

# If < 100 Mbps, network is bottleneck
```

---

### **Issue 4: Node crashes**

```bash
# Check memory
free -h

# If swap is being used, add more RAM

# Check logs for out-of-memory errors
journalctl -u belizechain | grep -i "out of memory"

# Temporary fix: Increase swap
sudo fallocate -l 8G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

---

## 📊 Monitoring

### **Basic Monitoring**

```bash
# Watch logs
journalctl -u belizechain -f

# Watch system resources
htop

# Watch disk space
watch -n 60 df -h /var/lib/belizechain
```

### **Advanced Monitoring**

See [Monitoring Guide](monitoring.md) for Prometheus + Grafana setup.

---

## 🔄 Updates

### **Update Node**

```bash
# Stop node
sudo systemctl stop belizechain

# Backup database (optional, recommended)
sudo cp -r /var/lib/belizechain /var/lib/belizechain.backup

# Download new binary
VERSION="1.1.0"
wget https://github.com/BelizeChain/belizechain/releases/download/v${VERSION}/belizechain-node-linux-x86_64.tar.gz
tar -xzf belizechain-node-linux-x86_64.tar.gz
sudo mv belizechain-node /usr/local/bin/
sudo chmod +x /usr/local/bin/belizechain-node

# Start node
sudo systemctl start belizechain

# Monitor logs
journalctl -u belizechain -f
```

---

## ✅ Installation Checklist

- [ ] Binary installed and working
- [ ] Configuration file created
- [ ] Systemd service created and enabled
- [ ] Firewall configured
- [ ] Node syncing (check logs)
- [ ] Node key backed up
- [ ] Monitoring set up

---

## 🚀 Next Steps

**Node is running!** Now:

1. **[Configuration →](configuration.md)**  
   Fine-tune your node settings

2. **[Validator Setup →](validator-setup.md)**  
   Become a validator (if applicable)

3. **[Monitoring →](monitoring.md)**  
   Set up comprehensive monitoring

4. **[Maintenance →](maintenance.md)**  
   Learn ongoing maintenance tasks

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) for deployment support! 🚀🇧🇿
