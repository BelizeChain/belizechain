# Dependency Advisory Residuals

Accepted dependency advisories that **cannot be fixed in this repository** and are waiting
on an upstream change. Each entry records the exact dependency path, why it cannot move, the
real exposure, and the concrete condition that clears it.

**Last reviewed:** 2026-10-13

Companion records:
- `deny.toml` — advisories triaged for `cargo-deny` (target/feature graph).
- `.github/workflows/security-audit.yml` — ignore lists for `cargo audit` and the yanked-crate
  check, which scan the whole `Cargo.lock` rather than the feature graph.

> **Note on the two tools.** `cargo-deny` resolves the real feature graph; `cargo audit` scans the
> entire lockfile. A crate reachable only through a non-default feature therefore fails
> `cargo audit` while `cargo deny check` stays green. Both lists are kept in sync by hand — the
> `Cargo Audit` and `Yanked Crate Check` steps of `security-audit.yml` must always carry the same
> `--ignore` set, or an advisory can pass one job and fail the other.

---

## 1. `hickory-resolver` — DNSSEC validation failures obscured — **HIGH** (open)

| | |
|---|---|
| Advisory | **GHSA-5j98-2g5x-46v6** |
| Summary | `Resolver::lookup()` and `Resolver::lookup_ip()` APIs obscure DNSSEC validation failures |
| Severity | High — CVSS 7.5 (`AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:H/A:N`) — **integrity only**, no availability impact |
| Published | 2026-10-05 |
| Vulnerable | `< 0.26.2` |
| In our lockfile | **0.24.4** (vulnerable) and 0.26.3 (not affected) |
| Source of report | GitHub Dependabot alert 31 |
| Status | **Accepted residual — waiting on upstream** (decision 2026-10-13) |

### Dependency path

```
hickory-resolver 0.24.4
  <- libp2p-dns 0.42.0
  <- libp2p 0.54.1
  <- sc-network 0.60.0        (polkadot-sdk rev 7a8daa68, stable2609)
  <- belizechain-node
```

This path is **direct**: `sc-network` is a normal dependency of `belizechain-node`, so unlike
`ruzstd` (see below) this crate **is present in the shipped node binary**, not just in
benchmark-only tooling.

A second copy, `hickory-resolver 0.26.3`, arrives via `litep2p 0.15.3 <- sc-network` and is
**past the fix**, so it is not affected.

### Why it cannot be fixed here

It cannot be moved by a version bump in this repository. Verified 2026-10-13:

```
$ cargo update -p hickory-resolver@0.24.4 --precise 0.26.2 --dry-run
error: failed to select a version for the requirement `hickory-resolver = "^0.24.1"`
candidate versions found which didn't match: 0.26.2
location searched: crates.io index
required by package `libp2p-dns v0.42.0`
  ... which satisfies dependency `libp2p-dns = "^0.42.0"` of package `libp2p v0.54.1`
  ... which satisfies dependency `libp2p = "^0.54.1"` of package `sc-network v0.60.0`
```

`libp2p-dns` is where the pin lives, and the required bump is a major-ish jump:

| `libp2p-dns` | requires `hickory-resolver` | clears GHSA-5j98-2g5x-46v6? |
|---|---|---|
| **0.42.0** (ours) | `^0.24.1` | no |
| 0.43.0 | `^0.25.0-alpha.4` | no |
| 0.44.0 | `^0.25.2` | no |
| **0.45.0** | `^0.26` | **yes** |

`libp2p 0.54.1` pins `libp2p-dns = "^0.42.0"`, so 0.45.0 is unreachable. Moving it requires
polkadot-sdk to ship a `libp2p` that depends on `libp2p-dns >= 0.45.0` — the same constraint
that keeps `RUSTSEC-2026-0119` open in `deny.toml`. **No runtime upgrade can help:** this is a
node binary dependency and is absent from the runtime graph entirely.

### Actual exposure

Low, for a specific and checkable reason: the resolver is engaged only for `/dns/` multiaddrs,
and this chain specifies none.

- `testnet-spec.json` sets `bootNodes: []` — there are no DNS bootnodes to resolve.
- `codeSubstitutes` is empty (no DNS-fetched runtime overrides).
- Bootstrap nodes are populated by CLI/config only; the built-in mainnet/testnet lists are empty.

So in the current Ceiba deployment this code path is effectively unexercised. The advisory is an
integrity issue (a caller may act on a response whose DNSSEC validation failed), which would
matter if DNS bootnodes were used — it is not a liveness or consensus risk.

**Not verified:** whether any operator-supplied `/dns/` multiaddrs or a future bootnode list
would change this. If DNS bootnodes are ever introduced, this residual must be re-assessed
before the change lands, not after.

### Verification

```bash
# versions present
python3 -c "
import tomllib; d=tomllib.load(open('Cargo.lock','rb'))
print([ (p['name'],p['version']) for p in d['package'] if p['name']=='hickory-resolver' ])"

# confirm the pin is still upstream, not ours
cargo update -p hickory-resolver@0.24.4 --precise 0.26.2 --dry-run   # must still fail
```

### Exit criteria

Cleared when `libp2p-dns >= 0.45.0` (i.e. `hickory-resolver >= 0.26.2`) enters the tree via a
polkadot-sdk bump of `libp2p`. On the next SDK bump, re-run the check above; if it succeeds,
move the pin and delete this entry.

### Related, distinct advisories on the same path

Do not conflate these — they are **different advisories** with different failure modes:

| Advisory | Crate | Nature | Where tracked |
|---|---|---|---|
| `GHSA-5j98-2g5x-46v6` | `hickory-resolver` | DNSSEC failures obscured (integrity) | **this document** |
| `RUSTSEC-2026-0119` | `hickory-proto` | O(n²) DNS name-compression CPU exhaustion (DoS) | `deny.toml` + `security-audit.yml` |

`RUSTSEC-2026-0119` has alias `GHSA-q2qq-hmj6-3wpp`, not the GHSA above. `GHSA-5j98-2g5x-46v6`
carries no RustSec identifier and is absent from the RustSec advisory DB, which is why only
Dependabot reports it — `cargo audit` does not.

---

## 2. `ruzstd` — decompression bomb — HIGH (suppressed, audit-only)

| | |
|---|---|
| Advisory | `RUSTSEC-2026-0344` |
| Vulnerable | `ruzstd 0.8.2`; patched `>= 0.9.1` |
| Path | `ruzstd <- smoldot <- smoldot-light <- subxt-lightclient <- subxt <- frame-benchmarking-cli <- belizechain-node` |
| Status | Suppressed in `security-audit.yml` only (2026-10-13) |

`frame-benchmarking-cli` is an **optional** dependency enabled solely by the non-default
`runtime-benchmarks` feature, which the production Dockerfile does not set. The crate is
therefore **absent from shipped node images**, and `cargo-denys`'s feature graph never sees it —
which is why `Cargo Deny` stays green. No upgrade path exists: every `smoldot` release
(2.2.0, 3.0.0, 4.0.0 and 5.0.0) still requires `ruzstd ^0.8.0`.

Deliberately **not** in `deny.toml`'s `ignore` list — a suppression there could never fire, which
that file's own rules forbid.
