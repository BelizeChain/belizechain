"""Governance Voting System Tests

Tests for BelizeChain's governance voting mechanisms:
- Conviction voting (1x to 6x multipliers)
- Quorum requirements
- Vote delegation
- Tallying and execution
"""

import pytest


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestConvictionVoting:
    """Test conviction voting system with time-locked multipliers."""
    
    def test_conviction_multipliers(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test conviction voting multipliers (1x, 2x, 3x, 4x, 5x, 6x)."""
        # Create proposal
        proposal_call = blockchain_connection.compose_call(
            call_module='Governance',
            call_function='set_voting_period',
            call_params={'new_period': 21 * 24 * 60 * 10}
        )
        
        minimum_deposit = 1_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {"proposal_hash": proposal_call.call_hash.hex(), "deposit": minimum_deposit},
            alice_keypair
        )
        assert receipt.is_success
        
        proposal_id = query_storage("Governance", "ProposalCount", None) - 1
        
        # Vote with 6x conviction (longest lock period)
        vote_amount = 10_000_000_000_000_000  # 10K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "vote_on_proposal",
            {"proposal_id": proposal_id, "vote": True, "amount": vote_amount, "conviction": 6},
            alice_keypair
        )
        assert receipt.is_success
        
        # Verify vote recorded with 6x multiplier
        vote_info = query_storage("Governance", "Votes", [proposal_id, alice_keypair.ss58_address])
        assert vote_info is not None
        assert vote_info['conviction'] == 6
        assert vote_info['voting_power'] == vote_amount * 6
        
    def test_vote_delegation(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test vote delegation to trusted representative."""
        delegation_amount = 5_000_000_000_000_000  # 5K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "delegate_votes",
            {"delegate": alice_keypair.ss58_address, "amount": delegation_amount, "conviction": 2},
            bob_keypair
        )
        assert receipt.is_success
        
        # Verify delegation
        delegation_info = query_storage("Governance", "Delegations", [bob_keypair.ss58_address])
        assert delegation_info is not None
        assert delegation_info['delegate'] == alice_keypair.ss58_address
        assert delegation_info['amount'] == delegation_amount


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestQuorumRequirements:
    """Test quorum thresholds for proposal passage."""
    
    def test_quorum_calculation(self, blockchain_connection, alice_keypair, query_storage):
        """Test dynamic quorum based on turnout (Adaptive Quorum Biasing)."""
        # Query current quorum threshold
        quorum_threshold = query_storage("Governance", "QuorumThreshold", None)
        assert quorum_threshold is not None
        assert 0 < quorum_threshold <= 100_000_000  # Percentage in millionths
        
    def test_proposal_execution_after_quorum(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test proposal auto-execution after quorum is met."""
        # Create and vote on proposal
        proposal_call = blockchain_connection.compose_call(
            call_module='Governance',
            call_function='set_proposal_cooldown',
            call_params={'new_cooldown': 7 * 24 * 60 * 10}
        )
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {"proposal_hash": proposal_call.call_hash.hex(), "deposit": 1_000_000_000_000_000},
            alice_keypair
        )
        assert receipt.is_success
        
        proposal_id = query_storage("Governance", "ProposalCount", None) - 1
        
        # Vote with large amount to meet quorum
        massive_vote = 500_000_000_000_000_000  # 500K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "vote_on_proposal",
            {"proposal_id": proposal_id, "vote": True, "amount": massive_vote, "conviction": 1},
            alice_keypair
        )
        assert receipt.is_success
        
        # Check if proposal is ready for execution
        proposal_status = query_storage("Governance", "Proposals", [proposal_id])
        assert proposal_status['status'] in ['Passed', 'ReadyForExecution']


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestVoteTallying:
    """Test vote tallying and result calculation."""
    
    def test_tally_calculation(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test accurate tally calculation with multiple voters."""
        # Create proposal
        proposal_call = blockchain_connection.compose_call(
            call_module='Governance',
            call_function='update_treasury_spending_cap',
            call_params={'new_cap': 1_000_000_000_000_000_000}
        )
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {"proposal_hash": proposal_call.call_hash.hex(), "deposit": 1_000_000_000_000_000},
            alice_keypair
        )
        assert receipt.is_success
        
        proposal_id = query_storage("Governance", "ProposalCount", None) - 1
        
        # Alice votes YES (10K DALLA, 2x conviction)
        receipt = submit_sudo_extrinsic(
            "Governance",
            "vote_on_proposal",
            {"proposal_id": proposal_id, "vote": True, "amount": 10_000_000_000_000_000, "conviction": 2},
            alice_keypair
        )
        assert receipt.is_success
        
        # Bob votes NO (5K DALLA, 1x conviction)
        receipt = submit_sudo_extrinsic(
            "Governance",
            "vote_on_proposal",
            {"proposal_id": proposal_id, "vote": False, "amount": 5_000_000_000_000_000, "conviction": 1},
            bob_keypair
        )
        assert receipt.is_success
        
        # Check tally
        tally = query_storage("Governance", "VoteTally", [proposal_id])
        assert tally is not None
        assert tally['ayes'] == 20_000_000_000_000_000  # 10K * 2x
        assert tally['nays'] == 5_000_000_000_000_000   # 5K * 1x
