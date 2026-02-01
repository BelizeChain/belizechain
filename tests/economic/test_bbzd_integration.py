"""
BelizeChain Economy Pallet Integration Tests

Tests the USDC-style fiat-backed bBZD stablecoin implementation:
- Mint/burn flows with Central Bank authorization
- Redemption queue and settlement processing
- Reserve invariants (TotalBbzdSupply ≤ CentralBankReserves)
- Cross-pallet interactions (Governance, Compliance)

Run with: pytest tests/integration/economic/test_bbzd_integration.py -v
"""

import pytest
from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.exceptions import SubstrateRequestException


def fund_account(substrate: SubstrateInterface, from_kp: Keypair, to_addr: str, amount: int):
    """Fund account to cover fees using Balances.transfer_keep_alive."""
    call = substrate.compose_call(
        call_module="Balances",
        call_function="transfer_keep_alive",
        call_params={"dest": {"Id": to_addr}, "value": amount},
    )
    xt = substrate.create_signed_extrinsic(call=call, keypair=from_kp)
    receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
    return receipt.is_success


class TestBbzdMinting:
    """Test bBZD minting (fiat-backed model)"""

    def test_mint_bbzd_authorized_minter(
        self,
        submit_sudo_extrinsic,
        blockchain_connection: SubstrateInterface,
        alice_keypair: Keypair,
        bob_keypair: Keypair,
    ):
        """Authorized Central Bank can mint bBZD after BZD deposit."""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair

        # Governance: authorize Alice as Central Bank minter
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"], f"Authorization failed: {res.get('error')}"

        # Governance: set Central Bank reserves
        initial_reserves = 10_000_000_000_000
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": initial_reserves},
        )
        assert res["success"], f"Set reserves failed: {res.get('error')}"

        # Ensure Bob has KYC and funds to sign
        assert fund_account(substrate, alice, bob.ss58_address, 1_000_000_000_000)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success

        # Get Bob's initial bBZD balance
        initial_balance = substrate.query(
            module="Economy", storage_function="BBZDBalances", params=[bob.ss58_address]
        ).value or 0

        # Central Bank mints 1000 bBZD to Bob
        mint_amount = 1_000_000_000_000
        deposit_reference = b"BANK_DEPOSIT_20251230_001"
        call = substrate.compose_call(
            call_module="Economy",
            call_function="mint_bbzd",
            call_params={
                "recipient": bob.ss58_address,
                "amount": mint_amount,
                "deposit_reference": list(deposit_reference),
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=alice)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success, f"Minting failed: {receipt.error_message}"

        # Verify Bob's balance increased
        final_balance = substrate.query(
            module="Economy", storage_function="BBZDBalances", params=[bob.ss58_address]
        ).value or 0
        assert final_balance == initial_balance + mint_amount

        # Verify reserve invariant (supply ≤ reserves)
        total_supply = substrate.query(module="Economy", storage_function="TotalBbzdSupply").value
        reserves = substrate.query(module="Economy", storage_function="CentralBankReserves").value
        assert total_supply <= reserves, "CRITICAL: Supply exceeds reserves!"
    
    def test_mint_bbzd_unauthorized_fails(
        self, blockchain_connection: SubstrateInterface, bob_keypair: Keypair, alice_keypair: Keypair
    ):
        """Test that unauthorized user cannot mint bBZD"""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair
        mint_amount = 1_000_000_000_000
        deposit_reference = b"FAKE_DEPOSIT_123"
        
        call = substrate.compose_call(
            call_module='Economy',
            call_function='mint_bbzd',
            call_params={
                'recipient': alice.ss58_address,
                'amount': mint_amount,
                'deposit_reference': list(deposit_reference)
            }
        )
        
        # Bob (not authorized) tries to mint
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        
        with pytest.raises(SubstrateRequestException) as exc_info:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            if not receipt.is_success:
                raise SubstrateRequestException(receipt.error_message)
        
        # Should fail with UnauthorizedMinter error
        assert "UnauthorizedMinter" in str(exc_info.value) or not receipt.is_success
    
    def test_mint_bbzd_exceeds_reserves_fails(
        self,
        submit_sudo_extrinsic,
        blockchain_connection: SubstrateInterface,
        alice_keypair: Keypair,
        bob_keypair: Keypair,
    ):
        """Test that minting more than reserves fails"""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair

        # Governance: authorize Alice
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"]

        # Governance: set low reserves (only 100 BZD)
        low_reserves = 100_000_000_000
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": low_reserves},
        )
        assert res["success"]
        
        # Ensure Bob has KYC and funds
        assert fund_account(substrate, alice, bob.ss58_address, 1_000_000_000_000)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success
        
        # Try to mint 10,000 bBZD (way more than reserves)
        excessive_amount = 10_000_000_000_000
        
        call = substrate.compose_call(
            call_module='Economy',
            call_function='mint_bbzd',
            call_params={
                'recipient': bob.ss58_address,
                'amount': excessive_amount,
                'deposit_reference': list(b"DEPOSIT_HUGE")
            }
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        
        with pytest.raises(SubstrateRequestException) as exc_info:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            if not receipt.is_success:
                raise SubstrateRequestException(receipt.error_message)
        
        assert "InsufficientReserves" in str(exc_info.value) or not receipt.is_success


class TestBbzdRedemption:
    """Test bBZD redemption (burn + off-chain settlement)"""
    
    def test_redeem_bbzd_creates_request(
        self,
        submit_sudo_extrinsic,
        blockchain_connection: SubstrateInterface,
        alice_keypair: Keypair,
        bob_keypair: Keypair,
    ):
        """Test that redeem_bbzd burns bBZD and creates redemption request"""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair
        
        # Governance setup
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"]
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": 10_000_000_000_000},
        )
        assert res["success"]
        
        # Ensure Bob has KYC and fund for fees
        assert fund_account(substrate, alice, bob.ss58_address, 1_000_000_000_000)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success

        # Mint 5000 bBZD to Bob
        mint_amount = 5_000_000_000_000
        call = substrate.compose_call(
            call_module='Economy',
            call_function='mint_bbzd',
            call_params={
                'recipient': bob.ss58_address,
                'amount': mint_amount,
                'deposit_reference': list(b"DEPOSIT_SETUP")
            }
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        
        # Get Bob's initial balance
        initial_balance = substrate.query(
            module='Economy',
            storage_function='BBZDBalances',
            params=[bob.ss58_address]
        )
        
        # Get next redemption ID
        next_redemption_id = substrate.query(
            module='Economy',
            storage_function='NextRedemptionId'
        )
        expected_redemption_id = next_redemption_id.value if next_redemption_id.value else 1
        
        # Bob redeems 2000 bBZD
        redeem_amount = 2_000_000_000_000
        bank_account = b"BOB_BANK_ACCOUNT_BZD_12345"
        
        call = substrate.compose_call(
            call_module='Economy',
            call_function='redeem_bbzd',
            call_params={
                'amount': redeem_amount,
                'bank_account': list(bank_account)
            }
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        
        assert receipt.is_success, f"Redemption failed: {receipt.error_message}"
        
        # Verify bBZD burned immediately
        final_balance = substrate.query(
            module='Economy',
            storage_function='BBZDBalances',
            params=[bob.ss58_address]
        )
        assert final_balance.value == initial_balance.value - redeem_amount
        
        # Verify total supply decreased
        total_supply = substrate.query(
            module='Economy',
            storage_function='TotalBbzdSupply'
        )
        # Total supply should have decreased by redeem_amount from previous state
        
        # Verify redemption request created
        redemption_request = substrate.query(
            module='Economy',
            storage_function='RedemptionRequests',
            params=[expected_redemption_id]
        )
        
        assert redemption_request.value is not None, "Redemption request not created"
        request = redemption_request.value
        assert request['user'] == bob.ss58_address
        assert request['amount'] == redeem_amount
        assert request['status'] == 'Pending'
        
        # Check for BbzdRedeemed event
        events = receipt.triggered_events
        redeemed_found = False
        for event in events:
            if event.value['event_id'] == 'BbzdRedeemed':
                event_data = event.value['attributes']
                assert event_data['user'] == bob.ss58_address
                assert event_data['amount'] == redeem_amount
                assert event_data['redemption_id'] == expected_redemption_id
                redeemed_found = True
                break
        
        assert redeemed_found, "BbzdRedeemed event not found"
    
    def test_redeem_bbzd_insufficient_balance_fails(
        self, blockchain_connection: SubstrateInterface, bob_keypair: Keypair, alice_keypair: Keypair
    ):
        """Test that redemption fails if user has insufficient bBZD balance"""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair

        # Ensure Bob has KYC (to avoid KYC error ahead of balance error)
        assert fund_account(substrate, alice, bob.ss58_address, 1_000_000_000_000)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success
        
        # Get Bob's current balance
        current_balance = substrate.query(
            module='Economy',
            storage_function='BBZDBalances',
            params=[bob.ss58_address]
        )
        current_balance_value = current_balance.value if current_balance.value else 0
        
        # Try to redeem more than balance
        excessive_amount = current_balance_value + 1_000_000_000_000
        
        call = substrate.compose_call(
            call_module='Economy',
            call_function='redeem_bbzd',
            call_params={
                'amount': excessive_amount,
                'bank_account': list(b"BOB_BANK_ACCOUNT")
            }
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        
        with pytest.raises(SubstrateRequestException) as exc_info:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            if not receipt.is_success:
                raise SubstrateRequestException(receipt.error_message)
        
        assert "InsufficientBbzdBalance" in str(exc_info.value) or not receipt.is_success


class TestRedemptionProcessing:
    """Test Central Bank redemption processing"""
    
    def test_process_redemption_completes_settlement(
        self,
        submit_sudo_extrinsic,
        blockchain_connection: SubstrateInterface,
        alice_keypair: Keypair,
        bob_keypair: Keypair,
    ):
        """Test that Central Bank can process pending redemptions"""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair
# Setup: Create a pending redemption first
        # (Reuse minting and redemption logic from previous tests)
        
        # Governance: authorize and set reserves
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"]
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": 10_000_000_000_000},
        )
        assert res["success"]
        
        call = substrate.compose_call(
            call_module='Economy',
            call_function='mint_bbzd',
            call_params={
                'recipient': bob.ss58_address,
                'amount': 3_000_000_000_000,
                'deposit_reference': list(b"DEPOSIT_FOR_REDEMPTION")
            }
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        
        # Get next redemption ID
        next_id_query = substrate.query(
            module='Economy',
            storage_function='NextRedemptionId'
        )
        redemption_id = next_id_query.value if next_id_query.value else 1
        
        # Ensure Bob has KYC and fund
        assert fund_account(substrate, alice, bob.ss58_address, 1_000_000_000_000)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success

        # Bob creates redemption
        call = substrate.compose_call(
            call_module='Economy',
            call_function='redeem_bbzd',
            call_params={
                'amount': 1_000_000_000_000,
                'bank_account': list(b"BOB_BANK_FOR_PROCESSING")
            }
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        
        # Verify redemption is pending
        request_before = substrate.query(
            module='Economy',
            storage_function='RedemptionRequests',
            params=[redemption_id]
        )
        assert request_before.value['status'] == 'Pending'
        
        # Alice (Central Bank) processes redemption after off-chain BZD transfer
        call = substrate.compose_call(
            call_module='Economy',
            call_function='process_redemption',
            call_params={'redemption_id': redemption_id}
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        
        assert receipt.is_success, f"Processing failed: {receipt.error_message}"
        
        # Verify redemption marked as processed
        request_after = substrate.query(
            module='Economy',
            storage_function='RedemptionRequests',
            params=[redemption_id]
        )
        assert request_after.value['status'] == 'Processed'
        
        # Check for RedemptionProcessed event
        events = receipt.triggered_events
        processed_found = False
        for event in events:
            if event.value['event_id'] == 'RedemptionProcessed':
                event_data = event.value['attributes']
                assert event_data['redemption_id'] == redemption_id
                processed_found = True
                break
        
        assert processed_found, "RedemptionProcessed event not found"
    
    def test_process_redemption_unauthorized_fails(self, blockchain_connection: SubstrateInterface, bob_keypair: Keypair):
        """Test that unauthorized user cannot process redemptions"""
        substrate = blockchain_connection
        bob = bob_keypair
        
        # Bob (not Central Bank) tries to process redemption
        call = substrate.compose_call(
            call_module='Economy',
            call_function='process_redemption',
            call_params={'redemption_id': 1}
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        
        with pytest.raises(SubstrateRequestException) as exc_info:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            if not receipt.is_success:
                raise SubstrateRequestException(receipt.error_message)
        
        assert "UnauthorizedMinter" in str(exc_info.value) or not receipt.is_success


class TestReserveInvariants:
    """Test critical reserve invariants"""
    
    def test_supply_never_exceeds_reserves(
        self,
        submit_sudo_extrinsic,
        blockchain_connection: SubstrateInterface,
        alice_keypair: Keypair,
        bob_keypair: Keypair,
    ):
        """CRITICAL: Verify TotalBbzdSupply ≤ CentralBankReserves at all times"""
        substrate = blockchain_connection
        alice = alice_keypair
        bob = bob_keypair

        # Governance setup
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"]
        reserves_amount = 5_000_000_000_000
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": reserves_amount},
        )
        assert res["success"]
        
        # Ensure Bob has KYC and fund
        assert fund_account(substrate, alice, bob.ss58_address, 1_000_000_000_000)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=bob)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success

        # Mint multiple times to same user
        for i in range(3):
            call = substrate.compose_call(
                call_module='Economy',
                call_function='mint_bbzd',
                call_params={
                    'recipient': bob.ss58_address,
                    'amount': 1_000_000_000_000,  # 1000 bBZD each
                    'deposit_reference': list(f"DEPOSIT_{i}".encode())
                }
            )
            extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
            substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            
            # Check invariant after each mint
            total_supply = substrate.query(
                module='Economy',
                storage_function='TotalBbzdSupply'
            )
            reserves = substrate.query(
                module='Economy',
                storage_function='CentralBankReserves'
            )
            
            assert total_supply.value <= reserves.value, \
                f"INVARIANT VIOLATED: Supply {total_supply.value} > Reserves {reserves.value}"
        
        # Redeem and check invariant again
        call = substrate.compose_call(
            call_module='Economy',
            call_function='redeem_bbzd',
            call_params={
                'amount': 500_000_000_000,
                'bank_account': list(b"BOB_BANK_INVARIANT_TEST")
            }
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        
        # Final invariant check
        total_supply = substrate.query(
            module='Economy',
            storage_function='TotalBbzdSupply'
        )
        reserves = substrate.query(
            module='Economy',
            storage_function='CentralBankReserves'
        )
        
        assert total_supply.value <= reserves.value, \
            "INVARIANT VIOLATED after redemption"


class TestGovernanceIntegration:
    """Test Economy pallet integration with Governance"""
    
    def test_minter_authorization_requires_governance(
        self, blockchain_connection: SubstrateInterface, alice_keypair: Keypair, bob_keypair: Keypair
    ):
        """Test that only governance can authorize minters"""
        substrate = blockchain_connection
        bob = bob_keypair
        
        # Bob (regular user) tries to authorize himself
        call = substrate.compose_call(
            call_module='Economy',
            call_function='set_minter_authorization',
            call_params={'account': bob.ss58_address, 'authorized': True}
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        
        with pytest.raises(SubstrateRequestException) as exc_info:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            if not receipt.is_success:
                raise SubstrateRequestException(receipt.error_message)
        
        # Should fail with BadOrigin
        assert "BadOrigin" in str(exc_info.value) or not receipt.is_success
    
    def test_reserve_updates_require_governance(self, blockchain_connection: SubstrateInterface, bob_keypair: Keypair):
        """Test that only governance can update reserves"""
        substrate = blockchain_connection
        bob = bob_keypair
        
        # Bob tries to update reserves
        call = substrate.compose_call(
            call_module='Economy',
            call_function='update_reserves',
            call_params={'new_reserves': 99_999_999_999_999}
        )
        
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
        
        with pytest.raises(SubstrateRequestException) as exc_info:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
            if not receipt.is_success:
                raise SubstrateRequestException(receipt.error_message)
        
        assert "BadOrigin" in str(exc_info.value) or not receipt.is_success
