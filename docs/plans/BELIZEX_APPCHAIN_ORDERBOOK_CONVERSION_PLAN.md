# BelizeX DEX — Appchain Orderbook Conversion Plan

**Created:** 2026-03-24  
**Status:** Future — Triggered by volume milestone  
**Owner:** BelizeChain Core Team  
**Classification:** Executive Plan — Strategic Infrastructure Upgrade  
**Priority:** Deferred (Phase 4 of DEX Roadmap)  
**Estimated Engineering Effort:** 6–9 months (3 engineers)

---

## Executive Summary

BelizeX currently operates as a **pure Automated Market Maker (AMM)** using the constant product formula (x × y = k), modeled after Uniswap V2. This plan documents the full conversion path to a **hybrid AMM + Central Limit Order Book (CLOB)** architecture, potentially running as a dedicated **appchain** (application-specific blockchain) optimized for high-throughput order matching.

**This conversion is NOT needed now.** The current AMM is the correct architecture for BelizeX's launch phase — low liquidity, limited trading pairs, and no active market makers. The conversion should be triggered when specific volume and user thresholds are met (see Section 2: Trigger Criteria).

The plan is structured as four sequential phases:
1. **Off-chain limit order matcher** (bot-driven, minimal chain changes)
2. **Concentrated liquidity** (Uniswap V3-style capital efficiency)
3. **On-chain CLOB pallet** (native order matching in Substrate runtime)
4. **Appchain separation** (dedicated chain for order execution)

---

## Table of Contents

1. [Current Architecture Assessment](#1-current-architecture-assessment)
2. [Trigger Criteria — When to Begin](#2-trigger-criteria--when-to-begin)
3. [Phase 1 — Off-Chain Limit Order Matcher](#3-phase-1--off-chain-limit-order-matcher)
4. [Phase 2 — Concentrated Liquidity Upgrade](#4-phase-2--concentrated-liquidity-upgrade)
5. [Phase 3 — On-Chain CLOB Pallet](#5-phase-3--on-chain-clob-pallet)
6. [Phase 4 — Appchain Separation](#6-phase-4--appchain-separation)
7. [Infrastructure & Cost Projections](#7-infrastructure--cost-projections)
8. [Risk Assessment](#8-risk-assessment)
9. [Competitive Landscape](#9-competitive-landscape)
10. [Success Metrics](#10-success-metrics)
11. [Appendix A — Current BelizeX Feature Matrix](#appendix-a--current-belizex-feature-matrix)
12. [Appendix B — Appchain Architecture Reference](#appendix-b--appchain-architecture-reference)
13. [Appendix C — Migration Checklist](#appendix-c--migration-checklist)

---

## 1. Current Architecture Assessment

### 1.1 Trading Model

BelizeX implements the **constant product AMM** formula:

```
amount_out = (amount_after_fee × reserve_out) / (reserve_in + amount_after_fee)
```

All trades execute atomically against liquidity pools. There is no order matching, no priority queue, and no mempool ordering concerns.

### 1.2 Existing Trading Pairs

| Pair | Fee Rate | Notes |
|------|----------|-------|
| DALLA / bBZD | 30 bps (0.3%) | Primary pair |
| TourismDALLA / bBZD | 15 bps (0.15%) | Tourism-optimized |
| WUSDC / bBZD | 30 bps (0.3%) | Remittance/cross-border |

### 1.3 Fee Architecture

Multi-tier fee structure with three discount layers:

- **Base fee**: 30 bps (configurable per pair)
- **Volume tier discount**: 0–6 bps (Oracle-driven, 4 tiers)
- **Tourism merchant discount**: ~5 bps additional (for verified merchants)
- **Absolute minimum floor**: 10 bps (hardcoded safety)
- **Fee split**: Protocol fee to Treasury + remainder stays in pool for LPs

### 1.4 Existing Limit Order Infrastructure

`place_limit_order` (call_index 4) **exists but is non-functional for matching**:

- **Storage**: `OrderBook` StorageMap stores `OrderEntry` structs with order_id, creator, pair, order_type, amount, price, remaining, expires_at
- **Missing**: No matching engine, no execution logic, no `remaining` field decrement
- **Spam protection**: `MaxOrdersPerAccount` constant caps per-account orders
- **Cancellation**: `cancel_order` (call_index 10) enforces creator-only access (audit fix H-24)

**Key takeaway**: The storage scaffolding exists for limit orders. Phase 1 leverages this directly rather than rebuilding.

### 1.5 Current Scalability Profile

| Metric | Value | Constraint |
|--------|-------|-----------|
| Block time | 6 seconds | Substrate BABE consensus |
| Max TPS | ~2,000 | Runtime weight budget |
| Multi-hop max | 5 legs | Hardcoded path limit |
| LP balance query | O(n) per user pair count | Off-chain only, acceptable |
| Trading pair count | Unbounded | Governance-gated creation |
| Oracle dependency | Hard requirement for WUSDC/bBZD | Rejects if Oracle unavailable |
| AKS resources | 2 vCPU, 8GB RAM (shared) | $75/month budget ceiling |

### 1.6 Audit Status

All critical audit findings have been remediated:

| Fix Code | Issue | Status |
|----------|-------|--------|
| X-1 | Per-user LP balance tracking | FIXED |
| X-2 | Multihop token flow | FIXED |
| M47 | LP token proportional minting | FIXED |
| H-3, H-4 | Multihop fee/Oracle consistency | FIXED |
| H-24 | Order cancellation access control | FIXED |
| CW-1 | Sanctions checking | FIXED |
| DOS-008 | Weight reporting in on_idle | FIXED |
| E-7 | Dead DailyVolume storage removal | FIXED |

---

## 2. Trigger Criteria — When to Begin

**Do NOT begin any phase until the corresponding trigger criteria are met.** Premature conversion wastes engineering resources and adds complexity without user benefit.

### Phase 1 Triggers (Off-Chain Matcher)

Begin when **ANY TWO** of the following are true:

- [ ] Daily trading volume consistently exceeds **$10,000 USD equivalent** for 30 consecutive days
- [ ] More than **50 unique traders** per week sustained for 4 weeks
- [ ] At least **3 active liquidity providers** per pair
- [ ] User demand: More than **10 community governance proposals** or forum requests for limit orders

### Phase 2 Triggers (Concentrated Liquidity)

Begin when **ANY TWO** of the following are true:

- [ ] Total Value Locked (TVL) exceeds **$500,000 USD equivalent**
- [ ] Capital efficiency complaints from LPs (documented via governance or support channels)
- [ ] At least **10 active trading pairs**
- [ ] Impermanent loss exceeds **5% annualized** for majority of LPs (tracked off-chain)

### Phase 3 Triggers (On-Chain CLOB)

Begin when **ALL** of the following are true:

- [ ] Daily trading volume exceeds **$1,000,000 USD equivalent**
- [ ] More than **500 unique traders** per day
- [ ] Institutional or market-maker partners onboarded (requiring deterministic order execution)
- [ ] Phase 1 off-chain matcher processing more than **1,000 orders per day**
- [ ] Latency complaints from traders (AMM slippage exceeding 2% on standard trades)

### Phase 4 Triggers (Appchain Separation)

Begin when **ALL** of the following are true:

- [ ] DEX transactions consume more than **40% of total block weight** on the main chain
- [ ] Daily trading volume exceeds **$10,000,000 USD equivalent**
- [ ] Order throughput requirements exceed **500 orders per second**
- [ ] Main chain block space contention causing non-DEX transaction delays
- [ ] Budget approval for dedicated appchain infrastructure (~$500–2,000/month additional)

---

## 3. Phase 1 — Off-Chain Limit Order Matcher

**Goal:** Activate the existing limit order storage with an off-chain bot that matches orders via AMM execution.  
**Timeline:** 4–6 weeks  
**Engineering:** 1 backend engineer + 1 DevOps  
**Chain changes:** Minimal — 1 new extrinsic, 1 storage enhancement  
**Cost impact:** ~$15/month additional (bot container on AKS)

### 3.1 Architecture

```
┌─────────────────────────────────────────────────────┐
│                   BelizeChain Node                    │
│                                                       │
│  ┌─────────────┐    ┌──────────────┐                 │
│  │  AMM Engine  │    │  OrderBook   │                 │
│  │ (unchanged)  │    │  Storage     │                 │
│  │              │    │  (existing)  │                 │
│  └──────┬───────┘    └──────┬───────┘                 │
│         │                   │                         │
│         │    ┌──────────────┴─────────┐               │
│         │    │  execute_matched_order │  ← NEW        │
│         │    │  (privileged extrinsic)│               │
│         │    └────────────────────────┘               │
└─────────┼───────────────────┼─────────────────────────┘
          │                   │
          │    ┌──────────────┴──────────────┐
          │    │    Order Matcher Bot         │
          │    │  (off-chain, AKS sidecar)   │
          │    │                              │
          │    │  1. Subscribe to OrderBook   │
          │    │     storage changes          │
          │    │  2. Check AMM price vs       │
          │    │     limit order price        │
          │    │  3. Submit execute_matched_  │
          │    │     order extrinsic if       │
          │    │     price condition met      │
          │    └─────────────────────────────┘
          │
    ┌─────┴──────────┐
    │  Maya Wallet    │
    │  (UI)           │
    │                 │
    │  • Place limit  │
    │    orders       │
    │  • View order   │
    │    book depth   │
    │  • Cancel       │
    │    orders       │
    └────────────────┘
```

### 3.2 On-Chain Changes

#### 3.2.1 New Extrinsic: `execute_matched_order`

```rust
// New call_index 11
#[pallet::call_index(11)]
#[pallet::weight(T::WeightInfo::execute_matched_order())]
pub fn execute_matched_order(
    origin: OriginFor<T>,
    order_id: u32,
) -> DispatchResult {
    // Only callable by registered matcher accounts (governance-approved)
    let matcher = ensure_signed(origin)?;
    ensure!(Self::is_authorized_matcher(&matcher), Error::<T>::Unauthorized);

    // Load order from OrderBook storage
    let order = OrderBook::<T>::get(order_id)
        .ok_or(Error::<T>::OrderNotFound)?;

    // Verify order hasn't expired
    ensure!(
        frame_system::Pallet::<T>::block_number() < order.expires_at,
        Error::<T>::OrderExpired
    );

    // Check current AMM price satisfies limit order price
    // Execute via existing AMM engine (execute_trade internally)
    // Decrement order.remaining
    // Emit OrderMatched event

    Ok(())
}
```

#### 3.2.2 Storage Additions

```rust
/// Authorized matcher accounts (governance-approved bots)
#[pallet::storage]
pub type AuthorizedMatchers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

/// Matcher registration origin (governance-only)
#[pallet::call_index(12)]
pub fn register_matcher(origin: OriginFor<T>, matcher: T::AccountId) -> DispatchResult;

/// Matcher deregistration (governance-only)
#[pallet::call_index(13)]
pub fn deregister_matcher(origin: OriginFor<T>, matcher: T::AccountId) -> DispatchResult;
```

#### 3.2.3 Order Status Enhancement

Add status tracking to `OrderEntry`:

```rust
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
}
```

Update `remaining` field to be decremented on each partial fill.

### 3.3 Off-Chain Matcher Bot

**Language:** Rust (shared toolchain with node)  
**Deployment:** AKS sidecar container alongside belizechain-node  
**Resource limits:** 50m CPU, 128Mi memory

**Bot logic:**
1. Connect to node via WebSocket RPC (`ws://belizechain-node:9944`)
2. Subscribe to `OrderBook` storage changes via `state_subscribeStorage`
3. On each new block, iterate open orders
4. For each open order:
   - Query current AMM price for the order's trading pair
   - If buy order: execute if AMM price ≤ order limit price
   - If sell order: execute if AMM price ≥ order limit price
5. Submit `execute_matched_order` extrinsic for qualifying orders
6. Handle partial fills (order.remaining > 0 → stay in book)

**Gas/fee handling:** Matcher bot pays transaction fees. Recouped via a small matcher reward (1 bps of matched trade value, funded from protocol fee).

### 3.4 UI Changes (Maya Wallet)

- **Order book view**: Display open orders aggregated by price level (bid/ask)
- **Limit order form**: Price input field, amount, expiry selector
- **Order history**: Track user's open/filled/cancelled orders
- **Real-time updates**: WebSocket subscription to OrderBook changes

### 3.5 Testing Strategy

- Unit tests for `execute_matched_order` (price validation, authorization, partial fills)
- Integration test: place order → bot detects → executes → verify fill
- Fuzz testing: random order placement + price movements
- Load test: 100 concurrent orders, verify matching latency

### 3.6 Rollback Plan

The off-chain bot is a pure addition. If issues arise:
1. Remove bot container from AKS deployment
2. Orders remain in storage but are unmatched (same as current state)
3. No chain state corruption possible — bot only submits standard extrinsics

---

## 4. Phase 2 — Concentrated Liquidity Upgrade

**Goal:** Replace constant product AMM with concentrated liquidity positions for dramatically improved capital efficiency.  
**Timeline:** 8–12 weeks  
**Engineering:** 2 backend engineers  
**Chain changes:** Major — new storage structures, modified swap logic  
**Cost impact:** Negligible (same infrastructure, more efficient capital)

### 4.1 Motivation

Constant product AMM spreads liquidity uniformly across the entire price range (0 to ∞). For a stablecoin pair like DALLA/bBZD, most trading occurs in a narrow band (e.g., 0.95–1.05). This means >95% of liquidity is idle, earning no fees.

**Concentrated liquidity** lets LPs specify a price range for their capital:

| Metric | Current AMM | Concentrated Liquidity |
|--------|-------------|----------------------|
| Capital efficiency (stablecoin pair) | ~0.5% | ~50–100% |
| Fee earnings per $1 locked | Low | 10–100x higher |
| Impermanent loss | Uniform | Higher within range, zero outside |
| Complexity | Simple | Moderate |

### 4.2 Architecture Changes

#### 4.2.1 New Storage: Position-Based Liquidity

Replace the single reserve pair per pool with tick-based positions:

```rust
/// A liquidity position within a specific price range
pub struct LiquidityPosition<AccountId> {
    pub owner: AccountId,
    pub pair: (AssetId, AssetId),
    pub tick_lower: i32,      // Lower price bound (log-space)
    pub tick_upper: i32,      // Upper price bound (log-space)
    pub liquidity: u128,      // Liquidity amount (L value)
    pub fees_owed_base: u128, // Uncollected fees in base asset
    pub fees_owed_quote: u128,// Uncollected fees in quote asset
}

/// Tick data aggregating all positions at a given price point
pub struct TickInfo {
    pub liquidity_net: i128,    // Net liquidity change when crossing this tick
    pub liquidity_gross: u128,  // Total liquidity referencing this tick
    pub fee_growth_outside_base: u128,
    pub fee_growth_outside_quote: u128,
}

/// Pool state with current tick tracking
pub struct PoolState {
    pub sqrt_price: u128,        // Current sqrt(price) in Q64.96 format
    pub current_tick: i32,       // Current active tick
    pub liquidity: u128,         // Currently active liquidity
    pub fee_growth_global_base: u128,
    pub fee_growth_global_quote: u128,
    pub protocol_fee_base: u128,
    pub protocol_fee_quote: u128,
}
```

#### 4.2.2 Tick Spacing Configuration

| Pair Type | Tick Spacing | Price Granularity | Use Case |
|-----------|-------------|-------------------|----------|
| Stablecoin (bBZD/WUSDC) | 1 | 0.01% | Tight peg pairs |
| Standard (DALLA/bBZD) | 10 | 0.10% | Normal trading |
| Volatile | 60 | 0.60% | Future exotic pairs |

#### 4.2.3 Modified Swap Logic

Swaps now cross ticks. For each swap:

1. Compute how much can be traded within the current tick range
2. If insufficient — cross to next initialized tick
3. Update fee accumulators per tick crossing
4. Continue until full swap amount consumed or price limit reached

**Integer-only arithmetic required** — no floating point in consensus. Use Q64.96 fixed-point for sqrt(price) calculations.

### 4.3 Migration Strategy

1. **Dual-mode period** (4 weeks): Both old constant-product pools and new concentrated pools active
2. LPs can migrate positions via `migrate_to_concentrated` extrinsic
3. Old pools frozen after migration window (governance vote)
4. Legacy `add_liquidity` / `remove_liquidity` remain functional for old pools during transition
5. New `mint_position` / `burn_position` / `collect_fees` for concentrated positions

### 4.4 Preserving BelizeX-Specific Features

All existing BelizeX features must carry forward:

- **Tourism fee discounts**: Applied per-swap, same logic but at position level
- **Oracle guards**: WUSDC/bBZD deviation check unchanged
- **KYC gating**: All position management + swaps still require KYC
- **Multi-hop routing**: Updated to use concentrated pool price calculation
- **Emergency pause**: Global circuit breaker covers all pool types

### 4.5 Testing Strategy

- Invariant testing: Pool solvency after any sequence of mints/burns/swaps
- Tick crossing correctness: Fee accumulation across tick boundaries
- Edge cases: Positions at min/max tick, zero-liquidity ranges, single-tick positions
- Gas/weight benchmarks: Ensure worst-case swap (crossing many ticks) fits in block
- Comparison testing: Same trade on old AMM vs concentrated pool → output must be ≥ old AMM (better capital efficiency)

---

## 5. Phase 3 — On-Chain CLOB Pallet

**Goal:** Implement a native Central Limit Order Book within the Substrate runtime for deterministic, high-speed order matching.  
**Timeline:** 12–16 weeks  
**Engineering:** 2 backend engineers + 1 researcher  
**Chain changes:** New pallet (`pallet-orderbook`) + runtime integration  
**Cost impact:** Higher block weight consumption; may require upgrading AKS node

### 5.1 Architecture

```
┌──────────────────────────────────────────────┐
│              BelizeChain Runtime               │
│                                                │
│  ┌──────────┐  ┌──────────┐  ┌─────────────┐ │
│  │ pallet-  │  │ pallet-  │  │  pallet-    │ │
│  │ belizex  │  │ orderbook│  │  economy    │ │
│  │ (AMM)    │  │ (CLOB)   │  │  (DALLA/   │ │
│  │          │  │          │  │   bBZD)     │ │
│  └────┬─────┘  └────┬─────┘  └──────┬──────┘ │
│       │              │               │        │
│       └──────┬───────┘               │        │
│              │                       │        │
│       ┌──────┴───────┐               │        │
│       │  Smart Order │               │        │
│       │  Router      │───────────────┘        │
│       │              │                        │
│       │ Routes to    │                        │
│       │ AMM or CLOB  │                        │
│       │ based on     │                        │
│       │ best price   │                        │
│       └──────────────┘                        │
└──────────────────────────────────────────────┘
```

### 5.2 CLOB Pallet Design

#### 5.2.1 Order Book Data Structure

```rust
/// Price-time priority order book using BTreeMap in storage
/// Organized as: Price Level → Vec<Order> (FIFO within price)

#[pallet::storage]
pub type Bids<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, PairKey,      // Trading pair
    Blake2_128Concat, u128,          // Price level
    BoundedVec<Order<T>, MaxOrdersPerLevel>,
>;

#[pallet::storage]
pub type Asks<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, PairKey,
    Blake2_128Concat, u128,
    BoundedVec<Order<T>, MaxOrdersPerLevel>,
>;

/// Best bid/ask cache for O(1) spread queries
#[pallet::storage]
pub type BestBid<T: Config> = StorageMap<_, Blake2_128Concat, PairKey, u128>;

#[pallet::storage]
pub type BestAsk<T: Config> = StorageMap<_, Blake2_128Concat, PairKey, u128>;
```

#### 5.2.2 Order Types

| Order Type | Description | Priority |
|-----------|-------------|----------|
| Market | Execute immediately at best available price | Highest |
| Limit | Execute at specified price or better | Price-time |
| Limit IOC | Immediate-or-Cancel: fill what you can, cancel rest | Price-time |
| Limit FOK | Fill-or-Kill: complete fill or nothing | Price-time |
| Stop-Loss | Trigger market order when price crosses threshold | Trigger-based |
| Stop-Limit | Trigger limit order when price crosses threshold | Trigger-based |

#### 5.2.3 Matching Engine

**Matching algorithm:** Price-time priority (standard CLOB)

```
For incoming BUY order at price P:
  1. Check asks where ask_price ≤ P
  2. Match against lowest ask first (best price)
  3. Within same price level, match oldest order first (FIFO)
  4. Partial fills allowed — remainder stays in book
  5. If no matching asks, order rests in bid book

For incoming SELL order at price P:
  1. Check bids where bid_price ≥ P
  2. Match against highest bid first (best price)
  3. Within same price level, FIFO
  4. Partial fills → remainder rests in ask book
```

**Weight budget:** Matching must complete within a single block's weight budget. Impose:
- `MaxMatchesPerExtrinsic`: 50 (limits worst-case execution)
- If order would require >50 matches, partial fill + rest in book
- Weight charged per match: `base_weight + (n_matches × per_match_weight)`

#### 5.2.4 Extrinsics

```rust
// Core order operations
fn place_order(pair, order_type, side, amount, price, time_in_force) -> OrderId;
fn cancel_order(order_id) -> ();
fn cancel_all_orders(pair) -> ();
fn modify_order(order_id, new_amount, new_price) -> ();

// Market maker operations
fn batch_place_orders(orders: Vec<OrderParams>) -> Vec<OrderId>;
fn batch_cancel_orders(order_ids: Vec<OrderId>) -> ();

// Query (off-chain / RPC)
fn get_orderbook_depth(pair, levels) -> OrderBookSnapshot;
fn get_order_status(order_id) -> OrderStatus;
fn get_user_open_orders(account) -> Vec<Order>;
```

### 5.3 Smart Order Router

The router sits between the user and both liquidity sources:

1. User submits trade request with amount and slippage tolerance
2. Router queries:
   - AMM: What output for this input? (constant product / concentrated)
   - CLOB: What output for this input? (walk the order book)
3. Router splits order to minimize price impact:
   - If CLOB has tight spread with depth → route to CLOB
   - If AMM has better price for size → route to AMM
   - For large orders → split across both venues
4. Execute atomically in a single extrinsic

### 5.4 Interaction with Existing BelizeX Pallet

**Option A (Recommended):** CLOB lives in a separate `pallet-orderbook`. BelizeX AMM remains unchanged. Smart Order Router is a cross-pallet call from a thin routing pallet.

**Option B:** Merge CLOB into BelizeX. More tightly coupled but simpler. Risk: bloats an already-large pallet.

**Recommendation:** Option A — separation of concerns. The AMM is proven and audited. Don't touch it.

### 5.5 Fee Model for CLOB

| Role | Fee | Rationale |
|------|-----|-----------|
| Taker (market order) | 20 bps | Consumes liquidity |
| Maker (limit order, filled) | 5 bps rebate | Provides liquidity |
| Tourism taker | 10 bps | 50% discount preserved |
| Tourism maker | 8 bps rebate | Enhanced rebate |
| Protocol (Treasury) | 5 bps of taker fee | Infrastructure funding |

Maker-taker model incentivizes limit order placement, building book depth.

---

## 6. Phase 4 — Appchain Separation

**Goal:** Migrate the order matching engine to a dedicated appchain optimized for trading throughput, while the main BelizeChain handles settlement, governance, identity, and all non-DEX pallets.  
**Timeline:** 16–24 weeks  
**Engineering:** 3 engineers + 1 DevOps + 1 infrastructure architect  
**Chain changes:** New chain binary, cross-chain messaging, settlement bridge  
**Cost impact:** Significant — dedicated infrastructure ($500–2,000/month additional)

### 6.1 Why an Appchain?

At scale, a CLOB on a general-purpose chain creates problems:

| Problem | Impact |
|---------|--------|
| Block weight contention | DEX orders crowd out governance, identity, land registry, payroll transactions |
| Latency | 6-second blocks too slow for active traders; market makers need sub-second |
| Mempool visibility | Front-running and MEV extraction become profitable at high volume |
| State bloat | Order book state grows linearly with active orders across all pairs |

An appchain solves these by giving the DEX its own block space, consensus parameters, and potentially faster block times.

### 6.2 Architecture Options

#### Option A: Substrate Solo Chain with Cross-Chain Bridge (Recommended for Phase 4)

```
┌─────────────────────┐           ┌─────────────────────┐
│   BelizeChain        │           │  BelizeX Appchain    │
│   (Main Chain)       │           │  (DEX Chain)         │
│                      │           │                      │
│  pallet-economy      │           │  pallet-orderbook    │
│  pallet-governance   │  Bridge   │  pallet-belizex-amm  │
│  pallet-identity     │◄────────►│  pallet-settlement   │
│  pallet-compliance   │  (XCM    │                      │
│  pallet-landledger   │   or     │  Block time: 1–2 sec │
│  pallet-payroll      │   custom)│  TPS: 10,000+        │
│  pallet-oracle       │           │  Dedicated resources │
│  pallet-settlement   │           │                      │
│  ... (all other      │           │  Matching engine     │
│       pallets)       │           │  runs in on_idle     │
│                      │           │  or on_finalize      │
│  Block time: 6 sec   │           │                      │
└─────────────────────┘           └─────────────────────┘
```

**Bridge responsibilities:**
- Asset transfers: Lock on main chain → Mint on appchain (and reverse)
- Settlement finality: Appchain batches trades → settles on main chain periodically
- Identity/KYC: Appchain queries main chain for compliance status
- Oracle feeds: Main chain Oracle → forwarded to appchain

#### Option B: Polkadot Parachain

If BelizeChain joins the Polkadot relay chain ecosystem, the DEX could be a separate parachain connected via XCMP (Cross-Chain Message Passing).

**Pros:** Shared security from relay chain, standardized XCM messaging  
**Cons:** Parachain slot costs, Polkadot governance dependency, slot auction complexity  
**Verdict:** Evaluate when Polkadot ecosystem maturity warrants it. Not the default path.

#### Option C: Rollup / Layer 2

The DEX operates as a rollup posting proofs to the main chain.

**Pros:** Inherits main chain security, lower infrastructure cost  
**Cons:** Proof generation overhead, complexity of rollup client  
**Verdict:** Consider if zero-knowledge rollup technology matures sufficiently.

### 6.3 Appchain Consensus Design

The appchain needs different consensus parameters than the main chain:

| Parameter | Main Chain | Appchain |
|-----------|-----------|----------|
| Block time | 6 seconds | 1–2 seconds |
| Consensus | BABE/GRANDPA | Aura/GRANDPA (simpler, faster) |
| Validator set | NPoS staking | Initially permissioned (BelizeChain-operated), then expand |
| Finality | ~12 seconds (2 blocks) | ~2–4 seconds |
| Block size | Standard Substrate | Optimized for order operations |
| MEV protection | N/A | Encrypted mempool or batch auction |

### 6.4 Settlement Flow

```
1. User deposits DALLA/bBZD to appchain via bridge
   Main Chain: Lock tokens in pallet-settlement escrow
   Appchain: Mint wrapped tokens (wDALLA / wbBZD)

2. Trading occurs entirely on appchain
   Orders matched, balances updated in appchain state
   No main chain transactions during trading

3. Periodic settlement (every N appchain blocks, e.g., every 5 minutes)
   Appchain: Compute net position changes since last settlement
   Appchain: Generate settlement proof (Merkle root of all trades)
   Bridge: Submit settlement proof to main chain
   Main Chain: Verify proof, update final balances

4. User withdraws from appchain
   Appchain: Burn wrapped tokens
   Bridge: Relay withdrawal request
   Main Chain: Unlock tokens from escrow → transfer to user
```

### 6.5 MEV Protection

At high volume, Maximal Extractable Value (MEV) becomes a real concern:

**Approach 1: Frequent Batch Auctions (Recommended)**
- Collect all orders within a 1-second window
- Execute all orders at a single clearing price
- Eliminates front-running entirely
- Used by CowSwap, Penumbra

**Approach 2: Encrypted Mempool**
- Orders encrypted until included in block
- Decrypted via threshold decryption by validators
- More complex but preserves continuous order flow

**Approach 3: FIFO within Same Block**
- Randomize transaction ordering within each block
- Reduces (but doesn't eliminate) MEV opportunities
- Simplest approach

**Recommendation:** Start with Approach 3 (FIFO randomization), upgrade to Approach 1 (batch auctions) when volume justifies the complexity.

### 6.6 Infrastructure Requirements

| Component | Specification | Monthly Cost (Est.) |
|-----------|---------------|-------------------|
| Appchain validator node(s) | 2x Standard_D4s_v3 (4 vCPU, 16GB) | $300 |
| Appchain RPC node | 1x Standard_D2s_v3 | $75 |
| Bridge relayer | Sidecar on main chain AKS | $0 (shared) |
| Monitoring (Grafana/Prom) | Shared with main cluster | $0 |
| Storage (appchain state) | 50GB Premium SSD | $10 |
| **Total** | | **~$385/month** |

**Note:** This exceeds the current $75/month budget. Governance approval required for infrastructure expansion. Revenue from trading fees should offset costs by Phase 4 trigger point.

### 6.7 Migration Plan

#### Stage 1: Parallel Operation (4 weeks)
- Launch appchain alongside main chain
- Both AMM (main chain) and CLOB (appchain) active
- Bridge operational but low limits ($1,000 per transfer)
- Monitor for bridge exploits, consensus issues

#### Stage 2: Incentivized Migration (4 weeks)
- Increase bridge limits
- Offer fee discounts on appchain (50% off for first month)
- Migrate liquidity pools to appchain (governance vote)
- Main chain AMM enters "maintenance mode" (no new pairs)

#### Stage 3: Full Migration (2 weeks)
- Main chain AMM frozen (view-only, withdrawals allowed)
- All new trading directed to appchain
- Smart Order Router updated to appchain-only
- Main chain `pallet-belizex` retained for legacy LP withdrawals

#### Stage 4: Cleanup (2 weeks)
- Final LP withdrawal deadline (governance announcement, 30 days notice)
- Main chain `pallet-belizex` deprecated (storage migration to archive)
- Bridge becomes the sole DEX entry point
- Documentation and SDK updated

---

## 7. Infrastructure & Cost Projections

### 7.1 Cost Timeline

| Phase | Monthly Infra Cost | Revenue Offset | Net Cost |
|-------|-------------------|----------------|----------|
| Current (AMM only) | $75 | $0 | $75 |
| Phase 1 (Off-chain matcher) | $90 | $50–500* | $0–90 |
| Phase 2 (Concentrated liquidity) | $90 | $200–2,000* | $0 |
| Phase 3 (On-chain CLOB) | $105 | $1,000–10,000* | $0 |
| Phase 4 (Appchain) | $460 | $5,000–50,000* | $0 |

*Revenue estimates assume 50% protocol fee capture of total DEX fees at each volume level.

### 7.2 Engineering Cost

| Phase | Duration | Team Size | Total Person-Months |
|-------|----------|-----------|-------------------|
| Phase 1 | 4–6 weeks | 2 | 2.5 |
| Phase 2 | 8–12 weeks | 2 | 5 |
| Phase 3 | 12–16 weeks | 3 | 10.5 |
| Phase 4 | 16–24 weeks | 5 | 25 |
| **Total** | | | **43 person-months** |

---

## 8. Risk Assessment

### 8.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Bridge exploit (Phase 4) | Medium | Critical | Formal verification of bridge logic; start with low transfer limits; time-delayed large withdrawals |
| CLOB weight overflow (Phase 3) | Medium | High | MaxMatchesPerExtrinsic cap; weight benchmarks before mainnet |
| Concentrated liquidity rounding errors | Low | High | Integer-only arithmetic; extensive invariant fuzzing with proptest |
| Appchain consensus failure | Low | Critical | Start permissioned (3 trusted validators); expand gradually |
| MEV extraction on appchain | High | Medium | Implement batch auctions by Phase 4 launch; monitor mempool |
| Migration data loss | Low | High | Parallel operation period; no force-migration; generous withdrawal windows |

### 8.2 Business Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Volume never reaches Phase 3 triggers | Medium | Low | No wasted effort — each phase is independently valuable |
| Market makers don't onboard | Medium | Medium | Phase 1 + 2 work perfectly without MMs; CLOB only needed with MMs |
| Regulatory change in Belize | Low | High | All code open-source; architecture adaptable to compliance mandates |
| Competing DEX launches on BelizeChain | Low | Low | First-mover advantage; governance controls pair listing |
| Engineering talent unavailable | Medium | Medium | All phases use Rust/Substrate — same stack as core team |

### 8.3 Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| AKS budget exceeded during transition | Medium | Medium | Each phase has explicit cost ceiling; governance pre-approval |
| Phase overlap causes instability | Low | Medium | Strict trigger criteria prevent premature phase starts |
| Community confusion during migration | Medium | Low | Clear documentation; 30-day notice periods; UI guidance |

---

## 9. Competitive Landscape

Understanding what other sovereign/national blockchain projects and DEX architectures have done:

| Project | DEX Model | Appchain? | Lessons for BelizeX |
|---------|----------|-----------|-------------------|
| **dYdX v4** | CLOB | Yes (Cosmos appchain) | Moved from Ethereum L2 → own chain for latency. Volume was $1B+/day. Don't move before you need to. |
| **Sei Network** | Built-in CLOB | Native (Cosmos) | Purpose-built for trading. 390ms finality. Overkill for early-stage. |
| **Injective** | CLOB | Yes (Cosmos) | MEV-resistant via Frequent Batch Auctions. Good model for Phase 4. |
| **Uniswap v3** | Concentrated AMM | No (Ethereum L1) | Concentrated liquidity alone provided 4,000x capital efficiency. Phase 2 may be enough. |
| **Osmosis** | AMM + orderbook | Appchain (Cosmos) | Started AMM, added concentrated liquidity, orderbook came much later. Same path we propose. |
| **Astroport** | AMM | No (Terra/Sei) | Multiple pool types (constant product, stableswap, concentrated). Good reference for Phase 2. |

**Key takeaway:** Every successful DEX that migrated to an appchain did so AFTER proving product-market fit with a simpler model. None started with an appchain orderbook.

---

## 10. Success Metrics

### Phase 1

| Metric | Target | Measurement |
|--------|--------|-------------|
| Limit order fill rate | >60% of placed orders within 24h | On-chain event tracking |
| Matcher bot uptime | >99.5% | AKS health checks |
| Fill latency | <2 blocks (12 seconds) from price crossing | Event timestamp analysis |
| User adoption | >20% of trades via limit orders within 3 months | Extrinsic ratio tracking |

### Phase 2

| Metric | Target | Measurement |
|--------|--------|-------------|
| Capital efficiency improvement | >10x vs constant product for stablecoin pairs | TVL per unit of volume |
| LP fee earnings increase | >5x per $1 locked | Fee accumulator tracking |
| Migration rate | >80% of TVL in concentrated positions within 60 days | Storage queries |
| Zero rounding-error incidents | 0 balance discrepancies | Invariant monitoring |

### Phase 3

| Metric | Target | Measurement |
|--------|--------|-------------|
| Order matching latency | <1 block (6 seconds) | Event analysis |
| Spread (DALLA/bBZD) | <10 bps during active hours | Best bid/ask monitoring |
| Market maker onboarding | ≥2 active MMs per major pair | Order placement patterns |
| CLOB vs AMM volume split | CLOB handles >50% of volume for pairs with depth | Router analytics |

### Phase 4

| Metric | Target | Measurement |
|--------|--------|-------------|
| Appchain TPS | >5,000 sustained | Block explorer |
| Settlement finality | <30 seconds end-to-end | Cross-chain event tracking |
| Bridge security | 0 exploits | Continuous monitoring + audits |
| Main chain block weight freed | >30% reduction in DEX-related weight | Weight analytics |
| Trading fees covering infra cost | Revenue ≥ 2x infrastructure cost | Monthly P&L |

---

## Appendix A — Current BelizeX Feature Matrix

| Feature | Status | Phase Impact |
|---------|--------|-------------|
| AMM Trading (constant product) | Live | Unchanged through Phase 1; upgraded Phase 2; coexists Phase 3–4 |
| Liquidity Provision (LP tokens) | Live (X-1 fixed) | Upgraded to positions in Phase 2 |
| Multi-Hop Routes (5 legs max) | Live | Updated for concentrated pools in Phase 2 |
| Tourism Fee Discounts | Live | Carried forward all phases |
| Volume Tier Discounts (Oracle) | Live | Carried forward; extended to CLOB in Phase 3 |
| Limit Orders (storage only) | Live (no matching) | Activated in Phase 1 |
| Oracle Integration (WUSDC guard) | Live | Extended to CLOB price feeds in Phase 3 |
| KYC Gating | Live | Required on appchain via bridge verification in Phase 4 |
| Sanctions Checking (CW-1) | Live | Cross-chain check in Phase 4 |
| Emergency Pause (circuit breaker) | Live | Extended to appchain in Phase 4 |
| Order Cancellation (H-24 fixed) | Live | Enhanced with batch cancel in Phase 3 |

---

## Appendix B — Appchain Architecture Reference

### B.1 Recommended Substrate Pallets for Appchain

```
# Core
frame-system
frame-support
pallet-timestamp
pallet-aura          # Fast block production (1-2 sec)
pallet-grandpa       # Finality

# DEX-Specific
pallet-orderbook     # Custom CLOB (Phase 3 code)
pallet-belizex-amm   # Migrated AMM pools
pallet-settlement    # Bridge settlement logic

# Utility
pallet-transaction-payment
pallet-balances      # Wrapped asset balances
```

### B.2 Bridge Message Format

```rust
enum BridgeMessage {
    /// Lock assets on main chain, mint on appchain
    Deposit {
        sender: AccountId,
        asset: AssetId,
        amount: u128,
        nonce: u64,
    },
    /// Burn on appchain, unlock on main chain
    Withdrawal {
        recipient: AccountId,
        asset: AssetId,
        amount: u128,
        nonce: u64,
        proof: MerkleProof,
    },
    /// Periodic settlement batch
    Settlement {
        batch_id: u64,
        merkle_root: H256,
        net_transfers: Vec<(AccountId, AssetId, i128)>,
        block_range: (u64, u64),
    },
    /// Identity/KYC status sync
    ComplianceSync {
        account: AccountId,
        kyc_level: u8,
        is_sanctioned: bool,
    },
}
```

### B.3 Appchain Genesis Configuration

```json
{
  "name": "BelizeX Exchange Chain",
  "id": "belizex-appchain",
  "chainType": "Live",
  "protocolId": "blzx",
  "properties": {
    "tokenSymbol": "wDALLA",
    "tokenDecimals": 18,
    "blockTime": 2000
  }
}
```

---

## Appendix C — Migration Checklist

### Pre-Phase 1

- [ ] Finalize BelizeX AMM audit remediation (COMPLETE)
- [ ] Launch testnet with current AMM
- [ ] Establish baseline trading volume metrics
- [ ] Set up DEX analytics dashboard (Grafana)
- [ ] Document current API surface for SDK consumers

### Pre-Phase 2

- [ ] Phase 1 trigger criteria met (documented evidence)
- [ ] Off-chain matcher running stable for 30+ days
- [ ] Research concentrated liquidity implementations (Uniswap v3, Algebra)
- [ ] Audit budget allocated for concentrated liquidity code
- [ ] UI/UX designs for position management interface

### Pre-Phase 3

- [ ] Phase 2 trigger criteria met (documented evidence)
- [ ] Concentrated liquidity running stable for 60+ days
- [ ] Market maker partnership(s) signed (LOI minimum)
- [ ] CLOB pallet prototype benchmarked for weight budget
- [ ] Security audit firm engaged for CLOB review
- [ ] Governance proposal for CLOB activation approved

### Pre-Phase 4

- [ ] Phase 3 trigger criteria met (documented evidence)
- [ ] On-chain CLOB running stable for 90+ days
- [ ] Bridge protocol formally verified (or extensively audited)
- [ ] Appchain testnet running 30+ days with load testing
- [ ] Governance vote: approve appchain infrastructure budget expansion
- [ ] Emergency procedures documented for bridge halt
- [ ] User communication plan for migration timeline
- [ ] SDK updated for dual-chain support
- [ ] Insurance/reserve fund established for bridge risk

---

*This plan is a living document. Review quarterly or when any trigger criteria threshold is approached. All phase transitions require governance approval.*

**Document History:**
| Date | Version | Author | Change |
|------|---------|--------|--------|
| 2026-03-24 | 1.0 | BelizeChain Core Team | Initial executive plan |
