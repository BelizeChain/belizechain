# ECOSYSTEM_STATE — BelizeChain live state and open workstreams

**This file is the single source of truth for VOLATILE facts about the BelizeChain
ecosystem: what is live right now, and what work is in flight.**

Read it before making any claim about live state, before starting any workstream, and
before writing a status report. If a fact here contradicts another document, this file
wins for *state* — for *conventions*, the repo's
`.github/instructions/belizechain_instructions.instructions.md` wins.

> **Why this file exists.** The per-repo `.github/copilot-instructions.md` files used to
> restate live values (spec versions, host IPs). Those restatements rotted: on
> 2026-10-14 six of eight repos still told agents to use host `100.81.45.25`, retired on
> 2026-10-07, and two named runtime spec 110 when the chain was on 116. A file that
> restates a value is a second copy that drifts. Conventions live in the instruction
> files; volatile state lives **here, once**.

---

## 1. How to use this file

**Rule: volatile facts must not be copied into other documents.** Link to this file
instead. If you find a live value (spec version, container count, host address, epoch
length) written into a doc outside this file, that is drift — fix the doc to point here.

**Every block below carries the date it was verified.** A value without a date is a bug.
When you re-verify, update the date. When you cannot verify, say so — do not restate an
old value as current.

**Updating.** Append to §4 and update the dated blocks in §2. Keep entries newest-first.
Do not rewrite history in this file — if a value changes, change it and note the previous
value inline, so the next agent can see what moved.

---

## 2. Live state

### 2.1 Chain — verified 2026-10-22

| | |
|---|---|
| Chain name | **Jade** (Chain D) |
| Genesis | `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef` |
| Runtime | **spec_version 117** live · impl 1 · transaction 2 |
| Head (2026-10-22) | **227,401** (`0x37849`) |
| Epoch | index **758** · **duration 300 slots** · start slot 298781245 |
| Peers | 0 (`shouldHavePeers: false` — single node, expected) |
| Repo agreement | **`runtime/src/lib.rs` reads `spec_version: 119`** — the repo is two versions ahead of the chain; see below |

**The repo is at 119 and the chain is at 117.** Four runtime changes are committed but
**not deployed**: `acknowledge_message` (interop call 12), `unlock_conviction` (governance
call 46) plus the conviction-bond enforcement, a corrected `transfer_bbzd`
self-transfer error, and the P1-08 BelizeX matching engine (spec 119, §4.3).

**119 is the first of these that needs a storage migration.** `OrderEntry` gained an
`escrow` field, so `pallet-belize-belizex`'s `STORAGE_VERSION` went 0 → 1 and
`runtime/src/migrations.rs` `CURRENT_RUNTIME_VERSION` went 2 → 3. The migration **purges**
`OrderBook` / `PairOrders` / `AccountOrderCount` rather than reinterpreting old bytes —
any limit orders resting on the live chain are dropped, which is correct because none can
ever have filled. 118's changes did not require a migration (`ConvictionLocks` is a new map
that starts empty) but the conviction change is behavioural: a vote at level > 0 now needs
the bond free (2,000 DALLA at level 3). Any upgrade bringing either forward must be built
with `--features testnet-fast-epoch`.

**117** was applied hot (`sudo.sudo(System::set_code)`, 2026-10-22) from 116, with no
node restart. Blob 1,492,283 bytes, blake2b-256
`e4f072179dc326b5063df2c4bebcccd00c57d93b50574812206f22bc29811ca0`, built with
`--features testnet-fast-epoch` and boot-verified at **duration 300, 1 authority** before
submission. Content = removal of `pallet-belize-community`'s `FeeCalculator` trait and the
whole SRS fee-exemption path (see §4.6). Verified live afterwards: `Community` metadata
has **no** `FeeExemptionUsage` storage, no `FeeExemption*` event, error or constant, and
still 15 calls / 24 events / 39 errors / 24 storage items. Rollback blob
`/data/upgrade-117/onchain-code-116-rollback.wasm` (1,489,640 B, blake2b
`5ec421fab9fbf6733a43779484fd46b7aebff33a00e0355d8a12ed30e69b3afe`).

⚠️ **`BabeApi_current_epoch` field [8:16] is the epoch's START slot, not the
current slot** — it is constant for the whole epoch. Read the slot from the
block's BABE `PreRuntime` digest instead (the live head advances ~1 slot/block).
Arithmetic that treats it as the current slot reports the wrong time to the next
boundary.

⚠️ **The genesis hash does NOT identify the chain.** Chain D re-genesised from Chain C's
spec, so it shares C's hash exactly. Disambiguate by **name (Jade) + head height + date**,
never by hash alone. Full lineage: `docs/operations/CHAIN_LINEAGE.md`.

Re-verify:
```bash
R=https://belizechain.org/rpc
curl -s -H 'Content-Type: application/json' -d '{"id":1,"jsonrpc":"2.0","method":"state_getRuntimeVersion","params":[]}' $R
curl -s -H 'Content-Type: application/json' -d '{"id":1,"jsonrpc":"2.0","method":"system_chain","params":[]}' $R
```
⚠️ The runtime-version method is **`state_getRuntimeVersion`** (integer result). There is
no `runtime_version` method — using it returns nothing and looks like a dead node.

### 2.2 Host — verified 2026-10-22

| | |
|---|---|
| Host | **`ceiba`** — Tailscale `100.119.97.38`, LAN `10.0.0.19` |
| Access | `ssh wicked@ceiba` (Tailscale). LAN fallback `10.0.0.19` |
| Containers | **19 running** · 0 exited · 0 failed units · host uptime 15 days |
| Node image | `belizechain/ceiba-node:76dea3f-stable2609-20261019` |
| Public ingress | **Oracle edge proxy** `129.213.95.153` → ceiba over the tailnet |

**Image pins on the host** (`/opt/belizechain/.env`, verified 2026-10-22):

| Service | Pinned image | Rollback tag |
|---|---|---|
| `ui` | `belizechain/blue-hole-portal:b08cd28-ceiba-20261022` | previous pin, see `.env` backups |
| `maya` | `belizechain/maya-wallet:b08cd28-ceiba-20261022` | previous pin, see `.env` backups |
| `nawal` | `belizechain/nawal:b3f1500-mesh-20261022` | `belizechain/nawal:39c4076-auth-20261020` |
| `nawal-mesh` | *(shares `NAWAL_IMAGE`)* | same |
| `kinich` | `belizechain/kinich-quantum:b1a048f-clean-20261022` | `belizechain/kinich-quantum:5220c8a-pgfix1` |
| `org` | `belizechain/org-site:698b2be-org-20261021` | — |

⚠️ **`.env` backups are the rollback mechanism**, not a documented previous-tag list.
Before any pin change the live file is copied to `backups/.env.pre-<change>-<stamp>`; to
roll back, restore that file and `up -d <service>`. Images are never pruned, so the old
tag still exists locally.

⚠️ **`100.81.45.25` / `10.0.0.222` are the RETIRED host.** Decommissioned 2026-10-07 — it
runs no containers. Any instruction naming it is stale. The name `ceiba` was reused for
the new box on 2026-10-07, so **the hostname no longer disambiguates** the two machines —
the IP does.
⚠️ `10.0.0.229` is **stale** (that was WiFi; WiFi was disabled 2026-10-07). LAN is `10.0.0.19`.

Deploy model: `/opt/belizechain` on ceiba is **not a git checkout**. Deploys copy changed
files in; script + doc details in `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md`.

### 2.3 Public endpoints — verified 2026-10-22

All five names resolve to `129.213.95.153` (Oracle edge, TLS terminates there):

`belizechain.org` · `www` · `testnet` · `wallet` · `portal` · `explorer`

| Kind | Endpoint |
|---|---|
| HTTP RPC | `https://belizechain.org/rpc` |
| WS RPC | `wss://testnet.belizechain.org/ws` |
| Per-name frontends | apex/`www` = org site · `testnet`/`portal` = Blue Hole Portal · `wallet` = Maya Wallet · `explorer` = explorer |

**Frontend images, 2026-10-22:** both built on ceiba from `ui@b08cd28` and pinned in
`/opt/belizechain/.env` — `UI_IMAGE=belizechain/blue-hole-portal:b08cd28-ceiba-20261022`
(226 MB), `MAYA_IMAGE=belizechain/maya-wallet:b08cd28-ceiba-20261022` (231 MB). Rollback
pin = the prior tags (`blue-hole-portal:3e30ab9-ceiba-20261021`,
`maya-wallet:nawalproxy-20261022`); `.env` backup
`backups/.env.pre-ui-b08cd28-20261022-164522`. Verified after the roll: six names serve the
right app, `/rpc` and `/ws` answer 405 (POST/upgrade only) on every name, `/api/nawal/health`
and `/api/pakit/health` 200, zero absolute `belizechain.org` URLs baked into either bundle,
and the new page strings (`Register Handle`, `Chain of Title`, `Verify Commitment`) are
present in the wallet image — the fabricated `Wicked (Founder Node)` / `Ambergris Caye &
Cayes` strings are gone from both.

**Backend services, 2026-10-22:** `nawal` and `kinich` were rebuilt on ceiba from the
commits above and rolled one at a time, each verified before the next.

- `nawal:b3f1500-mesh-20261022` (from `nawal-ai@b3f1500`) — healthy, logs
  `Connected to BelizeChain: ws://ceiba-node:9944` and
  `Identity verifier initialized (mode=production)`. Rollback `nawal:39c4076-auth-20261020`.
- `kinich-quantum:b1a048f-clean-20261022` (from `kinich-quantum@b1a048f`, 1.71 GB) —
  healthy, `Connected to BelizeChain`, `persisted API job state via PostgreSQL`, local
  simulator enabled, Azure/IBM disabled. `/health` and `/api/v1/status` 200 in-network and
  through the edge. Rollback `kinich-quantum:5220c8a-pgfix1`.
- `ceiba-nawal-mesh` — new, see §4.5. Brought the host to **19 containers**.

⚠️ **`gem` and `belizechain.org` needed no rebuild.** `gem@6ace3ff` changed only
`.gitignore` plus agent files, and `belizechain.org@d76c381` changed only a comment in
`lib/constants.ts` that the bundler strips — so `ORG_IMAGE` still points at
`698b2be-org-20261021` and that is correct, not stale. A pin that lags HEAD is only a
defect when the deployed bytes differ.

⚠️ **`infra`'s `.env.example` and `README.md` are repo-only.** They do not exist in
`/opt/belizechain`, so a "missing on host" report for them is expected — only
`docker-compose.ceiba.yml` is a deployed file.

⚠️ Hostname routing is by `$root_backend` map in the edge nginx. Every name also serves
`/rpc`, `/ws`, `/api/{nawal,kinich,pakit}`, `/ipfs` — the frontends derive those from
`window.location.origin`, so **any name that does not proxy them breaks the UIs**.
⚠️ From the workstation, chain RPC is reachable at `https://belizechain.org/rpc`. The
internal `ws://100.119.97.38:9944` needs tailnet access.

---

## 3. Authority order — when documents disagree

1. **Live chain / host** (RPC, `docker ps`, storage reads) — ground truth, always.
2. **Source of truth in code** — `Cargo.toml`, `runtime/src/lib.rs` `construct_runtime!`,
   `pallets/*/src/lib.rs`, `api_server.py`.
3. **This file** — for ecosystem state.
4. **Narrative docs** (`README.md`, `docs/*`) — lowest. Treat any live value in them as
   unverified until checked against 1–2.

Rule: **never resolve a disagreement by preference or by what sounds newer — read the
source.** A doc claiming a value that the code contradicts is drift, and the code wins.

---

## 4. Open workstreams

### 4.1 Honesty sweep (Tier-2 code audit) — IN PROGRESS

Scope: accidental mocks, incomplete implementations, dead ends, fail-open paths — claims
that succeed without doing the work. **Distinct from** the runtime-upgrade track (§4.3);
they are different workstreams that were previously only tracked in chat, which is why
this file now exists.

| repo | status |
|---|---|
| `belizechain` | **DONE** — `landledger` anchor, `community` deposit, orphan storage (277 items / 0 orphans), doc-API sweep (`e506ef4`) |
| `kinich-quantum` | **DONE** — 5 rounds, `2cce5db`; tracker fabrication, XCM false success, ZNE/readout, CID/proofs |
| `pakit-storage` | **SWEPT 2026-10-14** — 13 commits `3478c91`→`28b6ea7`; declared-but-unbound public API, README documented a facade that never existed, dual module identity, inert ML, fabricated quantum results, ZK proofs never generated, a logging regression introduced *and* reverted here, CI red since 2026-10-03 → green; see below |
| `nawal-ai` | **DONE 2026-10-20** — audit findings **F-01 … F-15 all closed**; 3 extrinsics corrected, 12 impossible storage reads remapped or made honest, mesh comms fixed and a runner added; see below |
| `gem` | **SWEPT 2026-10-22** — SDK stopped reporting failed calls as success (5 findings, `dc8bb38`); `access_control` admin set capped at `MAX_ADMINS` and its storage keys pinned by test (`e14f39f`). **34 proptest harnesses re-run 2026-10-23: 34 passed, 0 failed, 4.65s.** Actions remains dead for this repo (minutes exhausted), so all verification is local. **Not audit-clean beyond AUDIT-GEM-01.** |
| `infra` | **SWEPT 2026-10-14** — `api-gateway` + `indexer` (the only unaudited services) are **clean**. 2026-10-23: `relayer/` added, see below |
| `ui` | partial — fabricated service fallbacks removed 2026-10-07; **page-level seed data remains** |
| `belizechain.org` | **SWEPT 2026-10-22** (`cb10306`) then **again 2026-10-23** (`9529238`) — four fabricated feature pages rewritten (jaguar-mode, zk-proof, bridges, radio); the 93–100% coverage claim replaced with measured figures; 18→12 decimals; consensus pallet index corrected to 30; jaguar-mode's "one enforcement point" corrected to four; treasury corrected to 7-of-12 threshold multi-sig; the burn paths that DO exist documented (`burn_dalla`, `governance_burn`); Next.js upgraded 16.4.0→16.4.3. **COMPLETED 2026-10-23** — every page and every MDX doc read line-by-line. This pass found the **GEM SDK is not on npm** (`npm view @belizechain/gem-sdk` → 404), so all six install snippets were wrong; **Ed25519 does not exist** in any pallet (attestations are `blake2_256` of an off-chain-salted attribute); the **cookies page claimed no browser storage** while the language switcher writes the `belizechain.locale` localStorage key; Kinich QML is **4,314 lines across 22 files**, not "8,245+ / 33"; the faucet drips **100 DALLA**, not 1,000; Tutorial 5's marketplace **contract does not exist**; Nawal's teacher is **`Qwen/Qwen3.8-27B`**, not DeepSeek-Coder-33B, and **Opacus is declared but never imported**; the genome is **~1.3 KB**, not 5 KB; Pakit's ">3x compression" was **withdrawn by its own README**; Kinich's "Falcon" schemes are **ML-DSA aliases**. Gates: tsc 0, eslint 0, build exit 0, doc-claims PASSED. |

**Bridge state (2026-10-23).** The authoritative page is `docs/bridges/README.md`; scope for a
real relayer is `docs/bridges/RELAYER_SCOPE.md`. Summary:

- **XCM does not exist and will not be added.** `pallet_xcm` has zero commits in git history.
  The `staging-xcm`, `xcm-builder`, `xcm-executor`, `polkadot-cli` and `polkadot-primitives`
  workspace dependencies were removed from `Cargo.toml` on 2026-10-23 — all five were
  declared-but-unconsumed. `staging-xcm` remains transitively via `pallet-contracts` and
  `cumulus-primitives-core`; that is an SDK internal, unreachable from any runtime call.
  XCM requires a cumulus collator and a parachain slot, which contradicts the sovereignty model.
- **What exists:** `pallet-belize-interoperability` at runtime index 27 — 51 chain identifiers,
  ML-DSA-87 signatures, `BridgeAsset` limited to DALLA/bBZD, KYC L2 for messaging and L3 for
  operators, escrow, liquidity pools, disputes.
- **Gap 1 closed 2026-10-23:** `acknowledge_message` (call_index 12) now flips
  `CrossChainMessage.delivered` to `true`. Before it, nothing in the pallet ever wrote `true`,
  so outbound messaging was an append-only log. **Its weight is derived, not measured** — see
  the AMENDMENT block in `pallets/interoperability/src/weights.rs`. Regenerate before mainnet.
- **Gap 2 staffed 2026-10-23:** `infra/relayer/` exists and runs, but acknowledges nothing by
  design. The only shipped adapter is `null`; compose defaults to `RELAYER_ADAPTER=null` +
  `RELAYER_MODE=observe`. There is no deployed bridge contract on any foreign chain, so a
  relayer that acknowledged anything would be lying. A growing
  `belizechain_relayer_pending_messages` backlog is the expected signal.
- **Not built:** message delivery to a foreign chain. `send_cross_chain_message` records intent;
  no worker delivers it.

**Governance conviction — unbounded vote weight, found and fixed 2026-10-23.** The most
serious finding of the sweep, and it was live.

`cast_vote` and `reveal_vote` computed `(conviction as u32).max(1)` from a caller-supplied
`u8` with no ceiling, so `cast_vote(id, 0, 255)` bought **255× voting weight** and took no
lock. Nothing constrained it: no `MaxConviction`, no clamp, no balance lock anywhere in the
pallet. The pallet's own doc table described a 1×/2×/3×/6× ladder with 0/7/30/90-day locks
that the code did not implement. Pre-existing since `e8aa532` (2026-03-06).

Fixed by implementing the documented design rather than capping it down to what the code
happened to do:

- `MAX_CONVICTION = 3`, with `CONVICTION_MULTIPLIERS = [1, 2, 3, 6]` and
  `CONVICTION_BOND_MULTIPLIERS = [0, 1, 5, 20]` as named tables, not literals.
- Conviction above 3 returns `InvalidConviction` from both vote paths.
- The bond is reserved **before** weight is computed, so it leaves `free_balance` and
  already discounts the stake component. It is non-additive across proposals, tops up when a
  voter raises their level, and the unlock block only ever moves later.
- `unlock_conviction` (call 46) returns the bond after the lock expires, or fails with
  `ConvictionStillLocked`. At runtime the base is 100 DALLA, so levels cost 100 / 500 /
  2,000 DALLA for 7 / 30 / 90 days.

10 tests pin it: the table, the ceiling from both entry points, the bond amount per level,
non-additivity, top-up, early-reject, release, and no-lock. **The weight is derived, not
measured** — see the AMENDMENT block in `pallets/governance/src/weights.rs`; regenerate with
`scripts/bench_weights.sh pallet_belize_governance` before mainnet.

**`infra` sweep detail (2026-10-14).** `api-gateway` and `indexer` are the newest services
(added 2026-10-11) and had never been audited. They came out **clean**, and unusually so —
the docs state what is *not* verified, which is the opposite of the failure mode this sweep
looks for. Verified rather than trusted:

- README endpoint lists match the code exactly: 13 REST routes, 10 GraphQL queries, 9 gateway
  RPC methods. `belizechain_getChainInfo` is genuinely **not** served — it appears only in
  comments explaining the deliberate omission.
- The README's load-bearing claim (state is pruned, so a from-genesis backfill is impossible)
  **reproduces against the live node**: `state_getStorage` fails with `UnknownBlock: State
  already discarded` at blocks 100/1000/50000, and succeeds ~5 blocks back from the head.
  Boundary probe: readable at head−250, discarded at head−255 — consistent with the stated
  256-block window.
- Cross-service contract holds: the gateway queries `indexer.{blocks,transactions,gaps}` — the
  exact tables the indexer creates — behind a `to_regclass` existence guard.
- No committed secrets or build output: only `.env.example` is tracked (`dist/`, `node_modules/`,
  `.env` are ignored), and `keystores/`, `backups/`, `data/` have **0** tracked files.
- The services report absence honestly: misses report the range actually searched and
  distinguish a *confirmed* absence from an *unestablished* one.

One real finding, annotated not rewritten: `infra/docs/NEXT3_ROADMAP_LANDED_2026-09-10.md`
claims "all 4 validators healthy", which describes the local `docker-compose.validators.yml`
swarm rather than Ceiba — the live chain is **single-authority** (verified 2026-10-14 via
`GrandpaApi_grandpa_authorities`: a 41-byte payload = 1 × 32-byte key + 8-byte weight). Its
`ui` claims also conflict with `ui/UI_WIRING_STATUS_CURRENT.md`.

That swarm is not a throwaway harness, though, and "healthy" is only the container health
check. Measured 2026-10-24: all four containers are running against
`generated_specs/chainspec-mainnet-raw.json` (`chainType: "Live"`), genesis
`0x8b66304f…` — **a fifth chain, not Jade** — and it is forked and dead. `#13753` carries
two distinct best hashes across the four nodes, finality stopped at `#13751`, and the last
block timestamp is `2026-09-15T13:34:42Z`, so it has been stalled **38 days**. Logs date
from 2026-09-13; the 2026-10-18 restart only reset container uptime. All four publish
9944/30333/9615 on `0.0.0.0`. See the P1-24 section in
`docs/security/security-audit-results.md` for the evidence and the open decision.

**`pakit-storage` dependencies (2026-10-23).** Nine open Dependabot alerts, all cleared
on patch-level pins: `aiohttp 3.14.3 -> 3.14.4` (eight alerts: 3 high, 3 medium, 2 low)
and `black 26.3.1 -> 26.10.1` (one high). Verified, not assumed — both install and import
in `.venv`, `pytest` gives **633 passed / 10 skipped**, `ruff` clean.

Two things worth knowing before reading those numbers as "closed":

- **The alerts will lag the fix.** Dependabot tracks `requirements.txt`, whose only
  content is `-r pakit_requirements.txt`, so the pin lives in the included file and the
  alert state only updates after GitHub's "Configured Graph Update: pip" job runs. It was
  still `queued` when this was written. The fix is in the repo; the badge is not yet.
- **`black --check` reports 41 files would be reformatted — on both versions.** The
  identical 41 on the old 26.3.1 pin means this is pre-existing, not caused by the bump.
  Reformatting them is a separate decision and was deliberately not bundled.

**`pakit-storage` sweep detail (2026-10-14).** Security posture came out clean; the
**public API surface and the README** did not. Verified rather than trusted:

- Auth is fail-closed: the dev bypass needs `PAKIT_ENV` in {development,dev,test}, otherwise it
  logs and returns **False**; admin compares with `hmac.compare_digest` (503 unset / 403 mismatch).
  `payment_strict` defaults **True**. All 8 documented `/api/v1/*` routes exist — no fabricated endpoints.
- **Declared-but-unbound exports (real, silent).** `pakit/__init__.py` listed `Pakit`,
  `PakitStorageEngine`, `StorageTier` in `__all__`, but `find_packages()` does not install the
  repo-root `__init__.py`, so `from pakit import Pakit` raised `ImportError`. `backends/__init__.py`
  did the same with `DagBackend`. Both were resolved with lazy `__getattr__` re-exports. The
  regression test fails on pre-fix code (proven via `git worktree` of HEAD).
- **The README documented an API that has never existed.** 22 call sites used a `pakit` facade
  object (`pakit.store`, `pakit.quantum_compress`, `pakit.store_ipfs`, `pakit.pin`,
  `pakit.train_ml_optimizer`, `pakit.enable_ml_optimization`, `pakit.get_metrics`, …) and four
  more used `storage.get_dag_stats`/`get_info`; `DagBlock.create` and `MerkleDAG.add_block` do not
  exist either, and the MerkleDAG proof methods are `@staticmethod` taking *blocks*, not
  `target_hash=`/`root_hash=`. Also: `tier="auto"` could never match (`StorageTier` is a plain
  Enum, so the string passed through and was used as a member), `local_path=` is `storage_dir=`,
  `get_efficiency_report()` returns a **nested** dict, an invented cost/CO₂ metrics payload
  (`data_farm_killer_score: 98.5`) was removed, and one code fence was unbalanced (39 — odd).
  Features with no implementation are now marked **roadmap**, not shown as working code.
- ⚠️ **The logging finding was real but the fix was wrong, and this file recorded the wrong
  conclusion.** `api_server.py` and `blockchain/storage_proof_connector.py` are the only two
  modules using **loguru**, which interpolates **brace-style only**; every other module uses
  stdlib `logging` (percent-style). 49 brace-style calls in those two files were rewritten to
  `%s`, which made loguru print the placeholder literally and **drop the argument** — turning
  49 working calls into 49 silently wrong ones. Reverted from the exact pre-fix lines, plus 4
  further pre-existing percent-style calls in the same files. The guard test asserted `%s`
  **everywhere** and so passed through the regression; it is now style-aware (loguru → braces,
  stdlib → percent) and checks placeholder/argument counts. Verified it flags 52 sites on the
  pre-fix revision and none now. **Lesson: check which logging library a file imports before
  "fixing" a convention.**
- Checked and **correct**: `auth/`, the payment gate, route wiring, the tracked `.env` (no secrets,
  but it *was* tracked while `.gitignore` only covered `env/` — untracked, since secret scanning
  is disabled on that repo). `.env`'s redundancy with `.env.example` was noted.
- **Not fixed, flagged:** `setup.py` installs 23 generic top-level packages via `find_packages()`
  (`core`, `ml`, `p2p`, `auth`, `blockchain`, …), which pollutes the global namespace; because
  `core/__init__.py` imports via `pakit.core.*` while `api_server.py` imports flat `core.*`, the
  same classes can load as two distinct module objects, so `isinstance` across the two can fail.

**`pakit-storage` sweep, second pass (2026-10-14, later the same day).** The first pass found the
doc/API fabrications; this pass found **live bugs**, then built the features the README had been
describing. All verified against source and, where possible, against the live chain.

- **Dual module identity (live bug).** `pakit.core.compression.CompressionAlgorithm is not
  core.compression.CompressionAlgorithm` — two class objects, so `NS.AUTO != FLAT.AUTO` and a
  comparison silently took the wrong branch, raising `ValueError: Algorithm not available`.
  Resolved with an `_AliasLoader`/`_AliasFinder` so both import conventions yield the **same**
  module object (`32232cc`).
- **ML was completely inert.** Nothing ever called `ModelRegistry.register`, so the registry was
  always empty and `MLDAGIntegration` fell back on every call (`fallback_activations: 1`,
  `ml_suggestions_used: 0`). Now wired as **advisory hints only** (`6ff6e80`), behind
  `ENABLE_ML_OPTIMIZATION` (default off), with the hint validated against real
  `CompressionAlgorithm` members so determinism is never traded for a suggestion.
- **Quantum results were fabricated.** `_submit_quantum_job` returned `success=True` with invented
  counts (`{"00":500,"01":300,...}`) and an invented `circuit_depth`, against a backend name
  (`qiskit_simulator`) that is not a Kinich local alias. Rewritten to report failure honestly
  (`86bfd94`), plus a real `KinichClient` (`33da8f3`).
- **ZK storage proofs were never generated.** The call used
  `generate_storage_proof(content_id=, data_size=, storage_location=)` against a real signature of
  `(block_cid, block_data, merkle_proof, replication_peers)`, and read `zk_proof.get('type')` on a
  dataclass. A **test asserted the buggy kwargs**, which is why it went unnoticed (`4bf14bf`).
- **The documented `Pakit` facade now exists** (`core/facade.py`, `aa88f69`) and delegates only —
  so the imports the README always showed finally resolve. Then the features it described were
  actually built: a budgeted `ResearchJobRunner` (`f75d2e5`) that estimates cost via Kinich's
  `/api/v1/jobs/estimate`, **refuses to submit over budget**, and awards an achievement only on a
  genuine success; a `BeliNftConnector` (`fa3bde0`) whose `can_mint()` proves the signer *is* the
  contract owner before submitting; and IPFS `pin`/`unpin`/`is_pinned` (`7bc2dae`).
- **Signer wiring was missing entirely.** `get_storage_proof_connector()` accepted no keypair, so
  the economy/BNS/contracts connectors could never sign and every write returned "No keypair
  configured". Now resolves `PAKIT_SIGNER_URI` via one shared loader (`e59deeb`).
- ⚠️ **The `pakit.api_server` story — my first reading was wrong, and the live check disproved it.**
  `main()` ended in `uvicorn.run("pakit.api_server:app")`, and `pakit/api_server.py` **has never
  existed in git**. But the deployed `pakit/__init__.py` was a 297-byte redirect
  (`__path__ = [dirname(dirname(abspath(__file__)))]`) that made `pakit.api_server` resolve to
  `/app/api_server.py` — so **it worked in production**. Live check: `ceiba-pakit` **Up 20h,
  healthy**, health 200. The `integration` job broke when `32232cc` replaced that redirect with the
  alias finder (whose `_FLAT_PACKAGES` excludes `api_server`); `e59deeb` repointed uvicorn at
  `api_server:app`. **Do not report a passing assertion as a bug without checking the mechanism it
  encodes** — that test was correct at the time.
- **CI was red from 2026-10-03 to 2026-10-14**; last green run `2026-10-03T20:17`. Two causes: a
  pre-existing black failure on `blockchain/bns_integration.py`, and the `integration` break above.
  Both workflows now pass. Note **black runs before ruff, so a black failure hides any ruff
  violation** — check the gates separately.
- `.github-instructions.md` describes a layout that does not exist (everything under a `pakit/`
  package; `pakit/api_server.py` and `pakit/README.md` are neither present). It is not loaded by
  Copilot and nothing references it — marked superseded with the falsehoods named, flagged for
  deletion. Also present in `ui`; **not** an org convention (2 of 8 repos).


**`nawal-ai` sweep, first pass (2026-10-19).** Verified against the live runtime
rather than by reading alone: every `call_module`/`call_function` pair was resolved
against `construct_runtime!`, and every `call_params` payload was composed with
`substrate-interface` against Jade (spec 116) in a throwaway environment that never
submits.

- **Three extrinsics could never succeed** (`4c481d8`). Composing the shipped
  payloads raises `ValueError: Parameter ... not specified`, so none could be included
  in a block:
  - `Oracle.submit_iot_data` — sent `feed_type`/`quality_score`; the pallet takes
    `feed_type_index`, `data_hash` and the five separate quality metrics.
    `prepare_oracle_submission()` **already returned the correct shape**; the submitter
    rebuilt it wrongly and read a non-existent `feed_type` key.
  - `Community.record_participation` — sent a free-text activity name; the pallet takes
    a `u8` `activity_code` and returns `InvalidActivityType` otherwise. Added
    `ACTIVITY_CODES` from `pallets/community/src/types.rs`; unknown names and the two
    attestation-gated codes (2, 3) now fail before submission.
  - `Community` SRS reads — `get_srs_info` read five fields (`participation_count`,
    `volunteer_hours`, `education_modules_completed`, `green_project_contributions`,
    `monthly_fee_exemption`) that **do not exist** on `SRSData`, so every real call
    raised `KeyError` and was swallowed into `None`. Now reads `total_contributions`
    plus the separate `CompletedEducationCount` / `GreenContributionStats` maps.
    Volunteer hours and the fee-exemption **entitlement** have no on-chain source and
    are reported as 0 rather than invented.
  - Education and green contributions now use `complete_education_module` /
    `contribute_to_green_project`: `record_participation` codes 4/5 do **not** feed the
    SRS education/sustainability scores.
  - **The tests had mocked the fabricated shapes**, which is why all three survived.
    Corrected, and proven both ways: the new tests fail on the pre-fix source
    (`git stash`) and pass after.
- **Two latent fabrication paths removed** (`4dae6af`):
  `_qsvm_forward` returned `np.random.rand(...)` (dead code, but exported and
  test-asserted — the exact shape that becomes live); it now delegates to the real
  HTTP path. `nawal/server/__init__.py` set `globals()[name] = None` on any import
  failure; making it raise exposed a genuine circular import with `api_server`, so the
  eager exports now raise with context and `ServerConfig` resolves lazily via PEP 562.
- **Checked and correct:** all 13 extrinsic names exist; `payroll_connector` already
  sent `{}` to `batch_payment(origin)` with a comment explaining that the pallet pays
  registered employees; no `mock_mode=True` default exists anywhere in production code.
- **Flagged, not changed:** `control/executor.py` registers `respond`/`reason`/
  `validate`/`search`/`execute` as stubs returning `{"stub": ...}`, and
  `controller.py` lists them in `_DEFAULT_TOOLS`; `ExecutiveController` has no
  production caller. `_classical_fallback` returns a seeded random-projection matrix
  labelled as a fallback (counted in `stats["fallback_calls"]`, but indistinguishable
  from real output at the call site). Both need an owner decision.
- **Not verified:** the `ss58_format=42` settings. The chain's prefix is 1981, but
  `system_properties` returns `{}` and the value is cosmetic for AccountId encoding —
  left alone rather than churned.

**`nawal-ai` sweep, storage reads (2026-10-19, `4ef7059`).** The same method applied to
reads: every `storage_function` name was checked against **live runtime metadata**
(`substrate-interface` → `get_metadata()`, Jade spec 116) rather than against the source
tree, because a first pass that only scanned `pallets/*/src/lib.rs` reported `System.*` as
missing — a false positive, since `frame_system` is not in this repo. **12 of 26**
`(pallet, storage item)` pairs do not exist. Every one is wrapped in `try/except`, so each
failure was silent: the read returned `0` / `None` / `[]` and looked like empty state.

The pattern behind it: the blockchain layer was written against a Polkadot-shaped chain.
`Ledger`, `Bonded`, `CurrentEra`, `ValidatorPrefs`, `ActiveValidators`,
`ErasValidatorReward` and `ValidatorMetadata` are all Substrate-standard names that
`pallet_belize_staking` does not have.

Corrected where a real equivalent exists:

| read | was | now |
|---|---|---|
| `get_stake_info` | `Staking.Ledger` | `Staking.Validators[addr].stake` (no own/delegated split) |
| `get_current_era` | `Staking.CurrentEra` | `Staking.CurrentEpoch` |
| `get_active_validators` | `Staking.ActiveValidators` | keys of `Staking.Validators` |
| `get_stake` | `Staking.Bonded` | `Staking.Validators[addr].stake` |
| `get_operator_stats` | `Oracle.OperatorStats` | `Oracle.OracleOperatorStatsMap` |
| `get_payroll_stats` | `Payroll.PeriodStats` | `Payroll.GlobalStats` — a single all-time aggregate; no per-period buckets and no government/private split, so the result is marked `scope: "all-time"` |

Made honest where **nothing** exists on chain, rather than substituting a plausible value:
`get_commission` (no `ValidatorPrefs`, no commission field on `ValidatorInfo`),
`get_validator_rewards` (`EpochRewards` is one pool per epoch; there is no per-validator
entry), `get_reputation_score` (no reputation registry in `pallet_belize_identity` —
reputation is a local federated-learning signal), `get_employee_paystub`
(`pallet_belize_payroll` stores a `payment_commitment` hash and keeps the breakdown
off-chain by design), `get_pending_submissions` (`pallet_belize_oracle` has no pending-IoT
queue; `PendingKycSubmissions` is KYC).

Six tests asserted the fabricated shapes and were corrected; they fail on the pre-fix
source and pass after. Full suite: 4031 passed, 1 skipped.

**`nawal-ai` mesh comms layer (2026-10-19, `75a8d71`).** The mesh transport is a
**communication layer**, not the training topology, and it was documented as a delivered
v1.1.0 feature while being unusable three independent ways:

- It bound `127.0.0.1`, so a node that is supposed to accept inbound gossip could not. Bind
  host is now configurable (`NAWAL_MESH_LISTEN_HOST`), loopback default retained.
- `discover_peers` queried `Staking.ValidatorMetadata` for a `network_address` field that does
  not exist. **Neither on-chain source can supply an `ip:port`**: `ValidatorInfo` holds stake,
  scores and a free-text `location` blob, while `pallet_belize_mesh` stores LoRa node IDs and
  GPS coordinates. The query could only ever return zero peers. Replaced with peer exchange
  over `/peers`, seeded by `NAWAL_MESH_BOOTSTRAP`; isolation is now reported as a configuration
  fact instead of a silent empty list.
- `/peers` did not serve `public_key`, so a discovered peer could never have a signature
  verified and **every message from it was dropped**. Served now, and both payload shapes are
  accepted.

8 tests asserted the impossible chain query and were rewritten; all 8 fail on the previous
module and pass on the new one. 4032 passed, 1 skipped.

**Flagged, not changed:** `peer_id` is caller-supplied and not bound to the advertised public
key, so a peer can claim another's `peer_id` and have its own signatures accepted under that
identity. Fixing it changes the identity model.

**Architecture question — DECIDED 2026-10-19.** Nawal's training topology is a **mesh**;
federated learning is the interim implementation, not the destination. The decision, and the
reasoning behind every choice that follows from it, is recorded in
[`nawal-ai/docs/architecture/INTENT.md`](../../nawal-ai/docs/architecture/INTENT.md) — that file is
authoritative for *what Nawal is meant to be*. Do not re-litigate it from chat history.

What changed in code (`46a6a7f`, `622c618`, and the genome/architecture commits of
2026-10-19): `GossipStrategy` implements the existing `AggregationStrategy` protocol, so the
central aggregator is one implementation among several rather than the only path;
`server/adapter_exchange.py` moves LoRA-sized deltas over the mesh; `nawal mesh` is a real
CLI runner; sequence primitives (Mamba SSM, LSTM/GRU, RMSNorm) and a graph MoE expert are
buildable from a genome.

⚠️ **What is still not deployed, and the doc says so:** no compose service starts a mesh node,
`server/adapter_exchange.py` has no serving-path importer, and no code produces a chain graph
for the graph experts. `INTENT.md` §7 carries the "correct but not in the serving path" table;
an entry is deleted the moment the module reaches the serving path.

Method that has worked: verify every finding against source; **prove the new test fails on
the old code** (use a `git worktree` of HEAD) rather than only passing on the new; use
`git stash` to prove a fix changed behaviour.

**`nawal-ai` mesh serving path — 2026-10-22 (uncommitted at the time of writing).** The
2026-10-19 work built the mesh primitives; this pass made them reachable from a deployed
node. All of it is verified by tests locally and **none of it is deployed**.

- **Adapter bytes now move over the mesh.** `blockchain/mesh_network.py` serves
  `GET /adapters` and `GET /adapters/{id}`, and `fetch_adapter()` downloads and verifies a
  payload against the id it was requested under. That is the production importer
  `server/adapter_exchange.py` never had — before this, a peer announced a CID
  (`send_model_delta`) that nothing could resolve.
- **The mesh is a deployment unit.** `infra/docker-compose.ceiba.yml` gains a `nawal-mesh`
  service (`ceiba-nawal-mesh`) running `python -m cli.commands mesh`. It binds `0.0.0.0`
  inside the compose network and publishes no host port.
- **Identity is file-backed.** `--key-file` writes `${DATA_ROOT}/nawal/mesh/ed25519.key` on
  first run. A generated key would rotate per restart, and peers pin an id to the first key
  they see — a rotating identity is refused by every peer that already knew it.
- **Peer keys are pinned (TOFU).** `_remember_peer_key` refuses to rebind a peer id to a
  different public key, and discovery drops peers with a malformed or contradictory key.
  ⚠️ **Residual, not fixed:** first contact is unauthenticated. A peer that wins the race to
  announce an id defines it, because peer exchange carries no proof that the announcer holds
  the key. Closing that needs the peer id to become a fingerprint of the public key — a
  wire-format change.
- **The chain-graph producer exists.** `blockchain/chain_graph_producer.py` reads
  `Balances.Transfer` and `Economy.BbzdTransferred` (both confirmed against
  `pallet-balances` 33.0.0 and `pallets/economy/src/lib.rs:529`, not assumed) and returns
  `ChainTransfer` records; `nawal chain-graph` runs it. Graph experts now have an input feed.
  It does not yet feed training automatically — it produces a snapshot on request.

**🔴 Found during this pass: 22 loguru calls were using `%s`.** Loguru interpolates
**braces only**; `logger.warning("x %s", v)` prints a literal `%s` and **drops `v`**. The
calls came from two different commits — the 2026-10-19 mesh work (9) and the "make failures
honest" pass (5, in `payroll_connector`, `staking_interface`, `substrate_client`,
`validator_manager`, `oracle_pipeline`) — plus 8 in new code. Fixed to `{}`, and
`tests/test_logging_format_style.py` now guards **both directions**: loguru modules must use
braces, stdlib modules must use `%`. The guard is non-vacuous — it reports 10 offenders on
the pre-fix HEAD and none after. This is the same defect that made 49 lines in
`pakit-storage` silently wrong in the opposite direction; the lesson generalises: **check
which logging library a module imports before "fixing" a format convention.**

### 4.2 Code scanning / GHAS alerts — verified 2026-10-14

> **Why nothing was visible before 2026-10-14.** Code Security and Secret Protection are
> GitHub **Advanced Security** features. The org is on **GitHub Team**, where they are
> **paid add-ons for private repos** and free only on public ones — so six of the eight
> repos were structurally blind: `code_security: disabled`, and the alerts API answered
> **403**. CodeQL had been running and uploading successfully the whole time; the findings
> were simply never exposed. Code Security was enabled on all six (≈$30/committer/mo, one
> seat); Secret Protection stays off, as chosen. Enabling it is per-repo:
> `gh api -X PATCH repos/BelizeChain/<repo> -f 'security_and_analysis[code_security][status]=enabled'`.
>
> **Do not read "0 open" as "clean" on a repo whose Code Security is disabled** — that is
> the state that hid 307 open alerts.

Counts verified 2026-10-22:

> ⚠️ **13 actionable alerts were fixed in the working trees on 2026-10-22 and are
> uncommitted/unpushed**, so the counts below will not move until those commits land and
> CodeQL re-scans. Two of the ten kinich alerts are **not** code defects: `py/ineffectual-
> statement` on `await ticker_task` / `await task` flags an `await` whose value is discarded,
> which has effects. Dismiss those with justification rather than restructuring a test to
> satisfy the analyzer.

| repo | open | note |
|---|---|---|
| `belizechain` | **0** | public, so scanning always worked (132 dismissed / 35 fixed historically) |
| `infra` | **0** | 11 total, all fixed |
| `gem` | **0** | 17 total, all fixed |
| `nawal-ai` | **0** | was 256; the Trivy base-image noise was resolved |
| `ui` | **3** | 2 `js/trivial-conditional` (`land/page.tsx:183,188`), 1 `js/unused-local-variable` (`documents/page.tsx:152`) |
| `pakit-storage` | **1** | Trivy `CVE-2024-23342` (`ecdsa`) — **no fixed version exists** |
| `belizechain.org` | **1** | `js/stored-xss` — see below |
| `kinich-quantum` | **35** | 25 `py/undefined-export` (known FP) + 10 actionable: 3 `unused-global-variable`, 3 `empty-except`, 2 `multiple-definition`, 2 `ineffectual-statement` |

- **`infra` 11 → 0** and **`belizechain.org` 8 → 1**, fixed 2026-10-13/14 by the same two
  moves: workflow-level `permissions: contents: read`, and SHA-pinned third-party actions
  (CodeQL exempts the `actions/*` and `github/*` orgs, which is why `actions/checkout@v7`
  never fires while `tailscale/github-action@v4` does). Dependabot entries keep the pins
  fresh.
- **`belizechain.org`'s remaining `js/stored-xss` is a CodeQL modelling gap, not a live
  hole.** `sidebar.tsx` passes `href` from nav data to `next/link`; a `toSafeHref()` guard
  (root-relative only) was added and the *old* alert closed, but CodeQL re-raised it at the
  new line because **it does not model a custom JS function as a sanitizer**. Do **not**
  contort the code to satisfy the analyzer. Needs a dismissal with justification, or an
  untested inline-guard experiment.
- **Pending user action:** kinich's 40 dismissal requests have **not** landed
  (`state=dismissed` is empty). Approve in **Security → Code scanning → Dismissal
  requests**. pakit's 6 did land.
- ⚠️ org is near its **2,000 monthly Actions minutes**; private-repo CI is billed. Don't
  push speculatively.

### 4.3 Runtime upgrade track

- **Rule that must not be broken:** every runtime build passes
  `--features testnet-fast-epoch`. `BabeEpochDuration` is compile-time (300 with / 14 400
  without); a blob built without it silently changes epoch length and permanently halts
  block import. That is how Chain C died 2026-10-04.
- **Weights:** benchmark on **AC power only** (~1.6 GHz on battery vs 4.0–4.5 GHz on AC
  inflates weights 1.4–1.8×). `scripts/bench_weights.sh` refuses to run off AC and applies
  `BENCH_MUL=1.25`. `scripts/check_weights_freshness.sh` fails when a pallet's `weights.rs`
  predates its `lib.rs`; it needs `fetch-depth: 0` or it never fires.
- Upgrades are applied as a hot `sudo.sudo(System::set_code)` — **never** a regenerated
  spec (`testnet-spec.json` is gitignored and pins genesis). Rollback blob hashes live in
  `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md`.
- **Node image rebuilt 2026-10-19** (`76dea3f-stable2609-20261019`) for five new RustSec
  advisories that appeared 2026-10-14: RUSTSEC-2026-0346/-0347/-0348/-0349 (`rustls`
  ≤0.23.45 → **0.23.46**) and RUSTSEC-2026-0363 (`wasmtime` 36.0.16 → **36.0.18**).
  Both are node-level crates, so this needed a node image, **not** a runtime upgrade.
  Verified by booting before deploy: spec 116, `BabeApi_current_epoch` duration **300**,
  `belizechain_getChainInfo` live-state shape; then deployed and observed the live
  **epoch 612 → 613** boundary with duration held at 300 and continuous authoring.
  Rollback: `.env.bak-pre-node-76dea3f-20261019` + the `ff74ee9-stable2609-20261012` image.
  ⚠️ Still open and **not** fixed by this: `hickory-resolver 0.24.4` (GHSA-5j98-2g5x-46v6)
  needs `libp2p-dns 0.45` upstream.

#### P1-08 matching engine — built 2026-10-24, committed, NOT yet deployed

`pallet-belize-belizex` limit orders went from "written but never matched" to a real
escrowed price-time book. Committed on branch `belizechain` (spec 119), **not live**; the
live chain still runs the stub at spec 117.

- Escrow at placement into `OrderEscrowPalletId` (`py/bzord`): bid → quote
  `ceil(amount * price / PRICE_SCALE)`, ask → base `amount`. Refunded on cancel,
  including the partial-fill remainder.
- Matching runs inside `place_limit_order` against the opposite side of `PairOrders`,
  bounded by `T::MaxOrdersPerPair`; fills settle both legs through `AssetLedger`.
  Fill price is the **maker's**, so the taker keeps the improvement.
- Self-trade is skipped, not rejected. `SelfTradeNotAllowed` remains unraised by design.
- Expired makers are purged and refunded lazily during matching.
- Settlement is ordered **plan → settle → apply**, so a rejected ledger leg leaves both
  orders resting with their escrow intact instead of recording a fill that never moved
  funds. The first leg is reversed if the second is rejected.
- Fills round their quote **down**, placement rounds the escrow **up**. Per-slice ceilings
  summed over a whole order can exceed the single ceiling taken at placement, which
  stranded a fractional-price bid permanently after a few fills.
- `STORAGE_VERSION` 0 → 1 with `pallets/belizex/src/migrations.rs` **purging** the book
  (the `OrderEntry` encoding changed, so old bytes cannot be reinterpreted);
  `runtime/src/migrations.rs` `CURRENT_RUNTIME_VERSION` 2 → 3 runs it.
- ⚠️ `WeightInfo::match_orders` is **derived, not measured**. A
  `match_orders(n: Linear<1, MaxOrdersPerPair>)` benchmark now exists and passes, but it
  measures the fill shape only — a walk can consist entirely of expired-order purges, so
  the bound is `cancel_order() + execute_trade()` per iteration, written in terms of those
  two so regenerating either cannot silently under-charge it. At `MaxOrdersPerPair = 20`
  that is ~4.7 ms composed against a 2 s block. Regenerate with `scripts/bench_weights.sh`
  (AC power only) before mainnet.
- ⚠️ Time priority is order-id ascending, not a stored timestamp.

#### Benchmark smoke test — 5 failures fixed 2026-10-24

`scripts/bench_smoke.sh` runs the **real release node** (`benchmark pallet --chain dev`),
not the pallet mocks. Five benchmarks were red at `770b2a6` and none was caused by P1-08:
`governance::cast_vote` (conviction bond underfunded — the mock's `ConvictionBondBase` is
`MinimumDeposit`, the runtime's is 10× that), `oracle::verify_merchant` (P1-06 moved the
gate to `TechnicalCouncilMajority`; the benchmark still used `Signed`),
`mesh::submit_mesh_transaction` (P1-07 added sr25519 verification; it sent a dummy
signature), and `belizex::{add_liquidity, execute_trade, remove_liquidity}` (pair `(0,1)` is
DALLA/bBZD — different ledgers, only DALLA was funded; `MockAssetLedger` maps every asset
to `Balances`, which is why the mock never saw it).

The belizex fix added `AssetLedger::set_balance`, gated on
`#[cfg(feature = "runtime-benchmarks")]`, implemented in the runtime adapter. Traps:
`sp-core/full_crypto` **cannot** be enabled from a pallet's `runtime-benchmarks` feature
(it propagates into the runtime WASM build and breaks `app_crypto`); `sp_io::crypto::*`
needs a keystore that `impl_benchmark_test_suite!` mocks lack; `KeyTypeId` is `[u8; 4]`.

### 4.4 Documentation continuity — IN PROGRESS (opened 2026-10-14)

- ✅ This file created as the single state/doc anchor.
- ✅ Compose-vs-CI guard added to `infra/deploy/check-doc-claims.sh` (**CHECK 6**) — fails
  when a `${VAR:?}` in a compose file is missing from the `env:` block of the CI job that
  runs `docker compose config`. That trap broke `infra` CI on 2026-10-12 and had already
  fired twice before.
- ✅ `belizechain.org` had no `agents/`/`instructions/`/`prompts/` surface; now byte-identical
  to `ui`'s (closest sibling — also Next.js), so the profile is one profile, not eight
  drifting copies.
- ✅ All 8 `copilot-instructions.md` rewritten: volatile values removed, replaced with an
  imperative pointer here. Before this, **6 of 8 repos still named the host retired
  2026-10-07** and two named spec 110 while the chain ran 116.
- ✅ Retired host swept out of current-state docs and code across 7 repos — see §4.4.1.

#### 4.4.1 Host sweep — DONE 2026-10-14 (8 commits, all pushed)

`100.81.45.25` (retired 2026-10-07) removed from live code and current-state docs:

| Repo | Commit | What |
|---|---|---|
| `belizechain` | `b8e2e87` | this file; instructions pointer; `PHASE_STATUS.md` annotated |
| `ui` | `fde1579` | 2 probe routes, 2 pages, e2e + indexer test |
| `infra` | `c16c207` | README, shared/README, `benchmark_tps.ts` |
| `gem` | `7aba977` | 8 sdk examples + 3 scripts + 8 docs + 1 doc comment |
| `kinich-quantum` | `4c503a0` | instructions, readiness doc |
| `nawal-ai` | `281af73` | `run_fl_round.py`, 2 guides |
| `pakit-storage` | `e53f2e5` | instructions, `test_live_chain.py` |
| `belizechain.org` | `fa39728` | instructions pointer; agent surface added |

**Left deliberately** — each is a rationale, not a stale value; rewriting it deletes the
reason: `infra/docker-compose.ceiba.yml:539`, `infra/deploy/lib/host-env.sh:6`, and dated
records (`CHANGELOG.md`, `*_2026-*.md` baselines/drills, frozen `docs/audits/`,
`gem/deployment-*.json`) which were **annotated, not rewritten** — rewriting a dated record
falsifies it. In-repo scripts that run inside the container default to `ws://ceiba-node:9944`
(the in-network name), not the public endpoint.

**Still open:**
- ❗ **`belizechain.org` prints live values on public pages**, so it is the most exposed
  surface for this class of drift — see §4.1.

#### 4.4.2 Agent-surface hardening — DONE 2026-10-14

- ✅ All 8 `instructions/*.instructions.md` now lead with this document in *Sources Of Truth*
  and carry a **Verify Before Asserting** section. `applyTo` attaches these files to every
  code file, so they are the one place an agent is guaranteed to see the pointer — the
  highest-leverage spot, and until now it was empty.
- ✅ All 8 `agents/belizechain.agent.md` state that live state is not memory, and list this
  document first in the Working Method.
- ✅ `belizechain.org` gained the `.github/hooks/` it was missing. All 8 repos now carry the
  same surface: 3 agents, 1 instructions file, 1 prompt, 2 hook files.
- ✅ **The ops guard was broken.** `.github/hooks/belizechain-ops-guard.py` matched `ssh`
  against `100.81.45.25 | 10.0.0.222 | ceiba`. Both IPs have been dead since 2026-10-07, so
  the guard only ever fired on the hostname form. **Proven, not inferred:** piping
  `{"command":"ssh wicked@100.119.97.38"}` returned `permissionDecision: "allow"` before the
  fix and `"ask"` after; a benign `ls` returned `allow` both times. All 7 repos repaired.
  This is drift hiding in the one file nobody re-reads — a safety guard is not re-opened on
  a host migration.

**Blocked — needs a decision:**
- 🔴 **`ui` and `gem` do not track the agent surface at all.** Both exclude `.github/agents/`,
  `.github/hooks/`, `.github/instructions/`, `.github/prompts/` in **`.git/info/exclude`** — a
  per-clone file that is never pushed. The surface in those two repos therefore exists only
  on this machine, edits to it are invisible to `git status`, and a fresh clone gets nothing.
  Until that is lifted, "all 8 repos share one profile" is true only locally. (The same file
  also hides `ui`'s `blue-hole-portal/test-results/`, which is legitimate.)
- ✅ **`ui/blue-hole-portal/src/app/page.tsx` — the fabricated validator panel is GONE**
  (verified 2026-10-21). It was removed in `ba29bba` ("stop the portal fabricating its
  validator fleet"). The panel now maps over `useStaking()` — i.e. `staking.validators` —
  and every field it cannot read says so: `Not published` for PoUW quality, `Not tracked
  by pallet` for slashes, plus an explicit "No validators registered" empty state. Repo-wide
  grep for `Edge-Validator`, `Zero Drift` and `30334` returns nothing.
  This entry said the panel was still fabricated until 2026-10-21; it had already been fixed.
- ⚠️ `belizechain/scripts/live-pallet-integration-test.js` defaults its treasury key to a
  **committed-looking BIP39 mnemonic** in source (`TREASURY_URI || 'coil apart warrior…'`).
  The stale workspace-root duplicate used the safer `//treasury` dev URI and has been
  deleted. Worth a look even though it is testnet.

---

#### 4.4.3 Docs checker now runs in all 8 repos — DONE 2026-10-14

Only `infra` ran `check-doc-claims.sh`; the other seven had **0** callers, which is how a
host retired on 2026-10-07 survived in six of them unnoticed. It now lives once, at
`belizechain/scripts/doc-claims/` — public, so a sibling can fetch it with no token and
no cross-repo credentials — and all 8 repos run it in CI. `infra`'s copy is now a shim;
its `deploy/lib/host-env.sh` stays, because eight other infra scripts source it.

**The first run failed in all seven fetching repos, and that was a defect in this change.**
The workflow checks the script out at `.doc-claims-src`, a full clone of `belizechain`, and
`md_files()` walked it — reporting belizechain's own workspace-relative links
(`../../../ui/…`, `../../../gem/…`) as dead in whichever repo was calling. Fixed by
excluding `.doc-claims-src` from the walks; proven both ways against a real single-repo
checkout (with the exclusion → PASSED; reverted → the four failures reproduce verbatim).

**Cross-repo links now SKIP rather than FAIL.** `check_dead_links` reports a target that
resolves outside the repo root as `⏭️ SKIP … escapes this checkout`. Before that change any
correct `../../belizechain/…` link failed single-repo CI — which is what allows the
live-state pointer to sit in all 8 instruction files without turning CI red.

### 4.5 Mesh as the training topology — DEPLOYED 2026-10-22

The mesh stopped being a library and became a deployment unit. `ceiba-nawal-mesh` is now
running on the host (container 19 of 19).

| What | State |
|---|---|
| `nawal-mesh` compose service | **running** — `ceiba-nawal-mesh`, healthy, image `nawal:b3f1500-mesh-20261022` |
| `python -m cli mesh` entrypoint | **works** — `cli/__main__.py` added so the container command has no `RuntimeWarning` |
| File-backed signing key | **proven** — `/data/nawal/mesh/ed25519.key`, mode 0600, pubkey identical across a real restart |
| Adapter exchange over HTTP | **works** — `GET /adapters`, `GET /adapters/{id}`, verified on arrival by `fetch_adapter` |
| Peer identity pinning | **works** — `_remember_peer_key` refuses a rebind of an id to a new key |
| Chain-graph producer | **works** — `nawal chain-graph`, reads the two real transfer events |

**Verified against the live chain, not the source tree:** `Balances.Transfer { from, to,
amount }` (`pallet_balances` 33.0.0, runtime index 4) and `Economy.BbzdTransferred { from,
to, amount }` (`pallets/economy/src/lib.rs:529`). `Economy` has **no** `Transfer` event.

**Tests:** 95 pass across the mesh suite; 11 are new adapter tests using real sockets, 10
are new producer tests.

⚠️ **Deployed, and idle by construction.** `NAWAL_MESH_BOOTSTRAP` is empty, so the node has
`peers_count: 0` and serves nobody — a second peer must exist before this does any work.
It is a running endpoint waiting for a peer, not a working mesh. Off-switch:
`docker compose stop nawal-mesh`.

⚠️ **The node does not announce its own isolation at boot.** `discover_peers()` — which is
where the "mesh node {} is isolated" warning lives — is only reached when discovery is
invoked, so a freshly started node logs `Mesh network started` and nothing about being
alone. `GET /health` is the honest signal (`peers_count: 0`, `adapters: true`). Do not read
a quiet log as a connected node.

⚠️ **`/data/nawal/mesh` and `/data/nawal/adapters` must be owned by uid 999** (`nawal` in
the image) — `/data/nawal` itself is `root:root`, so the container cannot create these
subdirectories. Same trap as the other `/data/*` mounts. And because there is no passwd
entry for 999 on the host, `ls -l` shows them as `dnsmasq`; the numeric uid is what counts.

⚠️ **Still true, unchanged:** the chain-graph producer is a CLI snapshot rather than a feed
the training loop pulls, and first contact with a peer is unauthenticated — pinning
protects a known peer id, not the first claim of one.

### 4.6 UI-08 exports — VERIFIED against live metadata 2026-10-22

All **28** chain calls behind the 25 "REAL" dead exports resolve against live runtime
metadata. **None is fabricated, so none should be removed for that reason.** The
wire-vs-build split is a product question, not a correctness one.

Two corrections to `ui/docs/audits/AUDIT-UI-08-WIRING-PLAN.md`, both found by asking the
chain:

- **`registerBelizeID` is not a blocked product decision.** `Identity.register_identity(name)`
  is documented "Register a new identity **for the caller**" — self-service, one parameter.
  The plan called it "build — self-registration is a product decision". It is wireable; the
  service's `(address, data)` signature is what does not match, not the pallet.
- **`unstakeDalla` cannot be an amount-based unstake.** `Staking` has no partial unbond:
  `leave_validators()` takes no amount, is validator-only, and is followed by
  `withdraw_unbonded()`. The service already documents this and ignores its `amount`
  argument — so the `/staking` page's 24-hour unbonding notice describes a chain capability
  that does not exist for delegators.

`reportTrainingContribution` **fails closed** and cannot pollute state: `submit_model_delta`
requires an `ActiveFLTask` whose `task_id` matches and an `encrypted_delta` of ≥16 bytes, so
the shipped placeholder (`taskId=0`, `'0x'`) always errors. It belongs to the Nawal client,
as `nawal-ai/docs/architecture/INTENT.md` already states.

⚠️ **Verifying this needed three corrections to the checker before it was trustworthy** —
`storage` is `{prefix, entries: [...]}`, `calls` is `{ty}` resolving to `def.variant.variants`,
and metadata names are **snake_case** while the polkadot-js surface is camelCase. The first
two runs reported **0/28** and **8 missing**, both entirely false. A scan that reports
"everything is missing" is a broken scan, not a broken system.

**Wiring outcome 2026-10-22:** all 25 dead exports are resolved — **18 wired, 2 deleted, 5
deferred**; see `ui/docs/audits/AUDIT-UI-08-WIRING-PLAN.md` §0.2–§0.6. Findings that belong
here:

- **Two surfaces were reporting actions that never happened.** `/governance`'s create tab
  called `preventDefault()` and then reported *"Proposal submitted! Bonded 100 Ɗ"* — no
  transaction, and `proposeTreasurySpend` charges **no deposit** at all. `/staking` explained
  the unbonding rules but offered no button. Both now call the real extrinsics.
- **`proposeTreasurySpend` emits `TreasurySpendProposed`, not `Proposed`.** The service read
  the wrong event, so the returned proposal id was always `-1`.
- **The SRS fee discount never existed on chain, and is now removed.** `FeeCalculator` was
  implemented and `CommunityFeeCalculatorProvider` was defined, but **nothing consumed the
  trait**, so `calculate_effective_fee` never affected a transfer. Two further breakages
  confirmed it was never live: `FeeExemptionMonthlyLimit = ConstU32<100>` landed as **100
  planck** (not the "100 dBZD" its comment claimed), and the `FeeExemptionApplied` /
  `FeeExemptionLimitReached` events had already been deleted as never emitted, leaving
  `FeeExemptionUsage` provably empty. Deleted from pallet + runtime + UI, with 13 tests
  (the pallet suite went 97 → 84). Verified: `cargo test -p pallet-belize-community` 86 passed, runtime `cargo check` clean.
  **Needs a runtime upgrade + `spec_version` bump to take effect; no storage migration is
  required** (the only removed storage item was empty, and storage keys are per item name).
- **Two `/governance` and `/bns` surfaces were rendering data that could never be true.**
  The council tab hardcoded invented districts and councilors; the primary-handle card could
  never leave `None` because `isPrimary` was hardcoded `false`. Both now read the chain.
- **`/send` now accepts a BNS name.** Resolution happens once at review, the confirm modal
  states the resolved SS58, and the send uses that captured address; an unresolved name is a
  hard error and a recipient edited after review is rejected.
- **`registerBelizeID` was publishing PII to a public map, and the KYC plan never needed it.**
  The service joined 7 fields (name, DOB, nationality, address, district) into
  `Identity.register_identity(name)` — a field the pallet documents as *"Display name (not
  PII-critical but bounded)"*, stored plaintext in the public `Identities` map, and capped at
  `MaxNameLen = ConstU32<64>` so most real Belizean names (measured 78–99 bytes) were rejected
  outright. **The tiered KYC plan is real but unrelated**: `KycLevel` derives purely from
  attestations (L1 = SSN, L2 = + Passport, L3 = + Biometrics), and an `Attestation` stores only
  `blake2_256(salt ++ plaintext)` with the salt deliberately kept off chain. A grep for
  `date_of_birth|nationality|residence|age_verify` across `pallets/` + `runtime/` returns zero
  non-test hits. **The design was always: issuer holds the PII, chain holds a hash.** Fixed to
  send a single display handle; `getDisplayName` returns only the first line so a legacy or
  third-party record cannot surface extra fields. No cleanup needed — the 4 on-chain identities
  at head 226,827 are validator names with no PII, so the bad path never carried citizen data.
- **The live chain is nearly empty of the data several UIs read.** Measured at head 226,827:
  `Oracle.MerchantCategories` 0, `Bns.DomainRegistry` 0, `Bns.PrimaryDomain` 0,
  `Bns.HostedWebsites` 0, `Governance.DistrictRepresentation` 0, `Community.GreenProjects` 5.
  The newly-wired council tab correctly renders six "Vacant" seats.

**Deployed 2026-10-22.** `ui@b08cd28` is live on Ceiba (both images rebuilt there, pins and
verification in §2.3) and the fee-path removal is live as **spec 117** (§2.1). The removal
was confirmed in live metadata after the upgrade, not inferred from the source tree:
`Community` exposes 24 storage items with no `FeeExemptionUsage`, and no `FeeExemption*`
event, error or constant.

⚠️ **`reportTrainingContribution` was deleted, not wired.** Its only real extrinsic
(`staking.submit_model_delta`) requires an active FL task and a ≥16-byte delta, so the
shipped placeholder always failed. It belongs to the Nawal client, which should call
`submitModelDelta` with the full payload — see `nawal-ai/docs/architecture/INTENT.md`.

## 5. Standing traps (durable — these do not go stale)

- **A checker that walks its own checkout reports its own repo's links against the caller.**
  The shared doc-claims workflow checks the script out *inside* the repo it is checking, so
  the checker must exclude that path (`*/.doc-claims-src/*`) from every file walk. Watch for
  this shape whenever a tool is vendored into the tree it inspects.
- **"0 open" is not "clean" unless the scanner is enabled.** Six repos reported zero
  findings for months purely because `code_security` was disabled and the API returned 403.
  Enabling it revealed **307 open alerts**. Check the scanner is on before trusting the
  absence of findings.
- **Never restate a volatile value in a second document.** It will drift. Link here.
- **Verify before asserting.** Unsourced live claims are how two workstreams ended up
  tracked only in chat.
- **A doc that merely mentions a dead host is invisible to `check-doc-claims.sh`** — it
  checks links, compose/env coverage, Node alignment and image pins, not prose IPs. Grep
  for dead hosts by hand after any host retirement.
- **A safety guard is the file nobody re-reads.** `.github/hooks/belizechain-ops-guard.py`
  matched `ssh` against a host retired weeks earlier and so passed the live host straight
  through — a guard that silently stopped guarding. Any host/volatile sweep must include
  `.github/hooks/`, `.git/info/exclude` and `.github/workflows/`, not just prose docs.
- **An agent surface excluded from version control is not a surface.** `ui` and `gem` hide
  `.github/{agents,instructions,prompts,hooks}` in `.git/info/exclude`, so those files exist
  only on one machine and edits to them never appear in `git status`. Verify a
  customization file is *tracked* before relying on it.
- **A checker that a document can trip is a checker with a false-positive class.**
  `check_dead_links` extracted link targets with a regex that ignored inline code spans, so a
  doc *explaining* link syntax — including this file, describing that very check — was read as
  a dead link to the literal word `path`. Fixed by stripping code spans first. Rule: when a
  doc that documents a tool breaks that tool, fix the tool, not the wording.
- **`shellcheck deploy/*.sh` in CI passes no `--severity`, so an `info`-level finding fails the
  build.** A bare `sed 's/`…`//g'` trips SC2016 (backticks look like command substitution).
  The literal backticks are the point, so the fix is a scoped `# shellcheck disable=SC2016`
  with the reason, not quoting changes.
- **`docker compose config` aborts on the FIRST missing required var**, so one new
  `${VAR:?}` turns a whole CI job red while validating nothing. Now guarded by CHECK 6.
- **A `permissions:` block REPLACES the token defaults** rather than adding to them — list
  every scope a job uses, including `actions: read` if it uploads SARIF.
- **The 46-odd dismissed code-scanning alerts are verified false positives**, not deferred
  work: a PEP 562 lazy module CodeQL cannot follow (27×), pip's **vendored** copies that
  Trivy mis-attributes to the installed package, and test constructs that *are* the
  assertion. Do not "fix" them by contorting the code.
- **A `../` path in `.github/` is one level short.** `copilot-instructions.md` sits in
  `.github/`, so reaching this repo's own `docs/` needs `../docs/` — and reaching a sibling
  needs `../../belizechain/docs/`. Every one of the 8 files got this wrong on 2026-10-14
  because the path was copied from a repo-root README, where `../belizechain/` *is* correct.
  Agents and instructions files sit one level deeper still (`../../../`).
- **A backtick path is not a checked path.** `check-doc-claims.sh` validates markdown-link
  syntax `[x](path)` only, so the 35 inert `` `../../docs/…` `` spans were never verified —
  and were all wrong. They resolve now (checked by hand), but nothing enforces it, so they
  can rot silently. Prefer a real link where a link is what you mean.
- **Cross-repo links cannot be validated in single-repo CI**, and only `infra` runs the
  checker at all (1 workflow of 8 repos). `check_dead_links` now reports a link that
  resolves outside the repo root as `⏭️ SKIP … escapes this checkout` rather than dead —
  a skip, never a pass, so it stays visible. Do not read that skip as "verified".
