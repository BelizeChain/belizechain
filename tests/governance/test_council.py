"""District council elections, against the Governance pallet as it actually is.

Real surface used here:

- `start_district_election(district_index, seats, registration_blocks, voting_blocks)`
  — root only; **refuses any district that already has a `DistrictElections`
  entry, and nothing ever removes that entry** (see the test below).
- `register_candidate(district_index, platform)` — signed
- `vote_in_district_election(district_index, candidate)` — signed
- `finalize_district_election(district_index)` — root; needs `block >= voting_end`
- storage: `DistrictElections` keyed by the `BelizeDistrict` **enum**
  (`"Belize"`, `"Cayo"`, …), while the extrinsics take a 0-based `u8`.

The previous version of this file tested `register_council_candidate`,
`trigger_council_election`, `CouncilCandidates`, `CouncilTermLength` and
`CouncilMotionCount`. None of those exist, so every test silently self-skipped
through a `pytest.skip` in `submit_sudo_extrinsic` and the suite reported green
while asserting nothing.
"""

import pytest

from conftest import same_account

DISTRICT_NAMES = ("Belize", "Cayo", "Corozal", "OrangeWalk", "StannCreek", "Toledo")

# Elections run Registration -> Voting -> Finalized, and each phase must be
# crossed by the chain before the next call is valid. These are deliberately
# short (6s blocks) but long enough for the registration extrinsics to land.
REGISTRATION_BLOCKS = 5
VOTING_BLOCKS = 8

# `Governance` error index 49, as `submit_sudo_extrinsic` reports the inner
# error still SCALE-encoded rather than decoded.
ELECTION_ALREADY_ACTIVE = "0x31000000"


def _unused_districts(substrate):
    """District indices that have never held an election.

    `start_district_election` refuses a district whose `DistrictElections` entry
    exists, and finalization does not clear it, so each district is single-use.
    On a chain that has already held all six there is nothing left to open.
    """
    free = []
    for index, name in enumerate(DISTRICT_NAMES):
        if substrate.query("Governance", "DistrictElections", [name]).value is None:
            free.append((index, name))
    return free


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestDistrictElectionGuardrails:
    """Origin and range checks — cheap, and they leave no state behind."""

    def test_out_of_range_district_is_rejected(self, submit_extrinsic, alice_keypair):
        """Valid district indices are 0..=5; 6 must never start an election.

        The previous suite used 1-based districts, which is exactly the kind of
        off-by-one this asserts against.
        """
        receipt = submit_extrinsic(
            "Governance",
            "start_district_election",
            {
                "district_index": 6,
                "seats": 2,
                "registration_period_blocks": 10,
                "voting_period_blocks": 10,
            },
            alice_keypair,
        )
        assert not receipt.is_success, "district index 6 must not start an election"

    def test_start_requires_root(self, submit_extrinsic, alice_keypair):
        """A signed account cannot open an election."""
        index, _ = (0, None)
        receipt = submit_extrinsic(
            "Governance",
            "start_district_election",
            {
                "district_index": index,
                "seats": 1,
                "registration_period_blocks": 10,
                "voting_period_blocks": 10,
            },
            alice_keypair,
        )
        assert not receipt.is_success, "start_district_election must be root-only"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestDistrictElectionLifecycle:
    """Start -> register -> vote -> wait -> finalize, on one fresh district."""

    def test_full_election_cycle(
        self,
        submit_sudo_extrinsic,
        submit_extrinsic,
        verify_kyc,
        wait_for_block,
        blockchain_connection,
        alice_keypair,
        bob_keypair,
        charlie_keypair,
    ):
        substrate = blockchain_connection

        candidates = [alice_keypair, bob_keypair]
        free = _unused_districts(substrate)
        if not free:
            pytest.skip(
                "all six districts have already held an election on this chain. "
                "`start_district_election` refuses a district with an existing "
                "`DistrictElections` entry and finalization never clears it, so "
                "the council cannot be re-elected — see "
                "test_a_district_can_only_hold_one_election"
            )
        district_index, district_name = free[0]

        for account in candidates + [charlie_keypair]:
            verify_kyc(account, 1)

        started = submit_sudo_extrinsic(
            pallet="Governance",
            call="start_district_election",
            params={
                "district_index": district_index,
                "seats": 1,
                "registration_period_blocks": REGISTRATION_BLOCKS,
                "voting_period_blocks": VOTING_BLOCKS,
            },
        )
        assert started["success"], (
            f"start_district_election({district_name}) failed: "
            f"{started.get('error')}"
        )

        election = substrate.query(
            "Governance", "DistrictElections", [district_name]
        ).value
        assert election is not None, (
            "start_district_election succeeded but wrote no DistrictElections "
            "entry — the call and its storage disagree"
        )
        assert election["status"] == "Registration"

        # Signed calls: submitted by the actor, not through sudo, so the origin
        # under test is the candidate's/voter's rather than sudo's.
        for candidate, platform in (
            (candidates[0], b"roads"),
            (candidates[1], b"water"),
        ):
            registered = submit_extrinsic(
                "Governance",
                "register_candidate",
                {
                    "district_index": district_index,
                    "platform": "0x" + platform.hex(),
                },
                candidate,
            )
            assert registered.is_success, (
                f"register_candidate({platform!r}) failed: {registered.error_message}"
            )

        # Voting only opens once the registration window closes.
        # 6s blocks: give the poll more headroom than its 30s default.
        wait_for_block(REGISTRATION_BLOCKS + 2, timeout=120)

        # Two votes for Alice, one for Bob — a strict winner, not a tie.
        for voter, choice in (
            (bob_keypair, candidates[0]),
            (charlie_keypair, candidates[0]),
            (alice_keypair, candidates[1]),
        ):
            cast = submit_extrinsic(
                "Governance",
                "vote_in_district_election",
                {
                    "district_index": district_index,
                    "candidate": choice.ss58_address,
                },
                voter,
            )
            assert cast.is_success, (
                f"vote_in_district_election by {voter.ss58_address} failed: "
                f"{cast.error_message}"
            )

        # finalize needs block >= voting_end.
        wait_for_block(VOTING_BLOCKS + 2, timeout=180)

        finalized = submit_sudo_extrinsic(
            pallet="Governance",
            call="finalize_district_election",
            params={"district_index": district_index},
        )
        assert finalized["success"], (
            f"finalize_district_election({district_name}) failed: "
            f"{finalized.get('error')}"
        )

        after = substrate.query(
            "Governance", "DistrictElections", [district_name]
        ).value
        assert after["status"] == "Finalized", (
            f"expected Finalized, got {after['status']}"
        )

        # `CouncilMember` records no district at all — the district link lives
        # only in `DistrictRepresentation`. Assert what the pallet actually
        # writes rather than a `district` field that does not exist.
        seat = substrate.query(
            "Governance",
            "CouncilMembers",
            [candidates[0].ss58_address],
        ).value
        assert seat is not None, (
            "the candidate with the most votes did not receive the seat"
        )
        assert seat["role"] == "CitizenDelegate", (
            f"district winners should hold CitizenDelegate, got {seat['role']}"
        )
        assert seat["votes_received"] == 2, (
            f"expected 2 recorded votes, got {seat['votes_received']}"
        )
        assert seat["is_rotating"] is True, (
            "district seats must be marked rotating"
        )

        representation = substrate.query(
            "Governance", "DistrictRepresentation", [district_name]
        ).value
        assert any(same_account(addr, candidates[0].ss58_address)
                   for addr in representation), (
            f"{district_name} representation does not list the winner: "
            f"{representation}"
        )
        assert not any(same_account(addr, candidates[1].ss58_address)
                       for addr in representation), (
            "the losing candidate must not take a seat"
        )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestDistrictElectionLimits:
    """The one-shot-per-district limitation, asserted rather than assumed."""

    def test_a_district_can_only_hold_one_election(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """Re-opening a used district is rejected with `ElectionAlreadyActive`.

        This is a real limitation, not a test artefact: `start_district_election`
        requires `!DistrictElections::contains_key(district)` and
        `finalize_district_election` only updates `status`, so a district holds
        at most one election for the life of the chain and a term can never be
        re-run. Captured here so the behaviour is visible instead of surfacing
        as a mysterious skip elsewhere.
        """
        substrate = blockchain_connection

        used = [name for name in DISTRICT_NAMES
                if substrate.query("Governance", "DistrictElections", [name]).value]
        if not used:
            pytest.skip("no district has held an election yet on this chain")

        district_name = used[0]
        district_index = DISTRICT_NAMES.index(district_name)

        res = submit_sudo_extrinsic(
            pallet="Governance",
            call="start_district_election",
            params={
                "district_index": district_index,
                "seats": 1,
                "registration_period_blocks": 10,
                "voting_period_blocks": 10,
            },
        )
        assert not res["success"], (
            "re-opening a district that already held an election must be "
            "refused; if this now succeeds the one-shot limitation was fixed "
            "and the skip in test_full_election_cycle can go"
        )
        assert (
            "ElectionAlreadyActive" in str(res.get("error"))
            or ELECTION_ALREADY_ACTIVE in str(res.get("error"))
        ), f"expected ElectionAlreadyActive (index 49), got: {res.get('error')}"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestCouncilStorage:
    """Storage shape assertions the old suite got wrong."""

    def test_council_members_is_keyed_by_account(self, blockchain_connection):
        """`CouncilMembers` is a StorageMap keyed by account, not a plain list.

        The old test queried it with no key; the resulting exception was
        swallowed into a skip, so it never asserted anything.

        Note: nothing caps the size of this map from the district-election path
        — `finalize_district_election` inserts winners unconditionally and
        `CouncilSizeLimits` is only enforced by `add_board_member`. The "12
        seats across 6 districts" figure the old suite asserted is therefore
        not enforced anywhere, so it is not asserted here.
        """
        substrate = blockchain_connection
        members = list(substrate.query_map("Governance", "CouncilMembers"))
        for key, member in members:
            assert key.value == member.value["account"], (
                "every CouncilMembers entry must be keyed by its own account"
            )

    def test_district_representation_accepts_every_district(
        self, blockchain_connection
    ):
        """`DistrictRepresentation` is keyed by the district enum for all six."""
        substrate = blockchain_connection
        for name in DISTRICT_NAMES:
            # Absent is legitimate before an election; an encode error is not.
            substrate.query("Governance", "DistrictRepresentation", [name])
