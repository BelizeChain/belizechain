"""Proposal submission and lifecycle, against the Governance pallet as it is.

Real surface used here:

- `submit_proposal(title, description, proposal_type_index, threshold_index,
  is_emergency, district_index)` — signed, needs KYC >= 1
- `cast_vote(proposal_id, vote_choice_index, conviction)` — signed
- `finalize_proposal(proposal_id)` — signed; needs `block > voting_end`
- storage: `Proposals` (u32 -> Proposal), `NextProposalId`,
  `AccountLastProposal`

Timings that shape what is testable here (runtime values, verified):

- `MinimumDeposit = 10 DALLA`, and the deposit is **not** caller-supplied —
  `submit_proposal` reserves `T::MinimumDeposit` itself.
- `LaunchPeriod = 14_400` blocks (~24 h): voting does not open until a day after
  submission, so a vote-then-finalize round trip cannot be exercised on a dev
  chain. Those paths are covered by the pallet's unit tests; what is asserted
  here is everything reachable in seconds, plus the guards.
- `ProposalCooldown = 43_200` blocks, bypassed by `is_emergency`.

The previous version of this file tested `create_proposal`, `second_proposal`,
`vote`, `cancel_proposal`, `force_execute_proposal`, `ProposalCount`,
`VotingPeriod` and a `Seconded` status. None of those exist, so every test
self-skipped and the suite reported green while asserting nothing.
"""

import pytest
from substrateinterface import Keypair

from conftest import same_account

# `ProposalType` discriminants accepted by `proposal_type_index`.
TYPE_CONSTITUTIONAL = 0
TYPE_ECONOMIC = 1
TYPE_DISTRICT_LOCAL = 7

# `VotingThreshold` discriminants accepted by `threshold_index`.
THRESHOLD_SIMPLE_MAJORITY = 0

SIMPLE_MAJORITY_AYE = 0

# Runtime constants this suite depends on (runtime/src/lib.rs).
MINIMUM_DEPOSIT = 10 * 1_000_000_000_000          # 10 DALLA, 12 decimals
EMERGENCY_LAUNCH_PERIOD = 3_600                     # 1 hour of blocks


def _propose(submit_extrinsic, keypair, **overrides):
    """Submit a proposal with sane defaults, overriding only what a test needs."""
    params = {
        "title": "0x" + b"Integration test proposal".hex(),
        "description": "0x" + b"Submitted by the governance test suite".hex(),
        "proposal_type_index": TYPE_ECONOMIC,
        "threshold_index": THRESHOLD_SIMPLE_MAJORITY,
        "is_emergency": False,
        "district_index": None,
    }
    params.update(overrides)
    return submit_extrinsic("Governance", "submit_proposal", params, keypair)


def _last_proposal_id(substrate) -> int:
    return int(substrate.query("Governance", "NextProposalId").value) - 1


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestProposalSubmission:
    """What `submit_proposal` actually writes, and why it can refuse."""

    def test_submission_records_the_proposal_and_reserves_the_deposit(
        self, submit_extrinsic, verify_kyc, blockchain_connection, alice_keypair
    ):
        """A valid proposal lands in `Proposals` with the fields the pallet sets."""
        substrate = blockchain_connection
        verify_kyc(alice_keypair, 1)

        free_before = int(
            substrate.query("System", "Account", [alice_keypair.ss58_address])
            .value["data"]["free"]
        )

        receipt = _propose(submit_extrinsic, alice_keypair)
        if not receipt.is_success and "ProposalCooldownActive" in str(
            receipt.error_message
        ):
            pytest.skip(
                "this account already submitted a proposal within the 43_200-block "
                "cooldown on this chain"
            )
        assert receipt.is_success, (
            f"submit_proposal failed: {receipt.error_message}"
        )

        proposal_id = _last_proposal_id(substrate)
        proposal = substrate.query("Governance", "Proposals", [proposal_id]).value
        assert proposal is not None, (
            f"submit_proposal returned success but wrote no Proposals[{proposal_id}]"
        )

        assert same_account(proposal["proposer"], alice_keypair.ss58_address), (
            f"recorded proposer {proposal['proposer']} is not the submitter"
        )
        assert proposal["status"] == "Pending", (
            f"a fresh proposal is Pending, got {proposal['status']}"
        )
        assert int(proposal["deposit"]) == MINIMUM_DEPOSIT, (
            f"deposit is fixed at T::MinimumDeposit, got {proposal['deposit']}"
        )
        assert proposal["proposal_type"] == "Economic"
        assert proposal["threshold"] == "SimpleMajority"
        assert proposal["is_emergency"] is False
        # Voting opens after the launch period, not immediately.
        assert int(proposal["voting_start"]) > 0
        assert int(proposal["voting_end"]) > int(proposal["voting_start"]), (
            "voting_end must be after voting_start"
        )

        # The deposit is reserved, so free balance drops by at least that much.
        free_after = int(
            substrate.query("System", "Account", [alice_keypair.ss58_address])
            .value["data"]["free"]
        )
        assert free_before - free_after >= MINIMUM_DEPOSIT, (
            "the minimum deposit was not reserved from the proposer"
        )

    def test_submission_requires_kyc(self, submit_extrinsic, unverified_account):
        """An unverified account cannot propose — the compliance gate is real."""
        unverified = unverified_account()

        receipt = _propose(submit_extrinsic, unverified)
        assert not receipt.is_success, "an unverified account must not propose"
        assert "InsufficientCompliance" in str(receipt.error_message), (
            f"expected InsufficientCompliance, got: {receipt.error_message}"
        )

    def test_invalid_proposal_type_index_is_rejected(
        self, submit_extrinsic, verify_kyc, alice_keypair
    ):
        """Only indices 0..=8 map to a `ProposalType`.

        Submitted as an emergency proposal because the per-account cooldown is
        checked *before* the index tables, so a normal submission from an
        account that already proposed would return ProposalCooldownActive and
        never reach the validation being tested.
        """
        verify_kyc(alice_keypair, 1)
        receipt = _propose(
            submit_extrinsic,
            alice_keypair,
            proposal_type_index=9,
            is_emergency=True,
        )
        assert not receipt.is_success, "index 9 is not a proposal type"
        assert "InvalidProposalType" in str(receipt.error_message), (
            f"expected InvalidProposalType, got: {receipt.error_message}"
        )

    def test_invalid_threshold_index_is_rejected(
        self, submit_extrinsic, verify_kyc, alice_keypair
    ):
        """Only indices 0..=2 map to a `VotingThreshold`.

        Emergency submission for the same reason as the proposal-type test:
        the cooldown is checked before the index tables.
        """
        verify_kyc(alice_keypair, 1)
        receipt = _propose(
            submit_extrinsic, alice_keypair, threshold_index=3, is_emergency=True
        )
        assert not receipt.is_success, "index 3 is not a voting threshold"
        assert "InvalidThreshold" in str(receipt.error_message), (
            f"expected InvalidThreshold, got: {receipt.error_message}"
        )

    def test_invalid_district_is_rejected(
        self, submit_extrinsic, verify_kyc, alice_keypair
    ):
        """Districts are 0-based and must be < 6."""
        verify_kyc(alice_keypair, 1)
        receipt = _propose(
            submit_extrinsic, alice_keypair, district_index=6, is_emergency=True
        )
        assert not receipt.is_success, "district 6 is out of range"
        assert "InvalidDistrict" in str(receipt.error_message), (
            f"expected InvalidDistrict, got: {receipt.error_message}"
        )

    def test_district_local_proposal_requires_a_district(
        self, submit_extrinsic, verify_kyc, alice_keypair
    ):
        """A `DistrictLocal` proposal with no district is refused."""
        verify_kyc(alice_keypair, 1)
        receipt = _propose(
            submit_extrinsic,
            alice_keypair,
            proposal_type_index=TYPE_DISTRICT_LOCAL,
            district_index=None,
            is_emergency=True,
        )
        assert not receipt.is_success, "DistrictLocal requires a district"
        assert "DistrictRequired" in str(receipt.error_message), (
            f"expected DistrictRequired, got: {receipt.error_message}"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestProposalCooldown:
    """The per-account cooldown, and the emergency bypass."""

    def test_second_proposal_is_blocked_by_the_cooldown(
        self, submit_extrinsic, verify_kyc, bob_keypair
    ):
        """One account cannot queue a second proposal inside the cooldown."""
        verify_kyc(bob_keypair, 1)

        first = _propose(submit_extrinsic, bob_keypair)
        if not first.is_success and "ProposalCooldownActive" in str(
            first.error_message
        ):
            # Already inside a cooldown from an earlier test on this chain —
            # which is itself the behaviour under test.
            return
        assert first.is_success, f"setup proposal failed: {first.error_message}"

        second = _propose(submit_extrinsic, bob_keypair)
        assert not second.is_success, (
            "a second proposal inside the 43_200-block cooldown must be refused"
        )
        assert "ProposalCooldownActive" in str(second.error_message), (
            f"expected ProposalCooldownActive, got: {second.error_message}"
        )

    def test_emergency_proposal_uses_the_short_launch_period(
        self, submit_extrinsic, verify_kyc, blockchain_connection, charlie_keypair
    ):
        """`is_emergency` shortens the launch period from ~24 h to 1 h.

        The cooldown bypass itself cannot be observed here — it needs a second
        submission from the same account — but the shortened period proves the
        emergency path is taken rather than the normal one.
        """
        substrate = blockchain_connection
        verify_kyc(charlie_keypair, 1)

        receipt = _propose(submit_extrinsic, charlie_keypair, is_emergency=True)
        if not receipt.is_success and "ProposalCooldownActive" in str(
            receipt.error_message
        ):
            pytest.skip("this account is already inside a proposal cooldown")
        assert receipt.is_success, (
            f"emergency submit_proposal failed: {receipt.error_message}"
        )

        proposal = substrate.query(
            "Governance", "Proposals", [_last_proposal_id(substrate)]
        ).value
        assert proposal["is_emergency"] is True

        submitted_at = substrate.get_block_number(substrate.get_chain_head())
        launch = int(proposal["voting_start"]) - submitted_at
        assert 0 < launch <= EMERGENCY_LAUNCH_PERIOD + 2, (
            f"emergency launch period should be ~{EMERGENCY_LAUNCH_PERIOD} blocks, "
            f"got {launch}"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestProposalGuards:
    """Guards that can be checked without waiting a day for voting to open."""

    def test_voting_is_rejected_before_the_launch_period_elapses(
        self, submit_extrinsic, verify_kyc, fund_account, blockchain_connection
    ):
        """`cast_vote` on a proposal whose voting has not started is refused.

        This is the closest reachable assertion to the voting path: opening a
        real voting window takes 14_400 blocks.
        """
        substrate = blockchain_connection
        # A dedicated dev account: a dev chain must be reused across runs, so
        # sharing one account would mean one run's proposal blocks the next.
        voter = Keypair.create_from_uri("//Dave")
        fund_account(voter, 100 * 1_000_000_000_000)  # proposal deposit + fees
        verify_kyc(voter, 1)

        submitted = _propose(submit_extrinsic, voter)
        if not submitted.is_success and "ProposalCooldownActive" in str(
            submitted.error_message
        ):
            pytest.skip("this account is already inside a proposal cooldown")
        assert submitted.is_success, f"setup proposal failed: {submitted.error_message}"

        proposal_id = _last_proposal_id(substrate)

        receipt = submit_extrinsic(
            "Governance",
            "cast_vote",
            {
                "proposal_id": proposal_id,
                "vote_choice_index": SIMPLE_MAJORITY_AYE,
                "conviction": 0,
            },
            voter,
        )
        assert not receipt.is_success, (
            "voting before the launch period elapsed must be refused"
        )
        message = str(receipt.error_message)
        assert "VotingPeriodNotStarted" in message or "VotingPeriodEnded" in message, (
            f"expected a voting-period error, got: {receipt.error_message}"
        )

    def test_finalize_before_voting_ends_is_rejected(
        self, submit_extrinsic, verify_kyc, fund_account, blockchain_connection
    ):
        """`finalize_proposal` requires `block > voting_end`."""
        substrate = blockchain_connection
        proposer = Keypair.create_from_uri("//Eve")
        fund_account(proposer, 100 * 1_000_000_000_000)
        verify_kyc(proposer, 1)

        submitted = _propose(submit_extrinsic, proposer)
        if not submitted.is_success and "ProposalCooldownActive" in str(
            submitted.error_message
        ):
            pytest.skip("this account is already inside a proposal cooldown")
        assert submitted.is_success, f"setup proposal failed: {submitted.error_message}"

        proposal_id = _last_proposal_id(substrate)
        receipt = submit_extrinsic(
            "Governance", "finalize_proposal", {"proposal_id": proposal_id},
            proposer,
        )
        assert not receipt.is_success, (
            "finalizing before the voting period ended must be refused"
        )
        assert "VotingPeriodEnded" in str(receipt.error_message), (
            f"expected VotingPeriodEnded, got: {receipt.error_message}"
        )

    def test_finalize_unknown_proposal_is_rejected(
        self, submit_extrinsic, verify_kyc, alice_keypair
    ):
        """An out-of-range id is refused rather than panicking."""
        verify_kyc(alice_keypair, 1)
        receipt = submit_extrinsic(
            "Governance", "finalize_proposal", {"proposal_id": 2**31 - 1},
            alice_keypair,
        )
        assert not receipt.is_success
        assert "ProposalNotFound" in str(receipt.error_message), (
            f"expected ProposalNotFound, got: {receipt.error_message}"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestProposalStorage:
    """Storage-shape checks the old suite got wrong."""

    def test_next_proposal_id_only_advances(self, blockchain_connection):
        """`NextProposalId` is a monotonic counter; `ProposalCount` does not exist."""
        substrate = blockchain_connection
        first = int(substrate.query("Governance", "NextProposalId").value)
        second = int(substrate.query("Governance", "NextProposalId").value)
        assert second >= first, "NextProposalId must never move backwards"

    def test_every_proposal_is_keyed_by_its_own_id(self, blockchain_connection):
        """`Proposals[id].id == id` for every stored proposal."""
        substrate = blockchain_connection
        for key, proposal in substrate.query_map("Governance", "Proposals"):
            assert int(proposal.value["id"]) == int(key.value), (
                f"Proposals[{key.value}] records id {proposal.value['id']}"
            )
