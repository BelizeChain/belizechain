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
| `pakit-storage` | not started |
| `nawal-ai` | not started |
| `gem` | not started |
| `infra` | not started (alert hygiene done separately, §4.2) |
| `ui` | partial — fabricated service fallbacks removed 2026-10-07; **page-level seed data remains** |
| `belizechain.org` | not started |

Method that has worked: verify every finding against source; **prove the new test fails on
the old code** (use a `git worktree` of HEAD) rather than only passing on the new; use
`git stash` to prove a fix changed behaviour.

### 4.2 Code scanning / GHAS alerts — verified 2026-10-14

| repo | open | note |
|---|---|---|
| `belizechain` · `ui` · `gem` · `infra` | **0** | clean |
| `pakit-storage` | 1 | Trivy `ecdsa 0.19.2` — **no fixed version exists** |
| `nawal-ai` | 268 | all Trivy; the large majority are no-fix base-image noise, not code |
| `kinich-quantum` | 41 | 35 CodeQL + 6 Trivy, **all false positives** — dismissal requests still awaiting approval |
| `belizechain.org` | 1 | `js/stored-xss` — see below |

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
- 🔜 The shared `instructions/*.instructions.md` files do **not** yet cite this file in their
  *Sources Of Truth* section. They are auto-attached by `applyTo` on every code file, so they
  are the highest-value place for the pointer. Small edit × 8 repos.
- 🔜 Loose untracked duplicate at the **workspace root** (outside every repo):
  `/home/wicked/Projects/Belizechain/scripts/live-pallet-integration-test.js` — stale copy of
  the in-repo `belizechain/scripts/` version, still carrying the dead host. Nothing tracks it,
  so nothing will ever fix it. Container-side default for the in-repo copy should be verified.
- 🔜 `belizechain.org` prints live values on public pages, so it is the most exposed surface
  for this class of drift — see §4.1.
- ⚠️ **Seen while sweeping, not yet triaged:** `ui/blue-hole-portal/src/app/page.tsx` displays
  `100.119.97.38:30333/30334` and labels the producer **"Aura / Grandpa"**. The chain is
  **BABE** (with GRANDPA finality), not Aura. This looks like fabricated display data — the
  same class as the honesty sweep is chasing. Verify before trusting any UI status panel.

---

## 5. Standing traps (durable — these do not go stale)

- **Never restate a volatile value in a second document.** It will drift. Link here.
- **Verify before asserting.** Unsourced live claims are how two workstreams ended up
  tracked only in chat.
- **A doc that merely mentions a dead host is invisible to `check-doc-claims.sh`** — it
  checks links, compose/env coverage, Node alignment and image pins, not prose IPs. Grep
  for dead hosts by hand after any host retirement.
- **`docker compose config` aborts on the FIRST missing required var**, so one new
  `${VAR:?}` turns a whole CI job red while validating nothing. Now guarded by CHECK 6.
- **A `permissions:` block REPLACES the token defaults** rather than adding to them — list
  every scope a job uses, including `actions: read` if it uploads SARIF.
- **The 46-odd dismissed code-scanning alerts are verified false positives**, not deferred
  work: a PEP 562 lazy module CodeQL cannot follow (27×), pip's **vendored** copies that
  Trivy mis-attributes to the installed package, and test constructs that *are* the
  assertion. Do not "fix" them by contorting the code.
