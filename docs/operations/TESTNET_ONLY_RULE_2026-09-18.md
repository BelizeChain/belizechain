# BelizeChain Testnet-Only Rule & Chain-State Reality Check — 2026-09-18

**Audience**: every agent, every session, every future change to BelizeChain.
**Status**: MANDATORY, user-directed. Violations are regressions, not optimizations.

## RULE 1 — Operate THE testnet. Always. No exceptions.

All BelizeChain work happens on **this exact chain**:

| Property | Value |
|----------|-------|
| Chain name | `BelizeChain Testnet` |
| Runtime spec | `specVersion: 105` (`specName: belizechain`) |
| Genesis hash | `0xa22fc115d198777aea39c1b5d8b93f1e9ea68492a30c74118b4b53d546c61ba6` |
| Where it runs | Ceiba host, container `ceiba-node`, spec file `/data/chain/testnet-spec.json` |

**Never, without a brand-new explicit user decision:**
- start a dev chain (`--dev`, `--tmp --chain local`, anything ephemeral)
- build a "new/second" testnet spec or "testnet 2"
- work against mainnet or a mainnet spec on this infrastructure
- reset/replace the chain state or run a fresh genesis "because it's easier"
- change keys, accounts, sudo, or balances except through a documented, reviewed, one-time script approved by the user

Any request touching a different chain gets refused and explained — not silently done.

## RULE 2 — One authority model on testnet: //Alice dev keys

This is deliberate and documented (commit `796dba1`, May 2026): the original sudo SURI was lost/leaked, so the **single-node testnet** genesis was rotated to the well-known `//Alice` dev key pair. Consequences:

- **Alice = chain authority (Sudo) on testnet**, and the **only funded genesis account** (1,000,000,000 DALLA at genesis), plus governance council chair, identity issuers, etc.
- Alice is not "signed in anywhere" by default. To act as Alice, derive it from the standard dev seed (`//Alice` / the standard bottom-drive dev phrase) and pass it to tooling through the environment — operator scripts read the authority SURI from env (`SUDO_SURI`, `ISSUER_SURI`, `SMOKE_SENDER_SURI`) and never embed it themselves, e.g.
  `SMOKE_SENDER_SURI='//Alice' SMOKE_RECIPIENT_SURI='//Bob' python3 scripts/test/smoke_extrinsic.py`.
- **On testnet, "Alice" is effectively the operator's account.** Keep it that way until a deliberate handover script is approved by the user.

## RULE 3 — Address formats: one pubkey, two encodings. Always resolve explicitly.

The BelizeChain SS58 prefix is **1981** (`r1…` "Belize" format). Generic Substrate tooling defaults to prefix **42** (`5…` format).

- The user's founder key:
  - r1 format: `r1SaBq6Cszb9KEv69LAQyKERJyNhXFkMwx5Fy3mLXXyg9sj24`
  - 42 format: `5Cg3Ez7Upm8caDfjonnMKPZ14B3H5daWM75DkYj7yEt4XSKt`
  - same pubkey: `0x1af2d817a183f9b31aa21a7788d2b7b3226ee09a713c4989112bd13c79b53f5f`
- On testnet the access path is via **generic (5…)**: polkadot.js / polkadot-js accepts either, but **scripts, the wallet, and chain tools may render or match only one**. Comparison of two addresses is only valid after resolving both to their pubkey.

## The chain of events that caused today's confusion (all evidence-backed)

1. Testnet launched and stabilized on Alice keys — May 2026, intentional, documented.
2. Sovereign **mainnet** spec was built in August 2026 (commit `7a6e527`) with the founder key configured as root sudo — **for mainnet genesis**, not the running testnet.
3. 2026-09-18: the founder key was used in the Maya Wallet against the **testnet**. On the testnet it is unfunded (nonce 0, free 0) → send fails with `1010: Inability to pay fees`. The "20,000,000 MUnit sudo balance" the user saw elsewhere corresponds to the **mainnet-side connection** (or the mainnet spec state), not the testnet state at `100.81.45.25:9944`.
4. Same pubkey. Different chains. Different balances. That is the whole bug — absent any chain-affecting change today.

## Correct action going forward (the one-time repair, approved plan)

A one-time, non-reset repair on the testnet:
- One signed extrinsic from Alice: `balances.transferKeepAlive(founder_key, 100 DALLA)`.
- (Separate, also from Alice, optional and user-approved): `sudo.set_key(founder_key)` so the founder key owns Sudo authority going forward.
- Then retry the Maya Wallet 0.0001 send → this validates 3a end-to-end and closes the gap.

## Why "we didn't go backwards"

The testnet kept running blocks the whole time. The mainnet spec work stands as intentional parallel work, not wasted effort. The "circle" was only in balances visibility across two chains — the repair is a single transfer, not a rebuild.

## RULE 4 — Dev-keyed chain specs never reach a shared chain (added 2026-09-20)

RULE 2 accepts dev keys as the *testnet authority model*. This rule stops that
model from leaking into repository artifacts or into any future shared chain:

- The running testnet's spec (`/data/chain/testnet-spec.json` on Ceiba) is
  **operator-managed and untracked** (`.gitignore`). It carries the live sudo and
  session keys and must never be committed again.
- `scripts/deploy/validate_chain_spec.sh <spec>` fails closed when a spec carries
  well-known dev accounts (Alice…Ferdie, both SS58 and raw-hex forms) or the
  standard dev mnemonic. `--allow-dev-accounts` exists only for local devnets and
  the CI smoke artifact.
- `.github/workflows/deploy.yml` validates keyless templates plus a self-test that
  the dev-account guard really rejects a dev-keyed spec.
- Building a Live-typed spec from the built-in dev-seeded presets
  (`--chain testnet-template`, `--chain staging`) requires
  `BELIZECHAIN_ALLOW_DEV_SEEDS=1`; without it `build-spec` refuses and says why.
  Passing that switch only acknowledges a template — the finished spec still has
  to clear `validate_chain_spec.sh` (operator keys, no dev accounts).
