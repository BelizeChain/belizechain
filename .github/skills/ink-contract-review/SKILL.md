---
name: ink-contract-review
description: Review ink! smart contracts in the BelizeChain gem repository for storage layout, arithmetic safety, access control, PSP token semantics, reentrancy, and deployment readiness. Use when auditing or changing contracts under gem/*/lib.rs.
---

# ink! Contract Review

Review contracts in the sibling `gem` repository. Read the contract's `Cargo.toml` first:
the ink! version is a code fact and wins over any prose that says otherwise. Confirm the
toolchain and version before applying version-specific guidance.

## Read Before Reviewing

1. `gem/<contract>/Cargo.toml` — ink! and `scale` versions, features, crate type.
2. `gem/<contract>/lib.rs` — storage, constructors, messages, events, errors.
3. The contract's tests and any `gem/fuzz` target covering it.
4. `gem/docs/` and the deployment scripts for constructor arguments and network config.
5. `docs/ECOSYSTEM_STATE.md` for deployed addresses and code hashes; never invent one.

## Checklist

### Storage

- `#[ink(storage)]` fields have a deliberate layout; packed structs stay small.
- `Mapping` is used for unbounded sets; no `Vec` that can grow without a cap.
- Iteration over storage is bounded and does not run inside a message that anyone can
  call with attacker-controlled size.
- If the contract is upgradeable, storage layout changes are append-only and documented;
  otherwise state that the contract is immutable.
- No secret, key, or credential is stored on-chain.

### Arithmetic

- All balance and supply math uses `checked_*` / `saturating_*`. Overflow returns an
  error, never wraps.
- `u128 -> u64` or `Balance -> u32` conversions use `try_into` and reject overflow.
- Fees and percentages use integer math with an explicit rounding direction.
- `total_supply` equals the sum of balances after mint, burn, and transfer paths.

### Access Control

- Every privileged message checks `self.env().caller()` against the stored owner/admin.
- Ownership transfer is two-step (propose + accept); no single-call handover.
- No privileged action is reachable through a default or fallback path.
- `#[ink(payable)]` is applied only where value is expected; non-payable messages reject
  value.

### Value And Reentrancy

- State is updated before any cross-contract call or value transfer.
- `try_invoke` / `invoke` results are matched; a failed call does not leave state committed.
- Cross-contract calls are bounded; no callback loop over an attacker-sized list.
- `transferred_value()` is validated against the required amount before any state change.
- Refunds use the documented path and cannot be used to drain the contract.

### PSP Token Semantics

For PSP22 / PSP34 / PSP37 contracts:

- `transfer`, `transfer_from`, `approve`, and `allowance` match the standard's event and
  return-value requirements.
- Per-token approvals and operator approvals are distinct; a transfer approval does not
  authorize a burn.
- Batch operations enforce `MAX_BATCH_SIZE` and reject duplicate or zero-value entries
  where the standard requires it.
- Non-fungible balances stay in `{0, 1}` and `total_supply <= 1` per token ID.
- `increase_allowance` / `decrease_allowance` do not reintroduce the approve race.
- Token type is immutable after creation where the contract claims it is.
- Every state change emits the standard event (`Transfer`, `Approval`, etc.).

### Constructors And Metadata

- Constructors validate arguments (non-empty name/symbol, sane supply, owner not zero).
- Exactly one default constructor is intended; no selector collisions between messages.
- `#[ink(message)]` mutability matches the implementation (`&mut self` for state changes).
- Metadata builds without warnings.

### Build And Test

```bash
cd gem/<contract>
cargo contract build
cargo test
cargo clippy --all-targets -- -D warnings
```

If a fuzz target exists, run it for the changed entry points. For deployment changes,
confirm the constructor arguments and network URL against the deployment script and
`docs/ECOSYSTEM_STATE.md`; do not hardcode a live address in the contract or the review.

## Output

Report findings as:

- **Confirmed defect** — file:line, exploit or failure path, and the test/command that
  demonstrates it.
- **Probable risk** — file:line and the precise condition that triggers it.
- **Open question** — the standard clause or invariant that needs confirmation.

Do not apply a checklist item that the contract's ink! version does not support. Verify
the version first, and say which file you used to verify it.
