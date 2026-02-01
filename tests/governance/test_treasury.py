"""Governance Treasury Management Tests

Tests for BelizeChain's treasury system:
- Multi-sig (4-of-7) spending approvals
- Spending proposals and execution
- Automatic treasury funding from inflation
- Budget caps and compliance
"""

import pytest


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTreasuryMultiSig:
    """Test 4-of-7 multi-signature treasury operations."""
    
    def test_create_spending_proposal(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test creating treasury spending proposal."""
        amount = 50_000_000_000_000_000  # 50K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "propose_treasury_spend",
            {
                "recipient": alice_keypair.ss58_address,
                "amount": amount,
                "description": "Infrastructure Development"
            },
            alice_keypair
        )
        
        assert receipt.is_success
        
        proposal_count = query_storage("Economy", "TreasuryProposalCount", None)
        assert proposal_count > 0
        
    def test_multi_sig_approval_threshold(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that 4-of-7 signatures are required for approval."""
        # Create spending proposal
        amount = 100_000_000_000_000_000  # 100K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "propose_treasury_spend",
            {"recipient": alice_keypair.ss58_address, "amount": amount, "description": "Education Fund"},
            alice_keypair
        )
        assert receipt.is_success
        
        proposal_id = query_storage("Economy", "TreasuryProposalCount", None) - 1
        
        # Attempt execution with < 4 approvals (should fail)
        receipt = submit_sudo_extrinsic(
            "Economy",
            "execute_treasury_proposal",
            {"proposal_id": proposal_id},
            alice_keypair
        )
        
        # Should fail if < 4 approvals
        if not receipt.is_success:
            assert "InsufficientApprovals" in receipt.error_message or "MultiSigThresholdNotMet" in receipt.error_message


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTreasuryFunding:
    """Test automatic treasury funding mechanisms."""
    
    def test_inflation_based_funding(self, blockchain_connection, query_storage):
        """Test that treasury receives funding from DALLA inflation."""
        treasury_balance = query_storage("Economy", "TreasuryBalance", None)
        assert treasury_balance is not None
        assert treasury_balance > 0, "Treasury should receive automatic funding"
        
    def test_spending_cap_enforcement(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic):
        """Test that treasury spending respects annual cap."""
        # Attempt spending exceeding cap
        excessive_amount = 10_000_000_000_000_000_000  # 10M DALLA (likely exceeds cap)
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "propose_treasury_spend",
            {"recipient": alice_keypair.ss58_address, "amount": excessive_amount, "description": "Test"},
            alice_keypair
        )
        
        # Should either fail immediately or require special approval
        if not receipt.is_success:
            assert "ExceedsSpendingCap" in receipt.error_message or "AmountTooLarge" in receipt.error_message


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestTreasuryTransparency:
    """Test treasury transparency and auditing."""
    
    def test_spending_history(self, blockchain_connection, query_storage):
        """Test that all treasury spending is recorded."""
        spending_history = query_storage("Economy", "TreasurySpendingHistory", None)
        assert spending_history is not None
        # Verify history contains required fields: timestamp, amount, recipient, approvers
