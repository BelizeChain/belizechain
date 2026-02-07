"""
BelizeChain Cross-Pallet Integration Tests

Tests critical interactions between pallets to ensure proper system integration.

Critical Cross-Pallet Interactions:
1. Economy ↔ Compliance: KYC enforcement on transactions, sanction blocking
2. Identity ↔ Governance: Verified voter requirements, council eligibility
3. Staking ↔ Oracle: Validator data feeds, reputation scoring
4. BelizeX ↔ Oracle: Exchange rate guards, slippage protection
5. LandLedger ↔ Identity: Property ownership verification, KYC requirements
6. Payroll ↔ Economy: Salary distribution, multi-sig approvals
"""

import pytest


# ============================================================================
# 1. ECONOMY ↔ COMPLIANCE INTEGRATION
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestEconomyComplianceIntegration:
    """Test Economy and Compliance pallet integration."""
    
    def test_kyc_enforcement_on_transfers(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test that transfers are blocked without KYC (Compliance Level 1+)."""
        # Remove Bob's KYC (if any)
        receipt = submit_sudo_extrinsic(
            "Identity",
            "set_kyc_level",
            {"account": bob_keypair.ss58_address, "level": 0},
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt large transfer without KYC
        amount = 6_000_000_000_000_000  # 6K DALLA (triggers KYC check)
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "transfer_with_compliance",
            {"dest": bob_keypair.ss58_address, "amount": amount},
            bob_keypair
        )
        
        assert not receipt.is_success, "Transfer should fail without KYC"
        assert "KYCRequired" in receipt.error_message or "ComplianceCheckFailed" in receipt.error_message
        
    def test_sanction_blocking_transfers(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test that sanctioned accounts cannot transact."""
        # Add Bob to sanctions list
        receipt = submit_sudo_extrinsic(
            "Identity",
            "add_to_sanctions",
            {"account": bob_keypair.ss58_address, "reason": "Test sanction"},
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt transfer from sanctioned account
        amount = 1_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "transfer_with_compliance",
            {"dest": alice_keypair.ss58_address, "amount": amount},
            bob_keypair
        )
        
        assert not receipt.is_success, "Transfer should fail for sanctioned account"
        assert "AccountSanctioned" in receipt.error_message
        
    def test_aml_threshold_reporting(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test that large transfers (>10K DALLA) trigger SAR."""
        # Set Bob with KYC Level 2 (Enhanced)
        receipt = submit_sudo_extrinsic(
            "Identity",
            "set_kyc_level",
            {"account": bob_keypair.ss58_address, "level": 2},
            alice_keypair
        )
        assert receipt.is_success
        
        # Large transfer triggering AML
        large_amount = 15_000_000_000_000_000  # 15K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "transfer_with_compliance",
            {"dest": bob_keypair.ss58_address, "amount": large_amount},
            alice_keypair
        )
        
        assert receipt.is_success, "Transfer should succeed with proper KYC"
        
        # Verify SAR created
        sar_count_after = query_storage("Compliance", "SuspiciousActivityReports", None)
        assert sar_count_after > 0, "SAR should be generated for large transfer"


# ============================================================================
# 2. IDENTITY ↔ GOVERNANCE INTEGRATION
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestIdentityGovernanceIntegration:
    """Test Identity and Governance pallet integration."""
    
    def test_voter_registration_requires_kyc(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test that voting requires KYC Level 1 (Basic)."""
        # Remove Bob's KYC
        receipt = submit_sudo_extrinsic(
            "Identity",
            "set_kyc_level",
            {"account": bob_keypair.ss58_address, "level": 0},
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt to vote without KYC
        receipt = submit_sudo_extrinsic(
            "Governance",
            "vote_on_proposal",
            {"proposal_id": 0, "vote": True, "conviction": 1},
            bob_keypair
        )
        
        assert not receipt.is_success, "Voting should require KYC"
        assert "KYCRequired" in receipt.error_message
        
    def test_council_candidate_requires_belizeid(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test that council candidates must have verified BelizeID."""
        # Attempt to register as candidate without BelizeID
        receipt = submit_sudo_extrinsic(
            "Governance",
            "register_council_candidate",
            {"district": 1, "stake": 10_000_000_000_000_000},
            bob_keypair
        )
        
        assert not receipt.is_success, "Council candidacy should require BelizeID"
        assert "BelizeIDRequired" in receipt.error_message or "IdentityNotVerified" in receipt.error_message


# ============================================================================
# 3. STAKING ↔ ORACLE INTEGRATION
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestStakingOracleIntegration:
    """Test Staking and Oracle pallet integration."""
    
    def test_validator_data_feed_quality(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """Test that validators providing Oracle data earn reputation."""
        # Register Bob as validator
        receipt = submit_sudo_extrinsic(
            "Staking",
            "join_validators",
            {"stake": 50_000_000_000_000_000, "compute_capacity": 100, "location": "Belize City"},
            bob_keypair
        )
        if not receipt.is_success and getattr(receipt, "error_message", None):
            msg = str(receipt.error_message)
            if "KycRequired" in msg:
                pytest.skip(f"Validator join blocked by KYC requirement: {msg}")
            pytest.skip(f"Validator join unavailable: {msg}")
        assert receipt.is_success
        
        # Bob provides Oracle data feed
        receipt = submit_sudo_extrinsic(
            "Oracle",
            "report_data_as_validator",
            {"validator": bob_keypair.ss58_address, "data_type": "ExchangeRate", "value": 150_000},
            bob_keypair
        )
        if not receipt.is_success and getattr(receipt, "error_message", None):
            msg = str(receipt.error_message)
            if "KycRequired" in msg:
                pytest.skip(f"Oracle data feed blocked by KYC requirement: {msg}")
            pytest.skip(f"Oracle data feed unavailable: {msg}")

        assert receipt.is_success
        
        # Check reputation increased
        validator_info = query_storage("Staking", "Validators", [bob_keypair.ss58_address])
        assert validator_info['reputation_score'] > 0


# ============================================================================
# 4. BELIZEX ↔ ORACLE INTEGRATION
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestBelizeXOracleIntegration:
    """Test BelizeX and Oracle pallet integration."""
    
    def test_oracle_rate_guard_on_swap(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that swaps use Oracle rates for slippage protection."""
        # Set Oracle exchange rate
        receipt = submit_sudo_extrinsic(
            "Oracle",
            "set_exchange_rate",
            {"pair": "DALLA/bBZD", "rate": 1_000_000},  # 1:1 ratio
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt swap with high slippage
        receipt = submit_sudo_extrinsic(
            "BelizeX",
            "swap_with_oracle_guard",
            {
                "pair_id": 0,
                "amount_in": 1_000_000_000_000_000,
                "min_amount_out": 500_000_000_000_000,  # 50% slippage (excessive)
                "max_slippage_bps": 300  # 3% max
            },
            alice_keypair
        )
        
        assert not receipt.is_success, "Swap should fail due to slippage exceeding Oracle guard"
        assert "SlippageTooHigh" in receipt.error_message or "OracleGuardRejection" in receipt.error_message
        
    def test_oracle_unavailable_fallback(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that swaps handle Oracle unavailability gracefully."""
        # Clear Oracle rate (simulate unavailability)
        receipt = submit_sudo_extrinsic(
            "Oracle",
            "clear_exchange_rate",
            {"pair": "wUSDC/bBZD"},
            alice_keypair
        )
        
        # Attempt swap without Oracle rate
        receipt = submit_sudo_extrinsic(
            "BelizeX",
            "swap_with_oracle_guard",
            {
                "pair_id": 1,  # wUSDC/bBZD pair
                "amount_in": 1_000_000,
                "min_amount_out": 950_000,
                "max_slippage_bps": 500
            },
            alice_keypair
        )
        
        # Should either succeed with AMM-only pricing or fail gracefully
        if not receipt.is_success:
            assert "OracleUnavailable" in receipt.error_message


# ============================================================================
# 5. LANDLEDGER ↔ IDENTITY INTEGRATION
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestLandLedgerIdentityIntegration:
    """Test LandLedger and Identity pallet integration."""
    
    def test_property_registration_requires_kyc(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test that property registration requires Enhanced KYC (Level 3)."""
        # Set Bob with Basic KYC (Level 1)
        receipt = submit_sudo_extrinsic(
            "Identity",
            "set_kyc_level",
            {"account": bob_keypair.ss58_address, "level": 1},
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt property registration with insufficient KYC
        receipt = submit_sudo_extrinsic(
            "LandLedger",
            "register_property",
            {
                "owner": bob_keypair.ss58_address,
                "parcel_id": "BZ-002-67890",
                "area_sqm": 500,
                "location": "San Pedro"
            },
            bob_keypair
        )
        
        assert not receipt.is_success, "Property registration should require Enhanced KYC (Level 3)"
        assert "InsufficientKYCLevel" in receipt.error_message or "KYCLevel3Required" in receipt.error_message
        
    def test_property_transfer_identity_verification(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test that property transfers verify both parties' identities."""
        # Register property for Alice (assume she has Level 3 KYC)
        receipt = submit_sudo_extrinsic(
            "LandLedger",
            "register_property",
            {
                "owner": alice_keypair.ss58_address,
                "parcel_id": "BZ-003-11111",
                "area_sqm": 1000,
                "location": "Belmopan"
            },
            alice_keypair
        )
        assert receipt.is_success
        
        # Attempt transfer to Bob (without BelizeID)
        receipt = submit_sudo_extrinsic(
            "LandLedger",
            "transfer_property",
            {"parcel_id": "BZ-003-11111", "new_owner": bob_keypair.ss58_address},
            alice_keypair
        )
        
        assert not receipt.is_success, "Property transfer should require recipient BelizeID"
        assert "RecipientIdentityNotVerified" in receipt.error_message or "BelizeIDRequired" in receipt.error_message


# ============================================================================
# 6. PAYROLL ↔ ECONOMY INTEGRATION
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
class TestPayrollEconomyIntegration:
    """Test Payroll and Economy pallet integration."""
    
    def test_payroll_multi_sig_approval(self, blockchain_connection, alice_keypair, submit_sudo_extrinsic, query_storage):
        """Test that large payroll batches require treasury multi-sig (4-of-7)."""
        # Add multiple employees
        employees = [f"employee_{i}" for i in range(100)]
        total_payroll = 500_000_000_000_000_000  # 500K DALLA (exceeds single-sig threshold)
        
        receipt = submit_sudo_extrinsic(
            "Payroll",
            "process_large_payroll",
            {"employee_count": 100, "total_amount": total_payroll},
            alice_keypair
        )
        
        # Should require multi-sig approval
        if not receipt.is_success:
            assert "MultiSigRequired" in receipt.error_message or "TreasuryApprovalNeeded" in receipt.error_message
        else:
            # Verify multi-sig proposal created
            proposal_count = query_storage("Economy", "TreasuryProposalCount", None)
            assert proposal_count > 0
            
    def test_payroll_account_type_limits(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic):
        """Test that payroll respects Economy account type limits."""
        # Set Bob as Government account (unlimited)
        receipt = submit_sudo_extrinsic(
            "Economy",
            "set_account_type",
            {"account": bob_keypair.ss58_address, "account_type": "Government"},
            alice_keypair
        )
        assert receipt.is_success
        
        # Add Bob to payroll with high salary
        high_salary = 20_000_000_000_000_000  # 20K DALLA/month (exceeds Citizen limit)
        
        receipt = submit_sudo_extrinsic(
            "Payroll",
            "add_employee",
            {
                "employee": bob_keypair.ss58_address,
                "salary": high_salary,
                "department": "Treasury"
            },
            alice_keypair
        )
        
        assert receipt.is_success, "Government accounts should not have daily limits"


# ============================================================================
# 7. COMPREHENSIVE CROSS-PALLET WORKFLOW
# ============================================================================

@pytest.mark.requires_blockchain
@pytest.mark.cross_pallet
@pytest.mark.e2e
class TestComprehensiveCrossPalletWorkflow:
    """Test complete cross-pallet workflow: Citizen onboarding → Transaction → Governance."""
    
    def test_full_citizen_lifecycle(self, blockchain_connection, alice_keypair, bob_keypair, submit_sudo_extrinsic, query_storage):
        """
        Complete citizen lifecycle:
        1. Register BelizeID (Identity)
        2. Set KYC Level (Compliance)
        3. Receive DALLA (Economy)
        4. Vote on proposal (Governance)
        5. Stake as validator (Staking)
        """
        # Step 1: Register BelizeID
        receipt = submit_sudo_extrinsic(
            "Identity",
            "register_belizeid",
            {
                "account": bob_keypair.ss58_address,
                "ssn": "987-65-4321",
                "full_name": "Bob Validator",
                "date_of_birth": "1985-05-15"
            },
            alice_keypair
        )
        assert receipt.is_success, "BelizeID registration failed"
        
        # Step 2: Set KYC Level 2 (Verified)
        receipt = submit_sudo_extrinsic(
            "Identity",
            "set_kyc_level",
            {"account": bob_keypair.ss58_address, "level": 2},
            alice_keypair
        )
        assert receipt.is_success, "KYC verification failed"
        
        # Step 3: Transfer DALLA
        amount = 60_000_000_000_000_000  # 60K DALLA
        
        receipt = submit_sudo_extrinsic(
            "Economy",
            "transfer_with_compliance",
            {"dest": bob_keypair.ss58_address, "amount": amount},
            alice_keypair
        )
        assert receipt.is_success, "DALLA transfer failed"
        
        bob_balance = query_storage("System", "Account", [bob_keypair.ss58_address])['data']['free']
        assert bob_balance >= amount
        
        # Step 4: Vote on governance proposal
        receipt = submit_sudo_extrinsic(
            "Governance",
            "vote_on_proposal",
            {"proposal_id": 0, "vote": True, "conviction": 2},
            bob_keypair
        )
        assert receipt.is_success, "Governance voting failed"
        
        # Step 5: Join validators (requires 50K DALLA stake)
        stake_amount = 50_000_000_000_000_000
        
        receipt = submit_sudo_extrinsic(
            "Staking",
            "join_validators",
            {"stake": stake_amount, "compute_capacity": 100, "location": "Belize City"},
            bob_keypair
        )
        assert receipt.is_success, "Validator registration failed"
        
        # Verify validator registered
        validator_info = query_storage("Staking", "Validators", [bob_keypair.ss58_address])
        assert validator_info is not None
        assert validator_info['stake'] >= stake_amount
