"""Treasury spending, against the Governance pallet as it is.

The treasury primitives live in **Governance**, not Economy:

- `propose_treasury_spend(recipient, amount, description, district_index)`
- `approve_treasury_spend(proposal_id)`
- `execute_treasury_proposal(proposal_id)`
- storage: `TreasurySpendProposals`, `NextTreasuryProposalId`,
  `NationalTreasuryReserve`, `TreasurySpendTracker`, `DepartmentTreasuryBalances`

The approval threshold is **amount-tiered**, not a fixed 4-of-7:

| amount            | approvals |
|-------------------|-----------|
| < 10_000 DALLA    | 1         |
| < 100_000 DALLA   | 3         |
| otherwise         | 4         |

The previous version of this file called `Economy.propose_treasury_spend`,
`Economy.execute_treasury_proposal` and read `Economy.TreasuryProposalCount`,
`Economy.TreasuryBalance` and `Economy.TreasurySpendingHistory`.

None of those exist anywhere in the runtime — the treasury is not in the Economy
pallet at all — so every test self-skipped and the suite reported green while
asserting nothing.
"""

import pytest

from conftest import same_account

DALLA = 1_000_000_000_000  # 12 decimals

# Threshold tiers from `propose_treasury_spend`.
TIER_SINGLE = 1
TIER_THREE = 3
TIER_FOUR = 4

# `propose_treasury_spend` compares raw balances against 10_000_000_000_000 and
# 100_000_000_000_000. At 12 decimals those are 10 and 100 DALLA — the pallet's
# inline comment claims "10K DALLA" and "100K DALLA", which is off by 1000x.
SMALL_AMOUNT = 5 * DALLA              # < 10 DALLA  -> 1 approval
MID_AMOUNT = 50 * DALLA               # < 100 DALLA -> 3 approvals
LARGE_AMOUNT = 150 * DALLA            # >= 100 DALLA -> 4 approvals


def _propose(submit_extrinsic, keypair, amount, recipient=None, district=None):
    return submit_extrinsic(
        "Governance",
        "propose_treasury_spend",
        {
            "recipient": (recipient or keypair).ss58_address,
            "amount": amount,
            "description": "0x" + b"governance test spend".hex(),
            "district_index": district,
        },
        keypair,
    )


def _last_treasury_id(substrate) -> int:
    return int(substrate.query("Governance", "NextTreasuryProposalId").value) - 1


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTreasuryProposal:
    """Creation, the amount-tiered threshold, and the recorded fields."""

    def test_creating_a_proposal_records_it_with_the_tiered_threshold(
        self, submit_extrinsic, verify_kyc, blockchain_connection, alice_keypair
    ):
        """A small spend needs 1 approval; the record reflects that."""
        substrate = blockchain_connection
        verify_kyc(alice_keypair, 1)

        receipt = _propose(submit_extrinsic, alice_keypair, SMALL_AMOUNT)
        assert receipt.is_success, (
            f"propose_treasury_spend failed: {receipt.error_message}"
        )

        proposal_id = _last_treasury_id(substrate)
        proposal = substrate.query(
            "Governance", "TreasurySpendProposals", [proposal_id]
        ).value
        assert proposal is not None, (
            "propose_treasury_spend succeeded but wrote no "
            f"TreasurySpendProposals[{proposal_id}]"
        )

        assert same_account(proposal["proposer"], alice_keypair.ss58_address), (
            f"recorded proposer {proposal['proposer']} is not the submitter"
        )
        assert int(proposal["amount"]) == SMALL_AMOUNT
        assert int(proposal["threshold"]) == TIER_SINGLE, (
            f"5 DALLA is below the 10-DALLA tier and needs 1 approval, got "
            f"{proposal['threshold']}"
        )
        assert list(proposal["approvals"]) == [], (
            "a new proposal starts with no approvals"
        )
        assert proposal["executed"] is False
        assert int(proposal["expires_at"]) > int(proposal["created_at"]), (
            "the 30-day expiry must be in the future"
        )

    def test_threshold_scales_with_amount(
        self, submit_extrinsic, verify_kyc, blockchain_connection
    ):
        """10..100 DALLA needs 3 approvals; >=100 DALLA needs 4.

        Uses two separate funded accounts because each proposal is independent —
        the threshold is not cumulative.
        """
        substrate = blockchain_connection

        from substrateinterface import Keypair

        mid_proposer = Keypair.create_from_uri("//Bob")
        large_proposer = Keypair.create_from_uri("//Charlie")
        for account in (mid_proposer, large_proposer):
            verify_kyc(account, 1)

        mid = _propose(submit_extrinsic, mid_proposer, MID_AMOUNT)
        assert mid.is_success, f"mid-tier proposal failed: {mid.error_message}"
        mid_record = substrate.query(
            "Governance", "TreasurySpendProposals", [_last_treasury_id(substrate)]
        ).value
        assert int(mid_record["threshold"]) == TIER_THREE, (
            f"50 DALLA falls in the 10..100 DALLA tier and needs 3 approvals, "
            f"got {mid_record['threshold']}"
        )

        large = _propose(submit_extrinsic, large_proposer, LARGE_AMOUNT)
        assert large.is_success, f"large-tier proposal failed: {large.error_message}"
        large_record = substrate.query(
            "Governance", "TreasurySpendProposals", [_last_treasury_id(substrate)]
        ).value
        assert int(large_record["threshold"]) == TIER_FOUR, (
            f"150 DALLA is at or above the 100-DALLA tier and needs 4 "
            f"approvals, got {large_record['threshold']}"
        )

    def test_proposal_requires_compliance(self, submit_extrinsic, unverified_account):
        """An account without KYC cannot propose a spend."""
        unverified = unverified_account()
        receipt = _propose(submit_extrinsic, unverified, SMALL_AMOUNT)
        assert not receipt.is_success, "an unverified account must not propose"
        assert "NotCompliant" in str(receipt.error_message), (
            f"expected NotCompliant, got: {receipt.error_message}"
        )

    def test_invalid_district_is_rejected(
        self, submit_extrinsic, verify_kyc, alice_keypair
    ):
        """`district_index` is 0-based and must be < 6."""
        verify_kyc(alice_keypair, 1)
        receipt = _propose(submit_extrinsic, alice_keypair, SMALL_AMOUNT, district=6)
        assert not receipt.is_success, "district 6 is out of range"
        assert "InvalidDistrict" in str(receipt.error_message), (
            f"expected InvalidDistrict, got: {receipt.error_message}"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTreasuryApproval:
    """The approval path and the execution gate."""

    def test_approval_is_recorded_once_per_account(
        self, submit_extrinsic, verify_kyc, council_keypair, blockchain_connection,
        alice_keypair,
    ):
        """A council member's approval is recorded, and a repeat is refused.

        `approve_treasury_spend` is gated on `CouncilMembers`, so this needs a
        seat to exist — which `tests/governance/test_council.py` creates.
        """
        substrate = blockchain_connection
        member = council_keypair()
        if member is None:
            pytest.skip(
                "no dev account holds a council seat on this chain, so no "
                "treasury approval can be signed; run "
                "tests/governance/test_council.py first"
            )

        verify_kyc(alice_keypair, 1)
        created = _propose(submit_extrinsic, alice_keypair, MID_AMOUNT)
        assert created.is_success, f"setup failed: {created.error_message}"
        proposal_id = _last_treasury_id(substrate)

        first = submit_extrinsic(
            "Governance", "approve_treasury_spend", {"proposal_id": proposal_id},
            member,
        )
        assert first.is_success, f"approve_treasury_spend failed: {first.error_message}"

        record = substrate.query(
            "Governance", "TreasurySpendProposals", [proposal_id]
        ).value
        assert any(same_account(addr, member.ss58_address)
                   for addr in record["approvals"]), (
            "the approval was not recorded on the proposal"
        )

        repeat = submit_extrinsic(
            "Governance", "approve_treasury_spend", {"proposal_id": proposal_id},
            member,
        )
        assert not repeat.is_success, "a second approval from the same account must fail"
        message = str(repeat.error_message)
        assert (
            "AlreadyApproved" in message or "AlreadyVoted" in message
        ), f"expected an already-approved error, got: {repeat.error_message}"

    def test_execution_before_threshold_is_rejected(
        self, submit_extrinsic, verify_kyc, blockchain_connection, alice_keypair
    ):
        """A 3-of-7 proposal cannot execute on zero approvals."""
        substrate = blockchain_connection
        verify_kyc(alice_keypair, 1)

        created = _propose(submit_extrinsic, alice_keypair, MID_AMOUNT)
        assert created.is_success, f"setup failed: {created.error_message}"
        proposal_id = _last_treasury_id(substrate)

        receipt = submit_extrinsic(
            "Governance", "execute_treasury_proposal", {"proposal_id": proposal_id},
            alice_keypair,
        )
        assert not receipt.is_success, (
            "executing below the approval threshold must be refused"
        )
        message = str(receipt.error_message)
        assert "Threshold" in message or "Insufficient" in message, (
            f"expected a threshold error, got: {receipt.error_message}"
        )

        record = substrate.query(
            "Governance", "TreasurySpendProposals", [proposal_id]
        ).value
        assert record["executed"] is False, "a refused execution must not mark it executed"

    def test_unknown_proposal_is_rejected(
        self, submit_extrinsic, council_keypair
    ):
        """A non-existent id is refused rather than panicking."""
        member = council_keypair()
        if member is None:
            pytest.skip(
                "approve_treasury_spend is council-gated and no dev account "
                "holds a seat on this chain"
            )
        receipt = submit_extrinsic(
            "Governance", "approve_treasury_spend", {"proposal_id": 2**31 - 1},
            member,
        )
        assert not receipt.is_success
        message = str(receipt.error_message)
        assert "NotFound" in message or "TreasuryProposal" in message, (
            f"expected a not-found error, got: {receipt.error_message}"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTreasuryStorage:
    """Storage shape, including the items the old suite invented."""

    def test_national_reserve_exists(self, blockchain_connection):
        """`NationalTreasuryReserve` is the balance the treasury spends from.

        The old suite read `Economy.TreasuryBalance`, which does not exist.
        """
        substrate = blockchain_connection
        reserve = substrate.query("Governance", "NationalTreasuryReserve").value
        assert reserve is not None, (
            "NationalTreasuryReserve must always be readable; a missing item "
            "means the treasury has no funding source wired"
        )
        assert int(reserve) >= 0

    def test_department_balances_are_keyed_by_department(self, blockchain_connection):
        """`DepartmentTreasuryBalances` is a per-department map."""
        substrate = blockchain_connection
        for key, balance in substrate.query_map(
            "Governance", "DepartmentTreasuryBalances"
        ):
            assert key.value is not None
            assert int(balance.value) >= 0

    def test_every_proposal_is_keyed_by_its_own_id(self, blockchain_connection):
        """`TreasurySpendProposals[id].id == id`."""
        substrate = blockchain_connection
        for key, proposal in substrate.query_map(
            "Governance", "TreasurySpendProposals"
        ):
            assert int(proposal.value["id"]) == int(key.value), (
                f"TreasurySpendProposals[{key.value}] records id "
                f"{proposal.value['id']}"
            )
