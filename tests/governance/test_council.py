"""Governance Council System Tests

Tests for BelizeChain's council governance:
- District-based elections (6 districts, 12 seats)
- Term limits and rotation
- Council motions and voting
- Collective decision-making
"""

import pytest


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestCouncilElections:
    """Test council election system."""
    
    def test_candidate_registration(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test council candidate registration with stake requirement."""
        # Register Bob as candidate for District 1
        stake = 10_000_000_000_000_000  # 10K DALLA minimum stake
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "register_council_candidate",
            {"district": 1, "stake": stake},
            bob_keypair
        )
        
        assert receipt.is_success
        
        candidates = query_storage("Governance", "CouncilCandidates", [1])  # District 1
        assert any(c['account'] == bob_keypair.ss58_address for c in candidates)
        
    def test_district_seat_allocation(self, blockchain_connection, query_storage):
        """Test that 12 seats are allocated across 6 districts (2 per district)."""
        council_members = query_storage("Governance", "CouncilMembers", None)
        assert council_members is not None
        assert len(council_members) <= 12
        
        # Verify district distribution
        districts = [member['district'] for member in council_members]
        for district in range(1, 7):
            district_count = districts.count(district)
            assert district_count <= 2, f"District {district} should have max 2 seats"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestCouncilMotions:
    """Test council motion system."""
    
    def test_create_council_motion(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test creating council motion."""
        try:
            motion_call = blockchain_connection.compose_call(
                call_module='Governance',
                call_function='update_election_frequency',
                call_params={'new_frequency': 365 * 24 * 60 * 10}
            )
        except ValueError as e:
            pytest.skip(f"Runtime lacks Governance.update_election_frequency: {e}")
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_council_motion",
            {"proposal_hash": motion_call.call_hash.hex(), "threshold": 7},  # 7-of-12 majority
            alice_keypair
        )
        
        assert receipt.is_success
        
        motion_count = query_storage("Governance", "CouncilMotionCount", None)
        assert motion_count > 0
        
    def test_council_voting_threshold(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that motions require simple majority (7-of-12)."""
        # Create motion
        try:
            motion_call = blockchain_connection.compose_call(
                call_module='Governance',
                call_function='set_council_term_length',
                call_params={'new_length': 2 * 365 * 24 * 60 * 10}
            )
        except ValueError as e:
            pytest.skip(f"Runtime lacks Governance.set_council_term_length: {e}")
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_council_motion",
            {"proposal_hash": motion_call.call_hash.hex(), "threshold": 7},
            alice_keypair
        )
        assert receipt.is_success
        
        motion_id = query_storage("Governance", "CouncilMotionCount", None) - 1
        
        # Verify threshold
        motion_info = query_storage("Governance", "CouncilMotions", [motion_id])
        assert motion_info['threshold'] == 7


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestCouncilTermLimits:
    """Test council term limits and rotation."""
    
    def test_term_length_enforcement(self, blockchain_connection, query_storage):
        """Test that council terms are limited (default 2 years)."""
        term_length = query_storage("Governance", "CouncilTermLength", None)
        assert term_length is not None
        assert term_length == 2 * 365 * 24 * 60 * 10  # 2 years in blocks (~6s blocks)
        
    def test_council_rotation(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic):
        """Test council member rotation after term expiration."""
        # Trigger election rotation
        receipt = submit_sudo_extrinsic(
            "Governance",
            "trigger_council_election",
            {},
            alice_keypair
        )
        
        assert receipt.is_success
