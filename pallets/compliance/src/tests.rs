use crate::{mock::*, Error, Event, RiskLevel, VerificationLevel};
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
