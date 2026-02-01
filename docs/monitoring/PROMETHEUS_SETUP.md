# 📊 BelizeChain Network Monitoring Setup

**Version**: 1.0.0  
**Updated**: November 4, 2025  
**Setup Time**: 45-60 minutes  

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Prometheus Setup](#prometheus-setup)
4. [Grafana Setup](#grafana-setup)
5. [Alert Manager](#alert-manager)
6. [Custom Dashboards](#custom-dashboards)
7. [Maintenance](#maintenance)

---

## Overview

Proper monitoring is CRITICAL for validator operations. This guide sets up a complete monitoring stack:

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization and dashboards
- **Alertmanager**: Alert routing (email, Slack, PagerDuty)
- **Node Exporter**: System metrics (CPU, RAM, disk)
- **BelizeChain Node**: Blockchain metrics (blocks, peers, finality)

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Monitoring Stack                       │
└─────────────────────────────────────────────────────────┘

┌────────────┐       ┌────────────┐       ┌────────────┐
│  BelizeChain│──────▶│ Prometheus │──────▶│  Grafana   │
│  Node      │ :9615 │            │ :9090 │            │ :3000
│            │       │  (Metrics  │       │ (Dashboards)│
└────────────┘       │   Storage) │       └────────────┘
                     └──────┬─────┘              │
┌────────────┐              │                    │
│   Node     │──────────────┘                    │
│  Exporter  │ :9100                             │
│  (System)  │                                   │
└────────────┘                                   │
                                                 │
┌────────────┐       ┌─────────────┐            │
│   Alert    │◀──────│    Alert    │◀───────────┘
│  Channels  │       │   Manager   │ :9093
│ (Email,Slack)       └─────────────┘
└────────────┘
```

---

## Prometheus Setup

### 1. Install Prometheus

```bash
# Create prometheus user
sudo useradd --no-create-home --shell /bin/false prometheus

# Download Prometheus
cd /tmp
wget https://github.com/prometheus/prometheus/releases/download/v2.47.0/prometheus-2.47.0.linux-amd64.tar.gz
tar xvf prometheus-2.47.0.linux-amd64.tar.gz
cd prometheus-2.47.0.linux-amd64

# Install binaries
sudo cp prometheus /usr/local/bin/
sudo cp promtool /usr/local/bin/
sudo chown prometheus:prometheus /usr/local/bin/prometheus
sudo chown prometheus:prometheus /usr/local/bin/promtool

# Create directories
sudo mkdir -p /etc/prometheus
sudo mkdir -p /var/lib/prometheus
sudo chown prometheus:prometheus /etc/prometheus
sudo chown prometheus:prometheus /var/lib/prometheus

# Install config files
sudo cp -r consoles /etc/prometheus
sudo cp -r console_libraries /etc/prometheus
sudo chown -R prometheus:prometheus /etc/prometheus/consoles
sudo chown -R prometheus:prometheus /etc/prometheus/console_libraries

# Cleanup
cd ~
rm -rf /tmp/prometheus*
```

### 2. Configure Prometheus

Create `/etc/prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'belizechain-validator'
    network: 'testnet'  # or 'mainnet'

# Alerting configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - localhost:9093

# Load alert rules
rule_files:
  - "/etc/prometheus/alerts/*.yml"

# Scrape configurations
scrape_configs:
  # BelizeChain Node Metrics
  - job_name: 'belizechain-node'
    static_configs:
      - targets: ['localhost:9615']
        labels:
          instance: 'validator-1'
          node_type: 'validator'

  # System Metrics (Node Exporter)
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
        labels:
          instance: 'validator-1'

  # Prometheus Self-Monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
```

Set permissions:
```bash
sudo chown prometheus:prometheus /etc/prometheus/prometheus.yml
```

### 3. Create Systemd Service

Create `/etc/systemd/system/prometheus.service`:

```ini
[Unit]
Description=Prometheus Monitoring System
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/prometheus \
    --config.file=/etc/prometheus/prometheus.yml \
    --storage.tsdb.path=/var/lib/prometheus/ \
    --web.console.templates=/etc/prometheus/consoles \
    --web.console.libraries=/etc/prometheus/console_libraries \
    --storage.tsdb.retention.time=30d \
    --web.enable-admin-api

Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 4. Start Prometheus

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service
sudo systemctl enable prometheus

# Start service
sudo systemctl start prometheus

# Check status
sudo systemctl status prometheus

# Verify metrics endpoint
curl http://localhost:9090/metrics

# Open web UI
# Navigate to: http://your-server-ip:9090
```

---

## Grafana Setup

### 1. Install Grafana

```bash
# Add Grafana APT repository
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"

# Add GPG key
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -

# Update and install
sudo apt-get update
sudo apt-get install -y grafana

# Enable service
sudo systemctl daemon-reload
sudo systemctl enable grafana-server
sudo systemctl start grafana-server

# Check status
sudo systemctl status grafana-server
```

### 2. Configure Grafana

```bash
# Access Grafana web UI
# Navigate to: http://your-server-ip:3000
# Default login: admin / admin
# Change password on first login

# Configure firewall (if needed)
sudo ufw allow 3000/tcp
```

### 3. Add Prometheus Data Source

1. Login to Grafana (http://your-server-ip:3000)
2. Click **Configuration** (gear icon) → **Data Sources**
3. Click **Add data source**
4. Select **Prometheus**
5. Configure:
   ```
   Name: BelizeChain Prometheus
   URL: http://localhost:9090
   Access: Server (default)
   ```
6. Click **Save & Test**
7. Should see: "Data source is working"

---

## Alert Manager

### 1. Install Alertmanager

```bash
# Create alertmanager user
sudo useradd --no-create-home --shell /bin/false alertmanager

# Download Alertmanager
cd /tmp
wget https://github.com/prometheus/alertmanager/releases/download/v0.26.0/alertmanager-0.26.0.linux-amd64.tar.gz
tar xvf alertmanager-0.26.0.linux-amd64.tar.gz
cd alertmanager-0.26.0.linux-amd64

# Install binaries
sudo cp alertmanager /usr/local/bin/
sudo cp amtool /usr/local/bin/
sudo chown alertmanager:alertmanager /usr/local/bin/alertmanager
sudo chown alertmanager:alertmanager /usr/local/bin/amtool

# Create directories
sudo mkdir -p /etc/alertmanager
sudo mkdir -p /var/lib/alertmanager
sudo chown alertmanager:alertmanager /etc/alertmanager
sudo chown alertmanager:alertmanager /var/lib/alertmanager

# Cleanup
cd ~
rm -rf /tmp/alertmanager*
```

### 2. Configure Alertmanager

Create `/etc/alertmanager/alertmanager.yml`:

```yaml
global:
  # Slack webhook (optional)
  slack_api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'

# Alert routing
route:
  receiver: 'default-receiver'
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  
  routes:
    # Critical alerts go to PagerDuty + email
    - match:
        severity: critical
      receiver: 'critical-alerts'
      repeat_interval: 15m
    
    # Warning alerts go to Slack + email
    - match:
        severity: warning
      receiver: 'warning-alerts'
      repeat_interval: 1h

# Alert receivers
receivers:
  # Default: Email only
  - name: 'default-receiver'
    email_configs:
      - to: 'validator@example.com'
        from: 'alertmanager@belizechain.org'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'your-email@gmail.com'
        auth_password: 'your-app-password'
        headers:
          Subject: 'BelizeChain Alert: {{ .GroupLabels.alertname }}'
  
  # Critical: PagerDuty + Email
  - name: 'critical-alerts'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
    email_configs:
      - to: 'validator@example.com,backup@example.com'
        from: 'alertmanager@belizechain.org'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'your-email@gmail.com'
        auth_password: 'your-app-password'
  
  # Warning: Slack + Email
  - name: 'warning-alerts'
    slack_configs:
      - channel: '#belizechain-alerts'
        title: 'Warning: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
    email_configs:
      - to: 'validator@example.com'
        from: 'alertmanager@belizechain.org'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'your-email@gmail.com'
        auth_password: 'your-app-password'

# Inhibition rules (suppress redundant alerts)
inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

Set permissions:
```bash
sudo chown alertmanager:alertmanager /etc/alertmanager/alertmanager.yml
```

### 3. Create Systemd Service

Create `/etc/systemd/system/alertmanager.service`:

```ini
[Unit]
Description=Alertmanager
Wants=network-online.target
After=network-online.target

[Service]
User=alertmanager
Group=alertmanager
Type=simple
ExecStart=/usr/local/bin/alertmanager \
    --config.file=/etc/alertmanager/alertmanager.yml \
    --storage.path=/var/lib/alertmanager/

Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 4. Start Alertmanager

```bash
sudo systemctl daemon-reload
sudo systemctl enable alertmanager
sudo systemctl start alertmanager
sudo systemctl status alertmanager

# Test alert (should receive email)
amtool alert add alertname=test severity=warning
```

---

## Custom Dashboards

### 1. BelizeChain Validator Dashboard

Save as `/etc/prometheus/dashboards/validator-dashboard.json`:

```json
{
  "dashboard": {
    "title": "BelizeChain Validator Metrics",
    "panels": [
      {
        "title": "Block Production Rate",
        "targets": [
          {
            "expr": "rate(substrate_block_height[5m])"
          }
        ]
      },
      {
        "title": "Connected Peers",
        "targets": [
          {
            "expr": "substrate_sub_libp2p_peers_count"
          }
        ]
      },
      {
        "title": "Finality Lag",
        "targets": [
          {
            "expr": "substrate_block_height{status=\"finalized\"} - substrate_block_height{status=\"sync_target\"}"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "targets": [
          {
            "expr": "process_resident_memory_bytes"
          }
        ]
      }
    ]
  }
}
```

Import to Grafana:
1. Navigate to **Dashboards** → **Import**
2. Upload `validator-dashboard.json`
3. Select **BelizeChain Prometheus** data source
4. Click **Import**

### 2. Pre-built Dashboard (Quick Start)

```bash
# Download BelizeChain validator dashboard
wget https://raw.githubusercontent.com/BelizeChain/monitoring/main/grafana/validator-dashboard.json

# Import via Grafana UI (Dashboards → Import → Upload JSON)
```

---

## Alert Rules

### Create Alert Rules

Create `/etc/prometheus/alerts/validator-alerts.yml`:

```yaml
groups:
  - name: validator_alerts
    interval: 30s
    rules:
      # Critical: Node is down
      - alert: ValidatorNodeDown
        expr: up{job="belizechain-node"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Validator node is down"
          description: "BelizeChain node {{ $labels.instance }} has been down for more than 2 minutes."
      
      # Critical: Low peer count
      - alert: LowPeerCount
        expr: substrate_sub_libp2p_peers_count < 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Low peer count"
          description: "Validator {{ $labels.instance }} has only {{ $value }} peers (threshold: 10)."
      
      # Critical: Finality stalled
      - alert: FinalityStalled
        expr: increase(substrate_block_height{status="finalized"}[10m]) == 0
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Finality stalled"
          description: "Block finalization has stalled for 10 minutes on {{ $labels.instance }}."
      
      # Warning: High memory usage
      - alert: HighMemoryUsage
        expr: (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) < 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is above 90% on {{ $labels.instance }}."
      
      # Warning: High disk usage
      - alert: HighDiskUsage
        expr: (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) < 0.2
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High disk usage"
          description: "Disk usage is above 80% on {{ $labels.instance }}."
      
      # Warning: Missed blocks
      - alert: MissedBlocks
        expr: increase(substrate_proposer_block_constructed_count[1h]) == 0
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Validator missing blocks"
          description: "Validator {{ $labels.instance }} hasn't produced blocks in the last hour."
```

Reload Prometheus:
```bash
sudo systemctl reload prometheus

# Verify alerts
curl http://localhost:9090/api/v1/rules | jq
```

---

## Maintenance

### Log Rotation

Create `/etc/logrotate.d/prometheus`:

```
/var/log/prometheus/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
```

### Data Retention

Prometheus automatically deletes old data based on retention policy (default: 15 days).

To change retention:
```bash
# Edit /etc/systemd/system/prometheus.service
# Add: --storage.tsdb.retention.time=30d

sudo systemctl daemon-reload
sudo systemctl restart prometheus
```

### Backup Configuration

```bash
# Backup Prometheus config
sudo tar czf /backup/prometheus-config-$(date +%Y%m%d).tar.gz /etc/prometheus/

# Backup Grafana dashboards
sudo tar czf /backup/grafana-dashboards-$(date +%Y%m%d).tar.gz /var/lib/grafana/
```

---

## Troubleshooting

### Prometheus not scraping metrics

```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets | jq

# Verify BelizeChain metrics endpoint
curl http://localhost:9615/metrics

# Check firewall
sudo ufw status
```

### Alerts not firing

```bash
# Check Alertmanager status
sudo systemctl status alertmanager

# View active alerts
amtool alert

# Test alert
amtool alert add alertname=test severity=critical
```

### Grafana not displaying data

1. Verify Prometheus data source connection
2. Check time range in dashboard (top right)
3. Verify query syntax in panel editor
4. Check Prometheus logs: `sudo journalctl -u prometheus -f`

---

## Next Steps

- ✅ **Setup disaster recovery**: [../operations/DISASTER_RECOVERY.md](../operations/DISASTER_RECOVERY.md)
- ✅ **Configure security**: [../security/VALIDATOR_SECURITY.md](../security/VALIDATOR_SECURITY.md)
- ✅ **Join validator community**: [Discord](https://discord.gg/belizechain)

---

## Support

- **Discord**: https://discord.gg/belizechain (#monitoring channel)
- **Forum**: https://forum.belizechain.org/c/operations
- **Email**: support@belizechain.org

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
