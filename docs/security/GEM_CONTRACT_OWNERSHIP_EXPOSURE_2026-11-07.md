# Live exposure: every deployed GEM contract is owned by the compromised `//Alice` dev key

**Found:** 2026-11-07, while preparing the faucet redeploy (tracker item 1.5).
**Status:** OPEN — not remediated. Requires a decision, because remediation means
signing with the compromised key.
**Severity:** High. Administrative control of six live contracts plus a drainable
treasury, held by an account whose mnemonic is published in Substrate's docs.

---

## What is wrong

`pallet-contracts` stores each contract's owner in **contract storage**, not in
chain storage. Blocker #13 rotated `Sudo`, the BABE authority and the three
identity issuer lists — all of which are *pallet* storage and could be changed by
extrinsic. Contract ownership cannot be changed from outside the contract: it is a
field inside the instance, writable only by the contract's own messages, and those
messages require a signature from the current owner.

The contracts were deployed on 2026-10-08 by `//Alice` — the deployment manifest
records `"deployer": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"` — and
nothing has transferred ownership since. So the rotation left all six untouched.

## Evidence (read from `wss://testnet.belizechain.org/ws`, 2026-11-07)

`//Alice` = `r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur`
(derived: `encodeAddress(0xd43593c7…da27d, 1981)`).

| Contract | Address | Owner field | Value |
|---|---|---|---|
| DALLA token | `r1SqoNCdgah6zWqzPJSNmKcknW2KaCvruSz6Mg8jBTd6jqtyL` | `owner` | **compromised** |
| BeliNFT | `r1UxXWdW89AQ9ewvdRUthaEPR4vvWrgxn3VeHKVMB2rYsUUvv` | `contractOwner` | **compromised** |
| Simple DAO | `r1ScgRVx963TurkyzkTNLQbmhBfzhaj5x8YCHgM7yLFkmx8eh` | `admin` | **compromised** |
| **Faucet** | `r1Wcf2YSzSeziDHG82ku9zRbaLNqfQG3A3c6mbTsnJPGxKqCG` | `owner` | **compromised** + holds **10,000 DALLA** |
| PSP37 multi-token | `r1Vc9kffhRY2K5gTFdwMagLbGGSzwfhoH71EkZpTMpR9t6Dei` | `owner` | **compromised** |
| BelizeX factory | `r1WhHW5jeHRRn2H3JFnLi9ek8MKQi1rtxamcSASKXjkDs1yTw` | `admin` **and** `feeToSetter` | **compromised** |
| BelizeX router | `r1SwipZJQbrTvrUutUsUv7ZhLvp3Cb2dwpFW6gSKUks1poUPm` | `factory` → factory above | inherits |

Re-verify with:

```bash
cd gem && NODE_PATH=../ui/node_modules node -e '
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { ContractPromise } = require("@polkadot/api-contract");
(async () => {
  const api = await ApiPromise.create({ provider: new WsProvider("wss://testnet.belizechain.org/ws"), noInitWarn: true });
  const GAS = api.registry.createType("Weight", { refTime: 5e10, proofSize: 3e6 });
  const c = new ContractPromise(api, require("./sdk/contracts/faucet.json"), "r1Wcf2YSzSeziDHG82ku9zRbaLNqfQG3A3c6mbTsnJPGxKqCG");
  const r = await c.query.owner("r1T7BYVAQEF6GVZJFuvCqaHQt7tfdw8zdHyaNwjeHYmbdkYaP", { gasLimit: GAS, storageDepositLimit: null });
  console.log(r.output.toHuman());
  await api.disconnect();
})();'
```

## What the holder of that key can do right now

Every one of these is reachable by anyone who knows the seed, which is everyone:

| Action | Contract | Consequence |
|---|---|---|
| `emergency_withdraw()` | faucet | **Take all 10,000 DALLA.** No timelock, no veto, immediate. `ensure_owner()` → `env().transfer(self.owner, balance)` |
| `grant_role(me, MINTER_ROLE)` then `mint(...)` | DALLA | The constructor grants **both** `ADMIN_ROLE` and `MINTER_ROLE` to the deployer, so this needs no escalation. `MAX_SUPPLY` is `100_000_000_000_000_000_000` = **100,000,000 DALLA**, against a 1,000,000 DALLA initial supply — a **100× inflation ceiling** |
| `pause()` | DALLA | Halt transfers for every holder |
| `propose_code_hash_upgrade()` → wait 7,200 blocks → `execute_code_hash_upgrade()` | all seven | Replace the contract's code outright. This is the worst one: a malicious DALLA implementation ignores `max_supply` entirely |
| `set_fee_to()` / `set_fee_to_setter()` | BelizeX factory | Redirect all swap fees; seize fee-setter control permanently |
| `set_admin()` / `accept_admin()` | BelizeX factory, DAO | Hand control to another address, or take it |
| `propose_ownership()` | DALLA, BeliNFT, PSP37 | No timelock on ownership, unlike code upgrades |

The code-hash path has a 7,200-block (~12 h at 6 s) delay between propose and
execute, so an attacker who starts now is detectable for half a day. The faucet
drain and the mint path have no delay at all.

## Why this is not covered by the key rotation

| Was rotated | Storage kind | Changed by |
|---|---|---|
| `Sudo.Key`, `Session.Validators`, `Identity.*Issuers` | pallet storage | extrinsic / `sudo` |
| Contract `owner` / `admin` | **inside the contract instance** | only the contract's own messages, signed by the current owner |

The rotation closed the pallet-storage half of blocker #13 and could not touch the
contract half. Nothing in `docs/security/MAINNET_READINESS.md` listed contract
ownership, which is why it survived the audit.

## Remediation (NOT executed — needs approval)

All six use **two-step** ownership: `propose_ownership(new)` from the current
owner, then `accept_ownership()` from the new owner. That is a genuine advantage
here — the second signature cannot be forged, so a proposal cannot be hijacked by
an attacker who only front-runs the first step. It also means the remedy has to be
signed **by the compromised key**, because there is no other way to authorise it.

Suggested sequence per contract:

1. `propose_ownership(<new owner>)` signed by `//Alice`.
2. `accept_ownership()` signed by `<new owner>`.
3. Confirm from the public endpoint that `owner` now reads the new account.

Notes and risks:

- **Choosing the new owner is the decision.** It should be an account whose key is
  not on this workstation and not in `gem/.env.testnet`. The Ceiba sudo account is
  a reasonable short-term holder only if its seed is held properly there.
- **Race:** the proposer and the accepter must move together. A proposal sitting
  unaccepted is visible on-chain, and while it cannot be accepted by a third party,
  it does block a second proposal until the pending one is accepted or cancelled —
  a denial-of-service window. Do them back to back.
- **Drain the faucet first.** Its 10,000 DALLA is the only asset that can walk out
  the door immediately, and it can be moved to the new owner in the same session.
- **Do the DALLA token first.** `mint` needs no timelock, so the inflation ceiling
  is the largest irreversible exposure.

No command in this document has been run. It is written so the work is a
checklist, not a re-investigation.
