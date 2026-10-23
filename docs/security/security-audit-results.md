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
| **False positive** | P1-11 (`LongestChain` is the standard BABE+GRANDPA pattern, CONS-035), P1-17 (no order matching exists, so a maker/taker self-trade cannot occur; `SelfTradeNotAllowed` is a dead variant reserved for a future engine), P1-19 (only `dtolnay/rust-toolchain@1.90.0`, a version tag) |
| **Fixed 2026-10-23 (second pass)** | P1-06 (`verify_merchant` now requires `T::MerchantVerificationOrigin` = `TechnicalCouncilMajority`; it was a single `OracleOperators` member), P1-08 **settlement half** (the DEX moved no bBZD at all — see below) |
| **Open — needs a decision** | P1-02 (`Sudo` at index 6, deliberate for testnet bootstrap; owner has said do not remove) · P1-08 **matching half** (index built, engine deliberately not; see below) · P1-24 (`--force-authoring` is correct on a single-authority testnet, unsafe on multi-validator mainnet; annotated in `infra/docker-compose.validators.yml`) |
| **Not started** | External audit, penetration testing, economic modelling, formal verification |

#### P1-08 is worse than reported, and is a stub rather than a bug

The finding was "DEX limit orders don't reserve funds → unfunded order
exploitation". Orders are in fact never matched: `OrderBook` is written by
`place_limit_order` and read only by `cancel_order`, and `execute_trade` swaps
against the AMM pool without consulting it. An order therefore cannot fill, so
the stated exploitation risk does not exist — but the extrinsic advertises a
feature that is not implemented, which is a worse failure mode for a user.

A matching engine (price-time priority, partial fills, settlement) is a new
subsystem with economic consequences and has deliberately not been invented here.
The extrinsic's doc comment now says so. Until an engine exists, limit orders
should be treated as unavailable.

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

This is deliberately only the prerequisite. The engine itself still needs:

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
