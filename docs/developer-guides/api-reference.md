# 📖 BelizeChain API Reference

**Complete reference for all blockchain APIs and RPC methods**

---

## 📋 Overview

BelizeChain provides multiple APIs for interacting with the blockchain:

**API Types**:
- **RPC API**: Read blockchain data (queries, subscriptions)
- **Transaction API**: Submit transactions (extrinsics)
- **WebSocket API**: Real-time subscriptions
- **REST API**: HTTP endpoints (via middleware)

**Connection Details**:
- **Mainnet**: `wss://mainnet.belizechain.org`
- **Testnet**: Use the current operator-provided RPC URL or your own node
- **Local Dev**: `ws://localhost:9944`

---

## 🔌 Quick Start

### **JavaScript/TypeScript**

```bash
npm install @polkadot/api
```

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

// Connect to blockchain
const provider = new WsProvider('wss://mainnet.belizechain.org');
const api = await ApiPromise.create({ provider });

// Query data
const balance = await api.query.system.account('5GrwvaEF...');
console.log('Balance:', balance.data.free.toString());

// Subscribe to new blocks
await api.rpc.chain.subscribeNewHeads((header) => {
  console.log(`Block #${header.number}: ${header.hash}`);
});
```

### **Python**

```bash
pip install substrate-interface
```

```python
from substrateinterface import SubstrateInterface

# Connect to blockchain
substrate = SubstrateInterface(
    url="wss://mainnet.belizechain.org"
)

# Query data
balance = substrate.query('System', 'Account', ['5GrwvaEF...'])
print(f"Balance: {balance.value['data']['free']}")

# Subscribe to new blocks
def block_handler(obj, update_nr, subscription_id):
    print(f"Block #{obj['header']['number']}: {obj['header']['hash']}")

substrate.subscribe_block_headers(block_handler)
```

---

## 🔍 RPC Methods

### **System Methods**

#### `system_chain()` - Get Chain Name

```javascript
const chain = await api.rpc.system.chain();
// Returns: "BelizeChain"
```

#### `system_version()` - Get Node Version

```javascript
const version = await api.rpc.system.version();
// Returns: "1.0.0"
```

#### `system_health()` - Get Node Health

```javascript
const health = await api.rpc.system.health();
// Returns: { peers: 50, isSyncing: false, shouldHavePeers: true }
```

#### `system_properties()` - Get Chain Properties

```javascript
const properties = await api.rpc.system.properties();
// Returns: {
//   tokenDecimals: [12],
//   tokenSymbol: ['DALLA'],
//   ss58Format: 42
// }
```

---

### **Chain Methods**

#### `chain_getBlock(hash?)` - Get Block

```javascript
// Get latest block
const block = await api.rpc.chain.getBlock();

// Get specific block
const hash = '0x1a2b3c4d...';
const specificBlock = await api.rpc.chain.getBlock(hash);

// Block structure:
// {
//   block: {
//     header: {
//       number: 12345,
//       parentHash: '0x...',
//       stateRoot: '0x...',
//       extrinsicsRoot: '0x...'
//     },
//     extrinsics: [...]
//   }
// }
```

#### `chain_getBlockHash(number?)` - Get Block Hash

```javascript
// Get latest block hash
const latestHash = await api.rpc.chain.getBlockHash();

// Get block hash by number
const hash = await api.rpc.chain.getBlockHash(12345);
```

#### `chain_getFinalizedHead()` - Get Finalized Block Hash

```javascript
const finalizedHash = await api.rpc.chain.getFinalizedHead();
// Returns hash of last finalized block
```

#### `chain_subscribeNewHeads(callback)` - Subscribe to New Blocks

```javascript
const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
  console.log(`Block #${header.number}`);
  console.log(`Hash: ${header.hash}`);
  console.log(`Parent: ${header.parentHash}`);
});

// Unsubscribe later
unsubscribe();
```

#### `chain_subscribeFinalizedHeads(callback)` - Subscribe to Finalized Blocks

```javascript
const unsubscribe = await api.rpc.chain.subscribeFinalizedHeads((header) => {
  console.log(`Finalized block #${header.number}`);
});
```

---

### **State Methods**

#### `state_getStorage(key, hash?)` - Get Storage Value

```javascript
// Get storage at latest block
const key = '0x...';  // Storage key
const value = await api.rpc.state.getStorage(key);

// Get storage at specific block
const historicalValue = await api.rpc.state.getStorage(key, blockHash);
```

#### `state_getMetadata(hash?)` - Get Runtime Metadata

```javascript
const metadata = await api.rpc.state.getMetadata();
// Returns complete runtime metadata (pallets, functions, events, errors)
```

#### `state_getRuntimeVersion(hash?)` - Get Runtime Version

```javascript
const runtimeVersion = await api.rpc.state.getRuntimeVersion();
// Returns: {
//   specName: 'belizechain',
//   specVersion: 100,
//   implVersion: 1,
//   apis: [...]
// }
```

#### `state_subscribeStorage(keys, callback)` - Subscribe to Storage Changes

```javascript
const key = api.query.system.account.key('5GrwvaEF...');

const unsubscribe = await api.rpc.state.subscribeStorage([key], (changes) => {
  changes.forEach(([storageKey, value]) => {
    console.log('Storage changed:', value.toHuman());
  });
});
```

---

### **Author Methods** (Submitting Transactions)

#### `author_submitExtrinsic(extrinsic)` - Submit Transaction

```javascript
// Create transaction
const tx = api.tx.balances.transfer(recipient, amount);

// Sign transaction
const signedTx = await tx.signAsync(sender, { signer });

// Submit (fire-and-forget)
const hash = await api.rpc.author.submitExtrinsic(signedTx);
console.log(`Transaction hash: ${hash}`);
```

#### `author_submitAndWatchExtrinsic(extrinsic, callback)` - Submit and Watch

```javascript
const unsubscribe = await api.rpc.author.submitAndWatchExtrinsic(signedTx, (status) => {
  if (status.isInBlock) {
    console.log(`Included in block: ${status.asInBlock}`);
  } else if (status.isFinalized) {
    console.log(`Finalized in block: ${status.asFinalized}`);
    unsubscribe();
  } else if (status.isDropped || status.isInvalid) {
    console.error('Transaction failed');
    unsubscribe();
  }
});
```

---

## 💾 Storage Queries

### **Query Patterns**

#### **Single Value**

```javascript
// Get total issuance
const totalIssuance = await api.query.balances.totalIssuance();
console.log(totalIssuance.toString());
```

#### **Map Query**

```javascript
// Get account balance
const account = await api.query.system.account('5GrwvaEF...');
console.log('Free balance:', account.data.free.toString());
console.log('Reserved:', account.data.reserved.toString());
```

#### **Double Map Query**

```javascript
// Get allowance (if implemented)
const allowance = await api.query.economy.allowances(
  owner,
  spender
);
```

#### **Multi-Query (Batch)**

```javascript
// Query multiple accounts at once
const accounts = [
  '5GrwvaEF...',
  '5FHneW46...',
  '5DAAnrj7...'
];

const balances = await api.query.system.account.multi(accounts);
balances.forEach((account, index) => {
  console.log(`${accounts[index]}: ${account.data.free}`);
});
```

#### **Entries (All Items)**

```javascript
// Get all accounts (expensive!)
const entries = await api.query.system.account.entries();
entries.forEach(([key, account]) => {
  const address = key.args[0].toString();
  console.log(`${address}: ${account.data.free}`);
});
```

---

## 📝 Transactions (Extrinsics)

### **Transaction Lifecycle**

```javascript
// 1. Create transaction
const tx = api.tx.balances.transfer(recipient, 1000);

// 2. Sign transaction
const hash = await tx.signAndSend(sender, { signer }, ({ status, events }) => {
  if (status.isInBlock) {
    console.log(`Included in block ${status.asInBlock}`);
    
    // 3. Check for errors
    events.forEach(({ event }) => {
      if (api.events.system.ExtrinsicFailed.is(event)) {
        const [dispatchError] = event.data;
        console.error('Transaction failed:', dispatchError.toString());
      } else if (api.events.system.ExtrinsicSuccess.is(event)) {
        console.log('Transaction succeeded');
      }
    });
  } else if (status.isFinalized) {
    console.log(`Finalized in block ${status.asFinalized}`);
  }
});
```

### **Common Transactions**

#### **Transfer Balance**

```javascript
// Transfer DALLA
const tx = api.tx.balances.transfer(
  recipient,    // Address
  1000000000000 // Amount (12 decimals)
);

await tx.signAndSend(sender, { signer });
```

#### **Create Proposal**

```javascript
// Create governance proposal
const tx = api.tx.governance.propose(
  'Fix Main Street',           // Title
  'Repair potholes...',        // Description
  5000000000000000,            // Amount (5000 DALLA)
  beneficiary                  // Beneficiary address
);

await tx.signAndSend(sender, { signer });
```

#### **Vote on Proposal**

```javascript
// Vote on proposal
const tx = api.tx.governance.vote(
  proposalId,  // Proposal ID
  'Yes'        // Vote: 'Yes', 'No', or 'Abstain'
);

await tx.signAndSend(sender, { signer });
```

#### **Stake Tokens**

```javascript
// Bond tokens for staking
const tx = api.tx.staking.bond(
  controller,            // Controller account
  1000000000000000,      // Amount (1000 DALLA)
  'Staked'               // Reward destination
);

await tx.signAndSend(sender, { signer });
```

#### **Batch Transactions**

```javascript
// Execute multiple transactions atomically
const tx = api.tx.utility.batch([
  api.tx.balances.transfer(alice, 100),
  api.tx.balances.transfer(bob, 200),
  api.tx.balances.transfer(charlie, 300)
]);

await tx.signAndSend(sender, { signer });
// All succeed or all fail together
```

---

## 🎫 Events

### **Listening to Events**

```javascript
// Subscribe to all events
api.query.system.events((events) => {
  events.forEach((record) => {
    const { event } = record;
    
    if (api.events.balances.Transfer.is(event)) {
      const [from, to, amount] = event.data;
      console.log(`Transfer: ${from} -> ${to}: ${amount}`);
    }
    
    if (api.events.governance.ProposalCreated.is(event)) {
      const [proposer, proposalId] = event.data;
      console.log(`Proposal ${proposalId} created by ${proposer}`);
    }
  });
});
```

### **Common Events**

#### **Balance Events**

- `balances.Transfer` - Balance transferred
- `balances.BalanceSet` - Balance set (rare)
- `balances.Reserved` - Balance reserved
- `balances.Unreserved` - Balance unreserved

#### **Governance Events**

- `governance.ProposalCreated` - New proposal
- `governance.VoteCast` - Vote cast
- `governance.ProposalApproved` - Proposal approved
- `governance.ProposalRejected` - Proposal rejected
- `governance.ProposalExecuted` - Proposal executed

#### **Staking Events**

- `staking.Bonded` - Tokens bonded
- `staking.Unbonded` - Tokens unbonded
- `staking.Rewarded` - Staking reward paid
- `staking.Slashed` - Validator slashed

#### **System Events**

- `system.ExtrinsicSuccess` - Transaction succeeded
- `system.ExtrinsicFailed` - Transaction failed
- `system.NewAccount` - New account created

---

## 🔔 WebSocket Subscriptions

### **Real-Time Data**

```javascript
// New blocks
api.rpc.chain.subscribeNewHeads((header) => {
  console.log(`Block ${header.number}`);
});

// Finalized blocks
api.rpc.chain.subscribeFinalizedHeads((header) => {
  console.log(`Finalized ${header.number}`);
});

// Storage changes
api.query.system.account(address, (account) => {
  console.log(`Balance: ${account.data.free}`);
});

// All events
api.query.system.events((events) => {
  console.log(`${events.length} events in this block`);
});
```

---

## 🧮 Utility Functions

### **Balance Formatting**

```javascript
import { formatBalance } from '@polkadot/util';

// Format balance with decimals
formatBalance.setDefaults({ decimals: 12, unit: 'DALLA' });

const balance = 1000000000000; // 1 DALLA
console.log(formatBalance(balance)); // "1 DALLA"

const large = 5432100000000000; // 5432.1 DALLA
console.log(formatBalance(large)); // "5.4321 kDALLA"
```

### **Address Encoding**

```javascript
import { encodeAddress, decodeAddress } from '@polkadot/util-crypto';

// Convert public key to SS58 address
const publicKey = '0x1a2b3c...';
const address = encodeAddress(publicKey, 42); // 42 = BelizeChain prefix

// Decode address to public key
const decoded = decodeAddress(address);
```

### **Type Conversion**

```javascript
// BN (Big Number) to string
const balance = await api.query.system.account(address);
const freeBalance = balance.data.free.toString();

// String to BN
const amount = api.createType('Balance', '1000000000000');

// Human-readable output
const human = balance.toHuman();
console.log(human); // { free: "1.0000 DALLA", reserved: "0" }
```

---

## 🐍 Python Examples

### **Query Balance**

```python
from substrateinterface import SubstrateInterface

substrate = SubstrateInterface(url="wss://mainnet.belizechain.org")

# Query balance
result = substrate.query(
    module='System',
    storage_function='Account',
    params=['5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY']
)

print(f"Free balance: {result.value['data']['free']}")
print(f"Reserved: {result.value['data']['reserved']}")
```

### **Submit Transaction**

```python
from substrateinterface import SubstrateInterface, Keypair

substrate = SubstrateInterface(url="wss://mainnet.belizechain.org")
keypair = Keypair.create_from_mnemonic('your twelve word mnemonic here...')

# Create transaction
call = substrate.compose_call(
    call_module='Balances',
    call_function='transfer',
    call_params={
        'dest': '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
        'value': 1000000000000  # 1 DALLA
    }
)

# Sign and submit
extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)
receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)

print(f"Extrinsic hash: {receipt.extrinsic_hash}")
print(f"Block hash: {receipt.block_hash}")
print(f"Success: {receipt.is_success}")
```

---

## 🚦 Error Handling

### **Dispatch Errors**

```javascript
try {
  await tx.signAndSend(sender, { signer }, ({ status, events }) => {
    if (status.isInBlock) {
      events.forEach(({ event }) => {
        if (api.events.system.ExtrinsicFailed.is(event)) {
          const [dispatchError] = event.data;
          
          if (dispatchError.isModule) {
            // Module error (from pallet)
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            const { docs, name, section } = decoded;
            
            console.error(`${section}.${name}: ${docs.join(' ')}`);
            // Example: "Balances.InsufficientBalance: Balance too low"
          } else {
            // Other error
            console.error(dispatchError.toString());
          }
        }
      });
    }
  });
} catch (error) {
  console.error('Transaction error:', error);
}
```

### **Common Errors**

| Error | Meaning | Solution |
|-------|---------|----------|
| `InsufficientBalance` | Not enough tokens | Fund account |
| `BadOrigin` | Wrong permission level | Use correct account |
| `Module` | Pallet-specific error | Check pallet documentation |
| `CannotLookup` | Address not found | Check address format |
| `TxValidityError` | Invalid transaction | Check transaction params |

---

## 📊 Advanced Queries

### **Historical Data**

```javascript
// Query at specific block
const blockHash = await api.rpc.chain.getBlockHash(12345);
const historicalBalance = await api.query.system.account.at(
  blockHash,
  address
);
```

### **Pagination**

```javascript
// Get entries with pagination
const pageSize = 100;
let startKey = null;

while (true) {
  const entries = await api.query.system.account.entriesPaged({
    args: [],
    pageSize,
    startKey
  });
  
  if (entries.length === 0) break;
  
  entries.forEach(([key, account]) => {
    // Process account
  });
  
  startKey = entries[entries.length - 1][0];
}
```

---

## 🔐 Best Practices

### **Connection Management**

```javascript
// Always disconnect when done
const api = await ApiPromise.create({ provider });

try {
  // Use API
} finally {
  await api.disconnect();
}
```

### **Error Handling**

```javascript
// Handle connection failures
const provider = new WsProvider('wss://mainnet.belizechain.org');

provider.on('error', (error) => {
  console.error('Provider error:', error);
});

provider.on('disconnected', () => {
  console.log('Disconnected, reconnecting...');
});

const api = await ApiPromise.create({ provider });
```

### **Rate Limiting**

```javascript
// Batch queries instead of looping
const addresses = ['5GrwvaEF...', '5FHneW46...', '5DAAnrj7...'];

// ❌ Bad: Multiple individual queries
for (const addr of addresses) {
  const balance = await api.query.system.account(addr);
}

// ✅ Good: Single batch query
const balances = await api.query.system.account.multi(addresses);
```

---

## 📚 Resources

**Official Docs**:
- Polkadot.js API: https://polkadot.js.org/docs/
- Substrate RPC: https://docs.substrate.io/reference/rpc-api/

**BelizeChain**:
- Discord: https://discord.gg/belizechain
- GitHub: https://github.com/BelizeChain/belizechain
- API Playground: https://api.belizechain.org

**Tools**:
- Polkadot.js Apps: https://polkadot.js.org/apps
- Substrate Sidecar: https://github.com/paritytech/substrate-api-sidecar

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) for API support! 🚀🇧🇿
