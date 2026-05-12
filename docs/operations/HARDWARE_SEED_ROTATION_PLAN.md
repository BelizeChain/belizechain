# Hardware Seed Rotation Plan — Replace `//Alice` Sudo + Authority

Status: **planned, not executed**
Last update: 2026-05-12
Related: [`ALICE_SUDO_RESET_2026-05-11.md`](ALICE_SUDO_RESET_2026-05-11.md), [`SECURITY_HYGIENE_QUEUE_2026-05-02.md`](SECURITY_HYGIENE_QUEUE_2026-05-02.md)

## Why this is still open

The 2026-05-11 Alice-sudo reset rotated leaked BABE/GRANDPA validator mnemonics by creating a new chain genesis, then planted `//Alice` (the well-known Substrate dev seed) as:

- the chain `sudo` key,
- the sole BABE authority (sr25519, key type `babe`),
- the sole GRANDPA authority (ed25519, key type `gran`),
- `session.keys[0]`,
- `balances[0]` endowment,
- `governance.councilMembers[0]`,
- `identity.initial*Issuers[0]`.

`//Alice` is publicly known. It is acceptable for a single-node Tailscale-only testnet, but **must be rotated before**:

1. **Any external peer** joins (LAN-exposed bootnode, public DNS, or any host outside the Ceiba Tailnet).
2. **Any value-bearing extrinsic** is processed that is not pure throwaway test traffic.
3. **Any production GEM contract instantiation** intended to survive.
4. **Any non-developer accessing the RPC endpoint** with sudo or governance intent.

## Trigger criteria — rotate when ANY of these become true

| # | Trigger | Source of truth |
|---|---|---|
| T1 | First non-Ceiba validator/full-node is provisioned | Phase 4 multi-node task in `docs/deployment/PHASE2_CEIBA_SERVICES_PLAN.md` follow-on |
| T2 | RPC is exposed to anything broader than the current Tailnet (e.g. published WSS via nginx for external integrators) | `infra/docker-compose.ceiba.yml`, `nginx/` site files |
| T3 | A real-user (non-`//Alice`, non-`//Bob`, non-`//treasury`) account is provisioned and funded with intent to retain balance | governance/treasury workflow |
| T4 | Genesis is intended to harden into the **incentivized testnet** chain (not a development testnet) | governance decision recorded in `CHANGELOG.md` / `docs/ROADMAP.md` |
| T5 | A SECURITY-class incident requires fresh sudo (e.g. another seed-leak event) | post-mortem in `docs/operations/` |

Until any of T1–T5 fires, the current `//Alice`-as-sudo baseline is acceptable; the chain runs in single-node testnet mode behind Tailscale.

## Threat model going in

- We are protecting against (a) public dev-key impersonation, (b) opportunistic RPC-side sudo abuse if the binding ever leaves Tailscale, and (c) compromise of the developer workstation that currently holds `//Alice` SURIs in shell env.
- We are **not** yet trying to defend against host compromise of Ceiba itself; that requires HSM-backed signing at the validator (separate work).

## Hardware options (decision pending)

| Option | Pros | Cons | Recommended for |
|---|---|---|---|
| A. Ledger Nano X with [polkadot-vault-style derivation](https://polkadot.network/development/polkadot-vault/) (cold device, app-signed extrinsics) | Air-gapped private key, well-vetted | Manual signing per extrinsic; Substrate-Ledger app only supports sr25519 for the sudo path, not ed25519 for grandpa | Sudo + balances + council |
| B. Polkadot Vault / Parity Signer on a wiped offline Android device | Free, full sr25519 + ed25519, QR-channel | Requires a dedicated offline phone; operationally heavier | Validator session keys |
| C. YubiKey + age-encrypted seed in tamper-evident envelope + paper backup | Cheap, durable | Key still lives on a host at signing time (not strictly air-gapped) | Backup escrow only |
| D. SSS-split mnemonic (3-of-5 Shamir) across distinct trust domains | Survives single-trustee loss | High operational cost; only useful for recovery, not daily signing | Genesis sudo escrow |

**Tentative direction**: Option A for sudo + governance + balances, Option B for the validator session keys (BABE/GRANDPA). Option D for offline recovery escrow.

## Procedure outline (to be executed at trigger time)

This is **not** the runbook — that gets written in a dated companion doc (`HARDWARE_SEED_ROTATION_<stamp>.md`) when the trigger fires. The outline:

1. **Generate new keys on the chosen hardware**, fully air-gapped:
   - Sudo sr25519 (Option A)
   - BABE sr25519 + GRANDPA ed25519 (Option B)
   - Council member #1 sr25519
2. **Record SS58 + pubkey hex only** on the operator workstation. Never copy the SURI off the hardware.
3. **Capture chain pre-rotation snapshot** using the procedure now baselined in `CEIBA_BACKUP_RESTORE_DRILL_2026-05-02.md` (executed 2026-05-12 → backup dir `/data/backups/phase3-restore-drill-20260512-175820/`). Verify gzip + tar listings before proceeding.
4. **Two paths, decided at trigger time**:
   - **R1 — Genesis re-cut** (preferred when single-node, no external state we want to preserve): mirror the 2026-05-11 Alice-sudo reset but swap every `//Alice`-derived value for the new hardware-derived addresses. Same procedure documented in [`ALICE_SUDO_RESET_2026-05-11.md`](ALICE_SUDO_RESET_2026-05-11.md).
   - **R2 — In-place rotation** (required if chain state must be preserved): use `Sudo::sudo(Sudo::set_key(new_key))` and `Session::set_keys` for the validator, signed by `//Alice` once, then revoke `//Alice` balances. R2 is operationally riskier; only viable if the chain has accumulated state we cannot replay (real GEM contract addresses with off-chain references, real Pakit metadata tied to addresses, etc.).
5. **Verify rotation** before declaring done:
   - `system_chain` / `chain_getHeader` healthy
   - `sudo.key()` returns the new SS58
   - `Session::nextKeys` and `Session::queuedKeys` reflect the new authority set
   - Block production by the new BABE pubkey visible in `Babe::Authorities`
   - GRANDPA finalization advancing under the new authority set
6. **Rotate sibling expectations** in the same window:
   - `infra/.env` `BLOCKCHAIN_SUDO_*` if any sudo SS58 is referenced
   - GEM redeploy if R1 path (new genesis → new contract addresses)
   - UI rebuild only if hardcoded sudo SS58 lives in the bundle (currently none — genesis hash is read dynamically)
   - `ALICE_SUDO_RESET_*` runbook supersession notice in `docs/operations/`
7. **Shred dev-key fallback**: confirm `/data/chain/authority-keys-*` no longer exists on Ceiba (already done 2026-05-12), wipe `//Alice` from `/opt/belizechain/.env`, scrub shell history, rotate any operator workstation env vars.
8. **Record the rotation** in `CHANGELOG.md`, a dated runbook doc, and `SECURITY_HYGIENE_QUEUE_2026-05-02.md`.

## Rollback rules

- Keep the pre-rotation `/data/chain` tarball and pre-rotation `testnet-spec.json` until the new chain has produced and finalized at least one full epoch under the new authority set.
- If R1 fails to produce blocks: stop `ceiba-node`, restore `/data/chain.pre-rotation-<stamp>`, restart, verify finality, file post-mortem before retrying.
- If R2 fails (in-place): the rollback path is non-trivial because `Sudo::set_key` is finalized on-chain. R2 must only be attempted with `//Alice` SURI still accessible to the operator until the new key has signed at least one successful sudo extrinsic; otherwise the chain is locked out.

## Blast radius if NOT rotated before each trigger

| Trigger | Worst case |
|---|---|
| T1 ignored | Any peer can construct sudo extrinsics; chain governance is meaningless |
| T2 ignored | Internet-reachable RPC + known sudo = full chain takeover |
| T3 ignored | Real user balances at risk of unauthorized burn/transfer via sudo |
| T4 ignored | Incentivized testnet credibility destroyed; cannot be "fixed" by later rotation since history is signed |
| T5 ignored | Compounding incident — leaked key plus extant test sudo |

## Status flags

- [ ] Hardware procured (Ledger Nano X + offline Android device)
- [ ] Polkadot-Substrate Ledger app installed and tested against a throwaway dev chain
- [ ] Polkadot Vault flashed and air-gap verified (no SIM, no Wi-Fi, no Bluetooth)
- [ ] Genesis re-cut workflow rehearsed on a disposable dev chain (`belizechain_dev` or a fresh `--tmp` node)
- [ ] Sudo SS58 + pubkey hex recorded in `infra/.env.example` placeholders (not the real values)
- [ ] Trigger event fired (one of T1–T5)
- [ ] Rotation executed and recorded

## Open questions

1. Do we keep `//Bob` (and other dev seeds) anywhere in the runtime config after rotation? Decision: **no** — sweep them in the same pass.
2. Do we want a **multi-sig sudo** at rotation time (e.g. 2-of-3) instead of a single hardware key? This would change the procedure; recommend revisiting before T4 fires, not before T1.
3. Should validator session-key rotation be split from sudo rotation, or always done together? Recommend together for the first hardware rotation, separately afterwards (validator rotates more often than sudo).
