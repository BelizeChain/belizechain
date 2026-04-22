# Troubleshooting Guide

Common issues, diagnostic procedures, and solutions for BelizeChain development and operations.

## Node Operation Issues

### 1. Node Fails to Start

**Symptoms**:
```
Error: Failed to open database
thread 'main' panicked at 'Database corrupted'
```

**Diagnosis**:
```bash
# Check database integrity
ls -lh ~/.local/share/belizechain/chains/belizechain/db/

# Check disk space
df -h

# Check permissions
ls -la ~/.local/share/belizechain/
```

**Solutions**:

```bash
# Solution 1: Clear database and resync
rm -rf ~/.local/share/belizechain/chains/belizechain/db/
./target/release/belizechain-node --chain dev --tmp

# Solution 2: Use archived state snapshot
wget https://snapshots.belizechain.org/latest.tar.gz
tar -xzf latest.tar.gz -C ~/.local/share/belizechain/chains/belizechain/

# Solution 3: Fix permissions
chmod -R 755 ~/.local/share/belizechain/
chown -R $USER:$USER ~/.local/share/belizechain/
```

### 2. Peer Connection Problems

**Symptoms**:
```
No peers connected
Peer discovery timeout after 60s
```

**Diagnosis**:
```bash
# Check if ports are open
netstat -tuln | grep -E '30333|9933|9944'

# Test P2P connectivity
nc -zv rpc.belizechain.org 30333

# Check firewall rules
sudo ufw status
```

**Solutions**:

```bash
# Solution 1: Open required ports
sudo ufw allow 30333/tcp  # P2P
sudo ufw allow 9933/tcp   # HTTP-RPC
sudo ufw allow 9944/tcp   # WS-RPC

# Solution 2: Add bootnodes manually
./target/release/belizechain-node \
  --chain belizechain \
  --bootnodes /ip4/142.93.150.23/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
  --bootnodes /ip4/167.99.231.78/tcp/30333/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD

# Solution 3: Check network connectivity
ping rpc.belizechain.org
traceroute rpc.belizechain.org
```

### 3. Slow Block Sync

**Symptoms**:
```
Syncing from genesis... 45 blocks/min (expected 1000+)
```

**Diagnosis**:
```bash
# Check sync status
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933

# Check peer count
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers"}' \
  http://localhost:9933 | jq '.result | length'

# Check system resources
htop
iostat -x 1
```

**Solutions**:

```bash
# Solution 1: Use warp sync (fast sync)
./target/release/belizechain-node \
  --chain belizechain \
  --sync warp

# Solution 2: Download state snapshot
wget https://snapshots.belizechain.org/belizechain-state-latest.tar.gz
tar -xzf belizechain-state-latest.tar.gz
./target/release/belizechain-node --chain belizechain

# Solution 3: Increase database cache
./target/release/belizechain-node \
  --chain belizechain \
  --db-cache 4096  # 4GB cache (default 128MB)

# Solution 4: Optimize RocksDB
./target/release/belizechain-node \
  --chain belizechain \
  --state-pruning archive-canonical \
  --blocks-pruning 256
```

### 4. Session Key Errors

**Symptoms**:
```
Error: Session keys not set
ImOnline pallet heartbeat failed
```

**Diagnosis**:
```bash
# Check if session keys exist
ls ~/.local/share/belizechain/chains/belizechain/keystore/

# Verify keys via RPC
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_hasSessionKeys", "params": ["0x..."]}' \
  http://localhost:9933
```

**Solutions**:

```bash
# Solution 1: Generate new session keys
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys"}' \
  http://localhost:9933

# Response: "0x1234abcd..." (new session keys)

# Solution 2: Set session keys on-chain
polkadot-js-api \
  --ws ws://localhost:9944 \
  tx.session.setKeys \
  --args '{"keys": "0x1234abcd...", "proof": "0x00"}' \
  --seed "//YourValidatorSeed"

# Solution 3: Restore from backup
cp ~/session-keys-backup/* ~/.local/share/belizechain/chains/belizechain/keystore/
./target/release/belizechain-node --chain belizechain --validator
```

## Compilation Errors

### 1. Missing Dependencies

**Error**:
```
error: linking with `cc` failed
/usr/bin/ld: cannot find -lrocksdb
```

**Solution**:
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential git clang curl libssl-dev llvm libudev-dev pkg-config

# macOS
brew install openssl cmake llvm

# Verify installations
clang --version
cmake --version
```

### 2. Rust Version Issues

**Error**:
```
error: package `frame-support v4.0.0` cannot be built because it requires rustc 1.75.0 or newer
```

**Solution**:
```bash
# Update Rust
rustup update stable
rustup update nightly

# Verify version
rustc --version  # Should be 1.75.0+

# Set correct toolchain
rustup default stable

# For ink! development
rustup component add rust-src --toolchain stable
rustup target add wasm32-unknown-unknown --toolchain stable
```

### 3. WebAssembly Build Errors

**Error**:
```
error: failed to build runtime WASM
target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.wasm not found
```

**Solution**:
```bash
# Install wasm-gc
cargo install wasm-gc

# Clear build cache
cargo clean

# Rebuild with verbose output
cargo build --release -vv

# Check WASM target
rustup target list --installed | grep wasm32

# Add if missing
rustup target add wasm32-unknown-unknown
```

### 4. Cargo.lock Conflicts

**Error**:
```
error: failed to select a version for `sp-io`
required by package `pallet-belize-economy`
```

**Solution**:
```bash
# Update dependencies
cargo update

# Or force specific version
cargo update -p sp-io --precise 31.0.0

# Rebuild
cargo build --release

# If still failing, regenerate Cargo.lock
rm Cargo.lock
cargo generate-lockfile
cargo build --release
```

## Transaction Failures

### 1. Insufficient Balance

**Error**:
```json
{
  "error": {
    "code": 1001,
    "message": "Insufficient balance"
  }
}
```

**Diagnosis**:
```javascript
// Check account balance
const { data: balance } = await api.query.system.account(address);
console.log('Free:', balance.free.toString());
console.log('Reserved:', balance.reserved.toString());
console.log('Frozen:', balance.frozen.toString());

// Check transaction fee
const info = await api.tx.economy.transfer(to, amount).paymentInfo(from);
console.log('Estimated fee:', info.partialFee.toString());

// Calculate required balance
const required = amount + info.partialFee;
console.log('Required:', required.toString());
console.log('Available:', balance.free.toString());
```

**Solution**:
```javascript
// Top up account
const faucet = await api.tx.economy.transfer(
  insufficientAccount,
  1_000_000_000_000_000n  // 1,000 DALLA
);
await faucet.signAndSend(fundedAccount);
```

### 2. Nonce Errors

**Error**:
```
Error: Transaction nonce 15 is lower than account nonce 16
```

**Diagnosis**:
```javascript
// Check current nonce
const nonce = await api.rpc.system.accountNextIndex(address);
console.log('Current nonce:', nonce.toString());

// Check pending transactions
const pending = await api.rpc.author.pendingExtrinsics();
console.log('Pending txs:', pending.length);
```

**Solution**:
```javascript
// Solution 1: Let SDK manage nonce automatically
await api.tx.economy.transfer(to, amount).signAndSend(keypair);

// Solution 2: Manually specify nonce
const nonce = await api.rpc.system.accountNextIndex(address);
await api.tx.economy.transfer(to, amount).signAndSend(keypair, { nonce });

// Solution 3: Wait for pending txs to finalize
await new Promise(resolve => setTimeout(resolve, 12000)); // Wait 12s for finality
```

### 3. KYC Requirements

**Error**:
```json
{
  "error": {
    "code": 1010,
    "message": "KYC verification required"
  }
}
```

**Diagnosis**:
```javascript
// Check KYC level
const identity = await api.query.identity.identityOf(address);
console.log('KYC Level:', identity.unwrap().kyc_level.toString());
// Output: Basic | Verified | Enhanced

// Check required level for operation
const required = await api.query.economy.kycRequirement('transfer_large');
console.log('Required KYC:', required.toString());
```

**Solution**:
```javascript
// Upgrade KYC level
const updateKyc = api.tx.identity.updateKyc({
  level: 'Verified',
  documents: [documentHash1, documentHash2],
  biometrics: null  // Only for Enhanced
});

await updateKyc.signAndSend(keypair);

// Wait for FSC approval (2-5 days)
```

## Staking Issues

### 1. Bonding Fails

**Error**:
```
Error: AlreadyBonded
```

**Diagnosis**:
```javascript
const staking = await api.query.staking.bonded(address);
console.log('Already bonded:', staking.isSome);

const ledger = await api.query.staking.ledger(address);
console.log('Bonded amount:', ledger.unwrap().active.toString());
```

**Solution**:
```javascript
// Solution 1: Unbond first
await api.tx.staking.unbond(ledger.unwrap().active).signAndSend(keypair);

// Wait 14 days unbonding period

await api.tx.staking.withdrawUnbonded(0).signAndSend(keypair);

// Solution 2: Bond additional amount
await api.tx.staking.bondExtra(additionalAmount).signAndSend(keypair);
```

### 2. Reward Claim Fails

**Error**:
```
Error: No rewards pending
```

**Diagnosis**:
```javascript
// Check pending rewards
const rewards = await api.query.staking.payee(address);
console.log('Reward destination:', rewards.toString());

const ledger = await api.query.staking.ledger(address);
const lastRewardEra = ledger.unwrap().claimed_rewards;
console.log('Last claimed era:', lastRewardEra.toString());

const currentEra = await api.query.staking.currentEra();
console.log('Current era:', currentEra.toString());
```

**Solution**:
```javascript
// Calculate claimable eras
const claimableEras = [];
for (let era = lastRewardEra + 1; era < currentEra; era++) {
  claimableEras.push(era);
}

// Claim all pending rewards
for (const era of claimableEras) {
  await api.tx.staking.payoutStakers(validatorAddress, era).signAndSend(keypair);
}
```

### 3. PoUW Submission Errors

**Error**:
```
Error: Invalid PoUW proof
```

**Diagnosis**:
```bash
# Check Nawal node connectivity
curl http://localhost:8001/health
# Response: {"status": "healthy", "connected": true}

# Check training round status
curl http://localhost:8001/rounds/current
# Response: {"round_id": 42, "status": "collecting", "deadline": 1738435680}

# Verify gradient submission
cat ~/.nawal/submissions/round_42.json
```

**Solution**:

```python
# Solution 1: Verify gradient integrity
from nawal.client.trainer import FederatedTrainer

trainer = FederatedTrainer(validator_id='5GrwvaEF...')
gradients = trainer.compute_gradients()

# Check gradient norm (should be < max_grad_norm)
import torch
grad_norm = torch.norm(gradients).item()
print(f"Gradient norm: {grad_norm}")  # Should be ~1.0

if grad_norm > 10.0:
    print("ERROR: Gradient explosion detected")
    # Reduce learning rate or increase batch size

# Solution 2: Resubmit with correct proof
from nawal.blockchain.staking_connector import StakingConnector

connector = StakingConnector(ws_endpoint='wss://rpc.belizechain.org')
tx_hash = await connector.report_training(
    validator_id='5GrwvaEF...',
    round_id=42,
    encrypted_gradients=encrypted_gradients,
    privacy_proof={
        'epsilon_spent': 0.0987,
        'delta': 1e-5,
        'gradient_norm': grad_norm
    },
    timestamp=current_block_number
)
```

## Smart Contract Issues

### 1. Deployment Fails

**Error**:
```
Error: CodeTooLarge
```

**Diagnosis**:
```bash
# Check contract size
ls -lh target/ink/my_contract.wasm
# Should be < 128 KB

# Analyze binary
wasm-opt --print-size target/ink/my_contract.wasm
```

**Solution**:
```bash
# Solution 1: Optimize WASM
wasm-opt -Oz target/ink/my_contract.wasm -o my_contract_optimized.wasm

# Solution 2: Enable LTO and strip debug
# In Cargo.toml:
[profile.release]
lto = true
opt-level = "z"
strip = true

# Rebuild
cargo contract build --release

# Solution 3: Reduce code size
# Remove unused dependencies, inline small functions, use const generics
```

### 2. Gas Estimation Errors

**Error**:
```
Error: OutOfGas
```

**Diagnosis**:
```bash
# Estimate gas for call
cargo contract call \
  --contract 5GrwvaEF... \
  --message transfer \
  --args "5DTestU... 1000000000000" \
  --dry-run \
  --suri //Alice

# Output: Estimated gas: 250000000 (0.25 DALLA)
```

**Solution**:
```bash
# Solution 1: Increase gas limit
cargo contract call \
  --contract 5GrwvaEF... \
  --message transfer \
  --args "5DTestU... 1000000000000" \
  --gas 500000000 \  # 2x estimated
  --suri //Alice

# Solution 2: Optimize contract
# Reduce storage reads/writes, batch operations
```

## Quantum/Nawal Issues

### 1. Quantum Backend Connection Failed

**Error**:
```
Error: Unable to connect to configured quantum backend
```

**Diagnosis**:
```bash
# Check Kinich backend configuration
kinich-cli config --show

# Validate required environment variables
env | grep -E "KINICH_BACKEND|KINICH_API_KEY"

# Check Kinich node logs
tail -f ~/.kinich/logs/quantum_node.log
```

**Solution**:
```bash
# Solution 1: Reconfigure preferred backend
kinich-cli config --backend-preference ionq

# Solution 2: Switch to fallback backend
kinich-cli config --backend-preference ibm

# Solution 3: Use simulator mode during outage
kinich-cli config --backend-preference simulator

# Verify
kinich-cli test-connection
```

### 2. Federated Learning Timeout

**Error**:
```
Error: Training round timeout after 12 hours
```

**Diagnosis**:
```python
# Check training progress
from nawal.client.trainer import FederatedTrainer

trainer = FederatedTrainer(validator_id='5GrwvaEF...')
status = trainer.get_training_status(round_id=42)

print(f"Epoch: {status.current_epoch}/{status.total_epochs}")
print(f"Batch: {status.current_batch}/{status.total_batches}")
print(f"Time remaining: {status.time_remaining}s")
```

**Solution**:
```python
# Solution 1: Reduce epochs per round
from nawal.server.aggregator import FederatedAggregator

aggregator = FederatedAggregator(
    epochs_per_round=2,  # Reduced from 3
    batch_size=64        # Increased from 32 (faster training)
)

# Solution 2: Increase deadline
training_config = {
    'deadline': block_number + 10800  # 18 hours (was 12)
}

# Solution 3: Use GPU if available
trainer = FederatedTrainer(
    device='cuda',  # Use GPU
    validator_id='5GrwvaEF...'
)
```

## Getting Help

### Diagnostic Information

When reporting issues, include:

```bash
# Generate diagnostic report
./scripts/generate_diagnostics.sh > diagnostics.txt

# Contains:
# - Node version
# - Runtime version
# - System specs (CPU, RAM, disk)
# - Peer count
# - Sync status
# - Recent logs
# - Configuration files
```

### Support Channels

- **GitHub Issues**: https://github.com/belizechain/belizechain/issues
- **Discord**: https://discord.gg/belizechain
- **Stack Overflow**: Tag `belizechain`
- **Email**: support@belizechain.org
- **Emergency**: emergency@belizechain.org (validators only)

### Escalation Procedure

1. **Level 1**: Community Discord/GitHub (response: hours)
2. **Level 2**: Email support (response: 1 business day)
3. **Level 3**: Emergency contact (validators, response: immediate)
