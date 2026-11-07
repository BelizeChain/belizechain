---
name: substrate-pallet-review
description: Review BelizeChain Substrate pallets and runtime wiring for determinism, weight correctness, storage compatibility, origin checks, and upgrade safety. Use when reviewing pallet code, extrinsics, storage changes, migrations, benchmarks, cross-pallet provider wiring, or runtime `construct_runtime!` changes.
---

# Substrate Pallet Review

Use this skill for a findings-first review of `pallets/**` and `runtime/**`. It reviews
Substrate correctness, not general style. Report only issues you can tie to a file, a
test, a command, or a concrete contract violation.

## Read Before Reviewing

1. The changed pallet and its `types.rs`, `weights.rs`, `benchmarking.rs`, `mock.rs`, and tests.
2. Both sides of every cross-pallet trait: producer and consumer, plus the runtime impl.
3. `runtime/src/lib.rs` — `construct_runtime!`, pallet indices, `Migrations`, and provider impls.
4. `Cargo.toml` workspace members and pallet features.
5. `docs/ECOSYSTEM_STATE.md` for live versions; never reuse a version restated in prose.

## Checklist

### Determinism

- No floating point in dispatchables, hooks, or on-chain state.
- No `std::time`, filesystem, network, thread, or host-dependent behavior.
- Randomness only from an on-chain source; no hash of caller-controlled data as a seed.
- Iteration order is explicit and bounded; no `HashMap` iteration in consensus paths.

### Dispatch And State Safety

- Origin checks are explicit: `ensure_signed` / `ensure_root` / custom origin, and the
  checked account is the account mutated.
- Validate first, mutate second. Multi-step mutations that can fail midway use
  `with_transaction` or an equivalent rollback path.
- No `unwrap()` / `expect()` in dispatchables, hooks, or provider impls. Structured
  `DispatchError` via `ensure!` or `Result`.
- Reserved balances, consumer refs, and deposits are balanced on every exit path.
- Extrinsics that can grow storage enforce `BoundedVec` / `BoundedBTreeMap` limits.

### Arithmetic

- Checked or saturating math only; `as` casts are checked (`try_into`) where truncation is possible.
- No division by a value that can be zero; percentages use integer math.
- Balance conservation: every mint/burn has an issuance or treasury counterpart.
- Fee and slash paths cannot underflow or double-charge.

### Weights And Benchmarks

- Every dispatchable declares `#[pallet::weight(T::WeightInfo::extrinsic_name())]` or a
  documented constant weight.
- The weight function exists in `weights.rs` and is generated from a benchmark, not a
  hand-set placeholder. A placeholder is a defect even if the extrinsic is not hot.
- Each branch's DB reads/writes and `proof_size` are represented; an early-return branch
  may not charge the happy-path weight if it does strictly less work.
- Benchmark setup matches the worst case (max bounded collection, max key length).
- `runtime-benchmarks` compiles:
  `cargo check -p belizechain-runtime --features runtime-benchmarks`.

### Storage And Migrations

- Adding a map is additive; re-encoding an existing key or value needs a migration.
- `STORAGE_VERSION` is bumped only when existing entries need rewriting, and a migration
  is registered in the runtime `Migrations` tuple.
- Migrations are idempotent and gated (`on_chain < N`), and handle missing keys.
- Before backfilling, verify the affected maps are empty on the target chain; do not
  assume. Record the query used.
- No unbounded `iter()` over storage in a dispatchable.

### Events And Errors

- Every state change emits an event with enough context to reconstruct it off-chain.
- Events contain no secrets and no unbounded payloads.
- Error variants are specific enough to distinguish validation, authorization, and
  arithmetic failures.

### Runtime Wiring

- Pallet indices stay stable; inserting a pallet does not renumber existing ones.
- Cross-pallet provider impls live after `construct_runtime!` and implement exactly the
  trait the consumer expects.
- Trait bounds added to `Config` are satisfied by the runtime, including benchmark and
  test runtimes.
- No new circular dependency between pallets.

### Upgrade Safety

- A forward `system.set_code` upgrade requires a strictly greater `spec_version`.
- The runtime is built with `--features testnet-fast-epoch` for the 300-slot testnet
  epoch. A blob built without it changes epoch length and halts block import.
- Boot the blob in a throwaway chain before proposing a live upgrade:
  `scripts/set-spec-code.py`, `--tmp`, then `scripts/verify_epoch_config.py`.
- Rolling back `spec_version` requires `set_code_without_checks` and does not roll back
  storage migrations. State that storage stays ahead of code after a rollback.

## Validation

Run the cheapest check that can falsify the concern:

```bash
cargo check -p pallet-belize-<name>
cargo test -p pallet-belize-<name>
cargo check -p belizechain-runtime
cargo test -p belizechain-runtime
```

For weight or migration concerns, also compile the benchmark feature. If a check cannot
run, say what evidence is missing rather than asserting the code is safe.

## Output

Separate findings into:

- **Confirmed defect** — file:line, trigger, impact, and the command or test that proves it.
- **Probable risk** — file:line and the condition under which it becomes a defect.
- **Open question** — the specific invariant or upstream behavior that must be checked.

Do not recommend a re-genesis to fix a pallet defect that a runtime upgrade can fix.
Do not restate live spec versions, heights, or hashes from memory; read
`docs/ECOSYSTEM_STATE.md` first.
