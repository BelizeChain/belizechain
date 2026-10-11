# API Reference

Complete API documentation for BelizeChain's REST, WebSocket, and RPC interfaces.

## What actually serves these APIs

| Surface | Served by | Notes |
|---|---|---|
| WebSocket / HTTP JSON-RPC | the node itself (`ceiba-node`) | Substrate RPC, plus `belizechain_getChainInfo` |
| REST, GraphQL, gateway JSON-RPC | `api-gateway` (`infra/api-gateway/`) | A separate service in front of the node |

The node implements exactly one `belizechain_*` RPC method today:
`belizechain_getChainInfo` (`node/src/rpc.rs`). The other `belizechain_*` names are
TODOs there because they need `*-rpc-runtime-api` crates and a runtime upgrade —
`belizechain_governanceProposals`, `belizechain_stakingValidators`,
`belizechain_bridgeTransactions`, `belizechain_oracleFeeds`. The gateway answers
those names over the node's standard RPC, so they work without a chain change, and
it delegates nothing back: `belizechain_getChainInfo` is **not** reimplemented
there, so its payload always comes from the node.

### Base URLs

| Environment | Gateway REST | Gateway GraphQL | Gateway RPC | Node WS |
|---|---|---|---|---|
| Mainnet | **Not published** (planned `https://api.belizechain.org`) | Not published | Not published | Not published (planned `wss://rpc.belizechain.org`) |
| Ceiba testnet | `https://testnet.belizechain.org/api/gateway/v1` | `https://testnet.belizechain.org/api/gateway/graphql` | `https://testnet.belizechain.org/api/gateway/rpc` | `wss://testnet.belizechain.org/ws` |
| Local gateway | `http://localhost:8080/v1` | `http://localhost:8080/graphql` | `http://localhost:8080/rpc` | `ws://localhost:9944` |

> **Endpoint status.** The mainnet hosts above do not resolve — `rpc.belizechain.org`,
> `graphql.belizechain.org` and `relayer.belizechain.org` return NXDOMAIN, and
> `api.belizechain.org` points at the documentation web host, not an API. They are
> planned names, not working services. The Ceiba paths are the real ones, served by
> nginx via the `api-gateway` compose service; confirm with
> `curl https://testnet.belizechain.org/api/gateway/health` before relying on them,
> since `/health` reports `degraded` (HTTP 503) while the node is unreachable.

Throughout this document the REST base URL is written as `$BELIZECHAIN_API`:

```bash
export BELIZECHAIN_API="https://testnet.belizechain.org/api/gateway"
```

## Authentication

There is **no self-service API-key registration**. Keys are provisioned by the
operator in the gateway's `API_KEYS` environment variable; there is no
`/v1/auth/register` endpoint and no key-issuance portal.

```bash
# Send a provisioned key as either header form:
curl "$BELIZECHAIN_API/v1/blocks/latest" -H "X-API-Key: <provisioned-key>"
curl "$BELIZECHAIN_API/v1/blocks/latest" -H "Authorization: Bearer <provisioned-key>"
```

Enforcement is a single deployment-wide switch, `REQUIRE_API_KEY`:

| `REQUIRE_API_KEY` | Behaviour |
|---|---|
| `false` (default) | Open — no key required. Only sensible on a private network. |
| `true` | Every `/v1`, `/graphql` and `/rpc` request must present a valid key. **The gateway refuses to start if `API_KEYS` is empty** rather than silently running open. |

`GET /health` is always unauthenticated so container health checks keep working.

### Rate Limits

One fixed-window budget per caller, configured by `RATE_LIMIT_PER_MINUTE`
(default `120`). Callers are keyed by API key when one is presented, otherwise by
client IP. `0` disables the limiter. Responses carry `X-RateLimit-Remaining` and
`X-RateLimit-Reset`, and an exhausted budget returns `429` with `Retry-After`.

There are no per-tier limits and no GraphQL complexity budget. `bzc_live_…`-style
keys and the Free/Developer/Business/Enterprise tiers that used to be documented
here never existed.

## REST API

Every response uses the same envelope, so a client needs one parser:

```json
{ "ok": true,  "data": <payload>,          "meta": { … }? }
{ "ok": false, "error": { "code": "…", "message": "…" } }
```

Amounts are **strings**, not numbers: a u128 balance with 12 decimals exceeds
JavaScript's safe integer range. Addresses are normalised to SS58 prefix `1981`
and rejected with `400` if they do not decode.

### Block Endpoints

#### GET /v1/blocks/latest

Get the latest block.

```bash
curl "$BELIZECHAIN_API/v1/blocks/latest"
```

**Response**:
```json
{
  "ok": true,
  "data": {
    "number": 13753,
    "hash": "0x1b988fee7f99aea41fb26591657425b868828e79776d8542fcca9409237543e4",
    "parentHash": "0x53ac83675b5204641812473b16caaac3ac2a358f9aa9709f7f9499f858908f42",
    "stateRoot": "0x9b80129b7f47f0abdcf3ab6236884af8ce298c31ada346ab1147a84bf06db842",
    "extrinsicsRoot": "0x493834dd99c8b93565e57160a4f4a7682c61530f93fdfb662de15fd6f560ec26",
    "timestamp": 1789479282000,
    "extrinsicCount": 2,
    "extrinsics": [
      {
        "index": 0,
        "hash": "0x5f2a…",
        "section": "timestamp",
        "method": "set",
        "signer": null,
        "args": { "now": "1789479282000" }
      }
    ],
    "events": [
      { "phase": "Initialization", "section": "system", "method": "ExtrinsicSuccess", "data": { … } }
    ]
  }
}
```

The block author is not included, and neither is `events_count` — count
`events` yourself. `timestamp` is milliseconds since the Unix epoch, `null` if the
block carries no timestamp inherent.

#### GET /v1/blocks/{block_number}

Get a block by number.

```bash
curl "$BELIZECHAIN_API/v1/blocks/13753"
```

Returns the same object as `/v1/blocks/latest`, or `404` with code `not_found` if
no block exists at that height.

There is no `include_extrinsics` / `include_events` toggle: extrinsics and events
are always included. Note the route order — `/blocks/latest` and `/blocks/range`
are matched before `/blocks/:number`, so `latest` is never parsed as a height.

#### GET /v1/blocks/range

Get a contiguous range of blocks.

```bash
curl "$BELIZECHAIN_API/v1/blocks/range?from=13700&to=13720"
```

**Response**:
```json
{
  "ok": true,
  "data": [ { "number": 13700, … } ],
  "meta": { "blockScannedWindow": 128 }
}
```

`from` and `to` are required and inclusive; `to` before `from` is a `400`
`invalid_range`. The range is silently capped at `SCAN_WINDOW` blocks, and a
`meta.note` is added when that happens.

### Account Endpoints

#### GET /v1/accounts/{address}

Get account information.

```bash
curl "$BELIZECHAIN_API/v1/accounts/r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur"
```

**Response**:
```json
{
  "ok": true,
  "data": {
    "address": "r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur",
    "nonce": 6,
    "consumers": 1,
    "providers": 1,
    "balances": {
      "free": "10000205457878000",
      "reserved": "0",
      "frozen": "1000000000000000",
      "flags": "170141183460469231731687303715884105728"
    },
    "identityId": null,
    "identityName": null,
    "verificationLevel": "None",
    "riskLevel": "Low",
    "restricted": false
  }
}
```

`balances` is the native DALLA balance from `System.Account`. bBZD is a separate
asset and is **not** included here — the `dalla`/`bbzd` nesting that used to be
documented does not exist. `flags` is the raw `ExtraFlags` bitfield.

`identityId` is `null` until the account registers a BelizeID; when set,
`identityName` is decoded from the identity record's `name`. `verificationLevel`
and `riskLevel` come from `Compliance.ComplianceStatusOf` and read `None`/`Low`
when the account has never been verified.

#### GET /v1/accounts/{address}/transactions

Get recent transactions signed by an account.

```bash
curl "$BELIZECHAIN_API/v1/accounts/r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur/transactions?limit=50&offset=0"
```

**Query Parameters**:
- `limit` (integer): Results per page, default `50`, max `500`
- `offset` (integer): Skip this many matches

**Response**:
```json
{
  "ok": true,
  "data": [
    {
      "blockNumber": 13753,
      "blockHash": "0x1b98…",
      "extrinsicIndex": 1,
      "hash": "0x5f2a…",
      "section": "balances",
      "method": "transferKeepAlive",
      "args": { "dest": "r1XAsf…", "value": "1000000000000" }
    }
  ],
  "meta": {
    "blockScannedWindow": 128,
    "total": 12,
    "returned": 12,
    "note": "Substrate has no reverse index, so only the most recent 128 blocks are searched. An indexer is required for full history."
  }
}
```

> This is a **bounded scan**, not a real index. `total` counts matches inside the
> scanned window only. The `type`, `from_block` and `to_block` filters that used to
> be documented here do not exist; those require an indexer.

### Transaction Endpoints

#### POST /v1/transactions/submit

Relay an extrinsic the **caller already signed**.

```bash
curl -X POST "$BELIZECHAIN_API/v1/transactions/submit" \
  -H "Content-Type: application/json" \
  -d '{ "extrinsic": "0x4502849ac…" }'
```

**Response**:
```json
{ "ok": true, "data": { "hash": "0xabcd…1234" } }
```

The gateway holds no keys and cannot sign anything. Build and sign the extrinsic
client-side, then submit the encoded bytes. Only `{ "extrinsic": "0x…" }` is
accepted — there is no `signature` or `estimated_fee` field, and the response is
the extrinsic hash only, not a block number or status. The payload is validated as
hex of at least 8 bytes before reaching the node, so a malformed body fails fast;
a node rejection comes back as `400` `submit_rejected`.

#### GET /v1/transactions/{hash}

Find a transaction by hash.

```bash
curl "$BELIZECHAIN_API/v1/transactions/0x5f2a…"
```

**Response**:
```json
{
  "ok": true,
  "data": {
    "blockNumber": 13753,
    "blockHash": "0x1b98…",
    "extrinsic": {
      "index": 1,
      "hash": "0x5f2a…",
      "section": "balances",
      "method": "transferKeepAlive",
      "signer": "r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur",
      "args": { "dest": "r1XAsf…", "value": "1000000000000" }
    }
  }
}
```

> Substrate has no reverse index from extrinsic hash to block, so this walks
> backwards over the most recent `SCAN_WINDOW` blocks. A miss is a `404` with code
> `transaction_not_found` and a message saying an indexer is required — not an
> empty success. The `fee_paid`, `status` and `events` fields that used to be
> documented here would each need an indexer.

### Staking Endpoints

#### GET /v1/staking/validators

Get the validator set.

```bash
curl "$BELIZECHAIN_API/v1/staking/validators?active=true"
```

**Response**:
```json
{
  "ok": true,
  "data": [
    {
      "account": "r1XAsfvRmUhzULW5h8Gh4V4TzfZejPGF8D245kjxw61wrqLag",
      "stake": "1000000000000000",
      "computeCapacity": 100,
      "location": "ceiba",
      "complianceScore": 100,
      "qualityScore": 30,
      "timelinessScore": 15
    }
  ],
  "meta": { "total": 1, "returned": 1 }
}
```

- `active=true` filters to entries with a non-zero `stake`; without it every
  registered entry is returned, including ones that have left.
- `location` is a UTF-8 string decoded from the validator's bounded byte vector,
  or `null` when empty.

There is **no** nominator model on this chain: no `identity`, `nominators_count`,
`commission`, `apy`, `uptime` or `validator_type`. Validators stake directly via
`staking.join_validators(stake, compute_capacity, location)` — see
[Validator Setup](../validators/validator-setup.md). Scoring fields come from
Proof-of-Useful-Work participation, not block production.

#### GET /v1/staking/rewards/{address}

Get an account's staking position and the per-epoch reward pools.

```bash
curl "$BELIZECHAIN_API/v1/staking/rewards/r1XAsfvRmUhzULW5h8Gh4V4TzfZejPGF8D245kjxw61wrqLag"
```

**Response**:
```json
{
  "ok": true,
  "data": {
    "address": "r1XAsfvRmUhzULW5h8Gh4V4TzfZejPGF8D245kjxw61wrqLag",
    "currentEpoch": 1,
    "lastClaimedEpoch": 0,
    "epochPools": [ { "epoch": 0, "pool": "0" }, { "epoch": 1, "pool": "0" } ],
    "validatorQuantumStats": {
      "jobsExecuted": 0,
      "avgAccuracy": 0,
      "totalComplexity": "0",
      "totalQubits": "0",
      "totalShots": "0",
      "quantumScore": 0
    }
  },
  "meta": {
    "note": "Rewards are pooled per epoch and distributed by the pallet; there is no per-account claimable balance, so this reports the epoch pool alongside the account's recorded contributions."
  }
}
```

> Rewards are pooled per epoch and distributed by the staking pallet — there is no
> per-account claimable balance and no per-era reward history to list. The
> `total_rewards` / `rewards[]` shape that used to be documented here described a
> claim-based model this chain does not have. `epochPools` is chain-wide, not
> filtered to the account.

### Governance Endpoints

#### GET /v1/governance/proposals

Get governance proposals.

```bash
curl "$BELIZECHAIN_API/v1/governance/proposals?status=active"
```

**Response**:
```json
{
  "ok": true,
  "data": [
    {
      "id": 42,
      "proposer": "r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur",
      "title": "Increase validator rewards by 10%",
      "description": "…",
      "proposalType": "Standard",
      "threshold": "SimpleMajority",
      "deposit": "10000000000000",
      "votingStart": 13700,
      "votingEnd": 14300,
      "status": "Active"
    }
  ],
  "meta": { "total": 1, "returned": 1 }
}
```

`title` and `description` are UTF-8 decoded from their bounded byte vectors. The
optional `status` filter matches the on-chain variant name case-insensitively and
is applied after the read, so `meta.total` reports the unfiltered count.

There is no vote tally, no `hash`, no `type`, and no conviction breakdown: the
pallet's `vote_tally` is not exposed here, and `proposalType` / `threshold` /
`status` are variant names, not free text.

#### POST /v1/governance/vote

Relay a signed vote extrinsic.

```bash
curl -X POST "$BELIZECHAIN_API/v1/governance/vote" \
  -H "Content-Type: application/json" \
  -d '{ "extrinsic": "0x4502849ac…" }'
```

**Response**:
```json
{ "ok": true, "data": { "hash": "0xabcd…1234" } }
```

Same relay rule as `/v1/transactions/submit`: sign the extrinsic client-side. The
call is `governance.castVote(proposalId, voteChoiceIndex, conviction)`, where
`voteChoiceIndex` is `0` Aye / `1` Nay / `2` Abstain. There is no `proposal_id` /
`vote` / `conviction` / `amount` JSON body.

### DEX Endpoints (BelizeX)

#### GET /v1/dex/pairs

Get trading pairs.

```bash
curl "$BELIZECHAIN_API/v1/dex/pairs"
```

**Response**:
```json
{
  "ok": true,
  "data": [
    {
      "baseAsset": "DALLA",
      "quoteAsset": "BBZD",
      "baseReserve": "0",
      "quoteReserve": "0",
      "totalLpTokens": "0",
      "feeRate": 30,
      "active": true
    }
  ]
}
```

`feeRate` is in basis points (30 = 0.3%). There is no `id`, `price`,
`volume_24h` or `liquidity_usd`: price is derived from the reserves, and the
others need an indexer. Pairs are keyed `(baseAsset, quoteAsset)` on chain.

#### GET /v1/dex/orders

Get the limit order book.

```bash
curl "$BELIZECHAIN_API/v1/dex/orders?depth=20"
```

**Response**:
```json
{
  "ok": true,
  "data": [
    {
      "id": 1,
      "creator": "r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur",
      "pair": "DALLA/BBZD",
      "orderType": "Limit",
      "amount": "1000000000000",
      "price": "1980000",
      "remaining": "1000000000000",
      "expiresAt": 14000,
      "isTourismOrder": false
    }
  ],
  "meta": { "total": 1, "returned": 1 }
}
```

`depth` defaults to `50` and is capped at `200`. Orders come back as stored, not
as a `bids`/`asks` ladder: there is no `spread` or `last_price`, because
aggregating them belongs to the client. `pair` is rendered as `base/quote`.

## WebSocket API

### Connection

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

// Replace with the operator-provided RPC endpoint (the public host above is not live yet).
const provider = new WsProvider('<operator-provided-rpc-url>');
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
const address = 'r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur';

await api.query.system.account(address, ({ data: balance }) => {
  console.log(`Free DALLA: ${balance.free.toString()}`);
});
```

#### Subscribe to storage changes

```javascript
// Watch specific storage key
await api.query.staking.currentEpoch((epoch) => {
  console.log(`Current epoch: ${epoch.toString()}`);
});

// Watch multiple keys
await api.queryMulti([
  [api.query.staking.currentEpoch]
], ([currentEpoch]) => {
  console.log(`Current epoch: ${currentEpoch}`);
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

> **Not implemented.** The node exposes exactly ONE custom RPC method,
> `belizechain_getChainInfo`. There are no `economy.*`, `payroll.*`, `nawal.*`
> or `kinich.*` RPC namespaces — pallet state is read through `state_getStorage`
> / `api.query.<pallet>.<item>`, and the sibling-service APIs are the
> nginx-proxied HTTP endpoints (`/api/nawal`, `/api/kinich`, `/api/pakit`).

```javascript
// The only custom RPC the node registers:
const info = await api.rpc.belizechain.getChainInfo();
```

#### Gateway RPC Methods

The `api-gateway` service exposes additional `belizechain_*` methods over the
node's standard RPC, at `POST $BELIZECHAIN_API/rpc`. They do not require a runtime
upgrade, which the node-side TODOs above do.

| Method | Params | Returns |
|---|---|---|
| `belizechain_getChainStatus` | — | Chain/node identity, spec version, head, peers |
| `belizechain_latestBlock` | — | Latest block with extrinsics and events |
| `belizechain_block` | `number` | Block by number |
| `belizechain_accountInfo` | `address` | Balances, nonce, identity, compliance |
| `belizechain_stakingValidators` | — | Validator set |
| `belizechain_governanceProposals` | — | Proposals |
| `belizechain_tradingPairs` | — | BelizeX pairs |
| `belizechain_orders` | — | BelizeX order book |
| `belizechain_submitExtrinsic` | `0x…` | Relay of a **caller-signed** extrinsic |

`belizechain_getChainInfo` is deliberately absent from the gateway — the node
serves it, and answering the same method name with a different payload would be a
silent incompatibility. `system_chain`, `system_name`, `system_version` and
`system_health` are proxied for convenience.

```javascript
// Gateway: the methods the node has not implemented yet.
const proposals = await fetch('https://testnet.belizechain.org/api/gateway/rpc', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'belizechain_governanceProposals', params: [] }),
}).then((r) => r.json());
```

## GraphQL API

### Schema

Query-only. There is no `Mutation` type, and `mutation { … }` is rejected by the
executor — the gateway holds no keys.

Call arguments and raw event data are deliberately absent from the schema: they are
arbitrary JSON, which would need a custom scalar, and they are already available
structurally over REST and JSON-RPC.

```graphql
type Query {
  status: Status!
  block(number: Int, hash: String): Block
  latestBlock: Block!
  blocks(from: Int!, to: Int!): [Block!]!
  account(address: String!): Account
  transaction(hash: String!): TransactionInclusion
  validators: [Validator!]!
  proposals: [Proposal!]!
  tradingPairs: [TradingPair!]!
  orders: [Order!]!
}

type Status {
  chain: String!
  nodeName: String!
  nodeVersion: String!
  ss58Prefix: Int!
  tokenSymbol: String
  tokenDecimals: Int!
  specVersion: Int!
  bestBlock: Int!
  finalizedBlock: Int!
  peers: Int!
  isSyncing: Boolean!
}

type Block {
  number: Int!
  hash: String!
  parentHash: String!
  stateRoot: String!
  extrinsicsRoot: String!
  timestamp: Int
  extrinsicCount: Int!
  extrinsics: [Extrinsic!]!
  events: [Event!]!
}

type Extrinsic {
  index: Int!
  hash: String
  section: String!
  method: String!
  signer: String
}

type Event {
  phase: String!
  section: String!
  method: String!
}

type Balances {
  free: String!
  reserved: String!
  frozen: String!
  flags: String!
}

type Account {
  address: String!
  nonce: Int!
  consumers: Int!
  providers: Int!
  balances: Balances!
  identityId: Int
  identityName: String
  verificationLevel: String
  riskLevel: String
  restricted: Boolean
}

type TransactionInclusion {
  blockNumber: Int!
  blockHash: String!
  extrinsicIndex: Int!
  section: String!
  method: String!
}

type Validator {
  account: String!
  stake: String!
  computeCapacity: Int!
  location: String
  complianceScore: Int!
  qualityScore: Int!
  timelinessScore: Int!
}

type Proposal {
  id: Int!
  proposer: String!
  title: String!
  description: String!
  proposalType: String!
  threshold: String!
  deposit: String!
  votingStart: Int!
  votingEnd: Int!
  status: String
}

type TradingPair {
  baseAsset: String!
  quoteAsset: String!
  baseReserve: String!
  quoteReserve: String!
  totalLpTokens: String!
  feeRate: Int!
  active: Boolean!
}

type Order {
  id: Int!
  creator: String!
  pair: String!
  orderType: String!
  amount: String!
  price: String!
  remaining: String!
  expiresAt: Int!
  isTourismOrder: Boolean!
}
```

> `tokenSymbol` is nullable because this runtime advertises no `system.properties`
> at all — `system_properties` returns `{}`. The SS58 prefix (`1981`) and token
> decimals (`12`) come from the runtime constants, but there is no equivalent
> constant for the symbol, so it is reported as `null` rather than invented. The
> node's own `belizechain_getChainInfo` states the currency as `DALLA`.

> Amounts are strings, not numbers. A u128 balance with 12 decimals exceeds
> JavaScript's safe integer range, so serialising them as numbers would silently
> corrupt large values.

### Example Queries

#### Chain status

```graphql
query {
  status {
    chain
    ss58Prefix
    specVersion
    bestBlock
    finalizedBlock
  }
}
```

#### Latest block with its extrinsics and events

```graphql
query {
  latestBlock {
    number
    hash
    timestamp
    extrinsicCount
    extrinsics {
      index
      hash
      section
      method
      signer
    }
    events {
      phase
      section
      method
    }
  }
}
```

#### Account with balances and compliance

```graphql
query {
  account(address: "r1XAsfvRmUhzULW5h8Gh4V4TzfZejPGF8D245kjxw61wrqLag") {
    address
    nonce
    balances {
      free
      reserved
      frozen
    }
    identityName
    verificationLevel
    riskLevel
    restricted
  }
}
```

#### Validators and trading pairs

```graphql
query {
  validators {
    account
    stake
    computeCapacity
    location
    qualityScore
    timelinessScore
  }
  tradingPairs {
    baseAsset
    quoteAsset
    baseReserve
    quoteReserve
    feeRate
    active
  }
}
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';

// Connect to blockchain (operator-provided RPC endpoint)
const provider = new WsProvider('<operator-provided-rpc-url>');
const api = await ApiPromise.create({ provider });

// Create account from seed
await cryptoWaitReady();
const keyring = new Keyring({ type: 'sr25519' });
const alice = keyring.addFromUri('//Alice');

// Transfer DALLA (native currency → balances pallet, not economy)
const transfer = api.tx.balances.transferKeepAlive(
  'r1XAsfvRmUhzULW5h8Gh4V4TzfZejPGF8D245kjxw61wrqLag',
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

# Connect to blockchain (operator-provided RPC endpoint).
# 1981 is this chain's SS58 prefix (see System.Ss58Prefix). Using 42 here would
# encode and display addresses under the wrong prefix.
substrate = SubstrateInterface(
    url="<operator-provided-rpc-url>",
    ss58_format=1981,
    type_registry_preset='polkadot'
)

# Create keypair
keypair = Keypair.create_from_uri('//Alice')

# Transfer DALLA. The native currency lives in the balances pallet — the economy
# pallet has no `transfer` call (its only transfer is `transfer_bbzd`).
call = substrate.compose_call(
    call_module='Balances',
    call_function='transfer_keep_alive',
    call_params={
        'dest': 'r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur',
        'value': 1_000_000_000_000_000
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
    // Connect to blockchain (operator-provided RPC endpoint)
    let api = OnlineClient::<PolkadotConfig>::from_url("<operator-provided-rpc-url>").await?;
    
    // Create signer
    let signer = PairSigner::new(AccountKeyring::Alice.pair());
    
    // Transfer DALLA. DALLA is the native currency, so this is a balances call —
    // there is no `economy().transfer`.
    let transfer_tx = belizechain::tx().balances().transfer_keep_alive(
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

The gateway returns string codes, not numbered ones. Distinct surfaces have
distinct shapes, so read the `code` field rather than a status alone:

**REST** — `{ "ok": false, "error": { "code": …, "message": … } }`

| HTTP | `code` | Cause |
|---|---|---|
| 400 | `invalid_json` | Body is not valid JSON |
| 400 | `missing_param` | A required path or query parameter is absent |
| 400 | `invalid_parameter` | A parameter is present but not a valid integer |
| 400 | `invalid_range` | `to` is before `from` |
| 400 | `invalid_address` | The address does not decode as SS58 |
| 400 | `missing_extrinsic` | Write endpoint called without `{ "extrinsic": "0x…" }` |
| 400 | `submit_rejected` | The node refused the relayed extrinsic |
| 401 | `unauthorized` | Missing or invalid API key while `REQUIRE_API_KEY=true` |
| 404 | `not_found` | No such route |
| 404 | `transaction_not_found` | Not inside the scanned window |
| 429 | `rate_limited` | `RATE_LIMIT_PER_MINUTE` exhausted; see `Retry-After` |
| 500 | `internal_error` | Unexpected failure; details are in the service log |
| 503 | `chain_unavailable` | The node is unreachable |

**JSON-RPC** — standard codes, always HTTP 200 for method-level failures:

| `code` | Meaning |
|---|---|
| `-32700` | Parse error |
| `-32600` | Invalid request |
| `-32601` | Method not found |
| `-32602` | Invalid params |
| `-32603` | Internal error |
| `-32000` | Server error (chain call failed) |

> The `1001`–`5002` table that used to be here was invented. There is no numeric
error registry, and the pallet-level failures it described (Insufficient Balance,
Already Bonded, KYC Required) surface as Substrate dispatch errors inside a
`submit_rejected` message, not as gateway codes.

## Rate Limiting

Two headers are set on every limited response:

```http
X-RateLimit-Remaining: 42
X-RateLimit-Reset: 1738349340
```

There is no `X-RateLimit-Limit` header. When the budget is exhausted the response
is `429` in the normal REST envelope, plus `Retry-After`:

```json
{
  "ok": false,
  "error": { "code": "rate_limited", "message": "more than 120 requests per minute" }
}
```

## Pagination

Only two endpoints paginate, and both are bounded scans rather than true
pageable collections. They return the REST envelope with `meta`:

```json
{
  "ok": true,
  "data": [ … ],
  "meta": {
    "blockScannedWindow": 128,
    "total": 12,
    "returned": 12,
    "note": "Substrate has no reverse index, so only the most recent 128 blocks are searched. An indexer is required for full history."
  }
}
```

- `GET /v1/accounts/{address}/transactions` — `limit` (default 50, max 500) and `offset`
- `GET /v1/dex/orders` — `depth` (default 50, max 200)

`total` counts matches inside the scanned window only. It is not a chain-wide
count, and `has_more` does not exist.

## Further Documentation

- [Pallet APIs - Core](../developer-guides/pallet-apis-core.md)
- [Pallet APIs - Financial](../developer-guides/pallet-apis-financial.md)
- [Pallet APIs - Infrastructure](../developer-guides/pallet-apis-infrastructure.md)
- [Pallet APIs - Services](../developer-guides/pallet-apis-services.md)
- [Smart Contract Development](../developer-guides/smart-contract-development.md)
