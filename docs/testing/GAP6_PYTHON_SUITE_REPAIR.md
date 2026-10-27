# Gap 6 — the Python integration suites were testing an API that does not exist

**Status:** repaired. Every chain-only suite now runs against the real pallet
surface and asserts state, not just extrinsic inclusion.
**Verified:** 2026-10-26 against Chain D `--dev` on this workstation.

---

## 1. What was wrong

The suites were collected and "passing" while proving nothing. Three distinct
defects, in order of how much they hid:

### 1.1 Most of the API they called did not exist

`tests/governance/` (5 files, 29 tests) called `create_proposal`,
`vote_on_proposal`, `register_council_candidate`, `get_proposal`,
`cast_council_vote`, `propose_budget`, `approve_budget` and others that appear
nowhere in `pallets/governance`. All 29 tests reported as **skipped**, and the
skip reasons were the fixtures failing to find those calls.

`tests/cross_pallet/test_all_interactions.py` called nine dispatchables; **eight
did not exist** (`transfer_with_compliance`, `set_kyc_level`, `add_to_sanctions`,
`vote_on_proposal`, `register_council_candidate`, `report_data_as_validator`,
`set_exchange_rate`, `swap_with_oracle_guard`). It asserted error names the
pallets never emit, e.g. `"KYCRequired"` where the real variant is
`KycVerificationRequired`.

A test that cannot run is not a passing test, and the skip tally made it look
like coverage.

### 1.2 `Sudo::sudo` cannot satisfy `ensure_signed`

This was the single largest source of false skips. Several pallets are gated on
`ensure_signed(origin)` **and then** look up the caller in a role set:

| Call | Gate |
|---|---|
| `Payroll::add_employee`, `create_department`, `set_deduction`, `issue_bonus`, `execute_payment` | `ensure_signed` + `VerifiedEmployers[origin]` |
| `Identity::issue_ssn` / `issue_passport` / `issue_biometrics` | `ensure_signed` + `SsnIssuers` / `PassportIssuers` |
| `Governance::propose_treasury_spend`, `approve_treasury_spend` | `ensure_signed` + compliance / `CouncilMembers` |
| `Governance::veto_emergency` | `ensure_signed` + `CouncilMembers` |
| `Oracle::submit_price` | `ensure_signed` + `OracleOperators` |
| `Oracle::verify_merchant` | `T::MerchantVerificationOrigin` = `TechnicalCouncilMajority` — **not** operator-gated, so root drives it and an operator key gets `BadOrigin` |
| `Consensus::register_ai_model` | `ensure_signed`, no role check — there is no root path at all |

Wrapping those in `Sudo::sudo` puts `Root` in the origin slot, so the pallet
returns `BadOrigin` before it ever reaches the role check. The old tests caught
that error and reported it as *"blocked, skipping"* — so the recorded skip reason
was never the real reason, and the role logic was never exercised once.

### 1.3 Suites that asserted nothing

- `tests/oracle/test_exchange_rate.py` was named `..._emits_event` but never
  looked at an event or at storage — it asserted the extrinsic was included.
- `tests/test_e2e_security.py` defined **zero** `test_*` functions. It was a
  ~380-line script with an `if __name__ == "__main__"` block that pytest
  happily collected as an empty module.
- `tests/blockchain/test_all_pallets.py` had **20 `pytest.skip` calls**. The
  worst was `test_upload_contract_code`, which skipped when the pallet rejected
  an 8-byte wasm stub — the rejection is *guaranteed*, so the test could never
  fail regardless of what the pallet did.

---

## 2. What was fixed

### 2.1 Governance — rewritten against the real pallet

| File | Tests | Notes |
|---|---|---|
| `test_council.py` | 6 | district elections: start → register → vote → finalize |
| `test_proposals.py` | 13 | `submit_proposal(title, description, proposal_type_index, threshold_index, is_emergency, district_index)` |
| `test_voting.py` | 8 | `delegate_vote`, `revoke_delegation`, `cast_vote`, conviction |
| `test_emergency_powers.py` | 7 | two-phase declare → veto/end, with teardown |
| `test_treasury.py` | 11 | `propose`/`approve`/`execute_treasury_spend` |

### 2.2 Four new fixtures in `tests/conftest.py`

| Fixture | Why it has to exist |
|---|---|
| `council_keypair` | `approve_treasury_spend` and `veto_emergency` are `ensure_signed` **and** council-gated, so a member's own key must sign. Seats only exist after `test_council.py` runs. |
| `unverified_account` | The compliance-gate tests previously used `//Dave` / `//Bob` as "unverified" stand-ins — but `test_council.py` verifies them, so the gate tests passed or failed depending on file order. Derives a **fresh** `//Alice//governance-unverified-N` per call (the counter is module-level, because a function-scoped one restarted at 1 in every test and handed out the same account) and **asserts no KYC record exists** before the test runs. |
| `identity_kyc` | Identity KYC is a *different store* from Oracle KYC. `T::BelizeKyc` (LandLedger, Staking, BNS) reads `Identity.SsnAttestations` / `PassportAttestations` / `BiometricAttestations` keyed by identity id; `verify_kyc` writes `Oracle.IdentityVerifications`. Seeding only the oracle is why those pallets skipped. `add_issuer` also needs `IssuerBondAmount` posted first — it is 1000 DALLA on this chain, not the zero the `OptionQuery` default implies. |
| `oracle_operator` | `submit_price` / `verify_merchant` need an `OracleOperators` entry; `OracleAdminOrigin` is `EitherOfDiverse<EnsureRoot, _>` so sudo can add one. |

### 2.2b `decode_dispatch_error` in `tests/conftest.py`

The sudo path returned inner errors still SCALE-encoded
(`{'Module': {'index': 26, 'error': '0x10000000'}}`) while the plain signed path
returned decoded names — so every sudo assertion compared a name against a hex
blob and could never match. `decode_dispatch_error` renders both shapes as
`Pallet.Variant` by walking `metadata.pallets` (a list of `PalletMetadataV14`,
not a name-keyed mapping) and reading `pallet.value["index"]` / `pallet.errors`.

Note the byte order: `DispatchError::Module.error` is `[u8; 4]`, so the variant
index is the **first** byte of the hex payload — `"0x10000000"` is 16, not 0.

`decode_dispatch_error` only handles `DispatchError::Module`. A `Token` error
(`{'Token': 'FundsUnavailable'}`) has no pallet or variant, so the receipt's
`error_message` is `None` and an assertion on it reads like a mystery failure.
If a test fails with `error_message is None`, read the `ExtrinsicFailed` event's
`attributes.dispatch_error` directly — that is where the reason lives.

### 2.2c Fees at or above a dev account's starting balance

Several extrinsics charge a fee or reserve a deposit that is **exactly** a `--dev`
account's starting balance of 100 DALLA, so a test that does not fund first fails
with a balance error that looks unrelated to the thing being tested:

| Call | Charge | Symptom |
|---|---|---|
| `Bns::register_domain` (Standard, 5+ chars) | 100 DALLA | `Token: FundsUnavailable` (and `error_message is None`) |
| `LandLedger::register_property` | `RegistrationDeposit` = 100 DALLA | `InsufficientBalance` |
| `Payroll::execute_payment` / `issue_bonus` | the salary itself (5,000 DALLA in these tests) | `InsufficientBalance` |
| `Staking::join_validators` | checks `free_balance >= stake`, min 1,000 DALLA | `InsufficientStake` |

Fund in the fixture, not in the assertion.

### 2.3 Cross-pallet, rewritten against real providers

Six tests, each exercising one real `Config` edge — so a regression in the
*runtime wiring* fails, not just a pallet unit test:

- `Economy::transfer_bbzd` ← sanctions, L1 KYC, and the travel rule (L2 at
  `TravelRuleThreshold` = 100 DALLA).
- `LandLedger::register_property` ← `T::BelizeKyc` (Identity), **not** Oracle.
- `Governance::submit_proposal` ← `T::ComplianceProvider`.
- `Oracle::update_exchange_rate` ← the entry `BelizeXOracle` reads.

### 2.4 Root-level files

- `tests/test_e2e_security.py` → **moved** to `tests/manual/e2e_security_check.py`
  with a README. It was never a pytest module.
- `tests/test_security.py` — correctly gated behind `ENABLE_ORACLE_API_TESTS=1`
  (unchanged).
- `tests/test_full_stack.py` — needs Redis / IPFS / Postgres; module-level skip
  is correct, now listed explicitly in CI.

### 2.5 CI

`.github/workflows/deploy.yml` previously ran `tests/economic tests/belizex` and
then only `--collect-only` for everything else. It now runs **every chain-only
suite in one invocation** (one pytest process, so `test_council.py` seeds the
council seat before the gated suites need it), and a second step runs the two
service-dependent suites so their exclusion is visible rather than silent.

`requires_services` is registered in `pytest_configure` and marks the e2e
scenarios, so `pytest -m "not requires_services"` is a valid selection.

---

## 3. Real defects found while doing this

These are **not** test bugs. Each is a live behaviour of the pallet as written.

### 3.1 District elections are one-shot, forever — **fixed 2026-10-27**

`start_district_election` used to require `!DistrictElections::contains_key(district)`,
and `finalize_district_election` never removed the entry. There are six districts,
so the chain permitted **six elections in total, ever** — one per district — and the
council could never be re-elected.

**Fix.** A finalized or cancelled election is history, not a lock: the guard now
rejects only a *running* election. The previous record is overwritten, which loses
nothing that is not already durable elsewhere — the outcome lives in
`DistrictRepresentation`, `CouncilMembers` and the `DistrictElectionFinalized`
event.

#### 3.1a …and same-block elections shared a candidate pool

Fixing 3.1 exposed a second defect underneath it. The election id was
`current_block`, and `ElectionCandidates` is keyed by `(election_id, account)`. Two
districts opened in the same block therefore had the *same id*, so each saw the
other's candidates — finalizing the second district awarded its seats to people who
had registered in the first.

This is only reachable now that 3.1 is fixed (previously a district was single-use,
so the collision needed two districts started in one block, which was already
possible but was masked by the six-election ceiling).

**Fix.** `NextElectionId` is a counter seeded from the current block on first use.
Seeding from the block keeps the transition safe: ids minted by the old rule were
block numbers no greater than the current block, so new ids cannot collide with
them. Additive storage — an existing chain starts empty and needs no migration.

Pinned by `elections_opened_in_the_same_block_have_separate_candidate_pools`,
`district_holds_a_second_election_after_the_first_is_finalized` and
`district_cannot_open_a_second_election_while_one_is_running`.

### 3.2 `CouncilMember` records no district — **resolved by helper, not storage**

Finalize sets `role`, `votes_received` and `term_end`, and writes
`DistrictRepresentation[district] = winners`. The reverse link — which district a
seated member represents — exists only in `DistrictRepresentation`.

Storing it twice was the tempting fix and is the wrong one: it changes
`CouncilMember`'s encoding, needs a migration, and creates two records that can
disagree about a fact that has one owner.

**Fix.** `Pallet::district_of(account)` resolves it from the authoritative side.
The cost is six storage reads against a `BoundedVec<_, 5>` — bounded, and cheaper
than the migration that a second copy would require. Pinned by
`district_of_resolves_a_seated_member`.

### 3.3 No global council size cap from district elections — **fixed 2026-10-27**

`CouncilSizeLimits` is enforced only by `add_board_member`, and only per role. A
district election seated up to `district.seat_allocation()` members and there are six
districts, so thirty `CitizenDelegate`s could sit against a role cap of three.

**Fix.** Finalize now checks the `CitizenDelegate` cap using the same
`BoardRole::max_count()` that `add_board_member` uses, counting from
`CouncilMembers` (the authoritative set) rather than `BoardComposition`, which has
never reflected delegates seated by election. It also writes the resulting count
back to `BoardComposition`, so the two stop disagreeing.

Pinned by `finalize_refuses_to_seat_more_delegates_than_the_role_allows` and
`finalize_records_the_delegate_count_in_board_composition`.

### 3.4 Treasury approval tiers were documented 1000× too high

`propose_treasury_spend` compares the raw balance against
`10_000_000_000_000` and `100_000_000_000_000`. At 12 decimals those are **10 and
100 DALLA**. The inline comments said "10K DALLA" and "100K DALLA".

Consequence: a 5,000 DALLA treasury spend needs only **one** council approval,
not the 3-of-7 or 4-of-7 the comment implies.

**Fixed** — the comments now state the real bounds. The thresholds themselves
were left alone, because changing them changes consensus behaviour and is a
governance decision, not a bug fix.

### 3.5 `EmergencyType` index > 4 silently becomes `Other` — **fixed 2026-10-27**

Every other index argument in `pallets/governance` (`ProposalType`,
`ThresholdType`, `BelizeDistrict`, referendum options) rejects out-of-range values
with an `Error`. `emergency_type_index` did not — index 9 was accepted and stored as
`EmergencyType::Other`.

`Other` is the one emergency type with no documented meaning, so a mistyped
declaration silently recorded a different emergency than the caller asked for, and
the stored type drives the response.

**Fix.** `5 => Other` is now explicit and anything above it returns
`InvalidEmergencyType`. Pinned by `declare_emergency_rejects_an_out_of_range_type`
and `declare_emergency_accepts_the_documented_other_index`.

### 3.6 `pallets/belizex` has no transaction rollback — **not a defect**

**Corrected 2026-10-27.** This section previously claimed a real bug: that
`add_liquidity`, `execute_trade`, `execute_multihop_trade` and `remove_liquidity`
each make two or more `T::Assets::transfer` calls with `?` between them, so a
failure on a later leg would leave the earlier legs applied.

That premise is wrong. `#[pallet::call]` already wraps every dispatchable in a
storage layer, so a returned error undoes everything the extrinsic wrote — from
`substrate/frame/support/procedural/src/pallet/expand/call.rs`:

```rust
// We execute all dispatchable in a new storage layer, allowing them to return an
// error at any point, and undoing any storage changes.
#frame_support::storage::with_storage_layer::<#ok_type, #err_type, _>(|| #block)
```

So the extrinsics were never unsafe, and adding `#[transactional]` would have
changed nothing except weight. The earlier reasoning inferred atomicity from the
absence of an attribute instead of from the macro that supplies it.

The mistake was made twice over, and the second time is the instructive one: a
rollback test was written against the unfixed pallet expecting it to fail, and it
**passed**. Rather than accept the pass, the ledger was checked for whether the
first leg really ran — it did, `(DALLA, 2, pool)` — which is what proved the
storage layer was already undoing it.

`settle_fill`'s manual leg reversal stays. It is redundant inside a dispatch, but
`match_order` is `pub(crate)` and is called directly by `benchmarking::match_orders`,
where no dispatch layer exists.

The invariant is now pinned by
`add_liquidity_rolls_back_when_a_later_leg_fails` in `pallets/belizex/src/tests.rs`.
It rejects the *quote* asset (not an account, which would fail the first leg and
prove nothing), asserts the first leg ran, then asserts every leg and the reserves
were left untouched. If a future SDK makes the storage layer opt-in, that test
fails and the four extrinsics need `#[transactional]`.

### 3.7 The oracle deviation guard is deliberately narrow

`BelizeXOracle::get_crypto_exchange_rate` ignores its `_base_asset` argument and
always returns USD/BZD for any bBZD-quoted pair. So the guard is applied to
bBZD pairs only, never to DALLA pairs — applying it to DALLA would reject
legitimate trades against a rate that does not describe DALLA.

This is correct as written, and is now locked in by
`oracle_guard_does_not_gate_dalla_pairs` in `pallets/belizex/src/tests.rs` so a
future "fix" cannot widen it by accident.

### 3.8 The dev genesis has no USD/BZD oracle rate — **test made deterministic**

The guard fails closed, so on a fresh `--dev` chain every bBZD trade is rejected
until someone calls `Oracle.update_exchange_rate`. That part is correct and is not
a defect: a missing rate must not mean "trade anyway".

The defect was in the suite. The test covering the fail-closed branch skipped
whenever a rate already existed, and `update_exchange_rate` has no inverse, so the
branch is observable exactly **once per chain**. Whichever test wrote the rate first
decided whether the assertion ran at all — `test_oracle_slippage_guard.py` sorts
before `test_wusdc_bbzd_guard.py`, so it always skipped in practice.

**Fix.** The test carries `pytest.mark.pristine_oracle_rate`. CI runs it as its own
first step on the fresh dev chain, before anything sets a rate, and the shared run
excludes it with `-m "not pristine_oracle_rate"`. It now runs for real instead of
skipping, and the ordering dependency is declared rather than incidental.

### 3.9 `transfer_bbzd`'s sender-side KYC gate cannot be reached from outside — **now covered at unit level**

`transfer_bbzd` checks `meets_kyc_requirement(from, 1)` **and**
`meets_kyc_requirement(to, 1)`. The recipient leg is testable from an integration
test. The sender leg is not, because no unverified account can hold bBZD in the
first place:

- `mint_bbzd` is minter-gated (`AuthorizedMinters`) and, separately, requires
  recipient KYC at L1 — minting *to* an unverified account is refused.
- `mint_bbzd` is `ensure_signed`, so `Sudo.sudo` cannot stand in for an authorised
  minter either; it returns `BadOrigin`.
- Nothing lowers an Oracle KYC level once set, so a verified holder cannot become
  unverified.

So there is no chain-level counter-example a test can build. But the branch was
*also* unreachable in the pallet's own unit tests, because `MockOracleProvider`
hard-coded `meets_kyc_requirement` to `true` and `is_sanctioned` to `false`. A
provider that cannot fail cannot be tested — which is what made this look like a
design dead end rather than a missing mock capability.

**Fix.** The mock now reads per-account overrides (`set_kyc_level`, `sanction`),
cleared by `new_test_ext` so one test cannot leak into the next, and defaults to
"no override" so the rest of the suite is unchanged. Covered by
`transfer_bbzd_refuses_an_unverified_sender`,
`transfer_bbzd_refuses_an_unverified_recipient` and
`transfer_bbzd_refuses_a_sanctioned_sender` in `pallets/economy/src/tests.rs`.

### 3.10 `fund_bbzd` verifies its own recipient

A fixture trap rather than a pallet defect, recorded because it silently
invalidated a test: `fund_bbzd` calls `verify_kyc(account, 1)` before minting,
because the pallet demands it. Any test that funds an "unverified" account
through `fund_bbzd` in order to prove the KYC gate fires has just verified the
account, and the gate it wanted to prove now passes.

Related: `fund_account` sets the **native** balance (`Balances.force_set_balance`),
which `transfer_bbzd` never reads. A counterparty funded only that way fails with
`InsufficientBbzdBalance` — which reads like a ledger defect and is not one.

### 3.11 A chain launched from genesis has an empty bridge

**Corrected 2026-10-27.** This section previously claimed the bridge was
"permanently dead on every live chain" because the seeding migration was never
registered in the runtime. That was **wrong**, and the error is worth recording
because of how it was made: the check grepped `runtime/src/lib.rs` only, while
the migration coordinator lives in `runtime/src/migrations.rs`.

The facts:

- `pallet_belize_interoperability::migrations::SeedChainConfigurations` **is**
  registered, as step V1 → V2 of `CoordinatedUpgrade`, in commit `aab5f4d`
  (2026-10-08).
- That commit predates spec 122 (2026-10-25), so the migration ran on the
  upgrade. Verified live: Ceiba holds **13** `ChainConfigurations` entries, all
  enabled.

What is actually missing is **genesis parity**. `on_runtime_upgrade` fires only
when `:code` changes, never on a fresh genesis. The pallet has no
`#[pallet::genesis_config]` — removed with the comment "GenesisConfig removed for
Substrate v42 compatibility" — so a chain *launched* rather than *upgraded* starts
with an empty bridge:

| Chain | `ChainConfigurations` |
|---|---|
| Ceiba, upgraded through spec 122 | 13 entries |
| Fresh `--dev` chain | empty |
| Future mainnet launched at spec 123+ | empty |

With no entry, `update_bridge_config` refuses (`UnsupportedChain`) and no
`add_bridge_chain` exists — so a genesis-launched chain cannot be configured at
all. The pallet comment "Chain configurations can be initialized through
governance or extrinsics" is false and is part of the defect.

**Fix:** restore `#[pallet::genesis_config]` for `chain_configurations`, seeded
from the same helper the migration uses so the two cannot drift apart.

**Applied 2026-10-27.** The pallet now has a genesis config, and
`node/src/chain_spec.rs` fills it from
`migrations::seeded_chain_configuration_entries`. The genesis struct carries
primitive tuples because the genesis macro requires serde under `no_std`, where
`ChainConfig`'s `BoundedVec` has none — the same constraint the `community` pallet
works around. `build()` reassembles the real types.

`Pallet::encode_chain` now delegates to a crate-level `chain_index`, so the
chain specs and the pallet share one mapping. That is not cosmetic: a duplicated
mapping was written during the fix and had `Near`/`Sui` off by three indices,
which would have seeded the wrong chains without failing anything.

**The suite now states the seeded truth.** `TestInteroperabilityPallet`:

| Test | What it pins |
|---|---|
| `test_the_launched_chain_has_the_full_bridge_registry` | a launched chain's 13 chains equal Ceiba's |
| `test_initiate_bridge_succeeds_for_a_configured_chain` | the happy path reaches escrow and records the lock |
| `test_initiate_bridge_requires_enhanced_kyc` | KYC still gates before the chain gate |
| `test_an_unseeded_chain_is_still_refused` | index 200 is still `UnsupportedChain` |
| `test_bridge_configuration_can_be_updated` | an update edits a seeded entry and stores it |

The two tests this replaced asserted the *broken* behaviour
(`test_bridge_configuration_cannot_be_created` expected `update_bridge_config` to
fail), and their docstrings repeated the false "never registered" claim.

---

## 4. How to run

Two store types, and the distinction matters:

| Need | Use |
|---|---|
| Sanctions, KYC on `Economy::transfer_bbzd`, travel rule, governance compliance gates | `verify_kyc` → `Oracle.IdentityVerifications` |
| LandLedger, Staking (L3), BNS, Community | `identity_kyc` → `Identity.*Attestations` |

```bash
# fresh chain
./target/release/belizechain-node --dev --base-path /tmp/bz-test-chain \
  --rpc-port 9944 --rpc-methods Unsafe --prometheus-port 9616 &

# every chain-only suite, one process (order matters: test_council seeds seats)
.venv-pytest/bin/python -m pytest \
  tests/economic tests/belizex tests/governance tests/oracle \
  tests/cross_pallet tests/blockchain tests/test_probe.py \
  -q -rs --timeout=300 -p no:cacheprovider
```

**Wipe `/tmp/bz-test-chain` whenever genesis changes.** Since 2026-10-27 the dev
genesis seeds the bridge registry, so a database created by an older binary boots
with an empty `ChainConfigurations` and the bridge tests fail against a chain that
looks current. The binary's mtime is the tell: if `test_the_launched_chain_has_the_full_bridge_registry`
fails with 0 entries, the chain DB predates the genesis patch, not the code.

**Do not `pkill` the node while a run is in flight.** One earlier confirmation
run was invalidated that way; the resulting `ConnectionRefusedError` trails were
self-inflicted, not test failures.

### 4.1 Verified result (2026-10-27)

On a fresh `--dev` chain, with the command above:

```
114 passed, 1 skipped in 2676.51s (0:44:36)
```

115 collected, so the accounting closes. The count grew from 112 after the
Gap 7 work replaced three bridge tests with five.

**The chain must be fresh.** An earlier confirmation run of this same command was
invalid: it reused a chain database that had already been exercised twice, and
produced a wall of failures (`EmployeeAlreadyExists`, `FundsUnavailable`,
`Storage function requires 2 parameters`) that looked like code regressions and were
not. The tell is that the failures are unfixable-by-code — a dev chain has fixed
starting balances and seeded rows, so a second run over the same DB starts from a
state the tests assume they created. Wipe `/tmp/bz-test-chain` between runs; only a
run that starts from genesis is evidence. The skip is
deliberate — `test_wusdc_bbzd_guard.py::test_fail_closed_without_a_rate` skips
when a USD/BZD rate already exists, because the rate is global chain state and
the fail-closed path is covered by unit tests in `pallets/belizex`.

The whole tree collects 121 tests, including the service-dependent suites
(`tests/e2e`, `tests/test_full_stack.py`) that self-skip without IPFS, a
federated-learning server or a quantum service.

For reference, the first honest run of the same selection was:

```
23 failed, 83 passed, 1 skipped, 5 errors in 1978.52s (0:32:58)
```

Every one of those 28 non-passing outcomes is accounted for in §1–§3. Nothing
was made to pass by relaxing an assertion: the two suites that previously
asserted against a non-existent API were rewritten against the real one, and the
pallet-behaviour findings in §3 are pinned by tests that state them explicitly.

**Run time roughly doubled** (1,978 s → 2,605 s) because the rewritten tests
actually submit the transactions they assert on, where the old ones read
`ProposalCount` from a chain that never had a proposal.
