# BelizeChain Phase Status

> **Last Updated**: 2026-10-25
>
> **This file is a structural summary, not live state.** Volatile values —
> `spec_version`, head height, validator count, endpoints, deployed contract
> addresses — are deliberately **not** restated here, because a second copy
> drifts. Read [`ECOSYSTEM_STATE.md`](ECOSYSTEM_STATE.md) §2 for the current
> values and the date each was verified.
>
> **SDK**: Polkadot SDK `stable2609` (rev `7a8daa68e18718c18823c1109513fdc1c8aa71fe`)

---

## Custom Pallet Roster (20 pallets, runtime indexes 20–39)

All 20 custom pallets are compiled into the runtime. The source of truth is
`runtime/src/lib.rs` `construct_runtime!` plus the `Cargo.toml` workspace
members — not this table.

The "Phase" column is development history. "Verified" there means the pallet was
exercised on a testnet at that phase; it is **not** a statement about current
live behaviour. The "Live Testnet Extrinsics" section at the end of this file is
a dated record from 2026-09-03.

| Index | Pallet | Category | Status / Phase |
|---|---|---|---|
| 20 | Economy | Economy | Phase 1 (Verified) |
| 21 | Identity | Identity | Phase 1 (Verified) |
| 22 | Governance | Governance | Phase 1 (Verified) |
| 23 | Compliance | Identity | Phase 1 (Verified) |
| 24 | Staking | Economy | Phase 1 (Verified) |
| 25 | Oracle | Infrastructure | Phase 1 (Verified) |
| 26 | Payroll | Economy | Phase 2 (Verified) |
| 27 | Interoperability | Infrastructure | Phase 2 (Verified) |
| 28 | BelizeX (DEX) | Economy | Phase 2 (Verified) |
| 29 | LandLedger | Governance | Phase 2 (Verified) |
| 30 | Consensus (PoUW) | Infrastructure | Phase 2 (Verified) |
| 31 | Quantum | Infrastructure | Phase 2 (Verified) |
| 32 | Community | Governance | Phase 3 (Verified) |
| 33 | BNS | Infrastructure | Phase 3 (Verified) |
| 34 | Mesh | Infrastructure | Phase 3 (Verified) |
| 35 | Justice | Governance | **Phase 4A (Verified on live testnet)** |
| 36 | Whistleblower | Governance | **Phase 4B (Verified on live testnet)** |
| 37 | Moderation | Governance | **Phase 5C (Verified on live testnet)** |
| 38 | StorageProof | Infrastructure | In the runtime; present in live chain metadata |
| 39 | ZkAttestation | Identity | In the source at spec 122; verify-only until a trusted setup exists |

`pallet_contracts` lives at index 13 and is a Substrate-standard pallet; it is
**not** counted in the 20 custom pallets.

---

## Pre-Mainnet Removal Checklist

> **No mainnet is planned.** These items stay for as long as the network is a
> testnet — removing them is a mainnet-launch task, not general cleanup. Do not
> "fix" them opportunistically; doing so would remove the bootstrap path the
> testnet still depends on.

The following are scaffolded for testnet bootstrapping and **must be removed
before a mainnet launch**:

- `pallet_sudo` at runtime index 6 (see `runtime/src/lib.rs:600` and
  AR-3 note at line 221).
- `EnsureRoot` fallbacks in all governance origin types
  (`runtime/src/lib.rs:877–920`). Tracked by the `TODO: remove EnsureRoot
  fallback before mainnet launch` marker at line 880.
- `EnsureRoot` on collective `SetMembersOrigin` /
  `DisapproveOrigin` / `KillOrigin` (`runtime/src/lib.rs:844–870`) — replaced
  by on-chain governance post-launch.

---

## Service Activation Status (Ceiba testnet)

Ports and routes below mirror `infra/docker-compose.ceiba.yml` and
`infra/nginx/nginx.conf`; if they disagree, the compose/nginx files win. Live health
belongs in `ECOSYSTEM_STATE.md` §2, not here.

| Service | Container Port | Public Route | Status |
|---|---|---|---|
| `ceiba-node` (Substrate) | 9944 (internal) | `wss://testnet.belizechain.org/ws`, `/rpc` | Live |
| Nawal AI | 8080 | `/api/nawal/` | Live; FL workload + Prometheus pending |
| Kinich Quantum | 8888 | `/api/kinich/` | Live; activation workflows pending |
| Pakit Storage | 8001 | `/api/pakit/` | Live; DAG activation workflows pending |
| Pakit P2P | 8081 (internal) | — | Live |
| UI (Maya Wallet + Blue Hole Portal) | 3000 | `/` | Page wiring tracked in `ui/UI_WIRING_STATUS_CURRENT.md` — do not restate a count here |
| IPFS Gateway | container only | `/ipfs/` (read-only) | Live, legacy pinned content |

**Note**: Pakit and Nawal local development ports differ from the Ceiba
container ports (Pakit local 8080 / Ceiba 8001; Nawal local 8000 / Ceiba 8080).

---

## GEM Contract Deployment

Contract addresses are **not** listed here. The addresses this section used to
carry were wiped by the 2026-09-29 re-genesis and must not be reused or quoted.

Current addresses live in the gem repo's deployment manifest
(`gem/deployment-*.json`) and in the UI's `CEIBA_GEM_DEPLOYMENT` config. If those
two disagree, the manifest is the record of what was deployed — verify on-chain
before quoting any address.

Contract inventory and per-contract line counts are in the gem repo's README;
this file does not restate them.

---

## Remaining Engineering Gaps (as of 2026-10-26)

Tracked because "the pallets are all in the runtime" is not the same as "the
platform is finished". Each item below was verified by reading the source, not
inferred from a planning document.

| # | Gap | Status |
|---|---|---|
| 1 | Quantum achievement-NFT auctions had no settlement path | **Closed** — `settle_auction` wired; `pallets/quantum` 104 tests pass |
| 2 | LandLedger had no zoning update path | **Closed** — `set_zoning` implemented; `pallets/landledger` 91 tests pass |
| 6 | Python integration suites tested a non-existent API | **Closed** — `112 passed, 1 skipped, 0 failed, 0 errors` on a fresh dev chain; see [`testing/GAP6_PYTHON_SUITE_REPAIR.md`](testing/GAP6_PYTHON_SUITE_REPAIR.md) |
| 7 | A chain launched **from genesis** has an empty bridge | **Closed — genesis config seeds the same 13 chains the migration does** |
| 3 | 5 weight functions are hand-derived, not benchmarked | **Closed** — both pallets benchmarked on AC power 2026-10-27; all 5 now carry `Minimum execution time` provenance |
| 4 | No custom runtime APIs; `node/src/rpc.rs:117` still returns typed stubs | **Closed** — four runtime APIs implemented and served; see below |
| 5 | No vesting, account-recovery or scheduler pallet | **Scope decision, not a defect** — see below; the previously-stated provenance was wrong |
| — | Runtime source claimed spec 122 while holding undeployed changes | **Closed** — spec 123 built, rehearsed, deployed and verified live 2026-10-27 |
| — | `WUSDC/BBZD` oracle guard checked one direction; its rate source was empty | **Closed — live in spec 124** (2026-10-27). Bilateral guard + peg fallback; 11 new tests, both halves falsified |

**Oracle guard detail — live in spec 124.** The BEX deviation guard compared
one reference rate against `amount_out / amount_in`, which equals bBZD-per-USD only
for `WUSDC -> BBZD`. In the reverse direction the same expression yields the
reciprocal, so `BBZD -> WUSDC` could never have matched and was simply not checked.

Latent rather than live: `TradingPairs` is keyed by an ordered tuple and genesis lists
only `(WUSDC, BBZD)`, so the reverse market does not exist. `create_trading_pair` is
`PairListingOrigin`-gated — meaning the moment governance listed `(BBZD, WUSDC)` it
would have created a second market for the same two assets with no price guard
whatsoever.

The fix had to run in a specific order. Widening the guard first would have been
*worse* than the bug: `get_crypto_exchange_rate` answered only when bBZD was the
quote side and returned `None` otherwise, and the guard fails closed, so the reverse
direction would have become permanently untradeable — indistinguishable from the
guard working. So `reference_rate()` now answers both orientations (inverting when
bBZD is the base) and prefers the live oracle while falling back to `BBZD_USD_PEG`
(`2_000_000`; BZ$2.00 = US$1.00 since 1978 — a *definition*, not an observation, and
the reason a missing feed no longer dead-ends the pair given that `OracleOperators`
is empty against `MinConsensusOperators = 3`).

Two duplicated ~35-line inline blocks collapsed into `enforce_oracle_deviation()`.
11 tests added. **Falsified:** reverting the gate fails 3 reverse-direction tests;
removing the reciprocal fails 4 more, including the one that *accepts* a correctly
priced reverse trade — so neither half is decorative.

Deployed 2026-10-27 via `sudo.sudo(System::set_code)` and verified by reading the
chain: spec 124, epoch still 300, `:code` blake2b matching the submitted blob, tx
version 3, and 10 blocks in 60 s. Rollback target `artifacts/runtime-spec123-live.wasm`.

**`transaction_version` 2 → 3 was 123's omission, not 124's change.** 123 appended
`BelizeX::issue_wusdc` and `burn_wusdc` without the bump its own convention requires —
verified live, where spec 123 listed 13 BelizeX calls including both while reporting
`transactionVersion: 2`. 124 adds no extrinsic; the bump corrects 123. Indices were
appended, never reordered, so no call encoding changed.

**Spec 123 detail — deployed and verified 2026-10-27.** The source had been reading
`spec_version: 122` while carrying undeployed work (measured weights for five
extrinsics, four runtime APIs, the governance and economy fixes), so the version
number no longer described the code. Bumped to 123 and deployed hot.

Procedure actually followed, in this order — the order is the point:

1. **Capture rollback first.** `artifacts/runtime-spec122-live.wasm`, 1,592,424
   bytes, read out of `:code` before any source edit. Only 117 and 121 existed;
   rolling back from 123 would otherwise have had no target.
2. **Rehearse on a throwaway chain.** `scripts/rehearse-runtime-upgrade.py` (added
   in this change) applies the blob to a local `--dev` chain and asserts both
   outcomes that matter. Result: spec 122 → 123, epoch 14400 → **300**, and
   +4 blocks in 24 s with zero error lines afterwards.
3. **Dry-run against Ceiba** (`upgrade-runtime.py` without `--execute`) to confirm
   the live epoch was already 300 and the sudo key matched `//Alice`.
4. **Submit**, with pre-state recorded: spec 122, height 298785, `:code` blake2b
   `fc42727e…`.
5. **Verify by reading the chain, not the script's report.** Post-state: spec 123,
   impl 1, transaction 2, `slot_duration` 6000 ms, `epoch_length` **300**
   (unchanged), `:code` blake2b
   `9a5cce640841f47585cb5df991b25338cc1b8091003c58824fcad3677138eb58` matching the
   submitted blob exactly, and 10 blocks in 60 s.

The rehearsal is the mitigation for the 2026-10-04 stall (a blob built without
`testnet-fast-epoch` changed `BabeEpochDuration` 300 → 14400 and stopped block
import permanently). Running the blob locally first means the target chain is never
the first place a runtime executes. Note the rehearsal *proved the distinction*: the
`--dev` chain's own native runtime reports 14400, while the blob reports 300 — the
feature lives in the blob, which is exactly the thing being deployed.

**What 123 did *not* do:** the four JSON-RPC methods are still absent from Ceiba
(`-32601 Method not found`) because the RPC *server* lives in the node binary, and
Ceiba runs the pre-123 image. The runtime half is live and provable via `state_call`
(`GovernanceApi_council_term` → 1, `BridgeApi_chain_configurations` → compact 52 →
13 chains), but exposing the methods needs a node image rebuild and redeploy. No
storage migration was needed: no layout change, no new pallet, no extrinsic added or
reordered, so `CURRENT_RUNTIME_VERSION` stays u3.

**Gap 5 detail — corrected 2026-10-27.** The previous entry said a `pallet_vesting`
"is referenced by the treasury spec". **That was not verified, and it is wrong.**
Searching the whole repo (excluding `target/` and `.git/`) for `vesting` finds no
code reference at all — no `pallet_vesting` in any `Cargo.toml`, no
`pallet_vesting::` path, no vesting call in `runtime/src/lib.rs`. The only hits are
an unchecked to-do item in `docs/technical-reference/governance/README.md`, two
generic lines in the production-audit prompt ("look for bugs in vesting logic"), and
a third-party `scalecodec` type registry. There is no treasury spec document that
mentions it. The provenance was invented, which is exactly the unsourced-claim
failure mode the repo instructions call a defect.

Verified state of the three pallets the finding names:

| Pallet | In runtime? | Evidence |
|---|---|---|
| `pallet_vesting` | No | 0 references anywhere in code |
| `pallet_recovery` (account recovery) | No | 0 references |
| `pallet_scheduler` | No | 0 references |
| `pallet_proxy` | **Yes** | 3 references in `runtime/src/lib.rs` |

So the observation is factually correct; only its stated cause was fabricated. It is
also **not a defect**. Nothing in the runtime or its specs claims these pallets
exist, so no behaviour is broken and no caller is misled.

Adding any of them is a consensus-affecting change, not a bug fix:

- `pallet_vesting` encodes tokenomics policy — who is locked, for how long, with what
  cliff. Inventing schedules would be inventing monetary policy.
- `pallet_recovery` needs a recovery-config policy (guardians, threshold, delay).
- `pallet_scheduler` needs every `Call` origin constrained before it can be enabled
  safely, and adds a scheduled-dispatch attack surface that must be reasoned about
  per call.

Each therefore needs an explicit product decision before implementation. Tracked as
open scope, not as an outstanding defect.

**Gap 3 detail (corrected 2026-10-26).** An earlier note said "24 unweighted
extrinsics". Verified against the files: **19 of 20 pallets have
benchmark-generated `weights.rs`** (18 dated 2026-10-13/10-24/10-25 plus
`belizex`). The five functions still hand-derived are `quantum`'s
`create_auction`, `place_bid`, `settle_auction`, `cancel_auction` (added while
closing Gap 1) and `zk-attestation`'s `set_registry_root` — the only pallet whose
weights file was never generated. Every other weight function carries the
benchmark CLI's `Minimum execution time` provenance comment.

Re-run after any extrinsic is added; a weight that predates its extrinsic is the
failure mode this check is for.

**Gap 3 scope, corrected 2026-10-27.** The five functions were not one job:

| Pallet | What was needed | Outcome 2026-10-27 |
|---|---|---|
| `pallet_belize_quantum` | The 4 auction benchmarks already existed; only the run was missing | **Generated** — minimum execution time `create_auction` 18.5 µs, `place_bid` 50.3 µs, `settle_auction` 115.0 µs, `cancel_auction` 13.5 µs; `BENCH_MUL=1.25` stamped in the file |
| `pallet_belize_zk_attestation` | No `benchmarking.rs`, and absent from `define_benchmarks!` | **Written, registered, generated** — see below |

### zk-attestation: why the old `weights.rs` said benchmarking was impossible

The previous file opened with a rationale for hand-writing every value: "a benchmark
cannot produce a valid proof — proving needs a witness."  That is true of a
*synthetic* proof but not of the pallet: the benchmark embeds a real Groth16 proof
and its verifying key, generated once with the `prover` feature that the runtime
build omits, and verification then takes the **acceptance** path.

That distinction mattered. The hand-derived constant for `submit_attestation` was
10 ms; the measured value is **22.6 ms** `ref_time`. The estimate under-charged by
2.3×, which is the direction that lets a block be overfilled.

The measured trait also lost the `key_len` / `proof_len` parameters, so the two
`#[pallet::weight]` call sites in `lib.rs` were updated. Both operations are
dominated by a fixed cost (four pairings; one database write) against inputs already
bounded by `MaxProofLen` / `MaxVerifyingKeyLen`, so a length term modelled nothing
real.

`the_benchmark_fixture_is_a_valid_proof` guards the fixture: it asserts the embedded
key and proof verify against the embedded public inputs and that every input is a
canonical field element. It does **not** compare bytes against a regenerated proof —
`setup_for_testing` draws from `thread_rng`, so a fresh proof is valid but never
identical.

### The `settle_auction` run failed twice before it succeeded

Worth recording because the failure mode is a trap for the next person:

`Benchmark ... settle_auction failed: Account cannot exist with the funds that would
be given`. `settle_auction` splits the escrowed bid and drains the escrow account, so
any residue between 0 and the existential deposit fails the final transfer.

The benchmark seeded the escrow with a *relative* transfer. FRAME does **not** roll
back non-whitelisted storage between benchmarks, so the escrow still held dust left
by `place_bid` and `buy_nft`, which run earlier in the same pallet. Settlement pays
out the bid exactly, so the residue was that dust — and it was under the deposit
regardless of the sale price, which is why scaling the price 10× → 25× changed
nothing. Seeding with `make_free_balance_be` (absolute) fixed it; the 200×-deposit
sale price keeps every split above the deposit as well.

**AC power.** Verified `ADP1/online = 1` before either run. This matters:
`scripts/bench_weights.sh` documents 40–80 % inflation on battery (2026-10-13:
`bns buy_domain` 114 M on AC vs 179 M on battery).

**Gap 4 detail — four runtime APIs, 2026-10-27.** `node/src/rpc.rs` carried a TODO
listing four typed RPCs "requires GovernanceApi/StakingApi/BridgeApi/OracleApi".
None of the four runtime APIs existed, so the TODO was describing a missing
prerequisite, not missing wiring.

Each pallet now has a `<pallet>/rpc-runtime-api` crate (`decl_runtime_apis!`), an
`impl_runtime_apis!` block in `runtime/src/lib.rs`, and a `<pallet>/rpc` server
crate registered via `module.merge(...)`:

| Pallet | Methods | Why an API rather than client-side storage reads |
|---|---|---|
| `governance` | `belizechain_governanceCouncil`, `…CouncilTerm`, `…Elections`, `…NextProposalId` | `CouncilMembers` and `DistrictElections` are keyed by account and district; "who is seated" needs an iteration |
| `staking` | `belizechain_stakingValidators`, `…ValidatorCount`, `…CurrentEpoch`, `…LastRewardDistribution` | `Validators` is keyed by account |
| `interoperability` | `belizechain_bridgeChains`, `…BridgeTransactions` | `ChainConfigurations` is keyed by `BridgeChain`; no key-resolvable "all transactions" index |
| `oracle` | `belizechain_oracleFeeds`, `…ManualRates` | `PriceFeeds` is keyed by `CurrencyPair` |

**Types are the pallets' own, not mirrors.** Each API returns
`pallet_belize_*::CouncilMember` / `ValidatorInfo` / `BridgeTransaction` /
`PriceFeedData` directly. A crate-local mirror would be a second definition of a
fact the chain already owns, and would drift the first time the pallet changed
shape. That costs a `serde` derive on those types (`cfg_attr(feature = "std", …)`,
matching what `pallets/oracle` already did) and the `sp-core/serde` feature, which
is what makes `AccountId32` serialize as SS58 rather than hex.

Two behaviours are deliberately *not* hidden behind convenience defaults, and the
API docs say so:

- `oracle::price_feeds()` omits pairs with no rate. `BelizeXOracle` rejects a
  trade against a missing rate rather than assuming one; a client must reach the
  same answer, so the API does not invent a default.
- `staking::current_epoch()` is a caller-driven counter, not a clock — it only
  advances when `distribute_rewards` succeeds. `last_reward_distribution()` is the
  timing signal, and a client deciding whether rewards are due must use that one.

**Gap 7 detail — corrected 2026-10-27.** An earlier entry claimed the bridge was
"permanently dead on every live chain" because the seeding migration was never
registered. **That was wrong.** `pallet_belize_interoperability::migrations::SeedChainConfigurations`
is registered in `runtime/src/migrations.rs` as step V1 → V2 (commit `aab5f4d`,
2026-10-08), which shipped in spec 122. Verified live: Ceiba holds **13**
`ChainConfigurations` entries, all enabled.

The real, narrower gap was **genesis parity**. `on_runtime_upgrade` does not run
on a fresh genesis — it fires only when `:code` changes. The pallet had no
`#[pallet::genesis_config]` (removed with the comment "GenesisConfig removed for
Substrate v42 compatibility"), so:

| Chain | ChainConfigurations |
|---|---|
| Ceiba (upgraded through spec 122) | 13 entries ✓ |
| A fresh `--dev` chain | **empty** (before the fix) |
| A future mainnet launched at spec 123+ | **empty** (before the fix) |

A chain launched from genesis had no bridge, and nothing could create an entry:
`update_bridge_config` requires the key to already exist, and there is no
`add_bridge_chain` call. The stale comment in the pallet — "Chain configurations
can be initialized through governance or extrinsics" — was false and has been
replaced.

**Fixed 2026-10-27.** `#[pallet::genesis_config]` restored on
`pallet-belize-interoperability`, seeded from `migrations::seeded_chain_configuration_entries`
— the same helper the V1 → V2 migration uses, so an upgraded chain and a launched
chain derive their registries from one definition. `node/src/chain_spec.rs` writes
the `interoperability` genesis section from it.

Two implementation notes worth keeping:

- The genesis struct holds **primitive tuples**
  (`(chain_index, enabled, min_confirmations, max_amount, fee_rate, pq_signatures)`)
  rather than `(BridgeChain, ChainConfig)`. The genesis macro requires serde even
  under `no_std`, and `ChainConfig` contains a `BoundedVec`, whose serde impls are
  std-gated. That constraint is the original "v42 compatibility" problem; the
  `community` pallet solved it the same way. `build()` reassembles the real types.
- `Pallet::encode_chain` now delegates to a crate-level `chain_index`, because the
  chain specs need the mapping before a runtime exists. The mapping is not
  duplicated — a first attempt at duplicating it got `Near`/`Sui` wrong, which
  would have seeded the wrong chains.

Verification:

| Check | Result |
|---|---|
| `cargo test -p pallet-belize-interoperability` | 119 passed (`round_trip` guard included) |
| `cargo check` std **and** `--no-default-features` | clean |
| `cargo check -p belizechain-node` | clean, no warnings |
| `build-spec --chain dev` | 13 `chainConfigurations` entries |
| Dev chain over RPC (`ChainConfigurations`) | 13 chains, identical set to Ceiba |
| `tests/blockchain` (full suite) | **36 passed** in 868 s |
| Full chain suite, fresh dev chain | **114 passed, 1 skipped** in 2,676 s |
| `tests/test_chain_spec_guard.py` | 3 passed; proven falsifiable by a 1-planck genesis edit |

The `every_seeded_chain_survives_the_index_round_trip` test exists specifically
because a wrong index does not fail loudly — it seeds a different chain, or panics
at genesis. It was proven falsifiable by injecting `Near => 12` (expected failure
observed, then reverted).

`interoperability` was the only pallet whose migration *seeds* state, so this
class of defect does not repeat elsewhere: `belizex::MigrateV0ToV1` and
`mesh::MigrateV0ToV1` only purge unreadable legacy entries, and the V0 → V1
bootstrap only writes the version counter. A fresh genesis has nothing for them to
do.

**`testnet-spec.json` must not be regenerated — now enforced.** The committed file
(last touched in `b8887a2`, 2026-09-20) predates spec 122, so its genesis section
has no `interoperability` key — it lists only `babe`, `balances`, `community`,
`governance`, `grandpa`, `identity`, `session`, `sudo`.

That is **correct, not stale**. Verified 2026-10-27 by booting a node from it: the
genesis hash is `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef`,
exactly Chain D's. Its `name` is `Jade` and `chainType` is `Live`. Ceiba boots from
this file and receives the bridge registry from the V1 → V2 migration, so the running
chain needs nothing from genesis.

The hazard is regeneration: `build-spec --chain testnet > testnet-spec.json` would
change the genesis hash, and a node refuses to open a database whose genesis does not
match — so "refreshing" the file makes the live chain unopenable. That is now caught
by `tests/test_chain_spec_guard.py`, which pins

- the sha256 of the canonicalised `genesis` section
  (`fab66b0c…64084`), so a one-planck change fails,
- the absence of an `interoperability` genesis section, since adding it is the
  specific mistake a reader of the Gap 7 report would make,
- `id` / `name` / `chainType`, so a `--chain dev` spec landing in the file is caught.

Proven falsifiable by bumping one balance by 1 planck: the guard failed, and the
file was restored byte-identically (`git status` clean). The test is in the CI
`python-tests` job, so a regeneration cannot land unnoticed. Deliberate re-genesis
requires updating the pin in the same commit that announces the new chain.

Also open, and blocking a mainnet launch rather than a testnet one:

- `MAINNET_COUNCIL: [&str; 0] = []` and `MAINNET_TECHNICAL_COUNCIL: [&str; 0] = []`
  at `node/src/chain_spec.rs:124,132` — the production chain spec would launch
  with empty councils.
- `chain-specs/chain-spec-production.json` is an 802-byte stub, not a real spec.

- `MAINNET_COUNCIL: [&str; 0] = []` and `MAINNET_TECHNICAL_COUNCIL: [&str; 0] = []`
  at `node/src/chain_spec.rs:124,132` — the production chain spec would launch
  with empty councils.
- `chain-specs/chain-spec-production.json` is an 802-byte stub, not a real spec.

### Known behavioural findings (not yet fixed)

Documented rather than silently patched; each changes consensus behaviour or
needs a governance decision.

- **District elections are one-shot, forever.** `start_district_election`
  requires `!DistrictElections::contains_key(district)` and finalize never
  removes the entry, so six districts permit six elections in total.
  The council can never be re-elected.
- **`CouncilMember` records no district.** The link exists only in
  `DistrictRepresentation`; any member→district lookup has to scan all six.
- **No global council size cap from district elections.** `CouncilSizeLimits` is
  enforced only by `add_board_member`; elections can seat up to 5 per district.
- ~~**`pallets/belizex` has no `#[transactional]`.**~~ **Retracted 2026-10-27 — not a
  defect.** The concern was that `add_liquidity`, `execute_trade`,
  `execute_multihop_trade` and `remove_liquidity` each make several
  `T::Assets::transfer` calls with `?` between them, so a later failure would leave
  earlier legs applied. It does not: `#[pallet::call]` already wraps every
  dispatchable in `frame_support::storage::with_storage_layer`, which undoes any
  storage change when the body returns `Err`. Confirmed empirically before fixing
  anything — a rollback test written to fail against the unfixed pallet passed, and
  the ledger showed the first leg had really executed.
  `settle_fill`'s manual reversal stays, since `match_order` is also called
  directly from benchmarks where no dispatch layer exists. The invariant is pinned
  by `add_liquidity_rolls_back_when_a_later_leg_fails`.
- **`emergency_type_index > 4` silently becomes `EmergencyType::Other`**, unlike
  every other index argument in `pallets/governance`, which rejects out-of-range
  values with an `Error`.

---

## Live Testnet Extrinsics Verification (2026-09-03)

The ethical safeguard and governance pallets were exercised end-to-end against the live 2-validator Ceiba testnet (`100.81.45.25` — the host **retired 2026-10-07**; this paragraph is a dated record, not current state. See `docs/ECOSYSTEM_STATE.md` for the live chain) using `@polkadot/api` (`scripts/live-pallet-integration-test.js`). All submitted extrinsics were signed, included in canonical blocks, and verified in on-chain storage:

1. **`pallet-belize-moderation` (Index 37, Phase 5C)**:
   - Call: `belizeModeration.flagContent(contentHash, 2)` (Reason: Spam).
   - In block: `#8668` (`0xf5ad18a74ab405de044fce846cdb27d648b226fecf4ea773bc3665438083da3a`).
   - Verified on-chain: `contentFlags` storage entry set to `Spam`, `flagCounts` incremented to `1`.

2. **`pallet-belize-justice` (Index 35, Phase 4A)**:
   - Call: `belizeJustice.openDispute(target, evidenceHash, 1)` (Severity: Moderate).
   - In block: `#8669` (`0xfad71a385124ce3985554db66ec647b71bb9aadf2f98344fe02985cfb624b3ea`).
   - Verified on-chain: Dispute `#2` created with status `Pending`, 100 DALLA bond reserved from disputant, target placed in `RehabStatus::InCoolingOff` until block `#1304669`.

3. **`pallet-belize-whistleblower` (Index 36, Phase 4B)**:
   - Call: `belizeWhistleblower.submitReport(commitment, target, evidenceHash, 1)` (Category: Fraud).
   - In block: `#8670` (`0xd361f3f25ffee63845c378a417aa1a9da97ab194c2a9f6205652b8cac043c546`).
   - Verified on-chain: Whistleblower report `#2` committed with status `Pending`, 10 DALLA bond escrowed, Blake2-256 domain commitment verified.
