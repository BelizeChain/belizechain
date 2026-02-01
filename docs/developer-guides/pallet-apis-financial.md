# Financial Pallet APIs

**Staking • BelizeX • Treasury**

Comprehensive API reference for BelizeChain's financial infrastructure pallets.

---

## Staking Pallet

**Proof of Useful Work (PoUW) consensus with federated learning integration**

### Extrinsics

#### `bond`
Bond DALLA for staking

```rust
pub fn bond(
    origin: OriginFor<T>,
    controller: T::AccountId,
    value: BalanceOf<T>,
    payee: RewardDestination<T::AccountId>
) -> DispatchResult
```

**Parameters:**
- `controller`: Account to control staking actions (can be same as stash)
- `value`: Amount of DALLA to bond (minimum: 1000 DALLA)
- `payee`: Reward destination (`Staked`, `Stash`, `Controller`, or custom account)

**Events:**
- `Bonded(AccountId, Balance)`

**Weights:** 55M + 3 reads + 2 writes

```javascript
// JavaScript example
await api.tx.staking.bond(
  controllerAccount,
  1000_000_000_000_000n,  // 1000 DALLA
  { Staked: null }  // Auto-compound rewards
).signAndSend(stashAccount);
```

#### `report_training`
Report federated learning contribution (Nawal integration)

```rust
pub fn report_training(
    origin: OriginFor<T>,
    session_id: BoundedVec<u8, ConstU32<64>>,
    accuracy: u32,  // Basis points (10000 = 100%)
    time_seconds: u32,
    privacy_verified: bool
) -> DispatchResult
```

**Scoring:**
- **Quality** (40%): Model accuracy improvement
- **Timeliness** (30%): Submission before deadline
- **Honesty** (30%): Privacy compliance (differential privacy verified)

**Rewards:**
- Base: 50 DALLA per session
- Quality bonus: up to 200 DALLA for >95% accuracy
- Timeliness bonus: 100 DALLA if within first 10% of deadline
- Total max: 350 DALLA per session

**Events:**
- `TrainingReportSubmitted(AccountId, Vec<u8>, u32, u32)`
- `TrainingRewardIssued(AccountId, Balance, TrainingScore)`

**Weights:** 80M + 5 reads + 3 writes

```python
# Python example (Nawal → Blockchain)
from belizechain import substrate

receipt = substrate.compose_call(
    call_module='Staking',
    call_function='report_training',
    call_params={
        'session_id': 'nawal_session_12345',
        'accuracy': 9450,  # 94.5%
        'time_seconds': 3600,  # 1 hour
        'privacy_verified': True
    }
)
```

#### `validate`
Declare intention to validate blocks

```rust
pub fn validate(
    origin: OriginFor<T>,
    commission: Perbill  // Validator commission (0-100%)
) -> DispatchResult
```

**Requirements:**
- Minimum stake: 10,000 DALLA
- KYC level: Verified or Enhanced
- Active Nawal node (federated learning participation)

**Events:**
- `ValidatorRegistered(AccountId, Perbill)`

**Weights:** 45M + 2 reads + 1 write

#### `nominate`
Nominate validators to support

```rust
pub fn nominate(
    origin: OriginFor<T>,
    targets: Vec<T::AccountId>
) -> DispatchResult
```

**Parameters:**
- `targets`: Up to 16 validator accounts to nominate

**Reward sharing:** Nominator earns (1 - validator_commission) × rewards

**Events:**
- `Nominated(AccountId, Vec<AccountId>)`

**Weights:** 60M + 3 reads + 2 writes

### Storage

#### `Bonded`
```rust
pub type Bonded<T> = StorageMap<
    _,
    Twox64Concat,
    T::AccountId,  // Stash account
    T::AccountId,  // Controller account
    OptionQuery
>;
```

#### `Ledger`
```rust
pub type Ledger<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,  // Controller account
    StakingLedger<T::AccountId, BalanceOf<T>>,
    OptionQuery
>;

pub struct StakingLedger<AccountId, Balance> {
    pub stash: AccountId,
    pub total: Balance,
    pub active: Balance,
    pub unlocking: BoundedVec<UnlockChunk<Balance>, ConstU32<32>>
}
```

#### `TrainingScores`
```rust
pub type TrainingScores<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,  // Validator
    Blake2_128Concat, BoundedVec<u8, ConstU32<64>>,  // Session ID
    TrainingScore,
    OptionQuery
>;

pub struct TrainingScore {
    pub quality: u8,      // 0-100
    pub timeliness: u8,   // 0-100
    pub honesty: u8,      // 0-100
    pub total_reward: Balance
}
```

---

## BelizeX Pallet

**Decentralized exchange with order book and AMM**

### Extrinsics

#### `create_trading_pair`
Create new trading pair

```rust
pub fn create_trading_pair(
    origin: OriginFor<T>,
    asset_a: AssetId,
    asset_b: AssetId,
    fee_rate: Permill  // 0.1% = 1000, 0.3% = 3000
) -> DispatchResult
```

**Requirements:**
- Both assets must be registered
- Pair doesn't already exist
- Deposit: 100 DALLA (refundable on sufficient liquidity)

**Events:**
- `TradingPairCreated(AssetId, AssetId, Permill)`

**Weights:** 50M + 3 reads + 2 writes

```typescript
// TypeScript example
await api.tx.belizex.createTradingPair(
  'DALLA',
  'bBZD',
  3000  // 0.3% fee
).signAndSend(creator);
```

#### `place_limit_order`
Place limit order on order book

```rust
pub fn place_limit_order(
    origin: OriginFor<T>,
    pair_id: TradingPairId,
    side: OrderSide,  // Buy or Sell
    price: BalanceOf<T>,
    amount: BalanceOf<T>
) -> DispatchResult
```

**Matching:**
- Price-time priority (best price, then earliest timestamp)
- Partial fills allowed
- Unfilled orders stay in book

**Events:**
- `LimitOrderPlaced(AccountId, TradingPairId, OrderSide, Balance, Balance)`
- `OrderMatched(OrderId, OrderId, Balance)` - (maker, taker, filled_amount)

**Weights:** 70M + 5 reads + 4 writes

```javascript
// Place sell order: 1000 DALLA at 1.05 bBZD
await api.tx.belizex.placeLimitOrder(
  pairId,
  'Sell',
  1050_000_000_000n,   // Price: 1.05 bBZD (3 decimals precision)
  1000_000_000_000_000n  // Amount: 1000 DALLA
).signAndSend(trader);
```

#### `add_liquidity`
Add liquidity to AMM pool

```rust
pub fn add_liquidity(
    origin: OriginFor<T>,
    pair_id: TradingPairId,
    amount_a: BalanceOf<T>,
    amount_b: BalanceOf<T>,
    min_liquidity: BalanceOf<T>
) -> DispatchResult
```

**Returns:** LP tokens proportional to share of pool

**Formula:**
```
liquidity_minted = (amount_a / reserve_a) × total_supply
```

**Events:**
- `LiquidityAdded(AccountId, TradingPairId, Balance, Balance, Balance)` - (provider, pair, amount_a, amount_b, lp_tokens)

**Weights:** 65M + 4 reads + 3 writes

#### `swap`
Execute swap via AMM (constant product formula)

```rust
pub fn swap(
    origin: OriginFor<T>,
    pair_id: TradingPairId,
    asset_in: AssetId,
    amount_in: BalanceOf<T>,
    min_amount_out: BalanceOf<T>
) -> DispatchResult
```

**Formula (x × y = k):**
```
amount_out = (amount_in × 997 × reserve_out) / (reserve_in × 1000 + amount_in × 997)
// 0.3% fee (997/1000)
```

**Slippage protection:** Reverts if `amount_out < min_amount_out`

**Events:**
- `Swapped(AccountId, TradingPairId, AssetId, AssetId, Balance, Balance)` - (trader, pair, asset_in, asset_out, amount_in, amount_out)

**Weights:** 75M + 4 reads + 3 writes

```python
# Python example
tx = api.compose_call(
    call_module='BelizeX',
    call_function='swap',
    call_params={
        'pair_id': 0,
        'asset_in': 'DALLA',
        'amount_in': 100_000_000_000_000,  # 100 DALLA
        'min_amount_out': 95_000_000_000_000  # Accept up to 5% slippage
    }
)
```

### Storage

#### `TradingPairs`
```rust
pub type TradingPairs<T> = StorageMap<
    _,
    Blake2_128Concat,
    TradingPairId,
    TradingPairInfo<BalanceOf<T>>,
    OptionQuery
>;

pub struct TradingPairInfo<Balance> {
    pub asset_a: AssetId,
    pub asset_b: AssetId,
    pub reserve_a: Balance,
    pub reserve_b: Balance,
    pub total_supply: Balance,  // LP tokens
    pub fee_rate: Permill
}
```

#### `OrderBook`
```rust
pub type OrderBook<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, TradingPairId,
    Blake2_128Concat, OrderId,
    LimitOrder<T::AccountId, BalanceOf<T>, T::BlockNumber>,
    OptionQuery
>;

pub struct LimitOrder<AccountId, Balance, BlockNumber> {
    pub trader: AccountId,
    pub side: OrderSide,
    pub price: Balance,
    pub amount: Balance,
    pub filled: Balance,
    pub created_at: BlockNumber
}
```

#### `AssetRegistry`
```rust
pub type AssetRegistry<T> = StorageMap<
    _,
    Blake2_128Concat,
    AssetId,
    AssetMetadata,
    OptionQuery
>;

pub struct AssetMetadata {
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub symbol: BoundedVec<u8, ConstU32<8>>,
    pub decimals: u8,
    pub total_supply: Balance
}
```

---

## Treasury Pallet

**Multi-sig national treasury with governance oversight**

### Extrinsics

#### `propose_spend`
Propose treasury spending

```rust
pub fn propose_spend(
    origin: OriginFor<T>,
    value: BalanceOf<T>,
    beneficiary: T::AccountId,
    description: BoundedVec<u8, ConstU32<256>>
) -> DispatchResult
```

**Requirements:**
- Proposer bond: 5% of value (minimum 100 DALLA)
- Maximum proposal: 10% of treasury balance
- Governance approval needed (via Governance pallet)

**Events:**
- `ProposalSubmitted(ProposalIndex, AccountId, Balance, AccountId)`

**Weights:** 60M + 3 reads + 2 writes

```rust
// Rust example
let proposal_id = Treasury::propose_spend(
    Origin::signed(proposer),
    50_000 * DALLA,
    infrastructure_account,
    b"Road construction in Cayo District".to_vec().try_into().unwrap()
)?;
```

#### `approve_proposal`
Approve spending proposal (requires governance vote + multi-sig)

```rust
pub fn approve_proposal(
    origin: OriginFor<T>,
    proposal_id: ProposalIndex
) -> DispatchResult
```

**Requirements:**
- Governance referendum passed
- 4-of-7 multi-sig signatures from treasury council
- Sufficient treasury balance

**Events:**
- `ProposalApproved(ProposalIndex)`
- `FundsTransferred(ProposalIndex, AccountId, Balance)`

**Weights:** 80M + 5 reads + 4 writes

#### `reject_proposal`
Reject spending proposal (slashes proposer bond)

```rust
pub fn reject_proposal(
    origin: OriginFor<T>,
    proposal_id: ProposalIndex,
    reason: BoundedVec<u8, ConstU32<128>>
) -> DispatchResult
```

**Effects:**
- Proposer bond slashed and sent to treasury
- Proposal removed from queue

**Events:**
- `ProposalRejected(ProposalIndex, Vec<u8>)`

**Weights:** 50M + 3 reads + 3 writes

### Storage

#### `Proposals`
```rust
pub type Proposals<T> = StorageMap<
    _,
    Blake2_128Concat,
    ProposalIndex,
    TreasuryProposal<T::AccountId, BalanceOf<T>, T::BlockNumber>,
    OptionQuery
>;

pub struct TreasuryProposal<AccountId, Balance, BlockNumber> {
    pub proposer: AccountId,
    pub value: Balance,
    pub beneficiary: AccountId,
    pub bond: Balance,
    pub description: BoundedVec<u8, ConstU32<256>>,
    pub submitted_at: BlockNumber,
    pub governance_vote_id: Option<u32>
}
```

#### `TreasuryBalance`
```rust
pub type TreasuryBalance<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;
```

Current treasury balance (funded by transaction fees + inflation)

#### `TreasuryCouncil`
```rust
pub type TreasuryCouncil<T> = StorageValue<_, BoundedVec<T::AccountId, ConstU32<7>>, ValueQuery>;
```

7 council members requiring 4-of-7 signatures for approvals

---

## Type Definitions

```rust
// Staking
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RewardDestination<AccountId> {
    Staked,      // Auto-compound
    Stash,       // Send to stash account
    Controller,  // Send to controller account
    Account(AccountId)  // Custom account
}

// BelizeX
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OrderSide {
    Buy,
    Sell
}

pub type AssetId = BoundedVec<u8, ConstU32<8>>;  // e.g., "DALLA", "bBZD"
pub type TradingPairId = u32;
pub type OrderId = u64;
```

---

## Performance Benchmarks

### Staking
```
bond(): 55M gas
report_training(): 80M gas
validate(): 45M gas

Throughput: ~180 staking ops/second
```

### BelizeX
```
swap(): 75M gas (AMM)
place_limit_order(): 70M gas
add_liquidity(): 65M gas

Throughput:
- AMM swaps: ~130/second
- Limit orders: ~140/second
- Liquidity ops: ~150/second
```

### Treasury
```
propose_spend(): 60M gas
approve_proposal(): 80M gas

Multi-sig latency: ~30 seconds (collect 4 signatures)
```

---

## Related Documentation

- [Core Pallet APIs](./pallet-apis-core.md)
- [Infrastructure Pallet APIs](./pallet-apis-infrastructure.md)
- [Economics Overview](../economics/tokenomics.md)
- [Staking Guide](../validators/staking.md)
- [BelizeX Trading Guide](../user-guides/belizex-trading.md)
