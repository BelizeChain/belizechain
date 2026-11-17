# SDK bump — the 2026-11-17 hickory/chrono advisory cluster

> **Status: OPEN — escalated 2026-11-17 by owner decision.** The advisories are
> **deliberately not suppressed**. `Security Audit` is expected to stay red until
> the dependency graph moves, and that red is the reminder.

## What fails, measured

`cargo deny check advisories` in this repo against the advisory database fetched
2026-11-17 returns **12 errors that no existing `ignore` entry covers**:

| ID | Kind | Crate |
|---|---|---|
| RUSTSEC-2026-0489 | vulnerability | hickory-proto |
| RUSTSEC-2026-0490 | vulnerability | hickory-proto |
| RUSTSEC-2026-0491 | vulnerability | hickory-proto |
| RUSTSEC-2026-0505 | vulnerability | hickory-proto |
| RUSTSEC-2026-0511 | vulnerability | hickory-proto |
| RUSTSEC-2026-0512 | vulnerability | hickory-proto |
| RUSTSEC-2026-0514 | vulnerability | hickory-proto |
| RUSTSEC-2026-0530 | vulnerability | hickory-proto |
| RUSTSEC-2026-0531 | vulnerability | hickory-proto |
| RUSTSEC-2026-0550 | vulnerability | hickory-proto |
| RUSTSEC-2026-0551 | vulnerability | hickory-proto |
| RUSTSEC-2026-0543 | unmaintained | chrono |

Eleven of the twelve are one crate. `RUSTSEC-2026-0487` (found on 2026-11-16) is
the same crate and was briefly added to `deny.toml`; **that entry was removed on
2026-11-17** so the whole cluster stays visible together.

## Why no local fix exists

```
hickory-proto 0.24.x
  <- hickory-resolver 0.24.4
  <- libp2p-dns 0.42.0
  <- libp2p 0.54.1
  <- sc-network 0.60.0
  <- belizechain-node
```

- The hickory fixes are in **hickory-proto >= 0.26.2** (0487 states this
  explicitly; the rest are the same research sweep). There is no 0.24/0.25
  backport.
- Reaching 0.26 requires **libp2p-dns >= 0.45.0**, which requires **libp2p
  0.55+**.
- polkadot-sdk pins libp2p **0.54.1** in `stable2606`, `stable2609` and
  `master`, so no released or unreleased SDK rev clears it.
- `cargo update -p hickory-resolver --precise 0.26.x` is refused (semver
  incompatible), and `[patch.crates-io]` cannot make 0.26 satisfy a `^0.24`
  requirement.

A runtime upgrade cannot help: hickory is absent from the runtime graph. This is
a **node binary** dependency, so the fix ships as a node image rebuild, exactly
like the rustls/wasmtime rollout on 2026-10-19.

## Reachability today (why this is escalated, not urgent)

- The resolver is engaged only for `/dns4/`, `/dns6/`, `/dnsaddr/` multiaddrs.
  `testnet-spec.json` has `bootNodes: []`; the node has one authority, zero
  peers, and `shouldHavePeers: false`.
- Several advisories are DNSSEC-validation and zone-file-text issues. This node
  performs no DNSSEC validation and parses no zone file.
- That is a **deployment** mitigation, not a code guarantee. Adding a DNS
  bootnode or a second peer by hostname changes the exposure, so it must not be
  treated as permanent.

## Acceptance criteria for closing this task

Run from the repo root:

```bash
# 1. The failing set is empty (no hickory/chrono IDs, and no new ones).
cargo deny check advisories

# 2. Only the patched line resolves.
cargo tree -i hickory-proto      # every entry >= 0.26.2
cargo tree -i hickory-resolver   # only the post-0.45 libp2p-dns line
cargo tree -i chrono             # gone, or replaced by a maintained crate

# 3. The node still builds and its client surface is intact.
cargo build --release -p belizechain-node
cargo test -p belizechain-node
```

Then, for a rollout:

1. Build the image **on Ceiba** with
   `--build-arg CARGO_FEATURES=testnet-fast-epoch` (mandatory).
2. `scripts/check-node-image-parity.sh --tag <tag>` → PASS.
3. `scripts/check-node-image-parity.sh --tag <tag> --wasm <uncompressed blob>` →
   every imported host function is registered.
4. Rehearse with `scripts/rehearse-runtime-upgrade.py --rpc <throwaway container>`
   if the bump changes the runtime.
5. Roll out with an `.env` backup and a rollback image pinned, then
   `docker exec ceiba-nginx nginx -s reload`.

## Scope and risks

- The bump changes the **node binary**, not the runtime blob. No `set_code`, no
  `spec_version` bump, no storage migration.
- polkadot-sdk bumps can change host functions, inherents and digests. Steps 2–4
  above are what turn those from guesses into evidence; the 2026-10-27 ghost-RPC
  incident is the precedent for skipping them.
- A multi-version jump may also move `chrono`; if it does not, triage 0543
  separately (check whether the remaining consumer is in our workspace or
  upstream).

## What not to do

- Do not add the 12 IDs to `deny.toml` or to the `cargo audit --ignore` lists to
  make the job green. The owner decision on 2026-11-17 was to escalate.
- Do not pin the advisory database to an older snapshot. That hides the cluster
  rather than resolving it.
- Do not edit `testnet-spec.json` while doing this; it still pins the live
  genesis.
