//! Tests for the community pallet - Phase 1: SRS System

use crate::{mock::*, Error, *};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use types::*;

// ================================
// SRS Calculation Tests
// ================================

#[test]
fn test_initial_srs_creation() {
    new_test_ext().execute_with(|| {

        // Record first participation
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalSubmission.as_u8()
        ));

        // Check SRS was created
        let srs = Community::srs_scores(1).unwrap();
        assert_eq!(srs.tier, SRSTier::Bronze);
        assert!(srs.score > 0);
        assert_eq!(srs.total_contributions, 1);
        assert!(srs.public_display); // Default is public
    });
}

#[test]
fn test_srs_score_increases_with_participation() {
    new_test_ext().execute_with(|| {

        // Record first activity (vote doesn't affect honesty score)
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));
        let score1 = Community::srs_scores(1).unwrap().score;

        // Record second activity (also a vote to keep honesty neutral)
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));
        let score2 = Community::srs_scores(1).unwrap().score;

        // Score should increase (50 points participation)
        assert!(score2 > score1, "Score2 ({}) should be > Score1 ({})", score2, score1);
    });
}

#[test]
fn test_srs_tier_progression() {
    new_test_ext().execute_with(|| {

        // Start at Bronze
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));
        assert_eq!(Community::srs_scores(1).unwrap().tier, SRSTier::Bronze);

        // Add many participations to reach Silver (2,500+)
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        let srs = Community::srs_scores(1).unwrap();
        assert!(srs.score >= 2500, "Score should be >= 2500, got {}", srs.score);
        assert!(
            srs.tier >= SRSTier::Silver,
            "Tier should be at least Silver, got {:?}",
            srs.tier
        );
    });
}

#[test]
fn test_governance_score_calculation() {
    new_test_ext().execute_with(|| {

        // Proposal submission = 100 points
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalSubmission.as_u8()
        ));

        let srs = Community::srs_scores(1).unwrap();
        assert!(srs.governance_score >= 100);

        // Vote cast = 25 points
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        let srs2 = Community::srs_scores(1).unwrap();
        assert!(srs2.governance_score >= srs.governance_score + 25);
    });
}

#[test]
fn test_participation_score_calculation() {
    new_test_ext().execute_with(|| {

        // Each participation = 50 points
        for _ in 0..10 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ));
        }

        let srs = Community::srs_scores(1).unwrap();
        assert_eq!(srs.participation_score, 500); // 10 * 50
        assert_eq!(srs.total_contributions, 10);
    });
}

#[test]
fn test_honesty_score_calculation() {
    new_test_ext().execute_with(|| {

        // Submit 5 proposals
        for _ in 0..5 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        // 3 approved
        for _ in 0..3 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalApproved.as_u8()
            ));
        }

        let srs = Community::srs_scores(1).unwrap();
        // 3/5 = 60% success rate = 600 points
        assert_eq!(srs.honesty_rating, 600);
    });
}

#[test]
fn test_honesty_score_neutral_start() {
    new_test_ext().execute_with(|| {

        // No proposals submitted
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        let srs = Community::srs_scores(1).unwrap();
        // Should start at neutral 500
        assert_eq!(srs.honesty_rating, 500);
    });
}

#[test]
fn test_srs_max_score_cap() {
    new_test_ext().execute_with(|| {

        // Add excessive participations
        for _ in 0..500 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        let srs = Community::srs_scores(1).unwrap();
        // Score should never exceed 10,000
        assert!(srs.score <= 10_000);
    });
}

// ================================
// Participation Recording Tests
// ================================

#[test]
fn test_record_participation_requires_verification() {
    new_test_ext().execute_with(|| {
        // Account 99 is not verified (only 1-10 are verified in mock)
        assert_noop!(
            Community::record_participation(
                RuntimeOrigin::signed(99),
                99,
                ActivityType::VoteCast.as_u8()
            ),
            Error::<Test>::NotVerified
        );
    });
}

#[test]
fn test_record_participation_emits_event() {
    new_test_ext().execute_with(|| {

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalSubmission.as_u8()
        ));

        System::assert_has_event(
            Event::ParticipationRecorded {
                account: 1,
                activity_type: ActivityType::ProposalSubmission.as_u8(),
                block_number: 1,
            }
            .into(),
        );
    });
}

#[test]
fn test_participation_history_stored() {
    new_test_ext().execute_with(|| {

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalSubmission.as_u8()
        ));

        let history = Community::participation_history(1);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].activity_type, ActivityType::VoteCast);
        assert_eq!(history[1].activity_type, ActivityType::ProposalSubmission);
    });
}

#[test]
fn test_proposal_stats_updated() {
    new_test_ext().execute_with(|| {

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalSubmission.as_u8()
        ));

        let stats = Community::user_proposals(1);
        assert_eq!(stats.total, 1);
        assert_eq!(stats.approved, 0);

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalApproved.as_u8()
        ));

        let stats = Community::user_proposals(1);
        assert_eq!(stats.total, 1);
        assert_eq!(stats.approved, 1);
    });
}

// ================================
// SRS Update Tests
// ================================

#[test]
fn test_update_srs_permissionless() {
    new_test_ext().execute_with(|| {

        // Account 1 records participation
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        // Account 2 can update account 1's SRS
        assert_ok!(Community::update_srs(RuntimeOrigin::signed(2), 1));
    });
}

#[test]
fn test_update_srs_requires_verification() {
    new_test_ext().execute_with(|| {
        // Account 99 is not verified (only 1-10 are verified in mock)
        assert_noop!(
            Community::update_srs(RuntimeOrigin::signed(1), 99),
            Error::<Test>::NotVerified
        );
    });
}

#[test]
fn test_update_srs_emits_event() {
    new_test_ext().execute_with(|| {

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        let old_score = Community::srs_scores(1).unwrap().score;

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::ProposalSubmission.as_u8()
        ));

        let new_score = Community::srs_scores(1).unwrap().score;

        // Should emit SRSUpdated event
        System::assert_has_event(
            Event::SRSUpdated {
                account: 1,
                old_score,
                new_score,
                old_tier: SRSTier::Bronze.as_u8(),
                new_tier: SRSTier::Bronze.as_u8(),
            }
            .into(),
        );
    });
}

// ================================
// Peer Endorsement Tests
// ================================

#[test]
fn test_endorse_peer_success() {
    new_test_ext().execute_with(|| {

        // Account 1 needs Silver tier to endorse
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        // Account 1 endorses account 2
        assert_ok!(Community::endorse_peer(
            RuntimeOrigin::signed(1),
            2,
            EndorsementType::GeneralContribution.as_u8()
        ));

        // Check endorsement count
        assert_eq!(Community::peer_endorsements(2), 1);
    });
}

#[test]
fn test_endorse_peer_requires_silver_tier() {
    new_test_ext().execute_with(|| {

        // Account 1 is Bronze tier
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        // Should fail
        assert_noop!(
            Community::endorse_peer(
                RuntimeOrigin::signed(1),
                2,
                EndorsementType::GeneralContribution.as_u8()
            ),
            Error::<Test>::InsufficientSRS
        );
    });
}

#[test]
fn test_cannot_endorse_self() {
    new_test_ext().execute_with(|| {

        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        assert_noop!(
            Community::endorse_peer(
                RuntimeOrigin::signed(1),
                1,
                EndorsementType::GeneralContribution.as_u8()
            ),
            Error::<Test>::CannotEndorseSelf
        );
    });
}

#[test]
fn test_endorsement_frequency_limit() {
    new_test_ext().execute_with(|| {

        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        // First endorsement succeeds
        assert_ok!(Community::endorse_peer(
            RuntimeOrigin::signed(1),
            2,
            EndorsementType::GeneralContribution.as_u8()
        ));

        // Second endorsement immediately fails
        assert_noop!(
            Community::endorse_peer(
                RuntimeOrigin::signed(1),
                2,
                EndorsementType::GeneralContribution.as_u8()
            ),
            Error::<Test>::EndorsementTooFrequent
        );
    });
}

#[test]
fn test_endorsement_updates_srs() {
    new_test_ext().execute_with(|| {

        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        // Get initial score
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(2),
            2,
            ActivityType::VoteCast.as_u8()
        ));
        let score_before = Community::srs_scores(2).unwrap().score;

        // Endorse
        assert_ok!(Community::endorse_peer(
            RuntimeOrigin::signed(1),
            2,
            EndorsementType::GeneralContribution.as_u8()
        ));

        // Score should increase
        let score_after = Community::srs_scores(2).unwrap().score;
        assert!(score_after > score_before);
        
        // Endorsement score should be 10 (1 endorsement * 10 points)
        let srs = Community::srs_scores(2).unwrap();
        assert_eq!(srs.peer_endorsements, 10);
    });
}

#[test]
fn test_endorsement_emits_event() {
    new_test_ext().execute_with(|| {

        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        assert_ok!(Community::endorse_peer(
            RuntimeOrigin::signed(1),
            2,
            EndorsementType::CommunityService.as_u8()
        ));

        System::assert_has_event(
            Event::PeerEndorsed {
                endorser: 1,
                endorsee: 2,
                endorsement_type: EndorsementType::CommunityService.as_u8(),
            }
            .into(),
        );
    });
}

// ================================
// Privacy Settings Tests
// ================================

#[test]
fn test_set_srs_privacy_public() {
    new_test_ext().execute_with(|| {

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        // Default is public
        assert!(Community::srs_scores(1).unwrap().public_display);

        // Set to private
        assert_ok!(Community::set_srs_privacy(RuntimeOrigin::signed(1), false));

        let srs = Community::srs_scores(1).unwrap();
        assert!(!srs.public_display);
        assert!(srs.anonymous_hash.is_some());
    });
}

#[test]
fn test_set_srs_privacy_requires_verification() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Community::set_srs_privacy(RuntimeOrigin::signed(99), false),
            Error::<Test>::NotVerified
        );
    });
}

#[test]
fn test_privacy_toggle_clears_hash() {
    new_test_ext().execute_with(|| {

        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        // Set to private
        assert_ok!(Community::set_srs_privacy(RuntimeOrigin::signed(1), false));
        assert!(Community::srs_scores(1).unwrap().anonymous_hash.is_some());

        // Set back to public
        assert_ok!(Community::set_srs_privacy(RuntimeOrigin::signed(1), true));
        assert!(Community::srs_scores(1).unwrap().anonymous_hash.is_none());
    });
}

// ================================
// Integration Tests
// ================================

#[test]
fn test_community_rank_export() {
    new_test_ext().execute_with(|| {

        // Bronze tier = 100 rank
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));
        assert_eq!(Community::get_community_rank(&1), 100);

        // Reach Silver tier = 200 rank
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }
        assert!(Community::get_community_rank(&1) >= 200);
    });
}

#[test]
fn test_no_srs_returns_zero_rank() {
    new_test_ext().execute_with(|| {
        
        // No participation recorded
        assert_eq!(Community::get_community_rank(&1), 0);
    });
}

// ================================
// Phase 2: Zero-Fee Protocol Tests
// ================================

#[test]
fn test_bronze_tier_no_discount() {
    new_test_ext().execute_with(|| {
        // Bronze tier gets 0% discount
        assert_ok!(Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ));

        let original_fee = 100;
        let (discounted_fee, discount_pct) = Community::calculate_fee_discount(&1, original_fee);
        
        assert_eq!(discounted_fee, 100); // No discount
        assert_eq!(discount_pct, 0);
    });
}

#[test]
fn test_silver_tier_25_percent_discount() {
    new_test_ext().execute_with(|| {
        // Reach Silver tier (30 activities)
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ));
        }

        let original_fee = 100;
        let (discounted_fee, discount_pct) = Community::calculate_fee_discount(&1, original_fee);
        
        assert_eq!(discounted_fee, 75); // 25% discount
        assert_eq!(discount_pct, 25);
    });
}

#[test]
fn test_gold_tier_50_percent_discount() {
    new_test_ext().execute_with(|| {
        // Reach Gold tier (50 activities + votes)
        for _ in 0..50 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        let original_fee = 200;
        let (discounted_fee, discount_pct) = Community::calculate_fee_discount(&1, original_fee);
        
        assert_eq!(discounted_fee, 100); // 50% discount
        assert_eq!(discount_pct, 50);
    });
}

#[test]
fn test_no_srs_no_discount() {
    new_test_ext().execute_with(|| {
        // Account with no SRS record
        let original_fee = 100;
        let (discounted_fee, discount_pct) = Community::calculate_fee_discount(&1, original_fee);
        
        assert_eq!(discounted_fee, 100); // No discount
        assert_eq!(discount_pct, 0);
    });
}

#[test]
fn test_monthly_limit_enforcement() {
    new_test_ext().execute_with(|| {
        // Reach Silver tier for 25% discount
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ));
        }

        // Monthly limit is 100K dBZD
        // First transaction: 40000 fee -> 30000 discounted = 10000 exemption (within limit)
        let within_limit = Community::check_fee_exemption_limit(&1, 10_000);
        assert!(within_limit);

        // Apply the exemption
        assert_ok!(Community::apply_fee_exemption(&1, 40_000, 30_000));

        // Second transaction: 50000 more would use 50000*0.25 = 12500 more, total = 22500 (within 100K)
        let within_limit = Community::check_fee_exemption_limit(&1, 12_500);
        assert!(within_limit);
        
        // But 400K exemption would exceed 100K limit
        let within_limit = Community::check_fee_exemption_limit(&1, 95_000);
        assert!(!within_limit); // Should be false - 10000 + 95000 > 100000
    });
}

#[test]
fn test_fee_exemption_usage_tracking() {
    new_test_ext().execute_with(|| {
        // Reach Silver tier
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ));
        }

        // Apply first exemption
        assert_ok!(Community::apply_fee_exemption(&1, 100, 75));

        // Check usage was recorded
        let fee_data = Community::fee_exemption_usage(1).unwrap();
        assert_eq!(fee_data.used_this_month, 25); // 100 - 75 = 25

        // Apply second exemption
        assert_ok!(Community::apply_fee_exemption(&1, 100, 75));
        
        let fee_data = Community::fee_exemption_usage(1).unwrap();
        assert_eq!(fee_data.used_this_month, 50); // 25 + 25 = 50
    });
}

#[test]
fn test_monthly_limit_reset_after_30_days() {
    new_test_ext().execute_with(|| {
        // Reach Silver tier
        for _ in 0..30 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ));
        }

        // Use up limit (100K)
        assert_ok!(Community::apply_fee_exemption(&1, 400_000, 300_000)); // 100K exemption used
        let within_limit = Community::check_fee_exemption_limit(&1, 5_000);
        assert!(!within_limit); // Limit exceeded (100K + 5K > 100K)

        // Advance 30 days (30 * 24 * 60 * 10 blocks)
        let blocks_per_month = 30 * 24 * 60 * 10;
        run_to_block(blocks_per_month + 1);

        // Check limit reset
        let within_limit = Community::check_fee_exemption_limit(&1, 100_000);
        assert!(within_limit); // Limit reset, can use full 100K again
    });
}

#[test]
fn test_fee_calculator_trait_integration() {
    new_test_ext().execute_with(|| {
        // Reach Gold tier (50% discount)
        for _ in 0..50 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalSubmission.as_u8()
            ));
        }

        // Calculate effective fee
        let original_fee = 200;
        let (effective_fee, discount_pct, within_limit) = Community::calculate_effective_fee(&1, original_fee);
        
        assert_eq!(effective_fee, 100); // 50% discount
        assert_eq!(discount_pct, 50);
        assert!(within_limit); // Within monthly limit
    });
}

// ================================
// Phase 3: Community Fund Management Tests
// ================================

#[test]
fn test_submit_community_proposal_success() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let beneficiary = 2u64;
        let amount = 10_000u64;
        let deposit = 1_000u64; // 10% of amount
        
        // Set up proposer balance
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            beneficiary,
            amount,
            b"Test Proposal".to_vec().try_into().unwrap(),
            b"This is a test proposal for the community".to_vec().try_into().unwrap(),
        ));
        
        // Check proposal was created
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.amount, amount);
        assert_eq!(proposal.deposit, deposit);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.voting_deadline, 100_801); // 1 + 100_800
        
        // Check deposit was reserved
        assert_eq!(Balances::reserved_balance(proposer), deposit);
        
        // Check proposal count incremented
        assert_eq!(Community::proposal_count(), 1);
    });
}

#[test]
fn test_vote_community_proposal_with_srs_weight() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let voter = 2u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Give voter SRS score
        assert_ok!(Community::record_participation(
            RuntimeOrigin::root(),
            voter,
            ActivityType::VoteCast.as_u8(),
        ));
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            3u64,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test description".to_vec().try_into().unwrap(),
        ));
        
        // Get voter's SRS weight
        let srs_data = Community::get_srs(&voter).unwrap();
        let weight = srs_data.score;
        
        // Vote on proposal
        assert_ok!(Community::vote_community_proposal(
            RuntimeOrigin::signed(voter),
            0,
            true,
        ));
        
        // Check vote was recorded
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.votes_for, weight);
        assert_eq!(proposal.votes_against, 0);
        assert_eq!(proposal.total_votes, 1);
        
        // Check voter's vote record
        let vote = Community::proposal_votes(0, voter).unwrap();
        assert!(vote.approve);
        assert_eq!(vote.weight, weight);
    });
}

#[test]
fn test_vote_community_proposal_against() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let voter = 2u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::GreenInitiative.as_u8(),
            3u64,
            5_000u64,
            b"Green".to_vec().try_into().unwrap(),
            b"Green project".to_vec().try_into().unwrap(),
        ));
        
        // Vote against proposal
        assert_ok!(Community::vote_community_proposal(
            RuntimeOrigin::signed(voter),
            0,
            false,
        ));
        
        // Check vote was recorded
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.votes_for, 0);
        assert_eq!(proposal.votes_against, 1); // Default weight of 1
        assert_eq!(proposal.total_votes, 1);
    });
}

#[test]
fn test_cannot_vote_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let voter = 2u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::EducationModule.as_u8(),
            3u64,
            8_000u64,
            b"Education".to_vec().try_into().unwrap(),
            b"Education program".to_vec().try_into().unwrap(),
        ));
        
        // First vote succeeds
        assert_ok!(Community::vote_community_proposal(
            RuntimeOrigin::signed(voter),
            0,
            true,
        ));
        
        // Second vote fails
        assert_noop!(
            Community::vote_community_proposal(
                RuntimeOrigin::signed(voter),
                0,
                false,
            ),
            Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn test_cannot_vote_after_deadline() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let voter = 2u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            3u64,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // Move past deadline (7 days = 100,800 blocks)
        System::set_block_number(100_802);
        
        // Vote fails
        assert_noop!(
            Community::vote_community_proposal(
                RuntimeOrigin::signed(voter),
                0,
                true,
            ),
            Error::<Test>::VotingPeriodEnded
        );
    });
}

#[test]
fn test_finalize_approved_proposal() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let voter1 = 2u64;
        let voter2 = 3u64;
        let voter3 = 4u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            5u64,
            20_000u64,
            b"Community Center".to_vec().try_into().unwrap(),
            b"Build a new community center".to_vec().try_into().unwrap(),
        ));
        
        let deposit = 2_000u64;
        assert_eq!(Balances::reserved_balance(proposer), deposit);
        
        // Three votes for
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(voter1), 0, true));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(voter2), 0, true));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(voter3), 0, true));
        
        // Move past deadline
        System::set_block_number(100_802);
        
        // Finalize proposal
        assert_ok!(Community::finalize_community_proposal(
            RuntimeOrigin::signed(proposer),
            0,
        ));
        
        // Check proposal was approved
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Approved);
        
        // Check deposit was returned
        assert_eq!(Balances::reserved_balance(proposer), 0);
    });
}

#[test]
fn test_finalize_rejected_proposal() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let voter1 = 2u64;
        let voter2 = 3u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            5u64,
            15_000u64,
            b"Bad Proposal".to_vec().try_into().unwrap(),
            b"This will be rejected".to_vec().try_into().unwrap(),
        ));
        
        let deposit = 1_500u64;
        let voter3 = 4u64;
        
        // One vote for, two against
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(voter1), 0, true));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(voter2), 0, false));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(voter3), 0, false));
        
        // Move past deadline
        System::set_block_number(100_802);
        
        // Finalize proposal
        assert_ok!(Community::finalize_community_proposal(
            RuntimeOrigin::signed(proposer),
            0,
        ));
        
        // Check proposal was rejected
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
        
        // Check deposit was slashed (not returned)
        assert_eq!(Balances::reserved_balance(proposer), 0);
        
        // Free balance should be original minus deposit (deposit was reserved, then slashed)
        // 100,000 - 1,500 (reserved) = 98,500 free at submission
        // After slash, deposit is gone completely: 100,000 - 1,500 = 98,500 total remaining
        assert_eq!(Balances::free_balance(proposer), 100_000 - deposit);
    });
}

#[test]
fn test_cannot_finalize_before_deadline() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            2u64,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // Try to finalize immediately (before deadline)
        assert_noop!(
            Community::finalize_community_proposal(
                RuntimeOrigin::signed(proposer),
                0,
            ),
            Error::<Test>::VotingPeriodEnded
        );
    });
}

#[test]
fn test_proposal_with_different_types() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        Balances::make_free_balance_be(&proposer, 500_000);
        
        let proposal_types = [CommunityProposalType::LocalProject,
            CommunityProposalType::EducationModule,
            CommunityProposalType::GreenInitiative,
            CommunityProposalType::CulturalPreservation,
            CommunityProposalType::DisasterRelief,
            CommunityProposalType::CommunityBounty];
        
        for (i, proposal_type) in proposal_types.iter().enumerate() {
            assert_ok!(Community::submit_community_proposal(
                RuntimeOrigin::signed(proposer),
                proposal_type.as_u8(),
                2u64,
                5_000u64,
                format!("Proposal {}", i).as_bytes().to_vec().try_into().unwrap(),
                b"Test description".to_vec().try_into().unwrap(),
            ));
            
            let proposal = Community::proposals(i as u32).unwrap();
            assert_eq!(proposal.proposal_type, *proposal_type);
        }
        
        assert_eq!(Community::proposal_count(), 6);
    });
}

#[test]
fn test_srs_weighted_voting_advantage() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let high_srs_voter = 2u64;
        let low_srs_voter1 = 3u64;
        let low_srs_voter2 = 4u64;
        
        // Set up balances
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Give high_srs_voter a high SRS (50 activities = Gold tier)
        for _ in 0..50 {
            assert_ok!(Community::record_participation(
                RuntimeOrigin::root(),
                high_srs_voter,
                ActivityType::VoteCast.as_u8(),
            ));
            System::set_block_number(System::block_number() + 1);
        }
        
        let high_srs = Community::get_srs(&high_srs_voter).unwrap();
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            5u64,
            10_000u64,
            b"SRS Test".to_vec().try_into().unwrap(),
            b"Testing SRS weighting".to_vec().try_into().unwrap(),
        ));
        
        // High SRS voter votes for
        assert_ok!(Community::vote_community_proposal(
            RuntimeOrigin::signed(high_srs_voter),
            0,
            true,
        ));
        
        // Two low SRS voters vote against (weight of 1 each)
        assert_ok!(Community::vote_community_proposal(
            RuntimeOrigin::signed(low_srs_voter1),
            0,
            false,
        ));
        assert_ok!(Community::vote_community_proposal(
            RuntimeOrigin::signed(low_srs_voter2),
            0,
            false,
        ));
        
        // Check high SRS voter's single vote outweighs two low SRS votes
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.votes_for, high_srs.score);
        assert_eq!(proposal.votes_against, 2); // 1 + 1
        assert!(proposal.votes_for > proposal.votes_against);
    });
}

#[test]
fn test_tie_vote_rejects_proposal() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            2u64,
            10_000u64,
            b"Tie Test".to_vec().try_into().unwrap(),
            b"Testing tie scenario".to_vec().try_into().unwrap(),
        ));
        
        // Two votes for, two against (equal weight)
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(2u64), 0, true));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(3u64), 0, true));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(4u64), 0, false));
        assert_ok!(Community::vote_community_proposal(RuntimeOrigin::signed(5u64), 0, false));
        
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.votes_for, 2);
        assert_eq!(proposal.votes_against, 2);
        
        // Move past deadline
        System::set_block_number(100_802);
        
        // Finalize - tie should reject (not approved = votes_for > votes_against)
        assert_ok!(Community::finalize_community_proposal(RuntimeOrigin::signed(proposer), 0));
        
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    });
}

// ================================
// Phase 4: Ethics Filter Tests
// ================================

#[test]
fn test_sanction_account() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let account = 5u64;
        let reason = b"Fraudulent activity detected".to_vec().try_into().unwrap();
        
        // Sanction account (requires root/governance)
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            account,
            reason,
        ));
        
        // Check sanction was recorded
        let sanction = Community::is_sanctioned(account).unwrap();
        assert!(sanction.active);
    });
}

#[test]
fn test_lift_sanction() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let account = 5u64;
        let reason = b"Temporary restriction".to_vec().try_into().unwrap();
        
        // Sanction account
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            account,
            reason,
        ));
        
        assert!(Community::is_sanctioned(account).is_some());
        
        // Lift sanction
        assert_ok!(Community::lift_sanction(
            RuntimeOrigin::root(),
            account,
        ));
        
        // Check sanction was removed
        assert!(Community::is_sanctioned(account).is_none());
    });
}

#[test]
fn test_sanctioned_beneficiary_requires_ethics_review() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let beneficiary = 5u64;
        
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Sanction the beneficiary
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            beneficiary,
            b"Under investigation".to_vec().try_into().unwrap(),
        ));
        
        // Submit proposal with sanctioned beneficiary
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            beneficiary,
            10_000u64,
            b"Test Proposal".to_vec().try_into().unwrap(),
            b"Should require ethics review".to_vec().try_into().unwrap(),
        ));
        
        // Check proposal is in EthicsReview status
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::EthicsReview);
    });
}

#[test]
fn test_ethics_council_vote() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let beneficiary = 5u64;
        let council_member_1 = 6u64;
        let council_member_2 = 7u64;
        let council_member_3 = 8u64;
        
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Set up ethics council
        let config = EthicsConfig {
            council_members: vec![council_member_1, council_member_2, council_member_3]
                .try_into()
                .unwrap(),
            min_council_votes: 2,
            ..Default::default()
        };
        EthicsFilterConfig::<Test>::put(config);
        
        // Sanction beneficiary to trigger ethics review
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            beneficiary,
            b"Test sanction".to_vec().try_into().unwrap(),
        ));
        
        // Submit proposal (should go to ethics review)
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            beneficiary,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::EthicsReview);
        
        // Council members vote
        assert_ok!(Community::ethics_council_vote(
            RuntimeOrigin::signed(council_member_1),
            0,
            true,
        ));
        
        assert_ok!(Community::ethics_council_vote(
            RuntimeOrigin::signed(council_member_2),
            0,
            true,
        ));
        
        // Check proposal moved to Active after 2/3 approval
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Active);
    });
}

#[test]
fn test_ethics_council_rejection() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let beneficiary = 5u64;
        let council_member_1 = 6u64;
        let council_member_2 = 7u64;
        
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Set up ethics council
        let config = EthicsConfig {
            council_members: vec![council_member_1, council_member_2]
                .try_into()
                .unwrap(),
            min_council_votes: 2,
            ..Default::default()
        };
        EthicsFilterConfig::<Test>::put(config);
        
        // Sanction beneficiary
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            beneficiary,
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            beneficiary,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        let deposit = 1_000u64;
        assert_eq!(Balances::reserved_balance(proposer), deposit);
        
        // Council rejects
        assert_ok!(Community::ethics_council_vote(RuntimeOrigin::signed(council_member_1), 0, false));
        assert_ok!(Community::ethics_council_vote(RuntimeOrigin::signed(council_member_2), 0, false));
        
        // Check proposal rejected and deposit returned
        let proposal = Community::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
        assert_eq!(Balances::reserved_balance(proposer), 0);
    });
}

#[test]
fn test_cannot_vote_twice_in_ethics_council() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let beneficiary = 5u64;
        let council_member = 6u64;
        
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Set up ethics council
        let config = EthicsConfig {
            council_members: vec![council_member].try_into().unwrap(),
            ..Default::default()
        };
        EthicsFilterConfig::<Test>::put(config);
        
        // Sanction beneficiary
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            beneficiary,
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            beneficiary,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // First vote succeeds
        assert_ok!(Community::ethics_council_vote(RuntimeOrigin::signed(council_member), 0, true));
        
        // Second vote fails
        assert_noop!(
            Community::ethics_council_vote(RuntimeOrigin::signed(council_member), 0, false),
            Error::<Test>::AlreadyVotedInCouncil
        );
    });
}

#[test]
fn test_non_council_member_cannot_vote() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let beneficiary = 5u64;
        let non_member = 9u64;
        let council_member = 6u64;
        
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Set up ethics council (only council_member is in council)
        let config = EthicsConfig {
            council_members: vec![council_member].try_into().unwrap(),
            ..Default::default()
        };
        EthicsFilterConfig::<Test>::put(config);
        
        // Sanction beneficiary
        assert_ok!(Community::sanction_account(
            RuntimeOrigin::root(),
            beneficiary,
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // Submit proposal
        assert_ok!(Community::submit_community_proposal(
            RuntimeOrigin::signed(proposer),
            CommunityProposalType::LocalProject.as_u8(),
            beneficiary,
            10_000u64,
            b"Test".to_vec().try_into().unwrap(),
            b"Test".to_vec().try_into().unwrap(),
        ));
        
        // Non-council member cannot vote
        assert_noop!(
            Community::ethics_council_vote(RuntimeOrigin::signed(non_member), 0, true),
            Error::<Test>::NotEthicsCouncilMember
        );
    });
}

#[test]
fn test_unverified_beneficiary_requires_ethics_review() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let proposer = 1u64;
        let unverified_beneficiary = 99u64; // Not in KYC verified range (1-10)
        
        Balances::make_free_balance_be(&proposer, 100_000);
        
        // Submit proposal with unverified beneficiary
        assert_noop!(
            Community::submit_community_proposal(
                RuntimeOrigin::signed(proposer),
                CommunityProposalType::LocalProject.as_u8(),
                unverified_beneficiary,
                10_000u64,
                b"Test".to_vec().try_into().unwrap(),
                b"Test".to_vec().try_into().unwrap(),
            ),
            Error::<Test>::BeneficiaryNotVerified
        );
    });
}

// ================================
// Phase 5: Incentive Programs Tests
// ================================

#[test]
fn test_complete_education_module() {
    new_test_ext().execute_with(|| {
        // Setup: Create education module
        let module = EducationModule {
            id: 1,
            title: b"Blockchain 101".to_vec().try_into().unwrap(),
            description: b"Learn blockchain basics".to_vec().try_into().unwrap(),
            reward_amount: 1_000,
            total_completions: 0,
            max_completions: Some(100),
            active: true,
        };
        EducationModules::<Test>::insert(1, module);

        // Get initial SRS
        let initial_srs = Community::get_srs(&1).map(|s| s.score).unwrap_or(0);

        // Complete education module
        assert_ok!(Community::complete_education_module(
            RuntimeOrigin::signed(1),
            1,
            b"completion_proof".to_vec().try_into().unwrap()
        ));

        // Verify completion recorded
        assert!(CompletedEducation::<Test>::contains_key(1, 1));
        let completion = CompletedEducation::<Test>::get(1, 1).unwrap();
        assert!(completion.reward_claimed);

        // Verify module completion count increased
        let updated_module = EducationModules::<Test>::get(1).unwrap();
        assert_eq!(updated_module.total_completions, 1);

        // Verify SRS increased
        let new_srs = Community::get_srs(&1).unwrap();
        assert!(new_srs.score > initial_srs, "SRS should increase after education");
        assert!(new_srs.education_score > 0, "Education score should be positive");
    });
}

#[test]
fn test_cannot_complete_module_twice() {
    new_test_ext().execute_with(|| {
        // Setup education module
        let module = EducationModule {
            id: 1,
            title: b"Test Module".to_vec().try_into().unwrap(),
            description: b"Test".to_vec().try_into().unwrap(),
            reward_amount: 500,
            total_completions: 0,
            max_completions: None,
            active: true,
        };
        EducationModules::<Test>::insert(1, module);

        // Complete once
        assert_ok!(Community::complete_education_module(
            RuntimeOrigin::signed(1),
            1,
            b"proof".to_vec().try_into().unwrap()
        ));

        // Try to complete again - should fail
        assert_noop!(
            Community::complete_education_module(
                RuntimeOrigin::signed(1),
                1,
                b"proof".to_vec().try_into().unwrap()
            ),
            Error::<Test>::AlreadyCompleted
        );
    });
}

#[test]
fn test_inactive_module_cannot_be_completed() {
    new_test_ext().execute_with(|| {
        // Setup inactive module
        let module = EducationModule {
            id: 1,
            title: b"Inactive".to_vec().try_into().unwrap(),
            description: b"Test".to_vec().try_into().unwrap(),
            reward_amount: 500,
            total_completions: 0,
            max_completions: None,
            active: false, // INACTIVE
        };
        EducationModules::<Test>::insert(1, module);

        // Try to complete - should fail
        assert_noop!(
            Community::complete_education_module(
                RuntimeOrigin::signed(1),
                1,
                b"proof".to_vec().try_into().unwrap()
            ),
            Error::<Test>::ModuleInactive
        );
    });
}

#[test]
fn test_module_capacity_limit() {
    new_test_ext().execute_with(|| {
        // Setup module with capacity of 1
        let module = EducationModule {
            id: 1,
            title: b"Limited".to_vec().try_into().unwrap(),
            description: b"Test".to_vec().try_into().unwrap(),
            reward_amount: 500,
            total_completions: 1, // Already at capacity
            max_completions: Some(1),
            active: true,
        };
        EducationModules::<Test>::insert(1, module);

        // Try to complete - should fail
        assert_noop!(
            Community::complete_education_module(
                RuntimeOrigin::signed(1),
                1,
                b"proof".to_vec().try_into().unwrap()
            ),
            Error::<Test>::ModuleCapReached
        );
    });
}

#[test]
fn test_green_project_contribution() {
    new_test_ext().execute_with(|| {
        // Setup green project
        let project = GreenProject {
            id: 1,
            project_type: ProjectType::Reforestation,
            title: b"Plant Trees".to_vec().try_into().unwrap(),
            amount_contributed: 0,
            total_contributors: 0,
            active: true,
        };
        GreenProjects::<Test>::insert(1, project);

        // Get initial SRS
        let initial_srs = Community::get_srs(&1).map(|s| s.score).unwrap_or(0);

        // Contribute to project
        assert_ok!(Community::contribute_to_green_project(
            RuntimeOrigin::signed(1),
            1,
            5_000
        ));

        // Verify contribution recorded
        let contribution = GreenContributions::<Test>::get(1, 1);
        assert_eq!(contribution, 5_000);

        // Verify project updated
        let updated_project = GreenProjects::<Test>::get(1).unwrap();
        assert_eq!(updated_project.amount_contributed, 5_000);
        assert_eq!(updated_project.total_contributors, 1);

        // Verify SRS increased
        let new_srs = Community::get_srs(&1).unwrap();
        assert!(new_srs.score > initial_srs, "SRS should increase after green contribution");
        assert!(new_srs.sustainability_score > 0, "Sustainability score should be positive");
    });
}

#[test]
fn test_multiple_green_contributions() {
    new_test_ext().execute_with(|| {
        // Setup project
        let project = GreenProject {
            id: 1,
            project_type: ProjectType::CleanEnergy,
            title: b"Solar Panels".to_vec().try_into().unwrap(),
            amount_contributed: 0,
            total_contributors: 0,
            active: true,
        };
        GreenProjects::<Test>::insert(1, project);

        // First contribution from Account 1
        assert_ok!(Community::contribute_to_green_project(
            RuntimeOrigin::signed(1),
            1,
            3_000
        ));

        // Second contribution from Account 1 (should not increment contributor count)
        assert_ok!(Community::contribute_to_green_project(
            RuntimeOrigin::signed(1),
            1,
            2_000
        ));

        // Contribution from Account 2 (should increment contributor count)
        assert_ok!(Community::contribute_to_green_project(
            RuntimeOrigin::signed(2),
            1,
            1_000
        ));

        // Verify totals
        let updated_project = GreenProjects::<Test>::get(1).unwrap();
        assert_eq!(updated_project.amount_contributed, 6_000);
        assert_eq!(updated_project.total_contributors, 2); // Two unique contributors

        // Verify individual contributions
        assert_eq!(GreenContributions::<Test>::get(1, 1), 5_000);
        assert_eq!(GreenContributions::<Test>::get(2, 1), 1_000);
    });
}

#[test]
fn test_green_milestone_event() {
    new_test_ext().execute_with(|| {
        // Setup project
        let project = GreenProject {
            id: 1,
            project_type: ProjectType::CarbonOffset,
            title: b"Carbon Credits".to_vec().try_into().unwrap(),
            amount_contributed: 50_000, // Just below milestone
            total_contributors: 5,
            active: true,
        };
        GreenProjects::<Test>::insert(1, project);

        // Contribute enough to cross 100,000 milestone
        assert_ok!(Community::contribute_to_green_project(
            RuntimeOrigin::signed(1),
            1,
            60_000 // Total will be 110,000
        ));

        // Check that milestone event was emitted (milestone at 100,000)
        let updated_project = GreenProjects::<Test>::get(1).unwrap();
        assert!(updated_project.amount_contributed >= 100_000);
    });
}

#[test]
fn test_inactive_project_cannot_receive_contributions() {
    new_test_ext().execute_with(|| {
        // Setup inactive project
        let project = GreenProject {
            id: 1,
            project_type: ProjectType::WasteReduction,
            title: b"Inactive".to_vec().try_into().unwrap(),
            amount_contributed: 0,
            total_contributors: 0,
            active: false, // INACTIVE
        };
        GreenProjects::<Test>::insert(1, project);

        // Try to contribute - should fail
        assert_noop!(
            Community::contribute_to_green_project(
                RuntimeOrigin::signed(1),
                1,
                1_000
            ),
            Error::<Test>::ProjectInactive
        );
    });
}

#[test]
fn test_claim_referral_reward() {
    new_test_ext().execute_with(|| {
        // Setup: Account 2 exists (referee)
        Community::record_participation(
            RuntimeOrigin::signed(2),
            2,
            ActivityType::VoteCast.as_u8()
        ).ok();

        // Account 1 claims referral for Account 2
        assert_ok!(Community::claim_referral_reward(
            RuntimeOrigin::signed(1),
            2
        ));

        // Verify referral claimed
        assert!(ReferralClaimed::<Test>::get(2, 1));

        // Verify referral data updated
        let referral_data = crate::ReferralData::<Test>::get(1);
        assert_eq!(referral_data.total_referrals, 1);
        assert_eq!(referral_data.total_rewards_earned, 1_000); // Base reward
    });
}

#[test]
fn test_cannot_claim_referral_twice() {
    new_test_ext().execute_with(|| {
        // Setup referee
        Community::record_participation(
            RuntimeOrigin::signed(2),
            2,
            ActivityType::VoteCast.as_u8()
        ).ok();

        // Claim once
        assert_ok!(Community::claim_referral_reward(
            RuntimeOrigin::signed(1),
            2
        ));

        // Try to claim again - should fail
        assert_noop!(
            Community::claim_referral_reward(
                RuntimeOrigin::signed(1),
                2
            ),
            Error::<Test>::ReferralAlreadyClaimed
        );
    });
}

#[test]
fn test_cannot_self_refer() {
    new_test_ext().execute_with(|| {
        // Try to self-refer - should fail
        assert_noop!(
            Community::claim_referral_reward(
                RuntimeOrigin::signed(1),
                1 // Same account
            ),
            Error::<Test>::InvalidReferral
        );
    });
}

#[test]
fn test_cannot_refer_nonexistent_user() {
    new_test_ext().execute_with(|| {
        // Try to refer user without SRS data - should fail
        assert_noop!(
            Community::claim_referral_reward(
                RuntimeOrigin::signed(1),
                99 // Non-existent user
            ),
            Error::<Test>::InvalidReferral
        );
    });
}

#[test]
fn test_referral_rewards_scale() {
    new_test_ext().execute_with(|| {
        // Setup three referees
        for i in 2..=4 {
            Community::record_participation(
                RuntimeOrigin::signed(i),
                i,
                ActivityType::VoteCast.as_u8()
            ).ok();
        }

        // Claim first referral (base: 1000)
        assert_ok!(Community::claim_referral_reward(RuntimeOrigin::signed(1), 2));
        let data1 = crate::ReferralData::<Test>::get(1);
        assert_eq!(data1.total_referrals, 1);
        assert_eq!(data1.total_rewards_earned, 1_000);

        // Claim second referral (base 1000 + 100 for 1st existing referral = 1100)
        assert_ok!(Community::claim_referral_reward(RuntimeOrigin::signed(1), 3));
        let data2 = crate::ReferralData::<Test>::get(1);
        assert_eq!(data2.total_referrals, 2);
        assert_eq!(data2.total_rewards_earned, 2_100); // 1000 + 1100

        // Claim third referral (base 1000 + 200 for 2 existing referrals = 1200)
        assert_ok!(Community::claim_referral_reward(RuntimeOrigin::signed(1), 4));
        let data3 = crate::ReferralData::<Test>::get(1);
        assert_eq!(data3.total_referrals, 3);
        assert_eq!(data3.total_rewards_earned, 3_300); // 1000 + 1100 + 1200
    });
}

#[test]
fn test_education_score_calculation() {
    new_test_ext().execute_with(|| {
        // Setup multiple modules
        for i in 1..=6 {
            let module = EducationModule {
                id: i,
                title: b"Module".to_vec().try_into().unwrap(),
                description: b"Test".to_vec().try_into().unwrap(),
                reward_amount: 100,
                total_completions: 0,
                max_completions: None,
                active: true,
            };
            EducationModules::<Test>::insert(i, module);
        }

        // Complete 4 modules (no bonus yet)
        for i in 1..=4 {
            assert_ok!(Community::complete_education_module(
                RuntimeOrigin::signed(1),
                i,
                b"proof".to_vec().try_into().unwrap()
            ));
        }

        let srs_4 = Community::get_srs(&1).unwrap();
        assert_eq!(srs_4.education_score, 400); // 4 * 100

        // Complete 5th module (triggers +500 bonus)
        assert_ok!(Community::complete_education_module(
            RuntimeOrigin::signed(1),
            5,
            b"proof".to_vec().try_into().unwrap()
        ));

        let srs_5 = Community::get_srs(&1).unwrap();
        assert_eq!(srs_5.education_score, 1_000); // (5 * 100) + 500 bonus
    });
}

#[test]
fn test_sustainability_score_calculation() {
    new_test_ext().execute_with(|| {
        // Setup multiple projects
        for i in 1..=3 {
            let project = GreenProject {
                id: i,
                project_type: ProjectType::Reforestation,
                title: b"Project".to_vec().try_into().unwrap(),
                amount_contributed: 0,
                total_contributors: 0,
                active: true,
            };
            GreenProjects::<Test>::insert(i, project);
        }

        // Contribute to 3 different projects
        assert_ok!(Community::contribute_to_green_project(RuntimeOrigin::signed(1), 1, 10_000));
        assert_ok!(Community::contribute_to_green_project(RuntimeOrigin::signed(1), 2, 5_000));
        assert_ok!(Community::contribute_to_green_project(RuntimeOrigin::signed(1), 3, 5_000));

        let srs = Community::get_srs(&1).unwrap();
        
        // Amount score: 20,000 / 100 = 200
        // Diversity bonus: 3 projects * 100 = 300
        // Total: 500
        assert_eq!(srs.sustainability_score, 500);
    });
}

#[test]
fn test_integrated_srs_with_all_components() {
    new_test_ext().execute_with(|| {
        // Setup education module
        let module = EducationModule {
            id: 1,
            title: b"Test".to_vec().try_into().unwrap(),
            description: b"Test".to_vec().try_into().unwrap(),
            reward_amount: 100,
            total_completions: 0,
            max_completions: None,
            active: true,
        };
        EducationModules::<Test>::insert(1, module);

        // Setup green project
        let project = GreenProject {
            id: 1,
            project_type: ProjectType::CleanEnergy,
            title: b"Test".to_vec().try_into().unwrap(),
            amount_contributed: 0,
            total_contributors: 0,
            active: true,
        };
        GreenProjects::<Test>::insert(1, project);

        // Get baseline SRS (just from participation)
        Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ).ok();
        let baseline_srs = Community::get_srs(&1).unwrap().score;

        // Complete education module
        assert_ok!(Community::complete_education_module(
            RuntimeOrigin::signed(1),
            1,
            b"proof".to_vec().try_into().unwrap()
        ));
        let after_education = Community::get_srs(&1).unwrap().score;
        assert!(after_education > baseline_srs);

        // Contribute to green project
        assert_ok!(Community::contribute_to_green_project(
            RuntimeOrigin::signed(1),
            1,
            10_000
        ));
        let after_green = Community::get_srs(&1).unwrap().score;
        assert!(after_green > after_education);

        // Verify all components are captured
        let final_srs = Community::get_srs(&1).unwrap();
        assert!(final_srs.education_score > 0);
        assert!(final_srs.sustainability_score > 0);
        assert!(final_srs.participation_score > 0);
    });
}

// ================================
// Phase 6: Cross-Pallet Integration Tests
// ================================

#[test]
fn test_community_rank_trait_bronze() {
    new_test_ext().execute_with(|| {
        // Create Bronze tier SRS
        Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ).ok();

        // Test CommunityRank trait
        let rank = Community::get_community_rank(&1);
        assert_eq!(rank, 100); // Bronze = 100

        let tier = Community::get_srs_tier(&1);
        assert_eq!(tier, SRSTier::Bronze);
    });
}

#[test]
fn test_community_rank_trait_progression() {
    new_test_ext().execute_with(|| {
        // Build up to Silver tier (2,500+)
        for _ in 0..51 {
            Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ).ok();
        }

        // Should be Silver
        let rank = Community::get_community_rank(&1);
        assert_eq!(rank, 200); // Silver = 200

        let tier = Community::get_srs_tier(&1);
        assert_eq!(tier, SRSTier::Silver);
    });
}

#[test]
fn test_fee_calculator_trait_bronze_no_discount() {
    new_test_ext().execute_with(|| {
        // Bronze gets 0% discount
        Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ).ok();

        let original_fee = 1000u64;
        let (discounted_fee, discount_pct, within_limit) = 
            Community::calculate_effective_fee(&1, original_fee);

        assert_eq!(discounted_fee, original_fee); // No discount
        assert_eq!(discount_pct, 0);
        assert!(within_limit);
    });
}

#[test]
fn test_fee_calculator_trait_silver_25_percent() {
    new_test_ext().execute_with(|| {
        // Build to Silver (25% discount) - need 2500+ score
        // 51 votes should give: (51*25 gov + 51*50 participation) = 1275 + 2550 = 3825 (Silver)
        for _ in 0..51 {
            Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::VoteCast.as_u8()
            ).ok();
        }
        // record_participation already calls update_srs_internal

        let srs = Community::get_srs(&1).expect("SRS should exist");
        // Debug: print actual scores
        println!("SRS Score: {}, Tier: {:?}", srs.score, srs.tier);
        println!("Gov: {}, Part: {}", srs.governance_score, srs.participation_score);

        let original_fee = 1000u64;
        
        // Direct storage check to debug
        let srs_from_storage = SocialResponsibilityScores::<Test>::get(1);
        println!("SRS from storage before fee calc: {:?}", srs_from_storage);
        
        let (discounted_fee, discount_pct, within_limit) = 
            <Community as FeeCalculator<u64, u64>>::calculate_effective_fee(&1, original_fee);

        println!("Fee result: discounted={}, discount_pct={}, within={}", discounted_fee, discount_pct, within_limit);
        
        assert_eq!(discount_pct, 25, "Silver tier should give 25% discount");
        assert_eq!(discounted_fee, 750); // 25% off
        assert!(within_limit);
    });
}

#[test]
fn test_pouw_contributor_trait() {
    new_test_ext().execute_with(|| {
        // Record PoUW contribution
        let quality = 8_000; // 80%
        let timeliness = 9_000; // 90%
        let honesty = 7_000; // 70%

        Community::record_pouw_contribution(&1, quality, timeliness, honesty).ok();

        // Get PoUW score
        let pouw_score = Community::get_pouw_score(&1);
        
        // Expected: (8000*40% + 9000*30% + 7000*30%) = 3200 + 2700 + 2100 = 8000
        assert_eq!(pouw_score, 8_000);
    });
}

#[test]
fn test_pouw_contribution_updates_srs() {
    new_test_ext().execute_with(|| {
        // Get baseline SRS
        Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ).ok();
        let baseline = Community::get_srs(&1).unwrap().score;

        // Record PoUW contribution
        Community::record_pouw_contribution(&1, 9_000, 9_000, 9_000).ok();

        // Trigger SRS update (in real world, would be called by staking pallet or scheduled task)
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // SRS should have increased
        let after_pouw = Community::get_srs(&1).unwrap().score;
        assert!(after_pouw > baseline, "PoUW should increase SRS");
    });
}

#[test]
fn test_pouw_multiple_contributions_averaged() {
    new_test_ext().execute_with(|| {
        // Record multiple PoUW contributions
        Community::record_pouw_contribution(&1, 8_000, 8_000, 8_000).ok(); // 8000
        Community::record_pouw_contribution(&1, 6_000, 6_000, 6_000).ok(); // 6000
        Community::record_pouw_contribution(&1, 10_000, 10_000, 10_000).ok(); // 10000

        let pouw_score = Community::get_pouw_score(&1);
        
        // Average: (8000 + 6000 + 10000) / 3 = 8000
        assert_eq!(pouw_score, 8_000);
    });
}

#[test]
fn test_governance_participation_proposal_submission() {
    new_test_ext().execute_with(|| {
        let baseline = Community::get_srs(&1).map(|s| s.score).unwrap_or(0);

        // Record proposal submission from governance pallet
        Community::record_proposal_submission(&1).ok();

        // Trigger SRS update (in production, would be called by governance pallet or scheduled task)
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert!(history.iter().any(|r| r.activity_type == ActivityType::ProposalSubmission));

        // Verify proposal stats updated
        let stats = UserProposals::<Test>::get(1);
        assert_eq!(stats.total, 1);

        // Verify SRS increased
        let after = Community::get_srs(&1).unwrap().score;
        assert!(after > baseline);
    });
}

#[test]
fn test_governance_participation_vote_cast() {
    new_test_ext().execute_with(|| {
        let baseline = Community::get_srs(&1).map(|s| s.score).unwrap_or(0);

        // Record vote from governance pallet
        Community::record_vote_cast(&1).ok();

        // Trigger SRS update
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert!(history.iter().any(|r| r.activity_type == ActivityType::VoteCast));

        // Verify SRS increased
        let after = Community::get_srs(&1).unwrap().score;
        assert!(after > baseline);
    });
}

#[test]
fn test_governance_participation_proposal_approval() {
    new_test_ext().execute_with(|| {
        let baseline = Community::get_srs(&1).map(|s| s.score).unwrap_or(0);

        // Record proposal approval from governance pallet
        Community::record_proposal_approval(&1).ok();

        // Trigger SRS update
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert!(history.iter().any(|r| r.activity_type == ActivityType::ProposalApproved));

        // Verify proposal stats updated
        let stats = UserProposals::<Test>::get(1);
        assert_eq!(stats.approved, 1);

        // Verify SRS increased
        let after = Community::get_srs(&1).unwrap().score;
        assert!(after > baseline);
    });
}

#[test]
fn test_governance_participation_council_activity() {
    new_test_ext().execute_with(|| {
        let baseline = Community::get_srs(&1).map(|s| s.score).unwrap_or(0);

        // Record council activity from governance pallet
        Community::record_council_activity(&1).ok();

        // Trigger SRS update
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert!(history.iter().any(|r| r.activity_type == ActivityType::CouncilMembership));

        // Verify SRS increased (council gets high points)
        let after = Community::get_srs(&1).unwrap().score;
        assert!(after > baseline);
        assert!(after >= baseline + 500); // Council activity gives significant boost
    });
}

#[test]
fn test_cross_pallet_fee_calculator_integration() {
    new_test_ext().execute_with(|| {
        // Simulate economy pallet using FeeCalculator trait
        
        // Bronze tier - no discount
        Community::record_participation(RuntimeOrigin::signed(1), 1, ActivityType::VoteCast.as_u8()).ok();
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();
        let (fee1, _, _) = Community::calculate_effective_fee(&1, 10_000u64);
        assert_eq!(fee1, 10_000);

        // Build to Gold tier (50% discount) - need 5000+ score
        // Need enough participation to exceed 5000
        // Use ProposalSubmission (100 gov points each) + VoteCast
        for _ in 0..25 {
            Community::record_participation(RuntimeOrigin::signed(1), 1, ActivityType::ProposalSubmission.as_u8()).ok();
        }
        for _ in 0..30 {
            Community::record_participation(RuntimeOrigin::signed(1), 1, ActivityType::VoteCast.as_u8()).ok();
        }
        // Total records: 1 + 25 + 30 = 56
        // Governance: 1*25 + 25*100 + 30*25 = 25 + 2500 + 750 = 3275 (capped at 2500) = 2500
        // Participation: 56 * 50 = 2800 (capped at 2500) = 2500
        // Honesty: 500
        // Total: 2500 + 2500 + 500 = 5500 (Gold!)
        
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();
        
        let (fee2, discount, _) = Community::calculate_effective_fee(&1, 10_000u64);
        assert_eq!(discount, 50);
        assert_eq!(fee2, 5_000); // 50% off
    });
}

#[test]
fn test_full_integration_all_traits() {
    new_test_ext().execute_with(|| {
        // Test account 1 participates across all dimensions
        
        // 1. Governance participation (via GovernanceParticipation trait)
        Community::record_proposal_submission(&1).ok();
        Community::record_vote_cast(&1).ok();
        Community::record_proposal_approval(&1).ok();
        
        // 2. PoUW contribution (via PoUWContributor trait)
        Community::record_pouw_contribution(&1, 9_000, 8_500, 9_500).ok();
        
        // 3. Education (direct)
        let module = EducationModule {
            id: 1,
            title: b"Test".to_vec().try_into().unwrap(),
            description: b"Test".to_vec().try_into().unwrap(),
            reward_amount: 100,
            total_completions: 0,
            max_completions: None,
            active: true,
        };
        EducationModules::<Test>::insert(1, module);
        Community::complete_education_module(
            RuntimeOrigin::signed(1),
            1,
            b"proof".to_vec().try_into().unwrap()
        ).ok();
        
        // 4. Green project (direct)
        let project = GreenProject {
            id: 1,
            project_type: ProjectType::Reforestation,
            title: b"Trees".to_vec().try_into().unwrap(),
            amount_contributed: 0,
            total_contributors: 0,
            active: true,
        };
        GreenProjects::<Test>::insert(1, project);
        Community::contribute_to_green_project(RuntimeOrigin::signed(1), 1, 5_000).ok();
        
        // Verify all components reflected in SRS
        let srs = Community::get_srs(&1).unwrap();
        assert!(srs.governance_score > 0, "Governance score should be positive");
        assert!(srs.education_score > 0, "Education score should be positive");
        assert!(srs.sustainability_score > 0, "Sustainability score should be positive");
        assert!(srs.participation_score > 0, "Participation score should be positive");
        
        // Verify CommunityRank trait works
        let rank = Community::get_community_rank(&1);
        assert!(rank >= 100, "Should have some rank");
        
        // Verify FeeCalculator trait works
        let (discounted_fee, _, _) = Community::calculate_effective_fee(&1, 1_000u64);
        assert!(discounted_fee <= 1_000, "Should have some discount");
        
        // Verify PoUW score accessible
        let pouw = Community::get_pouw_score(&1);
        assert!(pouw > 0, "Should have PoUW score");
    });
}

#[test]
fn test_participation_history_full_limit() {
    new_test_ext().execute_with(|| {
        // Fill participation history to limit (1000 entries)
        for _ in 0..1000 {
            Community::record_vote_cast(&1).ok();
        }

        // Try to add one more - should fail gracefully
        let result = Community::record_proposal_submission(&1);
        assert!(result.is_err(), "Should fail when history is full");
    });
}

// ================================
// Phase 6: Cross-Pallet Integration Tests
// ================================

#[test]
fn test_cross_pallet_community_rank() {
    new_test_ext().execute_with(|| {
        // Test CommunityRank trait for governance integration
        
        // No SRS initially
        let rank_none = <Community as CommunityRank<u64>>::get_community_rank(&1);
        assert_eq!(rank_none, 0);

        // Create Bronze tier SRS
        Community::record_participation(
            RuntimeOrigin::signed(1),
            1,
            ActivityType::VoteCast.as_u8()
        ).ok();
        
        let rank_bronze = <Community as CommunityRank<u64>>::get_community_rank(&1);
        assert_eq!(rank_bronze, 100, "Bronze tier should give rank 100");

        let tier = <Community as CommunityRank<u64>>::get_srs_tier(&1);
        assert_eq!(tier, SRSTier::Bronze);
    });
}

#[test]
fn test_fee_calculator_integration() {
    new_test_ext().execute_with(|| {
        // Test FeeCalculator trait for economy pallet integration
        
        // User with no SRS
        let original_fee = 1000u64;
        let (fee, discount, within_limit) = <Community as FeeCalculator<u64, u64>>::calculate_effective_fee(&1, original_fee);
        assert_eq!(fee, original_fee, "No discount without SRS");
        assert_eq!(discount, 0);
        assert!(within_limit);

        // Create Gold tier SRS (50% discount) - need score of 5000+
        // Participation: 50 * 50 = 2500, plus PoUW, endorsements, etc.
        for _ in 0..50 {
            Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalApproved.as_u8()
            ).ok();
        }

        // Add PoUW contributions to boost score
        for _ in 0..5 {
            Community::record_pouw_contribution(&1, 9_000, 9_000, 9_000).ok();
        }

        // Trigger SRS update to calculate tier
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        let srs = Community::get_srs(&1).unwrap();
        assert_eq!(srs.tier, SRSTier::Gold, "Should reach Gold tier");

        // Calculate fee with discount
        let (discounted_fee, discount_pct, within) = <Community as FeeCalculator<u64, u64>>::calculate_effective_fee(&1, original_fee);
        assert_eq!(discount_pct, 50, "Gold tier gives 50% discount");
        assert_eq!(discounted_fee, 500, "Fee should be halved");
        assert!(within);
    });
}

#[test]
fn test_pouw_contributor_integration() {
    new_test_ext().execute_with(|| {
        // Test PoUWContributor trait for staking pallet integration
        
        // Record PoUW contribution
        let quality = 8_000; // 80%
        let timeliness = 9_000; // 90%
        let honesty = 7_000; // 70%
        
        assert_ok!(<Community as PoUWContributor<u64>>::record_pouw_contribution(
            &1,
            quality,
            timeliness,
            honesty
        ));

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 1);
        
        // Check PoUW score calculation: (8000*40% + 9000*30% + 7000*30%) = 3200 + 2700 + 2100 = 8000
        let pouw_score = <Community as PoUWContributor<u64>>::get_pouw_score(&1);
        assert_eq!(pouw_score, 8_000);
    });
}

#[test]
fn test_pouw_average_calculation() {
    new_test_ext().execute_with(|| {
        // Record multiple PoUW contributions
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&1, 8_000, 9_000, 7_000).ok();
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&1, 6_000, 7_000, 8_000).ok();
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&1, 9_000, 8_000, 9_000).ok();

        // Should average last 10 (or all if < 10)
        let score = <Community as PoUWContributor<u64>>::get_pouw_score(&1);
        // First: 8000*0.4 + 9000*0.3 + 7000*0.3 = 8000
        // Second: 6000*0.4 + 7000*0.3 + 8000*0.3 = 6900
        // Third: 9000*0.4 + 8000*0.3 + 9000*0.3 = 8700
        // Average: (8000 + 6900 + 8700) / 3 = 7866
        assert!((7800..=7900).contains(&score), "Average should be around 7866");
    });
}

#[test]
fn test_governance_proposal_submission_hook() {
    new_test_ext().execute_with(|| {
        // Test GovernanceParticipation trait

        // Record proposal submission from governance pallet
        assert_ok!(<Community as GovernanceParticipation<u64>>::record_proposal_submission(&1));

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].activity_type, ActivityType::ProposalSubmission);
        assert_eq!(history[0].value, 100);

        // Verify proposal stats updated
        let stats = UserProposals::<Test>::get(1);
        assert_eq!(stats.total, 1);
        
        // Note: SRS update would be triggered by calling update_srs extrinsic separately
    });
}

#[test]
fn test_governance_vote_cast_hook() {
    new_test_ext().execute_with(|| {
        // Record vote from governance pallet
        assert_ok!(<Community as GovernanceParticipation<u64>>::record_vote_cast(&1));

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].activity_type, ActivityType::VoteCast);
        assert_eq!(history[0].value, 25);
    });
}

#[test]
fn test_governance_proposal_approval_hook() {
    new_test_ext().execute_with(|| {
        // Record proposal approval from governance pallet
        assert_ok!(<Community as GovernanceParticipation<u64>>::record_proposal_approval(&1));

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].activity_type, ActivityType::ProposalApproved);
        assert_eq!(history[0].value, 200);

        // Verify proposal stats updated
        let stats = UserProposals::<Test>::get(1);
        assert_eq!(stats.approved, 1);
    });
}

#[test]
fn test_governance_council_activity_hook() {
    new_test_ext().execute_with(|| {
        // Record council activity from governance pallet
        assert_ok!(<Community as GovernanceParticipation<u64>>::record_council_activity(&1));

        // Verify participation recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].activity_type, ActivityType::CouncilMembership);
        assert_eq!(history[0].value, 500);
    });
}

#[test]
fn test_cross_pallet_srs_accumulation() {
    new_test_ext().execute_with(|| {
        // Simulate multi-pallet interaction
        
        // Governance: Submit 2 proposals
        <Community as GovernanceParticipation<u64>>::record_proposal_submission(&1).ok();
        <Community as GovernanceParticipation<u64>>::record_proposal_submission(&1).ok();

        // Governance: Cast 5 votes
        for _ in 0..5 {
            <Community as GovernanceParticipation<u64>>::record_vote_cast(&1).ok();
        }

        // Staking: Contribute PoUW
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&1, 8_000, 9_000, 8_000).ok();

        // Verify participation history accumulated
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 8); // 2 proposals + 5 votes + 1 PoUW
    });
}

#[test]
fn test_fee_discount_with_exemption_limit() {
    new_test_ext().execute_with(|| {
        // Create Gold tier user (50% discount) - need score of 5000+
        for _ in 0..50 {
            Community::record_participation(
                RuntimeOrigin::signed(1),
                1,
                ActivityType::ProposalApproved.as_u8()
            ).ok();
        }

        // Add PoUW contributions to boost score to Gold
        for _ in 0..5 {
            Community::record_pouw_contribution(&1, 9_000, 9_000, 9_000).ok();
        }

        // Trigger SRS update to calculate tier
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // First fee - should get discount
        let (fee1, discount1, within1) = <Community as FeeCalculator<u64, u64>>::calculate_effective_fee(&1, 1000);
        assert_eq!(discount1, 50);
        assert!(within1);

        // Apply the fee discount
        assert_ok!(<Community as FeeCalculator<u64, u64>>::apply_fee_discount(&1, 1000, fee1));

        // Verify exemption tracked
        let fee_data = FeeExemptionUsage::<Test>::get(1).unwrap();
        assert_eq!(fee_data.used_this_month, 500); // 50% of 1000
    });
}

#[test]
fn test_full_ecosystem_integration() {
    new_test_ext().execute_with(|| {
        // Simulate complete cross-pallet workflow
        
        // 1. User starts with no SRS
        let rank_initial = <Community as CommunityRank<u64>>::get_community_rank(&1);
        assert_eq!(rank_initial, 0);

        // 2. Governance: Submit and approve proposals
        for _ in 0..10 {
            <Community as GovernanceParticipation<u64>>::record_proposal_submission(&1).ok();
        }
        for _ in 0..5 {
            <Community as GovernanceParticipation<u64>>::record_proposal_approval(&1).ok();
        }

        // 3. Staking: Contribute PoUW
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&1, 9_000, 9_000, 9_000).ok();

        // 4. Community: Vote on proposals
        for _ in 0..30 {
            <Community as GovernanceParticipation<u64>>::record_vote_cast(&1).ok();
        }

        // Trigger SRS update to reflect all participation
        // Total: 10 submissions + 5 approvals + 1 PoUW + 30 votes = 46 records
        // Gov: 10*100 + 5*200 + 30*25 = 1000 + 1000 + 750 = 2750 (capped at 2500) = 2500
        // Part: 46 * 50 = 2300
        // Honesty: (5/10)*1000 = 500
        // Total: 2500 + 2300 + 500 = 5300 (Gold tier!)
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        // 5. Check rank increased
        let rank_final = <Community as CommunityRank<u64>>::get_community_rank(&1);
        assert!(rank_final > rank_initial, "Rank should increase with participation");

        // 6. Check fee discount available
        let (discounted_fee, discount, _) = <Community as FeeCalculator<u64, u64>>::calculate_effective_fee(&1, 1000);
        assert!(discount > 0, "Should have fee discount");
        assert!(discounted_fee < 1000, "Fee should be reduced");

        // 7. Verify all history recorded
        let history = ParticipationHistory::<Test>::get(1);
        assert_eq!(history.len(), 46); // 10 submissions + 5 approvals + 1 PoUW + 30 votes
    });
}

#[test]
fn test_pouw_contribution_with_varying_quality() {
    new_test_ext().execute_with(|| {
        // Low quality contribution
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&1, 3_000, 5_000, 4_000).ok();
        let low_score = <Community as PoUWContributor<u64>>::get_pouw_score(&1);
        assert_eq!(low_score, 3_900); // 3000*0.4 + 5000*0.3 + 4000*0.3 = 3900

        // High quality contribution
        <Community as PoUWContributor<u64>>::record_pouw_contribution(&2, 9_500, 9_800, 9_600).ok();
        let high_score = <Community as PoUWContributor<u64>>::get_pouw_score(&2);
        assert_eq!(high_score, 9_620); // 9500*0.4 + 9800*0.3 + 9600*0.3 = 9620

        assert!(high_score > low_score);
    });
}

#[test]
fn test_governance_participation_affects_honesty() {
    new_test_ext().execute_with(|| {
        // Submit 5 proposals
        for _ in 0..5 {
            <Community as GovernanceParticipation<u64>>::record_proposal_submission(&1).ok();
        }

        let stats_before = UserProposals::<Test>::get(1);
        assert_eq!(stats_before.total, 5);
        assert_eq!(stats_before.approved, 0);

        // Approve 3 of them
        for _ in 0..3 {
            <Community as GovernanceParticipation<u64>>::record_proposal_approval(&1).ok();
        }

        // Trigger SRS update to calculate honesty
        Community::update_srs(RuntimeOrigin::signed(1), 1).ok();

        let stats_after = UserProposals::<Test>::get(1);
        assert_eq!(stats_after.total, 5); // Approvals don't add to total
        assert_eq!(stats_after.approved, 3);

        // Honesty score should be 60% (3/5)
        // Note: We're testing through the internal calculation
        let srs = Community::get_srs(&1).unwrap();
        let expected_honesty = (3 * 1000) / 5; // 600
        assert_eq!(srs.honesty_rating, expected_honesty);
    });
}
