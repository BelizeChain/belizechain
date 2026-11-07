# Dependency Advisory Triage

> **Measured 2026-11-07** against advisory DB dated 2026-11-06, `cargo-audit-audit
> 0.22.1`, `cargo-deny 0.20.2`, lock file at `spec_version` 125.
>
> Every row below was produced by running the tool or by reading the dependency
> graph in this session. Nothing is inherited from an earlier note. Re-measure
> with the commands at the end rather than trusting the counts.

## Why this document exists

"13 advisories" was being carried around as an open item with no record of *which*
ones, whether they are fixable, or whether the affected crate is even compiled.
That is not a decision anyone can make. This file answers those three questions per
advisory so the item can be closed or escalated on evidence.

**Headline: none of the 13 is fixable from this repository.** Each is either a
semver-major pinned by the Polkadot SDK, or — in one case — an *exact* `=` pin. The
useful work is therefore to classify exposure and make the suppressions
self-checking, not to attempt a bump.

## 1. The 13 vulnerabilities

`cargo audit --json` reports 13 before any ignore is applied.

**"Compiled" means present in the node's normal dependency graph**, measured with
`cargo tree -p belizechain-node -e normal -i <crate>@<version>`. A crate that sits in
`Cargo.lock` but not in that graph is never built and never runs.

| Advisory | Crate | Ver | Fixed in | Compiled? | Why it cannot be fixed here |
|---|---|---|---|---|---|
| RUSTSEC-2026-0387 | crypto-bigint | 0.5.5 | `>=0.7.6` | **yes** | Major. Pinned by the SDK's crypto stack. |
| RUSTSEC-2026-0119 | hickory-proto | 0.24.4 | `>=0.26.1` | **yes** | Major. `libp2p-dns` requires `^0.24.1`. |
| RUSTSEC-2025-0055 | tracing-subscriber | 0.2.25 | `>=0.3.20` | **yes** | Major. Separate 0.2 line. |
| RUSTSEC-2025-0055 | tracing-subscriber | 0.3.19 | `>=0.3.20` | **yes** | **Exact pin.** `sp-tracing` requires `=0.3.19`, so even the patch bump is refused — see §3. |
| RUSTSEC-2026-0381 | webpki-roots | 0.25.4 | `>=1.0.10` | **yes** | Major. `libp2p-websocket` requires `^0.25`. |
| RUSTSEC-2026-0403 | zstd-safe | 6.0.6 | `^7.4.0`, `>=8.2.0` | **yes** | Major. `zstd 0.12` requires `^6`. |
| RUSTSEC-2026-0258 | h2 | 0.3.27 | `>=0.4.16` | no | Major; absent from the build. |
| RUSTSEC-2025-0009 | ring | 0.16.20 | `>=0.17.12` | no | Major; absent from the build. |
| RUSTSEC-2026-0344 | ruzstd | 0.8.3 | `>=0.9.1` | no | Major; absent from the build. |
| RUSTSEC-2026-0104 | rustls-webpki | 0.101.7 | `>=0.103.13` | no | Major; absent from the build. |
| RUSTSEC-2026-0099 | rustls-webpki | 0.101.7 | `>=0.103.12` | no | — as above — |
| RUSTSEC-2026-0382 | rustls-webpki | 0.101.7 | `>=0.103.16` | no | — as above — |
| RUSTSEC-2026-0098 | rustls-webpki | 0.101.7 | `>=0.103.12` | no | — as above — |

So: **6 advisories affect code that is compiled**, 7 affect crates that are never
built. The four `rustls-webpki 0.101.7` rows are one unreachable crate reached only
through libp2p's optional `quic` feature, which `sc-network`'s default feature set
does not enable.

## 2. The zstd-safe unsound cluster — tracked, not dismissed

`cargo audit` also reports **26 warning-class** advisories (unmaintained / unsound).
Warnings do not fail the build, so they were being tolerated silently. The one worth
naming is `zstd-safe 6.0.6`, which carries **9 unsound advisories** including a heap
buffer overflow (`RUSTSEC-2026-0402`), a double free (`RUSTSEC-2026-0404`) and
several use-after-frees.

It is in the compiled graph:

```
zstd-safe v6.0.6 <- zstd v0.12.4 <- sp-maybe-compressed-blob v11.0.0
                 <- sc-executor-common <- sc-executor <- belizechain-node
```

**Reachability argument, stated as an argument rather than a guarantee:** the only
consumer is `sp-maybe-compressed-blob`, which decompresses compressed runtime WASM
blobs — our own build output, read from disk or the chain spec. No first-party Rust
in `pallets/`, `runtime/` or `node/` calls `zstd` directly (verified by grep). The
affected APIs (`try_clone`, `train_from_buffer`, `StableInBuffer`, streaming dictionary
changes) are not on a one-shot decompress path. **This has not been proven by test**;
it is a reading of the dependency graph and the API surface. Treat it as
low-but-unquantified risk, and the SDK bump is the real fix.

## 3. Why the tracing-subscriber patch bump is refused

This is the one advisory that *looks* trivially fixable, so the failure is recorded
here to stop it being re-attempted:

```
$ cargo update -p tracing-subscriber@0.3.19 --precise 0.3.20
error: failed to select a version for the requirement `tracing-subscriber = "=0.3.19"`
candidate versions found which didn't match: 0.3.20
required by package `sp-tracing v16.0.0`
```

`sp-tracing` uses an **exact** (`=`) requirement, not a range. No `cargo update` and
no `[patch.crates-io]` can satisfy it. `Cargo.lock` was verified byte-identical after
the attempt.

## 4. Where the suppressions live, and why they are split

| Location | Covers | Why not everything |
|---|---|---|
| `deny.toml` | 7 advisories | Only the ones `cargo-deny` can actually see in a resolved graph. A suppression that can never fire protects nothing, so the 6 lock-only rows are deliberately absent. |
| `.github/workflows/security-audit.yml` | all 13 | `cargo audit` scans the whole `Cargo.lock`, including entries not built under current features. |

Two suppressions are self-checking rather than bare ignores. The workflow asserts
that `crypto-bigint` still resolves below `0.7.6` and that `zstd-safe 6.x` is still
present, and **fails if a patched version appears** — so an ignore cannot quietly rot
into a stale excuse.

## 5. Verification (run these; do not quote the counts)

```bash
# must print "advisories ok"
cargo deny check advisories

# must exit 0 — uses the workflow's own --ignore list
IGNORES=$(grep -oE '\-\-ignore RUSTSEC-[0-9-]+' .github/workflows/security-audit.yml \
  | awk '{print $2}' | sort -u)
cargo audit $(for i in $IGNORES; do printf -- '--ignore %s ' "$i"; done); echo "exit=$?"

# exposure: an empty result means the crate is not compiled
cargo tree -p belizechain-node -e normal -i <crate>@<version>
```

**Measured 2026-11-07:** `cargo deny check advisories` → `advisories ok`;
`cargo audit` with the workflow ignores → exit `0`.

## 6. What would actually clear these

A Polkadot SDK bump. Several rows are already known to be unaffected by the
`stable2606 → stable2609` move (tested 2026-10-09: `hickory-resolver` stays on 0.24
because `libp2p-dns v0.42.0` still requires `^0.24.1`, so the hickory advisory
survives the bump). Do not spend another cycle assuming the SDK bump fixes this
class; re-measure the graph at the new rev first.

The durable reduction is upstream: when `libp2p-dns`, `sp-tracing` and `zstd` move
their pins, the corresponding row disappears and the compensating assertion in the
workflow fires to say so.
