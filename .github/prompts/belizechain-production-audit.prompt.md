---
name: "BelizeChain Production Audit"
description: "Full production-readiness audit of the belizechain core blockchain repo: runtime, pallets, node, consensus, tokenomics, CI/CD, and Ceiba deployment."
argument-hint: "Scope: full repo, a single pallet, the runtime, node, consensus, tokenomics, or CI/CD"
agent: "BelizeChain Review"
---

# BelizeChain Core Chain — Production-Readiness Audit

I need you to perform an extremely thorough, methodical, line-by-line audit of the entire
`BelizeChain/belizechain` repository and everything it directly controls. Treat this as a
formal professional engineering review and full production-readiness audit of a live
Layer 1 blockchain — not a high-level code review, quick scan, or generic checklist exercise.

The objective is to determine whether this chain is genuinely ready to hold real economic
value and real citizen data in production: runtime correctness, pallet logic, consensus and
finality, tokenomics, storage migrations, cryptography and key management, node behaviour,
configuration and genesis, CI/CD, deployment, monitoring, and operational safeguards.

## SCOPE BOUNDARY — READ THIS FIRST

**In scope:** only this repository.

- `runtime/` — `src/lib.rs`, `src/migrations.rs`, construct_runtime, config traits, weights
- `pallets/` — all 19 pallets, plus `pallets/common` and `pallets/weights.rs`
- `node/` — `service.rs`, `cli.rs`, `rpc.rs`, `chain_spec.rs`, `chain_spec_configs.rs`,
  `validator_config.rs`, `block_announce_validator.rs`, `main.rs`
- Workspace-level `Cargo.toml`, `rust-toolchain.toml`, `deny.toml`, `patches/`
- `scripts/`, `tests/`, `chain-specs/`, `generated_keys/`, `generated_specs/`, `testnet-spec.json`
- `.github/workflows/`, `.github/dependabot.yml`, `.github/hooks/`
- `Dockerfile`, `Dockerfile.runtime`, deployment manifests that ship this chain
- `docs/` **only where it makes a factual claim about runtime or node behaviour**

**Out of scope:** sibling repos (`ui`, `infra`, `kinich-quantum`, `nawal-ai`, `pakit-storage`,
`gem`) and their services. Do not audit them. Where this repo depends on them (RPC consumers,
Compose stack, contract deployment), audit only the contract boundary — ports, chain spec,
runtime compatibility, and whether this repo's claims about them are true.

## IMPORTANT: REVIEW THE ACTUAL IMPLEMENTATION

Do not assume that existing code is correct simply because it compiles, currently produces
blocks, was previously reviewed, or appears logically sound.

Do not provide generic recommendations without verifying the actual implementation.

I want you to inspect the real codebase and base your findings on what is actually implemented.

Review pallets and modules individually and trace important functionality across the entire
runtime. When a behaviour crosses pallets, hooks, origins, storage, or the node, follow the
complete flow rather than reviewing each file in isolation.

Do not skip files because they appear unimportant. `mock.rs`, `weights.rs`, `migrations.rs`,
`types.rs`, test helpers, chain-spec JSON, CI workflows, Dockerfiles, and shell scripts can
all contain production-critical problems.

If a file appears unused, determine whether it is actually unused before recommending removal.
If something appears intentionally implemented, verify whether it is complete, correct, and
appropriate for production. If something is a workaround or temporary solution, identify it
explicitly and say what it is working around.

Do not stop after identifying a handful of obvious problems. Continue until the relevant
pallets, hooks, origins, weights, migrations, configs, workflows, and scripts have been
thoroughly reviewed.

---

# 1. COMPLETE CODEBASE AUDIT

Conduct a comprehensive review of the entire repository. Produce a real inventory first, then
audit each item. Inspect, where applicable:

**Workspace and toolchain**
- `Cargo.toml` workspace members, dependency versions, feature unification
- `rust-toolchain.toml` — pinning vs floating, and what CI actually uses
- `patches/` — every patched crate (`core2`, `frame-storage-access-test-runtime`), why it
  exists, whether it is still needed, and what it changes about upstream behaviour
- `deny.toml` — advisory ignores, licence policy, ban list, and whether each documented
  justification is still accurate

**Runtime**
- `construct_runtime!` — every pallet, its index, and its `PalletId`
- Every `impl pallet_x::Config for Runtime` — associated types, constants, and origins
- Runtime constants: `BlockWeights`, `BlockLength`, `BabeEpochDuration`, session length,
  existential deposit, `RuntimeVersion` (`spec_version`, `transaction_version`)
- Origin definitions and the governance origin ladder (`EnsureRoot`, `EnsureRootWithSuccess`,
  council/technical-committee proportions, `JaguarMode`-style emergency paths)
- `runtime/src/migrations.rs` and every `OnRuntimeUpgrade` implementation
- Signed extensions, `TransactionValidity`, and fee/multiplier configuration

**Pallets (all 19)**
`belizex`, `bns`, `community`, `compliance`, `consensus`, `economy`, `governance`, `identity`,
`interoperability`, `justice`, `landledger`, `mesh`, `moderation`, `oracle`, `payroll`,
`quantum`, `staking`, `storage-proof`, `whistleblower` — plus `common` and `weights.rs`.

For each: `lib.rs`, `types.rs`, `weights.rs`, `benchmarking.rs`, `mock.rs`, tests, and
`migrations.rs` if present.

**Node**
- `service.rs` — service assembly, BABE/Grandpa wiring, session keys, telemetry, RPC build
- `cli.rs`, `main.rs` — CLI surface, `--chain` handling, dev vs testnet vs mainnet paths
- `rpc.rs` — RPC method surface, what is exposed, custom methods, unsafe/unsafe-insecure flags
- `chain_spec.rs`, `chain_spec_configs.rs` — genesis construction, endowed accounts, bootnodes
- `validator_config.rs` — validator key handling
- `block_announce_validator.rs` — fork/announce validation

**Supporting surfaces**
- `scripts/` — `generate-production-keys.sh`, `verify-wasm-hash.sh`, `verify_security.sh`,
  `bench_smoke.sh`, `quick_audit.sh`, `audit_runner.sh`, `generate-mainnet-spec.sh`,
  `run_all_tests.sh`, `start_testnet.sh`, `monitor_polkadot_sdk.sh`
- `tests/` — the Python (pytest) end-to-end, security, cross-pallet, economic, governance,
  oracle, and belizex suites, and how/where they run
- `chain-specs/`, `generated_specs/`, `generated_keys/`, root `testnet-spec.json`
- `audit_results/` including `old_audits/`, and root `dep_audit_raw.txt`
- `.github/workflows/deploy.yml`, `.github/workflows/security-audit.yml`, `.github/dependabot.yml`
- `Dockerfile`, `Dockerfile.runtime`
- `docs/` — every claim it makes about runtime behaviour, spec versions, or production status

---

# 2. LINE-BY-LINE ENGINEERING REVIEW

I want the review performed with a line-by-line mindset.

For each meaningful implementation, ask:

- What is this code supposed to do?
- Does it actually do that?
- What assumptions does it make about callers, orderers, and state?
- What happens when the expected input is missing, zero, or empty?
- What happens with invalid or adversarial input?
- What happens when a storage key does not exist?
- What happens when the caller is not the intended origin?
- What happens when the pallet's own config is misconfigured at genesis?
- What happens when another pallet it depends on is in an unexpected state?
- What happens when the block weight or PoV limit is exhausted?
- What happens when a hook runs on an empty or partially-populated block?
- What happens when governance changes a parameter mid-epoch?
- What happens when a runtime upgrade lands on state written by an older spec?
- What happens when the node restarts mid-epoch, mid-session, mid-migration?
- What happens when the node is offline across an epoch boundary?
- What happens when the same extrinsic is submitted twice?
- What happens when a migration is interrupted part-way through?
- What happens on a re-org or when a competing fork is announced?
- What happens when the WASM blob and native runtime disagree?

Do not simply identify whether code "looks good." Determine whether it is correct under
realistic production conditions on a chain that must not halt, must not corrupt state, and
must not mint value it should not mint.

---

# 3. BUGS AND LOGIC

Identify:

- Logic errors and incorrect conditions
- Incorrect assumptions about ordering, callers, or prior state
- Arithmetic errors — overflow, underflow, truncation, precision loss, rounding direction
- Misuse of `saturating_*`, `checked_*`, `wrapping_*`, and `saturating_sub` where a checked
  operation was required (or a zero fallback that silently changes economics)
- Any floating-point value anywhere in consensus-reachable logic
- Panic paths reachable from an extrinsic or a hook
- `unwrap()`, `expect()`, `panic!()`, `unreachable!()`, direct indexing, and integer division
  by a caller-controlled value on any production path
- Incorrect `ensure!` ordering — state mutated before an origin or precondition check
- Missing or wrong error variants, and errors that leak state or are unmatchable off-chain
- `on_initialize` / `on_finalize` / `on_idle` / `offchain_worker` bugs, including weight
  under-reporting and work that should be deferred
- Weight functions that do not match the work actually performed
- Missing `#[pallet::compact]`, unbounded decoding, and unbounded `Vec`/`BTreeMap` growth
- Storage item misuse: wrong prefix, wrong key derivation, key collisions, unbounded keys
- Incorrect defaults — `Default` values that create phantom balances, phantom ids, or
  phantom timestamps
- Off-by-one errors in epochs, sessions, terms, vesting, and expiry logic
- Block-number versus timestamp confusion, and any timestamp assumption that survives a
  relayer or validator with a wrong clock
- Nonce, replay, and idempotency problems
- Incorrect pagination, filtering, and iteration over `iter()` that can grow without bound
- Double counting, double spending, double slashing, and rewards paid twice
- Ordering-dependent behaviour that differs between `on_initialize` and extrinsic execution
- Partial-failure paths that leave state half-updated
- Silent failures — a `Result` discarded, an error logged and ignored, a `let _ =`
- Non-determinism of any kind, including iteration order over hash collections

Pay particular attention to code that works under normal conditions but fails under realistic
edge cases: zero values, the last unit, the first block of an epoch, the exact epoch boundary,
the maximum validator set, and the maximum number of items in a storage map.

---

# 4. SECURITY AUDIT

Perform a serious production security review of a chain that custodies real value and
processes identity, land, and compliance records.

Look for:

**Authorization and privilege**
- Missing or incorrect origin checks on any dispatchable
- `ensure_signed` where `ensure_root` was required, and the reverse
- Privilege escalation through governance, council, technical committee, or emergency paths
- Client-side-only or off-chain-only enforcement of an on-chain rule
- Unsigned extrinsics (`ensure_none`) without a proper `ValidateUnsigned` and replay protection
- Signed extensions or `TransactionValidity` logic that can be bypassed
- Any path where a non-privileged account can mutate another account's state
- Filter/multisig/proxy misuse, and `ProxyType` values that are too broad

**Asset and supply safety**
- Unauthorized mint, burn, or force-transfer paths
- Supply invariants that can be broken through any call sequence
- Escrow, vesting, staking, and treasury flows that can be drained or locked permanently
- Missing checks when an account is reaped (existential deposit, dust, locks, holds)
- `transfer_allow_death` vs `transfer_keep_alive` misuse
- Free vs reserved vs frozen balance confusion

**External trust boundaries**
- Oracle inputs a caller can influence, and price feeds used without staleness validation
- XCM / bridge / interoperability paths, and any `ExecuteXcm` or transparent origin
- Barriers, `SafeCallFilter`, and `SendXcm` configuration
- Cross-pallet calls that transitively grant more authority than the original caller held

**DoS and resource exhaustion**
- Unbounded loops, unbounded storage reads, and unbounded collection iteration
- Extrinsics that can be called repeatedly at negligible cost with heavy effect
- Storage bloat, key spam, and state growth an attacker can drive
- Weight under-reporting that allows block-weight overrun
- Panic-based DoS — any reachable panic halts block production

**Cryptography**
- Custom cryptography and post-quantum paths, hash choices, signature verification, and
  domain separation
- Randomness usage — whether randomness can be predicted, biased, or influenced by the
  block author
- Nonce/domain reuse across signing contexts

**Secrets and leakage**
- Hardcoded keys, seeds, mnemonics, or `//Alice`-style dev accounts on any production path
- Secrets in `generated_keys/`, chain specs, Dockerfiles, or committed env files
- Secrets printed to logs, exposed over RPC, or embedded in the WASM
- Panic messages or error returns that disclose internal state

**Dependency and supply chain** (detail in §16)
- Vulnerable or unmaintained crates
- `patches/` overrides that silently change upstream behaviour

Do not assume something is secure because it has an origin check. Verify that authorization
is enforced at the correct layer and that a caller cannot manipulate call data, ids, ordering,
or account choice to reach functionality they should not reach.

---

# 5. RUNTIME, STATE TRANSITION, AND STORAGE AUDIT

Treat the runtime and its storage as separate production infrastructure, audited on its own.

**Runtime topology**
- Every pallet in `construct_runtime!` — index, `PalletId`, and whether it is actually used
- Every `Config` associated type — is the chosen value correct, and is it bound to the right
  constant or the right sibling pallet?
- Type aliases (`Balance = u128`, `BlockNumber = u32`, `Nonce = u32`) — are the widths safe
  for the intended lifetime and throughput of the chain?
- Block weight and block length limits vs the pallets that consume them
- `RuntimeVersion` — `spec_version`, `transaction_version`, and `impl_version` discipline

**State**
- Storage layout, prefixes, and key design for every labelled storage item
- Storage versioning (`STORAGE_VERSION`, `StorageVersion` checks) and whether it is honest
- Critical invariants and whether they are enforced rather than merely intended
- Mutable vs immutable boundaries, and whether `get`/`take`/`mutate` is used correctly
- Cross-pallet storage reads and writes, and any hidden coupling
- Migration completeness: does every migration handle the full state space, including
  empty state, partially-migrated state, and users who have never transacted?
- Migration weight reporting and `#[cfg(feature = "try-runtime")]` pre/post hooks
- `OnRuntimeUpgrade` ordering and whether a migration can run twice

**Hooks**
- `on_initialize` / `on_finalize` / `on_idle` / `offchain_worker` for every pallet, in
  execution order, looking for ordering assumptions between pallets
- Weight consumed by hooks vs weight declared
- Whether any hook does work proportional to unbounded state

**Genesis**
- `GenesisConfig` for every pallet — defaults, build-time validation, and whether an
  invalid genesis is accepted silently
- Endowed accounts, initial supply, initial validator set, and initial governance state

**Transactions**
- `ValidateUnsigned`, `SignedExtension` implementations, and fee/multiplier behaviour
- Transaction pool assumptions: can a valid extrinsic be stuck, dropped, or replaced?

**Compatibility**
- Whether the deployed chain's spec matches this repo's `spec_version` (verify, do not assume)
- Whether older clients, frontends, or indexers break on a metadata change
- Whether any storage change is backward compatible with existing state

---

# 6. CONSENSUS AND VALIDATOR AUDIT

Treat consensus and validator operations as the highest-stakes surface, because a fault here
halts or forks the chain.

**Consensus wiring**
- BABE and Grandpa configuration, session keys, and the `impl_opaque_keys!` key set
- `BabeEpochDuration`, session length, and the relationship between them
- Epoch and session boundary behaviour, especially epoch rotation
- Genesis epoch configuration vs runtime epoch configuration
- Slots-per-epoch, session rotation, and any change to these across specs
- Whether epoch/session parameter changes are safe on a live chain

**Finality**
- Grandpa authority set rotation and session change handling
- Behavior when finality stalls, when a validator set changes, and when an authority is offline
- Whether finality is a prerequisite anywhere it should not be
- Equivocation reporting and what happens if it is missing or wrong

**Validator lifecycle**
- Registration, key rotation, chilled/active states, and nomination logic
- Slashing conditions, slashing correctness, and whether slashing can be triggered spuriously
- Reward distribution timing and whether it matches the documented schedule
- Offline/online detection, and its interaction with block production
- Validator set size limits and the behaviour at the limit

**Fork and re-org behaviour**
- `block_announce_validator.rs` — what is accepted and what is rejected
- Testnet/dev/mainnet chain spec divergence
- Behaviour when two competing blocks arrive, and when a node is offline across a rotation

**Single-node reality check**
- The chain currently runs as a single-node Docker Compose deployment. Determine explicitly
  what that means for the consensus guarantees, for finality, for slashing, and for the
  truthfulness of any documentation that implies multi-validator security today.

Cite concrete precedent where you find weaknesses. For example, verify whether the epoch
rotation path that previously stalled is fully fixed in code, in the runtime constant, in the
chain spec, and in the deployed image tag — not just in one of the four.

---

# 7. TOKENOMICS AND ECONOMIC SECURITY AUDIT

Treat the economic model as an adversarial system, not a spreadsheet.

**Supply**
- Total supply, mint authority, burn authority, and every path that creates or destroys value
- Whether supply is invariant under every reachable call sequence
- Inflation/deflation activation, and whether the schedule is actually implemented as described

**Fees and weights**
- Fee model, `Multiplier`, `TargetedFeeAdjustment`, and weights-as-prices correctness
- Whether fees can be trivially avoided, or made unboundedly expensive
- Existential deposit vs real-world minimum balances

**Staking**
- Reward calculation, multiplier logic, and reward funding source
- Slashing math, slashing destination, and whether slashing can exceed a bond
- Lock-up, unbonding, and withdrawal timing
- Whether staking rewards are sustainable given the actual issuance

**Treasury and governance economics**
- Treasury spend paths, approval thresholds, and whether a single actor can drain it
- Council/committee proportions and the realistic capture cost
- Any parameter that lets a small group redirect funds
- Emergency powers — what they can do, who holds them, and how they are constrained

**Oracle and market surfaces**
- Price feed sourcing, staleness handling, and manipulation cost
- AMM/DEX math (`belizex`) — invariant preservation, rounding direction, and who the rounding
  favours, always
- Slippage, sandwich, and MEV surfaces
- Rounding and truncation that systematically benefits the protocol or the user

**Incentives**
- Whether any behaviour is profitable when it should not be
- Whether any honest behaviour is unprofitable and therefore unlikely
- Griefing vectors that cost the attacker less than the victim loses

For every economic finding, state the attacker's cost, the victim's loss, and whether the
attack is repeatable.

---

# 8. CROSS-PALLET AND INTEROPERABILITY AUDIT

- Every cross-pallet dependency, direct call, and storage read, mapped as a graph
- Circular trust relationships and cycles where pallet A's safety depends on pallet B's
  correctness, which depends back on A
- Coupling that makes upgrading one pallet require upgrading another
- Hidden coupling through shared storage prefixes or duplicated keys
- Ordering assumptions between pallets in the same block
- XCM: configured barriers, `SendXcm`, `ExecuteXcm`, origin conversion, and fee handling
- Bridge paths: message validation, replay protection, and trust assumptions
- Whether an external message can reach a privileged call
- Whether `compliance`, `identity`, `landledger`, `justice`, and `payroll` enforce their rules
  consistently, or whether one pallet is the enforcement point and another merely assumes it
- Whether any documented cross-pallet workflow is actually wired, or only described

---

# 9. PERFORMANCE, WEIGHT, AND SCALABILITY

Analyze the chain under realistic production load, not unit tests.

Look for:

- Storage reads and writes per extrinsic, and whether weights match them
- Proof-of-validity (PoV) size, and extrinsics that could exceed block length
- `BlockWeights` and `BlockLength` vs the pallets that could realistically saturate them
- Unbounded iteration over storage maps, and maps whose size is attacker-controllable
- Repeated decoding of large structures, and decode-on-every-call patterns
- `Vec`/`BTreeMap` clones in hot paths
- Overly generous or overly pessimistic weight functions, and any that are placeholders
- Missing benchmarks, benchmarks that cannot run, and `Weight::from_parts` literals used as
  substitutes for measurement
- Removal of weight-based fees that would let an attacker execute work for free
- State growth over time: which storage items only ever grow, and what bounds them
- Block production latency and whether any hook does unbounded work
- RPC throughput, subscription counts, and whether RPC can starve block production
- Disk/DB growth on a single-host deployment

Consider what happens at 10x, 100x, and 1000x the current number of accounts, validators,
transactions, and stored records. Identify code that works at today's scale and breaks — or
becomes prohibitively expensive — at a larger one.

---

# 10. RELIABILITY, FAILURE HANDLING, AND RECOVERY

Assume the node will crash, the disk will fill, the network will partition, and a service will
hang. Test the design mentally against each.

Check for:

- Crash recovery: does the node restart cleanly mid-epoch, mid-session, mid-migration?
- Database integrity — RocksDB/ParityDB corruption does not halt the chain, only the host,
  but determine the actual recovery path and whether it is documented and tested
- Node outage across an epoch boundary, and the resulting validator/session state
- Behaviour when a migration fails part-way, and whether the chain then refuses to produce
  blocks or produces them with corrupted state
- Whether a failed runtime upgrade is recoverable without a state wipe
- Backup and restore: what is backed up, how often, whether restores are actually drilled,
  and whether a restore produces a node that rejoins the correct chain
- Chain spec drift between repo, host, and running container
- Restart ordering in the Compose stack, and whether the node depends on services that
  depend on it
- Idempotency of every operational script in `scripts/`
- Whether repeated execution of any script is safe
- Retry and timeout behaviour anywhere the node talks to an external service
- Whether failures are loud or silent — a chain that stops producing blocks without alerting
  is a production failure regardless of the code being correct

A production blockchain should fail predictably and visibly, never silently.

---

# 11. NODE, RPC, AND OFF-CHAIN AUDIT

- `service.rs` assembly: task spawning, task cancellation, and shutdown behaviour
- RPC surface: enumerate every method exposed, and classify each as safe or dangerous
- Whether unsafe/unstable RPC modules are enabled, and whether that is intended
- Whether RPC is bound to the correct interface, and whether that matches the deployment
  claim (the host binds RPC to a Tailscale address, not `127.0.0.1`)
- RPC methods that expose keys, keystore state, authoring rights, or session keys
- Whether an RPC caller can trigger authoring, `author_*`, or admin-only operations
- Whether the custom `belizechain_getChainInfo` (or equivalent) method is accurate and cannot
  leak internal state
- CLI surface: `--dev`, `--chain`, `--validator`, `--rpc-methods`, `--unsafe-*` flags and
  which of them are used in production
- Telemetry configuration and what it discloses
- Prometheus endpoint exposure and what metrics reveal
- Chain spec construction: bootnodes, protocol id, `properties` (token symbol, decimals,
  ss58 format), and whether any of these are wrong or inconsistent with the deployed chain
- Keystore and `validator_config.rs`: where keys live, file permissions, and whether keys can
  be read by an unprivileged process or a container sidecar
- Logging: level in production, whether logs can grow without bound, and whether anything
  sensitive is logged

---

# 12. CODE QUALITY AND ARCHITECTURE

Evaluate whether the architecture is appropriately engineered for a blockchain runtime.

Identify:

- Pallet boundaries that are wrong — a pallet doing two unrelated jobs, or two pallets doing
  one job
- Excessive or unnecessary abstraction, and trait layers that add nothing
- God modules and oversized files or functions
- Duplicated logic across pallets, especially duplicated validation, duplicated arithmetic
  helpers, and duplicated origin checks
- Inconsistent patterns between pallets — different error styles, different storage idioms,
  different weight idioms
- Inconsistent data models for the same real-world concept across pallets (for example,
  multiple notions of identity, account, or asset)
- Circular dependencies and dependency cycles via `common`
- Dead code, obsolete code, and legacy implementations still compiled
- Confusing control flow and misleading naming
- Comments that contradict the code
- `#[allow(...)]` and `#[cfg(feature = "std")]` usage that hides problems
- Documentation that describes an architecture the code does not implement

Do not recommend refactoring merely because another architecture might be theoretically
cleaner. Only recommend architectural changes when they materially improve correctness,
security, performance, maintainability, or operational simplicity. On a live chain, churn is
itself a risk — say so when a refactor is not worth it.

---

# 13. PRODUCTION CLEANUP

Identify anything development-only, temporary, or inconsistent with production. Search for:

- Mock data, placeholder values, and fabricated constants presented as real
- Hardcoded test accounts, dev accounts, and `//Alice`-style seeds on any path that could
  reach production
- Hardcoded addresses, URLs, ports, and chain ids that should be configurable
- Hardcoded credentials, tokens, and RPC endpoints
- `generated_keys/` and `generated_specs/` contents that should not be in a repository
- Test flags, debug flags, and development bypasses
- `--dev` assumptions leaking into non-dev code paths
- Temporary feature flags and toggles that were never resolved
- `println!` / `dbg!` / `eprintln!` debugging, and `log::error!` used as a substitute for
  returning an error
- `TODO`, `FIXME`, `HACK`, `XXX`, and `unimplemented!` / `todo!()` in production code
- Commented-out code and disabled tests (`#[ignore]`, `#[cfg(feature = "skip")]`)
- Prototype implementations and demo content
- Chain specs, audit files, and raw audit output committed at the repo root
- Stale audit artifacts in `audit_results/` and `audit_results/old_audits/` that contradict
  current code
- Unused files, unused crates, unused dependencies in `Cargo.toml`, and unused workspace
  members
- Deprecated implementations kept alive without an owner
- Documentation marked current that describes a removed or replaced system

Do not automatically classify something as junk because it looks unusual. Verify its purpose
first, and say plainly when something is historical-but-useful versus actively misleading.

---

# 14. TESTING

Evaluate test coverage and, more importantly, test quality.

Review:

- Pallet unit tests and what they actually assert
- `mock.rs` runtimes — whether the mock diverges from the real runtime in ways that hide bugs
- Origin coverage: are root-only, signed-only, and unsigned paths all tested?
- Error-path and failure-path tests
- Edge cases: zero, max, empty, boundary-of-epoch, boundary-of-set-size
- Weight and benchmark tests, and whether benchmarks actually execute in CI
- Migration tests, including `try-runtime` pre/post upgrade hooks
- Integration and end-to-end tests, including the Python suites in `tests/` — what they cover,
  whether they run in CI, and whether they can pass while the chain is broken
- Genesis and chain-spec tests
- Runtime upgrade simulation
- Consensus/validator tests
- Cross-pallet workflow tests
- Regression tests for previously fixed incidents
- Whether tests assert real invariants or merely that calls do not error
- Tests that pass because of luck — thresholds, seeds, timing, ordering
- Tests that are silently skipped

Do not judge testing by coverage percentage. Determine whether the failure modes that would
halt the chain, corrupt state, or mint value are actually tested. Explicitly list the
highest-severity behaviours that have no test.

---

# 15. DEPENDENCIES AND SUPPLY CHAIN

Audit all dependencies.

Identify:

- Crates with known advisories, and whether each `deny.toml` ignore is still justified
- Crates that are unmaintained or abandoned
- Git dependencies pinned to a branch or a moving ref instead of a commit
- Duplicated and conflicting versions of the same crate across workspace members
- Unused dependencies, and workspace members not actually referenced
- Feature unification surprises, especially features enabled only in tests but compiled into
  the runtime
- The polkadot-sdk version in use versus the version this runtime's code assumes
- Any place where a crate is patched (`patches/`) and what the patch changes
- Whether `cargo deny`, `cargo audit`, and CodeQL actually run and actually fail the build
- Supply-chain risks in build scripts, proc macros, and vendored crates
- Whether the WASM blob published for production can be reproduced from a pinned toolchain

Determine whether removing or replacing a dependency would provide a meaningful benefit.
Do not recommend dependency churn for its own sake — on a blockchain runtime, a version bump
is a consensus-affecting change and must be justified accordingly.

---

# 16. CONFIGURATION, GENESIS, AND ENVIRONMENT

Review all configuration carefully.

Check:

- Every runtime constant and its justification
- Genesis configuration per pallet and the resulting initial state
- Endowed accounts and initial supply — do they match the documented tokenomics?
- Chain spec: protocol id, bootnodes, `properties` (symbol, decimals, ss58), and genesis hash
- Whether `testnet-spec.json`, `chain-specs/`, and `generated_specs/` agree with each other
  and with the runtime's `spec_version`
- Runtime `spec_version` vs the deployed chain's spec version — verify, do not assume
- Docker and Compose configuration for the node: ports, volumes, restart policy, resource
  limits, and `--chain` arguments
- Environment variables read by the node and by the scripts, and their defaults
- Feature flags: `runtime-benchmarks`, `try-runtime`, `std`, and whether any of them can be
  enabled in a production build
- Build settings: target, `--release`, WASM builder version, and whether builds are reproducible
- RPC bind addresses, RPC method filtering, and CORS
- Timeouts, rate limits, and log configuration
- Development vs testnet vs mainnet separation, and whether a dev config can be deployed by
  accident
- Assumptions that work locally but break on the production host

---

# 17. OBSERVABILITY AND OPERATIONS

Determine whether an engineer could diagnose a production incident from what exists.

Review:

- Prometheus metrics exported, and which are missing (block height, block time, peers,
  finality lag, mempool depth, epoch/session progress, migration status)
- Whether alerts exist for: chain halted, no new blocks, finality stalled, epoch rotation
  failure, validator offline, disk filling, RPC down, memory pressure
- Grafana dashboards and whether they match the deployed metric names
- Log quality: could an engineer reconstruct what happened from the logs alone?
- Error reporting for panics, hook failures, and migration failures
- Whether a runtime panic is visible, or only observable as blocks stopping
- Health-check scripts and whether they check meaningfully rather than just port-open
- `scripts/monitor_polkadot_sdk.sh` and what it monitors
- Chain-spec and WASM-hash verification (`verify-wasm-hash.sh`) and whether it runs in CI
  or deploy
- Backup monitoring and restore verification
- Whether anyone would be told when something breaks

Identify places where failures could happen silently. A blockchain that stops producing
blocks without alerting anyone is the single most likely production failure mode here —
assess it directly.

---

# 18. RESOURCE AND COST CONTROL

This chain runs on owned hardware, so cost is measured in resources and risk, not invoices.

Look for:

- Unbounded storage growth and which pallets can be driven to grow state cheaply
- Extrinsics that are cheap to call and expensive to store — the classic state-bloat attack
- Log verbosity that grows without bound, and log rotation
- Metrics cardinality that can explode
- Disk growth rate versus available capacity and backup size
- Memory usage that grows with state or peer count
- CPU cost of unbounded hooks and of benchmark-heavy code paths in production
- RPC abuse: expensive queries that a non-privileged caller can repeat
- The resource budget for the host and what happens when a second service competes for it
  (the node shares the box with other containers)
- Any place where resource exhaustion halts block production rather than degrading a
  non-essential service

Identify functionality that could unexpectedly consume the host's resources as traffic, state,
or data grows.

---

# 19. DATA INTEGRITY AND INVARIANTS

Review how state moves through the system.

Trace important data from:

genesis/input → extrinsic → validation → origin check → state transition → storage →
events → indexers → RPC consumers → user-facing systems.

Check for:

- Missing or insufficient validation of inputs
- State corruption paths, including partial writes within a single extrinsic
- Duplicate records, phantom records, and orphaned records
- Stale data that is read as if current (prices, exchange rates, identities, compliance status)
- Race conditions between extrinsics in the same block and between hooks and extrinsics
- Incorrect updates and deletes, and missing cleanup of expired state
- Missing transactional atomicity across pallets
- Inconsistent schemas between pallets representing the same concept
- Backwards compatibility of storage changes
- Unsafe or incomplete migrations
- Invariants that are stated in comments or docs but not enforced in code
- Invariants that are enforced at call time but not preserved by a later code path

For each significant invariant you find, state where it is enforced, where it is only assumed,
and what call sequence could break it. An invariant that is merely documented is not
an invariant.

---

# 20. PRODUCTION READINESS

Evaluate the chain as if it will hold real economic value and real citizen records tomorrow.

Ask:

- Can a user or node obtain value, authority, or data they should not have?
- Can an unauthorized account perform a privileged action?
- Can supply be inflated, or funds drained?
- Can the chain halt through a reachable panic or a weight-limit overrun?
- Can consensus stall through an epoch or session rotation failure?
- Can a runtime upgrade brick the chain or corrupt state?
- Can a failed upgrade be rolled back, and is the rollback actually tested?
- Can an operator restore from backup and rejoin the correct chain?
- Does the deployed chain spec match this repository's runtime?
- Can any development key or seed reach a production chain?
- Is the single-node deployment honestly described, or does documentation imply security
  guarantees it cannot provide?
- Are there mock, placeholder, or fabricated values that would become real on mainnet?
- Do the documentation and the code agree on what is actually deployed?
- Can engineers diagnose a failure from the available logs and metrics?
- Are there hidden assumptions that break on a different host, a fresh chain, or a new validator?

---

# 21. FINDINGS MUST BE SPECIFIC

Do not give me vague statements such as:

"Improve security."

"Add better error handling."

"Review the weights."

"Improve test coverage."

Instead, identify the exact issue.

For every significant finding, provide:

1. Severity
2. File path
3. Relevant pallet / module / function / storage item / extrinsic
4. Specific location, including line range where available
5. What the code currently does
6. What is wrong
7. Why it matters
8. Realistic production impact — funds, state, liveness, or correctness
9. How the issue could actually occur, with the call sequence or trigger
10. Recommended fix
11. Whether the fix is required before production
12. Related files that must also change

Use severity categories:

- **CRITICAL** — production blocker, or a path to loss of funds, state corruption, chain halt,
  or consensus failure
- **HIGH** — significant production risk that should be addressed before launch
- **MEDIUM** — meaningful issue that should be addressed but need not block launch
- **LOW** — minor issue or maintainability concern
- **ENHANCEMENT** — useful improvement that is not a defect

Do not inflate severity. Do not call something critical because it could theoretically cause a
problem. Base severity on realistic impact and likelihood on a chain that already runs in some
form. Equally, do not downgrade a consensus, funds, identity, compliance, or bridge issue out
of caution — those are the exceptions where the highest severity belongs.

---

# 22. DISTINGUISH BUGS FROM ENHANCEMENTS

This is extremely important.

Do not turn every possible improvement into a required change.

Clearly distinguish between:

- Actual bugs
- Security vulnerabilities
- Production blockers
- Reliability and liveness problems
- Performance and weight inaccuracies
- Economic and incentive flaws
- Technical debt
- Maintainability improvements
- Optional enhancements
- Architectural alternatives
- Documentation corrections
- Personal style preferences

I do not want unnecessary rewrites. On a live blockchain, an unjustified refactor is a risk,
not an improvement. The goal is a chain that is correct, safe, and operable — not maximally
elegant.

---

# 23. DO NOT MAKE ASSUMPTIONS

If something cannot be verified from the repository, clearly identify it as unverified.

Do not claim that something is secure, insecure, production-ready, broken, or complete without
evidence from the implementation or available configuration.

If verification requires something outside the repository — the running chain's spec version,
the deployed image tag, the host's chain spec, validator keys on the server, live backup
status — say exactly what must be verified and how.

If you find uncertainty, explain the uncertainty rather than inventing an answer.

Never infer that a documented control exists because it is documented. Never infer that a test
passes because it exists. Never infer that a deployment matches the repo because the repo is
the source of truth.

---

# 24. FINAL AUDIT REPORT

At the end, provide a complete professional audit report.

## Executive Summary
A concise summary of the actual state of the chain and the most important findings.

## Critical Production Blockers
Every issue that should prevent production use.

## High-Priority Issues
Significant issues that should be resolved before or immediately around launch.

## Medium/Low Priority Issues
Remaining meaningful issues.

## Security Assessment
Origins, authorization, privilege escalation, panic paths, DoS, cryptography, key material,
secret exposure, and the security posture of the RPC surface.

## Runtime and Pallet Assessment
Runtime topology, config correctness, pallet boundaries, hook behaviour, storage design,
invariants, and runtime-version discipline.

## Consensus and Validator Assessment
BABE/Grandpa wiring, epoch and session rotation, finality, validator lifecycle, slashing,
fork handling, and what the single-node deployment actually guarantees.

## Tokenomics Assessment
Supply integrity, mint/burn authority, fees, staking, slashing, treasury, oracle, DEX math,
and incentive analysis.

## Node and Operations Assessment
Service assembly, CLI, RPC exposure, chain spec, keystore handling, logging, and recovery.

## Migration and Upgrade Assessment
Storage versioning, migration completeness, try-runtime coverage, upgrade safety, and
rollback feasibility.

## Interoperability Assessment
Cross-pallet coupling and XCM/bridge trust boundaries.

## Performance and Weight Assessment
Weight accuracy, block limits, unbounded iteration, state growth, and scalability limits.

## Reliability Assessment
Failure modes, liveness risks, crash recovery, backup/restore validity, and silent-failure risk.

## Code Quality Assessment
Architecture, duplication, dead code, documentation drift, and maintainability.

## Testing Assessment
What is well tested, what is untested, and specifically which high-severity behaviours lack
meaningful tests.

## Observability Assessment
Metrics, alerts, logging, and diagnostics gaps — including whether a halted chain would be
noticed.

## Dependency Assessment
Advisories, unmaintained crates, patched crates, feature unification, and reproducibility.

## Configuration and Genesis Assessment
Constants, genesis state, chain spec consistency, and dev/prod separation.

## Resource Assessment
State growth, storage bloat, disk and memory trajectory, and host-level contention.

## Documentation Accuracy Assessment
Every place where `docs/` claims something the code or deployment does not deliver.

## Production Readiness Checklist
A concrete checklist of everything that must be resolved or verified before production.

---

# 25. FINAL PRIORITIZED ACTION PLAN

Finish with a practical action plan organized into:

### Phase 1 — Production Blockers
Issues that must be fixed before production.

### Phase 2 — High-Priority Hardening
Important security, consensus, reliability, and operational fixes.

### Phase 3 — Cleanup
Dead code, placeholder values, stale audit artifacts, dev-only paths, unused dependencies,
duplication, and documentation drift.

### Phase 4 — Improvements
Meaningful enhancements that improve the chain without unnecessary complexity.

### Phase 5 — Final Verification
Tests, benchmarks, try-runtime checks, migration dry-runs, restore drills, and spec-versus-
deployment reconciliations that must pass before declaring the chain production-ready.

For every recommended change, prioritize by actual risk and impact rather than preference.
For every change that touches consensus, storage, or the economy, state its upgrade path and
whether it requires a runtime upgrade, a chain spec change, or a state wipe.

---

# 26. IMPORTANT FINAL REQUIREMENT

Do not stop after identifying a handful of obvious problems.

Continue examining the codebase until the pallets, runtime configuration, hooks, weights,
migrations, origins, node, chain specs, workflows, and scripts have been thoroughly reviewed.

Do not provide a superficial "looks good" conclusion.

Do not claim production readiness merely because the project compiles, because tests pass,
because the node produces blocks, or because a previous audit found nothing.

A clean `cargo build` is not proof of correctness.
A green test suite is not proof of consensus safety.
A running node is not proof of production readiness.
An existing implementation is not proof of correctness.
A prior audit is not a substitute for verifying the current code.

The purpose of this audit is to uncover problems that are not immediately obvious.

The final goal is a chain with:

- No known critical security vulnerabilities
- No known path to loss of funds, supply corruption, or state corruption
- No reachable panic on a production path
- No unbounded-weight or unbounded-storage DoS
- No incorrect or missing origin checks
- No broken invariants
- No incomplete or unsafe storage migrations
- No development keys, seeds, or bypasses reachable in production
- No mock or placeholder values that become real on mainnet
- No consensus or finality liveness risk left unexamined
- No silent failure modes on block production or epoch rotation
- No undocumented divergence between the repository and the deployed chain
- No misleading claims in documentation
- No uncontrolled state growth
- Appropriate tests for every high-severity behaviour
- Appropriate metrics, alerts, and diagnostics
- Appropriate deployment safeguards and a tested rollback path
- Appropriate backup and restore capability, verified by drill
- Clean, maintainable, appropriately engineered pallet architecture
- Reasonable complexity, with intentional non-changes explicitly documented

Most importantly, base the entire audit on the actual implementation.

I want evidence-based findings, specific file and line references, realistic impact analysis,
and actionable fixes — not generic advice.

Treat this as a formal senior-level production-readiness audit for a sovereign Layer 1
blockchain that will hold real citizen funds, real identity and land records, real compliance
obligations, and real validator stakes, and that must not halt, must not corrupt state, and
must not mint value it should not mint.

Be thorough, skeptical, methodical, and practical. Assume that hidden problems matter and that
anything capable of failing in production should be identified before it does.
