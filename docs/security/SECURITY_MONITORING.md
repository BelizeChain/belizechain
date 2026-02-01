# BelizeChain Security Monitoring Infrastructure

**Version:** 1.0  
**Last Updated:** November 4, 2025  
**Status:** Implementation Ready  
**Budget:** $75k setup + $30k/year operational  

---

## 1. Executive Summary

This document defines BelizeChain's comprehensive security monitoring infrastructure for real-time threat detection and response. The system monitors blockchain pallets, cross-chain bridges, node infrastructure, and applications across 3 layers:

**Layer 1: Infrastructure Monitoring** (CPU, memory, network, disk)  
**Layer 2: Blockchain Monitoring** (transactions, balances, governance, consensus)  
**Layer 3: Application Monitoring** (APIs, UIs, smart contracts)

**Detection Targets:**
- Treasury draining (anomalous withdrawals)
- Bridge exploits (double-spending, validator compromise)
- Consensus attacks (51%, long-range, finality stalls)
- DDoS attacks (high request rates, resource exhaustion)
- Governance manipulation (vote buying, Sybil attacks)
- Smart contract exploits (reentrancy, overflow, access control)

**Architecture:**
```
┌─────────────────────────────────────────────────────────────────┐
│                     Detection Layer                             │
│  Prometheus (metrics) + Loki (logs) + Substrate Telemetry      │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    Analysis & Alerting Layer                    │
│  Grafana Dashboards + Alert Manager + Custom Rules Engine      │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    Response & Automation Layer                  │
│  PagerDuty (alerting) + Incident Bot (Slack) + Auto-Remediation│
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Infrastructure Monitoring (Layer 1)

### 2.1 Prometheus Metrics Collection

**Deployment:** Prometheus server (2 instances, HA mode)  
**Retention:** 30 days (metrics), 90 days (alerts)  
**Scrape Interval:** 15 seconds (real-time)  

**Monitored Targets:**

**Blockchain Nodes (5 validators + 3 RPC + 1 archive):**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'substrate-nodes'
    scrape_interval: 15s
    static_configs:
      - targets:
          - 'validator1.belizechain.org:9615'
          - 'validator2.belizechain.org:9615'
          - 'validator3.belizechain.org:9615'
          - 'validator4.belizechain.org:9615'
          - 'validator5.belizechain.org:9615'
          - 'rpc1.belizechain.org:9615'
          - 'rpc2.belizechain.org:9615'
          - 'rpc3.belizechain.org:9615'
          - 'archive.belizechain.org:9615'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
```

**Key Metrics:**

**System Metrics:**
- `node_cpu_seconds_total` - CPU usage per core
- `node_memory_MemAvailable_bytes` - Available RAM
- `node_disk_read_bytes_total` - Disk I/O read
- `node_disk_write_bytes_total` - Disk I/O write
- `node_network_receive_bytes_total` - Network inbound
- `node_network_transmit_bytes_total` - Network outbound

**Substrate Metrics:**
- `substrate_block_height` - Current block number
- `substrate_finality_lag_blocks` - Finality lag (blocks behind)
- `substrate_peers_count` - Connected peers
- `substrate_ready_transactions_number` - Transaction pool size
- `substrate_cpu_usage_percentage` - CPU usage (process)
- `substrate_memory_usage_bytes` - Memory usage (process)

**Alert Rules:**

```yaml
# alerts/infrastructure.yml
groups:
  - name: infrastructure
    interval: 30s
    rules:
      # High CPU usage
      - alert: HighCPUUsage
        expr: (100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is {{ $value }}% (threshold: 80%)"
      
      # Low memory available
      - alert: LowMemory
        expr: node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes < 0.15
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Low memory on {{ $labels.instance }}"
          description: "Available memory is {{ $value | humanizePercentage }} (threshold: 15%)"
      
      # Disk space low
      - alert: DiskSpaceLow
        expr: (node_filesystem_avail_bytes / node_filesystem_size_bytes) < 0.15
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space on {{ $labels.instance }}"
          description: "Available disk space is {{ $value | humanizePercentage }} (threshold: 15%)"
      
      # Node down
      - alert: NodeDown
        expr: up{job="substrate-nodes"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Node {{ $labels.instance }} is down"
          description: "Substrate node has been unreachable for 1 minute"
```

---

### 2.2 Loki Log Aggregation

**Deployment:** Loki server + Promtail (log shipper on all nodes)  
**Retention:** 14 days (all logs), 90 days (security events)  
**Storage:** S3-compatible (MinIO, 500GB allocated)  

**Log Sources:**

```yaml
# promtail-config.yml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: https://loki.internal.belizechain.org/loki/api/v1/push

scrape_configs:
  # Substrate node logs
  - job_name: substrate
    static_configs:
      - targets:
          - localhost
        labels:
          job: substrate
          __path__: /var/log/substrate/*.log
    pipeline_stages:
      # Extract severity level
      - regex:
          expression: '^(?P<timestamp>\S+) (?P<level>\w+) (?P<message>.*)$'
      - labels:
          level:
      # Extract transaction hash
      - regex:
          expression: 'extrinsic=(?P<extrinsic>\w+)'
      - labels:
          extrinsic:
  
  # NGINX access logs (RPC traffic)
  - job_name: nginx
    static_configs:
      - targets:
          - localhost
        labels:
          job: nginx
          __path__: /var/log/nginx/access.log
    pipeline_stages:
      - regex:
          expression: '^(?P<remote_ip>\S+) .* "(?P<method>\S+) (?P<path>\S+) .*" (?P<status>\d+) .*$'
      - labels:
          method:
          status:
      # Flag suspicious requests
      - match:
          selector: '{status="403"}'
          stages:
            - labels:
                suspicious: "true"
```

**Log Queries (LogQL):**

```logql
# Failed authentication attempts
{job="substrate"} |= "authentication failed" | json | line_format "{{.account}} failed login from {{.ip}}"

# Large transactions (>100K DALLA)
{job="substrate"} |= "Transfer" | json | amount > 100000

# Governance proposals submitted
{job="substrate"} |= "Proposed" | json | line_format "Proposal {{.proposal_id}} by {{.proposer}}"

# Bridge events
{job="substrate"} |= "BridgeTransfer" | json | line_format "Bridge {{.bridge}}: {{.amount}} {{.asset}}"

# Error rate spike (>10 errors/min)
rate({job="substrate", level="ERROR"}[1m]) > 10
```

---

### 2.3 Node Exporter Metrics

**Deployment:** Node Exporter on all servers (Kubernetes DaemonSet)  

```yaml
# k8s/node-exporter-daemonset.yml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: node-exporter
  namespace: monitoring
spec:
  selector:
    matchLabels:
      app: node-exporter
  template:
    metadata:
      labels:
        app: node-exporter
    spec:
      hostNetwork: true
      hostPID: true
      containers:
      - name: node-exporter
        image: prom/node-exporter:v1.6.1
        args:
          - '--path.procfs=/host/proc'
          - '--path.sysfs=/host/sys'
          - '--path.rootfs=/host/root'
          - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
        ports:
          - containerPort: 9100
            name: metrics
        volumeMounts:
          - name: proc
            mountPath: /host/proc
            readOnly: true
          - name: sys
            mountPath: /host/sys
            readOnly: true
          - name: root
            mountPath: /host/root
            readOnly: true
      volumes:
        - name: proc
          hostPath:
            path: /proc
        - name: sys
          hostPath:
            path: /sys
        - name: root
          hostPath:
            path: /
```

---

## 3. Blockchain Monitoring (Layer 2)

### 3.1 Custom Blockchain Metrics

**Implementation:** Custom Prometheus exporter (Rust)  

**File:** `node/src/monitoring/blockchain_exporter.rs`

```rust
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Registry,
};
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

pub struct BlockchainMetrics {
    // Block production
    pub blocks_produced: IntCounter,
    pub block_time: Histogram,
    pub finality_lag: IntGauge,
    
    // Transactions
    pub transactions_total: IntCounter,
    pub transactions_failed: IntCounter,
    pub transaction_fee_avg: Gauge,
    
    // Treasury
    pub treasury_balance: Gauge,
    pub treasury_withdrawals: IntCounter,
    pub treasury_withdrawal_amount: Gauge,
    
    // Staking
    pub staking_total_staked: Gauge,
    pub staking_validator_count: IntGauge,
    pub staking_slash_count: IntCounter,
    
    // Governance
    pub governance_proposals_active: IntGauge,
    pub governance_votes_cast: IntCounter,
    pub governance_proposals_passed: IntCounter,
    
    // Bridge
    pub bridge_deposits: IntCounter,
    pub bridge_withdrawals: IntCounter,
    pub bridge_volume: Gauge,
    
    // Oracle
    pub oracle_price_updates: IntCounter,
    pub oracle_staleness_seconds: IntGauge,
}

impl BlockchainMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            blocks_produced: IntCounter::new(
                "substrate_blocks_produced_total",
                "Total blocks produced"
            ).unwrap(),
            
            block_time: Histogram::with_opts(
                HistogramOpts::new("substrate_block_time_seconds", "Block production time")
                    .buckets(vec![1.0, 3.0, 6.0, 12.0, 30.0, 60.0])
            ).unwrap(),
            
            finality_lag: IntGauge::new(
                "substrate_finality_lag_blocks",
                "Blocks behind finality"
            ).unwrap(),
            
            treasury_balance: Gauge::new(
                "belizechain_treasury_balance",
                "Treasury balance in DALLA"
            ).unwrap(),
            
            treasury_withdrawals: IntCounter::new(
                "belizechain_treasury_withdrawals_total",
                "Total treasury withdrawals"
            ).unwrap(),
            
            // ... register all metrics with registry ...
        }
    }
    
    // Update metrics from blockchain state
    pub fn update<Block: BlockT>(&self, client: &Arc<Client<Block>>) {
        // Query treasury balance
        let treasury_balance = client.runtime_api()
            .treasury_balance(client.info().best_hash)
            .unwrap_or(0);
        self.treasury_balance.set(treasury_balance as f64 / 1_000_000_000_000.0); // Convert to DALLA
        
        // Query finality lag
        let best_block = client.info().best_number;
        let finalized_block = client.info().finalized_number;
        self.finality_lag.set((best_block - finalized_block) as i64);
        
        // ... update other metrics ...
    }
}
```

**Prometheus Queries:**

```promql
# Treasury balance (DALLA)
belizechain_treasury_balance

# Treasury withdrawal rate (withdrawals/minute)
rate(belizechain_treasury_withdrawals_total[1m])

# Large withdrawal alert (>100K DALLA in single tx)
belizechain_treasury_withdrawal_amount > 100000

# Finality lag (blocks)
substrate_finality_lag_blocks

# Transaction success rate
(rate(substrate_transactions_total[5m]) - rate(substrate_transactions_failed[5m])) / rate(substrate_transactions_total[5m])

# Validator count
belizechain_staking_validator_count

# Governance participation rate
rate(belizechain_governance_votes_cast[1h]) / belizechain_governance_proposals_active

# Bridge volume (last 24 hours)
increase(belizechain_bridge_volume[24h])

# Oracle staleness (seconds since last update)
belizechain_oracle_staleness_seconds
```

---

### 3.2 Transaction Monitoring

**Goal:** Detect suspicious transaction patterns in real-time

**Monitored Patterns:**

**1. Large Transfers (Whale Watching):**
```sql
-- Query SubQuery GraphQL API
query LargeTransfers {
  transfers(
    filter: {
      amount: { greaterThan: "100000000000000000" }  # >100K DALLA
      timestamp: { greaterThan: "2025-11-04T00:00:00Z" }
    }
    orderBy: TIMESTAMP_DESC
  ) {
    nodes {
      id
      from
      to
      amount
      timestamp
      extrinsicHash
    }
  }
}
```

**Alert Logic:**
- Single transfer >100K DALLA: Warning (log)
- Single transfer >1M DALLA: Critical (alert + freeze if to unknown address)
- Multiple transfers >50K DALLA in 1 hour from same account: Investigate
- Transfer to known exploit address (blacklist): Block + alert

**2. Rapid Transaction Sequences:**
```python
# transaction_analyzer.py
import asyncio
from collections import defaultdict
from datetime import datetime, timedelta

class TransactionAnalyzer:
    def __init__(self):
        self.account_tx_count = defaultdict(list)
    
    async def analyze_transaction(self, tx):
        account = tx['from']
        timestamp = datetime.fromisoformat(tx['timestamp'])
        
        # Add to account history
        self.account_tx_count[account].append(timestamp)
        
        # Remove old transactions (>5 minutes)
        cutoff = timestamp - timedelta(minutes=5)
        self.account_tx_count[account] = [
            ts for ts in self.account_tx_count[account]
            if ts > cutoff
        ]
        
        # Check for rapid sequences
        tx_count = len(self.account_tx_count[account])
        
        if tx_count > 100:
            # >100 tx in 5 minutes = possible attack
            await self.alert_rapid_sequence(account, tx_count)
        elif tx_count > 50:
            # >50 tx in 5 minutes = suspicious
            await self.log_suspicious_activity(account, tx_count)
    
    async def alert_rapid_sequence(self, account, count):
        # Send PagerDuty alert
        alert = {
            'severity': 'high',
            'title': f'Rapid transaction sequence detected',
            'details': f'Account {account} sent {count} transactions in 5 minutes',
            'account': account,
            'transaction_count': count
        }
        await self.pagerduty.trigger(alert)
```

**3. Unusual Time Patterns:**
- Transactions at unusual hours (2-6 AM local time) from government accounts: Investigate
- First transaction ever from dormant account (>6 months): Verify (possible compromise)
- Transaction immediately after governance vote: Possible insider trading (front-running)

---

### 3.3 Balance Monitoring

**Treasury Balance Anomaly Detection:**

```python
# treasury_monitor.py
import numpy as np
from prometheus_client import Gauge, Counter
from scipy import stats

class TreasuryMonitor:
    def __init__(self):
        self.balance_history = []
        self.balance_gauge = Gauge('treasury_balance_anomaly_score', 
                                   'Treasury balance anomaly score (z-score)')
        self.alert_counter = Counter('treasury_alerts_total', 
                                      'Total treasury alerts triggered')
    
    def update_balance(self, current_balance):
        # Add to history
        self.balance_history.append(current_balance)
        
        # Keep last 1000 data points (rolling window)
        if len(self.balance_history) > 1000:
            self.balance_history.pop(0)
        
        # Calculate z-score (how many std deviations from mean)
        if len(self.balance_history) > 30:  # Need enough data
            mean = np.mean(self.balance_history)
            std = np.std(self.balance_history)
            z_score = (current_balance - mean) / std if std > 0 else 0
            
            # Update metric
            self.balance_gauge.set(abs(z_score))
            
            # Alert if significant deviation
            if z_score < -3:  # 3+ std deviations below mean
                self.alert_treasury_drop(current_balance, mean, z_score)
    
    def alert_treasury_drop(self, current, mean, z_score):
        loss = mean - current
        self.alert_counter.inc()
        
        # Critical alert
        alert = {
            'severity': 'critical',
            'title': 'Treasury balance dropped significantly',
            'details': f'Current: {current} DALLA, Mean: {mean} DALLA, Loss: {loss} DALLA',
            'z_score': z_score,
            'recommended_action': 'Review recent proposals and withdrawals'
        }
        self.send_alert(alert)
```

**Bridge Reserve Monitoring:**

```python
# bridge_monitor.py
class BridgeMonitor:
    def __init__(self):
        self.reserves = {
            'ethereum': {'usdc': 0, 'usdt': 0, 'dai': 0},
            'polkadot': {'dot': 0, 'aca': 0}
        }
    
    async def monitor_reserves(self):
        while True:
            # Query both chains
            belizechain_reserves = await self.query_belizechain_reserves()
            ethereum_reserves = await self.query_ethereum_reserves()
            polkadot_reserves = await self.query_polkadot_reserves()
            
            # Check consistency
            for bridge, assets in belizechain_reserves.items():
                for asset, balance in assets.items():
                    if bridge == 'ethereum':
                        actual_balance = ethereum_reserves.get(asset, 0)
                    else:
                        actual_balance = polkadot_reserves.get(asset, 0)
                    
                    # Calculate discrepancy
                    discrepancy = abs(balance - actual_balance)
                    discrepancy_pct = (discrepancy / balance * 100) if balance > 0 else 0
                    
                    # Alert if >1% discrepancy (possible double-spend)
                    if discrepancy_pct > 1:
                        await self.alert_bridge_discrepancy(
                            bridge, asset, balance, actual_balance, discrepancy_pct
                        )
            
            await asyncio.sleep(60)  # Check every minute
```

---

### 3.4 Governance Monitoring

**Proposal Analysis:**

```python
# governance_monitor.py
class GovernanceMonitor:
    def __init__(self):
        self.proposal_db = {}
        self.suspicious_patterns = [
            'transfer_all_treasury',  # Drain treasury
            'sudo_call',  # Elevated privileges
            'set_code',  # Runtime upgrade (unsigned)
            'emergency_pause',  # Service disruption
        ]
    
    async def analyze_proposal(self, proposal):
        proposal_id = proposal['id']
        call_data = proposal['call']
        proposer = proposal['proposer']
        
        # Check for suspicious calls
        for pattern in self.suspicious_patterns:
            if pattern in call_data:
                await self.alert_suspicious_proposal(
                    proposal_id, proposer, pattern, call_data
                )
        
        # Check proposer reputation
        proposer_history = await self.get_proposer_history(proposer)
        if proposer_history['proposals_submitted'] == 0:
            # First proposal ever = higher scrutiny
            await self.flag_new_proposer(proposal_id, proposer)
        
        # Check for vote buying (unusual voting patterns)
        await self.monitor_votes(proposal_id)
    
    async def monitor_votes(self, proposal_id):
        votes = await self.get_votes(proposal_id)
        
        # Detect Sybil attack (many votes from similar accounts)
        account_similarities = self.calculate_account_similarities(votes)
        if account_similarities > 0.8:  # 80% similar accounts
            await self.alert_possible_sybil(proposal_id, votes)
        
        # Detect coordinated voting (many votes at same timestamp)
        vote_times = [v['timestamp'] for v in votes]
        time_clustering = self.calculate_time_clustering(vote_times)
        if time_clustering > 0.9:  # 90% of votes within 1 minute
            await self.alert_coordinated_voting(proposal_id, votes)
```

**JaguarMode Abuse Detection:**

```python
# jaguar_mode_monitor.py
class JaguarModeMonitor:
    def __init__(self):
        self.jaguar_activations = []
        self.max_activations_per_month = 5  # Reasonable limit
    
    async def monitor_jaguar_mode(self):
        # Listen for JaguarMode activation events
        async for event in self.subscribe_events('JaguarModeActivated'):
            activator = event['account']
            reason = event['reason']
            timestamp = event['timestamp']
            
            # Log activation
            self.jaguar_activations.append({
                'activator': activator,
                'reason': reason,
                'timestamp': timestamp
            })
            
            # Check activation frequency
            recent_activations = [
                a for a in self.jaguar_activations
                if a['timestamp'] > datetime.now() - timedelta(days=30)
            ]
            
            if len(recent_activations) > self.max_activations_per_month:
                await self.alert_excessive_jaguar_activations(recent_activations)
            
            # Alert stakeholders (always notify for JaguarMode)
            await self.notify_jaguar_activation(activator, reason)
```

---

## 4. Application Monitoring (Layer 3)

### 4.1 API Monitoring

**Monitored APIs:**
- RPC API (WebSocket + HTTP)
- GraphQL API (SubQuery)
- REST API (custom endpoints)

**Metrics:**

```yaml
# API metrics (OpenMetrics format)
# HTTP request duration (histogram)
http_request_duration_seconds{method="POST",endpoint="/rpc",status="200"} 0.05

# HTTP request rate (counter)
http_requests_total{method="POST",endpoint="/rpc",status="200"} 1543

# HTTP error rate (counter)
http_requests_total{method="POST",endpoint="/rpc",status="500"} 12

# WebSocket connections (gauge)
websocket_connections_active{endpoint="/ws"} 342

# Rate limit violations (counter)
http_rate_limit_exceeded_total{endpoint="/rpc"} 87
```

**Alert Rules:**

```yaml
# alerts/api.yml
groups:
  - name: api
    interval: 30s
    rules:
      # High error rate
      - alert: HighAPIErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High API error rate on {{ $labels.endpoint }}"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"
      
      # Slow API responses
      - alert: SlowAPIResponses
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow API responses on {{ $labels.endpoint }}"
          description: "95th percentile latency is {{ $value }}s (threshold: 2s)"
      
      # Rate limit abuse
      - alert: RateLimitAbuse
        expr: rate(http_rate_limit_exceeded_total[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Rate limit abuse detected"
          description: "{{ $value }} rate limit violations per second"
```

---

### 4.2 UI Monitoring (2 Portals: Maya Wallet + Blue Hole Portal)

**Monitored Portals:**
1. Maya Wallet (citizen portal)
2. Blue Hole Portal (government dashboard)
3. Winik Governance (district democracy)
4. Pek Business (merchant POS)
5. Gúbida Validator (validator dashboard)
6. Kijka Explorer (public blockchain explorer)

**Real User Monitoring (RUM):**

```typescript
// ui/shared/monitoring/rum.ts
import { datadogRum } from '@datadog/browser-rum';

datadogRum.init({
  applicationId: 'belizechain-ui',
  clientToken: process.env.DATADOG_CLIENT_TOKEN,
  site: 'datadoghq.com',
  service: 'maya-wallet',  // Different for each portal
  env: 'production',
  version: '1.0.0',
  sessionSampleRate: 100,  // 100% of sessions tracked
  sessionReplaySampleRate: 20,  // 20% replay recording
  trackUserInteractions: true,
  trackResources: true,
  trackLongTasks: true,
  defaultPrivacyLevel: 'mask-user-input',  // Mask sensitive input
});

// Track custom events
export function trackWalletOperation(operation: string, success: boolean, duration: number) {
  datadogRum.addAction('wallet_operation', {
    operation,
    success,
    duration_ms: duration,
  });
}

// Track errors
window.addEventListener('error', (event) => {
  datadogRum.addError(event.error, {
    context: 'global_error_handler',
  });
});
```

**Key Metrics:**
- Page load time (target: <2 seconds)
- Time to interactive (target: <3 seconds)
- Error rate (target: <0.1%)
- Session duration (engagement metric)
- Transaction success rate (wallet operations)

---

### 4.3 Smart Contract Monitoring

**Monitored Contracts (ink! WASM):**
1. PSP22 Token (ERC-20 equivalent)
2. AMM Pool (DEX)
3. Lending Protocol
4. Ethereum Bridge
5. Polkadot XCM Bridge

**Event Monitoring:**

```rust
// contracts/psp22/lib.rs
#[ink(event)]
pub struct Transfer {
    #[ink(topic)]
    from: Option<AccountId>,
    #[ink(topic)]
    to: Option<AccountId>,
    value: Balance,
}

// Emit events for monitoring
Self::env().emit_event(Transfer {
    from: Some(caller),
    to: Some(to),
    value: amount,
});
```

**Off-Chain Monitor:**

```python
# contract_monitor.py
from substrateinterface import SubstrateInterface
import asyncio

class ContractMonitor:
    def __init__(self, substrate: SubstrateInterface):
        self.substrate = substrate
        self.contract_addresses = {
            'psp22': '5GContractPSP22...',
            'amm': '5GContractAMM...',
            'lending': '5GContractLending...',
        }
    
    async def monitor_events(self):
        # Subscribe to contract events
        subscription = self.substrate.subscribe_event('Contracts', 'ContractEmitted')
        
        async for event in subscription:
            contract_address = event['contract']
            event_data = event['data']
            
            # Identify contract
            contract_name = self.identify_contract(contract_address)
            if not contract_name:
                continue
            
            # Parse event (ABI decoding)
            decoded_event = self.decode_event(contract_name, event_data)
            
            # Check for anomalies
            await self.check_anomalies(contract_name, decoded_event)
    
    async def check_anomalies(self, contract, event):
        if contract == 'psp22':
            if event['type'] == 'Transfer':
                amount = event['value']
                if amount > 1_000_000 * 10**12:  # >1M tokens
                    await self.alert_large_transfer(contract, event)
        
        elif contract == 'amm':
            if event['type'] == 'Swap':
                price_impact = event['price_impact']
                if price_impact > 0.05:  # >5% price impact
                    await self.alert_high_slippage(contract, event)
        
        elif contract == 'lending':
            if event['type'] == 'Liquidation':
                # All liquidations are interesting
                await self.log_liquidation(contract, event)
```

---

## 5. Threat Intelligence

### 5.1 Blockchain Security Feeds

**Integrated Feeds:**

**1. Chainalysis KYT (Know Your Transaction):**
- Real-time address screening
- Identify high-risk addresses (exchanges, mixers, sanctions)
- Alert on interactions with flagged addresses

```python
# chainalysis_integration.py
import httpx

class ChainalysisKYT:
    def __init__(self, api_key: str):
        self.api_key = api_key
        self.base_url = 'https://api.chainalysis.com/api/kyt/v2'
    
    async def screen_address(self, address: str, asset: str = 'DALLA'):
        # Check address against Chainalysis database
        response = await httpx.post(
            f'{self.base_url}/users/{address}/transfers',
            headers={'Token': self.api_key},
            json={
                'asset': asset,
                'network': 'BELIZECHAIN',
                'transferDirection': 'sent',
            }
        )
        
        data = response.json()
        risk_score = data['alerts'][0]['exposureScore'] if data['alerts'] else 0
        
        if risk_score > 0.7:  # High risk (0-1 scale)
            await self.alert_high_risk_address(address, risk_score, data)
        
        return {
            'address': address,
            'risk_score': risk_score,
            'category': data.get('category'),  # e.g., 'exchange', 'mixer', 'darknet'
            'alerts': data.get('alerts'),
        }
```

**2. Elliptic Lens:**
- Cross-chain risk assessment
- Identify stolen funds (bridge exploits, hacks)
- Compliance screening (OFAC, sanctions lists)

**3. OpenZeppelin Defender Sentinel:**
- Smart contract monitoring (events, function calls)
- Automated alerts (Slack, Telegram, webhook)
- Transaction simulation (before execution)

---

### 5.2 CVE Monitoring

**Monitored Sources:**
- Substrate CVEs (GitHub Security Advisories)
- Polkadot CVEs (Web3 Foundation announcements)
- Rust CVEs (RustSec Advisory Database)
- Dependencies (cargo-audit)

**Automated Scanning:**

```bash
#!/bin/bash
# scripts/security_scan.sh

# Run cargo audit (check for known vulnerabilities)
cargo audit

# Check for outdated dependencies
cargo outdated

# Run Clippy (lint for suspicious patterns)
cargo clippy --all-targets --all-features -- -D warnings

# Check for supply chain attacks (cargo-vet)
cargo vet

# Scan Docker images (Trivy)
trivy image belizechain-node:latest

# Scan Kubernetes manifests (Kubesec)
kubesec scan k8s/deployment.yml
```

**Scheduled:** Daily (CI/CD pipeline) + On-demand (pre-release)

---

## 6. Automated Response

### 6.1 Auto-Remediation Rules

**Safe Automated Actions:**

**1. Rate Limiting (DDoS Mitigation):**
```python
# auto_remediation/rate_limiter.py
class AutoRateLimiter:
    async def on_high_request_rate(self, ip: str, rate: int):
        # If request rate >1000 req/s from single IP
        if rate > 1000:
            # Temporarily block IP (15 minutes)
            await self.firewall.block_ip(ip, duration=900)
            await self.log_action(f'Auto-blocked {ip} for 15 minutes (rate: {rate} req/s)')
            
            # Alert team
            await self.pagerduty.trigger({
                'title': f'Auto-blocked IP {ip} due to high request rate',
                'severity': 'warning',
                'details': f'Rate: {rate} req/s (threshold: 1000 req/s)',
            })
```

**2. Account Freezing (Suspicious Activity):**
```python
# auto_remediation/account_freeze.py
class AutoAccountFreezer:
    async def on_suspicious_transaction(self, account: str, tx: dict):
        # Criteria for auto-freeze:
        # - Transfer to known exploit address
        # - Multiple failed authentication attempts (>10 in 1 minute)
        # - Large transfer from newly created account (<24 hours old)
        
        if self.should_freeze(account, tx):
            # Freeze account (requires multi-sig approval to unfreeze)
            await self.blockchain.freeze_account(account)
            
            # Alert security team (manual review required)
            await self.pagerduty.trigger({
                'title': f'Auto-froze account {account} due to suspicious activity',
                'severity': 'high',
                'details': tx,
                'action_required': 'Review and approve/reject freeze',
            })
```

**3. Service Degradation (Graceful Degradation):**
```python
# auto_remediation/service_degradation.py
class AutoServiceDegrader:
    async def on_high_load(self, service: str, cpu: float):
        # If CPU >90% for 5 minutes
        if cpu > 0.9:
            # Reduce non-critical features
            if service == 'rpc':
                # Disable expensive RPC methods (state queries)
                await self.rpc.disable_methods([
                    'state_getKeys',
                    'state_getPairs',
                    'state_queryStorage',
                ])
                await self.log_action(f'Disabled expensive RPC methods (CPU: {cpu}%)')
            
            elif service == 'graphql':
                # Reduce query complexity limit
                await self.graphql.set_complexity_limit(100)  # From 1000 to 100
                await self.log_action(f'Reduced GraphQL complexity limit (CPU: {cpu}%)')
```

---

### 6.2 Incident Bot (Slack Integration)

**Features:**
- Automated incident channel creation
- Status updates every 15 minutes (SEV-1)
- Action tracking (who's working on what)
- Timeline generation

```python
# incident_bot.py
from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError

class IncidentBot:
    def __init__(self, slack_token: str):
        self.slack = WebClient(token=slack_token)
        self.incidents = {}
    
    async def create_incident(self, incident_id: str, severity: str, title: str):
        # Create dedicated channel
        channel_name = f'incident-{incident_id}'
        response = self.slack.conversations_create(
            name=channel_name,
            is_private=False
        )
        channel_id = response['channel']['id']
        
        # Post initial message
        self.slack.chat_postMessage(
            channel=channel_id,
            text=f':rotating_light: *{severity.upper()} INCIDENT* :rotating_light:',
            blocks=[
                {
                    'type': 'header',
                    'text': {
                        'type': 'plain_text',
                        'text': f'🚨 {severity.upper()}: {title}',
                    }
                },
                {
                    'type': 'section',
                    'fields': [
                        {'type': 'mrkdwn', 'text': f'*Incident ID:*\n{incident_id}'},
                        {'type': 'mrkdwn', 'text': f'*Severity:*\n{severity}'},
                        {'type': 'mrkdwn', 'text': f'*Status:*\nInvestigating'},
                        {'type': 'mrkdwn', 'text': f'*Commander:*\nUnassigned'},
                    ]
                },
                {
                    'type': 'actions',
                    'elements': [
                        {
                            'type': 'button',
                            'text': {'type': 'plain_text', 'text': 'I\'ll handle this'},
                            'action_id': f'assign_{incident_id}',
                            'style': 'primary',
                        },
                        {
                            'type': 'button',
                            'text': {'type': 'plain_text', 'text': 'Update status'},
                            'action_id': f'update_{incident_id}',
                        },
                    ]
                }
            ]
        )
        
        # Invite key people
        await self.invite_responders(channel_id, severity)
        
        # Store incident
        self.incidents[incident_id] = {
            'channel_id': channel_id,
            'severity': severity,
            'title': title,
            'timeline': [],
        }
        
        # Schedule status updates (every 15 minutes for SEV-1)
        if severity == 'SEV-1':
            asyncio.create_task(self.auto_status_updates(incident_id))
    
    async def auto_status_updates(self, incident_id: str):
        while incident_id in self.incidents:
            await asyncio.sleep(900)  # 15 minutes
            
            # Post reminder
            channel_id = self.incidents[incident_id]['channel_id']
            self.slack.chat_postMessage(
                channel=channel_id,
                text=':hourglass_flowing_sand: *Status Update Due* :hourglass_flowing_sand:\nPlease provide an update on incident progress.'
            )
```

---

## 7. Dashboards

### 7.1 Grafana Dashboards

**Dashboard 1: Infrastructure Overview**
- Node health (CPU, memory, disk, network)
- Block production rate
- Finality lag
- Peer count
- Transaction pool size

**Dashboard 2: Security Monitoring**
- Failed authentication attempts (last 24 hours)
- Large transactions (>100K DALLA)
- Treasury balance (trending)
- Bridge volume (by asset)
- Governance proposals (active, voting, executed)

**Dashboard 3: Performance Metrics**
- API latency (P50, P95, P99)
- RPC request rate
- WebSocket connections
- Database query time
- Cache hit rate

**Dashboard 4: Alerts & Incidents**
- Active alerts (by severity)
- Incident count (by week)
- Mean time to detect (MTTD)
- Mean time to resolve (MTTR)
- False positive rate

**Example Dashboard JSON:**

```json
{
  "dashboard": {
    "title": "BelizeChain Security Monitoring",
    "panels": [
      {
        "title": "Treasury Balance (DALLA)",
        "type": "graph",
        "targets": [
          {
            "expr": "belizechain_treasury_balance",
            "legendFormat": "Treasury Balance"
          }
        ],
        "alert": {
          "conditions": [
            {
              "evaluator": {
                "type": "lt",
                "params": [10000000]  # Alert if <10M DALLA
              },
              "operator": {
                "type": "and"
              },
              "query": {
                "params": ["A", "5m", "now"]
              },
              "type": "query"
            }
          ],
          "executionErrorState": "alerting",
          "frequency": "1m",
          "handler": 1,
          "name": "Treasury balance alert"
        }
      },
      {
        "title": "Large Transactions (>100K DALLA)",
        "type": "table",
        "targets": [
          {
            "expr": "topk(10, belizechain_transaction_amount > 100000)",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

---

## 8. Implementation Timeline

**Phase 1 (Weeks 1-2): Core Infrastructure**
- Deploy Prometheus + Grafana + Loki (HA mode)
- Configure node exporters (all servers)
- Implement basic alert rules (CPU, memory, disk)
- **Budget:** $15k (infrastructure setup)

**Phase 2 (Weeks 3-4): Blockchain Monitoring**
- Deploy custom blockchain metrics exporter
- Implement transaction monitoring
- Set up treasury/bridge balance monitoring
- **Budget:** $20k (development + testing)

**Phase 3 (Weeks 5-6): Application Monitoring**
- Integrate RUM (Datadog) for UI portals
- Deploy API monitoring (OpenMetrics)
- Implement smart contract event monitoring
- **Budget:** $15k (integrations)

**Phase 4 (Weeks 7-8): Threat Intelligence**
- Integrate Chainalysis KYT API
- Set up CVE monitoring (automated scanning)
- Deploy incident bot (Slack)
- **Budget:** $10k (API subscriptions + development)

**Phase 5 (Weeks 9-10): Auto-Remediation**
- Implement safe auto-remediation rules
- Deploy auto-rate limiting
- Test auto-account freezing (testnet)
- **Budget:** $15k (development + testing)

**Total Setup Cost:** $75k  
**Operational Cost:** $30k/year (API subscriptions, infrastructure)

---

## 9. Success Metrics

**Detection Performance:**
- **MTTD (Mean Time To Detect):** <15 minutes (target), <5 minutes (stretch goal)
- **False Positive Rate:** <10% (target), <5% (stretch goal)
- **Alert Coverage:** 100% of critical events (SEV-1 scenarios)

**Response Performance:**
- **MTTR (Mean Time To Resolve):** <4 hours (SEV-1), <24 hours (SEV-2)
- **Auto-Remediation Success Rate:** >90% (safe actions only)
- **Incident Escalation Time:** <2 minutes (on-call notified)

**System Reliability:**
- **Monitoring Uptime:** 99.9% (3 nines)
- **Data Retention:** 30 days (metrics), 90 days (security logs)
- **Query Performance:** <500ms (P95 dashboard load time)

---

**Document Status:** ✅ IMPLEMENTATION READY

**Dependencies:** Incident Response Plan, Security Audit (validates detection rules)

**Next Review:** February 2026 (post-implementation)

---

*Built with 💎 for the sovereign nation of Belize 🇧🇿*  
*Continuous vigilance: Protecting citizen funds 24/7/365.*
