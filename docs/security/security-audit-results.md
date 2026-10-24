# Security Audit Results

**Internal Audits • Automated Scanning • Vulnerability Disclosure**

---

## Status: No External Professional Audit Completed

**A professional third-party audit has NOT been completed.**
A qualified external audit firm must be engaged before mainnet launch.

---

## Internal Comprehensive Audit (2026)

### Methodology
14-phase automated and manual internal audit using `cargo audit`, `cargo clippy`,
`cargo deny`, custom analysis scripts, and manual code review.

### Summary

| Priority | Count | Description |
|----------|-------|-------------|
| **P0 — Blocker** | 22 | Must fix before mainnet |
| **P1 — Critical** | 30 themes | High-priority remediation |
| **P2 — Important** | ~200 | Should fix before mainnet |
| **P3 — Minor** | ~445 | Recommended improvements |

**Overall Verdict**: NOT READY FOR MAINNET — Conditional NO-GO

**Full Report**: See `audit_results/old_audits/COMPREHENSIVE_AUDIT_2026.md`
(the path previously given here, `audit_results/COMPREHENSIVE_AUDIT_2026.md`, does
not exist — the report was archived under `old_audits/` and the link was never
updated).

### Remediation Progress
- Consensus audit: 38/38 findings fixed
- Sprint 1 P0 fixes: **closed as of 2026-10-23**, with one correction — the audit's
  P0-05 verdict ("FALSE FINDING — `distribute_rewards()` does not exist") was wrong;
  the function existed since the initial commit. A real churn loop in it was found
  and fixed on 2026-10-23 (`3151665`). See the note below.
- **Sprint 2 P1 items: 20 of 30 closed (2026-10-23).** The remaining ten are
  either obsolete (AKS), false positives, or require a design decision. See the
  triage table below.

### Corrections to the archived report (2026-10-23)

- **P0-05 was a real finding, not a false one.** `distribute_rewards` has existed
  since the initial commit (`6eafac8`). It was permissionless, ungated, and cleared
  the `ModelSubmissions` guard, so `submit_model_delta` -> `distribute_rewards`
  could be repeated inside one block, minting a reward round per iteration.
  Fixed by enforcing the pre-existing but unread `EpochDuration`.
- **P0-12 is genuinely closed** — `RealPqVerifier` / `MLDsaVerifier`
  (`fips204::ml_dsa_87`) are wired in all non-benchmark runtime builds.
- P0-19 and P0-21 verified closed (benchmark-generated weights; 40% coverage gate
  over all pallet packages).
- **P1-07 closed (2026-10-23).** `submit_mesh_transaction` only required
  `signature_hash != H256::zero()`, so any non-zero value was accepted and a gateway
  could bridge an arbitrary transfer attributed to any compact sender. The field was
  a 32-byte `H256` documented as an "Ed25519 or post-quantum compressed signature",
  which cannot hold either. It is now `signature: [u8; 64]`, verified with sr25519
  against a key the sender's gateway registered via new call index 16
  `register_mesh_sender_key`. New storage `MeshSenderKeys`; mesh `STORAGE_VERSION`
  0 -> 1.
- **P1-03 was already fixed and the checklist never said so.** nginx applies
  `limit_req zone=rpc burst=20 nodelay` to `/ws` and `/rpc`, and
  `limit_req zone=api burst=30 nodelay` to every `/api/*` path. The directives live
  in `infra/nginx/snippets/stack-paths.conf`, not `nginx.conf`, which is why a grep
  of the main file reports none. The `node/src/service.rs` header that lists RPC
  rate limiting as a roadmap item is about in-node middleware and is accurate as
  written — it is a "should be implemented" list, not a claim.

### P1 triage (2026-10-23)

Each row was checked against the source, not against the checklist.

| Status | Items |
|---|---|
| **Already fixed, never ticked** | P1-03 (nginx `limit_req` on `/ws`, `/rpc`, `/api/*` — in `infra/nginx/snippets/stack-paths.conf`, not `nginx.conf`), P1-09 (`transfer_property` requires `government_verified` + `surveyed` + KYC 2 + no active encumbrance), P1-10 (median aggregation with `MaxPriceDeviation` and variance), P1-12 (zero hardcoded numeric weights), P1-13, P1-14, P1-16, P1-25, P1-27, P1-28 (`tests/cross_pallet`, `e2e`, `oracle`, `governance`, `belizex`, `economic`), P1-26 (162 governance tests) |
| **Fixed 2026-10-23** | P1-05, P1-07, **P1-15** (approved proposal no longer forfeits its deposit when the treasury cannot pay), **P1-30** (removed the unsupported "544% APY" and "3-6% APY inflation" claims; derived ceiling is ~16% APY at full PoUW weight and 1,000 DALLA stake, and emissions are ~0.003% of `MaxDallaSupply` per year) |
| **Obsolete — AKS removed** | P1-04, P1-20, P1-21, P1-22, P1-23 |
| **False positive** | P1-11 (`LongestChain` is the standard BABE+GRANDPA pattern, CONS-035), P1-19 (only `dtolnay/rust-toolchain@1.90.0`, a version tag) |
| **False positive, and its stated reason has since expired** | P1-17 — it was dismissed on the grounds that "no order matching exists, so a maker/taker self-trade cannot occur". That was true when it was written and is not true now: P1-08 added a matching engine. Self-trade is still not an exploit, but for a different reason — `match_order` **skips** an account's own orders instead of filling them, so a self-crossing order simply rests. `SelfTradeNotAllowed` therefore remains unraised, now deliberately rather than incidentally; the variant is annotated as such in `pallets/belizex/src/lib.rs`. Re-read that annotation before assuming the finding is still moot |
| **Fixed 2026-10-23 (second pass)** | P1-06 (`verify_merchant` now requires `T::MerchantVerificationOrigin` = `TechnicalCouncilMajority`; it was a single `OracleOperators` member), P1-08 **settlement half** (the DEX moved no bBZD at all — see below) |
| **Fixed 2026-10-24** | P1-08 **matching half** — limit orders now escrow at placement and fill against each other in price-time order (see below) |
| **Fixed 2026-10-24 (CI fallout)** | The five benchmark smoke-test failures that P1-06/P1-07/conviction-bonding left behind — `governance::cast_vote` (bond underfunded), `oracle::verify_merchant` (origin moved off `Signed`), `mesh::submit_mesh_transaction` (unsigned after P1-07), `belizex::{add_liquidity, execute_trade, remove_liquidity}` (quote leg unfunded). See *The benchmark smoke test* below |
| **Open — needs a decision** | P1-02 (`Sudo` at index 6, deliberate for testnet bootstrap; owner has said do not remove) · **P1-24 is now observed, not hypothetical** — a four-node swarm has been running a `chainType: Live` spec with `--force-authoring` on every node since 2026-09-13; it is forked and stalled 38 days. See below |
| **Not started** | External audit, penetration testing, economic modelling, formal verification |

#### P1-08 is worse than reported, and is a stub rather than a bug

The finding was "DEX limit orders don't reserve funds → unfunded order
exploitation". Orders are in fact never matched: `OrderBook` is written by
`place_limit_order` and read only by `cancel_order`, and `execute_trade` swaps
against the AMM pool without consulting it. An order therefore cannot fill, so
the stated exploitation risk does not exist — but the extrinsic advertises a
feature that is not implemented, which is a worse failure mode for a user.

A matching engine (price-time priority, partial fills, settlement) was a new
subsystem with economic consequences, so it was not invented on the first pass.
It has since been built — see *The matching engine* below — and limit orders are
now live rather than advertised-but-absent.

##### Worse still: the DEX moved no bBZD

The finding understated the problem. `pallet-belize-belizex` had **no dependency
on `pallet-belize-economy`**, and `AssetId` was a label carried on the order and
pair structs, never consulted for a transfer. Every settlement site moved the
caller's native DALLA balance regardless of which asset the pair named, so a
"DALLA/bBZD" trade debited and credited DALLA on both legs. The pair was
decorative.

Fixed with an `AssetLedger` seam:

```rust
pub trait AssetLedger<AccountId> {
    fn balance_of(asset: AssetId, who: &AccountId) -> u128;
    fn transfer(asset: AssetId, from: &AccountId, to: &AccountId, amount: u128)
        -> DispatchResult;
}
```

Six settlement sites converted. The runtime adapter (`BelizeXAssetLedger`) routes
DALLA and TourismDALLA to `Balances`, bBZD to `Economy::bbzd_balances` with the
same sanction re-check `pallet-economy` applies, and returns `UnsupportedAsset`
for WUSDC rather than silently moving the wrong token. The mock implements the
trait with a `thread_local!` transfer log so the tests can assert *which* asset
moved, not just that a balance changed.

##### The pair index (matching prerequisite)

`OrderBook` is a flat `u32 -> OrderEntry` map, so "what is open on DALLA/bBZD?"
had no bounded answer — a matching pass would have to iterate every order on
every pair, which cannot be given a static weight. `PairOrders` now holds per-pair
`bids` and `asks` as `BoundedVec<u32, T::MaxOrdersPerPair>`, maintained by
`place_limit_order` and `cancel_order`, and read by `open_orders(pair)` which
returns best-price-first on each side and skips IDs no longer in `OrderBook`.

This was deliberately only the prerequisite. The engine itself still needs:

- **fund reservation.** Orders reserve nothing today, so a fill would be
  attempted against a balance that may be gone. Reserving on placement changes
  `place_limit_order` into a value-transferring extrinsic.
- **a bounded iteration with a per-item weight.** A book walk reads a
  data-dependent number of entries; that needs `T::MaxOrdersPerPair` as an
  explicit loop bound and a weight that scales with it, not a flat figure.
- **self-trade handling.** `SelfTradeNotAllowed` exists and has never been
  raised; it should be wired when matching lands rather than left dead.
- **price-time priority.** `open_orders` currently sorts by price only, with
  order id ascending as the tiebreak — a proxy for arrival, not a timestamped
  queue. That is adequate for a first engine but must be stated, not assumed.

#### The matching engine (fixed 2026-10-24)

All four prerequisites are now closed.

**Escrow at placement.** `OrderEntry` gained an `escrow: u128` field and the
pallet a dedicated `OrderEscrowPalletId` account. `place_limit_order` moves funds
*before* inserting the order:

- a **bid** escrows the quote asset, `ceil(amount * price / PRICE_SCALE)`, so a
  rounding error can never leave the book promising more than it holds;
- an **ask** escrows the base asset, `amount` — it already owns what it is selling.

`cancel_order` refunds whatever remains in `escrow`, so a partially filled order
returns only its remainder. A placement that cannot fund the escrow fails before
touching the book, which is what removes the original unfunded-order finding
rather than merely hiding it.

**Price-time matching.** `place_limit_order` runs `match_order` against the
opposite side after inserting, walking `PairOrders` best-price-first and bounded
by `T::MaxOrdersPerPair`. A crossing pair settles both legs through
`AssetLedger`, so the asset actually named by the pair moves. Fill price is the
**resting (maker) order's** price, not the taker's — the standard
price-improvement rule; the taker keeps the difference. Partial fills decrement
`remaining` on both sides and leave the larger order resting.

**Self-trade** is skipped rather than rejected: an account that crosses its own
order leaves it on the book and continues to the next maker. Rejecting the whole
placement would let a maker grief the book with a self-crossing order. The
`SelfTradeNotAllowed` variant is therefore still unraised — it is now dead by
design rather than by omission, and is annotated as such in the source.

**Expiry** is purged lazily during matching — an expired resting order is removed
and refunded instead of filling, so the book cannot hold a claim the owner has
walked away from.

**Settlement atomicity.** The first cut decremented both orders' `remaining` and
`escrow`, wrote them, and only then moved funds. That ordering is wrong: a
transfer can still be rejected after `plan_fill` has proved the *bookkeeping*
covers it, because the ledger adapter re-checks sanctions on bBZD and a party can
be sanctioned between placement and fill. On a rejected leg the two orders would
have claimed a fill that never happened, with the funds left stranded in the
escrow account and no owner.

Split into three steps so the book cannot get ahead of the ledger:

| Step | Effect |
|---|---|
| `plan_fill` | computes the amounts, mutates nothing, returns `None` if either side is short |
| `settle_fill` | moves both legs, **reverses the first if the second is rejected**, returns `bool` |
| `apply_fill` | decrements `remaining`/`escrow` — only after settlement succeeded |

A rejected fill now leaves both orders resting with their escrow intact and emits
no `OrderFilled`. Matching stops rather than skipping to the next maker, because
the rejection is a property of that maker's counterparty, not of the price.

`a_rejected_second_leg_puts_the_first_leg_back_and_leaves_the_book_untouched`
covers it: the mock ledger can now reject incoming transfers to a chosen account,
which drives the case where the base leg has already moved. Against the old code
that test fails on the first assertion after the fill, because the fully-filled
bid had already been removed from the book.

**Rounding.** A bid escrows `ceil(amount × price / PRICE_SCALE)` for its whole
size, but a fill charges one slice at a time. Summing per-slice ceilings can
exceed that single ceiling — ten 1-unit fills at price 1.5 need 20 quote against
an escrow of 15 — after which the escrow check fails on every later walk and the
order can never fill again. It stays cancellable, so this is a liveness bug rather
than a loss of funds, but the order is silently un-matchable.

Fills now round **down**: `sum(floor(aᵢ·p/S)) ≤ floor(A·p/S) ≤ ceil(A·p/S)`, so
the escrow always covers the fill. The escrow at placement still rounds up, so an
order can always pay its own limit price. Rounding down strands at most one base
unit of dust per fill, refunded when the order leaves the book.

`a_bid_at_a_fractional_price_can_still_fill_in_pieces` covers it: ten asks of 1
base fill a single 10-base bid priced at 1.5. Against round-up the bid strands
after seven fills and the test fails on `the bid must be fully consumed`.

**Weight.** `WeightInfo` gained `match_orders(orders)`, and `place_limit_order`'s
weight is now `place_order() + match_orders(T::MaxOrdersPerPair::get())`.

An iteration is **either** a fill **or** the purge of an expired order, and a walk
can be entirely one or the other, so the per-iteration charge is
`cancel_order() + execute_trade()` — a purge is `remove_order` plus a refund
(what `cancel_order` measures) and a fill writes one order and moves two ledger
legs (what `execute_trade` measures). Charging both is a strict upper bound on
either. It is written in terms of those two functions rather than as literals, so
regenerating either cannot silently leave this one under-charging.

That bound is **derived, not measured**, and is marked as such in an AMENDMENT
block. The benchmark of the same name seeds live orders, so it measures the fill
shape only — measuring it alone would under-charge a walk of expired orders. The
figure must be regenerated with `scripts/bench_weights.sh` before mainnet, and
the derivation above is what that regeneration has to preserve.

**Migration.** `OrderEntry`'s encoding changed, so `STORAGE_VERSION` went 0 → 1
and `pallets/belizex/src/migrations.rs` purges `OrderBook`, `PairOrders` and
`AccountOrderCount` rather than reinterpreting old bytes. `runtime/src/migrations.rs`
runs it at `CURRENT_RUNTIME_VERSION` 3.

**Weight caveat:** a `match_orders(n: Linear<1, MaxOrdersPerPair>)` benchmark now exists and
passes in the mock, but `weights.rs` still holds the derived placeholder until
`scripts/bench_weights.sh` regenerates it on AC power. There is also no time-priority
*timestamp* — the tiebreak is still order id, which is a proxy for arrival.

#### The benchmark smoke test was red, and it was not noise

`scripts/bench_smoke.sh` runs the **real release node** against `--chain dev`, not the
pallet mocks. That distinction is the whole story: every one of the five failures was a
benchmark that passes under `cargo test --features runtime-benchmarks` and fails in the
node, because a mock config collapses differences the runtime keeps.

| Benchmark | Cause |
|---|---|
| `governance::cast_vote` | conviction bonding reserves `ConvictionBondBase × multiplier`. The mock sets the base to `MinimumDeposit`; the runtime sets it to **10× that**. The voter was funded for the mock. |
| `oracle::verify_merchant` | P1-06 moved the gate to `T::MerchantVerificationOrigin` (`TechnicalCouncilMajority`); the benchmark still used `Signed(operator)`. |
| `mesh::submit_mesh_transaction` | P1-07 added sender-key registration and sr25519 verification; the benchmark sent `[0xBB; 64]` and never reached the measured code. |
| `belizex::{add_liquidity, execute_trade, remove_liquidity}` | Pair `(0,1)` is DALLA/bBZD, and those live in **different ledgers**. Only the native balance was funded. `MockAssetLedger` maps every asset to `Balances`, so the mock could not see it. |

The belizex three needed a production seam, not just a benchmark edit: `AssetLedger` gained
`set_balance(asset, who, amount)`, gated on `#[cfg(feature = "runtime-benchmarks")]`, so a
benchmark can fund bBZD without the pallet taking a dependency on `pallet-belize-economy`
(the reason the adapter lives in the runtime at all).

Three traps worth recording:

- **`sp-core/full_crypto` cannot be enabled from a pallet's `runtime-benchmarks` feature.**
  It propagates into the runtime's WASM build and fails inside `app_crypto_pair_common`.
- **`sp_io::crypto::{sr25519_generate, sr25519_sign}` need a keystore.** The node running
  benchmarks has one; the `impl_benchmark_test_suite!` mock does not and panics. Sign for
  real under `cfg(not(test))`, use a dummy under `cfg(test)` where the mock verifier is
  `AcceptAll`.
- **`KeyTypeId` is `[u8; 4]`.** A five-byte literal is a type error.

#### P1-24 — `--force-authoring` is live on a forked chain, not a hypothetical

`infra/docker-compose.validators.yml` runs a four-node swarm, and **all four**
services pass `--force-authoring`. The flag is correct on a single-authority
testnet — it is what lets one node produce blocks with no peers — and dangerous
on a multi-validator network, where a node that has not finished syncing will
still author and fork the chain.

**An earlier revision of this section said the swarm "has never been started
against mainnet, which is the only reason it has not caused a fork". That was
wrong.** It was written from the compose file alone, without checking the host.
Measured 2026-10-24:

| Observation | Value |
|---|---|
| Containers | `val-ceiba`, `val-maya`, `val-reef`, `val-cayo` — all running, all healthy |
| Chain spec | `generated_specs/chainspec-mainnet-raw.json` (gitignored, dated 2026-08-27), `chainType: "Live"`, `id: belizechain` |
| Genesis | `0x8b66304f267a91ed4af49cd5ccf2b32900de780f2355402d3f217b2057c9c59c` — **not** Jade's `0xb2664568…`, so this is a separate chain |
| Last block | `2026-09-15T13:34:42Z` — stalled **38 days** |
| Height | `#13753`, finalized `#13751` (finality 2 blocks behind) |
| Best hash | `0x1b98…43e4` on ceiba/cayo vs `0xbbec…488d` on maya/reef — **two distinct tips at the same height** |
| Logs | date from 2026-09-13; containers restarted 2026-10-18 but the chain data is older |
| BABE | `No block for 554390 slots. Applying exponential lenience, total proposing duration: 516000ms` — retrying forever |
| Exposure | every service publishes RPC/P2P/Prometheus on `0.0.0.0`, not loopback |

A fork at equal height with four authorities is the P1-24 signature, and the flag
is present on all four. **Causation is not proven** — the logs carry no
equivocation or import-failure line, so the divergence mechanism was not captured
before it happened. What is proven is that the failure mode this finding
describes is not theoretical: a `chainType: Live` spec has been running for weeks
with the flag on every node, and the chain is dead and forked.

It is also on a workstation, not a server, holding ports open on all interfaces
and consuming ~250 MiB across the four containers.

Not fixed by deleting the flag from the file: the running containers would keep
their existing `command:`, so a file edit alone changes nothing on the host. The
decisions are (a) stop the swarm and remove the volumes, (b) rebuild the compose
file to gate the flag the way the live Ceiba stack already does —
`${VALIDATOR:+--force-authoring}` — and (c) never launch a multi-validator
network from a spec that does not first prove finality across all authorities.

As of 2026-10-24 the warning is stated once in the file header and pointed at
from each of the four services, instead of sitting above `validator-ceiba` only
while the other three carried the flag unannotated. Verified by parsing the file:
all four commands contain the flag, and no `#` leaked into a `command: >` block
scalar (YAML does not strip comments inside a block scalar — a comment placed
there is passed to the node as arguments and the container fails to start).

**Lesson.** The original text asserted host state from a configuration file. A
compose file says what *would* run, not what *is* running; the two had diverged
by six weeks. Check the host before writing a sentence about it.

### Findings that are obsolete, not fixed

The P1 list was written against the AKS deployment that has since been removed.
These can never be "fixed" in code and should be closed as no-longer-applicable:
P1-04 (RPC behind a LoadBalancer), P1-20 (ACR admin credentials), P1-21 (K8s
securityContext), P1-22 (emptyDir chain data), P1-23 (AKS NetworkPolicy).
The live stack is Docker Compose on Ceiba with nginx and Tailscale-bound admin
ports (`infra/docker-compose.ceiba.yml`).

P1-11 (`LongestChain`) is a documented false positive — it is the standard
Substrate pattern for BABE + GRANDPA, and `node/src/service.rs` records that as
CONS-035.

---

## Automated Scanning

### Cargo Audit (Dependency CVEs)
```bash
cargo audit
```
Results tracked in `audit_results/cargo_audit_*.txt`

### Cargo Clippy (Lint Warnings)
Results tracked in `audit_results/cargo_clippy_*.txt`

### Cargo Deny (License & Advisory)
Configuration: `deny.toml`

---

## Vulnerability Disclosure Policy

**We welcome security researchers to report vulnerabilities.**

1. Email: security@belizechain.org (PGP key available)
2. Do NOT publicly disclose until patched
3. Provide detailed reproduction steps
4. Allow 90 days for fix before public disclosure

See [Bug Bounty Program](BUG_BOUNTY_PROGRAM.md) for reward details.

---

## Planned External Audit

A professional external audit is budgeted and planned. Requirements:
- Firm with Substrate/Polkadot ecosystem expertise
- Scope: Full runtime, all 19 custom pallets (runtime indexes 20-38), node infrastructure
- See [Audit Framework](AUDIT_FRAMEWORK.md) for evaluation criteria and planning

---

## Related Documentation

- [Audit Framework](AUDIT_FRAMEWORK.md) - Planning for professional external audit
- [Bug Bounty Program](BUG_BOUNTY_PROGRAM.md) - Vulnerability reward policy (planned)
- [Security Overview](SECURITY_OVERVIEW.md) - Overall security architecture
- [Compliance Pallet API](../developer-guides/pallet-apis-core.md#compliance-pallet)
