//! # BelizeChain Common Module
//!
//! This module provides common types, traits, and utilities shared across
//! multiple BelizeChain pallets.
//!
//! ## Modules
//!
//! - `temporal_anchor`: Temporal anchoring system for immutable audit trails

#![cfg_attr(not(feature = "std"), no_std)]

pub mod temporal_anchor;

// Re-export commonly used types
pub use temporal_anchor::{
    helpers as temporal_helpers, AnchorType, TemporalAnchor, TemporalAnchoring,
};
