# `chain-specs/` — what each file is for

Two specs, two jobs. **They are not interchangeable.**

| File | Purpose | Safe to boot the live chain from? |
|---|---|---|
| `../testnet-spec.json` (repo root, gitignored) | The genesis that **already happened**. Ceiba boots from a byte-identical copy at `/data/chain/testnet-spec.json` | ✅ Yes — it *is* the current chain |
| `genesis-hardened-2026-11-07.json` | A genesis for a **future** chain, with every privileged slot moved off well-known dev keys | ❌ No — it produces a different chain |
| `chain-spec-production.json` | Production/mainnet candidate | Only for a mainnet that has not launched |

## Why editing `testnet-spec.json` is dangerous

Measured 2026-11-07: the repo copy and `/data/chain/testnet-spec.json` on Ceiba
were byte-identical (md5 `31171e1020cc772d47ace036bb6be1fa`).

The genesis patch feeds the genesis hash. Rewriting a field changes that hash
under the same chain id (`belizechain_testnet`), and the node then refuses the
chain data it has been running on. The result is not a hardened chain — it is a
node that will not start.

That is why `scripts/harden-genesis-spec.js` **refuses** `--out` equal to
`--spec`, and why the hardened output lives here rather than replacing the
operational file.

## `genesis-hardened-2026-11-07.json`

Produced by:

```bash
NODE_PATH=../ui/node_modules node scripts/harden-genesis-spec.js \
  --sudo      0x32967ce9efd49154ebea4afd4b599563a7dae718a2da237f10f2abed2f3fb017 \
  --validator 0x6e451283e1e4dc9ab5bc8b624c7f743c11b626cbc59dca2cbdf7723332cc981e \
  --grandpa   0xfdf2056326eb4a755279d81f2449c13441e9d207f7c86c70517fafa27a7daced \
  --out chain-specs/genesis-hardened-2026-11-07.json
```

What it changes relative to the operational spec:

| Slot | Operational spec | Hardened spec |
|---|---|---|
| `sudo.key` | `//Alice` (`5GrwvaEF…`) | `5DD2xP514R5ZprsrPYaDaSYaCh1Pm2DBh1PdeX2sz2oYQ3yW` |
| `session.keys` | 1 authority, `//Alice` BABE + `//Alice` grandpa | 1 authority, `5EZHeG8L…` BABE + `5HofsFi7j…` grandpa |
| `identity.*Issuers` (×3) | `//Alice` | the root account |
| `governance.councilMembers` | `//Alice`, `//Bob`, `//Charlie` | 1 seat, root account |
| `balances` | 1,000,000 DALLA to `//Alice` | moved to the root account |

Every `//Alice`/`//Bob`/`//Charlie`/`//Dave`/`//Eve`/`//Ferdie` seed (plus `//stash`
variants) is published in Substrate's documentation. Any of them in a privileged
slot is a chain that anyone can administer.

## Verify before trusting either spec

```bash
# Fails (exit 1) and names every offending slot while a dev key remains.
NODE_PATH=../ui/node_modules node scripts/harden-genesis-spec.js \
  --check-only --spec chain-specs/genesis-hardened-2026-11-07.json

# Confirm at the raw genesis-storage layer, not just in the patch JSON.
./belizechain-node build-spec --chain chain-specs/genesis-hardened-2026-11-07.json --raw > /tmp/raw.json
```

The filename carries a date because the keys will rotate again. A spec older
than the current key rotation should be treated as stale, not reused.
