# API Reference

Complete API documentation for BelizeChain's REST, WebSocket, and RPC interfaces.

## API Endpoints

### Base URLs

| Environment | REST API | WebSocket | GraphQL |
|-------------|----------|-----------|---------|
| Mainnet | `https://api.belizechain.org` | `wss://rpc.belizechain.org` | `https://graphql.belizechain.org` |
| Testnet | `https://api.testnet.belizechain.org` | `wss://rpc.testnet.belizechain.org` | `https://graphql.testnet.belizechain.org` |
| Local | `http://localhost:9933` | `ws://localhost:9944` | `http://localhost:8080/graphql` |

## Authentication

### API Key Authentication

```bash
# Register for API key at https://portal.belizechain.org
curl -X POST https://api.belizechain.org/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "developer@example.com", "tier": "free"}'

# Response:
# {
#   "api_key": "bzc_live_a1b2c3d4e5f6...",
#   "tier": "free",
#   "rate_limit": "1000/min"
# }

# Use API key in requests
curl https://api.belizechain.org/v1/blocks/latest \
  -H "X-API-Key: bzc_live_a1b2c3d4e5f6..."
```

### Rate Limits

| Tier | Requests/Minute | WebSocket Subscriptions | GraphQL Complexity |
|------|-----------------|-------------------------|-------------------|
| Free | 1,000 | 5 | 1,000 |
| Developer | 5,000 | 20 | 5,000 |
| Business | 25,000 | 100 | 25,000 |
| Enterprise | Unlimited | Unlimited | Unlimited |

## REST API

### Block Endpoints

#### GET /v1/blocks/latest

Get the latest block.

```bash
curl https://api.belizechain.org/v1/blocks/latest \
  -H "X-API-Key: YOUR_API_KEY"
```

**Response**:
```json
{
  "number": 1234567,
  "hash": "0x1234...abcd",
  "parent_hash": "0xabcd...1234",
  "state_root": "0x5678...efgh",
  "extrinsics_root": "0xijkl...mnop",
  "timestamp": 1738349280,
  "validator": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "extrinsics_count": 42,
  "events_count": 158
}
```

#### GET /v1/blocks/{block_number}

Get block by number or hash.

```bash
curl https://api.belizechain.org/v1/blocks/1234567
```

**Query Parameters**:
- `include_extrinsics` (boolean): Include full extrinsic data
- `include_events` (boolean): Include event logs

#### GET /v1/blocks/range

Get multiple blocks.

```bash
curl "https://api.belizechain.org/v1/blocks/range?from=1234000&to=1234100"
```

**Response**:
```json
{
  "blocks": [...],
  "total": 100,
  "from": 1234000,
  "to": 1234100
}
```

### Account Endpoints

#### GET /v1/accounts/{address}

Get account information.

```bash
curl https://api.belizechain.org/v1/accounts/5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

**Response**:
```json
{
  "address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "nonce": 42,
  "balances": {
    "dalla": {
      "free": "1500000000000000",
      "reserved": "500000000000000",
      "frozen": "0"
    },
    "bbzd": {
      "free": "25000000000000",
      "reserved": "0",
      "frozen": "0"
    }
  },
  "identity": {
    "belize_id": "BZ-ID-12345",
    "kyc_level": "Verified",
    "ssn": "SSN-***789"
  },
  "staking": {
    "is_validator": true,
    "bonded": "1000000000000000",
    "nominations": []
  }
}
```

#### GET /v1/accounts/{address}/transactions

Get account transaction history.

```bash
curl "https://api.belizechain.org/v1/accounts/5Grw.../transactions?limit=50&offset=0"
```

**Query Parameters**:
- `limit` (integer): Results per page (max 100)
- `offset` (integer): Pagination offset
- `type` (string): Filter by transaction type
- `from_block` (integer): Start block
- `to_block` (integer): End block

### Transaction Endpoints

#### POST /v1/transactions/submit

Submit signed transaction.

```bash
curl -X POST https://api.belizechain.org/v1/transactions/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: YOUR_API_KEY" \
  -d '{
    "extrinsic": "0x4502849ac...",
    "signature": "0x1234abcd..."
  }'
```

**Response**:
```json
{
  "hash": "0xabcd...1234",
  "block_number": 1234568,
  "status": "pending",
  "estimated_fee": "150000000000"
}
```

#### GET /v1/transactions/{hash}

Get transaction details.

```bash
curl https://api.belizechain.org/v1/transactions/0xabcd...1234
```

**Response**:
```json
{
  "hash": "0xabcd...1234",
  "block_number": 1234568,
  "block_hash": "0x5678...efgh",
  "timestamp": 1738349286,
  "from": "5GrwvaEF...",
  "to": "5DTestU...",
  "pallet": "Economy",
  "call": "transfer",
  "args": {
    "recipient": "5DTestU...",
    "amount": "1000000000000000",
    "currency": "DALLA"
  },
  "status": "success",
  "fee_paid": "148750000000",
  "events": [
    {
      "pallet": "Economy",
      "event": "Transfer",
      "data": {
        "from": "5GrwvaEF...",
        "to": "5DTestU...",
        "amount": "1000000000000000"
      }
    }
  ]
}
```

### Staking Endpoints

#### GET /v1/staking/validators

Get list of validators.

```bash
curl "https://api.belizechain.org/v1/staking/validators?active=true"
```

**Response**:
```json
{
  "validators": [
    {
      "address": "5GrwvaEF...",
      "identity": {
        "display": "Belize Validator 1",
        "legal": "Belize Blockchain Inc."
      },
      "total_stake": "5000000000000000",
      "own_stake": "1000000000000000",
      "nominators_count": 42,
      "commission": "10%",
      "apy": "544%",
      "uptime": "99.95%",
      "pouw_score": 0.85,
      "validator_type": ["Standard", "Nawal"]
    }
  ],
  "total": 12,
  "active": 12
}
```

#### GET /v1/staking/rewards/{address}

Get staking rewards history.

```bash
curl https://api.belizechain.org/v1/staking/rewards/5GrwvaEF...
```

**Response**:
```json
{
  "total_rewards": "250000000000000",
  "rewards": [
    {
      "era": 1234,
      "block_number": 1234567,
      "timestamp": 1738349280,
      "amount": "2100000000000",
      "type": "block_production",
      "validator": "5GrwvaEF..."
    },
    {
      "era": 1234,
      "block_number": 1234580,
      "timestamp": 1738349358,
      "amount": "350000000000",
      "type": "pouw_nawal",
      "score": 0.92
    }
  ]
}
```

### Governance Endpoints

#### GET /v1/governance/proposals

Get governance proposals.

```bash
curl "https://api.belizechain.org/v1/governance/proposals?status=active"
```

**Response**:
```json
{
  "proposals": [
    {
      "id": 42,
      "hash": "0x1234...",
      "type": "referendum",
      "title": "Increase validator rewards by 10%",
      "proposer": "5GrwvaEF...",
      "deposit": "10000000000000",
      "status": "voting",
      "voting_ends": 1738435680,
      "votes": {
        "aye": "15000000000000000",
        "nay": "3000000000000000",
        "abstain": "500000000000000"
      },
      "conviction": {
        "locked_1x": "8000000000000000",
        "locked_2x": "5000000000000000",
        "locked_4x": "2000000000000000"
      }
    }
  ]
}
```

#### POST /v1/governance/vote

Submit governance vote.

```bash
curl -X POST https://api.belizechain.org/v1/governance/vote \
  -H "Content-Type: application/json" \
  -H "X-API-Key: YOUR_API_KEY" \
  -d '{
    "proposal_id": 42,
    "vote": "aye",
    "conviction": "locked_2x",
    "amount": "1000000000000000",
    "signature": "0x..."
  }'
```

### DEX Endpoints (BelizeX)

#### GET /v1/dex/pairs

Get trading pairs.

```bash
curl https://api.belizechain.org/v1/dex/pairs
```

**Response**:
```json
{
  "pairs": [
    {
      "id": "DALLA-bBZD",
      "token0": "DALLA",
      "token1": "bBZD",
      "reserve0": "10000000000000000",
      "reserve1": "20000000000000000",
      "price": "2.0",
      "volume_24h": "500000000000000",
      "liquidity_usd": "50000",
      "fee": "0.3%"
    }
  ]
}
```

#### GET /v1/dex/orders

Get limit order book.

```bash
curl "https://api.belizechain.org/v1/dex/orders?pair=DALLA-bBZD&depth=20"
```

**Response**:
```json
{
  "bids": [
    {"price": "1.98", "amount": "1000000000000"},
    {"price": "1.97", "amount": "2000000000000"}
  ],
  "asks": [
    {"price": "2.02", "amount": "1500000000000"},
    {"price": "2.03", "amount": "1800000000000"}
  ],
  "spread": "0.04",
  "last_price": "2.0"
}
```

## WebSocket API

### Connection

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

const provider = new WsProvider('wss://rpc.belizechain.org');
const api = await ApiPromise.create({ provider });

console.log(`Connected to chain: ${await api.rpc.system.chain()}`);
// Output: Connected to chain: BelizeChain
```

### Subscriptions

#### Subscribe to new blocks

```javascript
const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
  console.log(`Block #${header.number}: ${header.hash}`);
});

// Output:
// Block #1234567: 0x1234...abcd
// Block #1234568: 0x5678...efgh
```

#### Subscribe to finalized blocks

```javascript
await api.rpc.chain.subscribeFinalizedHeads((header) => {
  console.log(`Finalized #${header.number}`);
});
```

#### Subscribe to account balance changes

```javascript
const address = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

await api.query.system.account(address, ({ data: balance }) => {
  console.log(`Free DALLA: ${balance.free.toString()}`);
});
```

#### Subscribe to storage changes

```javascript
// Watch specific storage key
await api.query.staking.currentEra((era) => {
  console.log(`Current era: ${era.toString()}`);
});

// Watch multiple keys
await api.queryMulti([
  [api.query.staking.currentEra],
  [api.query.staking.activeEra]
], ([currentEra, activeEra]) => {
  console.log(`Current: ${currentEra}, Active: ${activeEra.index}`);
});
```

### RPC Methods

#### Chain Methods

| Method | Description | Example |
|--------|-------------|---------|
| `chain_getBlock(hash?)` | Get block by hash | `api.rpc.chain.getBlock()` |
| `chain_getBlockHash(number)` | Get block hash | `api.rpc.chain.getBlockHash(1234567)` |
| `chain_getFinalizedHead()` | Get finalized head | `api.rpc.chain.getFinalizedHead()` |
| `chain_getHeader(hash?)` | Get block header | `api.rpc.chain.getHeader()` |

#### State Methods

| Method | Description | Example |
|--------|-------------|---------|
| `state_getStorage(key, hash?)` | Get storage value | `api.rpc.state.getStorage(key)` |
| `state_getMetadata(hash?)` | Get runtime metadata | `api.rpc.state.getMetadata()` |
| `state_getRuntimeVersion(hash?)` | Get runtime version | `api.rpc.state.getRuntimeVersion()` |
| `state_queryStorageAt(keys, hash)` | Query storage at block | `api.rpc.state.queryStorageAt([key], hash)` |

#### Author Methods

| Method | Description | Example |
|--------|-------------|---------|
| `author_submitExtrinsic(extrinsic)` | Submit transaction | `api.rpc.author.submitExtrinsic(tx)` |
| `author_pendingExtrinsics()` | Get pending txs | `api.rpc.author.pendingExtrinsics()` |
| `author_rotateKeys()` | Rotate session keys | `api.rpc.author.rotateKeys()` |

#### System Methods

| Method | Description | Example |
|--------|-------------|---------|
| `system_chain()` | Get chain name | `api.rpc.system.chain()` |
| `system_name()` | Get node name | `api.rpc.system.name()` |
| `system_version()` | Get node version | `api.rpc.system.version()` |
| `system_health()` | Get node health | `api.rpc.system.health()` |
| `system_peers()` | Get connected peers | `api.rpc.system.peers()` |

#### Custom Pallet Methods

**Economy Pallet**:
```javascript
// Get exchange rate (Oracle-based for merchant verification, NOT for bBZD peg)
const rate = await api.rpc.economy.getExchangeRate('DALLA', 'USD');

// Get treasury balance
const balance = await api.rpc.economy.treasuryBalance();
```

**Payroll Pallet**:
```javascript
// Generate payslip
const payslip = await api.rpc.payroll.generatePayslip({
  employerId: 'employer-123',
  employeeId: 'BZ-ID-12345',
  period: '2026-01'
});
```

**Nawal Pallet**:
```javascript
// Get training round status
const round = await api.rpc.nawal.getTrainingRound(42);

// Get PoUW score
const score = await api.rpc.nawal.getPoUWScore(validatorAddress, 42);
```

**Kinich Pallet**:
```javascript
// Get quantum job status
const job = await api.rpc.kinich.getQuantumJob('kinich-job-a3f7b2c9');

// Get PQW score
const score = await api.rpc.kinich.getPQWScore(validatorAddress);
```

## GraphQL API

### Schema

```graphql
type Query {
  block(number: Int, hash: String): Block
  blocks(from: Int!, to: Int!): [Block!]!
  account(address: String!): Account
  transaction(hash: String!): Transaction
  validators(active: Boolean): [Validator!]!
  proposals(status: ProposalStatus): [Proposal!]!
}

type Block {
  number: Int!
  hash: String!
  parentHash: String!
  timestamp: Int!
  validator: String!
  extrinsics: [Extrinsic!]!
  events: [Event!]!
}

type Account {
  address: String!
  balances: Balances!
  identity: Identity
  staking: StakingInfo
}

type Transaction {
  hash: String!
  blockNumber: Int!
  from: String!
  to: String
  status: TransactionStatus!
  events: [Event!]!
}
```

### Example Queries

#### Get block with extrinsics

```graphql
query {
  block(number: 1234567) {
    number
    hash
    timestamp
    validator
    extrinsics {
      hash
      call {
        pallet
        method
        args
      }
      signature
      fee
    }
  }
}
```

#### Get account with balances and staking

```graphql
query {
  account(address: "5GrwvaEF...") {
    address
    balances {
      dalla {
        free
        reserved
      }
      bbzd {
        free
        reserved
      }
    }
    staking {
      bonded
      nominations {
        validator
        amount
      }
      rewards {
        era
        amount
      }
    }
  }
}
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';

// Connect to blockchain
const provider = new WsProvider('wss://rpc.belizechain.org');
const api = await ApiPromise.create({ provider });

// Create account from seed
await cryptoWaitReady();
const keyring = new Keyring({ type: 'sr25519' });
const alice = keyring.addFromUri('//Alice');

// Transfer DALLA
const transfer = api.tx.economy.transfer(
  '5DTestULRhqKBKwZvZSrZjBwvZN2Mu9WkJJMYEDmLpkNuMQKB',
  1_000_000_000_000_000n  // 1,000 DALLA (12 decimals)
);

const hash = await transfer.signAndSend(alice);
console.log(`Transaction hash: ${hash}`);

// Query balance
const { data: balance } = await api.query.system.account(alice.address);
console.log(`Free DALLA: ${balance.free.toString()}`);
```

### Python

```python
from substrateinterface import SubstrateInterface, Keypair

# Connect to blockchain
substrate = SubstrateInterface(
    url="wss://rpc.belizechain.org",
    ss58_format=42,
    type_registry_preset='polkadot'
)

# Create keypair
keypair = Keypair.create_from_uri('//Alice')

# Transfer DALLA
call = substrate.compose_call(
    call_module='Economy',
    call_function='transfer',
    call_params={
        'recipient': '5DTestULRhqKBKwZvZSrZjBwvZN2Mu9WkJJMYEDmLpkNuMQKB',
        'amount': 1_000_000_000_000_000
    }
)

extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)
receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)

print(f"Extrinsic '{receipt.extrinsic_hash}' sent and included in block '{receipt.block_hash}'")

# Query balance
result = substrate.query('System', 'Account', [keypair.ss58_address])
print(f"Free DALLA: {result.value['data']['free']}")
```

### Rust

```rust
use subxt::{OnlineClient, PolkadotConfig};
use subxt::tx::PairSigner;
use sp_keyring::AccountKeyring;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod belizechain {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to blockchain
    let api = OnlineClient::<PolkadotConfig>::from_url("wss://rpc.belizechain.org").await?;
    
    // Create signer
    let signer = PairSigner::new(AccountKeyring::Alice.pair());
    
    // Transfer DALLA
    let transfer_tx = belizechain::tx().economy().transfer(
        AccountKeyring::Bob.to_account_id().into(),
        1_000_000_000_000_000u128
    );
    
    let hash = api.tx()
        .sign_and_submit_default(&transfer_tx, &signer)
        .await?;
    
    println!("Transaction hash: {:?}", hash);
    
    // Query balance
    let storage_query = belizechain::storage().system().account(
        &AccountKeyring::Alice.to_account_id()
    );
    
    if let Some(account) = api.storage().at_latest().await?.fetch(&storage_query).await? {
        println!("Free DALLA: {}", account.data.free);
    }
    
    Ok(())
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 1001 | Insufficient Balance | Account doesn't have enough funds |
| 1002 | Invalid Signature | Transaction signature verification failed |
| 1003 | Nonce Too Low | Transaction nonce already used |
| 1004 | Nonce Too High | Transaction nonce skipped values |
| 1010 | KYC Required | Operation requires KYC verification |
| 1011 | KYC Level Too Low | Higher KYC level needed |
| 2001 | Validator Not Found | Validator doesn't exist |
| 2002 | Already Bonded | Account already has staked funds |
| 2003 | Not Bonded | Account has no staked funds |
| 3001 | Proposal Not Found | Governance proposal doesn't exist |
| 3002 | Already Voted | Account already voted on proposal |
| 4001 | Pair Not Found | DEX trading pair doesn't exist |
| 4002 | Insufficient Liquidity | Not enough liquidity for trade |
| 5001 | Rate Limit Exceeded | Too many API requests |
| 5002 | Invalid API Key | API key is invalid or expired |

## Rate Limiting

Rate limit headers are included in all responses:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 847
X-RateLimit-Reset: 1738349340
```

When limit exceeded:
```json
{
  "error": "Rate limit exceeded",
  "code": 5001,
  "retry_after": 42
}
```

## Pagination

Paginated endpoints return:

```json
{
  "data": [...],
  "pagination": {
    "offset": 0,
    "limit": 50,
    "total": 1247,
    "has_more": true
  }
}
```

## Further Documentation

- [Pallet APIs - Core](../developer-guides/pallet-apis-core.md)
- [Pallet APIs - Financial](../developer-guides/pallet-apis-financial.md)
- [Pallet APIs - Infrastructure](../developer-guides/pallet-apis-infrastructure.md)
- [Pallet APIs - Services](../developer-guides/pallet-apis-services.md)
- [Smart Contract Development](../developer-guides/smart-contract-development.md)
