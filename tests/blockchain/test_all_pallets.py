"""
BelizeChain Blockchain Core Tests: All Custom Pallets

Tests aligned with actual runtime API (spec_version=104).
Each test calls real pallet extrinsics with correct parameter names and types.

Custom Pallets tested:
1. Economy - bBZD minting/redeem, DALLA burns, tourism payments
2. Identity - register_identity, issue_ssn, issue_passport
3. Governance - submit_proposal, cast_vote, declare_emergency, treasury
4. Compliance - verify_account, restrict, sanctions
5. Staking - join_validators, distribute_rewards
6. Oracle - submit_price, verify_merchant, add_operator
7. Payroll - verify_employer, add_employee, departments, deductions, bonuses
8. Interoperability - initiate_bridge, update_bridge_config
9. BelizeX - create_trading_pair, add_liquidity
10. LandLedger - register_property, transfer_property
11. Consensus - register_ai_model, submit_ai_work
12. Quantum - submit_quantum_job
13. Community - submit_community_proposal, record_participation
14. BNS - register_domain, set_resolution
15. Contracts - upload_code
"""

import hashlib

import pytest
from substrateinterface import Keypair

from conftest import same_account


# ============================================================================
# 1. ECONOMY PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_economy
class TestEconomyPallet:
    """Test Economy pallet: bBZD stablecoin, DALLA burns, tourism."""

    def test_dalla_transfer(self, blockchain_connection, alice_keypair, bob_keypair, submit_extrinsic, query_storage):
        """Test DALLA token transfers between accounts via Balances.transfer_keep_alive."""
        amount = 1_000_000_000_000  # 1 DALLA (12 decimals)

        receipt = submit_extrinsic(
            "Balances",
            "transfer_keep_alive",
            {"dest": bob_keypair.ss58_address, "value": amount},
            alice_keypair
        )

        assert receipt.is_success, f"Transfer failed: {receipt.error_message}"

    def test_bbzd_mint(self, blockchain_connection, alice_keypair, fund_bbzd):
        """`mint_bbzd` is minter-gated, so it cannot be driven by sudo.

        It is `ensure_signed` and checks `AuthorizedMinters[origin]`, then requires
        collateralized reserves — which is exactly what `fund_bbzd` sets up.
        """
        before = blockchain_connection.query(
            "Economy", "BBZDBalances", [alice_keypair.ss58_address]
        ).value or 0
        # Kept below `TravelRuleThreshold` (100 DALLA): above it the mint
        # itself demands Enhanced KYC (L2) on the recipient.
        amount = 50 * 1_000_000_000_000

        fund_bbzd(alice_keypair, amount)

        after = blockchain_connection.query(
            "Economy", "BBZDBalances", [alice_keypair.ss58_address]
        ).value or 0
        assert int(after) >= int(before) + amount, (
            f"bBZD balance did not increase by {amount}: {before} -> {after}"
        )

    def test_burn_dalla(self, blockchain_connection, alice_keypair, submit_extrinsic):
        """Test voluntary DALLA burn via Economy.burn_dalla."""
        amount = 1_000_000_000  # small amount

        receipt = submit_extrinsic(
            "Economy",
            "burn_dalla",
            {"amount": amount},
            alice_keypair
        )

        assert receipt.is_success, f"DALLA burn failed: {receipt.error_message}"

    @pytest.mark.timeout(120)
    def test_transfer_bbzd(
        self, blockchain_connection, alice_keypair, bob_keypair, submit_extrinsic,
        verify_kyc, fund_bbzd,
    ):
        """A verified sender moves bBZD to a verified recipient.

        Both sides need L1 KYC, and the amount must stay below
        `TravelRuleThreshold` (100 DALLA) or the sender needs L2.
        """
        verify_kyc(alice_keypair, 1)
        verify_kyc(bob_keypair, 1)
        fund_bbzd(alice_keypair, 10 * 1_000_000_000_000)

        receipt = submit_extrinsic(
            "Economy",
            "transfer_bbzd",
            {"to": bob_keypair.ss58_address, "amount": 100},
            alice_keypair,
        )
        assert receipt.is_success, f"transfer_bbzd failed: {receipt.error_message}"

        credited = blockchain_connection.query(
            "Economy", "BBZDBalances", [bob_keypair.ss58_address]
        ).value
        assert int(credited or 0) >= 100, (
            f"recipient bBZD balance is {credited}, expected at least 100"
        )


# ============================================================================
# 2. IDENTITY PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_identity
class TestIdentityPallet:
    """Test Identity pallet: registration, SSN issuance, passport issuance.

    `issue_ssn` / `issue_passport` are `ensure_signed` and check the caller
    against `SsnIssuers` / `PassportIssuers`. Sudo therefore returns `BadOrigin`,
    which the previous version of this class reported as "requires authorized
    issuer" — the issuer was never authorised, so the skip was really a failure.
    `identity_kyc` does the authorisation.
    """

    def test_register_identity(self, submit_extrinsic, fund_account):
        """A fresh account can register a BelizeID."""
        applicant = Keypair.create_from_uri("//Alice//identity-applicant")
        fund_account(applicant, 100 * 1_000_000_000_000)

        receipt = submit_extrinsic(
            "Identity", "register_identity", {"name": "Identity Applicant"}, applicant
        )
        message = str(receipt.error_message)
        assert receipt.is_success or "IdentityExists" in message, (
            f"register_identity failed: {message}"
        )

    def test_registration_is_keyed_by_account(
        self, submit_extrinsic, blockchain_connection, identity_kyc, bob_keypair
    ):
        """`IdentityOf` maps the account to an identity id."""
        identity_kyc(bob_keypair, 1)

        identity_id = blockchain_connection.query(
            "Identity", "IdentityOf", [bob_keypair.ss58_address]
        ).value
        assert identity_id is not None, "no IdentityOf entry after registration"

        record = blockchain_connection.query(
            "Identity", "Identities", [int(identity_id)]
        ).value
        assert record is not None, "IdentityOf points at a missing Identities record"

    def test_issue_ssn_reaches_an_issuer(self, identity_kyc, blockchain_connection, bob_keypair):
        """An authorised issuer writes an SSN attestation."""
        identity_kyc(bob_keypair, 1)

        identity_id = blockchain_connection.query(
            "Identity", "IdentityOf", [bob_keypair.ss58_address]
        ).value
        attestation = blockchain_connection.query(
            "Identity", "SsnAttestations", [int(identity_id)]
        ).value
        assert attestation is not None, "issue_ssn wrote no SsnAttestations entry"
        assert attestation["status"] == "Active"

    def test_issue_passport_reaches_an_issuer(self, identity_kyc, blockchain_connection, bob_keypair):
        """An authorised issuer writes a passport attestation (L2)."""
        identity_kyc(bob_keypair, 2)

        identity_id = blockchain_connection.query(
            "Identity", "IdentityOf", [bob_keypair.ss58_address]
        ).value
        attestation = blockchain_connection.query(
            "Identity", "PassportAttestations", [int(identity_id)]
        ).value
        assert attestation is not None, "issue_passport wrote no PassportAttestations"
        assert attestation["status"] == "Active"


# ============================================================================
# 3. GOVERNANCE PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_governance
class TestGovernancePallet:
    """Test Governance pallet: proposals, voting, emergency, treasury.

    `submit_proposal` has a per-account cooldown that is checked *before* the
    type/threshold tables, so a second ordinary proposal from the same account
    returns `ProposalCooldownActive`. Submitting as an emergency proposal skips
    the cooldown, which is how these tests stay order-independent.
    """

    def test_submit_proposal(self, blockchain_connection, alice_keypair, submit_extrinsic, verify_kyc):
        """An emergency proposal is recorded and Pending."""
        verify_kyc(alice_keypair, 1)
        before = blockchain_connection.query(
            "Governance", "NextProposalId"
        ).value

        receipt = submit_extrinsic(
            "Governance",
            "submit_proposal",
            {
                "title": "Test Proposal",
                "description": "Integration test proposal",
                "proposal_type_index": 0,
                "threshold_index": 0,
                "is_emergency": True,
                "district_index": None,
            },
            alice_keypair,
        )
        assert receipt.is_success, f"submit_proposal failed: {receipt.error_message}"

        after = blockchain_connection.query("Governance", "NextProposalId").value
        assert int(after) > int(before), "NextProposalId did not advance"

        proposal = blockchain_connection.query(
            "Governance", "Proposals", [int(before)]
        ).value
        assert proposal is not None, f"no Proposals entry for id {before}"
        assert proposal["is_emergency"] is True
        assert proposal["status"] == "Pending"

    def test_declare_and_end_emergency(
        self, blockchain_connection, submit_sudo_extrinsic
    ):
        """Declaration is pending, and `end_emergency` clears it.

        `JaguarMode` is single global state: leaving an emergency active would
        make every later test in the session run in an emergency. Hence the
        explicit teardown at the end.
        """
        declared = submit_sudo_extrinsic(
            "Governance",
            "declare_emergency",
            {
                "emergency_type_index": 0,
                "description": "integration test emergency",
                "duration_hours": 1,
            },
        )
        assert declared.is_success, (
            f"declare_emergency failed: {declared.error_message}"
        )

        mode = blockchain_connection.query("Governance", "JaguarMode").value
        assert mode is not None, "declare_emergency wrote no JaguarMode entry"
        assert mode["active"] is False, "a fresh declaration must not be active"
        assert mode["is_pending"] is True, "a fresh declaration must be pending"

        ended = submit_sudo_extrinsic("Governance", "end_emergency", {})
        assert ended.is_success, f"end_emergency failed: {ended.error_message}"
        assert blockchain_connection.query("Governance", "JaguarMode").value is None, (
            "end_emergency left JaguarMode set"
        )

    def test_propose_treasury_spend(
        self, blockchain_connection, alice_keypair, bob_keypair, submit_extrinsic,
        verify_kyc,
    ):
        """A compliant account can propose a spend, keyed by a fresh id.

        `propose_treasury_spend` is `ensure_signed` and checks the *proposer's*
        compliance, so sudo can only ever return `BadOrigin` here.
        """
        verify_kyc(alice_keypair, 1)
        before = blockchain_connection.query(
            "Governance", "NextTreasuryProposalId"
        ).value

        receipt = submit_extrinsic(
            "Governance",
            "propose_treasury_spend",
            {
                "recipient": bob_keypair.ss58_address,
                "amount": 1_000_000_000_000,
                "description": "Infrastructure fund",
                "district_index": None,
            },
            alice_keypair,
        )
        assert receipt.is_success, (
            f"propose_treasury_spend failed: {receipt.error_message}"
        )

        after = blockchain_connection.query(
            "Governance", "NextTreasuryProposalId"
        ).value
        assert int(after) > int(before), "NextTreasuryProposalId did not advance"

        proposal = blockchain_connection.query(
            "Governance", "TreasurySpendProposals", [int(before)]
        ).value
        assert proposal is not None, f"no TreasurySpendProposals entry for {before}"
        assert proposal["executed"] is False


# ============================================================================
# 4. COMPLIANCE PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_compliance
class TestCompliancePallet:
    """Test Compliance pallet: account verification, restrictions, sanctions."""

    def test_verify_account(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test KYC account verification via Compliance.verify_account."""
        receipt = submit_sudo_extrinsic(
            "Compliance",
            "verify_account",
            {"account": bob_keypair.ss58_address, "level": 2, "risk_level": 1},
            alice_keypair
        )

        assert receipt.is_success, f"Account verification failed: {receipt.error_message}"

    def test_restrict_and_lift(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test account restriction and lifting via Compliance."""
        receipt = submit_sudo_extrinsic(
            "Compliance",
            "restrict_account",
            {"account": bob_keypair.ss58_address, "reason": list(b"Test restriction")},
            alice_keypair
        )
        assert receipt.is_success, f"Restrict failed: {receipt.error_message}"

        receipt = submit_sudo_extrinsic(
            "Compliance",
            "lift_restriction",
            {"account": bob_keypair.ss58_address},
            alice_keypair
        )
        assert receipt.is_success, f"Lift restriction failed: {receipt.error_message}"


# ============================================================================
# 5. STAKING PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_staking
class TestStakingPallet:
    """Test Staking pallet: validators, rewards."""

    def test_join_validators(
        self, blockchain_connection, bob_keypair, submit_extrinsic, identity_kyc,
        fund_account,
    ):
        """Validator registration needs Enhanced identity KYC (L3).

        `join_validators` calls `T::Identity::meets_validator_kyc`, which is
        L3 — SSN, passport *and* biometrics in the Identity pallet. An
        oracle-only verification is not enough.
        """
        identity_kyc(bob_keypair, 3)

        # `join_validators` checks `free_balance(who) >= stake` *and* that the
        # stake clears `MinValidatorStake` (1,000 DALLA). Dev accounts start
        # with 100 DALLA, so funding is part of the setup, not a workaround.
        stake = 1_000 * 1_000_000_000_000
        fund_account(bob_keypair, 5 * stake)

        receipt = submit_extrinsic(
            "Staking",
            "join_validators",
            {
                "stake": stake,
                "compute_capacity": 100,
                "location": "Belize City",
            },
            bob_keypair,
        )
        message = str(receipt.error_message)
        assert receipt.is_success or "ValidatorAlreadyActive" in message, (
            f"join_validators failed: {message}"
        )

    def test_distribute_rewards(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic):
        """Test rewards distribution via Staking.distribute_rewards (no params)."""
        receipt = submit_sudo_extrinsic(
            "Staking",
            "distribute_rewards",
            {},
            alice_keypair
        )

        assert receipt.is_success, f"Rewards distribution failed: {receipt.error_message}"


# ============================================================================
# 6. ORACLE PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_oracle
class TestOraclePallet:
    """Test Oracle pallet: price feeds, merchant verification."""

    def test_submit_price(self, oracle_operator, submit_extrinsic):
        """A registered operator can submit a price."""
        operator = oracle_operator()
        receipt = submit_extrinsic(
            "Oracle",
            "submit_price",
            {"base_currency": 0, "quote_currency": 1, "price": 150_000},
            operator,
        )
        assert receipt.is_success, f"submit_price failed: {receipt.error_message}"

    def test_non_operator_cannot_submit_price(
        self, oracle_operator, submit_extrinsic, unverified_account
    ):
        """The operator gate is real: an outsider is refused."""
        oracle_operator()  # ensure the operator set is populated
        outsider = unverified_account()

        receipt = submit_extrinsic(
            "Oracle",
            "submit_price",
            {"base_currency": 0, "quote_currency": 1, "price": 150_000},
            outsider,
        )
        assert not receipt.is_success, "a non-operator submitted a price"
        assert "NotOperator" in str(receipt.error_message) or (
            "NotAuthorized" in str(receipt.error_message)
        ), f"expected an operator gate, got: {receipt.error_message}"

    def test_verify_merchant(
        self, blockchain_connection, submit_sudo_extrinsic, bob_keypair
    ):
        """Merchant verification is council-gated, not operator-gated.

        `T::MerchantVerificationOrigin` is `TechnicalCouncilMajority`, an
        `EitherOfDiverse<EnsureRoot, _>`. An oracle operator is not a council
        member, so signing with one returns `BadOrigin` — root is the way in.
        """
        submit_sudo_extrinsic(
            "Oracle",
            "verify_merchant",
            {
                "merchant": bob_keypair.ss58_address,
                "category": 1,
                "certification": "BTB-2026-001",
                "license": "LIC-REST-001",
                "location": None,
            },
        )
        merchant = blockchain_connection.query(
            "Oracle", "MerchantCategories", [bob_keypair.ss58_address]
        ).value
        assert merchant is not None, "verify_merchant stored no Merchants entry"


# ============================================================================
# 7. PAYROLL PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_payroll
class TestPayrollPallet:
    """Test Payroll pallet: employers, employees, departments, deductions, bonuses.

    `verify_employer` takes `T::VerifierOrigin` (`TechnicalCouncilMember`, which
    admits Root), so it can be driven by sudo. Every other call here is
    `ensure_signed` and reads `VerifiedEmployers[origin]`, so those have to be
    signed by the employer itself. Driving them through sudo returns `BadOrigin`
    — which is what this class previously reported as "employer not verified".
    """

    SALARY = 5_000_000_000_000_000

    @pytest.fixture
    def verified_employer(
        self, submit_sudo_extrinsic, submit_extrinsic, alice_keypair,
        blockchain_connection, fund_account, verify_kyc,
    ):
        """Verify the employer, create a department, and KYC the employee.

        Funding is part of the setup: `execute_payment` and `issue_bonus` move
        the employer's own balance, and a dev account starts with 100 DALLA
        against a 5,000-DALLA salary.

        `add_employee` gates on `T::Oracle::meets_kyc_requirement`, so the
        employee leg needs the *oracle* KYC record (`verify_kyc`), not the
        Identity attestations `identity_kyc` writes.
        """
        fund_account(alice_keypair, 100_000 * 1_000_000_000_000)
        employee = Keypair.create_from_uri("//Bob")
        fund_account(employee, 100 * 1_000_000_000_000)
        verify_kyc(employee, 1)

        submit_sudo_extrinsic(
            "Payroll",
            "verify_employer",
            {"employer": alice_keypair.ss58_address, "employer_type": "Enterprise"},
        )
        # `EmployerAlreadyRegistered` is the expected result on a re-run, so the
        # invariant to assert is the stored profile, not the extrinsic's own
        # verdict.
        profile = blockchain_connection.query(
            "Payroll", "EmployerProfiles", [alice_keypair.ss58_address]
        ).value
        assert profile is not None, "verify_employer left no EmployerProfiles entry"
        assert profile["verified"] is True, "employer profile is not verified"

        created = submit_extrinsic(
            "Payroll",
            "create_department",
            {"name_hash": "0x" + hashlib.sha256(b"Finance").hexdigest()},
            alice_keypair,
        )
        assert created.is_success, f"department creation failed: {created.error_message}"
        return alice_keypair

    def test_verify_employer(self, verified_employer, blockchain_connection):
        """Employer verification persists in `EmployerProfiles`."""
        profile = blockchain_connection.query(
            "Payroll", "EmployerProfiles", [verified_employer.ss58_address]
        ).value
        assert profile is not None, "verify_employer wrote no EmployerProfiles entry"
        assert profile["verified"] is True

    def test_create_department(self, verified_employer, blockchain_connection):
        """A verified employer's department is recorded under their own account.

        `Departments` is a `StorageDoubleMap<employer, department_id, hash>`, so
        a one-key query raises "Storage function requires 2 parameters".
        """
        name_hash = blockchain_connection.query(
            "Payroll", "Departments", [verified_employer.ss58_address, 1]
        ).value
        assert name_hash is not None, "create_department recorded no Departments entry"

    def test_add_employee(
        self, verified_employer, submit_extrinsic, bob_keypair, blockchain_connection
    ):
        """`add_employee` keys the employee under the signing employer."""
        self._add_employee(submit_extrinsic, verified_employer, bob_keypair)

        employee = blockchain_connection.query(
            "Payroll", "Employees", [verified_employer.ss58_address,
                                     bob_keypair.ss58_address]
        ).value
        assert int(employee["salary"]) == self.SALARY

    def _add_employee(self, submit_extrinsic, employer, employee):
        """Enrol `employee`, tolerating a re-run of this suite.

        Every payroll test shares one dev chain, so the second enrolment of the
        same employee returns `EmployeeAlreadyExists`. The invariant each test
        actually needs is the stored `Employees` entry, so assert that rather
        than the extrinsic's own verdict.
        """
        receipt = submit_extrinsic(
            "Payroll",
            "add_employee",
            {
                "employee": employee.ss58_address,
                "salary": self.SALARY,
                "worker_type": "FullTime",
                "department_id": 1,
                "metadata_hash": "0x" + "00" * 32,
            },
            employer,
        )
        message = str(receipt.error_message)
        assert receipt.is_success or "EmployeeAlreadyExists" in message, (
            f"add_employee failed: {message}"
        )

    def test_set_deduction(
        self, verified_employer, submit_extrinsic, bob_keypair
    ):
        """A per-period deduction is accepted for an enrolled employee."""
        self._add_employee(submit_extrinsic, verified_employer, bob_keypair)

        receipt = submit_extrinsic(
            "Payroll",
            "set_deduction",
            {
                "employee": bob_keypair.ss58_address,
                "deduction_type": "IncomeTax",
                "amount": 500_000_000_000_000,
            },
            verified_employer,
        )
        assert receipt.is_success, f"set_deduction failed: {receipt.error_message}"

    def test_issue_bonus(self, verified_employer, submit_extrinsic, bob_keypair):
        """A bonus is accepted for an enrolled employee."""
        self._add_employee(submit_extrinsic, verified_employer, bob_keypair)

        receipt = submit_extrinsic(
            "Payroll",
            "issue_bonus",
            {
                "employee": bob_keypair.ss58_address,
                "amount": 1_000_000_000_000_000,
                "category": "Bonus",
            },
            verified_employer,
        )
        assert receipt.is_success, f"issue_bonus failed: {receipt.error_message}"

    def test_execute_payment(
        self, verified_employer, submit_extrinsic, bob_keypair
    ):
        """Payment execution moves the enrolled employee's salary."""
        self._add_employee(submit_extrinsic, verified_employer, bob_keypair)

        receipt = submit_extrinsic(
            "Payroll",
            "execute_payment",
            {"employee": bob_keypair.ss58_address},
            verified_employer,
        )
        assert receipt.is_success, f"execute_payment failed: {receipt.error_message}"


# ============================================================================
# 8. INTEROPERABILITY PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_interoperability
class TestInteroperabilityPallet:
    """Test Interoperability pallet: cross-chain bridges."""

    # `decode_chain`: 0 = Bitcoin, 1 = Ethereum, 8 = Polygon.
    # `decode_asset`: 0 = DALLA.
    ETHEREUM = 1
    POLYGON = 8
    ASSET_DALLA = 0
    MIN_BRIDGE_AMOUNT = 50 * 1_000_000_000_000  # runtime MinBridgeAmount

    # `pallets/interoperability/src/migrations.rs` -> `SEEDED_CHAINS`.
    SEEDED_CHAINS = frozenset({
        'ArbitrumOne', 'Avalanche', 'Base', 'BinanceSmartChain', 'Bitcoin',
        'Ethereum', 'Near', 'Optimism', 'Polkadot', 'Polygon', 'Solana',
        'Sui', 'Tron',
    })

    def test_the_launched_chain_has_the_full_bridge_registry(
        self, blockchain_connection
    ):
        """Gap 7: a chain launched from genesis matches an upgraded one.

        Ceiba got its 13 bridge chains from the V1 -> V2 seeding migration, which
        only runs on an upgrade. A chain launched from genesis never runs a
        migration, so it gets them from the pallet's genesis config. Both sides
        read `migrations.rs`, and this asserts the resulting registries agree.
        """
        rows = list(
            blockchain_connection.query_map("Interoperability", "ChainConfigurations")
        )
        seeded = {str(key.value) for key, _ in rows}
        assert seeded == self.SEEDED_CHAINS, (
            f"seeded registry differs from migrations.rs: {sorted(seeded)}"
        )

    def test_initiate_bridge_succeeds_for_a_configured_chain(
        self, submit_extrinsic, identity_kyc, alice_keypair, fund_account,
        blockchain_connection,
    ):
        """A verified account bridges to a chain that genesis seeded.

        The bridge registry is seeded at genesis from
        `pallets/interoperability/src/migrations.rs`, so Ethereum is already
        configured when a fresh dev chain starts. `initiate_bridge` therefore gets
        all the way through: the net amount moves to the escrow account, the fee
        to the treasury, and a `BridgeTransaction` is recorded.
        """
        identity_kyc(alice_keypair, 2)
        fund_account(alice_keypair, 1_000 * 1_000_000_000_000)

        locked_before = blockchain_connection.query(
            "Interoperability", "TotalLockedAssets", ["Ethereum", "DALLA"]
        ).value or 0

        receipt = submit_extrinsic(
            "Interoperability",
            "initiate_bridge",
            {
                "target_chain_index": self.ETHEREUM,
                "target_address": "0x1234567890abcdef1234567890abcdef12345678",
                "amount": self.MIN_BRIDGE_AMOUNT,
                "asset_index": self.ASSET_DALLA,
            },
            alice_keypair,
        )
        assert receipt.is_success, (
            f"the bridge refused a configured chain: {receipt.error_message}"
        )

        locked_after = blockchain_connection.query(
            "Interoperability", "TotalLockedAssets", ["Ethereum", "DALLA"]
        ).value or 0
        assert locked_after > locked_before, (
            "the bridge reported success without recording the locked amount"
        )

    def test_initiate_bridge_requires_enhanced_kyc(
        self, submit_extrinsic, unverified_account
    ):
        """Without Identity L2 the bridge refuses before it reaches the chain gate."""
        applicant = unverified_account()

        receipt = submit_extrinsic(
            "Interoperability",
            "initiate_bridge",
            {
                "target_chain_index": self.ETHEREUM,
                "target_address": "0x1234567890abcdef1234567890abcdef12345678",
                "amount": self.MIN_BRIDGE_AMOUNT,
                "asset_index": self.ASSET_DALLA,
            },
            applicant,
        )
        assert not receipt.is_success, "an unverified account bridged funds"
        assert "KycRequired" in str(receipt.error_message), (
            f"expected KycRequired, got: {receipt.error_message}"
        )

    def test_an_unseeded_chain_is_still_refused(
        self, submit_extrinsic, identity_kyc, alice_keypair, fund_account
    ):
        """Seeding is a fixed list, not a wildcard.

        Index 200 decodes to no `BridgeChain` at all, so it must be refused even
        though the caller is fully verified and funded. Without this, undoing the
        gate would go unnoticed.
        """
        identity_kyc(alice_keypair, 2)
        fund_account(alice_keypair, 1_000 * 1_000_000_000_000)

        receipt = submit_extrinsic(
            "Interoperability",
            "initiate_bridge",
            {
                "target_chain_index": 200,
                "target_address": "0x1234567890abcdef1234567890abcdef12345678",
                "amount": self.MIN_BRIDGE_AMOUNT,
                "asset_index": self.ASSET_DALLA,
            },
            alice_keypair,
        )
        assert not receipt.is_success, "the bridge accepted an unknown chain index"
        assert "UnsupportedChain" in str(receipt.error_message), (
            f"expected UnsupportedChain, got: {receipt.error_message}"
        )

    def test_bridge_configuration_can_be_updated(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """`update_bridge_config` edits a seeded entry rather than creating one.

        It still refuses unknown chains — it is an update, not an insert — but a
        seeded chain accepts the change and stores it. Polygon is used so this
        test does not perturb Ethereum, which the bridging tests depend on.
        """
        new_fee_rate = 25
        new_max_amount = 500_000 * 1_000_000_000_000

        receipt = submit_sudo_extrinsic(
            "Interoperability",
            "update_bridge_config",
            {
                "chain_index": self.POLYGON,
                "enabled": True,
                "fee_rate": new_fee_rate,
                "max_amount": new_max_amount,
            },
        )
        assert receipt.is_success, (
            f"update_bridge_config refused a seeded chain: {receipt.error_message}"
        )

        stored = blockchain_connection.query(
            "Interoperability", "ChainConfigurations", ["Polygon"]
        ).value
        assert stored is not None, "Polygon was not seeded at genesis"
        assert stored["fee_rate"] == new_fee_rate, (
            f"fee_rate not updated: {stored}"
        )
        assert stored["max_amount"] == new_max_amount, (
            f"max_amount not updated: {stored}"
        )


# ============================================================================
# 10. LANDLEDGER PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_landledger
class TestLandLedgerPallet:
    """Test LandLedger pallet: property registration.

    `register_property` gates on `T::BelizeKyc` -> `Identity::is_kyc_verified`,
    i.e. an SSN attestation. The oracle-based `verify_kyc` fixture does not
    satisfy it, which is why this test used to skip.
    """

    def test_register_property(
        self, blockchain_connection, submit_extrinsic, identity_kyc, alice_keypair,
        fund_account,
    ):
        """An L1 Identity holder can register a parcel.

        `register_property` reserves `RegistrationDeposit` — 100 DALLA, exactly a
        dev account's starting balance — so it must be funded first.
        """
        identity_kyc(alice_keypair, 1)
        fund_account(alice_keypair, 500 * 1_000_000_000_000)

        import random
        title = f"BZ-{random.randint(10000, 99999)}"

        receipt = submit_extrinsic(
            "LandLedger",
            "register_property",
            {
                "title_number": title,
                "description": "Beachfront lot",
                "coordinates": (17_500_000, -88_500_000),
                "area_sqm": 1000,
                "property_type_index": 0,
                "assessed_value": 500_000_000_000_000_000,
            },
            alice_keypair,
        )
        assert receipt.is_success, f"register_property failed: {receipt.error_message}"


# ============================================================================
# 11. CONSENSUS PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_consensus
class TestConsensusPallet:
    """Test Consensus pallet: PoUW, AI model registration."""

    def test_register_ai_model(
        self, blockchain_connection, alice_keypair, submit_extrinsic
    ):
        """`register_ai_model` is signed, and checks the signature's length.

        There is no root path: the call is `ensure_signed` with no role check, so
        sudo only ever returns `BadOrigin`. It does require the PQ signature to be
        exactly 4627 bytes (ML-DSA-87).
        """
        receipt = submit_extrinsic(
            "Consensus",
            "register_ai_model",
            {
                "model_type_index": 0,
                "parameters_hash": "0x" + "ab" * 32,
                "training_data_size": 1000,
                "pq_signature": "0x" + "00" * 4627,
            },
            alice_keypair,
        )
        assert receipt.is_success, (
            f"register_ai_model failed: {receipt.error_message}"
        )

    def test_register_ai_model_rejects_a_truncated_signature(
        self, submit_extrinsic, alice_keypair
    ):
        """A signature of the wrong length is refused, not silently accepted."""
        receipt = submit_extrinsic(
            "Consensus",
            "register_ai_model",
            {
                "model_type_index": 0,
                "parameters_hash": "0x" + "cd" * 32,
                "training_data_size": 1000,
                "pq_signature": "0x" + "00" * 64,
            },
            alice_keypair,
        )
        assert not receipt.is_success, "a 64-byte PQ signature was accepted"
        assert "PqVerificationFailed" in str(receipt.error_message), (
            f"expected PqVerificationFailed, got: {receipt.error_message}"
        )


# ============================================================================
# 12. QUANTUM PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_quantum
class TestQuantumPallet:
    """Test Quantum pallet: quantum job submission."""

    def test_submit_quantum_job(self, blockchain_connection, alice_keypair, submit_extrinsic):
        """Test quantum job submission via Quantum.submit_quantum_job."""
        import random
        job_suffix = random.randint(10000, 99999)
        receipt = submit_extrinsic(
            "Quantum",
            "submit_quantum_job",
            {
                "job_id": f"QJOB-TEST-{job_suffix}",
                "backend_index": 0,
                "circuit_hash": "0x" + "00" * 32,
                "num_qubits": 5,
                "circuit_depth": 10,
                "num_shots": 1000,
            },
            alice_keypair
        )

        message = str(receipt.error_message)
        assert receipt.is_success or "JobAlreadyExists" in message, (
            f"submit_quantum_job failed: {message}"
        )


# ============================================================================
# 13. COMMUNITY PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_community
class TestCommunityPallet:
    """Test Community pallet: proposals, participation."""

    def test_submit_community_proposal(self, blockchain_connection, alice_keypair, submit_extrinsic, ensure_kyc):
        """Test community proposal via Community.submit_community_proposal."""
        receipt = submit_extrinsic(
            "Community",
            "submit_community_proposal",
            {
                "proposal_type_code": 0,
                "beneficiary": alice_keypair.ss58_address,
                "amount": 10_000_000_000_000_000,
                "title": "Build playground",
                "description": "Community playground in Belize City",
            },
            alice_keypair
        )

        assert receipt.is_success, f"Community proposal failed: {receipt.error_message}"


# ============================================================================
# 14. BNS PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_bns
class TestBNSPallet:
    """Test BNS pallet: .bz domain registration."""

    def test_register_domain(
        self, blockchain_connection, alice_keypair, submit_extrinsic, identity_kyc,
        fund_account,
    ):
        """`register_domain(domain_name: Vec<u8>, tier: u8)`.

        Passing a `str` produces a scale-encoding failure with no error message
        at all, which is why this previously failed with `error_message is None`.
        """
        import random

        identity_kyc(alice_keypair, 1)
        # A 9-character Standard domain costs 100 DALLA, which is exactly a dev
        # account's starting balance — the fee transfer would leave nothing.
        fund_account(alice_keypair, 500 * 1_000_000_000_000)
        domain = f"test{random.randint(10000, 99999)}"

        receipt = submit_extrinsic(
            "Bns",
            "register_domain",
            {"domain_name": domain, "tier": 0},
            alice_keypair,
        )
        assert receipt.is_success, f"register_domain failed: {receipt.error_message}"

        record = blockchain_connection.query(
            "Bns", "DomainRegistry", [domain]
        ).value
        assert record is not None, f"no DomainRegistry entry for {domain}"
        assert same_account(record["owner"], alice_keypair.ss58_address), (
            f"domain is owned by {record['owner']}, not the registrant"
        )

        owned = blockchain_connection.query(
            "Bns", "AccountDomains", [alice_keypair.ss58_address]
        ).value
        assert owned, "AccountDomains does not list the new domain"


# ============================================================================
# 15. CONTRACTS PALLET TESTS
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.pallet_contracts
class TestContractsPallet:
    """Test Contracts pallet: Wasm smart contracts (ink!)."""

    def test_upload_contract_code(self, blockchain_connection, alice_keypair, submit_extrinsic):
        """Test uploading contract code via Contracts.upload_code."""
        wasm_code = list(bytes.fromhex("0061736d01000000"))

        receipt = submit_extrinsic(
            "Contracts",
            "upload_code",
            {
                "code": wasm_code,
                "storage_deposit_limit": None,
                "determinism": "Enforced",
            },
            alice_keypair
        )

        # A wasm magic header with no body is not a deployable module, so the
        # pallet *must* reject it. This asserts the validation path is live;
        # the previous version skipped, which made the test unfalsifiable.
        assert not receipt.is_success, (
            "an 8-byte wasm stub was accepted as deployable contract code"
        )
        assert "CodeRejected" in str(receipt.error_message) or (
            "InvalidModule" in str(receipt.error_message)
        ), f"expected a validation error, got: {receipt.error_message}"
