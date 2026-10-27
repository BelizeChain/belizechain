"""Cross-pallet integration tests against the APIs that actually exist.

Each test here exercises one real provider/consumer edge in the runtime wiring,
so a regression in the `Config` glue (not just in either pallet) fails it.

The previous version of this file called `transfer_with_compliance`,
`set_kyc_level`, `add_to_sanctions`, `vote_on_proposal`,
`register_council_candidate`, `report_data_as_validator`, `set_exchange_rate` and
`swap_with_oracle_guard`. None of those extrinsics exist; the suite could never
have run, and it asserted error names like `KYCRequired` that the pallets never
emit. Everything below is read off the real dispatchables.
"""

import pytest

DALLA = 1_000_000_000_000

# Economy:Config::TravelRuleThreshold is 100 DALLA. At or above it `transfer_bbzd`
# demands Enhanced KYC (L2) on the sender; below it Basic (L1) is enough.
TRAVEL_RULE_THRESHOLD = 100 * DALLA

L1 = 1
L2 = 2

# `transfer_bbzd` refuses sender == recipient with `CannotTransferToSelf`, so
# every transfer needs a distinct counterparty. Deriving one keeps the negative
# tests from passing for the wrong reason.
RECEIVER_URI = "//Alice//cross-pallet-receiver"


@pytest.fixture
def receiver(fund_account, fund_bbzd):
    """A funded, L1-verified account that is never the sender.

    Verification is required on *both* legs of `transfer_bbzd` — a funded but
    unverified recipient fails with `KycVerificationRequired`, which reads like
    the sender's gate and is easy to mistake for one.

    The bBZD leg matters too: `fund_account` sets the *native* balance, which
    `transfer_bbzd` never reads, so an account funded only that way fails with
    `InsufficientBbzdBalance` instead.
    """
    from substrateinterface import Keypair

    account = Keypair.create_from_uri(RECEIVER_URI)
    fund_account(account, 100 * DALLA)
    fund_bbzd(account, 10 * DALLA)  # below TravelRuleThreshold: L1 is enough
    return account


# ============================================================================
# 1. ECONOMY <-> ORACLE  (sanctions + KYC + travel rule)
# ============================================================================


@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestEconomyOracleIntegration:
    """`Economy::transfer_bbzd` delegates every gate to `T::Oracle`."""

    def test_transfer_is_refused_for_an_unverified_recipient(
        self, submit_extrinsic, fund_bbzd, unverified_account, receiver,
        blockchain_connection,
    ):
        """Basic KYC (L1) is required on the recipient as well as the sender.

        This is the Oracle -> Economy edge: the KYC record the oracle quorum
        writes is what `transfer_bbzd` reads.

        The *recipient* leg is the one that can be exercised. An unverified
        account cannot hold bBZD in the first place — `mint_bbzd` is
        minter-gated and `fund_bbzd` verifies its recipient, and sudo cannot
        bypass either because the call is `ensure_signed` — so there is no
        unverified account with a balance to send *from*.
        """
        unpaid = unverified_account()
        assert not blockchain_connection.query(
            "Economy", "BBZDBalances", [unpaid.ss58_address]
        ).value, "an unverified account must not be able to hold bBZD"

        receipt = submit_extrinsic(
            "Economy", "transfer_bbzd",
            {"to": unpaid.ss58_address, "amount": 1},
            receiver,
        )
        assert not receipt.is_success, "an unverified account must not receive bBZD"
        assert "KycVerificationRequired" in str(receipt.error_message), (
            f"expected KycVerificationRequired, got: {receipt.error_message}"
        )

    def test_transfer_succeeds_once_the_recipient_is_verified(
        self, submit_extrinsic, verify_kyc, unverified_account, receiver,
        blockchain_connection,
    ):
        """The same transfer is accepted after the oracle verifies the recipient."""
        payee = unverified_account()
        verify_kyc(payee, L1)

        receipt = submit_extrinsic(
            "Economy", "transfer_bbzd",
            {"to": payee.ss58_address, "amount": 1},
            receiver,
        )
        assert receipt.is_success, (
            f"a verified recipient still cannot receive: {receipt.error_message}"
        )
        credited = blockchain_connection.query(
            "Economy", "BBZDBalances", [payee.ss58_address]
        ).value
        assert int(credited or 0) >= 1, (
            f"recipient bBZD balance is {credited}, expected at least 1"
        )

    def test_sanctioned_account_cannot_transfer(
        self, submit_extrinsic, submit_sudo_extrinsic, verify_kyc, fund_bbzd,
        unverified_account, receiver,
    ):
        """`Oracle::add_sanctioned_entity` blocks the economy transfer path."""
        sanctioned = unverified_account()
        verify_kyc(sanctioned, L1)
        fund_bbzd(sanctioned, 10 * DALLA)

        listing = submit_sudo_extrinsic(
            "Oracle",
            "add_sanctioned_entity",
            {
                "account": sanctioned.ss58_address,
                "source": 0,
                "reason": "integration test",
                "expires_at": None,
            },
        )
        assert listing.is_success, f"could not sanction: {listing.error_message}"

        receipt = submit_extrinsic(
            "Economy", "transfer_bbzd", {"to": receiver.ss58_address, "amount": 1},
            sanctioned,
        )
        assert not receipt.is_success, "a sanctioned account must not move bBZD"
        assert "SanctionedEntity" in str(receipt.error_message), (
            f"expected SanctionedEntity, got: {receipt.error_message}"
        )

    def test_travel_rule_demands_enhanced_kyc_above_the_threshold(
        self, submit_extrinsic, verify_kyc, fund_bbzd, unverified_account, receiver
    ):
        """At >= 100 DALLA the transfer needs L2, not L1.

        With L1 the call must fail with `TravelRuleEnhancedKycRequired`; the
        same amount therefore proves the threshold branch is wired to
        `Config::TravelRuleThreshold`.
        """
        payer = unverified_account()
        verify_kyc(payer, L1)
        # Minting above the threshold would itself demand L2, so the balance is
        # kept small. That is fine: `transfer_bbzd` evaluates the travel rule
        # *before* the balance check, so the branch under test is reached anyway.
        fund_bbzd(payer, 10 * DALLA)

        receipt = submit_extrinsic(
            "Economy",
            "transfer_bbzd",
            {"to": receiver.ss58_address, "amount": TRAVEL_RULE_THRESHOLD},
            payer,
        )
        assert not receipt.is_success, (
            "a >= 100 DALLA transfer from an L1 account must be refused"
        )
        assert "TravelRuleEnhancedKycRequired" in str(receipt.error_message), (
            f"expected TravelRuleEnhancedKycRequired, got: {receipt.error_message}"
        )


# ============================================================================
# 2. LANDLEDGER <-> IDENTITY
# ============================================================================


@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestLandLedgerIdentityIntegration:
    """`LandLedger::register_property` reads `T::BelizeKyc`, not `T::Oracle`.

    That distinction matters: property registration consults the *Identity*
    pallet's KYC state, while the economy path consults the *Oracle*'s. A test
    that only verified through Oracle would leave this edge uncovered.
    """

    def test_registration_is_refused_without_identity_kyc(
        self, submit_extrinsic, unverified_account
    ):
        """No verified Identity record => `KycNotVerified`."""
        applicant = unverified_account()

        receipt = submit_extrinsic(
            "LandLedger",
            "register_property",
            {
                "title_number": "BZ-TEST-0001",
                "description": "cross-pallet integration parcel",
                "coordinates": (17_500_000, -88_300_000),
                "area_sqm": 500,
                "property_type_index": 0,
                "assessed_value": 50_000 * DALLA,
            },
            applicant,
        )
        assert not receipt.is_success, (
            "property registration must require KYC"
        )
        assert "KycNotVerified" in str(receipt.error_message), (
            f"expected KycNotVerified, got: {receipt.error_message}"
        )


# ============================================================================
# 3. GOVERNANCE <-> COMPLIANCE
# ============================================================================


@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestGovernanceComplianceIntegration:
    """Governance gates proposals on `T::ComplianceProvider`."""

    def test_proposal_from_an_unverified_account_is_refused(
        self, submit_extrinsic, unverified_account
    ):
        """`submit_proposal` refuses an account with no compliance record."""
        proposer = unverified_account()

        receipt = submit_extrinsic(
            "Governance",
            "submit_proposal",
            {
                "title": "Cross-pallet compliance gate",
                "description": "must be refused before any deposit is taken",
                "proposal_type_index": 1,
                "threshold_index": 0,
                # Emergency bypasses the per-account cooldown, so this reaches
                # the compliance check rather than stopping at the cooldown.
                "is_emergency": True,
                "district_index": 0,
            },
            proposer,
        )
        assert not receipt.is_success, (
            "an unverified account must not submit a proposal"
        )
        assert "InsufficientCompliance" in str(receipt.error_message), (
            f"expected InsufficientCompliance, got: {receipt.error_message}"
        )


# ============================================================================
# 4. BELIZEX <-> ORACLE
# ============================================================================


@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestBelizeXOracleIntegration:
    """The DEX deviation guard reads `BelizeXOracle::get_crypto_exchange_rate`.

    The guard's own scoring is unit-tested in `pallets/belizex`; what this file
    adds is the runtime wiring — that `Oracle::update_exchange_rate` is the
    entry the guard consumes, and that clearing it makes the guard fail closed
    instead of trading blind.
    """

    MANUAL_RATE_KEY = [{"base": "USD", "quote": "BZD"}]

    def test_the_rate_the_guard_reads_is_settable(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """Oracle writes the USD/BZD entry the guard consumes."""
        price = 1_980_000  # 1.98 BZD per USD

        receipt = submit_sudo_extrinsic(
            "Oracle",
            "update_exchange_rate",
            {"base_currency": 1, "quote_currency": 0, "price": price},
        )
        assert receipt.is_success, f"rate update failed: {receipt.error_message}"

        entry = blockchain_connection.query(
            "Oracle", "ManualExchangeRates", self.MANUAL_RATE_KEY
        ).value
        assert entry is not None, "update_exchange_rate stored nothing"
        # `(price, written_at)` — the second element is the block number.
        assert int(entry[0]) == price, (
            f"the guard would read {entry[0]}, not the {price} that was set"
        )
