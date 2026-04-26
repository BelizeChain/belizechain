# 🔗 Blockchain Core Integration Tests

## Purpose
Tests for BelizeChain's core blockchain functionality:
- Pallet integration (13 custom pallets)
- Runtime operations
- Cross-pallet dependencies
- Node RPC/WebSocket connectivity

## Test Categories

### 1. Pallet Integration Tests
- Economy pallet (DALLA/bBZD tokens, multi-sig treasury)
- Identity pallet (BelizeID, KYC verification)
- Governance pallet (district councils, democracy)
- Compliance pallet (KYC/AML, FSC oversight)
- Oracle pallet (price feeds, merchant verification)
- BelizeX pallet (DEX, asset registry)
- LandLedger pallet (property registry)

### 2. Cross-Pallet Interaction Tests
- Oracle → Economy (exchange rate feeds)
- Governance → Economy (treasury proposals)
- Compliance → All pallets (KYC checks)
- Identity → Governance (voter eligibility)

### 3. Runtime Tests
- Transaction processing
- Block production
- Event emission
- Storage queries

## Running Tests

```bash
# All blockchain tests
pytest tests/integration/blockchain/ -v

# Specific test file
pytest tests/integration/blockchain/test_pallets.py -v

# Individual test
pytest tests/integration/blockchain/test_pallets.py::test_economy_token_transfer -v
```

## Prerequisites
```bash
# Start BelizeChain node
./target/release/belizechain-node --chain=local --tmp --alice
```

## Test Files (To Be Created)
- `test_pallets.py` - Individual pallet functionality
- `test_cross_pallet.py` - Pallet interactions
- `test_runtime.py` - Runtime operations
- `test_rpc.py` - RPC endpoint validation
