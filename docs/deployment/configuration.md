# ⚙️ Configuration Guide

**Complete reference for configuring BelizeChain nodes**

---

## 📋 Configuration Methods

BelizeChain nodes can be configured through:

1. **Command-line flags** (highest priority)
2. **Configuration file** (`config.toml`)
3. **Environment variables** (lowest priority)

**Recommendation**: Use config file for persistent settings, command-line flags for overrides.

---

## 🚀 Quick Start

**Minimal validator configuration**:

```bash
belizechain-node \
  --chain mainnet \
  --name "MyValidator" \
  --validator \
  --base-path /var/lib/belizechain
```

**Minimal RPC node configuration**:

```bash
belizechain-node \
  --chain mainnet \
  --name "MyRpcNode" \
  --rpc-external \
  --ws-external \
  --rpc-cors all \
  --base-path /var/lib/belizechain
```

---

## 📄 Configuration File

**Location**: `/etc/belizechain/config.toml`

### **Complete Configuration Template**

```toml
#######################
# Node Configuration  #
#######################

[node]
# Node name (shown in telemetry)
name = "MyBelizeNode"

# Chain specification
chain = "mainnet"  # Options: mainnet, testnet, devnet

# Base path for blockchain data
base_path = "/var/lib/belizechain"

# Node role
role = "full"  # Options: full, validator, light

#######################
# Network             #
#######################

[network]
# P2P listening addresses
listen_addresses = [
  "/ip4/0.0.0.0/tcp/30333",
  "/ip6/::/tcp/30333"
]

# Public addresses (advertised to peers)
# Leave empty for auto-detection
public_addresses = []

# Reserved peers (always connected)
reserved_peers = []

# Bootstrap nodes
bootnodes = [
  "/dns/bootnode1.belizechain.org/tcp/30333/p2p/12D3KooW...",
  "/dns/bootnode2.belizechain.org/tcp/30333/p2p/12D3KooW..."
]

# Maximum peer connections
max_peers = 50

# Minimum peer connections
min_peers = 5

# Peer discovery
enable_mdns = false  # Local network discovery (dev only)

#######################
# RPC Configuration   #
#######################

[rpc]
# HTTP RPC port
port = 9933

# WebSocket RPC port
ws_port = 9944

# Maximum connections
max_connections = 100

# Request timeout (seconds)
timeout = 30

# Enable external access (DANGEROUS without reverse proxy!)
external = false

# CORS (Cross-Origin Resource Sharing)
# Use ["*"] to allow all origins
cors = ["http://localhost:3000"]

# Rate limiting (requests per minute)
rate_limit = 100

# Methods to expose
# Options: Unsafe, Safe, All
methods = "Safe"

#######################
# Telemetry           #
#######################

[telemetry]
enabled = true

# Telemetry endpoints
endpoints = [
  { url = "wss://telemetry.polkadot.io/submit/", verbosity = 0 }
]

#######################
# Validator Settings  #
#######################

[validator]
# Enable validator mode
enabled = false

# Offchain worker
offchain_worker = false

#######################
# Database            #
#######################

[database]
# Database backend
# Options: rocksdb, paritydb
backend = "rocksdb"

# Cache size (MB)
cache_size = 1024

# WAL (Write-Ahead Logging)
enable_wal = true

#######################
# Pruning             #
#######################

[pruning]
# Pruning mode
# Options: archive, constrained
mode = "constrained"

# Blocks to keep (if constrained)
# Minimum: 256
keep_blocks = 256

#######################
# Logging             #
#######################

[logging]
# Log level
# Options: error, warn, info, debug, trace
level = "info"

# Module-specific log levels
modules = [
  "runtime=debug",
  "babe=trace"
]

# Log to file
log_file = "/var/log/belizechain/node.log"

# Rotate logs
rotate_logs = true
max_log_files = 5
max_log_size = "100MB"

#######################
# Prometheus          #
#######################

[prometheus]
enabled = true
port = 9615

# Expose to external (for Prometheus server)
external = false

#######################
# Execution           #
#######################

[execution]
# Execution strategy for syncing
syncing = "NativeElseWasm"

# Execution strategy for importing blocks
importing = "NativeElseWasm"

# Execution strategy for block production
block_construction = "Wasm"

# Execution strategy for offchain workers
offchain_worker = "NativeElseWasm"

# Execution strategy for other operations
other = "NativeElseWasm"

#######################
# State Cache         #
#######################

[state_cache]
# State cache size (MB)
size = 1024

# Shared cache between instances
shared = true
```

---

## 🎯 Configuration by Use Case

### **1. Validator Node**

```toml
[node]
name = "MyValidator"
chain = "mainnet"
base_path = "/var/lib/belizechain"
role = "validator"

[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/30333"]
max_peers = 50

[rpc]
# Keep RPC private for validators
external = false

[validator]
enabled = true
offchain_worker = true

[database]
cache_size = 2048  # Validators need more cache

[pruning]
mode = "constrained"
keep_blocks = 256

[prometheus]
enabled = true
external = false  # Expose to monitoring server only
```

**Command-line equivalent**:

```bash
belizechain-node \
  --chain mainnet \
  --name "MyValidator" \
  --validator \
  --base-path /var/lib/belizechain \
  --db-cache 2048 \
  --pruning 256 \
  --prometheus-external
```

---

### **2. RPC Node (Public API)**

```toml
[node]
name = "MyRpcNode"
chain = "mainnet"
base_path = "/var/lib/belizechain"
role = "full"

[network]
max_peers = 100  # More peers for better data availability

[rpc]
port = 9933
ws_port = 9944
max_connections = 500  # Handle many clients
external = true
cors = ["*"]  # Allow all origins (use reverse proxy!)
rate_limit = 100
methods = "Safe"  # Never expose Unsafe methods!

[database]
cache_size = 4096  # Large cache for fast queries

[pruning]
mode = "archive"  # Full historical data

[prometheus]
enabled = true
```

---

### **3. Archive Node (Full History)**

```toml
[node]
name = "MyArchiveNode"
chain = "mainnet"
base_path = "/var/lib/belizechain"

[pruning]
mode = "archive"  # Keep ALL blocks

[database]
backend = "rocksdb"
cache_size = 8192  # Very large cache
enable_wal = true

[rpc]
max_connections = 200
timeout = 60  # Longer timeout for historical queries
```

---

### **4. Development Node**

```toml
[node]
name = "DevNode"
chain = "devnet"
base_path = "/tmp/belizechain-dev"

[network]
listen_addresses = ["/ip4/127.0.0.1/tcp/30333"]
max_peers = 10
enable_mdns = true  # Auto-discover local peers

[rpc]
external = true
cors = ["*"]
methods = "Unsafe"  # Allow unsafe methods for dev

[logging]
level = "debug"  # Verbose logging

[pruning]
keep_blocks = 256  # Minimal history
```

---

## 🔐 Security Configuration

### **1. Firewall-Protected RPC**

```toml
[rpc]
# Only listen on localhost
port = 9933
ws_port = 9944
external = false

# Use reverse proxy (Nginx, Caddy) for external access
```

**Nginx reverse proxy** (`/etc/nginx/sites-available/belizechain`):

```nginx
upstream belizechain_rpc {
    server 127.0.0.1:9933;
}

server {
    listen 443 ssl http2;
    server_name rpc.mynode.com;

    ssl_certificate /etc/letsencrypt/live/mynode.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mynode.com/privkey.pem;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=rpc_limit:10m rate=10r/s;
    limit_req zone=rpc_limit burst=20;

    location / {
        proxy_pass http://belizechain_rpc;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

---

### **2. Validator Network Isolation**

```toml
[network]
# Only connect to trusted peers
reserved_peers = [
  "/ip4/10.0.1.2/tcp/30333/p2p/12D3KooW...",  # Sentry 1
  "/ip4/10.0.1.3/tcp/30333/p2p/12D3KooW..."   # Sentry 2
]

# Don't accept other peers
reserved_only = true

# Don't advertise public address
public_addresses = []
```

**Architecture**: Validator → Sentry nodes → Public network

---

## 📊 Performance Tuning

### **High-Performance Configuration**

```toml
[database]
backend = "rocksdb"
cache_size = 8192  # 8 GB cache

[state_cache]
size = 4096  # 4 GB state cache
shared = true

[execution]
syncing = "NativeElseWasm"  # Fastest for syncing
importing = "NativeElseWasm"
block_construction = "Wasm"

[network]
max_peers = 50  # Balance between resilience and bandwidth
```

**System tuning** (`/etc/sysctl.conf`):

```ini
# Increase file descriptor limits
fs.file-max = 2097152

# Network buffer sizes
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 67108864
net.ipv4.tcp_wmem = 4096 65536 67108864

# Connection handling
net.core.somaxconn = 8192
net.ipv4.tcp_max_syn_backlog = 8192
```

---

## 🔍 Logging Configuration

### **Structured Logging**

```toml
[logging]
level = "info"

# Module-specific levels
modules = [
  "afg=trace",                    # GRANDPA finality
  "babe=debug",                   # BABE consensus
  "runtime::governance=debug",    # Governance pallet
  "sc_network=warn",              # Network (reduce noise)
  "sync=info"                     # Sync progress
]

# JSON logging (for log aggregation)
json = true

# Include timestamps
timestamps = true
```

---

### **Log Rotation** (systemd journal)

```bash
# Edit /etc/systemd/journald.conf
sudo nano /etc/systemd/journald.conf
```

```ini
[Journal]
SystemMaxUse=5G
SystemMaxFileSize=200M
MaxRetentionSec=1month
```

```bash
sudo systemctl restart systemd-journald
```

---

## 🌐 Network Configuration

### **Custom Bootnodes**

```toml
[network]
# Use custom bootnodes (e.g., for private network)
bootnodes = [
  "/ip4/192.168.1.100/tcp/30333/p2p/12D3KooWYourNodeID1...",
  "/ip4/192.168.1.101/tcp/30333/p2p/12D3KooWYourNodeID2..."
]

# Don't use default bootnodes
no_default_bootnode = true
```

---

### **Port Configuration**

| Service | Default Port | Config |
|---------|--------------|--------|
| P2P | 30333 | `listen_addresses` |
| HTTP RPC | 9933 | `rpc.port` |
| WebSocket RPC | 9944 | `rpc.ws_port` |
| Prometheus | 9615 | `prometheus.port` |

**Change ports**:

```toml
[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/30334"]  # Changed from 30333

[rpc]
port = 8933  # Changed from 9933
ws_port = 8944  # Changed from 9944

[prometheus]
port = 8615  # Changed from 9615
```

---

## 📦 Database Configuration

### **RocksDB vs ParityDB**

| Feature | RocksDB | ParityDB |
|---------|---------|----------|
| **Maturity** | Stable | Experimental |
| **Performance** | Good | Excellent |
| **Storage efficiency** | Good | Better |
| **Memory usage** | Higher | Lower |
| **Recommendation** | Production | Testing |

```toml
[database]
backend = "rocksdb"  # Safe choice

# OR (experimental)
backend = "paritydb"  # Faster, but less tested
```

---

### **Database Cache Sizing**

**Rule of thumb**: `cache_size = 25% of RAM`

```toml
[database]
# 8 GB RAM → 2 GB cache
cache_size = 2048

# 16 GB RAM → 4 GB cache
cache_size = 4096

# 32 GB RAM → 8 GB cache
cache_size = 8192
```

---

## 🛠️ Advanced Configuration

### **Custom Chain Spec**

```bash
# Generate chain spec
belizechain-node build-spec --disable-default-bootnode --chain mainnet > custom-spec.json

# Edit custom-spec.json (change bootnodes, genesis, etc.)

# Convert to raw format
belizechain-node build-spec --disable-default-bootnode --chain custom-spec.json --raw > custom-spec-raw.json

# Use custom spec
belizechain-node --chain custom-spec-raw.json
```

---

### **Offchain Workers**

```toml
[validator]
offchain_worker = true  # Enable offchain workers

[execution]
offchain_worker = "NativeElseWasm"
```

**When needed**:
- Oracle merchant verification
- HTTP requests from pallets
- Offchain indexing

---

## ✅ Configuration Checklist

- [ ] Base path configured
- [ ] Network ports open in firewall
- [ ] RPC access configured (local vs external)
- [ ] Database cache sized appropriately
- [ ] Pruning mode set correctly
- [ ] Logging configured
- [ ] Prometheus enabled
- [ ] Telemetry enabled (optional)

---

## 🐛 Troubleshooting

### **Issue: Configuration not loading**

```bash
# Verify config file syntax
belizechain-node --help  # Should load without errors

# Check file permissions
ls -l /etc/belizechain/config.toml
# Should be readable: -rw-r--r--
```

---

### **Issue: RPC not accessible**

```bash
# Check if listening
sudo netstat -tlnp | grep 9933

# Check firewall
sudo ufw status

# Check RPC settings
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933
```

---

## 🚀 Next Steps

1. **[Validator Setup →](validator-setup.md)**  
   Configure as a validator

2. **[Monitoring →](monitoring.md)**  
   Set up monitoring and alerts

3. **[Maintenance →](maintenance.md)**  
   Learn ongoing maintenance

---

**Questions?** Check our [FAQ](../getting-started/faq.md) or join [Discord](https://discord.gg/belizechain)! ⚙️🇧🇿
