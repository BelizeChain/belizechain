# 🚀 Deployment Guides

**Deploy and manage BelizeChain infrastructure**

---

## 📚 Overview

These guides are for **system administrators**, **DevOps engineers**, and **infrastructure teams** deploying BelizeChain nodes, validators, and infrastructure.

> Current operational target: Ceiba self-hosted testnet. Kubernetes/Azure material is legacy unless explicitly reactivated.

---

## 🎯 Who These Guides Are For

### System Administrators
- Deploy full nodes
- Manage infrastructure
- Monitor performance
- Handle backups

### Validator Operators
- Setup validator nodes
- Configure staking
- Secure validators
- Optimize performance

### DevOps Teams
- Automate deployments
- Configure CI/CD
- Container orchestration
- Infrastructure as Code

### Enterprise IT
- Private blockchain networks
- High-availability setups
- Disaster recovery
- Compliance requirements

---

## 📖 Available Guides

- [Phase 2 Ceiba Services Plan](./PHASE2_CEIBA_SERVICES_PLAN.md) - Sibling service rollout order, health gates, and rollback pattern

### 1. [System Requirements](./requirements.md)
**Hardware and software prerequisites**

**What you'll learn**:
- Minimum hardware requirements
- Recommended specifications
- Operating system compatibility
- Network requirements
- Storage considerations
- Cost estimates

**Time**: 10 minutes  
**Level**: ⭐ Beginner

---

### 2. [Installation Guide](./installation.md)
**Install BelizeChain node from scratch**

**What you'll learn**:
- Install dependencies
- Download BelizeChain binary
- Compile from source (optional)
- Configure node
- Start node
- Verify installation
- Troubleshoot issues

**Time**: 30-60 minutes  
**Level**: ⭐⭐ Intermediate

**Installation methods**:
- Binary installation (easiest)
- Docker and Docker Compose (recommended)
- Build from source (advanced)
- Kubernetes deployment (legacy reference)

---

### 3. [Configuration Guide](./configuration.md)
**Configure your BelizeChain node**

**What you'll learn**:
- Node configuration file
- Network settings (mainnet/testnet)
- RPC endpoints
- Validator configuration
- Security hardening
- Performance tuning
- Logging configuration

**Time**: 45 minutes  
**Level**: ⭐⭐⭐ Advanced

---

### 4. [Validator Setup](./validator-setup.md)
**Become a validator and secure the network**

**What you'll learn**:
- Validator requirements (10,000 DALLA stake)
- Setup validator node
- Generate session keys
- Bond DALLA tokens
- Set commission rate
- Start validating
- Monitor validator performance

**Time**: 2-3 hours  
**Level**: ⭐⭐⭐⭐ Expert

**Prerequisites**:
- Completed installation
- 10,000+ DALLA tokens
- Dedicated server
- Technical expertise

**Rewards**: ~15% APY on staked DALLA

---

### 5. [Maintenance Guide](./maintenance.md)
**Keep your node running smoothly**

**What you'll learn**:
- Update node software
- Database maintenance
- Log rotation
- Backup strategies
- Restore from backup
- Monitor disk space
- Handle chain reorganizations

**Time**: Ongoing  
**Level**: ⭐⭐ Intermediate

---

### 6. [Monitoring & Alerting](./monitoring.md)
**Setup comprehensive monitoring**

**What you'll learn**:
- Prometheus metrics
- Grafana dashboards
- Alert rules
- Performance metrics
- Block production monitoring
- Peer connections
- Alerting channels (email, Slack, Discord)

**Time**: 1-2 hours  
**Level**: ⭐⭐⭐ Advanced

**Monitoring tools**:
- Prometheus + Grafana (recommended)
- DataDog
- New Relic
- Custom monitoring

---

### 7. [Security Guide](./security.md)
**Secure your node and validator**

**What you'll learn**:
- Firewall configuration
- SSH hardening
- Key management
- DDoS protection
- Intrusion detection
- Security auditing
- Incident response

**Time**: 2-3 hours  
**Level**: ⭐⭐⭐⭐ Expert

---

### 8. [High Availability Setup](./high-availability.md)
**Deploy fault-tolerant infrastructure**

**What you'll learn**:
- Load balancing
- Failover configuration
- Database replication
- Geographic distribution
- Disaster recovery
- SLA targets

**Time**: 4-6 hours  
**Level**: ⭐⭐⭐⭐⭐ Expert

**Use cases**:
- Validator operations
- RPC endpoint providers
- Enterprise deployments
- Mission-critical infrastructure

---

### 9. [Docker Deployment](./docker-deployment.md)
**Deploy using Docker containers**

**What you'll learn**:
- Docker image
- docker-compose setup
- Volume management
- Container networking
- Update containers
- Backup containers

**Time**: 1 hour  
**Level**: ⭐⭐ Intermediate

**Benefits**:
- Easy deployment
- Consistent environment
- Simple updates
- Portable

---

### 10. [Testnet Deployment (Ceiba)](./TESTNET_DEPLOYMENT.md)
**Deploy to the current Ceiba self-hosted testnet target**

**What you'll learn**:
- Host prerequisites
- Node build and rollout
- Validator/full-node topology
- Monitoring and maintenance
- Troubleshooting

**Time**: 2-3 hours  
**Level**: ⭐⭐⭐ Advanced

**Use cases**:
- Current BelizeChain testnet operations
- Validator onboarding
- Self-hosted node administration

---

## 🛤️ Deployment Paths

### Path 1: Basic Node Operator
**Just want to run a node**

1. ✅ [System Requirements](./requirements.md)
2. ✅ [Installation Guide](./installation.md)
3. ✅ [Configuration Guide](./configuration.md)
4. ✅ [Maintenance Guide](./maintenance.md)

**Time**: 1 day  
**Outcome**: Running full node

---

### Path 2: Validator Operator
**Want to become a validator**

1. ✅ [System Requirements](./requirements.md)
2. ✅ [Installation Guide](./installation.md)
3. ✅ [Configuration Guide](./configuration.md)
4. ✅ [Security Guide](./security.md)
5. ✅ [Validator Setup](./validator-setup.md)
6. ✅ [Monitoring & Alerting](./monitoring.md)
7. ✅ [Maintenance Guide](./maintenance.md)

**Time**: 1 week  
**Outcome**: Active validator earning rewards

---

### Path 3: Enterprise Deployment
**Production-grade infrastructure**

1. ✅ [System Requirements](./requirements.md)
2. ✅ [Testnet Deployment (Ceiba)](./TESTNET_DEPLOYMENT.md)
3. ✅ [Configuration Guide](./configuration.md)
4. ✅ [Security Guide](./security.md)
5. ✅ [High Availability Setup](./high-availability.md)
6. ✅ [Monitoring & Alerting](./monitoring.md)
7. ✅ [Maintenance Guide](./maintenance.md)

**Time**: 2-3 weeks  
**Outcome**: Production-ready, highly available infrastructure

---

## 🏗️ Architecture Options

### Single Node (Basic)
```
┌─────────────────────┐
│  BelizeChain Node   │
│                     │
│  - Mainnet          │
│  - RPC enabled      │
│  - Archive mode     │
└─────────────────────┘
```

**Use case**: Personal node, development, testing  
**Cost**: $50-100/month  
**Guides**: Installation + Configuration

---

### Validator Node (Staking)
```
┌─────────────────────────────┐
│  BelizeChain Validator Node │
│                             │
│  - Session keys             │
│  - Bonded: 10,000 DALLA     │
│  - Commission: 5-20%        │
│  - Monitoring enabled       │
│  - Backups configured       │
└─────────────────────────────┘
```

**Use case**: Earn staking rewards, secure network  
**Cost**: $200-500/month  
**Guides**: Validator Setup + Security + Monitoring

---

### High Availability (Production)
```
┌──────────────────────────────────────┐
│  Load Balancer                       │
└────────┬─────────────────────┬───────┘
         │                     │
   ┌─────▼──────┐       ┌─────▼──────┐
   │  Node 1    │       │  Node 2    │
   │  Primary   │◄─────►│  Standby   │
   └────────────┘       └────────────┘
         │                     │
   ┌─────▼─────────────────────▼───────┐
   │  Replicated Database              │
   └───────────────────────────────────┘
```

**Use case**: RPC endpoints, critical infrastructure  
**Cost**: $500-2000/month  
**Guides**: High Availability + Monitoring + Security

---

### Multi-Region (Enterprise)
```
    ┌──────────────────────────────┐
    │  Global Load Balancer        │
    └───┬──────────────────────┬───┘
        │                      │
  ┌─────▼─────┐          ┌─────▼─────┐
  │  US-East  │          │  EU-West  │
  │  Cluster  │          │  Cluster  │
  │  (3 nodes)│          │  (3 nodes)│
  └───────────┘          └───────────┘
        │                      │
  ┌─────▼──────────────────────▼─────┐
  │  Global Database Replication     │
  └──────────────────────────────────┘
```

**Use case**: Global applications, 99.99% uptime  
**Cost**: $2000+/month  
**Guides**: High Availability + Security. Treat Kubernetes content as legacy until it is rebuilt from current Ceiba operations.

---

## 💻 Deployment Environments

### Development
- **Purpose**: Testing, development
- **Network**: Testnet
- **Size**: Small (2 CPU, 4GB RAM)
- **Cost**: $20-50/month
- **Guides**: Basic installation

### Staging
- **Purpose**: Pre-production testing
- **Network**: Testnet (mainnet config)
- **Size**: Medium (4 CPU, 8GB RAM)
- **Cost**: $100-200/month
- **Guides**: Installation + Configuration + Monitoring

### Production
- **Purpose**: Live applications
- **Network**: Mainnet
- **Size**: Large (8+ CPU, 16+ GB RAM)
- **Cost**: $500+/month
- **Guides**: All deployment guides

---

## 🔧 Quick Start Commands

### Install with Docker (Fastest)
```bash
# Pull image
docker pull belizechain/belizechain:latest

# Run node
docker run -d \
  --name belizechain-node \
  -p 9944:9944 \
  -p 9933:9933 \
  -p 30333:30333 \
  -v /data/belizechain:/data \
  belizechain/belizechain:latest \
  --chain mainnet \
  --rpc-external \
  --ws-external
```

### Install from Binary
```bash
# Download
wget https://releases.belizechain.org/latest/belizechain

# Make executable
chmod +x belizechain

# Run
./belizechain --chain mainnet
```

### Build from Source
```bash
# Clone repository
git clone https://github.com/belizechain/belizechain.git
cd belizechain

# Build
cargo build --release

# Run
./target/release/belizechain --chain mainnet
```

---

## 📊 Resource Calculators

### Node Resource Calculator

| Node Type | CPU | RAM | Storage | Network | Cost/Month |
|-----------|-----|-----|---------|---------|-----------|
| Light Node | 2 cores | 4 GB | 50 GB | 1 Mbps | $20-40 |
| Full Node | 4 cores | 8 GB | 500 GB | 5 Mbps | $80-150 |
| Archive Node | 8 cores | 16 GB | 2 TB | 10 Mbps | $200-400 |
| Validator | 4 cores | 16 GB | 500 GB | 10 Mbps | $200-500 |
| RPC Endpoint | 8 cores | 32 GB | 1 TB | 100 Mbps | $500-1000 |

**Calculator**: https://calculator.belizechain.org

---

## 🌍 Recommended Cloud Providers

### AWS (Amazon Web Services)
- **Instances**: t3.medium (basic), m5.xlarge (production)
- **Pros**: Reliable, global, well-supported
- **Cons**: Expensive
- **Guide**: [AWS Deployment Guide](./cloud-providers/aws.md)

### Google Cloud Platform (GCP)
- **Instances**: n2-standard-2 (basic), n2-standard-4 (production)
- **Pros**: Good performance, competitive pricing
- **Cons**: Complex billing
- **Guide**: [GCP Deployment Guide](./cloud-providers/gcp.md)

### DigitalOcean
- **Instances**: $40 droplet (basic), $160 droplet (production)
- **Pros**: Simple, affordable, good docs
- **Cons**: Limited regions
- **Guide**: [DigitalOcean Deployment Guide](./cloud-providers/digitalocean.md)

### Hetzner
- **Instances**: CX31 (basic), CPX51 (production)
- **Pros**: Cheapest, good hardware
- **Cons**: Limited support
- **Guide**: [Hetzner Deployment Guide](./cloud-providers/hetzner.md)

### Bare Metal (On-Premises)
- **Setup**: Dell PowerEdge, HP ProLiant
- **Pros**: Full control, no monthly fees
- **Cons**: High upfront cost, maintenance
- **Guide**: [Bare Metal Deployment Guide](./bare-metal.md)

---

## 🔐 Security Checklist

Before going live, ensure:

- [ ] SSH keys configured (no password auth)
- [ ] Firewall rules applied
- [ ] Automatic security updates enabled
- [ ] Backups configured and tested
- [ ] Monitoring and alerting setup
- [ ] Session keys secured (validators only)
- [ ] DDoS protection enabled
- [ ] Intrusion detection configured
- [ ] Incident response plan documented
- [ ] Team access controls defined

**Security Guide**: [Complete Security Guide](./security.md)

---

## 📈 Performance Optimization

### Database Tuning
```toml
# config.toml
[database]
cache_size = 8192      # MB
state_cache_size = 4096 # MB
```

### Network Optimization
```toml
[network]
max_peers = 50
out_peers = 25
reserved_peers = [
  "/ip4/1.2.3.4/tcp/30333/p2p/12D3..."
]
```

### RPC Optimization
```toml
[rpc]
max_connections = 1000
max_request_size = 15   # MB
max_response_size = 15  # MB
```

**Performance Guide**: [Configuration Guide](./configuration.md)

---

## 🆘 Support & Resources

### Official Documentation
- **Website**: https://docs.belizechain.org
- **GitHub**: https://github.com/belizechain/belizechain
- **API Docs**: https://api-docs.belizechain.org

### Community Support
- **Discord**: #validators, #node-operators channels
- **Forum**: forum.belizechain.org/infrastructure
- **Telegram**: @belizechain_validators

### Professional Support
- **Email**: infra@belizechain.org
- **Phone**: +501-CHAIN (24246)
- **Managed Services**: https://managed.belizechain.org

---

## 📚 Related Documentation

- [Technical Reference](../technical-reference/README.md) - Architecture deep dive
- [Developer Guides](../developer-guides/README.md) - Build applications
- [User Guides](../user-guides/README.md) - End-user documentation
- [Tutorials](../tutorials/README.md) - Step-by-step guides

---

## ✅ Deployment Checklist

Track your deployment progress:

### Planning Phase
- [ ] Read system requirements
- [ ] Choose deployment architecture
- [ ] Select cloud provider
- [ ] Budget approved
- [ ] Team assigned

### Installation Phase
- [ ] Server provisioned
- [ ] Dependencies installed
- [ ] BelizeChain installed
- [ ] Configuration completed
- [ ] Node started

### Security Phase
- [ ] Firewall configured
- [ ] SSH hardened
- [ ] Keys secured
- [ ] Backups setup
- [ ] Monitoring enabled

### Production Phase
- [ ] Synced with network
- [ ] Performance verified
- [ ] Alerts tested
- [ ] Documentation updated
- [ ] Team trained

### Validator Phase (if applicable)
- [ ] 10,000 DALLA acquired
- [ ] Session keys generated
- [ ] Tokens bonded
- [ ] Commission set
- [ ] Validation started

---

**Ready to deploy?** Start with [System Requirements](./requirements.md) →
