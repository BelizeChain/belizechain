# Bridge Security & Monitoring

## Overview

Comprehensive security monitoring system for BelizeChain cross-chain bridges. Provides real-time anomaly detection, automated incident response, and forensic analysis capabilities.

## Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Security Monitoring Stack                     │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Detector   │  │   Analyzer   │  │   Responder  │         │
│  │              │  │              │  │              │         │
│  │ • Anomalies  │─▶│ • Pattern    │─▶│ • Pause      │         │
│  │ • Thresholds │  │   matching   │  │ • Alert      │         │
│  │ • Rules      │  │ • ML models  │  │ • Slash      │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│         │                  │                  │                  │
│         └──────────────────┴──────────────────┘                  │
│                            │                                      │
│                  ┌─────────▼─────────┐                          │
│                  │  Event Stream      │                          │
│                  │  (Kafka/Redis)     │                          │
│                  └─────────┬─────────┘                          │
└────────────────────────────┼──────────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
         ┌────▼────┐    ┌───▼────┐    ┌───▼────┐
         │Ethereum │    │Polkadot│    │BelizeC │
         │ Bridge  │    │  XCM   │    │ Chain  │
         └─────────┘    └────────┘    └────────┘
```

## Monitoring Components

### 1. Real-Time Event Streaming

**Kafka Topics**:
- `bridge.ethereum.deposits` - ETH→BZC deposits
- `bridge.ethereum.withdrawals` - BZC→ETH withdrawals
- `bridge.polkadot.xcm` - XCM messages
- `bridge.security.alerts` - Security events
- `bridge.relayer.health` - Relayer status

**Event Schema**:
```json
{
  "timestamp": "2025-11-04T12:00:00Z",
  "bridge": "ethereum",
  "type": "deposit",
  "transaction_hash": "0xabc123...",
  "user": "5GrwvaEF...",
  "asset": "wETH",
  "amount": "1500000000000000000",
  "fee": "1500000000000000",
  "status": "confirmed",
  "confirmations": 12,
  "signature_count": 3
}
```

### 2. Anomaly Detection Engine

**Detection Rules**:

#### Volume Anomalies
```python
class VolumeAnomalyDetector:
    def detect(self, current_volume: float, window_hours: int = 24) -> Alert:
        # Calculate baseline (30-day moving average)
        baseline = self.get_baseline(window_hours)
        
        # Z-score calculation
        z_score = (current_volume - baseline.mean) / baseline.std
        
        if z_score > 3.0:  # >3 sigma
            return Alert(
                severity='CRITICAL',
                type='VOLUME_SPIKE',
                message=f'Volume spike: {current_volume:.2f} ({z_score:.1f}σ)',
                action='PAUSE_BRIDGE'
            )
        elif z_score > 2.0:  # >2 sigma
            return Alert(
                severity='WARNING',
                type='VOLUME_INCREASE',
                message=f'Volume increase: {current_volume:.2f} ({z_score:.1f}σ)',
                action='NOTIFY_TEAM'
            )
```

#### Behavioral Anomalies
```python
class BehavioralAnomalyDetector:
    def detect_rapid_withdrawal(self, user_address: str) -> Alert:
        # Get user's transaction history
        recent_txs = self.get_recent_transactions(user_address, hours=1)
        
        if len(recent_txs) > 100:  # >100 tx/hour
            return Alert(
                severity='HIGH',
                type='RAPID_WITHDRAWAL',
                user=user_address,
                count=len(recent_txs),
                action='RATE_LIMIT_USER'
            )
    
    def detect_unusual_pattern(self, transaction: Transaction) -> Alert:
        # ML-based pattern detection
        features = self.extract_features(transaction)
        anomaly_score = self.ml_model.predict(features)
        
        if anomaly_score > 0.9:  # 90% confidence of anomaly
            return Alert(
                severity='MEDIUM',
                type='UNUSUAL_PATTERN',
                score=anomaly_score,
                action='FLAG_FOR_REVIEW'
            )
```

#### Price Deviation Detection
```python
class PriceDeviationDetector:
    def detect(self, asset: str) -> Alert:
        # Get price from multiple oracles
        bzc_price = self.get_belizechain_price(asset)
        external_prices = [
            self.get_coingecko_price(asset),
            self.get_binance_price(asset),
            self.get_coinbase_price(asset),
        ]
        
        median_external = statistics.median(external_prices)
        deviation = abs(bzc_price - median_external) / median_external
        
        if deviation > 0.05:  # >5% deviation
            return Alert(
                severity='CRITICAL',
                type='PRICE_MANIPULATION',
                bzc_price=bzc_price,
                external_price=median_external,
                deviation=f'{deviation*100:.2f}%',
                action='PAUSE_ASSET'
            )
```

### 3. Automated Response System

**Response Actions**:

#### Emergency Pause
```rust
#[ink(message)]
pub fn emergency_pause(&mut self, reason: String) -> Result<()> {
    // Requires 2-of-5 government relayers
    self.verify_government_multisig()?;
    
    // Pause all bridge operations
    self.paused = true;
    self.pause_reason = reason.clone();
    self.pause_timestamp = self.env().block_timestamp();
    
    // Emit event
    self.env().emit_event(EmergencyPause {
        reason,
        initiated_by: self.env().caller(),
        timestamp: self.env().block_timestamp(),
    });
    
    // Notify all stakeholders
    self.send_emergency_alerts()?;
    
    Ok(())
}
```

#### Rate Limiting
```rust
#[ink(message)]
pub fn apply_rate_limit(&mut self, user: AccountId, duration_seconds: u64) -> Result<()> {
    self.only_security_module()?;
    
    let release_time = self.env().block_timestamp() + duration_seconds;
    self.rate_limited_users.insert(user, &release_time);
    
    self.env().emit_event(UserRateLimited {
        user,
        duration_seconds,
        release_time,
    });
    
    Ok(())
}
```

#### Asset Freeze
```rust
#[ink(message)]
pub fn freeze_asset(&mut self, asset: AccountId, reason: String) -> Result<()> {
    self.only_governance()?;
    
    // Prevent further bridging of this asset
    if let Some(mut config) = self.asset_configs.get(asset) {
        config.active = false;
        config.freeze_reason = Some(reason.clone());
        self.asset_configs.insert(asset, &config);
    }
    
    self.env().emit_event(AssetFrozen {
        asset,
        reason,
        timestamp: self.env().block_timestamp(),
    });
    
    Ok(())
}
```

### 4. Circuit Breakers

Automatically halt operations when thresholds exceeded:

| Circuit Breaker | Threshold | Cool-down | Auto-resume |
|----------------|-----------|-----------|-------------|
| **Hourly Volume** | >$1M | 1 hour | Yes |
| **TVL Drop** | >20% | 24 hours | No (governance) |
| **Failed Signatures** | >10/hour | 2 hours | Yes |
| **Gas Price Spike** | >1000 gwei | 30 minutes | Yes |
| **Relayer Downtime** | >3 offline | 1 hour | No (manual) |

**Implementation**:
```python
class CircuitBreaker:
    def __init__(self):
        self.thresholds = {
            'hourly_volume': 1_000_000,  # $1M
            'tvl_drop': 0.20,  # 20%
            'failed_sigs': 10,
            'gas_price': 1000,  # gwei
        }
        self.state = 'CLOSED'  # CLOSED, OPEN, HALF_OPEN
    
    def check(self, metric: str, value: float):
        if self.state == 'OPEN':
            if self.should_resume():
                self.state = 'HALF_OPEN'
        
        if value > self.thresholds[metric]:
            self.trip(metric, value)
    
    def trip(self, metric: str, value: float):
        self.state = 'OPEN'
        self.trip_time = time.time()
        
        # Pause bridge
        blockchain.pause_bridge()
        
        # Alert team
        send_alert(f'Circuit breaker tripped: {metric} = {value}')
```

## Dashboards

### Grafana Dashboards

#### 1. Bridge Health Dashboard

**Panels**:
- Bridge status (operational/paused)
- Transactions per minute (line chart)
- Success rate (gauge)
- Average latency (line chart)
- TVL by chain (bar chart)
- Relayer uptime (table)

**Alerts**:
- Success rate <95%
- Average latency >5 minutes
- Any relayer uptime <99%

#### 2. Security Dashboard

**Panels**:
- Active alerts (table)
- Anomaly detection score (heatmap)
- Suspicious transactions (list)
- Circuit breaker status (status panel)
- Failed signatures (counter)
- Rate-limited users (table)

#### 3. Economic Dashboard

**Panels**:
- Daily volume (line chart)
- Fee revenue (area chart)
- TVL trends (multi-line chart)
- Top assets by volume (bar chart)
- Bridge utilization (gauge)
- Profit/loss (line chart)

### Alert Configuration

**Prometheus Alerting Rules**:
```yaml
groups:
  - name: bridge_alerts
    interval: 30s
    rules:
      - alert: BridgeVolumeSpike
        expr: rate(bridge_volume_total[5m]) > 10000000
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Bridge volume spike detected"
          description: "Volume: {{ $value }} (>10M DALLA/5min)"
      
      - alert: RelayerDown
        expr: up{job="relayer"} == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Relayer {{ $labels.instance }} is down"
      
      - alert: HighFailureRate
        expr: rate(bridge_failed_transactions[5m]) / rate(bridge_total_transactions[5m]) > 0.05
        for: 10m
        labels:
          severity: high
        annotations:
          summary: "Bridge failure rate >5%"
```

## Incident Response

### Runbooks

#### Runbook 1: Volume Spike Investigation

**Trigger**: >10M DALLA bridged in 1 hour

**Steps**:
1. Check alert details in Grafana
2. Query top users: `SELECT user, SUM(amount) FROM transfers WHERE timestamp > NOW() - INTERVAL '1 hour' GROUP BY user ORDER BY SUM(amount) DESC LIMIT 10`
3. Review transaction patterns (deposits vs withdrawals)
4. Check external news (hack, market event)
5. If suspicious: Pause bridge, investigate further
6. If legitimate: Increase relayer capacity

**Escalation**: If >$10M, notify CFO immediately

#### Runbook 2: Relayer Compromise

**Trigger**: Relayer signs invalid transaction

**Steps**:
1. Auto-slash relayer stake (via smart contract)
2. Remove from validator set immediately
3. Pause bridge operations
4. Rotate multi-sig keys (within 1 hour)
5. Audit other relayer nodes (security scan)
6. Investigate root cause (log analysis, forensics)
7. Deploy fix if vulnerability found
8. Publish post-mortem (within 24 hours)

**Recovery Time**: 1-2 hours

#### Runbook 3: Price Manipulation

**Trigger**: Asset price deviates >5% from external markets

**Steps**:
1. Pause trading for affected asset
2. Query oracle sources: CoinGecko, Binance, Coinbase
3. Recalculate median price
4. If BelizeChain price is outlier: Investigate AMM pool
5. Check for flash loan attacks or large trades
6. If manipulation detected: Revert transactions (if <1 hour old)
7. Resume trading after price normalizes

**Prevention**: Implement TWAP oracle, increase liquidity

## Security Audits

### Regular Audit Schedule

| Audit Type | Frequency | Provider | Last Audit |
|------------|-----------|----------|------------|
| **Smart Contract** | Quarterly | Trail of Bits | Q4 2024 |
| **Infrastructure** | Semi-annual | NCC Group | Q3 2024 |
| **Penetration Test** | Annual | HackerOne | Q2 2024 |
| **Economic Model** | Annual | Gauntlet | Q1 2024 |

### Bug Bounty Program

**Rewards**:
- **Critical** (bridge drain): $50,000
- **High** (fund loss): $25,000
- **Medium** (DoS): $10,000
- **Low** (info disclosure): $1,000

**Scope**:
- Ethereum bridge contract
- XCM bridge contract
- Relayer network software
- Asset manager contracts

**Out of Scope**:
- Front-end UI bugs
- Social engineering
- Third-party dependencies

## Compliance & Reporting

### Daily Reports

Generated automatically at 00:00 UTC:

- Total volume (ETH, XCM)
- Number of transactions
- Fee revenue
- TVL changes
- Alert summary
- Relayer performance

**Distribution**: Email to finance team, post to Slack

### Monthly Reports

Generated on 1st of each month:

- Bridge utilization metrics
- Security incident summary
- Economic performance
- User growth
- Governance actions
- Recommendations

**Distribution**: Board of directors, public blog post

### Regulatory Compliance

**KYC/AML Checks**:
- Transfers >$10,000: Requires KYC
- Transfers >$50,000: Enhanced due diligence
- Sanctioned addresses: Auto-block (OFAC list)

**Transaction Monitoring**:
- Pattern analysis (structuring detection)
- Velocity checks (rapid transfers)
- Geographic risk scoring
- PEP (Politically Exposed Person) screening

## Performance Metrics

### SLAs (Service Level Agreements)

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Uptime** | 99.9% | 99.95% | ✅ |
| **Deposit latency** | <5 min | 3.2 min | ✅ |
| **Withdrawal latency** | <8 min | 5.7 min | ✅ |
| **Alert response** | <5 min | 2.1 min | ✅ |
| **Incident resolution** | <2 hours | 1.3 hours | ✅ |

### Key Performance Indicators (KPIs)

- **Daily Active Users**: 1,500 (target: 5,000)
- **Daily Volume**: $2.5M (target: $10M)
- **Success Rate**: 99.2% (target: 99.5%)
- **Customer Satisfaction**: 4.3/5 (target: 4.5/5)

## Resources

- **Security Dashboard**: https://security.belizechain.org
- **Grafana**: https://grafana.belizechain.org
- **Prometheus**: https://prometheus.belizechain.org
- **Bug Bounty**: https://hackerone.com/belizechain
- **Incident History**: https://status.belizechain.org
- **Contact**: security@belizechain.org

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
