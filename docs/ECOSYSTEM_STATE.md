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

### 2.1 Chain — verified 2026-10-14

| | |
|---|---|
| Chain name | **Jade** (Chain D) |
| Genesis | `0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef` |
| Runtime | **spec_version 116** · impl 1 · transaction 2 |
| Head (2026-10-14) | **110104** |
| Epoch | index 367 · **duration 300 slots** · 114-byte `Epoch` |
| Peers | 0 (`shouldHavePeers: false` — single node, expected) |
| Repo agreement | `runtime/src/lib.rs:479` also reads `spec_version: 116` |

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

### 2.2 Host — verified 2026-10-14

| | |
|---|---|
| Host | **`ceiba`** — Tailscale `100.119.97.38`, LAN `10.0.0.19` |
| Access | `ssh wicked@ceiba` (Tailscale). LAN fallback `10.0.0.19` |
| Containers | **16 running** · host uptime 1 week |
| Node image | `belizechain/ceiba-node:ff74ee9-stable2609-20261012` |
| Public ingress | **Oracle edge proxy** `129.213.95.153` → ceiba over the tailnet |

⚠️ **`100.81.45.25` / `10.0.0.222` are the RETIRED host.** Decommissioned 2026-10-07 — it
runs no containers. Any instruction naming it is stale. The name `ceiba` was reused for
the new box on 2026-10-07, so **the hostname no longer disambiguates** the two machines —
the IP does.
⚠️ `10.0.0.229` is **stale** (that was WiFi; WiFi was disabled 2026-10-07). LAN is `10.0.0.19`.

Deploy model: `/opt/belizechain` on ceiba is **not a git checkout**. Deploys copy changed
files in; script + doc details in `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md`.

### 2.3 Public endpoints — verified 2026-10-14

All five names resolve to `129.213.95.153` (Oracle edge, TLS terminates there):

`belizechain.org` · `www` · `testnet` · `wallet` · `portal` · `explorer`

| Kind | Endpoint |
|---|---|
| HTTP RPC | `https://belizechain.org/rpc` |
| WS RPC | `wss://testnet.belizechain.org/ws` |
| Per-name frontends | apex/`www` = org site · `testnet`/`portal` = Blue Hole Portal · `wallet` = Maya Wallet · `explorer` = explorer |

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
| `nawal-ai` | not started |
| `gem` | not started |
| `infra` | **SWEPT 2026-10-14** — `api-gateway` + `indexer` (the only unaudited services) are **clean**; see below |
| `ui` | partial — fabricated service fallbacks removed 2026-10-07; **page-level seed data remains** |
| `belizechain.org` | not started |

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
harness rather than Ceiba — the live chain is **single-authority** (verified 2026-10-14 via
`GrandpaApi_grandpa_authorities`: a 41-byte payload = 1 × 32-byte key + 8-byte weight). Its
`ui` claims also conflict with `ui/UI_WIRING_STATUS_CURRENT.md`.

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


Method that has worked: verify every finding against source; **prove the new test fails on
the old code** (use a `git worktree` of HEAD) rather than only passing on the new; use
`git stash` to prove a fix changed behaviour.

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

| repo | open | note |
|---|---|---|
| `belizechain` · `ui` | **0** | public, so scanning always worked (`belizechain`: 132 dismissed / 35 fixed) |
| `infra` | **0** | 11 total, all fixed |
| `gem` | **0** | 17 total, all fixed |
| `pakit-storage` | **1** | Trivy `CVE-2024-23342` (`ecdsa`) — **no fixed version exists** |
| `belizechain.org` | **1** | `js/stored-xss` — see below |
| `kinich-quantum` | **49** | 35 CodeQL + 14 Trivy; 25 are `py/undefined-export`, the known FP class |
| `nawal-ai` | **256** | **all Trivy**; the bulk are no-fix base-image noise, not our code |

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
- ⚠️ **`ui/blue-hole-portal/src/app/page.tsx` — the validator panel is fabricated.** The
  Aura→BABE label is fixed (`7b7f9a7`), but the block around it is hardcoded markup: a
  **second validator** ("Edge-Validator-2", "Byzantine Backup & Validator", "1.2ms Zero
  Drift") that does not exist — the live authority set is **1** — on ports **30333/30334**
  where only 30333 is real. It sits beside one genuinely live value
  (`systemInfo?.blockNumber`), which makes the invented rows read as verified. That is a
  content decision, not a label fix: the owner's call.
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
