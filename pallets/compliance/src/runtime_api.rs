//! # Compliance Runtime API (AR-11)
//!
//! Declares the off-chain-accessible runtime APIs that expose compliance audit
//! history and suspicious-activity reports for a given on-chain account.
//!
//! ## Usage
//! 1. The runtime implements [`ComplianceApi`] inside `impl_runtime_apis!`.
//! 2. Light clients / RPC callers invoke `compliance_api_getAccountComplianceHistory`
//!    and `compliance_api_getSuspiciousActivityReports` via the standard substrate
//!    RPC state-call interface.
//!
//! ## Type stability
//! [`ComplianceAuditEntry`] and [`SuspiciousActivityEntry`] are flat, concrete structs
//! (no generic params) so they cross the host-runtime boundary cleanly.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

// ── Flat response types ───────────────────────────────────────────────────────

/// Serialisable representation of a single compliance audit event.
///
/// Mirrors [`crate::ComplianceAuditRecord`] but uses only primitive types
/// so that it can be used across the host-runtime API boundary.
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ComplianceAuditEntry {
    /// SCALE-encoded `ActionType` discriminant (u8).
    pub action_type: u8,
    /// Unix timestamp recorded at the time of the action.
    pub timestamp: u64,
    /// Chain block number at which the audit record was written.
    pub block_number: u32,
    /// Whether the compliance action succeeded.
    pub success: bool,
    /// UTF-8 detail string (may be empty).
    pub details: Vec<u8>,
}

/// Serialisable representation of a single suspicious-activity report.
///
/// Mirrors [`crate::SuspiciousActivityReport`] but uses only primitive types.
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SuspiciousActivityEntry {
    /// SCALE-encoded `SuspiciousActivityType` discriminant (u8).
    pub activity_type: u8,
    /// Chain block number at which this report was submitted.
    pub block_number: u32,
    /// Human-readable description supplied by the reporter.
    pub description: Vec<u8>,
}

// ── API declaration ───────────────────────────────────────────────────────────

sp_api::decl_runtime_apis! {
    /// On-chain compliance data queries for the BelizeChain compliance pallet.
    ///
    /// Implemented by the runtime inside `impl_runtime_apis!`.
    pub trait ComplianceApi {
        /// Return all audit records for `account`, oldest first.
        ///
        /// An empty `Vec` is returned when the account has no audit history.
        fn get_account_compliance_history(
            account: sp_runtime::AccountId32,
        ) -> Vec<ComplianceAuditEntry>;

        /// Return all suspicious activity reports filed for `account`, oldest first.
        ///
        /// An empty `Vec` is returned when no SARs exist for the account.
        fn get_suspicious_activity_reports(
            account: sp_runtime::AccountId32,
        ) -> Vec<SuspiciousActivityEntry>;
    }
}
