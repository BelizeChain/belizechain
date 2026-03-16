# Bridge Relayer Network

## Overview

The BelizeChain bridge relayer network securely facilitates cross-chain asset transfers between BelizeChain and external networks (Ethereum, Polkadot parachains). Relayers monitor source chains, validate transactions, and submit multi-signed proofs to destination chains.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Relayer Network                           │
│                                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Relayer 1 │  │Relayer 2 │  │Relayer 3 │  │Relayer 4 │   │
│  │(Gov't)   │  │(Gov't)   │  │(Partner) │  │(Partner) │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │             │             │           │
│       └─────────────┴─────────────┴─────────────┘           │
│                     │                                        │
│              ┌──────▼──────┐                                │
│              │ Coordinator  │                                │
│              │  (BelizeChain)│                               │
│              └──────┬──────┘                                │
└──────────────────────┼───────────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    ┌────▼────┐   ┌───▼────┐   ┌───▼────┐
    │Ethereum │   │Polkadot│   │BelizeC │
    │ Bridge  │   │  XCM   │   │ Chain  │
    └─────────┘   └────────┘   └────────┘
```

## Relayer Roles

### 1. Government Relayers (2/5)
- Operated by Belize Ministry of Finance
- High trust, critical infrastructure
- 24/7 monitoring and uptime guarantees

### 2. Partner Relayers (3/5)
- Operated by approved financial institutions
- Must stake 1M DALLA as collateral
- Subject to slashing for misbehavior

## Multi-Signature Scheme

**Threshold**: 3-of-5 validators required for all cross-chain operations

**Signature Aggregation**:
- BLS signature aggregation for efficiency
- Schnorr multi-signatures for Ethereum compatibility
- Ed25519 for Polkadot ecosystem

## Relayer Operations

### Ethereum → BelizeChain

1. **Event Monitoring**:
   ```python
   # Monitor Ethereum bridge contract
   event_filter = bridge_contract.events.Deposit.createFilter(fromBlock='latest')
   
   while True:
       for event in event_filter.get_new_entries():
           process_deposit_event(event)
   ```

2. **Proof Generation**:
   - Fetch transaction receipt
   - Extract Merkle proof
   - Validate with Ethereum light client
   - Sign deposit transaction

3. **Submission**:
   - Aggregate 3-of-5 signatures
   - Submit to BelizeChain bridge contract
   - Wait for finalization (2 blocks)

### BelizeChain → Ethereum

1. **Event Monitoring**:
   ```rust
   // Subscribe to WithdrawalRequested events
   let events = api.events().at_latest().await?;
   for event in events.iter() {
       if let Ok(withdrawal) = event.as_event::<WithdrawalRequested>() {
           process_withdrawal(withdrawal);
       }
   }
   ```

2. **Multi-Sig Unlock**:
   - Validate withdrawal request on BelizeChain
   - Sign Ethereum unlock transaction
   - Coordinate with other relayers
   - Submit to Ethereum multi-sig wallet

### Polkadot ↔ BelizeChain (XCM)

1. **XCM Message Relay**:
   - Monitor HRMP (relay chain) for incoming XCM messages
   - Validate message authenticity
   - Execute on destination chain
   - Emit confirmation event

## Security Measures

### 1. Light Client Verification
- Ethereum: Header sync via RPC
- Polkadot: GRANDPA finality proofs
- Prevents false deposit claims

### 2. Replay Protection
- Nonce tracking per user
- Transaction hash deduplication
- Time-based expiry (24 hours)

### 3. Rate Limiting
- Per-user daily limits (1M DALLA)
- Per-chain aggregate limits (10M DALLA)
- Circuit breaker for anomaly detection

### 4. Slashing Conditions
- **Double-signing**: Lose 100% stake (1M DALLA)
- **Downtime** (>24h): Lose 10% stake
- **Censorship**: Fail to relay valid tx for >1 hour, lose 5% stake
- **Invalid signature**: Lose 50% stake

### 5. Emergency Pause
- Any 2 government relayers can pause bridges
- Requires on-chain governance vote to unpause
- Maximum pause duration: 7 days

## Monitoring & Alerts

### Key Metrics

| Metric | Alert Threshold | Action |
|--------|----------------|--------|
| **Relayer uptime** | <99% over 24h | Warning email |
| **Signature delay** | >5 minutes | Escalate to on-call |
| **Failed signatures** | >3 in 1 hour | Auto-investigate |
| **TVL change** | >20% in 1 hour | Pause bridge |
| **Gas price spike** | >500 gwei | Delay submissions |
| **Withdrawal queue** | >100 pending | Add relayer capacity |

### Alerting Channels
- **Slack**: #bridge-alerts (all events)
- **PagerDuty**: Critical failures
- **Email**: Daily summaries
- **Telegram**: Community notifications

## Deployment

### Requirements

**Hardware**:
- CPU: 4 cores (8 recommended)
- RAM: 16GB (32GB for Ethereum full node)
- Storage: 1TB SSD (2TB for archival)
- Network: 100Mbps (1Gbps recommended)

**Software**:
- Ubuntu 22.04 LTS
- Docker 24.0+
- Python 3.11+
- Rust 1.75+

### Installation

```bash
# Clone relayer repository
git clone https://github.com/BelizeChain/relayer-network
cd relayer-network

# Install dependencies
pip install -r requirements.txt

# Configure environment
cp .env.example .env
nano .env  # Add RPC endpoints, keys, etc.

# Start relayer
docker-compose up -d

# Check logs
docker-compose logs -f relayer
```

### Configuration

```yaml
# config.yaml
relayer:
  name: "Relayer-1"
  keystore: "/path/to/keystore"
  
networks:
  ethereum:
    rpc: "https://mainnet.infura.io/v3/YOUR-KEY"
    bridge_address: "0x1234..."
    confirmation_blocks: 12
    
  belizechain:
    rpc: "wss://rpc.belizechain.org"
    bridge_address: "5GrwvaEF5zXb..."
    
  polkadot:
    relay_rpc: "wss://rpc.polkadot.io"
    parachain_id: 2000

monitoring:
  prometheus_port: 9090
  grafana_enabled: true
  alert_webhooks:
    - "https://hooks.slack.com/services/..."
    
slashing:
  stake_amount: 1000000000000000000  # 1M DALLA
  slash_address: "5FHneW46xGXgs5m..."
```

## Relayer API

### REST Endpoints

```
GET /health                  # Health check
GET /metrics                 # Prometheus metrics
GET /status                  # Relayer status
GET /queue                   # Pending transactions
POST /withdraw               # Submit withdrawal (internal)
POST /sign                   # Request signature (internal)
```

### WebSocket Subscriptions

```javascript
const ws = new WebSocket('wss://relayer.belizechain.org');

// Subscribe to transfer events
ws.send(JSON.stringify({
  method: 'subscribe',
  params: ['transfers'],
}));

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Transfer:', data);
};
```

## Incident Response

### Playbook: Bridge Halt

**Trigger**: TVL drops >20% in 1 hour

**Steps**:
1. Auto-pause bridge contracts (2 gov relayers)
2. Notify all relayers via PagerDuty
3. Investigate transaction logs
4. Identify exploit or bug
5. Deploy fix
6. Test on testnet
7. Submit governance proposal to unpause
8. Resume after 24h cool-down

**Recovery Time**: 24-48 hours

### Playbook: Relayer Compromise

**Trigger**: Relayer signs invalid transaction

**Steps**:
1. Slash relayer stake (automatic)
2. Remove from validator set
3. Rotate multi-sig keys
4. Audit other relayer nodes
5. Investigate root cause
6. Publish post-mortem

**Recovery Time**: 1-2 hours

## Performance Benchmarks

| Operation | Latency | Throughput |
|-----------|---------|----------|
| **ETH→BZC deposit** | 3-5 minutes | 100 tx/min |
| **BZC→ETH withdrawal** | 5-8 minutes | 80 tx/min |
| **XCM transfer** | 12-18 seconds | 200 tx/min |
| **Signature aggregation** | 500ms | - |

## Economic Model

### Relayer Revenue

**Fee Share**: 50% of bridge fees (0.05% of volume)

**Estimated Annual Revenue** (per relayer):
- ETH bridge: $10M volume × 0.1% × 50% × 20% = $1,000
- XCM bridge: $5M volume × 0.1% × 50% × 20% = $500
- **Total**: $1,500/year per relayer

**Note**: Assumes 5 relayers split fees equally

### Slashing Distribution
- 80% burned (deflationary)
- 20% to treasury (bug bounties)

## Governance Integration

### Parameter Updates (via on-chain governance)
- Signature threshold (3-of-5)
- Daily transfer limits (1M DALLA)
- Slashing parameters
- Fee distribution
- Emergency pause duration

### Relayer Rotation
- Quarterly applications open
- Governance vote required
- 7-day transition period
- Old relayer stake locked for 30 days

## Security Audits

**Completed**:
- [ ] Internal security review (Q1 2025)
- [ ] External audit by professional firm (TBD, not yet engaged)
- [ ] Formal verification of signature scheme (Q2 2025)

**Pending**:
- [ ] Chaos engineering tests (Q3 2025)
- [ ] Bug bounty program launch ($50k pool)

## Resources

- **Documentation**: https://docs.belizechain.org/bridges/relayers
- **GitHub**: https://github.com/BelizeChain/relayer-network
- **Relayer Dashboard**: https://relayers.belizechain.org
- **Support**: relayers@belizechain.org

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
