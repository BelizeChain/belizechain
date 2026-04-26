---
name: "BelizeChain Engineering and Review Rules"
description: "Use when writing or reviewing BelizeChain Rust, Substrate runtime, pallets, node code, deployment guidance, project docs, consensus logic, or evidence-driven bug triage. Covers determinism, true-positive review standards, multi-repo accuracy, Ceiba ops alignment, and validation."
applyTo:
  - "runtime/**/*.rs"
  - "node/**/*.rs"
  - "pallets/**/*.rs"
  - "**/Cargo.toml"
  - "README.md"
  - "docs/**/*.md"
  - ".github/**/*.md"
---
# BelizeChain Engineering and Review Rules

## Sources Of Truth

- For pallet presence, runtime topology, and wiring, prefer `Cargo.toml` and `runtime/src/lib.rs` over README prose.
- For live operations, prefer `docs/operations/CEIBA_OPERATIONS_RUNBOOK.md` over older deployment docs unless the task explicitly targets a legacy path.
- For multi-repo architecture and sibling responsibilities, use `docs/architecture/multi-repo-overview.md`.
- For sibling service rollout state, use `docs/deployment/PHASE2_CEIBA_SERVICES_PLAN.md`.
- For GitHub-facing repo status, releases, workflows, and PR state, prefer `gh` data over handwritten compatibility tables.

## Review Standard

- Call something a bug only when you can connect it to code, tests, command output, logs, or a concrete contract violation.
- Separate confirmed defects, probable risks, and open questions.
- If documentation disagrees with implementation, call out the mismatch explicitly and state which source is authoritative for the task.
- Mention missing tests, missing rollback steps, stale docs, or validation gaps when they materially affect confidence.
- If you find adjacent stale docs or operational drift while touching a file, either fix it in the same slice or call it out explicitly.

## Runtime And Pallet Safety

- Keep runtime and pallet logic deterministic.
- No floating point in consensus or on-chain state logic.
- No network, filesystem, wall-clock, or host-dependent behavior in runtime code.
- Avoid `unwrap()` and `expect()` in production paths; return `Result<T, Error>` and propagate structured errors.
- Preserve explicit pallet indices, storage compatibility, and bounded iteration when changing runtime wiring or storage access.
- When a change touches cross-pallet traits or providers, inspect both the producer and consumer sides plus the runtime wiring.

## Documentation Hygiene

- Do not hardcode pallet counts, versions, or rollout status unless they are validated against the current workspace or repo metadata.
- Prefer references to authoritative files when a detail is likely to drift.
- If a document is historical or generated from an older snapshot, label it as such instead of presenting it as live project state.

## Multi-Repo Expectations

- Identify the owning repo and downstream consumers before proposing a fix that spans core, UI, infra, GEM, Nawal, Kinich, or Pakit.
- Call out cross-repo API, RPC, workflow, or deployment implications explicitly.

## Validation

- After changes, prefer the smallest relevant check first: targeted `cargo test`, targeted `cargo check`, then wider workspace validation only when needed.
- For review-only tasks, cite the specific test or command that would prove or disprove the concern.
- If no direct validation is possible, say what evidence is missing instead of overstating confidence.

## Ops And Deployment

- Ceiba is the active self-hosted environment. Verify commands and bind addresses against the current runbook before suggesting changes.
- Treat `sudo`, `systemctl`, Docker restarts, and chain data resets as high-impact actions. State the blast radius and rollback path before recommending them.