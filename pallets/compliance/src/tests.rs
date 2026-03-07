use crate::{mock::*, Error, Event, RiskLevel, VerificationLevel, ComplianceStatus, SuspiciousActivityType};
use frame_support::{assert_noop, assert_ok};

#[test]
fn verify_account_works() {
    new_test_ext().execute_with(|| {
        // Verify account with Standard level
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        // Check storage
        let status = Compliance::compliance_status(1);
        assert_eq!(status.verification_level, VerificationLevel::Standard);
        assert_eq!(status.risk_level, RiskLevel::Low);

        // Check event (level is encoded as u8)
        System::assert_last_event(Event::VerificationLevelUpdated {
            account: 1,
            level: 2, // Standard = 2
        }.into());
    });
}

#[test]
fn update_risk_level_works() {
    new_test_ext().execute_with(|| {
        // First verify account
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        // Update risk level
        assert_ok!(Compliance::update_risk_level(
            RuntimeOrigin::root(),
            1,
            2 // High = 2
        ));

        // Check storage
        let status = Compliance::compliance_status(1);
        assert_eq!(status.risk_level, RiskLevel::High);
    });
}

#[test]
fn whitelist_account_works() {
    new_test_ext().execute_with(|| {
        // Whitelist account
        assert_ok!(Compliance::whitelist_account(RuntimeOrigin::root(), 1));

        // Check storage
        let status = Compliance::compliance_status(1);
        assert!(status.whitelisted);
        assert!(Compliance::is_whitelisted(1));

        // Check event
        System::assert_last_event(Event::AccountWhitelisted { account: 1 }.into());
    });
}

#[test]
fn restrict_account_works() {
    new_test_ext().execute_with(|| {
        // Restrict account
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(),
            1,
            b"Suspicious activity detected".to_vec()
        ));

        // Check storage
        let status = Compliance::compliance_status(1);
        assert!(status.restricted);

        let (restricted, _reason) = Compliance::is_restricted(1);
        assert!(restricted);
    });
}

#[test]
fn lift_restriction_works() {
    new_test_ext().execute_with(|| {
        // Restrict then lift
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(),
            1,
            b"Test restriction".to_vec()
        ));

        assert_ok!(Compliance::lift_restriction(RuntimeOrigin::root(), 1));

        // Check storage
        let status = Compliance::compliance_status(1);
        assert!(!status.restricted);
    });
}

#[test]
fn flag_suspicious_activity_works() {
    new_test_ext().execute_with(|| {
        // Flag suspicious activity
        assert_ok!(Compliance::flag_suspicious_activity(
            RuntimeOrigin::root(),
            1,
            0, // UnusualVolumePattern = 0
            b"High frequency trading detected".to_vec()
        ));

        // Check that risk level was auto-elevated
        let status = Compliance::compliance_status(1);
        assert_eq!(status.risk_level, RiskLevel::High);

        // Check suspicious activities storage
        let activities = Compliance::suspicious_activities(1);
        assert_eq!(activities.len(), 1);
    });
}

#[test]
fn sanctions_list_works() {
    new_test_ext().execute_with(|| {
        let entity_hash = [1u8; 32];

        // Add to sanctions list
        assert_ok!(Compliance::add_sanctions_entry(
            RuntimeOrigin::root(),
            entity_hash,
            b"OFAC".to_vec()
        ));

        // Check it exists
        assert!(Compliance::is_sanctioned(&entity_hash));

        // Try to add duplicate
        assert_noop!(
            Compliance::add_sanctions_entry(
                RuntimeOrigin::root(),
                entity_hash,
                b"UN".to_vec()
            ),
            Error::<Test>::SanctionsEntryExists
        );

        // Remove from list
        assert_ok!(Compliance::remove_sanctions_entry(
            RuntimeOrigin::root(),
            entity_hash
        ));

        // Check it's gone
        assert!(!Compliance::is_sanctioned(&entity_hash));
    });
}

#[test]
fn can_be_validator_checks_work() {
    new_test_ext().execute_with(|| {
        // Unverified account cannot be validator
        assert!(!Compliance::can_be_validator(&1));

        // Verify with Enhanced level (required for validators)
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            3, // Enhanced = 3
            0  // Low = 0
        ));

        // Now can be validator
        assert!(Compliance::can_be_validator(&1));

        // Restrict account
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(),
            1,
            b"Test".to_vec()
        ));

        // Cannot be validator when restricted
        assert!(!Compliance::can_be_validator(&1));
    });
}

#[test]
fn can_participate_in_governance_works() {
    new_test_ext().execute_with(|| {
        // Unverified cannot participate
        assert!(!Compliance::can_participate_in_governance(&1));

        // Verify with Standard level (minimum for governance)
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        // Can participate
        assert!(Compliance::can_participate_in_governance(&1));

        // High risk prevents participation
        assert_ok!(Compliance::update_risk_level(
            RuntimeOrigin::root(),
            1,
            3  // Prohibited = 3
        ));

        assert!(!Compliance::can_participate_in_governance(&1));
    });
}

#[test]
fn can_access_treasury_works() {
    new_test_ext().execute_with(|| {
        // Unverified cannot access treasury
        assert!(!Compliance::can_access_treasury(&1));

        // Verify with Enhanced level (required for treasury)
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            3, // Enhanced = 3
            0  // Low = 0
        ));

        // Can access treasury
        assert!(Compliance::can_access_treasury(&1));

        // Whitelisted accounts can access even without verification
        assert_ok!(Compliance::whitelist_account(RuntimeOrigin::root(), 2));
        assert!(Compliance::can_access_treasury(&2));
    });
}

#[test]
fn travel_rule_threshold_works() {
    new_test_ext().execute_with(|| {
        // Below threshold
        assert!(!Compliance::requires_travel_rule(1_000_000_000)); // 1 DALLA

        // Above threshold
        assert!(Compliance::requires_travel_rule(15_000_000_000)); // 15K DALLA
    });
}

#[test]
fn audit_records_created() {
    new_test_ext().execute_with(|| {
        // Perform actions that create audit records
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        assert_ok!(Compliance::whitelist_account(RuntimeOrigin::root(), 1));

        // Check audit records exist
        let records = Compliance::audit_records(1);
        assert!(records.len() >= 2);
    });
}

#[test]
fn compliance_stats_updated() {
    new_test_ext().execute_with(|| {
        // Initial stats
        let (verified, restricted, suspicious, _sanctions) = Compliance::get_stats();
        assert_eq!(verified, 0);
        assert_eq!(restricted, 0);
        assert_eq!(suspicious, 0);

        // Verify account
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        let (verified, _, _, _) = Compliance::get_stats();
        assert_eq!(verified, 1);

        // Restrict account
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(),
            1,
            b"Test".to_vec()
        ));

        let (_, restricted, _, _) = Compliance::get_stats();
        assert_eq!(restricted, 1);

        // Flag suspicious activity
        assert_ok!(Compliance::flag_suspicious_activity(
            RuntimeOrigin::root(),
            2,
            4, // MoneyLaundering = 4
            b"Test".to_vec()
        ));

        let (_, _, suspicious, _) = Compliance::get_stats();
        assert_eq!(suspicious, 1);
    });
}

#[test]
fn sync_verification_from_identity_still_valid_works() {
    new_test_ext().execute_with(|| {
        // Set up: verify account (sets last_verification = now = 1000 secs)
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        let status_before = Compliance::compliance_status(1);
        assert_eq!(status_before.verification_level, VerificationLevel::Standard);

        // Sync immediately while still within validity period
        assert_ok!(Compliance::sync_verification_from_identity(
            RuntimeOrigin::signed(1),
        ));

        // Level should remain Standard (not expired)
        let status_after = Compliance::compliance_status(1);
        assert_eq!(status_after.verification_level, VerificationLevel::Standard);
        assert!(!status_after.restricted);
    });
}

#[test]
fn sync_verification_from_identity_expired_downgrades() {
    new_test_ext().execute_with(|| {
        // Verify account at initial timestamp (1_000_000 ms = 1_000 secs)
        assert_ok!(Compliance::verify_account(
            RuntimeOrigin::root(),
            1,
            2, // Standard = 2
            0  // Low = 0
        ));

        let status_before = Compliance::compliance_status(1);
        assert_eq!(status_before.verification_level, VerificationLevel::Standard);
        assert_eq!(status_before.last_verification, 1_000); // 1_000_000 ms / 1000

        // Advance time past the validity period (1 year = 31_536_000 secs)
        // Set timestamp to 40 billion ms = 40_000_000 secs
        // Age = 40_000_000 - 1_000 = 39_999_000 > 31_536_000 → expired
        Timestamp::set_timestamp(40_000_000_000u64);

        // Sync should detect expiry, downgrade level to None and restrict
        assert_ok!(Compliance::sync_verification_from_identity(
            RuntimeOrigin::signed(1),
        ));

        let status_after = Compliance::compliance_status(1);
        assert_eq!(status_after.verification_level, VerificationLevel::None);
        assert!(status_after.restricted);

        System::assert_last_event(Event::VerificationLevelUpdated {
            account: 1,
            level: 0, // None = 0
        }.into());
    });
}

// ================================================================
// Origin / access control error-path tests
// ================================================================

#[test]
fn verify_account_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::verify_account(RuntimeOrigin::signed(1), 2, 2, 0),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn update_risk_level_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::update_risk_level(RuntimeOrigin::signed(1), 2, 2),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn whitelist_account_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::whitelist_account(RuntimeOrigin::signed(1), 2),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn restrict_account_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::restrict_account(RuntimeOrigin::signed(1), 2, b"test".to_vec()),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn lift_restriction_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::lift_restriction(RuntimeOrigin::signed(1), 2),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn flag_suspicious_activity_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::flag_suspicious_activity(
                RuntimeOrigin::signed(1), 2, 0, b"test".to_vec()
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_sanctions_entry_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::add_sanctions_entry(
                RuntimeOrigin::signed(1), [0u8; 32], b"OFAC".to_vec()
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn remove_sanctions_entry_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::remove_sanctions_entry(RuntimeOrigin::signed(1), [0u8; 32]),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn remove_sanctions_entry_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Compliance::remove_sanctions_entry(RuntimeOrigin::root(), [99u8; 32]),
            Error::<Test>::SanctionsEntryNotFound
        );
    });
}

#[test]
fn restrict_account_reason_too_long_fails() {
    new_test_ext().execute_with(|| {
        // Reason longer than 256 bytes should fail.
        let long_reason = vec![b'x'; 300];
        assert_noop!(
            Compliance::restrict_account(RuntimeOrigin::root(), 1, long_reason),
            Error::<Test>::InvalidRiskAssessment
        );
    });
}

#[test]
fn flag_suspicious_activity_details_too_long_fails() {
    new_test_ext().execute_with(|| {
        let long_details = vec![b'y'; 300];
        assert_noop!(
            Compliance::flag_suspicious_activity(
                RuntimeOrigin::root(), 1, 0, long_details
            ),
            Error::<Test>::InvalidRiskAssessment
        );
    });
}

// ================================================================
// Event emission tests
// ================================================================

#[test]
fn update_risk_level_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 2, 0));
        assert_ok!(Compliance::update_risk_level(RuntimeOrigin::root(), 1, 1));
        System::assert_last_event(Event::RiskLevelUpdated {
            account: 1,
            risk_level: 1, // Medium
        }.into());
        // Also verify AuditRecordCreated was emitted.
        System::assert_has_event(Event::AuditRecordCreated {
            account: 1,
            action_type: 1, // RiskAssessmentUpdated
        }.into());
    });
}

#[test]
fn restrict_account_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(), 1, b"AML concern".to_vec()
        ));
        let bounded: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<256>> =
            b"AML concern".to_vec().try_into().unwrap();
        System::assert_has_event(Event::AccountRestricted {
            account: 1,
            reason: bounded,
        }.into());
    });
}

#[test]
fn lift_restriction_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(), 1, b"test".to_vec()
        ));
        assert_ok!(Compliance::lift_restriction(RuntimeOrigin::root(), 1));
        System::assert_has_event(Event::RestrictionLifted { account: 1 }.into());
    });
}

#[test]
fn flag_suspicious_activity_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::flag_suspicious_activity(
            RuntimeOrigin::root(), 1, 4, b"ML indicators".to_vec()
        ));
        System::assert_has_event(Event::SuspiciousActivityReported {
            account: 1,
            activity_type: 4,
        }.into());
    });
}

#[test]
fn add_sanctions_entry_emits_event() {
    new_test_ext().execute_with(|| {
        let hash = [42u8; 32];
        assert_ok!(Compliance::add_sanctions_entry(
            RuntimeOrigin::root(), hash, b"UN".to_vec()
        ));
        System::assert_has_event(Event::SanctionsEntryAdded { entity_hash: hash }.into());
    });
}

#[test]
fn remove_sanctions_entry_emits_event() {
    new_test_ext().execute_with(|| {
        let hash = [42u8; 32];
        assert_ok!(Compliance::add_sanctions_entry(
            RuntimeOrigin::root(), hash, b"UN".to_vec()
        ));
        assert_ok!(Compliance::remove_sanctions_entry(RuntimeOrigin::root(), hash));
        System::assert_last_event(Event::SanctionsEntryRemoved { entity_hash: hash }.into());
    });
}

// ================================================================
// Helper / public API tests
// ================================================================

#[test]
fn meets_verification_level_checks() {
    new_test_ext().execute_with(|| {
        // Unverified account does not meet Basic.
        assert!(!Compliance::meets_verification_level(&1, VerificationLevel::Basic));

        // Verify at Standard.
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 2, 0));
        assert!(Compliance::meets_verification_level(&1, VerificationLevel::Basic));
        assert!(Compliance::meets_verification_level(&1, VerificationLevel::Standard));
        assert!(!Compliance::meets_verification_level(&1, VerificationLevel::Enhanced));
    });
}

#[test]
fn is_verification_valid_within_period() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 2, 0));
        assert!(Compliance::is_verification_valid(&1));
    });
}

#[test]
fn is_verification_valid_expired() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 2, 0));
        // Advance past validity period (31_536_000 seconds).
        Timestamp::set_timestamp(40_000_000_000u64);
        assert!(!Compliance::is_verification_valid(&1));
    });
}

#[test]
fn can_be_validator_prohibited_risk_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 3, 0));
        assert!(Compliance::can_be_validator(&1));
        assert_ok!(Compliance::update_risk_level(RuntimeOrigin::root(), 1, 3)); // Prohibited
        assert!(!Compliance::can_be_validator(&1));
    });
}

#[test]
fn can_participate_governance_basic_level_insufficient() {
    new_test_ext().execute_with(|| {
        // Basic (1) is below Standard (2) minimum for governance.
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 1, 0));
        assert!(!Compliance::can_participate_in_governance(&1));
    });
}

#[test]
fn can_access_treasury_restricted_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 3, 0));
        assert!(Compliance::can_access_treasury(&1));
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(), 1, b"test".to_vec()
        ));
        assert!(!Compliance::can_access_treasury(&1));
    });
}

#[test]
fn verification_level_from_u8_roundtrip() {
    for v in 0..=4u8 {
        let level = VerificationLevel::from_u8(v);
        assert_eq!(level.as_u8(), v);
    }
    // Unknown maps to None (0).
    assert_eq!(VerificationLevel::from_u8(255), VerificationLevel::None);
}

#[test]
fn compliance_status_default() {
    let status = ComplianceStatus::default();
    assert_eq!(status.verification_level, VerificationLevel::None);
    assert_eq!(status.risk_level, RiskLevel::Low);
    assert!(!status.whitelisted);
    assert!(!status.restricted);
    assert_eq!(status.last_verification, 0);
}

#[test]
fn re_verify_does_not_double_count_stats() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 2, 0));
        let (v1, _, _, _) = Compliance::get_stats();
        assert_eq!(v1, 1);

        // Re-verify the same account at a higher level.
        assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 3, 0));
        let (v2, _, _, _) = Compliance::get_stats();
        assert_eq!(v2, 1); // Should not increment again.
    });
}

#[test]
fn lift_restriction_decrements_stats() {
    new_test_ext().execute_with(|| {
        assert_ok!(Compliance::restrict_account(
            RuntimeOrigin::root(), 1, b"test".to_vec()
        ));
        let (_, r1, _, _) = Compliance::get_stats();
        assert_eq!(r1, 1);

        assert_ok!(Compliance::lift_restriction(RuntimeOrigin::root(), 1));
        let (_, r2, _, _) = Compliance::get_stats();
        assert_eq!(r2, 0);
    });
}

#[test]
fn verify_all_risk_levels() {
    new_test_ext().execute_with(|| {
        for risk in 0..=3u8 {
            assert_ok!(Compliance::verify_account(RuntimeOrigin::root(), 1, 2, risk));
            let status = Compliance::compliance_status(1);
            let expected = match risk {
                1 => RiskLevel::Medium,
                2 => RiskLevel::High,
                3 => RiskLevel::Prohibited,
                _ => RiskLevel::Low,
            };
            assert_eq!(status.risk_level, expected);
        }
    });
}

#[test]
fn suspicious_activity_types_all_mapped() {
    new_test_ext().execute_with(|| {
        // Flag one of each type (0..=7) — verifies no panic.
        for t in 0..=7u8 {
            assert_ok!(Compliance::flag_suspicious_activity(
                RuntimeOrigin::root(), 1, t, b"test".to_vec()
            ));
        }
        let activities = Compliance::suspicious_activities(1);
        assert_eq!(activities.len(), 8);
    });
}

#[test]
fn sanctions_entry_structure_correct() {
    new_test_ext().execute_with(|| {
        let hash = [0xAA; 32];
        assert_ok!(Compliance::add_sanctions_entry(
            RuntimeOrigin::root(), hash, b"EU".to_vec()
        ));
        let entry = Compliance::sanctions_list(hash).expect("entry must exist");
        assert!(entry.active);
        assert_eq!(entry.entity_hash, hash);
    });
}

#[test]
fn sync_verification_unverified_account_noop() {
    new_test_ext().execute_with(|| {
        // Account 3 never verified (last_verification = 0). Sync should not panic or restrict.
        assert_ok!(Compliance::sync_verification_from_identity(
            RuntimeOrigin::signed(3),
        ));
        let status = Compliance::compliance_status(3);
        // Should remain at defaults since last_verification == 0 doesn't trigger expiry.
        assert_eq!(status.verification_level, VerificationLevel::None);
        assert!(!status.restricted);
    });
}

#[test]
fn add_sanctions_source_too_long_fails() {
    new_test_ext().execute_with(|| {
        // list_source bounded to 32 bytes.
        let long_source = vec![b'z'; 64];
        assert_noop!(
            Compliance::add_sanctions_entry(RuntimeOrigin::root(), [0u8; 32], long_source),
            Error::<Test>::InvalidRiskAssessment
        );
    });
}
