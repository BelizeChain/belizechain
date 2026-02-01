# 📊 Monitoring Guide

**Comprehensive monitoring and alerting for BelizeChain nodes**

---

## 📋 What to Monitor

**Essential metrics**:
- ✅ Node health (uptime, sync status, peers)
- ✅ Block production (for validators)
- ✅ System resources (CPU, memory, disk, network)
- ✅ Validator performance (missed blocks, slashing events)
- ✅ Network connectivity (peer count, bandwidth)

**Advanced metrics**:
- Transaction pool size
- Database performance
- Federated learning contributions (PoUW)
- Session rotation events

---

## ⚡ Quick Setup (Prometheus + Grafana)

**Time**: 30 minutes

### **Step 1: Install Prometheus** (10 minutes)

```bash
# Download Prometheus
VERSION="2.47.0"
wget https://github.com/prometheus/prometheus/releases/download/v${VERSION}/prometheus-${VERSION}.linux-amd64.tar.gz
tar -xzf prometheus-${VERSION}.linux-amd64.tar.gz
sudo mv prometheus-${VERSION}.linux-amd64 /opt/prometheus

# Create user
sudo useradd --no-create-home --shell /bin/false prometheus

# Create directories
sudo mkdir -p /etc/prometheus /var/lib/prometheus
sudo chown prometheus:prometheus /var/lib/prometheus

# Configure Prometheus
sudo nano /etc/prometheus/prometheus.yml
```

**prometheus.yml**:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'belizechain'
    static_configs:
      - targets: ['localhost:9615']  # Node metrics
        labels:
          instance: 'validator1'
  
  - job_name: 'node_exporter'
    static_configs:
      - targets: ['localhost:9100']  # System metrics
```

**Create systemd service** (`/etc/systemd/system/prometheus.service`):

```ini
[Unit]
Description=Prometheus
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/opt/prometheus/prometheus \
  --config.file=/etc/prometheus/prometheus.yml \
  --storage.tsdb.path=/var/lib/prometheus \
  --web.console.templates=/opt/prometheus/consoles \
  --web.console.libraries=/opt/prometheus/console_libraries

[Install]
WantedBy=multi-user.target
```

**Start Prometheus**:

```bash
sudo systemctl daemon-reload
sudo systemctl enable prometheus
sudo systemctl start prometheus

# Verify
curl http://localhost:9090
```

---

### **Step 2: Install Node Exporter** (5 minutes)

```bash
# Download Node Exporter
VERSION="1.6.1"
wget https://github.com/prometheus/node_exporter/releases/download/v${VERSION}/node_exporter-${VERSION}.linux-amd64.tar.gz
tar -xzf node_exporter-${VERSION}.linux-amd64.tar.gz
sudo mv node_exporter-${VERSION}.linux-amd64/node_exporter /usr/local/bin/

# Create systemd service
sudo nano /etc/systemd/system/node_exporter.service
```

```ini
[Unit]
Description=Node Exporter
After=network.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/node_exporter

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable node_exporter
sudo systemctl start node_exporter

# Verify
curl http://localhost:9100/metrics
```

---

### **Step 3: Install Grafana** (10 minutes)

```bash
# Add Grafana repository
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -

# Install Grafana
sudo apt-get update
sudo apt-get install grafana

# Start Grafana
sudo systemctl enable grafana-server
sudo systemctl start grafana-server

# Access Grafana
# http://YOUR_SERVER_IP:3000
# Default login: admin / admin
```

---

### **Step 4: Configure Grafana** (5 minutes)

1. **Add Prometheus data source**:
   - Settings → Data Sources → Add data source
   - Select "Prometheus"
   - URL: `http://localhost:9090`
   - Click "Save & Test"

2. **Import BelizeChain dashboard**:
   - Create → Import
   - Upload `grafana-dashboard.json` (see below)
   - Select Prometheus data source
   - Click "Import"

**Done!** You now have monitoring set up. 🎉

---

## 📈 BelizeChain Grafana Dashboard

**grafana-dashboard.json**:

```json
{
  "dashboard": {
    "title": "BelizeChain Validator Monitoring",
    "panels": [
      {
        "title": "Node Status",
        "targets": [
          {
            "expr": "up{job=\"belizechain\"}"
          }
        ]
      },
      {
        "title": "Peer Count",
        "targets": [
          {
            "expr": "substrate_sub_libp2p_peers_count"
          }
        ]
      },
      {
        "title": "Best Block",
        "targets": [
          {
            "expr": "substrate_block_height{status=\"best\"}"
          }
        ]
      },
      {
        "title": "Finalized Block",
        "targets": [
          {
            "expr": "substrate_block_height{status=\"finalized\"}"
          }
        ]
      },
      {
        "title": "CPU Usage",
        "targets": [
          {
            "expr": "100 - (avg by (instance) (irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "targets": [
          {
            "expr": "(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100"
          }
        ]
      },
      {
        "title": "Disk Usage",
        "targets": [
          {
            "expr": "(node_filesystem_size_bytes{mountpoint=\"/var/lib/belizechain\"} - node_filesystem_avail_bytes{mountpoint=\"/var/lib/belizechain\"}) / node_filesystem_size_bytes{mountpoint=\"/var/lib/belizechain\"} * 100"
          }
        ]
      },
      {
        "title": "Network I/O",
        "targets": [
          {
            "expr": "irate(node_network_receive_bytes_total{device=\"eth0\"}[5m])"
          },
          {
            "expr": "irate(node_network_transmit_bytes_total{device=\"eth0\"}[5m])"
          }
        ]
      }
    ]
  }
}
```

Save this as `/tmp/grafana-dashboard.json` and import in Grafana.

---

## 🔔 Alerting

### **Prometheus Alertmanager Setup**

**Step 1: Install Alertmanager**

```bash
VERSION="0.26.0"
wget https://github.com/prometheus/alertmanager/releases/download/v${VERSION}/alertmanager-${VERSION}.linux-amd64.tar.gz
tar -xzf alertmanager-${VERSION}.linux-amd64.tar.gz
sudo mv alertmanager-${VERSION}.linux-amd64 /opt/alertmanager

sudo mkdir -p /etc/alertmanager
sudo nano /etc/alertmanager/alertmanager.yml
```

---

**alertmanager.yml**:

```yaml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'instance']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'email-alerts'

receivers:
  - name: 'email-alerts'
    email_configs:
      - to: 'admin@example.com'
        from: 'alerts@belizechain.org'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'alerts@belizechain.org'
        auth_password: 'YOUR_PASSWORD'
  
  - name: 'slack-alerts'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#belizechain-alerts'
```

---

**Step 2: Configure alert rules**

Create `/etc/prometheus/alerts.yml`:

```yaml
groups:
  - name: belizechain_alerts
    interval: 30s
    rules:
      # Node is down
      - alert: NodeDown
        expr: up{job="belizechain"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "BelizeChain node is down"
          description: "Node {{ $labels.instance }} has been down for more than 1 minute"
      
      # Low peer count
      - alert: LowPeerCount
        expr: substrate_sub_libp2p_peers_count < 5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count"
          description: "Node {{ $labels.instance }} has {{ $value }} peers (threshold: 5)"
      
      # Node not syncing
      - alert: NodeNotSyncing
        expr: increase(substrate_block_height{status="best"}[5m]) == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Node not syncing"
          description: "Node {{ $labels.instance }} hasn't imported new blocks in 5 minutes"
      
      # High CPU usage
      - alert: HighCPUUsage
        expr: 100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 90
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage on {{ $labels.instance }} is {{ $value }}%"
      
      # High memory usage
      - alert: HighMemoryUsage
        expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100 > 90
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage on {{ $labels.instance }} is {{ $value }}%"
      
      # Low disk space
      - alert: LowDiskSpace
        expr: (node_filesystem_size_bytes{mountpoint="/var/lib/belizechain"} - node_filesystem_avail_bytes{mountpoint="/var/lib/belizechain"}) / node_filesystem_size_bytes{mountpoint="/var/lib/belizechain"} * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space"
          description: "Disk usage on {{ $labels.instance }} is {{ $value }}% (threshold: 80%)"
      
      # Validator missed blocks
      - alert: ValidatorMissedBlocks
        expr: increase(substrate_proposer_block_constructed_total[1h]) == 0
        for: 1h
        labels:
          severity: critical
        annotations:
          summary: "Validator missed blocks"
          description: "Validator {{ $labels.instance }} hasn't produced blocks in 1 hour"
```

---

**Step 3: Update Prometheus config**

Edit `/etc/prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']

rule_files:
  - "/etc/prometheus/alerts.yml"

scrape_configs:
  - job_name: 'belizechain'
    static_configs:
      - targets: ['localhost:9615']
  
  - job_name: 'node_exporter'
    static_configs:
      - targets: ['localhost:9100']
```

**Restart services**:

```bash
sudo systemctl restart prometheus
sudo systemctl restart alertmanager
```

---

## 📱 Mobile Notifications

### **Telegram Alerts**

**Step 1: Create Telegram bot**

1. Message [@BotFather](https://t.me/botfather) on Telegram
2. Send `/newbot` and follow prompts
3. Save bot token: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`

**Step 2: Get your chat ID**

1. Message your bot
2. Visit: `https://api.telegram.org/bot123456789:ABCdefGHIjklMNOpqrsTUVwxyz/getUpdates`
3. Find `"chat":{"id":YOUR_CHAT_ID}`

---

**Step 3: Configure Alertmanager**

Add to `/etc/alertmanager/alertmanager.yml`:

```yaml
receivers:
  - name: 'telegram-alerts'
    webhook_configs:
      - url: 'http://localhost:8080/telegram'
        send_resolved: true
```

**Step 4: Install telegram forwarder**

```bash
# Install telegram-send
pip install telegram-send

# Configure
telegram-send --configure

# Test
echo "Test alert" | telegram-send --stdin
```

---

## 🎯 Key Metrics Explained

### **Node Metrics** (Port 9615)

| Metric | Description | Normal Range |
|--------|-------------|--------------|
| `substrate_block_height{status="best"}` | Latest block number | Increasing |
| `substrate_block_height{status="finalized"}` | Latest finalized block | Increasing |
| `substrate_sub_libp2p_peers_count` | Connected peers | 20-50 |
| `substrate_ready_transactions_number` | Transactions in pool | 0-100 |
| `substrate_proposer_block_constructed_total` | Blocks produced (validators) | Increasing |

---

### **System Metrics** (Port 9100)

| Metric | Description | Normal Range |
|--------|-------------|--------------|
| `node_cpu_seconds_total` | CPU time | - |
| `node_memory_MemAvailable_bytes` | Available memory | > 2 GB |
| `node_filesystem_avail_bytes` | Available disk space | > 50 GB |
| `node_network_receive_bytes_total` | Network received | - |
| `node_network_transmit_bytes_total` | Network transmitted | - |

---

## 📊 Custom Queries

### **Query Examples in Prometheus**

**Average block time (last hour)**:

```promql
rate(substrate_block_height{status="best"}[1h]) * 60
```

**Finalization lag**:

```promql
substrate_block_height{status="best"} - substrate_block_height{status="finalized"}
```

**Memory usage %**:

```promql
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100
```

**Disk I/O rate**:

```promql
rate(node_disk_read_bytes_total[5m])
rate(node_disk_written_bytes_total[5m])
```

---

## 🔍 Log Aggregation

### **Loki + Promtail Setup**

**Step 1: Install Loki**

```bash
VERSION="2.9.0"
wget https://github.com/grafana/loki/releases/download/v${VERSION}/loki-linux-amd64.zip
unzip loki-linux-amd64.zip
sudo mv loki-linux-amd64 /usr/local/bin/loki

sudo mkdir /etc/loki
sudo nano /etc/loki/config.yml
```

**loki config.yml**:

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /tmp/loki/boltdb-shipper-active
    cache_location: /tmp/loki/boltdb-shipper-cache
  filesystem:
    directory: /tmp/loki/chunks
```

---

**Step 2: Install Promtail**

```bash
wget https://github.com/grafana/loki/releases/download/v${VERSION}/promtail-linux-amd64.zip
unzip promtail-linux-amd64.zip
sudo mv promtail-linux-amd64 /usr/local/bin/promtail

sudo nano /etc/loki/promtail-config.yml
```

**promtail-config.yml**:

```yaml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://localhost:3100/loki/api/v1/push

scrape_configs:
  - job_name: belizechain
    journal:
      max_age: 12h
      labels:
        job: belizechain
    relabel_configs:
      - source_labels: ['__journal__systemd_unit']
        target_label: 'unit'
```

**Start services**:

```bash
sudo systemctl start loki
sudo systemctl start promtail
```

---

**Step 3: Add Loki to Grafana**

1. Settings → Data Sources → Add data source
2. Select "Loki"
3. URL: `http://localhost:3100`
4. Save & Test

**Query logs in Grafana**:

```logql
{unit="belizechain.service"} |= "ERROR"
{unit="belizechain.service"} |= "block" | logfmt | block_number > 1000000
```

---

## 🛠️ Monitoring Tools

### **Command-Line Monitoring**

**Watch block height**:

```bash
watch -n 1 'curl -s http://localhost:9933 -H "Content-Type: application/json" -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getHeader\"}" | jq ".result.number" | xargs printf "%d\n"'
```

**Watch peer count**:

```bash
watch -n 5 'curl -s http://localhost:9933 -H "Content-Type: application/json" -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_health\"}" | jq ".result.peers"'
```

**Monitor system resources**:

```bash
# Install htop
sudo apt install htop

# Run
htop
```

---

## ✅ Monitoring Checklist

**Setup**:
- [ ] Prometheus installed and scraping metrics
- [ ] Node Exporter collecting system metrics
- [ ] Grafana dashboard configured
- [ ] Alert rules configured
- [ ] Alertmanager sending notifications
- [ ] Test alerts working

**Daily checks**:
- [ ] Review Grafana dashboard (5 min)
- [ ] Check for triggered alerts
- [ ] Verify peer count > 5
- [ ] Verify node is syncing

**Weekly checks**:
- [ ] Review performance trends
- [ ] Check disk space growth rate
- [ ] Verify backup monitoring

---

## 🚀 Next Steps

1. **[Maintenance →](maintenance.md)**  
   Learn ongoing maintenance tasks

2. **[Security →](security.md)**  
   Secure your monitoring setup

3. **[Validator Setup →](validator-setup.md)**  
   Monitor validator-specific metrics

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) for monitoring support! 📊🇧🇿
