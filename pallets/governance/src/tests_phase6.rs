// Phase 6: Council Election System Tests
// Tests for district-based council elections in Belize

use crate::{
    mock::*, Error, Event, BelizeDistrict, ElectionStatus,
};
use frame_support::{assert_noop, assert_ok};

// Test account constants
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const DAVE: u64 = 4;
const EVE: u64 = 5;
const FERDIE: u64 = 6;

// ===== BASIC ELECTION WORKFLOW TESTS =====

#[test]
fn start_district_election_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election for Belize District (index 0)
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0, // Belize District
            3, // 3 seats
            100, // Registration period: 100 blocks
            200, // Voting period: 200 blocks
        ));

        // Check election was created
        let district = BelizeDistrict::Belize;
        let election = BelizeGovernance::get_district_election(district).unwrap();
        
        assert_eq!(election.district, district);
        assert_eq!(election.seats, 3);
        assert_eq!(election.status, ElectionStatus::Registration);
        assert_eq!(election.registration_end, 101); // Current block (1) + 100
        assert_eq!(election.voting_end, 301); // Registration end (101) + 200

        // Check event
        System::assert_has_event(
            Event::DistrictElectionStarted {
                election_id: 1,
                district_index: 0,
                seats: 3,
                registration_end: 101,
                voting_end: 301,
            }
            .into(),
        );
    });
}

#[test]
fn cannot_start_duplicate_election() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start first election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0, // Belize District
            3,
            100,
            200,
        ));

        // Try to start another election for same district
        assert_noop!(
            BelizeGovernance::start_district_election(
                RuntimeOrigin::root(),
                0, // Same district
                2,
                50,
                100,
            ),
            Error::<Test>::ElectionAlreadyActive
        );
    });
}

#[test]
fn invalid_district_index_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Try district index 6 (invalid, only 0-5 exist)
        assert_noop!(
            BelizeGovernance::start_district_election(
                RuntimeOrigin::root(),
                6, // Invalid
                2,
                100,
                200,
            ),
            Error::<Test>::InvalidDistrict
        );
    });
}

// ===== CANDIDATE REGISTRATION TESTS =====

#[test]
fn register_candidate_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0, // Belize District
            3,
            100,
            200,
        ));

        // Register candidate
        let platform = b"I will improve healthcare and education".to_vec();
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0, // Belize District
            platform.clone(),
        ));

        // Check candidate was registered
        let election = BelizeGovernance::get_district_election(BelizeDistrict::Belize).unwrap();
        let candidate = BelizeGovernance::get_election_candidate(election.id, ALICE).unwrap();
        
        assert_eq!(candidate.account, ALICE);
        assert_eq!(candidate.district, BelizeDistrict::Belize);
        assert_eq!(candidate.votes, 0);
        assert_eq!(candidate.platform.to_vec(), platform);

        // Check event
        System::assert_has_event(
            Event::CandidateRegistered {
                election_id: 1,
                candidate: ALICE,
                district_index: 0,
            }
            .into(),
        );
    });
}

#[test]
fn register_multiple_candidates_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            1, // Cayo District
            2,
            100,
            200,
        ));

        // Register 4 candidates
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            1,
            b"Alice platform".to_vec(),
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(BOB),
            1,
            b"Bob platform".to_vec(),
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(CHARLIE),
            1,
            b"Charlie platform".to_vec(),
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(DAVE),
            1,
            b"Dave platform".to_vec(),
        ));

        // Verify all 4 candidates registered
        let election = BelizeGovernance::get_district_election(BelizeDistrict::Cayo).unwrap();
        assert!(BelizeGovernance::get_election_candidate(election.id, ALICE).is_some());
        assert!(BelizeGovernance::get_election_candidate(election.id, BOB).is_some());
        assert!(BelizeGovernance::get_election_candidate(election.id, CHARLIE).is_some());
        assert!(BelizeGovernance::get_election_candidate(election.id, DAVE).is_some());
    });
}

#[test]
fn duplicate_candidate_registration_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            3,
            100,
            200,
        ));

        // Register once
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform 1".to_vec(),
        ));

        // Try to register again
        assert_noop!(
            BelizeGovernance::register_candidate(
                RuntimeOrigin::signed(ALICE),
                0,
                b"Platform 2".to_vec(),
            ),
            Error::<Test>::CandidateAlreadyRegistered
        );
    });
}

#[test]
fn register_after_registration_period_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election with short registration period
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            3,
            10, // Only 10 blocks
            200,
        ));

        // Move past registration end
        System::set_block_number(15);

        // Try to register
        assert_noop!(
            BelizeGovernance::register_candidate(
                RuntimeOrigin::signed(ALICE),
                0,
                b"Late platform".to_vec(),
            ),
            Error::<Test>::NotInRegistrationPhase
        );
    });
}

#[test]
fn register_without_election_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Try to register without starting election
        assert_noop!(
            BelizeGovernance::register_candidate(
                RuntimeOrigin::signed(ALICE),
                0,
                b"Platform".to_vec(),
            ),
            Error::<Test>::ElectionNotFound
        );
    });
}

// ===== VOTING TESTS =====

#[test]
fn vote_in_district_election_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0, // Belize District
            2,
            10, // Short registration
            200,
        ));

        // Register candidates
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Alice platform".to_vec(),
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(BOB),
            0,
            b"Bob platform".to_vec(),
        ));

        // Move to voting period
        System::set_block_number(15);

        // Vote for Alice
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(CHARLIE),
            0, // Belize District
            ALICE,
        ));

        // Check vote was recorded
        let election = BelizeGovernance::get_district_election(BelizeDistrict::Belize).unwrap();
        assert!(BelizeGovernance::has_voted_in_district_election(election.id, &CHARLIE));

        // Check candidate vote count
        let candidate = BelizeGovernance::get_election_candidate(election.id, ALICE).unwrap();
        assert_eq!(candidate.votes, 1);

        // Check event
        System::assert_has_event(
            Event::ElectionVoteCast {
                election_id: 1,
                voter: CHARLIE,
                candidate: ALICE,
                district_index: 0,
            }
            .into(),
        );
    });
}

#[test]
fn multiple_votes_recorded_correctly() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            2,
            10,
            200,
        ));

        // Register 3 candidates
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform A".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(BOB),
            0,
            b"Platform B".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(CHARLIE),
            0,
            b"Platform C".to_vec(),
        ));

        // Move to voting
        System::set_block_number(15);

        // Cast votes: Alice gets 3, Bob gets 2, Charlie gets 1
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(DAVE),
            0,
            ALICE,
        ));
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(EVE),
            0,
            ALICE,
        ));
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(FERDIE),
            0,
            ALICE,
        ));
        
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(7),
            0,
            BOB,
        ));
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(8),
            0,
            BOB,
        ));
        
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(9),
            0,
            CHARLIE,
        ));

        // Check vote counts
        let election = BelizeGovernance::get_district_election(BelizeDistrict::Belize).unwrap();
        assert_eq!(BelizeGovernance::get_election_candidate(election.id, ALICE).unwrap().votes, 3);
        assert_eq!(BelizeGovernance::get_election_candidate(election.id, BOB).unwrap().votes, 2);
        assert_eq!(BelizeGovernance::get_election_candidate(election.id, CHARLIE).unwrap().votes, 1);
        assert_eq!(election.total_votes, 6);
    });
}

#[test]
fn double_voting_prevented() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Setup election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            2,
            10,
            200,
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(BOB),
            0,
            b"Platform".to_vec(),
        ));

        // Move to voting
        System::set_block_number(15);

        // Vote once
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(CHARLIE),
            0,
            ALICE,
        ));

        // Try to vote again (even for different candidate)
        assert_noop!(
            BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(CHARLIE),
                0,
                BOB,
            ),
            Error::<Test>::AlreadyVotedInDistrictElection
        );
    });
}

#[test]
fn vote_for_nonexistent_candidate_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Setup election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            2,
            10,
            200,
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform".to_vec(),
        ));

        // Move to voting
        System::set_block_number(15);

        // Try to vote for non-registered candidate
        assert_noop!(
            BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(CHARLIE),
                0,
                BOB, // Not registered
            ),
            Error::<Test>::CandidateNotFound
        );
    });
}

#[test]
fn vote_during_registration_phase_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Setup election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            2,
            100, // Long registration
            200,
        ));

        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform".to_vec(),
        ));

        // Try to vote during registration (block 5, voting starts at 101)
        System::set_block_number(5);

        assert_noop!(
            BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(BOB),
                0,
                ALICE,
            ),
            Error::<Test>::NotInVotingPhase
        );
    });
}

// ===== ELECTION FINALIZATION TESTS =====

#[test]
fn finalize_district_election_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            2, // Corozal District (1 seat)
            1,
            10,
            20,
        ));

        // Register 3 candidates
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            2,
            b"Platform A".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(BOB),
            2,
            b"Platform B".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(CHARLIE),
            2,
            b"Platform C".to_vec(),
        ));

        // Vote (Alice: 5, Bob: 3, Charlie: 2)
        System::set_block_number(15);
        for voter in 10..15 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                2,
                ALICE,
            ));
        }
        for voter in 15..18 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                2,
                BOB,
            ));
        }
        for voter in 18..20 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                2,
                CHARLIE,
            ));
        }

        // Move past voting end
        System::set_block_number(35);

        // Finalize election
        assert_ok!(BelizeGovernance::finalize_district_election(
            RuntimeOrigin::root(),
            2, // Corozal
        ));

        // Check election finalized
        let election = BelizeGovernance::get_district_election(BelizeDistrict::Corozal).unwrap();
        assert_eq!(election.status, ElectionStatus::Finalized);

        // Check Alice is council member (won with 5 votes)
        assert!(BelizeGovernance::is_council_member(&ALICE));
        
        // Check district representation
        let reps = BelizeGovernance::get_district_representatives(BelizeDistrict::Corozal);
        assert_eq!(reps.len(), 1);
        assert_eq!(reps[0], ALICE);
    });
}

#[test]
fn finalize_multi_seat_election_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election with 3 seats
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0, // Belize District (3 seats)
            3,
            10,
            20,
        ));

        // Register 5 candidates
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform A".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(BOB),
            0,
            b"Platform B".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(CHARLIE),
            0,
            b"Platform C".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(DAVE),
            0,
            b"Platform D".to_vec(),
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(EVE),
            0,
            b"Platform E".to_vec(),
        ));

        // Vote: Alice=10, Bob=8, Charlie=6, Dave=4, Eve=2
        System::set_block_number(15);
        for voter in 10..20 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                0,
                ALICE,
            ));
        }
        for voter in 20..28 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                0,
                BOB,
            ));
        }
        for voter in 28..34 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                0,
                CHARLIE,
            ));
        }
        for voter in 34..38 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                0,
                DAVE,
            ));
        }
        for voter in 38..40 {
            assert_ok!(BelizeGovernance::vote_in_district_election(
                RuntimeOrigin::signed(voter),
                0,
                EVE,
            ));
        }

        // Move past voting end
        System::set_block_number(35);

        // Finalize
        assert_ok!(BelizeGovernance::finalize_district_election(
            RuntimeOrigin::root(),
            0,
        ));

        // Check top 3 winners are council members
        assert!(BelizeGovernance::is_council_member(&ALICE));
        assert!(BelizeGovernance::is_council_member(&BOB));
        assert!(BelizeGovernance::is_council_member(&CHARLIE));
        
        // Check losers are NOT council members
        assert!(!BelizeGovernance::is_council_member(&DAVE));
        assert!(!BelizeGovernance::is_council_member(&EVE));

        // Check district representation
        let reps = BelizeGovernance::get_district_representatives(BelizeDistrict::Belize);
        assert_eq!(reps.len(), 3);
        assert!(reps.contains(&ALICE));
        assert!(reps.contains(&BOB));
        assert!(reps.contains(&CHARLIE));
    });
}

#[test]
fn cannot_finalize_before_voting_ends() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            2,
            10,
            100, // Long voting period
        ));

        // Register and vote
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform".to_vec(),
        ));
        System::set_block_number(15);
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(BOB),
            0,
            ALICE,
        ));

        // Try to finalize before voting ends (block 15, ends at 111)
        assert_noop!(
            BelizeGovernance::finalize_district_election(
                RuntimeOrigin::root(),
                0,
            ),
            Error::<Test>::VotingPeriodNotEnded
        );
    });
}

#[test]
fn cannot_finalize_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Start election
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0,
            1,
            10,
            20,
        ));

        // Register and vote
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(ALICE),
            0,
            b"Platform".to_vec(),
        ));
        System::set_block_number(15);
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(BOB),
            0,
            ALICE,
        ));

        // Finalize once
        System::set_block_number(35);
        assert_ok!(BelizeGovernance::finalize_district_election(
            RuntimeOrigin::root(),
            0,
        ));

        // Try to finalize again
        assert_noop!(
            BelizeGovernance::finalize_district_election(
                RuntimeOrigin::root(),
                0,
            ),
            Error::<Test>::ElectionAlreadyFinalized
        );
    });
}

// ===== HELPER FUNCTION TESTS =====

#[test]
fn district_index_helper_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Belize), 0);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Cayo), 1);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Corozal), 2);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::OrangeWalk), 3);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::StannCreek), 4);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Toledo), 5);
    });
}

#[test]
fn election_status_index_helper_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Registration), 0);
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Voting), 1);
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Finalized), 2);
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Cancelled), 3);
    });
}

#[test]
fn district_seat_allocation_correct() {
    new_test_ext().execute_with(|| {
        // Verify seat allocations match Belize population distribution
        assert_eq!(BelizeDistrict::Belize.seat_allocation(), 3); // Largest
        assert_eq!(BelizeDistrict::Cayo.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::Corozal.seat_allocation(), 1); // Smallest
        assert_eq!(BelizeDistrict::OrangeWalk.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::StannCreek.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::Toledo.seat_allocation(), 2);
        
        // Total: 12 seats (matches MAX_COUNCIL_MEMBERS)
        let total_seats: u32 = [
            BelizeDistrict::Belize,
            BelizeDistrict::Cayo,
            BelizeDistrict::Corozal,
            BelizeDistrict::OrangeWalk,
            BelizeDistrict::StannCreek,
            BelizeDistrict::Toledo,
        ]
        .iter()
        .map(|d| d.seat_allocation())
        .sum();
        
        assert_eq!(total_seats, 12);
    });
}
