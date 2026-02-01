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
):
    substrate = blockchain_connection
    alice = alice_keypair
    bob = bob_keypair

    # 1) Governance authorizes Alice as Central Bank minter
    call = substrate.compose_call(
        call_module='Economy',
        call_function='set_minter_authorization',
        call_params={'minter': alice.ss58_address, 'authorized': True}
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    assert receipt.is_success, f"Authorization failed: {receipt.error_message}"

    # 2) Governance sets reserves (simulate off-chain BZD holdings)
    reserves = 10_000_000_000_000  # 10,000 BZD (12 decimals)
    call = substrate.compose_call(
        call_module='Economy',
        call_function='update_reserves',
        call_params={'new_reserves': reserves}
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    assert receipt.is_success

    # 3) Central Bank mints to Bob
    mint_amount = 3_000_000_000_000  # 3,000 bBZD
    call = substrate.compose_call(
        call_module='Economy',
        call_function='mint_bbzd',
        call_params={
            'recipient': bob.ss58_address,
            'amount': mint_amount,
            'deposit_reference': list(b'E2E_DEPOSIT_0001')
        }
    )
    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    assert receipt.is_success, f"Mint failed: {receipt.error_message}"

    # 4) Capture NextRedemptionId before redeem to determine created ID
    next_id_query = substrate.query(module='Economy', storage_function='NextRedemptionId')
    next_id = next_id_query.value if next_id_query.value is not None else 1

    # 5) Bob redeems a portion
    redeem_amount = 1_200_000_000_000  # 1,200 bBZD
    call = substrate.compose_call(
        call_module='Economy',
        call_function='redeem_bbzd',
        call_params={'amount': redeem_amount, 'bank_account': list(b'BOB_BZD_BANK_001')}
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
