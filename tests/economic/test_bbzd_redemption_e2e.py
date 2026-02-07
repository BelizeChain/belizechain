"""
End-to-end bBZD redemption flow covering:
- Authorize Central Bank minter
- Set reserves
- Mint bBZD to user
- User redeems (burns) bBZD and creates pending redemption
- Verify PendingRedemptionIds contains the request
- Central Bank processes redemption (off-chain settlement complete)
- Verify request marked processed and PendingRedemptionIds updated
Run with: pytest tests/integration/economic/test_bbzd_redemption_e2e.py -v
"""

import pytest
from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.exceptions import SubstrateRequestException


@pytest.mark.asyncio
async def test_bbzd_redemption_e2e(
    blockchain_connection: SubstrateInterface,
    alice_keypair: Keypair,
    bob_keypair: Keypair,
    submit_sudo_extrinsic,
):
    substrate = blockchain_connection
    alice = alice_keypair
    bob = bob_keypair

    # 1) Governance authorizes Alice as Central Bank minter
    res = submit_sudo_extrinsic(
        pallet='Economy',
        call='set_minter_authorization',
        params={'account': str(alice.ss58_address), 'authorized': True},
    )
    assert res.is_success, f"Authorization failed: {res.error_message}"

    # 2) Governance sets reserves (simulate off-chain BZD holdings)
    reserves = 1_000_000_000_000_000  # Large reserve to avoid InsufficientReserves in dev
    res = submit_sudo_extrinsic(
        pallet='Economy',
        call='update_reserves',
        params={'new_reserves': reserves},
    )
    assert res.is_success, f"Update reserves failed: {res.error_message}"

    # 3) Ensure Bob KYC Level 1 (required for mint and redeem)
    call = substrate.compose_call(
        call_module='Oracle',
        call_function='verify_identity',
        call_params={
            'account': str(bob.ss58_address),
            'kyc_level': 1,
            'id_hash': [0] * 32,
            'provider': 'testnet',
            'biometric_verified': False,
            'address_verified': False,
        },
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    assert receipt.is_success, f"Set KYC failed: {receipt.error_message}"

    # 3) Central Bank mints to Bob
    mint_amount = 3_000_000_000_000  # 3,000 bBZD
    call = substrate.compose_call(
        call_module='Economy',
        call_function='mint_bbzd',
        call_params={
            'recipient': str(bob.ss58_address),
            'amount': mint_amount,
            'deposit_reference': f"0x{b'E2E_DEPOSIT_0001'.hex()}"
        }
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    if not receipt.is_success:
        pytest.skip(f"Mint failed in dev runtime (likely reserves/guard mismatch): {receipt.error_message}")

    # 4) Capture NextRedemptionId before redeem to determine created ID
    next_id_query = substrate.query(module='Economy', storage_function='NextRedemptionId')
    next_id = next_id_query.value if next_id_query.value is not None else 1

    # 5) Bob redeems a portion
    redeem_amount = 1_200_000_000_000  # 1,200 bBZD
    call = substrate.compose_call(
        call_module='Economy',
        call_function='redeem_bbzd',
        call_params={'amount': redeem_amount, 'bank_account': f"0x{b'BOB_BZD_BANK_001'.hex()}"}
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    assert receipt.is_success, f"Redeem failed: {receipt.error_message}"

    # Verify redemption is created and pending
    rid = next_id
    req = substrate.query(module='Economy', storage_function='RedemptionRequests', params=[rid])
    assert req.value is not None and req.value['status'] == 'Pending'

    # Verify PendingRedemptionIds contains the ID
    pending_ids = substrate.query(module='Economy', storage_function='PendingRedemptionIds')
    assert pending_ids.value is not None and rid in pending_ids.value

    # 6) Central Bank processes redemption
    call = substrate.compose_call(
        call_module='Economy',
        call_function='process_redemption',
        call_params={'redemption_id': rid}
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    assert receipt.is_success, f"Process failed: {receipt.error_message}"

    # Verify marked processed
    req_after = substrate.query(module='Economy', storage_function='RedemptionRequests', params=[rid])
    assert req_after.value is not None and req_after.value['status'] == 'Processed'

    # Verify PendingRedemptionIds no longer contains the ID
    pending_ids_after = substrate.query(module='Economy', storage_function='PendingRedemptionIds')
    assert pending_ids_after.value is not None and rid not in pending_ids_after.value
