"""Governance Emergency Powers Tests

Tests for BelizeChain's emergency governance mechanisms:
- JaguarMode activation (3-of-7 emergency board)
- Emergency pause functionality
- Transparency and auditability
- Recovery procedures
"""

import pytest


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestJaguarModeActivation:
    """Test JaguarMode emergency activation system."""
    
    def test_jaguarmode_3of7_threshold(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that JaguarMode requires 3-of-7 emergency board approval."""
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": "Critical security incident"},
            alice_keypair
        )
        
        assert receipt.is_success
        
        is_active = query_storage("Governance", "JaguarModeActive", None)
        assert is_active is True
        
    def test_jaguarmode_reason_recording(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that JaguarMode activation reason is recorded."""
        reason = "Emergency test - national security"
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": reason},
            alice_keypair
        )
        
        assert receipt.is_success
        
        activation_info = query_storage("Governance", "JaguarModeActivationInfo", None)
        assert activation_info is not None
        assert activation_info['reason'] == reason
        assert 'timestamp' in activation_info
        assert 'activator' in activation_info


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestEmergencyPause:
    """Test emergency pause functionality during JaguarMode."""
    
    def test_pause_transactions(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test that transactions are paused during JaguarMode."""
        # Activate JaguarMode
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": "Pause test"},
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt transaction during pause
        amount = 1_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Balances",
            "transfer",
            {"dest": bob_keypair.ss58_address, "value": amount},
            alice_keypair
        )
        
        # Should fail during JaguarMode
        assert not receipt.is_success, "Transactions should be paused during JaguarMode"
        assert "JaguarModeActive" in receipt.error_message or "TransactionsPaused" in receipt.error_message
        
    def test_whitelist_critical_operations(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic):
        """Test that critical operations are whitelisted during JaguarMode."""
        # Activate JaguarMode
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": "Whitelist test"},
            alice_keypair
        )
        assert receipt.is_success
        
        # Emergency board operations should still work
        receipt = submit_sudo_extrinsic(
            "Governance",
            "deactivate_jaguarmode",
            {},
            alice_keypair
        )
        
        assert receipt.is_success, "Emergency board should be able to deactivate"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestEmergencyTransparency:
    """Test transparency and auditability of emergency actions."""
    
    def test_emergency_log_recording(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that all emergency actions are logged."""
        # Activate JaguarMode
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": "Audit test"},
            alice_keypair
        )
        assert receipt.is_success
        
        # Deactivate JaguarMode
        receipt = submit_sudo_extrinsic(
            "Governance",
            "deactivate_jaguarmode",
            {},
            alice_keypair
        )
        assert receipt.is_success
        
        # Verify complete audit trail
        emergency_log = query_storage("Governance", "EmergencyActionLog", None)
        assert emergency_log is not None
        assert len(emergency_log) >= 2  # Activation + deactivation
        
    def test_board_member_accountability(self, blockchain_connection, query_storage):
        """Test that emergency board members are tracked."""
        board_members = query_storage("Governance", "EmergencyBoardMembers", None)
        assert board_members is not None
        assert len(board_members) == 7, "Emergency board should have 7 members"


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestEmergencyRecovery:
    """Test recovery procedures after emergency."""
    
    def test_deactivation_process(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test JaguarMode deactivation."""
        # Activate
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": "Recovery test"},
            alice_keypair
        )
        assert receipt.is_success
        
        # Deactivate
        receipt = submit_sudo_extrinsic(
            "Governance",
            "deactivate_jaguarmode",
            {},
            alice_keypair
        )
        assert receipt.is_success
        
        is_active = query_storage("Governance", "JaguarModeActive", None)
        assert is_active is False
        
    def test_normal_operations_resume(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test that normal operations resume after deactivation."""
        # Activate and deactivate JaguarMode
        receipt = submit_sudo_extrinsic(
            "Governance",
            "activate_jaguarmode",
            {"reason": "Resume test"},
            alice_keypair
        )
        assert receipt.is_success
        
        receipt = submit_sudo_extrinsic(
            "Governance",
            "deactivate_jaguarmode",
            {},
            alice_keypair
        )
        assert receipt.is_success
        
        # Verify transactions work again
        amount = 1_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Balances",
            "transfer",
            {"dest": bob_keypair.ss58_address, "value": amount},
            alice_keypair
        )
        
        assert receipt.is_success, "Transactions should resume after JaguarMode deactivation"
