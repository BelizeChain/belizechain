# 📦 System Requirements

**Hardware and software requirements for running BelizeChain nodes**

---

## 🖥️ Validator Node Requirements

### **Minimum Requirements**

| Component | Specification |
|-----------|---------------|
| **CPU** | 4 cores @ 2.5+ GHz |
| **RAM** | 8 GB |
| **Storage** | 200 GB SSD |
| **Network** | 100 Mbps symmetrical |
| **OS** | Ubuntu 22.04 LTS |

**Expected Performance**: Can validate, but may experience delays during high load.

---

### **Recommended Requirements**

| Component | Specification |
|-----------|---------------|
| **CPU** | 8+ cores @ 3.0+ GHz |
| **RAM** | 16 GB |
| **Storage** | 500 GB NVMe SSD |
| **Network** | 1 Gbps symmetrical |
| **OS** | Ubuntu 22.04 LTS |

**Expected Performance**: Smooth operation under all conditions.

---

### **Production Requirements** (High-stake validators)

| Component | Specification |
|-----------|---------------|
| **CPU** | 16+ cores @ 3.5+ GHz (AMD EPYC, Intel Xeon) |
| **RAM** | 32+ GB ECC |
| **Storage** | 1+ TB NVMe SSD (RAID 1) |
| **Network** | 10 Gbps + redundant connections |
| **OS** | Ubuntu 22.04 LTS (hardened) |
| **Backup** | Daily snapshots, hot standby node |

**Expected Performance**: Enterprise-grade reliability with 99.9%+ uptime.

---

## 🔍 Full Node Requirements

### **Archive Node** (Full history)

| Component | Specification |
|-----------|---------------|
| **CPU** | 8+ cores |
| **RAM** | 16+ GB |
| **Storage** | 2+ TB SSD (grows ~50 GB/month) |
| **Network** | 500 Mbps+ |

**Use Case**: Block explorers, analytics, historical queries.

---

### **RPC Node** (API access)

| Component | Specification |
|-----------|---------------|
| **CPU** | 16+ cores |
| **RAM** | 32+ GB |
| **Storage** | 500 GB SSD |
| **Network** | 1+ Gbps |

**Use Case**: dApp backends, public API endpoints.

---

## 💻 Supported Operating Systems

### **Linux** (Recommended)

✅ **Ubuntu 22.04 LTS** - Best supported, most tested  
✅ **Ubuntu 20.04 LTS** - Fully supported  
✅ **Debian 11** - Fully supported  
✅ **CentOS/RHEL 8+** - Supported  
✅ **Arch Linux** - Community supported  

### **macOS**

✅ **macOS 12+ (Monterey)** - Development only, not recommended for production validators

### **Windows**

❌ **Not recommended** - Use Ubuntu via WSL2 for development

---

## 🌐 Network Requirements

### **Bandwidth**

| Node Type | Download | Upload | Monthly Data |
|-----------|----------|--------|--------------|
| Validator | 50 Mbps | 50 Mbps | ~500 GB |
| Full Node | 100 Mbps | 50 Mbps | ~1 TB |
| Archive | 200 Mbps | 100 Mbps | ~2 TB |

### **Ports**

| Port | Protocol | Purpose |
|------|----------|---------|
| 30333 | TCP | P2P (peer discovery) |
| 9933 | TCP | HTTP RPC (optional, local only) |
| 9944 | TCP | WebSocket RPC (optional, local only) |
| 9615 | TCP | Prometheus metrics (optional) |

**Firewall Rules**:
- Port 30333: **Must be open** (incoming and outgoing)
- Ports 9933, 9944: **Should be firewalled** (local only, use reverse proxy for public access)
- Port 9615: **Optional**, open only for monitoring servers

---

## 📊 Storage Growth

**Expected disk usage growth**:
- **Pruned node**: ~5 GB/month (keeps last 256 blocks)
- **Full node**: ~50 GB/month (full state, recent blocks)
- **Archive node**: ~100 GB/month (complete history)

**Calculation** (2025-10-14):
- Genesis: October 2024
- Current age: ~1 year
- Current size: ~600 GB (archive), ~200 GB (pruned)
- By October 2026: ~1.8 TB (archive), ~320 GB (pruned)

---

## 🔌 Power & Internet

### **Power Requirements**

| Component | Power Draw |
|-----------|------------|
| Validator (recommended) | ~200W average |
| Validator (production) | ~400W average |
| UPS (recommended) | 1500VA (900W) |

**Uptime Considerations**:
- Validators should have **UPS backup** (15-30 min runtime)
- Internet should have **redundant connection** (4G/5G failover)

### **Internet Reliability**

**Minimum uptime**: 95%  
**Recommended uptime**: 99%+  
**Production uptime**: 99.9%+ (consider redundant ISPs)

**Downtime consequences**:
- < 1 hour: No penalty
- 1-6 hours: Minor slash (0.1% stake)
- > 6 hours: Major slash (1% stake)
- > 24 hours: Forced unbond

---

## 🐳 Docker Requirements

If running via Docker:

```bash
# Docker version
docker --version  # 20.10+

# Docker Compose version
docker-compose --version  # 2.0+

# Check resources
docker system df
docker stats
```

**Recommended Docker settings**:
- **CPU limit**: No limit (use host CPUs)
- **Memory limit**: 16 GB minimum
- **Storage driver**: overlay2
- **Logging**: json-file with rotation

---

## ☁️ Cloud Provider Recommendations

### **AWS**

| Node Type | Instance Type | Monthly Cost |
|-----------|---------------|--------------|
| Validator | t3.xlarge or m5.xlarge | ~$150-200 |
| Validator (production) | m5.2xlarge | ~$280 |
| Archive | m5.2xlarge + 2TB EBS | ~$400 |

### **Google Cloud**

| Node Type | Instance Type | Monthly Cost |
|-----------|---------------|--------------|
| Validator | n2-standard-4 | ~$140 |
| Validator (production) | n2-standard-8 | ~$280 |

### **Digital Ocean**

| Node Type | Droplet | Monthly Cost |
|-----------|---------|--------------|
| Validator | 4 vCPU, 8 GB RAM | ~$60 |
| Validator (production) | 8 vCPU, 16 GB RAM | ~$120 |

### **Contabo** (Budget option)

| Node Type | VPS | Monthly Cost |
|-----------|-----|--------------|
| Validator | VPS L | ~$20 |
| Validator (production) | VPS XL | ~$40 |

**Note**: Budget providers may have lower network quality. Test latency before committing.

---

## 🔍 Hardware Recommendations

### **CPU**

**Best CPUs for validation** (price/performance):
1. AMD Ryzen 9 5950X (16 cores) - $550
2. AMD Ryzen 7 5800X (8 cores) - $300
3. Intel Core i7-12700K (12 cores) - $350

**Key factors**:
- Single-thread performance matters (block authoring)
- More cores help with parallel verification
- AES-NI support required (encryption)

### **RAM**

**Recommended**:
- **Type**: DDR4-3200 or faster
- **Size**: 16 GB minimum, 32 GB recommended
- **ECC**: Recommended for production validators

### **Storage**

**SSD Selection**:
- **Type**: NVMe M.2 (fastest)
- **Endurance**: 600+ TBW (terabytes written)
- **IOPS**: 50,000+ read, 30,000+ write

**Recommended SSDs**:
1. Samsung 980 Pro (1TB) - $150
2. WD Black SN850 (1TB) - $130
3. Crucial P5 Plus (1TB) - $110

**Not recommended**:
- ❌ QLC NAND SSDs (low endurance)
- ❌ SATA SSDs (too slow for validators)
- ❌ Hard drives (unusable for blockchain)

---

## 📈 Scalability

### **Vertical Scaling** (upgrade existing node)

Easy upgrades:
- **RAM**: Add more DIMMs (30 min)
- **Storage**: Add NVMe drive (1 hour)

Difficult upgrades:
- **CPU**: Requires motherboard change (4+ hours downtime)

### **Horizontal Scaling** (add nodes)

**Multiple validators**:
- Each validator requires separate infrastructure
- No shared resources (full independence)

**Load-balanced RPC**:
- Multiple RPC nodes behind load balancer
- Share read load, no writes

---

## ✅ Pre-Installation Checklist

Before installing BelizeChain:

**Hardware**:
- [ ] CPU meets minimum requirements
- [ ] RAM meets minimum requirements
- [ ] SSD meets minimum requirements
- [ ] UPS installed (validators only)

**Network**:
- [ ] Internet speed tested (speedtest.net)
- [ ] Port 30333 open and forwarded
- [ ] Backup internet connection configured (optional)

**Software**:
- [ ] Ubuntu 22.04 LTS installed
- [ ] System fully updated
- [ ] Firewall configured
- [ ] SSH access secured

**Monitoring**:
- [ ] Monitoring solution chosen
- [ ] Alert system configured
- [ ] Backup strategy planned

---

## 🚀 Next Steps

**Ready to install?**

1. **[Installation Guide →](installation.md)**  
   Step-by-step installation instructions

2. **[Configuration →](configuration.md)**  
   Configure your node

3. **[Validator Setup →](validator-setup.md)**  
   Become a validator

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) for deployment support! 🚀🇧🇿
