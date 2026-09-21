# Runtime Upgrade: spec 105 → 107 (Ceiba, 2026-09-20)

**Status:** Executed and verified. Chain progressing; on-chain runtime is spec 107.
The upgrade was enacted in **block 37901** (105 at 37900 → 107 at 37901), which
carries `system.CodeUpdated` and `sudo.Sudid {"sudoResult":"Ok"}`. This is a
**testnet-only** operation under
[TESTNET_ONLY_RULE_2026-09-18.md](TESTNET_ONLY_RULE_2026-09-18.md) — no external peers,
no users beyond the operator.

## What this upgrade carries

Cumulative from the repo's `belizechain` branch (commit `b8887a2` plus the doc fix
`33deb61`):

1. **M-5 — real benchmarked weights.** Every hardcoded/placeholder extrinsic weight in
   all 19 pallets is replaced with a measured value (steps 50 / repeat 20 against the
   `dev` chain). No pallet consumes an estimate in production builds.
2. **Technical-council origin fix.** `EitherOfDiverse<EnsureRootWithSuccess, EnsureMember>`
   was replaced by `SignedTechnicalCouncilMember`. `EnsureMember` only matches
   `RawOrigin::Member`, which only `pallet_collective::execute` can produce — so the 7
   configs using it (oracle, payroll, landledger, quantum, community, moderation, bns)
   silently accepted Root only. Community's `attest_participation` was unreachable by
   any origin. Justice's `MediatorOrigin` had the same defect plus a benchmark-only
   `EnsureSigned` override, so its benchmarks tested an origin production never had.
3. **Oracle price-feed bootstrap.** `submit_price` propagated
   `aggregate_price_feed`'s `InsufficientConsensus` via `?`, reverting the insert and
   making the first price un-writable. Now routed through `try_aggregate_price_feed`,
   which swallows *only* `InsufficientConsensus`; every later step is infallible, so no
   real error is masked.
4. **NEMO emergency-authority registry.** `is_emergency_authority` used a KYC-L3 +
   not-sanctioned proxy, so any L3-verified non-sanctioned citizen could issue **and
   resolve** emergency alerts. It now reads `mesh::EmergencyAuthorities`.

## Storage impact

**No migration required.** The only new storage is:

| Item | Type | Default |
|------|------|---------|
| `mesh::EmergencyAuthorities` | `StorageMap<Blake2_128Concat, AccountId, bool, ValueQuery>` | `false` |
| `mesh::EmergencyAuthorityCount` | `StorageValue<u32, ValueQuery>` | `0` |

Both are `ValueQuery`-backed, so they read empty/false/0 immediately after the upgrade.
Mesh call indices are 0–15 with no gaps or duplicates — `add_emergency_authority` (14)
and `remove_emergency_authority` (15) are appended. `pallet-storage-proof` sits at
runtime index 38, also appended. **No call index moved, so no client re-encoding is
needed.**

## Verified evidence

| Property | Value |
|----------|-------|
| Upgrade block | **37901** (`state_getRuntimeVersion` returns 105 at 37900, 107 at 37901) |
| Events in block 37901 | `system.CodeUpdated`, `sudo.Sudid {"sudoResult":"Ok"}`, `transactionPayment.TransactionFeePaid` |
| On-chain `:code` bytes | 1,417,379 |
| On-chain `:code` blake2b-256 | `1aa7d55d017dd3fabb16b1d525dea6a6969c1f397fa65d85883903bb4aec0547` |
| Built blob blake2b-256 | `1aa7d55d017dd3fabb16b1d525dea6a6969c1f397fa65d85883903bb4aec0547` |
| Previous (105) `:code` blake2b-256 | `1b5a8b580829d1b9e1faa27d3a219a2da566031267baf6f9a809cd1c0d7e1b42` |
| `state_getRuntimeVersion` | `specName: belizechain`, `specVersion: 107`, `transactionVersion: 1` |
| New mesh extrinsics present | `add_emergency_authority`, `remove_emergency_authority` |
| `emergencyAuthorityCount` | `0` (registry empty, as designed) |
| Block production | advancing ~1 block / 6 s after the upgrade |

The on-chain hash matching the locally built blob is the load-bearing check: it proves
*which* WASM is executing, not merely that a number changed.

### Submission

`sudo.sudo(system.setCode(<wasm>))`, signed by the on-chain sudo key, which is the
well-known `//Alice` account (`d43593c7…da27d`) per
[ALICE_SUDO_RESET_2026-05-11.md](ALICE_SUDO_RESET_2026-05-11.md). It renders as
`r1Wm6WgKeFpYHgngPgEF2p4NjzJs8QY5FyctGVcDnN24RGRur` because this chain uses ss58 prefix
**1981** — always compare accounts as raw hex, never as SS58 strings.

`sudo.sudo` dispatches through `dispatch_bypass_filter(RawOrigin::Root)`, and the runtime
sets no `frame_system::BaseCallFilter`, so nothing filters the call.

## Preflight checks performed

1. `specName` on-chain matched the blob before submitting.
2. On-chain `specVersion` was still the expected 105.
3. Signer identity matched `Sudo::Key`, compared as raw bytes.
4. Blob 1.42 MB < the 3.93 MB normal-dispatch limit (`NORMAL_DISPATCH_RATIO` 75% of 5 MiB).
5. No `BaseCallFilter` blocking `sudo.sudo`.

## Rollback

Blobs are kept **outside** `/tmp` in `~/belizechain-runtime-backups/` on the operator
workstation:

- `spec105-rollback-20260920.wasm` — the previous on-chain runtime
- `spec107-live-20260920.wasm` / `onchain-code-verified-spec107.wasm` — the live runtime
- `DO-NOT-DEPLOY-spec107-benchmark-build.wasm` — a contaminated artifact, see below

**TRAP — `set_code` cannot roll back.** `frame_system::set_code` enforces a *strictly
increasing* spec version (`new_version.spec_version <= current_version.spec_version` →
`SpecVersionNeedsToIncrease`), so the 105 blob cannot be re-applied with `set_code`, nor
with `apply_authorized_upgrade`, which shares the check. Rollback requires:

```
sudo.sudo(system.setCodeWithoutChecks(<spec105 blob>))
```

`set_code_without_checks` is `call_index(3)` and Root-only.

## Two traps worth remembering

### 1. Benchmark feature leaks into the deployable blob

`cargo build --release -p belizechain-node --features runtime-benchmarks` also leaves a
blob at `target/release/wbuild/belizechain-runtime/*.wasm` whose generated
`Cargo.toml` records:

```toml
features = ["serde", "runtime-benchmarks", "frame-benchmarking"]
```

Deploying that blob would ship the ~17 permissive `runtime-benchmarks` provider
bypasses (KYC level, sanctions, oracle-operator and bridge-operator checks) into
production. **Always rebuild with no feature flags** and confirm the recorded features
are `["serde"]` before extracting the blob. The contaminated build here was
`b71c41f4…` (1,522,783 bytes) and was never deployed; the live blob is `1aa7d55d…`
(1,417,379 bytes).

### 2. polkadot-js cannot decode blocks (pre-existing, not caused by this upgrade)

The pinned polkadot-sdk rev `2e4dd0bc…` declares `EXTRINSIC_FORMAT_VERSION = 5`, and
bare/unsigned extrinsics — including the `timestamp.set` inherent at index 0 of every
block — are emitted as version 5. `@polkadot/api 10.13.1`, which is what the `ui` repo
currently installs, rejects them:

```
Unsupported unsigned extrinsic version 5
```

This fails identically on pre-upgrade blocks (#37905, #37935) and post-upgrade blocks,
so it is **not** a regression from 105 → 107. It does mean any block-decoding feature in
the UI (the explorer in particular) is broken against this chain regardless of spec
version, and needs a newer polkadot-js.

## Follow-up (not done, needs a decision)

- **Rebuild and redeploy the node image.** The running container still uses the
  `6c447f1-epochfix-spec105-20260502` binary, whose *native* runtime is spec 105 while
  the chain is 107. The node correctly executes the on-chain WASM, but native and WASM
  are out of sync. Rebuilding the node against `b8887a2` (no `runtime-benchmarks`)
  restores parity. Requires a container restart — needs explicit approval.
- **Seed the NEMO registry.** `EmergencyAuthorities` is empty, so emergency alerts only
  work through the governance origin until authorities are added.
- **UI polkadot-js upgrade.** See trap 2.
- **Remove `pallet_sudo`** before opening the chain to external peers (deferred by
  operator decision on 2026-09-20; acceptable while the operator is the only user).
