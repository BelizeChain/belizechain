# BelizeChain Runtime

Substrate FRAME v42 runtime implementation for BelizeChain sovereign blockchain.

## Overview

The BelizeChain runtime is the core state transition function that defines:
- **Blockchain logic**: How transactions modify state
- **Consensus rules**: Block validation and finality
- **Pallets integration**: 13 custom pallets + 8 system pallets
- **Governance mechanisms**: Democratic decision-making
- **Economic model**: DALLA/bBZD dual-token system

## Architecture

### Runtime Components

```
BelizeChain Runtime
├── System Pallets (8)
│   ├── System - Core blockchain functionality
│   ├── Timestamp - Block timestamps
│   ├── Aura - Block production (Authority Round)
│   ├── Grandpa - Block finalization
│   ├── Balances - Account balances
│   ├── TransactionPayment - Transaction fees
│   ├── Sudo - Superuser access (testnet only)
│   └── RandomnessCollectiveFlip - On-chain randomness
│
└── Custom Pallets (13)
    ├── Economy - DALLA/bBZD tokens + treasury
    ├── Identity - BelizeID + KYC/AML
    ├── Governance - Democratic governance
    ├── Compliance - Regulatory compliance
    ├── Staking - PoUW consensus + validator rewards
    ├── Oracle - Price feeds + merchant verification
    ├── Payroll - Government/private payroll
    ├── Interoperability - Cross-chain bridges
    ├── BelizeX - On-chain DEX
    ├── LandLedger - Property registry
    ├── Consensus - PoUW implementation
    ├── Quantum - Quantum workload integration
    └── Community - Community governance
```

## Key Features

### 1. Multi-Currency System
- **DALLA**: Native token (12 decimals, max 501B supply)
- **bBZD**: BZD-pegged stablecoin (1:1 Belize Dollar), peg target set by governance
### 2. National Identity System (BelizeID)
- SSN integration with validation
- Passport verification
- KYC/AML compliance
- Privacy-preserving credentials

### 3. Proof of Useful Work (PoUW)
- Validators rewarded for federated learning contributions
- Quality (40%) + Timeliness (30%) + Honesty (30%)
- Integration with Nawal AI system

### 4. Quantum Integration
- Proof of Quantum Work (PQW) rewards
- Azure Quantum backend integration
- Quantum-resistant cryptography

### 5. Cross-Pallet Communication
Pallets communicate via provider traits defined after runtime construction:

```rust
// Example: Economy pallet uses governance-controlled BZD peg target
pub struct EconomyOracleProvider;
impl pallet_belize_economy::OracleProvider<AccountId> for EconomyOracleProvider {
    fn get_exchange_rate() -> Option<u128> {
        // Peg target is stored in Economy pallet configuration
        // Oracle feeds are used for auxiliary pricing (FX), not for the peg itself
        None
    }
}
```

## Runtime Configuration

### Block Production
- **Block Time**: 6 seconds (Aura consensus)
- **Max Block Weight**: 2,000,000,000,000 (2 seconds of ref time)
- **Max Block Length**: 5 * 1024 * 1024 (5 MB)

### Transaction Fees
- **Transaction Base Fee**: 1,000,000 (0.000001 DALLA)
- **Transaction Byte Fee**: 10,000 (0.00000001 DALLA per byte)
- **Weight-to-Fee**: 1 weight unit = 1 fee unit

### Treasury
- **Multi-signature**: 4-of-7 required for treasury operations
- **Inflation**: 2% annual, minted to treasury
- **Burn mechanisms**: User-initiated and governance-controlled

## Building the Runtime

### Development Build
```bash
# Fast compilation (no optimization)
cargo build -p belizechain-runtime
```

### Release Build
```bash
# Optimized WASM runtime (production)
cargo build --release -p belizechain-runtime
```

### WASM Binary
The runtime is compiled to WebAssembly for on-chain execution:
- Location: `target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm`
- Size limit: ~2 MB for efficient on-chain storage
- Compression: Enabled for smaller binary size

## Runtime Upgrades

### Forkless Upgrades
BelizeChain supports forkless runtime upgrades via:
1. **Build new runtime**: `cargo build --release -p belizechain-runtime`
2. **Submit upgrade proposal**: Via governance or sudo
3. **Automatic activation**: After approval and delay period

### Version Management
```rust
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("belizechain"),
    impl_name: create_runtime_str!("belizechain"),
    authoring_version: 1,
    spec_version: 100,  // Increment for breaking changes
    impl_version: 1,    // Increment for non-breaking changes
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};
```

## Testing

### Unit Tests
```bash
# Test runtime logic
cargo test -p belizechain-runtime
```

### Runtime API Tests
```bash
# Test runtime APIs
cargo test -p belizechain-runtime --features runtime-benchmarks
```

## Weight Calculation

All extrinsics require precise weight calculations:
```rust
impl<T: Config> WeightInfo for SubstrateWeight<T> {
    fn function_name() -> Weight {
        Weight::from_parts(25_000_000, 0)  // Computational weight
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
```

## Storage Optimization

### BoundedVec Usage
All storage items use `BoundedVec` for size limits:
```rust
pub name: BoundedVec<u8, ConstU32<64>>,  // Max 64 bytes
```

### Storage Keys
- Well-structured naming for easy querying
- Prefix isolation to prevent collisions
- Double-map for efficient lookups

## Security Features

1. **Multi-signature Treasury**: 4-of-7 approval required
2. **Rate Limiting**: DoS attack prevention
3. **Emergency Shutdown**: Governance-controlled circuit breakers
4. **Compliance Integration**: KYC/AML checks across pallets
5. **Audit Trails**: Comprehensive event logs

## Integration Points

### Nawal AI (Federated Learning)
- Staking pallet receives training results
- PoUW rewards based on contribution quality
- Privacy-preserving aggregation

### Kinich Quantum
- Consensus pallet accepts PQW proofs
- Quantum workload verification
- Rewards for quantum computations

### Pakit Storage
- LandLedger stores document proofs
- IPFS/Arweave content addressing
- On-chain proof verification

## Performance Characteristics

- **TPS**: ~100-200 transactions per second
- **Finality**: ~30 seconds (5 blocks with Grandpa)
- **Storage Growth**: ~1-2 GB per million transactions
- **Memory Usage**: ~500 MB for full node

## Migration from Previous Versions

### Polkadot SDK stable2509 Migration
Key changes in current version:
- ✅ Removed `RuntimeEvent` from Config traits (auto-appended)
- ✅ Migrated to `WasmExecutor` from deprecated `NativeElseWasmExecutor`
- ✅ Added `sp-io` dependency for substrate host functions
- ✅ Updated all pallets to FRAME v42 compatibility

## File Structure

```
runtime/
├── Cargo.toml          # Runtime dependencies
├── build.rs            # Build script for WASM compilation
└── src/
    └── lib.rs          # Main runtime construction (1132 lines)
```

## Documentation

- **Inline docs**: Run `cargo doc --open -p belizechain-runtime`
- **Architecture**: See `/docs/architecture/`
- **Pallet docs**: See individual pallet README files

## Contributing

When modifying the runtime:
1. Update `spec_version` for breaking changes
2. Update `impl_version` for non-breaking changes
3. Run all tests: `cargo test -p belizechain-runtime`
4. Update weight calculations if extrinsics change
5. Document storage migrations if needed

## Resources

- **Substrate Docs**: https://docs.substrate.io
- **FRAME Pallets**: https://docs.substrate.io/reference/frame-pallets/
- **Runtime Development**: https://docs.substrate.io/build/runtime-development/

---

**Note**: This runtime is production-ready and has been migrated to Polkadot SDK stable2509 (FRAME v42).
