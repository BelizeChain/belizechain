"""District council elections, against the Governance pallet as it actually is.

Real surface used here:

- `start_district_election(district_index, seats, registration_blocks, voting_blocks)`
  — root only; **refuses a district whose current election is still in
  `Registration` or `Voting`**. A `Finalized`/`Cancelled` record is history and
  may be replaced (see `TestDistrictElectionLimits`).
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

#: Election statuses that keep a district's slot locked. `Finalized` and
#: `Cancelled` are history, not a lock — see `_districts_without_active_election`.
ACTIVE_STATUSES = ("Registration", "Voting")


def _district_election(substrate, district_name):
    """Current `DistrictElections` record for a district, or `None`."""
    return substrate.query("Governance", "DistrictElections", [district_name]).value


def _districts_without_active_election(substrate):
    """District indices whose election slot is currently free.

    `start_district_election` rejects only an election still in `Registration` or
    `Voting`. It used to require `!DistrictElections::contains_key(district)` and
    nothing ever removed the entry, which made all six districts single-use and
    the council un-re-electable; that rule was removed, so a district whose
    election finished is available again.
    """
    free = []
    for index, name in enumerate(DISTRICT_NAMES):
        election = _district_election(substrate, name)
        if election is None or election.get("status") not in ACTIVE_STATUSES:
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
        free = _districts_without_active_election(substrate)
        if not free:
            pytest.skip(
                "all six districts already have an election in Registration or "
                "Voting on this chain, so there is no free slot to open one in"
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
    """The district election-slot guard, asserted rather than assumed.

    `start_district_election` originally required that `DistrictElections` held no
    entry at all, and nothing ever removed one — so each of the six districts was
    single-use, the chain allowed six elections in total, and the council could
    never be re-elected. That rule was removed: the guard now rejects only an
    election still in `Registration` or `Voting`.

    Both halves are asserted, because a regression in either direction is silent.
    This class previously asserted the limitation itself still held, which is how
    it failed — loudly, as designed — once the rule was fixed.
    """

    def test_a_finished_election_can_be_reopened(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """A district whose election finished accepts a new one.

        Note this consumes a free slot: the district is left in `Registration`,
        which locks it until `finalize_district_election` runs.
        """
        substrate = blockchain_connection

        finished = [
            (index, name)
            for index, name in enumerate(DISTRICT_NAMES)
            if (_district_election(substrate, name) or {}).get("status")
            in ("Finalized", "Cancelled")
        ]
        if not finished:
            pytest.skip("no district has finished an election yet on this chain")

        district_index, district_name = finished[0]

        res = submit_sudo_extrinsic(
            pallet="Governance",
            call="start_district_election",
            params={
                "district_index": district_index,
                "seats": 1,
                "registration_period_blocks": REGISTRATION_BLOCKS,
                "voting_period_blocks": VOTING_BLOCKS,
            },
        )
        assert res["success"], (
            f"{district_name} already finished an election, so re-opening it must be "
            f"allowed — the one-shot-per-district rule was removed. Got: "
            f"{res.get('error')}"
        )

        reopened = _district_election(substrate, district_name)
        assert reopened is not None and reopened["status"] == "Registration", (
            "start_district_election succeeded but left the district in "
            f"{reopened and reopened.get('status')!r}, not Registration"
        )

    def test_an_active_election_blocks_a_second_one(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """A district with an election in flight refuses a second one."""
        substrate = blockchain_connection

        active = [
            (index, name)
            for index, name in enumerate(DISTRICT_NAMES)
            if (_district_election(substrate, name) or {}).get("status")
            in ACTIVE_STATUSES
        ]
        if not active:
            pytest.skip("no district has an election in flight on this chain")

        district_index, district_name = active[0]

        res = submit_sudo_extrinsic(
            pallet="Governance",
            call="start_district_election",
            params={
                "district_index": district_index,
                "seats": 1,
                "registration_period_blocks": REGISTRATION_BLOCKS,
                "voting_period_blocks": VOTING_BLOCKS,
            },
        )
        assert not res["success"], (
            f"{district_name} has an election in flight; starting a second one must "
            "be refused"
        )
        assert "ElectionAlreadyActive" in str(res.get("error")), (
            f"expected ElectionAlreadyActive, got: {res.get('error')}"
        )


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
