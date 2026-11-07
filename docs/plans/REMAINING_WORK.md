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
| 1.5 | **Faucet is unusable by a brand-new account** (see §3.4) | `claim()` is a plain `#[ink(message)]`, so the caller pays gas. An account with zero DALLA cannot pay it — which is exactly the user a faucet exists for. Fixing it properly means a decision (meta-tx relayer vs. seeding accounts vs. documenting a two-step). | Testnet onboarding needs a manual step forever. |

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
| Uncommitted: `AUDIT-OF-RECORD.md` §10 deployment record, `tests/test_dockerignore.py`, `pyproject.toml` isort config, one import move in `integration/oracle_pipeline.py` | **Not committed** — the work is done, the record of it is not |
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
`Delegated alert dismissal is enabled… when the create_request parameter is false`, and the
`create_request=true` path answers `An alert dismissal request already exists` and asks for
the `admin:repo_hook` scope — while the `gh` token holds only
`gist, read:org, repo, workflow`. Clearing these 25 needs either a repo admin approving
requests in the Security tab, or `gh auth refresh` with the extra scope.

That is the whole of the remaining kinich work package: the code is correct, the evidence
exists, and only a permission stands between the two.

### 3.4 `gem` — one real bug, plus CI that cannot run

| Item | State |
|---|---|
| **Faucet cannot serve a fresh account** | `faucet/lib.rs::claim` is a non-payable `#[ink(message)]` that transfers `drip_amount` to the caller. The caller still pays **gas**, and an account holding 0 DALLA cannot. Confirmed by reading the message; the drip never reaches the people it targets. Decision required — see 1.5 |
| GitHub Actions | Unrecoverable on billing (minutes exhausted; owner will not buy more). **Local verification is the substitute and it works**: 10 contracts, 213 tests, ~13s total, measured 2026-10-15. Use `-j 2` *before* `--` |
| e2e tests | Type-checked but **never executed** — `substrate-contracts-node` is not installed. A real gap, not a pass |

### 3.5 `infra` — decisions, then cleanup

| Item | State |
|---|---|
| `relayer` service | Declared in `docker-compose.ceiba.yml`, **never deployed** — no image, no container. `RELAYER_ADAPTER=null` would confirm nothing. Remove or build |
| `docker-compose.validators.yml` | 4-node swarm stopped 2026-10-24 after a fork; volumes preserved for forensics. Decide: resume or delete |
| `.bak` files on Ceiba | Deliberate hold until a stable day; then delete |
| Third-party backup target | Undecided |

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

---

## 5. Sequence that minimises rework

1. **1.2** (re-genesis spec) before any re-genesis, ever — it is the only item
   that silently undoes today's work.
2. **2.3 → 2.1 → 2.2** — seat oracles, then register `.bz` names, then seed DALLA
   pools. Names and pools are both governance acts and both need operators for
   their price/attestation gates to mean anything.
3. **3.1** — commit nawal's record; it costs minutes and the work is already done.
4. **2.4** — grow the validator set before anything that assumes >1 authority.
5. **1.1** — DEX bootstrap, once funded deliberately.
6. **3.3 → 3.4 → 3.5 → 3.6** — kinich, gem, infra, then UI last.
