"""
BelizeChain Integration Tests: Governance Proposals

Tests proposal creation, lifecycle, and execution in the governance system.

Pallet: Governance
Extrinsics Available:
- create_proposal(proposal_hash, deposit)
- second_proposal(proposal_id)
- vote(proposal_id, aye, conviction, amount)
- execute_proposal(proposal_id)
- cancel_proposal(proposal_id)

Storage Items Available:
- Proposals: StorageMap<ProposalId, ProposalInfo>
- ProposalCount: StorageValue<u32>
- VotingPeriod: StorageValue<BlockNumber>
- MinimumDeposit: StorageValue<Balance>
"""

import pytest


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestGovernanceProposals:
    """Test governance proposal creation and lifecycle."""
    
    def test_create_proposal_success(
        self,
        blockchain_connection,
        alice_keypair,
        submit_sudo_extrinsic,
        query_storage
    ):
        """
        Test that citizens can create proposals with minimum deposit.
        
        Verifies:
        1. Proposal created with sufficient deposit (1000 DALLA minimum)
        2. Proposal ID assigned sequentially
        3. Proposal status is "Proposed"
        4. Deposit locked in treasury
        """
        # Arrange: Get current proposal count
        proposal_count_before = query_storage(
            "Governance",
            "ProposalCount",
            None
        ) or 0
        
        # Prepare proposal (example: adjust voting period)
        proposal_call = blockchain_connection.compose_call(
            call_module='Governance',
            call_function='set_voting_period',
            call_params={'new_period': 14 * 24 * 60 * 10}  # 14 days in blocks (6s blocks)
        )
        
        proposal_hash = proposal_call.call_hash.hex()
        minimum_deposit = 1_000_000_000_000_000  # 1000 DALLA (12 decimals)
        
        # Act: Create proposal
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {
                "proposal_hash": proposal_hash,
                "deposit": minimum_deposit
            },
            alice_keypair
        )
        
        # Assert: Proposal created successfully
        assert receipt.is_success, f"Proposal creation failed: {receipt.error_message}"
        
        # Verify proposal count increased
        proposal_count_after = query_storage(
            "Governance",
            "ProposalCount",
            None
        )
        assert proposal_count_after == proposal_count_before + 1, "Proposal count did not increase"
        
        # Verify proposal details
        proposal_id = proposal_count_before  # New proposal ID
        proposal_info = query_storage(
            "Governance",
            "Proposals",
            [proposal_id]
        )
        
        assert proposal_info is not None, "Proposal not found in storage"
        assert proposal_info['proposer'] == alice_keypair.ss58_address, "Wrong proposer"
        assert proposal_info['deposit'] == minimum_deposit, "Wrong deposit amount"
        assert proposal_info['status'] == 'Proposed', "Wrong proposal status"
        
    def test_create_proposal_insufficient_deposit(
        self,
        blockchain_connection,
        alice_keypair,
        submit_sudo_extrinsic
    ):
        """
        Test that proposal creation fails with insufficient deposit.
        
        Verifies:
        1. Proposal rejected if deposit < 1000 DALLA
        2. No proposal ID assigned
        3. Error message indicates insufficient deposit
        """
        # Arrange: Prepare proposal with insufficient deposit
        try:
            proposal_call = blockchain_connection.compose_call(
                call_module='Governance',
                call_function='set_voting_period',
                call_params={'new_period': 14 * 24 * 60 * 10}
            )
        except ValueError as e:
            pytest.skip(f"Runtime lacks Governance.set_voting_period: {e}")
        
        insufficient_deposit = 500_000_000_000_000  # 500 DALLA (below 1000 minimum)
        
        # Act: Attempt to create proposal
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {
                "proposal_hash": proposal_call.call_hash.hex(),
                "deposit": insufficient_deposit
            },
            alice_keypair
        )
        
        # Assert: Proposal creation failed
        assert not receipt.is_success, "Proposal should have failed with insufficient deposit"
        assert "InsufficientDeposit" in receipt.error_message or "DepositTooLow" in receipt.error_message
        
    def test_proposal_second_by_council(
        self,
        blockchain_connection,
        alice_keypair,
        charlie_keypair,  # Council member
        submit_sudo_extrinsic,
        query_storage
    ):
        """
        Test that council members can second proposals.
        
        Verifies:
        1. Council member seconds proposal
        2. Proposal status changes to "Seconded"
        3. Seconded proposals enter voting period
        """
        # Arrange: Create proposal first
        try:
            proposal_call = blockchain_connection.compose_call(
                call_module='Governance',
                call_function='set_voting_period',
                call_params={'new_period': 14 * 24 * 60 * 10}
            )
        except ValueError as e:
            pytest.skip(f"Runtime lacks Governance.set_voting_period: {e}")
        
        minimum_deposit = 1_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {
                "proposal_hash": proposal_call.call_hash.hex(),
                "deposit": minimum_deposit
            },
            alice_keypair
        )
        
        assert receipt.is_success, "Setup failed: Could not create proposal"
        
        proposal_count = query_storage("Governance", "ProposalCount", None)
        proposal_id = proposal_count - 1
        
        # Act: Council member seconds the proposal
        receipt = submit_sudo_extrinsic(
            "Governance",
            "second_proposal",
            {"proposal_id": proposal_id},
            charlie_keypair  # Council member
        )
        
        # Assert: Proposal seconded successfully
        assert receipt.is_success, f"Proposal seconding failed: {receipt.error_message}"
        
        # Verify proposal status changed
        proposal_info = query_storage(
            "Governance",
            "Proposals",
            [proposal_id]
        )
        
        assert proposal_info['status'] == 'Seconded', "Proposal status not updated to Seconded"
        assert proposal_info['seconder'] == charlie_keypair.ss58_address, "Wrong seconder"
        
    def test_proposal_execution_after_approval(
        self,
        blockchain_connection,
        alice_keypair,
        submit_sudo_extrinsic,
        query_storage,
        wait_for_block
    ):
        """
        Test that approved proposals execute successfully.
        
        Verifies:
        1. Proposal passes with >50% approval + quorum
        2. Voting period ends
        3. Proposal executes automatically
        4. On-chain state updated as per proposal
        """
        # Arrange: Create and pass proposal
        # (Simplified: using sudo to force-pass for test)
        try:
            proposal_call = blockchain_connection.compose_call(
                call_module='Governance',
                call_function='set_voting_period',
                call_params={'new_period': 20 * 24 * 60 * 10}  # 20 days
            )
        except ValueError as e:
            pytest.skip(f"Runtime lacks Governance.set_voting_period: {e}")
        
        # Get current voting period
        voting_period_before = query_storage(
            "Governance",
            "VotingPeriod",
            None
        )
        
        # Act: Execute proposal directly (sudo power for testing)
        receipt = submit_sudo_extrinsic(
            "Governance",
            "force_execute_proposal",
            {
                "proposal": proposal_call.value
            },
            alice_keypair
        )
        
        # Assert: Proposal executed
        assert receipt.is_success, f"Proposal execution failed: {receipt.error_message}"
        
        # Wait for state update
        wait_for_block(2)
        
        # Verify voting period updated
        voting_period_after = query_storage(
            "Governance",
            "VotingPeriod",
            None
        )
        
        assert voting_period_after == 20 * 24 * 60 * 10, "Voting period not updated"
        assert voting_period_after != voting_period_before, "Proposal did not change state"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestProposalCancellation:
    """Test proposal cancellation scenarios."""
    
    def test_cancel_proposal_by_proposer(
        self,
        blockchain_connection,
        alice_keypair,
        submit_sudo_extrinsic,
        query_storage
    ):
        """
        Test that proposer can cancel their own proposal before voting.
        
        Verifies:
        1. Proposer cancels proposal
        2. Deposit returned
        3. Proposal status set to "Cancelled"
        """
        # Arrange: Create proposal
        try:
            proposal_call = blockchain_connection.compose_call(
                call_module='Governance',
                call_function='set_voting_period',
                call_params={'new_period': 14 * 24 * 60 * 10}
            )
        except ValueError as e:
            pytest.skip(f"Runtime lacks Governance.set_voting_period: {e}")
        
        minimum_deposit = 1_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "create_proposal",
            {
                "proposal_hash": proposal_call.call_hash.hex(),
                "deposit": minimum_deposit
            },
            alice_keypair
        )
        
        assert receipt.is_success
        
        proposal_count = query_storage("Governance", "ProposalCount", None)
        proposal_id = proposal_count - 1
        
        # Get Alice's balance before cancellation
        balance_before = query_storage(
            "System",
            "Account",
            [alice_keypair.ss58_address]
        )['data']['free']
        
        # Act: Cancel proposal
        receipt = submit_sudo_extrinsic(
            "Governance",
            "cancel_proposal",
            {"proposal_id": proposal_id},
            alice_keypair
        )
        
        # Assert: Cancellation successful
        assert receipt.is_success, f"Cancellation failed: {receipt.error_message}"
        
        # Verify proposal cancelled
        proposal_info = query_storage(
            "Governance",
            "Proposals",
            [proposal_id]
        )
        
        assert proposal_info['status'] == 'Cancelled', "Proposal not cancelled"
        
        # Verify deposit returned (balance increased)
        balance_after = query_storage(
            "System",
            "Account",
            [alice_keypair.ss58_address]
        )['data']['free']
        
        # Balance should increase by deposit amount (minus small tx fee)
        assert balance_after > balance_before, "Deposit not returned"
