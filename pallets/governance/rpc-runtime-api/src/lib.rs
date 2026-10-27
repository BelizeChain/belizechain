//! Runtime API surface for `pallet-belize-governance`.
//!
//! Exists so the node can answer governance queries without the client having to
//! decode storage keys. Every function here is a read-only projection of pallet
//! storage; none of them can mutate state, so they are safe to serve over RPC.
//!
//! The types are the pallet's own (`CouncilMember`, `Election`, `BelizeDistrict`)
//! rather than crate-local mirrors. A mirror would be a second definition of a
//! fact the chain already owns, and it would drift silently the first time the
//! pallet changed.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
pub use pallet_belize_governance::{BelizeDistrict, CouncilMember, Election};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// Read-only access to governance state.
    pub trait GovernanceApi<AccountId, BlockNumber>
    where
        AccountId: Codec,
        BlockNumber: Codec,
    {
        /// Every seated council member, in unspecified order.
        fn council_members() -> Vec<CouncilMember<AccountId, BlockNumber>>;

        /// The current council term number.
        fn council_term() -> u32;

        /// Every district's election record — the running one, or the most
        /// recent finalized/cancelled one. Districts that have never held an
        /// election are absent.
        fn district_elections() -> Vec<(BelizeDistrict, Election<BlockNumber>)>;

        /// The id the next submitted proposal will receive. Also the total
        /// number of proposals ever submitted, since ids are never reused.
        fn next_proposal_id() -> u32;
    }
}
