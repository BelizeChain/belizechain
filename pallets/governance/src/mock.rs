use crate::{self as pallet_belize_governance, *};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, ConstU8, Everything, Get, Hooks, Randomness},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

// Simple test randomness implementation
pub struct TestRandomness;
impl Randomness<H256, u64> for TestRandomness {
    fn random(_subject: &[u8]) -> (H256, u64) {
        (H256::zero(), 0)
    }
}

// Mock compliance provider - allows all accounts except 999 (restricted test account)
pub struct MockCompliance;
impl ComplianceCheck<u64> for MockCompliance {
    fn can_participate_in_governance(account: &u64) -> bool {
        // Account 999 is restricted for testing
        *account != 999
    }

    fn eligible_voter_count() -> u32 {
        100 // Test assumes 100 eligible voters
    }
}

// Mock behavior flag provider - no flags by default, account 998 is flagged
pub struct MockBehaviorFlags;
impl BehaviorFlagProvider<u64, u64> for MockBehaviorFlags {
    fn has_active_flag(account: &u64) -> bool {
        *account == 998
    }
    fn cooldown_end(account: &u64) -> Option<u64> {
        if *account == 998 {
            Some(1_000_000)
        } else {
            None
        }
    }
}

// Mock dual-house provider - accounts 1 and 2 are technical, 2 and 3 are governance.
// `make_house_member` seats an arbitrary account in both houses from test and
// benchmark setup, mirroring what the runtime hook does with `pallet_collective`.
pub struct MockDualHouse;
impl DualHouseProvider<u64> for MockDualHouse {
    fn is_technical_house_member(account: &u64) -> bool {
        *account == 1 || *account == 2 || seated_house_members().contains(account)
    }
    fn is_governance_house_member(account: &u64) -> bool {
        *account == 2 || *account == 3 || seated_house_members().contains(account)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn make_house_member(account: &u64) {
        let mut members = seated_house_members();
        if !members.contains(account) {
            members.push(*account);
            frame_support::storage::unhashed::put(SEATED_HOUSE_MEMBERS_KEY, &members);
        }
    }
}

/// Raw-storage key holding accounts seated via `make_house_member`.
///
/// The mock has no `pallet_collective` to write to, so the seats live in a
/// dedicated key that is created fresh with each test externalities.
const SEATED_HOUSE_MEMBERS_KEY: &[u8] = b"pallet_belize_governance::mock::seated_house_members";

fn seated_house_members() -> sp_std::vec::Vec<u64> {
    frame_support::storage::unhashed::get::<sp_std::vec::Vec<u64>>(SEATED_HOUSE_MEMBERS_KEY)
        .unwrap_or_default()
}

// Mock community participation provider (Phase 6 integration)
pub struct MockCommunityParticipation;
impl pallet_belize_community::GovernanceParticipation<u64> for MockCommunityParticipation {
    fn record_proposal_submission(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }

    fn record_vote_cast(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }

    fn record_proposal_approval(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }

    fn record_council_membership(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }
}

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        BelizeGovernance: pallet_belize_governance,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeTask = ();
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const GovernancePalletId: PalletId = PalletId(*b"bz/govnc");
    pub const MinimumDeposit: u128 = 1_000_000_000; // 1000 DALLA
    pub const VotingPeriod: u64 = 100_800; // 7 days in blocks
    pub const LaunchPeriod: u64 = 28_800; // 2 days in blocks
    pub const ProposalCooldown: u64 = 0;
    pub const EnactmentPeriodConstitutional: u64 = 200;
    pub const EnactmentPeriodEconomic: u64 = 100;
    pub const EnactmentPeriodStandard: u64 = 50;
    pub const MaxVotingUnits: u32 = 1_000;
    pub const StakeUnitSize: u128 = 1_000_000_000; // 1000 DALLA per unit
    pub const LargeHolderStakeThreshold: u128 = 100_000_000_000; // 100K DALLA
    pub const ExitProofValidity: u64 = 432_000; // ~30 days
}

impl pallet_belize_governance::Config for Test {
    type Currency = Balances;
    type Randomness = TestRandomness;
    type CouncilOrigin = EnsureRoot<u64>;
    type CommunityOrigin = EnsureRoot<u64>;
    type ComplianceProvider = MockCompliance;
    type CommunityParticipation = MockCommunityParticipation;
    type MinimumDeposit = MinimumDeposit;
    type VotingPeriod = VotingPeriod;
    type LaunchPeriod = LaunchPeriod;
    type WeightInfo = ();
    type MaxCandidatesPerElection = ConstU32<50>;
    type MaxConsecutiveTerms = ConstU8<3>;
    type MaxPermanentTermExtensions = ConstU8<2>;
    type ProposalCooldown = ProposalCooldown;
    type EnactmentPeriodConstitutional = EnactmentPeriodConstitutional;
    type EnactmentPeriodEconomic = EnactmentPeriodEconomic;
    type EnactmentPeriodStandard = EnactmentPeriodStandard;
    type QuadraticVotingEnabled = frame_support::traits::ConstBool<false>;
    type MaxVotingUnits = MaxVotingUnits;
    type StakeUnitSize = StakeUnitSize;
    type LargeHolderStakeThreshold = LargeHolderStakeThreshold;
    type LargeHolderMinParticipationRate = ConstU8<50>;
    type ExitProofValidity = ExitProofValidity;
    type BehaviorFlags = MockBehaviorFlags;
    type DualHouseProvider = MockDualHouse;
    type ConstitutionalAdminOrigin = EnsureRoot<u64>;
    type EmergencyVetoWindow = ConstU64<100>;
    type RuntimeUpgradeMinTimelock = ConstU64<200>;
    type ParameterChangeMinTimelock = ConstU64<200>;
    type MaxTreasurySpendPerPeriod = ConstU128<500_000_000_000_000_000>;
    type TreasurySpendPeriod = ConstU64<14_400>;
    type MinQuorumPercentage = ConstU8<10>;
    // H-3: any signed account may reach the origin check in tests; the
    // dual-house ratification gate is what those tests exercise.
    type RuntimeUpgradeOrigin = frame_system::EnsureSigned<u64>;
}

// Build genesis storage according to the mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000_000_000), // Alice: 100k DALLA
            (2, 100_000_000_000), // Bob: 100k DALLA
            (3, 100_000_000_000), // Charlie: 100k DALLA
            (4, 50_000_000_000),  // Dave: 50k DALLA
            (5, 50_000_000_000),  // Eve: 50k DALLA
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Initialize governance with council members
    pallet_belize_governance::GenesisConfig::<Test> {
        council_members: vec![
            (1, 100, 200), // Alice: community_rank=100, pouw=200, weight=300
            (2, 80, 150),  // Bob: community_rank=80, pouw=150, weight=230
            (3, 60, 100),  // Charlie: community_rank=60, pouw=100, weight=160
        ],
        democracy_launch_period: 28_800,
        democracy_voting_period: 100_800,
        democracy_minimum_deposit: 1_000_000_000,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    // `frame_system::set_code` validates that the new blob's `spec_version` is
    // greater than the in-code one, reading it from the wasm via the
    // `runtime_version` host function. Pallet mocks have no executor, so the
    // host call is stubbed with a bumped copy of the mock's own version — the
    // same approach polkadot-sdk's `frame_system` tests take.
    ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(
        StubRuntimeVersion(bumped_runtime_version::<Test>()),
    ));
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

/// SCALE-encoded copy of `T`'s runtime version with `spec_version` bumped by one.
fn bumped_runtime_version<T: frame_system::Config>() -> Vec<u8> {
    let mut version = T::Version::get();
    version.spec_version = version.spec_version.saturating_add(1);
    codec::Encode::encode(&version)
}

/// Stub for the `runtime_version` host function used by `frame_system::set_code`.
struct StubRuntimeVersion(Vec<u8>);

impl sp_core::traits::ReadRuntimeVersion for StubRuntimeVersion {
    fn read_runtime_version(
        &self,
        _wasm_code: &[u8],
        _ext: &mut dyn sp_core::traits::Externalities,
    ) -> Result<Vec<u8>, String> {
        Ok(self.0.clone())
    }
}

// Test helper functions
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            BelizeGovernance::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        BelizeGovernance::on_initialize(System::block_number());
    }
}

/// Declare an emergency and advance past the CONS-029 veto window so it becomes active.
pub fn declare_and_activate_emergency(emergency_type: u8, description: &[u8], duration_hours: u32) {
    BelizeGovernance::declare_emergency(
        RuntimeOrigin::root(),
        emergency_type,
        description.to_vec(),
        duration_hours,
    )
    .expect("declare_emergency should succeed");
    // EmergencyVetoWindow = 100; advance past it to activate
    let current = System::block_number();
    run_to_block(current + 101);
}

pub fn last_event() -> RuntimeEvent {
    System::events().pop().expect("Event expected").event
}
