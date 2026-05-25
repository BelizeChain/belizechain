# Legacy docs

Quarantined documents that no longer reflect the current state of the
BelizeChain runtime, Ceiba testnet, or sibling repositories. They are kept
here for historical reference only.

Do **not** use these as a source of truth for:

- Runtime pallet counts, indexes, or names (see `runtime/src/lib.rs` and the
  current `belizechain.org` Appendix B for the live list of 18 custom pallets
  at indexes 20\u201337).
- Polkadot SDK / Substrate version (see `Cargo.toml` workspace dependencies
  for the current `stable2603` rev).
- Service ports or endpoints (see `infra/nginx/nginx.conf` and the
  `belizechain.org` Appendix A / D for the current local + Ceiba testnet
  layout).
- ink! contracts list, sizes, or versions (see the `gem/` repo and
  `belizechain.org` Appendix C for the current 8-contract list on ink! 5.1.1).

Authoritative documentation lives under `docs/architecture/`,
`docs/operations/`, `docs/deployment/`, and the per-repo READMEs.

## Files

- `ARCHITECTURAL_RECONSTRUCTION_REPORT.md` \u2014 historical reconstruction snapshot
  from an earlier phase of the project. Pre-dates the current 18-pallet
  runtime, `stable2603` SDK pin, and the live Ceiba testnet
  (`testnet.belizechain.org`). Retained for narrative context only.
