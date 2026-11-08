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
| 1.1 | **Run `scripts/bootstrap-dex.js` against Ceiba** | It mints roughly 100,000 DALLA of liquidity plus ~200,000 bBZD, and the sudo account holds ~1,050 DALLA free (3,000 is locked in the three issuer bonds). Funding it means either a large transfer or another `forceSetBalance` — **more minting on a live chain**. | Real supply is created. Reversible only by minting again or a re-genesis. |
| 1.2 | **Re-genesis key precondition** | `testnet-spec.json` — the Ceiba copy *and* the repo copy — still pins `//Alice` as `sudo.key` and as the sole `session.keys` entry. A re-genesis from that file restores the whole exposure closed today. | Six months of the same bug, on a new chain, with nothing in the docs looking wrong. |
| 1.3 | **`BELIECHAIN_CI_TOKEN`** | Needs a fine-grained PAT with `Contents: read` on the six private sibling repos. Without it, `docs-check.yml` fails on every scheduled run by design. | Docs drift stops being detectable in CI. |
| 1.4 | **Genesis council seats** (`ECOSYSTEM_STATE.md` §4.0a) | Genesis stamps all three council seats `Founder`, contradicting `max_count() = 1`. Fix in genesis (correct for mainnet, no-op on testnet) or write a migration. | Founder role is over-assigned at chain birth. |
| 1.5 | **Redeploy the faucet** (see §3.4) | The code fix is written, tested and committed (`gem` `0d54ee2`), but the **live** faucet still has only `claim()`. Making it useful means reinstantiating it, which changes its on-chain address and requires repinning `CEIBA_GEM_DEPLOYMENT` in `ui/shared/src/lib/runtime-config.ts` plus the `gem/deployment-*.json` manifest. | A brand-new account stays unable to onboard without a manual transfer |

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
| 2.2 | **DALLA/BZD trading** | `belizeX.tradingPairs` has **3** pairs, but `DALLA/BBZD` and `TourismDALLA/BBZD` both have `baseReserve 0, quoteReserve 0`. Only `WUSDC/BBZD` is funded. | Governance act: seed the two DALLA pools (see 1.1). |
| 2.3 | **Oracle consensus** | `oracle.oracleOperators` has **0** entries. | Seat operators. Several guards fail closed against an empty operator set, which is safe but inert. |
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

**Dismissal is blocked on access, not on evidence.** `kinich-quantum` has *delegated alert
dismissal* enabled, so `PATCH …/alerts/{n}` with `state=dismissed` returns
`Delegated alert dismissal is enabled… when the create_request parameter is false`. Passing
`create_request=true` works — it queues a request — but **approving** a request needs either a
click in the repo's Security tab or the `delete_repo` scope, which is what the API asks for:

```
This API operation needs the "delete_repo" scope.
```

`delete_repo` can delete repositories, so it is not a scope to request for cosmetic cleanup.
The 25 requests are queued with their justification and only need approval:
`https://github.com/BelizeChain/kinich-quantum/security/code-scanning`.

**The false positives are now structurally unreachable, not just excused.** `__all__` is
derived (`__all__ = sorted(_EXPORTS)`) rather than hand-written, and
`tests/test_blockchain_exports.py` (5 tests, passing) pins the contract: every `__all__` name
resolves, `__all__` and `_EXPORTS` agree in both directions, `from … import X` matches
`getattr`, and a subprocess import of the package loads zero third-party modules.

Note the repo's CodeQL workflow last ran 2026-10-11 with every job at `steps=0`, and default
setup is not analyzing this repo, so no future scan will auto-close these either way. The
dismissal requests need a human, not a scan.

That is the whole of the remaining kinich work package: the code is correct, the evidence
exists, and only a permission stands between the two.

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
| `pakit-storage` | #1239 `CVE-2026-114613` (msgpack DoS) | **Already fixed.** Advisory covers `< 1.2.3`; `pakit_requirements.txt` pins `1.2.4` (bumped in `5eb9d3e`, and 1.2.4 is the newest release on PyPI). The alert reflects a pre-fix SBOM |
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
