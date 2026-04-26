# Migration Guides

Guides for migrating to BelizeChain from other blockchain platforms, upgrading existing deployments, and transitioning data.

## Ethereum to BelizeChain

### Smart Contract Migration (Solidity → ink!)

#### 1. Architecture Comparison

| Ethereum | BelizeChain | Notes |
|----------|-------------|-------|
| Solidity | ink! (Rust) | Compiled to WebAssembly |
| EVM | Wasm VM | Higher performance, smaller binary |
| Gas | Weight | More predictable costs |
| msg.sender | self.env().caller() | Same concept, different syntax |
| Events | Events | Similar implementation |

#### 2. ERC-20 to PSP22 Migration

**Ethereum ERC-20**:
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract MyToken {
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    
    uint256 private _totalSupply;
    string private _name;
    string private _symbol;
    
    function transfer(address to, uint256 amount) public returns (bool) {
        require(_balances[msg.sender] >= amount, "Insufficient balance");
        _balances[msg.sender] -= amount;
        _balances[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }
    
    function approve(address spender, uint256 amount) public returns (bool) {
        _allowances[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }
}
```

**BelizeChain PSP22 (ink!)**:
```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::string::String;
use ink::storage::Mapping;

#[ink::contract]
mod my_token {
    use super::*;
    
    #[ink(storage)]
    pub struct MyToken {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>,
        name: String,
        symbol: String,
    }
    
    #[ink(message)]
    pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<(), PSP22Error> {
        let from = self.env().caller();
        let from_balance = self.balance_of(from);
        
        if from_balance < amount {
            return Err(PSP22Error::InsufficientBalance);
        }
        
        self.balances.insert(from, &(from_balance - amount));
        let to_balance = self.balance_of(to);
        self.balances.insert(to, &(to_balance + amount));
        
        self.env().emit_event(Transfer { from: Some(from), to: Some(to), value: amount });
        Ok(())
    }
    
    #[ink(message)]
    pub fn approve(&mut self, spender: AccountId, amount: Balance) -> Result<(), PSP22Error> {
        let owner = self.env().caller();
        self.allowances.insert((owner, spender), &amount);
        self.env().emit_event(Approval { owner, spender, value: amount });
        Ok(())
    }
}
```

#### 3. Migration Workflow

```bash
# Step 1: Install ink! CLI
cargo install cargo-contract --force

# Step 2: Create new PSP22 project from template
cargo contract new my_token --template psp22
cd my_token

# Step 3: Copy business logic from Solidity
# Translate functions manually (see examples above)

# Step 4: Build contract
cargo contract build --release

# Step 5: Deploy to BelizeChain testnet
cargo contract instantiate \
  --constructor new \
  --args "1000000 MyToken MTK" \
  --suri //Alice \
    --url wss://<current-testnet-rpc-url>

# Step 6: Verify deployment
cargo contract call \
  --contract 5GrwvaEF... \
  --message total_supply \
  --suri //Alice \
    --url wss://<current-testnet-rpc-url>
```

#### 4. Asset Bridge (Ethereum ↔ BelizeChain)

Transfer ERC-20 tokens to BelizeChain:

```javascript
// Ethereum side: Lock tokens
const BridgeContract = new web3.eth.Contract(BRIDGE_ABI, BRIDGE_ADDRESS);

await BridgeContract.methods.lockTokens(
  ERC20_ADDRESS,
  '1000000000000000000',  // 1 token (18 decimals)
  'BELIZECHAIN_ADDRESS'
).send({ from: ethereumAccount });

// Wait for 12 block confirmations (~3 minutes)

// BelizeChain side: Mint wrapped tokens
const api = await ApiPromise.create({ provider });
const bridge = api.tx.interoperability.mintWrappedToken(
  'ETHEREUM',
  ethereumTxHash,
  '1000000000000000000'
);

await bridge.signAndSend(belizechainAccount);

// Wrapped token now available as wToken on BelizeChain
```

### Account Migration

```python
# Convert Ethereum address to BelizeChain account
from substrateinterface import Keypair
from eth_account import Account

# Option 1: Import Ethereum private key
eth_private_key = '0x1234...'
eth_account = Account.from_key(eth_private_key)

# Derive BelizeChain keypair from same entropy
substrate_keypair = Keypair.create_from_private_key(
    eth_private_key,
    ss58_format=42
)

print(f"Ethereum: {eth_account.address}")
print(f"BelizeChain: {substrate_keypair.ss58_address}")

# Option 2: Generate new BelizeChain account and bridge assets
new_keypair = Keypair.create_from_mnemonic(Keypair.generate_mnemonic())
print(f"New BelizeChain account: {new_keypair.ss58_address}")
print(f"Use bridge to transfer assets from Ethereum")
```

## Polkadot/Kusama Parachain Migration

### XCM Integration

BelizeChain natively supports XCM v3 for seamless interoperability:

```rust
// Configure XCM in runtime
impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    // ...
}

// Transfer assets from Polkadot to BelizeChain
let dest = MultiLocation {
    parents: 1,
    interior: X1(Parachain(2000))  // BelizeChain parachain ID
};

let beneficiary = MultiLocation {
    parents: 0,
    interior: X1(AccountId32 {
        network: None,
        id: beneficiary_account.into()
    })
};

let assets = MultiAssets::from((
    MultiLocation::here(),
    1_000_000_000_000u128  // 1,000 DOT
));

pallet_xcm::Pallet::<Runtime>::reserve_transfer_assets(
    origin,
    Box::new(dest.into()),
    Box::new(beneficiary.into()),
    Box::new(assets.into()),
    0  // Fee asset index
)?;
```

### Runtime Migration

For parachains upgrading to BelizeChain runtime:

```bash
# 1. Export current state
polkadot-parachain export-state \
  --chain your-chain \
  --output state.json

# 2. Convert storage format (if needed)
python scripts/convert_storage.py state.json belizechain_state.json

# 3. Build BelizeChain runtime
cd belizechain
cargo build --release

# 4. Generate genesis with imported state
./target/release/belizechain-node build-spec \
  --chain staging \
  --raw \
  --disable-default-bootnode \
  --import-state belizechain_state.json \
  > chain-spec.json

# 5. Start validators with new runtime
./target/release/belizechain-node \
  --chain chain-spec.json \
  --validator \
  --name "Validator-1"
```

## Data Migration

### IPFS to Pakit DAG Storage

```python
# Migrate files from IPFS to Pakit
from pakit.core.dag_storage import DAGStorage
import ipfshttpclient

# Connect to IPFS
ipfs_client = ipfshttpclient.connect('/ip4/127.0.0.1/tcp/5001')

# Connect to Pakit
pakit = DAGStorage(endpoint='https://pakit.belizechain.org')

# Migrate directory
ipfs_cid = 'QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco'

def migrate_ipfs_to_pakit(cid):
    # Download from IPFS
    data = ipfs_client.cat(cid)
    
    # Upload to Pakit with compression
    pakit_hash = pakit.store(
        data,
        compression='hybrid',  # Quantum compression
        deduplication=True
    )
    
    # Register mapping on-chain
    api.tx.pakit.registerMigration(
        ipfs_cid=cid,
        pakit_hash=pakit_hash
    ).signAndSend(account)
    
    return pakit_hash

new_hash = migrate_ipfs_to_pakit(ipfs_cid)
print(f"Migrated {ipfs_cid} → {new_hash}")
print(f"Compression: {ipfs_size / pakit_size:.2f}x")
```

### Centralized Database to Blockchain

```python
# Migrate SQL database to BelizeChain LandLedger
import psycopg2
from substrateinterface import SubstrateInterface, Keypair

# Connect to PostgreSQL
pg_conn = psycopg2.connect("dbname=land_registry user=admin")
cursor = pg_conn.cursor()

# Connect to BelizeChain
substrate = SubstrateInterface(url="wss://rpc.belizechain.org")
keypair = Keypair.create_from_uri('//GovernmentAdmin')

# Fetch all properties
cursor.execute("SELECT id, owner, address, size, title_deed FROM properties")

for row in cursor.fetchall():
    property_id, owner, address, size, title_deed_path = row
    
    # Upload title deed to Pakit
    with open(title_deed_path, 'rb') as f:
        deed_hash = pakit.store(f.read())
    
    # Register property on-chain
    call = substrate.compose_call(
        call_module='LandLedger',
        call_function='register_property',
        call_params={
            'property_id': property_id,
            'owner': owner,
            'location': address,
            'area_sqm': size,
            'title_deed_hash': deed_hash
        }
    )
    
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    
    print(f"Migrated property {property_id}: {receipt.extrinsic_hash}")

print(f"Migration complete: {cursor.rowcount} properties")
```

## Version Upgrades

### Runtime Upgrade (Forkless)

```bash
# 1. Build new runtime
cargo build --release -p belizechain-runtime

# 2. Generate runtime WASM
./target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm

# 3. Calculate WASM hash
WASM_HASH=$(sha256sum belizechain_runtime.compact.compressed.wasm | awk '{print $1}')
echo "Runtime hash: $WASM_HASH"

# 4. Submit governance proposal
polkadot-js-api \
  --ws wss://rpc.belizechain.org \
  tx.democracy.propose \
  '{"call":"system.setCode","args":{"code":"0x'$(xxd -p -c 0 belizechain_runtime.compact.compressed.wasm)'"}}' \
  --seed "//GovernmentCouncil"

# 5. Wait for referendum to pass

# 6. Enact upgrade
polkadot-js-api tx.democracy.enactProposal 42 --seed "//GovernmentCouncil"
```

### Storage Migration

```rust
// Migration from v1 to v2 storage format
pub mod v2 {
    use super::*;
    
    pub fn migrate<T: Config>() -> Weight {
        let mut weight = Weight::zero();
        
        // Translate old storage to new format
        PropertyRegistry::<T>::translate::<OldPropertyData, _>(
            |key, old_data| {
                weight += T::DbWeight::get().reads_writes(1, 1);
                
                Some(PropertyData {
                    owner: old_data.owner,
                    location: old_data.location,
                    area_sqm: old_data.size,  // Renamed field
                    title_deed_hash: old_data.title_hash,
                    registered_at: old_data.timestamp,
                    status: PropertyStatus::Active,  // New field with default
                    metadata: BoundedVec::default()  // New field
                })
            }
        );
        
        // Update storage version
        StorageVersion::new(2).put::<Pallet<T>>();
        
        weight
    }
}

// Trigger migration in runtime upgrade
impl OnRuntimeUpgrade for Pallet<T> {
    fn on_runtime_upgrade() -> Weight {
        let current = StorageVersion::get::<Pallet<T>>();
        
        if current == 1 {
            log::info!("Migrating LandLedger storage from v1 to v2");
            v2::migrate::<T>()
        } else {
            Weight::zero()
        }
    }
}
```

### Breaking API Changes

**v1.0 → v2.0 Breaking Changes**:

| Category | v1.0 | v2.0 | Migration |
|----------|------|------|-----------|
| Transfer | `economy.transfer(to, amount)` | `economy.transfer(to, amount, currency)` | Add currency parameter |
| KYC | `identity.setKycLevel(level)` | `identity.updateKyc(data)` | Use structured data object |
| Staking | `staking.bond(amount)` | `staking.bond(amount, payee)` | Specify reward destination |

**Migration script**:
```javascript
// Update all transfer calls
const oldCalls = await api.query.system.events();

for (const record of oldCalls) {
  if (record.event.method === 'transfer' && record.event.section === 'economy') {
    // Old: transfer(to, amount)
    // New: transfer(to, amount, 'DALLA')
    
    const [to, amount] = record.event.data;
    
    // Resubmit with currency parameter
    await api.tx.economy.transfer(to, amount, 'DALLA').signAndSend(account);
  }
}
```

## Testing Migration

### Staging Environment

```bash
# 1. Clone mainnet state to testnet
polkadot-js-api \
  --ws wss://rpc.belizechain.org \
  rpc.state.getKeysPaged null 100 \
  --output mainnet_state.json

# 2. Start staging network
./target/release/belizechain-node \
  --chain staging \
  --tmp \
  --import-state mainnet_state.json \
  --validator

# 3. Test migration scripts on staging
python test_migration.py --network staging

# 4. Verify state consistency
python verify_migration.py \
  --source wss://rpc.belizechain.org \
  --target ws://localhost:9944
```

### Rollback Procedures

```bash
# Emergency rollback to previous runtime version
# (Requires sudo or governance emergency powers)

polkadot-js-api \
  --ws wss://rpc.belizechain.org \
  --sudo \
  tx.system.setCode \
  --code previous_runtime.wasm \
  --seed "//EmergencySudo"

# Restore storage from backup
./scripts/restore_storage.sh \
  --backup storage_backup_2026_01_30.json \
  --target wss://rpc.belizechain.org
```

## Migration Support

For assistance with complex migrations:
- **Developer Discord**: https://discord.gg/belizechain
- **Migration Tool Repository**: https://github.com/belizechain/migration-tools
- **Professional Services**: migrations@belizechain.org
- **Documentation**: https://docs.belizechain.org/migrations
