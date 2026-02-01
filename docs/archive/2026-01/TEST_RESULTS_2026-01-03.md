# Integration Test Results — 2026-01-03

- Scope: Economic compliance tests for bBZD (fiat-backed) flows
- Branch: feature/bbzd-peg-bzd
- Node: Dev chain (WASM, local runner)

Results:
- Passed: tests/integration/economic/test_bbzd_compliance.py::TestBbzdCompliance::test_mint_requires_kyc
- Passed: tests/integration/economic/test_bbzd_compliance.py::TestBbzdCompliance::test_redeem_blocks_for_sanctioned

Notes:
- Used `Balances.transfer_keep_alive` to fund the KYC signer prior to `Oracle.verify_identity()`.
- Corrected `Economy.set_minter_authorization` parameter to `account`.
- Ensured clean KYC/sanction state across tests to prevent contamination.
