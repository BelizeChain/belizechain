# Security Hygiene Queue - 2026-05-02

Status: Stabilization queue, not an activation blocker
Scope: BelizeChain core, Nawal, UI, and shared Phase 2 operations

This queue captures the non-blocking security and repository hygiene work found during the Phase 2 post-push sweep. It separates CI blockers from broader dependency debt.

## Current Counts

Open Dependabot alerts from GitHub API on 2026-05-02:

| Repo | Critical | High | Medium | Low | Total |
|---|---:|---:|---:|---:|---:|
| `BelizeChain/belizechain` | 2 | 2 | 11 | 7 | 22 |
| `BelizeChain/nawal-ai` | 0 | 2 | 0 | 0 | 2 |
| `BelizeChain/ui` | 4 | 45 | 40 | 9 | 98 |

Sample alert packages:

| Repo | Ecosystem | Packages |
|---|---|---|
| `belizechain` | Rust | `wasmtime`, `rustls-webpki`, `hickory-proto`, `tracing-subscriber`, `libsecp256k1` |
| `nawal-ai` | pip | `black` duplicate high-severity alert entries |
| `ui` | npm | `next`, `protobufjs`, `lodash`, `defu`, `postcss`, `uuid`, `dompurify` |

## Queue

| Priority | Item | Owner Repo | Notes | Next Action |
|---|---|---|---|---|
| P0 | Keep CI audit visibility green | `belizechain` | Security audit now has current SDK-blocked RustSec triage and `pipefail`; path filters now include `deny.toml` and the workflow file. | Watch commit `260a6b0` plus the follow-up workflow-path commit until green. |
| P1 | Polkadot SDK dependency upgrade | `belizechain` | Core Rust alerts are mainly SDK-blocked through pinned `polkadot-sdk`, including `wasmtime`, `rustls-webpki`, `hickory-proto`, `tracing-subscriber`, and `libsecp256k1`. | Plan a focused SDK upgrade branch; validate runtime build, node build, session/BABE regression tests, and Ceiba reset/spec flow. |
| P1 | UI npm alert reduction | `ui` | UI has the largest alert count and includes critical/high runtime packages. | Start with `next`, `protobufjs`, `lodash`, `defu`, and `dompurify`; run workspace type-check/build for shared, Maya Wallet, and Blue Hole Portal. |
| P2 | Nawal dev dependency cleanup | `nawal-ai` | Current alerts are duplicate `black` entries. | Update formatter/dev dependency pins and run focused pytest plus ruff/format checks. |
| P2 | Tracked binary cleanup | `belizechain` | Root `belizechain-node` is a tracked 66 MB ELF binary and triggers GitHub large-file warnings on push. | Prefer `git rm --cached belizechain-node` plus `.gitignore` and release/container artifacts for binaries. Do not rewrite history without explicit approval. |
| P2 | Dependabot alert deduplication | `ui`, `nawal-ai` | Some alerts may duplicate across workspaces or lockfile paths. | Group by package, advisory, workspace, and runtime/dev dependency before changing pins. |

## Tracked Binary Notes

Confirmed on 2026-05-02:

- Path: `belizechain-node`
- Size: 66 MB
- Type: ELF 64-bit x86-64 executable, stripped
- Git state: tracked in the core repo

Recommended cleanup without history rewrite:

```bash
git rm --cached belizechain-node
printf '/belizechain-node\n' >> .gitignore
```

Only consider history rewriting with `git filter-repo` or BFG after explicit approval, team coordination, and backup of all protected refs.

## CI Notes

- The 2026-05-02 core CI fix updated `deny.toml` for target-graph advisories and kept full-lockfile exceptions in `.github/workflows/security-audit.yml`.
- `cargo audit` now runs under `set -o pipefail`; future unignored vulnerabilities should fail the audit job correctly.
- Local validation passed with the workflow ignore set:
  - `cargo-deny 0.19.0 check advisories`
  - `cargo audit`
  - `cargo audit --deny yanked`

## Exit Criteria For This Queue

- No tracked generated node binary in future core commits.
- Core Dependabot alerts reduced or explicitly mapped to the next SDK upgrade.
- UI critical/high alerts reduced to an agreed baseline or tracked with package-level blockers.
- Nawal duplicate `black` alerts resolved.
- Security workflows trigger on dependency config changes, not only manifest changes.