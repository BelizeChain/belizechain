# BelizeChain Scripts

Production-ready automation scripts for building, testing, and deploying BelizeChain testnet.

## Directory Structure

```
scripts/
├── build_release.sh               # Production build (optimized, WASM included)
├── start_testnet.sh               # Start local testnet node (dev/validator/archive)
├── run_all_tests.sh               # Complete test suite (Rust + Python)
├── audit_runner.sh                # Security audit (bandit + cargo-audit + safety)
├── quick_audit.sh                 # Fast security scan
├── verify_security.sh             # Security verification
├── check_build_deps.sh            # Build dependency verification
├── monitor_polkadot_sdk.sh        # Track Polkadot SDK updates
├── deploy/
│   ├── deploy_testnet.sh          # Testnet deployment automation
│   └── validate_env.sh            # Environment validation
├── test/
│   └── quick_test.sh              # Quick test runner
└── testing/
    ├── check_pallets.sh           # Individual pallet compilation checks
    ├── load_test.py               # Load testing
    └── run_integration_tests.sh   # Integration test runner
```

## Core Scripts

### 🔨 Build & Development

#### `build_release.sh`
Production build with full optimizations and WASM runtime.

```bash
./scripts/build_release.sh
```

**Output**: `./target/release/belizechain-node` (optimized binary)

---

#### `start_testnet.sh`
Start local testnet node in different modes.

```bash
# Development mode (temporary, Alice sudo, unsafe RPC)
./scripts/start_testnet.sh dev

# Validator mode (persistent, production)
./scripts/start_testnet.sh validator

# Archive mode (full history, for explorers)
./scripts/start_testnet.sh archive
```

**Endpoints**:
- RPC: `http://localhost:9933`
- WebSocket: `ws://localhost:9944`
- Prometheus: `http://localhost:9615/metrics`

---

### 🧪 Testing

#### `run_all_tests.sh`
Complete test suite: Rust unit tests + Python integration tests.

```bash
./scripts/run_all_tests.sh
```

**Runs**:
1. Rust unit tests (`cargo test --workspace`)
2. Starts testnet node
3. Python integration tests (all categories)
4. Cleanup

**Exit Codes**:
- `0`: All tests passed
- `1`: Some tests failed

---

#### `testing/run_integration_tests.sh`
Run specific integration test categories.

```bash
# All tests
./scripts/testing/run_integration_tests.sh all

# Specific categories
./scripts/testing/run_integration_tests.sh blockchain      # Core pallet coverage
./scripts/testing/run_integration_tests.sh cross-pallet    # Cross-pallet integration
./scripts/testing/run_integration_tests.sh governance      # Governance system
./scripts/testing/run_integration_tests.sh economic        # DALLA/bBZD
./scripts/testing/run_integration_tests.sh belizex         # DEX
./scripts/testing/run_integration_tests.sh e2e             # End-to-end scenarios
```

**Prerequisites**: Testnet node running at `localhost:9933`

---

#### `testing/check_pallets.sh`
Check individual pallet compilation (fast validation).

```bash
./scripts/testing/check_pallets.sh
```

Verifies all 16 custom pallets compile without errors.

---

### 🔒 Security

#### `audit_runner.sh`
Comprehensive security audit.

```bash
./scripts/audit_runner.sh
```

**Checks**:
- Python: `bandit` (code scanning) + `safety` (dependency vulnerabilities)
- Rust: `cargo-audit` (dependency vulnerabilities)

**Output**: `audit_results/` directory with detailed reports

---

#### `quick_audit.sh`
Fast security scan (subset of full audit).

```bash
./scripts/quick_audit.sh
```

Runs critical security checks in <1 minute.

---

#### `verify_security.sh`
Security verification for production readiness.

```bash
./scripts/verify_security.sh
```

Validates security compliance before deployment.

---

### 🚀 Deployment

#### `deploy/deploy_testnet.sh`
Automated testnet deployment.

```bash
./scripts/deploy/deploy_testnet.sh
```

**Steps**:
1. Validates environment (`validate_env.sh`)
2. Runs all tests (`run_all_tests.sh`)
3. Creates deployment package
4. Generates systemd service file
5. Creates `DEPLOY.md` instructions
6. Archives as `.tar.gz`

**Output**: `deploy-YYYYMMDD-HHMMSS.tar.gz` (ready for server upload)

---

#### `deploy/validate_env.sh`
Environment validation for deployment.

```bash
./scripts/deploy/validate_env.sh
```

**Checks**:
- System resources (CPU, RAM, storage)
- Required ports availability
- Network connectivity
- Dependencies installed

---

### 📊 Monitoring & Maintenance

#### `monitor_polkadot_sdk.sh`
Track Polkadot SDK updates (stable releases).

```bash
./scripts/monitor_polkadot_sdk.sh
```

Checks for new Substrate/Polkadot SDK versions.

---

#### `check_build_deps.sh`
Verify build dependencies.

```bash
./scripts/check_build_deps.sh
```

**Checks**:
- Rust toolchain (stable)
- wasm32 target
- Required system libraries
- Python 3.13+
- Node.js 18+

---

## Common Workflows

### 🏗️ Initial Setup
```bash
# 1. Check dependencies
./scripts/check_build_deps.sh

# 2. Build production binary
./scripts/build_release.sh

# 3. Run security audit
./scripts/audit_runner.sh

# 4. Run all tests
./scripts/run_all_tests.sh
```

### 🧪 Testing Workflow
```bash
# 1. Start testnet node
./scripts/start_testnet.sh dev

# 2. In another terminal: Run tests
./scripts/testing/run_integration_tests.sh all

# 3. Quick pallet check
./scripts/testing/check_pallets.sh
```

### 🚀 Deployment Workflow
```bash
# 1. Security audit
./scripts/quick_audit.sh

# 2. Run all tests
./scripts/run_all_tests.sh

# 3. Create deployment package
./scripts/deploy/deploy_testnet.sh

# 4. Transfer to server
scp deploy-*.tar.gz user@server:/tmp/

# 5. Follow DEPLOY.md instructions
```

### 🔍 Debugging
```bash
# Check individual pallet compilation
./scripts/testing/check_pallets.sh

# Run specific test category
./scripts/testing/run_integration_tests.sh governance

# View node logs
tail -f /tmp/belizechain-node.log
```

## Environment Variables

### Build Configuration
```bash
# Cargo build parallelism
export CARGO_BUILD_JOBS=4

# Target triple (default: native)
export CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu
```

### Runtime Configuration
```bash
# Blockchain endpoints
export BLOCKCHAIN_RPC_URL=http://localhost:9933
export BLOCKCHAIN_WS_URL=ws://localhost:9944

# Test mode
export RUST_LOG=info
export PYTHONPATH=$(pwd):$PYTHONPATH
```

## Testnet Deployment Requirements

### Hardware
- **CPU**: 4+ cores (8+ recommended)
- **RAM**: 8 GB minimum (16 GB recommended)
- **Storage**: 100 GB SSD (500 GB recommended for archive)
- **Network**: 100 Mbps (1 Gbps recommended)

### Software
- **OS**: Ubuntu 22.04 LTS or newer
- **Rust**: stable toolchain (1.75+)
- **Python**: 3.13+
- **Node.js**: 18+ (for UI portals)
- **Docker**: 24+ (optional, for containerized deployment)

### Network Ports
- `30333`: P2P communication (must be open)
- `9933`: RPC endpoint (optional, firewall for security)
- `9944`: WebSocket endpoint (optional, firewall for security)
- `9615`: Prometheus metrics (optional, internal only)

## Troubleshooting

### Build Failures
```bash
# Clean and rebuild
cargo clean
./scripts/build_release.sh

# Check dependencies
./scripts/check_build_deps.sh
```

### Test Failures
```bash
# Check if node is running
curl http://localhost:9933

# Start node manually
./scripts/start_testnet.sh dev

# Run specific test category
./scripts/testing/run_integration_tests.sh blockchain
```

### Deployment Issues
```bash
# Validate environment
./scripts/deploy/validate_env.sh

# Check node health
systemctl status belizechain-testnet
journalctl -u belizechain-testnet -f
```

## Security Best Practices

1. **Always run audits before deployment**:
   ```bash
   ./scripts/audit_runner.sh
   ```

2. **Verify test coverage**:
   ```bash
   ./scripts/run_all_tests.sh
   ```

3. **Use validator mode for production**:
   ```bash
   ./scripts/start_testnet.sh validator
   ```

4. **Enable firewall on production servers**:
   - Only open P2P port (30333)
   - Restrict RPC/WS to internal network
   - Monitor Prometheus metrics internally

5. **Regular security updates**:
   ```bash
   ./scripts/monitor_polkadot_sdk.sh
   cargo update
   ./scripts/audit_runner.sh
   ```

## Contributing

When adding new scripts:
1. Add `#!/bin/bash` shebang
2. Use `set -e` for error handling
3. Include help text (`--help` flag)
4. Make executable: `chmod +x script.sh`
5. Document in this README
6. Test thoroughly before committing

## Resources

- [BelizeChain Documentation](https://docs.belizechain.org)
- [Substrate Documentation](https://docs.substrate.io)
- [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)
- [Deployment Guide](../docs/deployment/)

---

**Last Updated**: January 31, 2026  
**BelizeChain Version**: stable2603
