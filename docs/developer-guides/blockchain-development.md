# Blockchain Development Guide

**Building Pallets • Runtime Configuration • Testing • Debugging**

Complete guide to developing and modifying BelizeChain's core blockchain.

---

## Understanding the Architecture

### Substrate-based Stack

BelizeChain uses **Polkadot SDK stable2512** (January 2026 release):

```
┌─────────────────────────────────────┐
│   Client Applications (Maya, Blue  │
│   Hole Portal, GEM SDK)             │
└──────────────┬──────────────────────┘
               │ RPC / WebSocket
┌──────────────▼──────────────────────┐
│   Node (belizechain-node)           │
│   - Network layer                   │
│   - RPC endpoints                   │
│   - Database (RocksDB)              │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│   Runtime (WASM + Native)           │
│   - 16 Custom Pallets               │
│   - 8 System Pallets                │
│   - Consensus (BABE + GRANDPA)       │
└─────────────────────────────────────┘
```

### Directory Structure

```
belizechain/
├── node/                    # Node implementation
│   ├── src/
│   │   ├── service.rs      # Network, database, consensus
│   │   ├── rpc.rs          # RPC endpoint configuration
│   │   └── chain_spec.rs   # Genesis configuration
│   └── Cargo.toml
│
├── runtime/                 # Runtime (on-chain logic)
│   ├── src/
│   │   └── lib.rs          # Runtime construction
│   └── Cargo.toml
│
└── pallets/                 # Custom pallets (15 total)
    ├── economy/            # DALLA/bBZD economy
    ├── identity/           # BelizeID system
    ├── governance/         # Democracy & councils
    ├── compliance/         # KYC/AML
    ├── staking/            # PoUW staking
    ├── oracle/             # Merchant verification
    ├── payroll/            # Payroll management
    ├── interoperability/   # Cross-chain bridges
    ├── belizex/            # DEX (AMM + order book)
    ├── landledger/         # Property registry
    ├── consensus/          # Proof of Useful Work
    ├── quantum/            # Quantum orchestration
    ├── community/          # Community governance
    ├── bns/                # Belize Name Service
    └── contracts/          # Wasm contract execution
```

---

## Running a Development Node

### Start Development Node

```bash
cd belizechain-belizechain/

# Build first (if not already built)
cargo build --release

# Start node in development mode
./target/release/belizechain-node --dev --tmp

# Output:
# 2026-01-31 10:00:00 BelizeChain Node
# 2026-01-31 10:00:00 ✨  version 4.0.0-stable2512
# 2026-01-31 10:00:00 ❤️  by BelizeChain Team
# 2026-01-31 10:00:00 📋 Chain specification: Development
# 2026-01-31 10:00:00 🏷  Node name: fuzzy-donkey-1234
# 2026-01-31 10:00:00 👤 Role: AUTHORITY
# 2026-01-31 10:00:00 💾 Database: RocksDb at /tmp/substrate...
# 2026-01-31 10:00:00 🔨 Initializing Genesis block/state
# 2026-01-31 10:00:00 👶 Creating empty BABE epoch changes
# 2026-01-31 10:00:00 Using default protocol ID "sup" because none is configured
# 2026-01-31 10:00:01 🏷  Local node identity: 12D3KooWQZ9X...
# 2026-01-31 10:00:01 📦 Highest known block at #0
# 2026-01-31 10:00:01 〽️ Prometheus exporter started at 127.0.0.1:9615
# 2026-01-31 10:00:01 Running JSON-RPC server: addr=127.0.0.1:9933
# 2026-01-31 10:00:06 🙌 Starting consensus session on top of parent 0x...
# 2026-01-31 10:00:06 🎁 Prepared block for proposing at 1 (0 ms)
# 2026-01-31 10:00:06 ✨ Imported #1 (0x...)
```

### Command Options

```bash
# Development mode (Alice as validator, temp database)
./target/release/belizechain-node --dev

# Persistent development database
./target/release/belizechain-node --dev --base-path /tmp/belizechain-dev

# Custom RPC/WebSocket ports
./target/release/belizechain-node --dev \
  --rpc-port 9944 \
  --ws-port 9955 \
  --rpc-cors all

# Enable detailed logging
RUST_LOG=debug ./target/release/belizechain-node --dev

# Specific pallet logging
RUST_LOG=pallet_belize_economy=trace ./target/release/belizechain-node --dev
```

### Connect via Polkadot.js Apps

```bash
# 1. Start node (as above)
./target/release/belizechain-node --dev

# 2. Open Polkadot.js Apps in browser
open https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944

# 3. Developer → Chain state → Select pallet/storage query
# Example: economy → totalDallaSupply
```

---

## Creating a New Pallet

### Generate Pallet Template

```bash
cd belizechain-belizechain/pallets/

# Create new pallet directory
cargo new my-pallet --lib
cd my-pallet/
```

### Pallet Structure (Minimal Example)

**File: `pallets/my-pallet/src/lib.rs`**

```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// RuntimeEvent is automatic (no need to declare)
        
        /// Currency for handling balances
        type Currency: Currency<Self::AccountId>;
        
        /// Maximum name length
        #[pallet::constant]
        type MaxNameLength: Get<u32>;
    }
    
    /// Storage: Simple value
    #[pallet::storage]
    #[pallet::getter(fn my_value)]
    pub type MyValue<T> = StorageValue<_, u32, ValueQuery>;
    
    /// Storage: Mapping (key → value)
    #[pallet::storage]
    pub type MyMap<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u8, T::MaxNameLength>  // MUST use BoundedVec for MaxEncodedLen
    >;
    
    /// Events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Value was set
        ValueSet { who: T::AccountId, value: u32 },
        /// Name was stored
        NameStored { who: T::AccountId, name: Vec<u8> },
    }
    
    /// Errors
    #[pallet::error]
    pub enum Error<T> {
        /// Name too long
        NameTooLong,
        /// Value too large
        ValueTooLarge,
    }
    
    /// Extrinsics (functions callable by users)
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set a value
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_value())]
        pub fn set_value(
            origin: OriginFor<T>,
            new_value: u32
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Validate input
            ensure!(new_value <= 1_000_000, Error::<T>::ValueTooLarge);
            
            // Update storage
            MyValue::<T>::put(new_value);
            
            // Emit event
            Self::deposit_event(Event::ValueSet { who, value: new_value });
            
            Ok(())
        }
        
        /// Store a name
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::store_name())]
        pub fn store_name(
            origin: OriginFor<T>,
            name: Vec<u8>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Convert to BoundedVec
            let bounded_name: BoundedVec<u8, T::MaxNameLength> = name.clone()
                .try_into()
                .map_err(|_| Error::<T>::NameTooLong)?;
            
            // Store in map
            MyMap::<T>::insert(&who, bounded_name);
            
            // Emit event
            Self::deposit_event(Event::NameStored { who, name });
            
            Ok(())
        }
    }
    
    /// Weight functions (required for all extrinsics)
    pub trait WeightInfo {
        fn set_value() -> Weight;
        fn store_name() -> Weight;
    }
    
    /// Default weight implementation
    impl<T> WeightInfo for () {
        fn set_value() -> Weight {
            Weight::from_parts(10_000_000, 0)
                .saturating_add(RocksDbWeight::get().writes(1))
        }
        
        fn store_name() -> Weight {
            Weight::from_parts(15_000_000, 0)
                .saturating_add(RocksDbWeight::get().reads(1))
                .saturating_add(RocksDbWeight::get().writes(1))
        }
    }
}
```

### Add to Runtime

**File: `belizechain/runtime/Cargo.toml`**

```toml
[dependencies]
pallet-my-pallet = { path = "../pallets/my-pallet", default-features = false }

[features]
std = [
    # ... other pallets
    "pallet-my-pallet/std",
]
```

**File: `belizechain/runtime/src/lib.rs`**

```rust
// 1. Configure pallet
impl pallet_my_pallet::Config for Runtime {
    type Currency = Balances;
    type MaxNameLength = ConstU32<64>;  // Max 64 bytes
    type WeightInfo = ();  // Use default weights
}

// 2. Add to construct_runtime! macro
construct_runtime!(
    pub struct Runtime {
        // ... system pallets
        System: frame_system,
        Balances: pallet_balances,
        
        // ... custom pallets
        Economy: pallet_belize_economy,
        Identity: pallet_belize_identity,
        
        // Add your pallet
        MyPallet: pallet_my_pallet,
    }
);
```

---

## Testing

### Unit Tests

**Add tests to your pallet:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop};
    use sp_core::H256;
    use sp_runtime::BuildStorage;
    
    type Block = frame_system::mocking::MockBlock<Test>;
    
    // Configure mock runtime for testing
    frame_support::construct_runtime!(
        pub struct Test {
            System: frame_system,
            MyPallet: pallet_my_pallet,
        }
    );
    
    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type Block = Block;
        type BlockHashCount = ConstU64<250>;
        // ... other config
    }
    
    impl pallet_my_pallet::Config for Test {
        type Currency = ();
        type MaxNameLength = ConstU32<64>;
        type WeightInfo = ();
    }
    
    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();
        storage.into()
    }
    
    #[test]
    fn set_value_works() {
        new_test_ext().execute_with(|| {
            // Set value to 42
            assert_ok!(MyPallet::set_value(RuntimeOrigin::signed(1), 42));
            
            // Check storage
            assert_eq!(MyPallet::my_value(), 42);
        });
    }
    
    #[test]
    fn set_value_fails_if_too_large() {
        new_test_ext().execute_with(|| {
            // Value > 1,000,000 should fail
            assert_noop!(
                MyPallet::set_value(RuntimeOrigin::signed(1), 2_000_000),
                Error::<Test>::ValueTooLarge
            );
        });
    }
    
    #[test]
    fn store_name_works() {
        new_test_ext().execute_with(|| {
            let name = b"Alice".to_vec();
            
            assert_ok!(MyPallet::store_name(RuntimeOrigin::signed(1), name.clone()));
            
            // Check storage
            let stored = MyPallet::my_map(1).unwrap();
            assert_eq!(stored.to_vec(), name);
        });
    }
}
```

**Run tests:**

```bash
# Test single pallet
cargo test -p pallet-my-pallet

# Test all pallets
cargo test --workspace

# Test with output
cargo test -p pallet-my-pallet -- --nocapture

# Test specific function
cargo test -p pallet-my-pallet set_value_works
```

### Integration Tests

**File: `tests/integration/test_my_pallet.py`**

```python
import pytest
from substrateinterface import SubstrateInterface, Keypair

@pytest.fixture
def substrate():
    """Connect to local development node"""
    return SubstrateInterface(
        url="ws://127.0.0.1:9944",
        ss58_format=42,  # Generic Substrate
        type_registry_preset='polkadot'
    )

@pytest.fixture
def alice():
    """Alice test account"""
    return Keypair.create_from_uri('//Alice')

def test_set_value(substrate, alice):
    """Test setting a value via extrinsic"""
    call = substrate.compose_call(
        call_module='MyPallet',
        call_function='set_value',
        call_params={'new_value': 100}
    )
    
    extrinsic = substrate.create_signed_extrinsic(
        call=call,
        keypair=alice
    )
    
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    
    assert receipt.is_success, f"Extrinsic failed: {receipt.error_message}"
    
    # Verify storage
    value = substrate.query('MyPallet', 'MyValue')
    assert value.value == 100

def test_store_name(substrate, alice):
    """Test storing a name"""
    call = substrate.compose_call(
        call_module='MyPallet',
        call_function='store_name',
        call_params={'name': list(b'Alice')}  # Convert bytes to list
    )
    
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    
    assert receipt.is_success
    
    # Query storage map
    stored_name = substrate.query('MyPallet', 'MyMap', [alice.ss58_address])
    assert bytes(stored_name.value) == b'Alice'
```

**Run integration tests:**

```bash
# Start development node in background
./target/release/belizechain-node --dev --tmp &
NODE_PID=$!

# Wait for node to start
sleep 5

# Run integration tests
pytest tests/integration/test_my_pallet.py -v

# Stop node
kill $NODE_PID
```

---

## Debugging

### Enable Verbose Logging

```bash
# Log all pallets
RUST_LOG=debug ./target/release/belizechain-node --dev

# Log specific pallets
RUST_LOG=pallet_belize_economy=trace,pallet_my_pallet=debug \
    ./target/release/belizechain-node --dev

# Log runtime execution
RUST_LOG=runtime=trace ./target/release/belizechain-node --dev
```

### Add Debug Prints in Pallets

```rust
use frame_support::log::{info, warn, error, debug};

#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn set_value(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        debug!(target: "my-pallet", "Setting value to {} for {:?}", new_value, who);
        
        if new_value > 1_000_000 {
            warn!(target: "my-pallet", "Value {} exceeds maximum", new_value);
            return Err(Error::<T>::ValueTooLarge.into());
        }
        
        MyValue::<T>::put(new_value);
        
        info!(target: "my-pallet", "✅ Value set successfully");
        
        Ok(())
    }
}
```

### Use Polkadot.js for Debugging

```javascript
// Connect to node
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function debug() {
  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider });
  
  // Query storage
  const value = await api.query.myPallet.myValue();
  console.log('Current value:', value.toString());
  
  // Listen to events
  api.query.system.events((events) => {
    events.forEach((record) => {
      const { event } = record;
      if (event.section === 'myPallet') {
        console.log(`MyPallet event: ${event.method}`, event.data.toHuman());
      }
    });
  });
}

debug().catch(console.error);
```

---

## Runtime Upgrades

### Build New Runtime

```bash
# Build WASM runtime
cargo build --release

# WASM file location:
# ./target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm
```

### Submit Upgrade (via Sudo)

```javascript
// Using Polkadot.js
const fs = require('fs');
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function upgradeRuntime() {
  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider });
  
  // Read WASM binary
  const code = fs.readFileSync(
    './target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm'
  );
  
  // Submit via sudo (development only)
  const sudoKey = /* your sudo keypair */;
  
  const tx = api.tx.sudo.sudoUncheckedWeight(
    api.tx.system.setCode(code),
    { refTime: 0, proofSize: 0 }
  );
  
  await tx.signAndSend(sudoKey, ({ status, events }) => {
    if (status.isInBlock) {
      console.log(`✅ Runtime upgraded in block ${status.asInBlock}`);
    }
  });
}
```

---

## Related Documentation

- [Environment Setup](./environment-setup.md)
- [Pallet API Reference](./pallet-apis-core.md)
- [Smart Contract Development](./smart-contract-development.md)
- [Substrate Documentation](https://docs.substrate.io)
