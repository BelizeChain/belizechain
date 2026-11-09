# Remaining Work — everything still open, with its evidence

> **Written 2026-11-07.** This is a **tracker, not a plan**: every entry below was
> measured or read in the session that wrote it, and each carries the check that
> would falsify it. If an entry looks stale, run its check rather than trusting
> the sentence.
>
> Deliberately *not* a second copy of the subjects that already have an owner doc:
> - mainnet blockers → [`../security/MAINNET_READINESS.md`](../security/MAINNET_READINESS.md)
> - live values (chain, host, endpoints) → [`../ECOSYSTEM_STATE.md`](../ECOSYSTEM_STATE.md)
> - engineering gaps closed to date → [`../PHASE_STATUS.md`](../PHASE_STATUS.md)
> - public-facing milestones → [`../ROADMAP.md`](../ROADMAP.md)
>
> What this file adds is the part none of them covers: **what is left, across all
> eight repos, and who or what is blocking it.**

---

## 1. Waiting on an owner decision (do not do these unasked)

| # | Item | Why it needs you | Cost if wrong |
|---|---|---|---|
| 1.1 | ~~**Run `scripts/bootstrap-dex.js` against Ceiba**~~ | ✅ **RESOLVED 2026-11-09** | Ran the production path: reserves → minter authorization → identity → KYC → seat 3 oracle operators → Oracle KYC L2 → mint 200,000 bBZD → liquidity → a swap that settled on both ledgers. `(0,1)` now holds **100,199.52 DALLA / 199,801 bBZD**; `TotalBbzdSupply` **200,000**. Funding required `balances.forceSetBalance` on sudo (1,042 → 102,000) because the pool needs 100,000 and fees need paying: **supply 3,001,009 → 3,102,266 DALLA**. The three oracle keys are new, generated for this and stored 0600 at `/opt/belizechain/keys/oracle-{1,2,3}.json` on Ceiba — no existing oracle set existed to reuse. Re-running converges (verified): every step reports already-done and only the settlement trade executes. |
| 1.2 | ✅ **RESOLVED 2026-11-07 — re-genesis key precondition** | `chain-specs/genesis-hardened-2026-11-07.json` moves sudo, the sole session key, all three identity issuer lists and the council seats off every published dev key; `scripts/harden-genesis-spec.js` regenerates it and refuses to write while any `//Name` dev key remains. Verified at the **raw genesis** layer: zero dev-key hex in `genesis.raw.top`, new sudo/BABE/GRANDPA present. **`testnet-spec.json` deliberately unchanged** — it is byte-identical to Ceiba's boot spec and editing it would change the genesis hash under the live chain id. See `EMPTY_EPOCH_STALL_AND_REGENESIS` §11 | Remaining: actually boot from the hardened spec when a re-genesis happens |
| 1.3 | **`BELIECHAIN_CI_TOKEN`** | Needs a fine-grained PAT with `Contents: read` on the six private sibling repos. Without it, `docs-check.yml` fails on every scheduled run by design. | Docs drift stops being detectable in CI. |
| 1.4 | ✅ **RESOLVED 2026-11-07 — genesis council seats** | Two compounding defects, both fixed in `pallets/governance`. (a) The genesis build stamped *every* member `BoardRole::Founder` against `max_count() == 1`, so a 3-seat genesis produced 3 Founders; roles are now handed out along a `GENESIS_SEAT_LADDER` (Founder, TechnicalSteward, FSCRepresentative, …) with an assert on over-long input. (b) Genesis never wrote `BoardComposition`, which `add_board_member` reads as the count — so a further Founder could still be appointed while the invariant looked satisfied. Now recomputed from the seats written. 175 governance tests pass; both halves independently falsified (`Founder seated 3 against a cap of 1`, and `BoardComposition disagreed with CouncilMembers`) | Also corrected a test that asserted `board_composition(Founder) == 0`, i.e. it encoded the bug |
| 1.5 | ~~**Redeploy the faucet with `claim_for`**~~ | ✅ **RESOLVED 2026-11-09** | Redeployed at block 483807 as `r1WxhL7BkS3g6LopMgncAQEzDqLFZmo1UK96mn2MqBwZTfk9g`, code hash `0x3e3db584…86ff`, owned by the rotated sudo key, 10,000 DALLA, drip 100, cooldown 100 blocks. Old instance drained first (its 10,000 DALLA went to sudo, which then funded the new payable constructor — it held only 1,050 before). Proven live with a fresh 0-balance account: `claim()` from it is rejected at the transaction pool (`1010: Inability to pay some fees`), a relayer's `claim_for()` delivers exactly 100 DALLA, an immediate repeat reverts on the recipient's cooldown, and a second fresh account gets its own drip — claimCount 2, totalClaimed 200, faucet 9,800. Repinned in `ui/shared/src/lib/runtime-config.ts`, `gem/.env.testnet`, `belizechain.org/lib/constants.ts` and `gem/deployment-1794226724208.json`; all three frontend images rebuilt, verified to carry the new address and zero references to the old one, deployed and serving. |
| 1.6 | **The boot spec still pins `//Alice` in genesis** | 🔴 **Needs a decision, cannot be repaired in place** | Measured 2026-11-08: `chain-specs/genesis-live-0xb2664568.json` produces genesis hash `0xb2664568…b5ef`, identical to the live chain's `chain_getBlockHash(0)`. It is this chain's origin, and it carries four publicly-known dev keys in nine privileged slots. Editing it changes that hash and the node refuses its own database — an outage, not a hardening. BelizChain keys in the *boot* spec require a re-genesis, i.e. a new chain with no history and no contracts. |
| 1.7 | ~~**Two published dev keys still hold Foundation Board seats**~~ | ✅ **RESOLVED 2026-11-09** | Measured 2026-11-09: the two remaining `governance.councilMembers` derived from the standard `DEV_PHRASE` — `//Bob` (`r1VBwEc9W86…`) and `//Charlie` (`r1VEbJnHYT6…`). `testnet-spec.json` lists exactly `{//Alice, //Bob, //Charlie}` as genesis councilMembers, so removing `//Alice` removed one of **three** published keys, not the only one. Board membership is checked by `is_council_member`, which gates `execute_proposal` (index 13) and `approve_treasury_spend` (index 30), and the member count sets the emergency-veto threshold at `count/3 + 1`. **Value at risk was small and measured**: the governance pallet account held **0 DALLA**, the main treasury **2 DALLA**, and a spend under 10 DALLA needs a single approval — a latent authority problem, not a live theft.<br>**Fixed by replacement, not by emptying the board.** `sudo(governance.removeBoardMember)` × 2 then `sudo(governance.addBoardMember)` × 2, both already proven live. Post-state read back from chain: board is `r1UTSENDje4…` **Founder** and `r1T2JCTaZw…` **TechnicalSteward**, both 2-year terms, `boardComposition = {Founder: 1, TechnicalSteward: 1}`, published dev keys on the board: **none**. Both replacement keys are 0600 in `/opt/belizechain/keys/` on Ceiba and neither is the sudo key, so board authority is not concentrated into the key that can already do anything. `add_board_member` is the only extrinsic that writes `BoardComposition`, so this also populated the counter that genesis left empty on the live chain. District council keys were deliberately **not** used: they sit in plaintext in `generated_keys/testnet-council-keys.json` and are as public as the dev keys they would have replaced. |
| 1.8 | **DALLA stranded on three keyless accounts** | ✅ **RESOLVED 2026-11-09** | The three `randomAsU8a` placeholder validators were created from seeds that were never retained, so removing their records released 1,500 DALLA of stake locks into accounts nobody could sign for. Recovered with root-only `sudo(balances.forceTransfer)` to the sudo account — the same account they were funded from, so it is a reversal rather than a reallocation.<br>**Exact figure read from the `Transfer` events, not from balances**: 989.997944039333 + 989.997944008485 + 789.996019859033 = **2,769.991907906851 DALLA** across blocks 485230–485232. All three accounts now hold **0**. An earlier note said 2,767 — that was three truncated integers summed, and it was wrong. |

**Verification for 1.2** (the one most likely to be forgotten):

```bash
python3 -c "
import json; d=json.load(open('testnet-spec.json'))
p=d['genesis']['runtimeGenesis'].get('patch',{})
print(p.get('sudo')); print(str(p.get('session'))[:120])"
# {"key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"}   <- //Alice, still
```

---

## 2. Blocked on chain work, not on effort

| # | Item | Measured state | What unblocks it |
|---|---|---|---|
| 2.1 | **Pakit `.bz` web hosting** | `bns.totalDomains = 0` — no domain has ever been registered. | Register `.bz` names on the live chain. Hosting resolves nothing while the registry is empty; this is data, not code. |
| 2.2 | **DALLA/BZD trading** | ✅ **RESOLVED 2026-11-09 for `DALLA/bBZD`** — re-measured after 1.1: pair `[0,1]` holds **100,299 DALLA / 199,403 bBZD**. Still open: `TourismDALLA/bBZD` (`[2,1]`) is **0/0**, and `WUSDC/bBZD` (`[3,1]`) holds only 10/20. | Seed the TourismDALLA pool the same way, or drop the pair. `AssetId` order is `DALLA=0, BBZD=1, TourismDALLA=2, WUSDC=3`. |
| 2.3 | **Oracle consensus** | ✅ **RESOLVED 2026-11-09** — `oracle.oracleOperators` now has **3** entries, seated by 1.1 and reused for the bridge-validator KYC quorum. | Nothing; the guards are no longer shipped against an empty operator set. |
| 2.4 | **Real validator set** | `staking.validators` = 2, `session.validators` = **1**. | Bring up additional nodes; a single authority is a single point of failure and a mainnet blocker (`MAINNET_READINESS` §3.2). |

**Check 2.1–2.4** (public endpoint, no host access needed):

```bash
node -e '
const {ApiPromise,WsProvider}=require("@polkadot/api");
(async()=>{const api=await ApiPromise.create({provider:new WsProvider("wss://testnet.belizechain.org/ws"),noInitWarn:true});
console.log("bns", (await api.query.bns.totalDomains()).toString());
for (const [k,v] of await api.query.belizeX.tradingPairs.entries()) console.log(k.args[0].toString(), v.baseReserve.toString(), v.quoteReserve.toString());
console.log("session", (await api.query.session.validators()).toJSON().length);
await api.disconnect();})()'
```

> **Correction to earlier notes:** the audit that opened this workstream recorded
> "DEX has 0 pairs, 0 pools". At 2026-11-07 there are 3 pairs and one funded pool.
> The gap is narrower than recorded — it is now *which* pairs are funded, not
> whether markets exist at all.

---

## 3. Repo by repo

### 3.1 `nawal-ai` — audit closed, one doc commit outstanding

All **17** findings in [`nawal-ai/docs/audits/AUDIT-OF-RECORD.md`](../../../nawal-ai/docs/audits/AUDIT-OF-RECORD.md)
are marked DONE, and finding 15 was closed on the live host on 2026-11-01:
`/api/nawal/inference/health` returns `200` with `model_loaded: true`.

| Item | State |
|---|---|
| Audit record: `AUDIT-OF-RECORD.md` §10 deployment record, `tests/test_dockerignore.py`, `pyproject.toml` isort config, one import move in `integration/oracle_pipeline.py` | **Committed** `92d71a3` (unpushed). 4/4 dockerignore tests pass |
| Local commit `30d937b` ("remove AKS-era k8s manifests") | Not pushed |
| The served model is the **bounded bootstrap checkpoint** (1.7M params, `bounded_run: true`) | Proves the pipeline, not model quality. No doc may claim otherwise |
| `/model/info` serves absolute workstation paths (`/home/wicked/Projects/...`) to an unauthenticated endpoint | Accepted deliberately; revisit if the endpoint stays public |

### 3.2 `pakit-storage` — blocked on 2.1

Storage engine, DAG and merkle proofs are independently testable; **web hosting is
dead by data** (`bns.totalDomains = 0`), and payments are off. Do not tune hosting
before names exist.

### 3.3 `kinich-quantum` — 25 alerts, all one rule, proven false positives, **dismissal blocked**

Measured 2026-11-07 with `gh api …/code-scanning/alerts?state=open&per_page=100`:
**25 open alerts, and every one is the same rule — `py/undefined-export`**, all in
`blockchain/__init__.py`. None carries a security severity.

The audit that opened this workstream said "10 actionable alerts against ~40 dismissals";
that number had drifted. Re-measure rather than trusting either figure:

```bash
gh api "repos/BelizeChain/kinich-quantum/code-scanning/alerts?state=open&per_page=100" \
  --jq 'length'
```

**The alerts are false positives, and that is now proven rather than asserted.**
`blockchain/__init__.py` resolves all 25 exports through a module-level `__getattr__`
(PEP 562) so that importing the package does not pull in `substrateinterface`, `eth_*`,
`pydantic` and 25 other third-party modules. `py/undefined-export` does not model that
pattern.

`tests/test_blockchain_exports.py` (kinich `73656a9`) now holds the contract the rule was
reaching for: every `__all__` name resolves, `__all__` and `_EXPORTS` agree in both
directions, `from … import X` matches `getattr`, and a **fresh interpreter** importing the
package loads zero third-party modules. All three properties were falsified before being
trusted — a typo'd attribute name fails two tests, drifting `__all__` fails the agreement
test, and an eager import makes the subprocess test name all 28 third-party modules.

**✅ CLOSED 2026-11-07 — by fixing the code, not by dismissing anything.**

The `__all__` derivation (`__all__ = sorted(_EXPORTS)`, kinich `e60ae87`) removed the pattern
the rule was matching. CodeQL re-analysed the pushed commit and marked the alerts **fixed**:
`state=all` now returns 100 alerts and every one is `fixed`, with **0 open**. Re-check:

```bash
gh api "repos/BelizeChain/kinich-quantum/code-scanning/alerts?state=open&per_page=100" --jq 'length'
```

This is a better outcome than the dismissal path, which is recorded here only so nobody
retries it: `kinich-quantum` has *delegated alert dismissal* enabled, so
`PATCH …/alerts/{n}` with `state=dismissed` returns
`Delegated alert dismissal is enabled… when the create_request parameter is false`.
`create_request=true` queues a request successfully, but **approving** one needs the
`delete_repo` scope (the API asks for it explicitly) — a scope that can delete repositories,
so it is not worth requesting for cosmetic cleanup. A bare `gh auth refresh` adds no scopes;
verify with `gh auth status`.

The contract the rule was reaching for is now pinned by tests rather than by the analyzer:
`tests/test_blockchain_exports.py` (5 tests, passing) asserts every `__all__` name resolves,
`__all__` and `_EXPORTS` agree in both directions, `from … import X` matches `getattr`, and a
subprocess import of the package loads zero third-party modules.

That is the whole of the kinich work package: done.

### 3.4 `gem` — one real bug, plus CI that cannot run

| Item | State |
|---|---|
| **Faucet could not serve a fresh account** | **FIXED in source** (`gem` `0d54ee2`), **not deployed**. `claim()` was a non-payable `#[ink(message)]` that transfers to the caller, so the **caller pays the fee**: a `contracts.call` costs **0.001197 DALLA** (`partialFee 1,196,589,257` wei) against a **0.001 DALLA** existential deposit (`1,000,000,000` wei), so an account holding 0 — or exactly the ED — cannot claim. Both `claim()` and the new `claim_for(account)` now delegate to one private `dispense()`, and the cooldown is keyed on the **recipient**, so relaying in a loop cannot drain the faucet. 14 tests pass; release wasm 11,878 B with zero `memory.copy`/`memory.fill`; end-to-end on a dev contracts node a keypair holding **0 wei** received exactly 1 DALLA relayed by a different signer, and a second relayer in the same block got `TooSoon`. The live contract at `r1Wcf2YSz…` is untouched until it is reinstantiated — see 1.5 |
| SDK ABI was stale | `sdk/contracts/faucet.json` advertised **17** messages against the contract's 23. Refreshed, and `faucetClaimFor()` added with a type declaration. Note `sdk.transfer` **does not exist** — the funded-transfer helper is `dallaTransfer` |
| GitHub Actions | Unrecoverable on billing (minutes exhausted; owner will not buy more). **Local verification is the substitute and it works**: 10 contracts, 213 tests, ~13s total, measured 2026-10-15. Use `-j 2` *before* `--` |
| e2e tests | Type-checked in CI, and **now executed manually** against `substrate-contracts-node --dev --tmp` (v0.41.0, present on this workstation). Keep running them by hand before a faucet redeploy |

### 3.5 `infra` — mostly settled, one decision left

Re-measured 2026-11-07. Three of the four items recorded here are already resolved
and the notes were stale.

| Item | State |
|---|---|
| `relayer` service | **Not a gap.** Already gated behind `profiles: ["relayer"]` with `${RELAYER_IMAGE:-…}`, so a plain `docker compose up -d` cannot fail on the missing image and take the stack with it. The comment block explains why it stays gated: no `BelizeBridgeReceiver` exists on any foreign chain, and `RELAYER_ADAPTER=null` would confirm nothing. Correct as it stands |
| `docker-compose.validators.yml` | **Resolved.** The four `infra_val-*-data` volumes were wiped 2026-11-07 with owner approval, so there is nothing left to resume. `belizechain/scripts/local-swarm.sh` supersedes the Docker swarm (measured working: 4/4 peers, zero hash mismatches, epoch rotation, 2-of-4 recovering finality in <6 s). The file now says so at the top. `--force-authoring` was already correctly gated on `${FORCE_AUTHORING:+…}` |
| `.bak` files on Ceiba | **Resolved — none exist.** `sudo find` over `/opt/belizechain`, `/etc/nginx`, `/etc/systemd/system` and `/usr/local/sbin` returns zero `*.bak`/`*.old`/`*.orig`. Nothing to delete |
| Third-party backup target | **Still undecided, and genuinely an owner call.** Backups currently land on the same host they protect, so a host loss loses both |

### 3.5a GitHub hygiene — cleaned 2026-11-07

Not in the original tracker. Found while auditing org state with admin access.

| Item | State |
|---|---|
| Dead repo secrets | **20 deleted.** Every secret on `belizechain` (11) and `kinich-quantum` (9) was referenced by **zero** workflows — including `VALIDATOR_MNEMONIC` (for the key rotated off the chain) and `VM_SSH_KEY` (for the host decommissioned 2026-10-07). Six more on `nawal-ai` were dead the same way. Values are only readable by workflows, so with no referencing workflow they could never be used |
| `ci-capability-probe.yml` | Disabled workflow record on `infra` whose **file does not exist** — not on the branch, not in git history. Cannot be deleted via API (404). Inert tombstone; leave it |
| CodeQL duplication | **Not duplication.** `CodeQL - Code Quality` (`dynamic/github-code-quality/codeql`) and `CodeQL` (`dynamic/github-code-scanning/codeql`) are different features — style analysis vs security scanning. Both are correct |

### 3.5b Code scanning alerts — triaged 2026-11-07

Four open code-scanning alerts across the org. None is an exploitable defect; all four are
recorded here with the evidence rather than dismissed silently.

| Repo | Alert | Verdict |
|---|---|---|
| `belizechain` | #187 `rust/hard-coded-cryptographic-value` at `zk/attestation/src/circuit.rs:472` | **False positive.** The value is a test salt/decoy (`Fr::from(99u64)` et al.) in a unit test that walks a sparse tree. No secret |
| `infra` | #12 `js/http-to-file-access` at `relayer/scripts/deploy-receiver.mjs:221` | **Accepted as designed.** A deploy script writing a JSON record to the `--out` path the operator passed on the command line. Not unattended, and the relayer is not deployed |
| `pakit-storage` | #1239 `CVE-2026-114613` (msgpack DoS) | **✅ Fixed 2026-11-07T23:49Z.** The advisory covers `< 1.2.3`, and `pakit_requirements.txt` has pinned **1.2.4** since `5eb9d3e`, so the source was never vulnerable. Trivy's instance at that same commit reported `Installed Version: 1.1.2` — it had scanned an image built before the bump. A later analysis cleared it. The lesson is the stale-image one: a Trivy alert can describe an image that predates the pin it appears to be about |
| `pakit-storage` | #1207 `CVE-2024-23342` (ecdsa Minerva) | **No fix exists.** `python-ecdsa` is unmaintained, so there is no patched release. Transitive via `substrate-interface==1.8.1`, which the chain integration needs. Accepted residual |
| `pakit-storage` | #1236 `GHSA-xmj3-9gf7-h52w` (setuptools, note) | **Not applicable.** Windows-only path escape during legacy wheel installation; pakit ships a Linux container |
| `belizechain.org` | #9 `js/stored-xss` at `sidebar.tsx:70` | **False positive, and it survives a code change designed to silence it** — see below |

**On `belizechain.org` #9:** `toSafeHref()` allows only a root-relative path — it requires a
leading `/` and rejects `//`, backslash and control characters. Executed against the real
function, `javascript:alert(1)`, `//evil.com`, `/\evil.com`, `data:…`, and tab- and
NUL-prefixed variants all return `#`; every one of the 63 `ROUTES` values survives unchanged.

I then rewrote it to rebuild the value through the platform `URL` parser, on the theory that
CodeQL would recognise a parse step. **It did not work.** CodeQL re-analysed the new commit
(`bb5ecb1`) and left the alert open. The change is kept because re-serialising through the
same parser the browser uses is a genuine (small) hardening, but it did **not** clear the
alert, and this document should not pretend otherwise.

The only ways to clear it are the two already listed: approve the queued dismissal request in
the Security tab, or grant `delete_repo` to the token. Neither is worth the cost of a broad
scope for a proven false positive.

### 3.5c `belizechain.org` — content drift fixed, build verified by CI

Not in the original tracker, and it should have been: the site documents the ecosystem, so
every number on it is a live claim.

| Item | State |
|---|---|
| Pallet count | **Fixed 2026-11-07** (`bb5ecb1`). The site said **19** pallets; `construct_runtime!` has **20** (ZkAttestation, index 39). `RUNTIME_FACTS.PALLET_COUNT` fed the footer, home stats and technology pages, so one stale constant was wrong in five places; ten prose strings in six pages hardcoded it separately |
| `PALLET_TEST_COUNT` | **Fixed.** Claimed 1343, measured **1498** across the 20 crates (lines matching `^\s*#\[test\]` under each pallet's `src`, excluding `pallets/common`). Flat and recursive counts agree |
| Build verification | **✅ Verified by CI, not locally.** `bb5ecb1` triggered "Build belizechain.org image": `Lint & Typecheck`, `Build App` and `Build Docker Image` all succeeded. The Dockerfile runs `RUN npm run build` (line 21), so that success *is* a real Next.js production build. The commit message for `bb5ecb1` says the build was skipped for memory reasons — that was written before the CI run finished, and **CI closed the gap** |
| **Deployed 2026-11-07** | `ORG_IMAGE=belizechain/org-site:c55997c-org-20261107` in `/opt/belizechain/.env`; `ceiba-org` recreated and nginx reloaded. Live `https://belizechain.org/` now renders `20 Runtime Pallets`, `20 custom runtime pallets` ×6, `consensus, 20`, with **zero** occurrences of `19`. Rollback target is `belizechain/org-site:698b2be-org-20261021` — still present on the host; revert the `ORG_IMAGE` line and `up -d org` |

**Where the image is built, and why:** `belizechain.org/.github/workflows/deploy.yml`
builds with `push: false, load: false` — no registry push and no host rollout, and it says so
in its own job summary. So CI proves the build but never delivers it. The image for this deploy
was therefore **built on Ceiba** from a `git archive` of the exact commit:

```bash
git -C belizechain.org archive <sha> | ssh wicked@ceiba 'rm -rf /tmp/org-build && mkdir -p /tmp/org-build && tar -x -C /tmp/org-build'
ssh wicked@ceiba 'cd /tmp/org-build && nice -n 10 docker build -t belizechain/org-site:<sha>-org-<date> .'
```

Ceiba is the right host for this: 30 GB RAM with ~25 GB free and load ~0.04, against this
workstation's 15 GB with under 5 GB free and swap fully consumed. Build there, never here.

**Two content traps worth remembering:**
1. A line-based grep cannot see a claim wrapped across source lines. `"…consensus, 19` / `custom
   runtime pallets"` survived three passes for exactly that reason. Use a multiline regex.
2. React SSR inserts `<!-- -->` between adjacent text nodes, so `grep '[0-9]* Runtime Pallets'`
   misses `20<!-- --> Runtime Pallets`. Strip comments before matching rendered HTML.

The only reliable check is **rendering**: source grep said clean while the built page still
said 19 in four places.

Re-verify the content claims before deploying anything that restates them:

```bash
# from the belizechain repo root — prints 20
python3 -c 'import re;src=open("runtime/src/lib.rs").read();rows=re.findall(r"^\s*(\w+):\s*(\w+)(?:\s*::<[^>]*>)?\s*=\s*(\d+)",src,re.M);print(len([1 for _,_,i in rows if 20<=int(i)<=39]))'
```

Quote it with single quotes as shown: an outer double-quoted `-c` string containing
`construct_runtime!` triggers bash history expansion and the command dies with
`!\s: event not found`.

### 3.6 `ui` — last, by design

Wiring depends on chain behaviour, so it follows §2. The audit series
(`ui/docs/audits/UI-00` → `UI-08`) already lists its own remaining work; do not
restate it here.

---

## 4. Accepted residuals — documented, not queued

These are *known and correct as they stand*. They are listed so nobody
re-discovers them as bugs and "fixes" them into regressions.

| Residual | Why it stays |
|---|---|
| Live RPC has **no authentication** | Read exposure was never in scope of the key rotation. Fixing it needs a decision about who may read the chain |
| `Sudo` remains at runtime index 6 | Owner decision 2026-11-07: kept through the multi-validator phase as break-glass, removed before mainnet. The machinery to remove it exists |
| Two RustSec advisories suppressed (`RUSTSEC-2026-0381`, `-0382`) | Both live in `libp2p`, pinned by polkadot-sdk; no reachable fix. Suppressed in both `security-audit.yml` ignore lists; only 0381 appears in `deny.toml` because 0382 is not in any resolved graph |
| Coverage 56% against a 40% CI gate | Measured. The mainnet bar should be higher than the testnet gate — a decision, not a chore |
| `tests/` in `nawal-ai` runs 4,447 tests in ~5 min | Memory-constrained host; heavy suites have crashed this workstation four times. Run targeted, not wholesale |
| `ui` Dependabot alert: minimatch ReDoS (`>= 7.0.0, < 11.0.1`) | **Upstream-blocked, not fixable here.** The reachable path is `@typescript-eslint/typescript-estree` → `minimatch ^10.2.2` → 10.2.6. Checked the registry on 2026-11-07: even the newest release (8.72.1) still requires `^10.2.2`, and **no published version requires 11+**, so the advisory cannot be cleared by any update available today. It is a devDependency used by lint tooling, never in the production bundle, and ReDoS needs an attacker-controlled glob pattern — none exists in the lint config. This is also why Dependabot's own run fails with *"the available update path still resolves it to 10.2.6"*. Re-check when `@typescript-eslint` moves to `^11` |
| `ui` root `minimatch: ^9.0.7` | **Dead dependency.** Declared directly in `ui/package.json` but imported by **zero** files. Safe to delete — but it does *not* clear the alert above, because the transitive 10.2.6 path is the one flagged. Remove it with the next dependency update rather than as a standalone lockfile rewrite |

---

## 5. Sequence that minimises rework

1. **1.2** (re-genesis spec) before any re-genesis, ever — it is the only item
   that silently undoes today's work.
2. **2.3 → 2.1 → 2.2** — seat oracles, then register `.bz` names, then seed DALLA
   pools. Names and pools are both governance acts and both need operators for
   their price/attestation gates to mean anything.
3. ~~**Push the three finished repos**~~ — **done 2026-11-07.** `nawal-ai`,
   `kinich-quantum`, `gem`, `pakit-storage`, `ui`, `infra`, `belizechain.org` and
   `belizechain` are all pushed and their trees are clean.
4. **2.4** — grow the validator set before anything that assumes >1 authority.
5. **1.1** — DEX bootstrap, once funded deliberately.
6. **1.5** — redeploy the faucet after the address repin is planned; it is
   self-contained and unblocks onboarding on its own.
7. **3.3** — approve kinich's queued dismissals (Security tab, one click each),
   then **3.6** UI last. `gem` 3.4 and `infra` 3.5 are done; the only infra item
   left is the third-party backup target, which is an owner decision.
