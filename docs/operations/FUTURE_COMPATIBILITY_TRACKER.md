# Future Compatibility Tracker

Status: Open
Updated: 2026-04-21

## Current Finding

Running:

cargo report future-incompatibilities --id 1

Reported:
- Affected crate: trie-db 0.30.0
- Newer version available: 0.31.0
- Risk: never type fallback behavior becomes a hard error in Rust 2024/future release

## Impact

- Current workspace check/build succeeds.
- Future Rust/toolchain updates may break compilation until dependency graph is updated.

## Recommended Action

1. Attempt dependency update path to trie-db 0.31.0 through Polkadot SDK dependency set.
2. If blocked by transitive constraints, open tracking issue with explicit upstream dependency path.
3. Re-run:
   - cargo check --workspace --all-targets
   - cargo report future-incompatibilities --id 1
4. Keep this file updated with status and owner.

## Command Output Summary

- cargo reports trie-db 0.30.0 as future-incompatible.
- Suggested fix includes explicit type annotations upstream and/or updating crate version.
- Upstream repository: https://github.com/paritytech/trie
