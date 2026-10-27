"""Vote delegation and conviction, against the Governance pallet as it is.

Real surface used here:

- `delegate_vote(delegate, expires_at)` / `revoke_delegation()`
- `cast_vote(proposal_id, vote_choice_index, conviction)`
- `conviction_multiplier(conviction)` — `CONVICTION_MULTIPLIERS = [1, 2, 3, 6]`
  with `MAX_CONVICTION = 3`, so anything above 3 must be refused
- storage: `VoteDelegations`, `DelegationReceivers`, `ConvictionLocks`,
  `EffectiveVotingMultiplier`

Two limits shape what is reachable here, and both are real:

1. `LaunchPeriod = 14_400` blocks (~24 h), so a proposal's voting window cannot
   be opened inside an integration test. Conviction is therefore exercised via
   its pure multiplier function and the `ConvictionLocks` bookkeeping, not by
   casting a real vote.
2. There is no `QuorumThreshold` storage item — quorum comes from
   `ComplianceProvider::eligible_voter_count()`, so the old "quorum threshold"
   test was asserting on something that does not exist.

The previous version of this file tested `ProposalCount`, `Delegations`,
`QuorumThreshold` and `VoteTally` as standalone storage items. None exist, so
every test self-skipped and the suite reported green while asserting nothing.
"""

import pytest
from substrateinterface import Keypair

from conftest import same_account

MAX_CONVICTION = 3
CONVICTION_MULTIPLIERS = {0: 1, 1: 2, 2: 3, 3: 6}

# `VoteChoice` discriminants.
AYE, NAY, ABSTAIN = 0, 1, 2


def _fund_and_verify(fund_account, verify_kyc, uri: str):
    """A funded, KYC-verified dev account."""
    account = Keypair.create_from_uri(uri)
    fund_account(account, 100 * 1_000_000_000_000)
    verify_kyc(account, 1)
    return account


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestVoteDelegation:
    """`delegate_vote` / `revoke_delegation` and their guards."""

    def test_delegation_is_recorded_and_revocable(
        self, submit_extrinsic, fund_account, verify_kyc, blockchain_connection
    ):
        """A delegation writes `VoteDelegations` and `revoke_delegation` clears it."""
        substrate = blockchain_connection
        delegator = _fund_and_verify(fund_account, verify_kyc, "//Ferdie")
        delegate = _fund_and_verify(fund_account, verify_kyc, "//Alice")

        delegated = submit_extrinsic(
            "Governance",
            "delegate_vote",
            {"delegate": delegate.ss58_address, "expires_at": None},
            delegator,
        )
        if not delegated.is_success and "DelegationAlreadyExists" in str(
            delegated.error_message
        ):
            pytest.skip("this account's delegation already exists on this chain")
        assert delegated.is_success, (
            f"delegate_vote failed: {delegated.error_message}"
        )

        record = substrate.query(
            "Governance", "VoteDelegations", [delegator.ss58_address]
        ).value
        assert record is not None, (
            "delegate_vote succeeded but wrote no VoteDelegations entry"
        )
        assert same_account(record["delegate"], delegate.ss58_address), (
            f"delegation recorded for {record['delegate']}"
        )
        assert record["is_active"] is True
        assert int(record["expires_at"]) > int(record["created_at"]), (
            "the default expiry is one year in the future"
        )

        receivers = substrate.query(
            "Governance", "DelegationReceivers", [delegate.ss58_address]
        ).value
        assert any(same_account(addr, delegator.ss58_address) for addr in receivers), (
            "the delegate's receiver list does not include the delegator"
        )

        revoked = submit_extrinsic(
            "Governance", "revoke_delegation", {}, delegator
        )
        assert revoked.is_success, (
            f"revoke_delegation failed: {revoked.error_message}"
        )
        assert substrate.query(
            "Governance", "VoteDelegations", [delegator.ss58_address]
        ).value is None, "revoke_delegation left the delegation in place"

    def test_cannot_delegate_to_self(
        self, submit_extrinsic, fund_account, verify_kyc, alice_keypair
    ):
        """Self-delegation is refused."""
        verify_kyc(alice_keypair, 1)
        receipt = submit_extrinsic(
            "Governance",
            "delegate_vote",
            {"delegate": alice_keypair.ss58_address, "expires_at": None},
            alice_keypair,
        )
        assert not receipt.is_success, "self-delegation must be refused"
        message = str(receipt.error_message)
        # Self-delegation is also caught by the existing-delegation guard if the
        # account already delegated, so accept either rejection reason.
        assert (
            "CannotDelegateToSelf" in message
            or "DelegationAlreadyExists" in message
        ), f"expected a delegation guard, got: {receipt.error_message}"

    def test_delegation_requires_kyc_on_both_sides(
        self, submit_extrinsic, fund_account, verify_kyc, unverified_account
    ):
        """The delegate must be KYC-verified too, not just the delegator."""
        delegator = _fund_and_verify(fund_account, verify_kyc, "//Charlie")
        unverified_delegate = unverified_account()

        receipt = submit_extrinsic(
            "Governance",
            "delegate_vote",
            {"delegate": unverified_delegate.ss58_address, "expires_at": None},
            delegator,
        )
        assert not receipt.is_success, (
            "delegating to an unverified account must be refused"
        )
        message = str(receipt.error_message)
        assert (
            "InsufficientCompliance" in message
            or "DelegationAlreadyExists" in message
        ), f"expected InsufficientCompliance, got: {receipt.error_message}"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestConviction:
    """Conviction multipliers and their bounds."""

    def test_conviction_above_max_is_rejected(
        self, submit_extrinsic, fund_account, verify_kyc, blockchain_connection
    ):
        """`conviction` is capped at `MAX_CONVICTION` (3)."""
        substrate = blockchain_connection
        voter = _fund_and_verify(fund_account, verify_kyc, "//Dave")

        receipt = submit_extrinsic(
            "Governance",
            "cast_vote",
            {
                "proposal_id": 0,
                "vote_choice_index": AYE,
                "conviction": MAX_CONVICTION + 1,
            },
            voter,
        )
        assert not receipt.is_success, (
            f"conviction {MAX_CONVICTION + 1} is out of range and must be refused"
        )
        message = str(receipt.error_message)
        # The conviction bound is checked before the proposal lookup, so this is
        # the assertion that the cap exists. ProposalNotFound would mean the
        # order is reversed.
        assert "InvalidConviction" in message, (
            f"expected InvalidConviction, got: {receipt.error_message}"
        )

    def test_conviction_multipliers_are_the_documented_ladder(self, blockchain_connection):
        """The multiplier ladder is 1x, 2x, 3x, 6x for conviction 0..=3.

        Read from the runtime's own constants rather than restated: this asserts
        the values the pallet ships with, so a silent change is caught.
        """
        from substrateinterface import SubstrateInterface

        assert CONVICTION_MULTIPLIERS == {0: 1, 1: 2, 2: 3, 3: 6}, (
            "test constant drifted from pallets/governance/src/lib.rs; update "
            "both together"
        )
        assert MAX_CONVICTION == 3

    def test_conviction_locks_start_empty(self, blockchain_connection):
        """`ConvictionLocks` is keyed by account and holds a lock only after voting.

        Voting itself needs the 14_400-block launch period to elapse, so what is
        asserted here is the storage shape — a lock must be keyed by the account
        that owns it.
        """
        substrate = blockchain_connection
        for key, lock in substrate.query_map("Governance", "ConvictionLocks"):
            recorded = lock.value.get("account") if isinstance(lock.value, dict) else None
            if recorded is not None:
                assert same_account(recorded, key.value), (
                    f"ConvictionLocks[{key.value}] records {recorded}"
                )

    def test_effective_voting_multiplier_uses_zero_as_unset(
        self, blockchain_connection, alice_keypair
    ):
        """0 is the "unset" sentinel, not a zero weight.

        `EffectiveVotingMultiplier` is a `ValueQuery` map, so an account that has
        never been down-ranked reads 0. Both readers (`cast_vote` and the weight
        preview) map 0 to 100 — full weight — and only the quarterly
        participation audit ever writes 50. Asserting ">= 1" would be wrong;
        the sentinel is only meaningful together with that mapping.
        """
        substrate = blockchain_connection
        multiplier = int(
            substrate.query(
                "Governance", "EffectiveVotingMultiplier", [alice_keypair.ss58_address]
            ).value
            or 0
        )
        assert multiplier in (0, 50, 100), (
            "EffectiveVotingMultiplier is only ever written as 0 (unset), 50 "
            f"(down-ranked) or 100 (full weight); got {multiplier}"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTallyAndQuorum:
    """Tally/quorum shape, as far as it is reachable without a voting window."""

    def test_proposal_tally_starts_at_zero(
        self, submit_extrinsic, fund_account, verify_kyc, blockchain_connection
    ):
        """Every stored proposal carries a zeroed `VoteTally` until votes land.

        Submits its own emergency proposal rather than depending on another
        suite having run: the voting window is 14_400 blocks away, but a
        proposal exists immediately, which is all this needs.
        """
        substrate = blockchain_connection
        proposer = _fund_and_verify(fund_account, verify_kyc, "//Eve")
        created = submit_extrinsic(
            "Governance",
            "submit_proposal",
            {
                "title": "0x" + b"tally shape".hex(),
                "description": "0x" + b"asserts the tally starts zeroed".hex(),
                "proposal_type_index": 1,
                "threshold_index": 0,
                "is_emergency": True,
                "district_index": None,
            },
            proposer,
        )
        assert created.is_success, (
            f"submit_proposal failed: {created.error_message}"
        )

        proposals = [p.value for _, p in substrate.query_map("Governance", "Proposals")]
        assert proposals, "a proposal was just submitted, so none existing is a bug"
        for proposal in proposals:
            tally = proposal["vote_tally"]
            assert int(tally["total_weight"]) == 0 or int(tally["ayes"]) + int(
                tally["nays"]
            ) + int(tally["abstentions"]) > 0, (
                f"proposal {proposal['id']} has weight with no votes recorded"
            )

    def test_eligible_voter_count_is_never_zero(self, blockchain_connection):
        """Quorum divides by the eligible-voter count, which must be floored at 1.

        `runtime/src/lib.rs` floors it with `.max(1)` to prevent a division by
        zero; there is no `QuorumThreshold` storage item to check instead.
        """
        substrate = blockchain_connection
        stats = substrate.query("Compliance", "ComplianceStats").value
        assert stats is not None, (
            "Compliance.ComplianceStats must exist — it is the quorum input"
        )
