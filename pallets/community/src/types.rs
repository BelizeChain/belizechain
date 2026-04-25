//! Type definitions for the community pallet

use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::{BoundedVec, pallet_prelude::ConstU32};
use scale_info::TypeInfo;
use sp_core::H256;

// ================================
// Social Responsibility Score Types
// ================================

/// Complete SRS data for an account
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SRSData<BlockNumber> {
    /// Current total score (0-10,000 scale)
    pub score: u32,
    
    /// Score breakdown by category
    pub governance_score: u32,      // 0-2,500 (25%)
    pub education_score: u32,       // 0-1,500 (15%)
    pub sustainability_score: u32,  // 0-1,500 (15%)
    pub participation_score: u32,   // 0-2,500 (25%)
    pub peer_endorsements: u32,     // 0-1,000 (10%)
    pub honesty_rating: u32,        // 0-1,000 (10%)
    
    /// Tracking metadata
    pub last_updated: BlockNumber,
    pub tier: SRSTier,
    pub total_contributions: u32,
    
    /// Privacy settings
    pub public_display: bool,
    pub anonymous_hash: Option<H256>,
}

/// SRS tier classification
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, TypeInfo, MaxEncodedLen)]
#[derive(Default)]
pub enum SRSTier {
    #[default]
    Bronze,      // 0-2,499
    Silver,      // 2,500-4,999
    Gold,        // 5,000-7,499
    Platinum,    // 7,500-9,999
    Diamond,     // 10,000
}


impl SRSTier {
    pub fn as_u8(&self) -> u8 {
        match self {
            SRSTier::Bronze => 0,
            SRSTier::Silver => 1,
            SRSTier::Gold => 2,
            SRSTier::Platinum => 3,
            SRSTier::Diamond => 4,
        }
    }
}

// ================================
// Participation Tracking Types
// ================================

/// Single participation activity record
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct ParticipationRecord<BlockNumber> {
    pub activity_type: ActivityType,
    pub block_number: BlockNumber,
    pub value: u32, // Optional value for weighted activities
}

/// Type of participation activity tracked
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum ActivityType {
    /// Governance participation
    ProposalSubmission,
    VoteCast,
    ProposalApproved,
    CouncilMembership,
    
    /// Community participation
    EducationModuleCompleted,
    GreenProjectContribution,
    ReferralCompleted,
    
    /// Staking participation
    PoUWContribution(u32), // Value is PoUW score
    ValidatorActive,
}

impl ActivityType {
    pub fn as_u8(&self) -> u8 {
        match self {
            ActivityType::ProposalSubmission => 0,
            ActivityType::VoteCast => 1,
            ActivityType::ProposalApproved => 2,
            ActivityType::CouncilMembership => 3,
            ActivityType::EducationModuleCompleted => 4,
            ActivityType::GreenProjectContribution => 5,
            ActivityType::ReferralCompleted => 6,
            ActivityType::PoUWContribution(_) => 7,
            ActivityType::ValidatorActive => 8,
        }
    }
    
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ActivityType::ProposalSubmission),
            1 => Some(ActivityType::VoteCast),
            2 => Some(ActivityType::ProposalApproved),
            3 => Some(ActivityType::CouncilMembership),
            4 => Some(ActivityType::EducationModuleCompleted),
            5 => Some(ActivityType::GreenProjectContribution),
            6 => Some(ActivityType::ReferralCompleted),
            7 => Some(ActivityType::PoUWContribution(0)), // Default value for PoUW
            8 => Some(ActivityType::ValidatorActive),
            _ => None,
        }
    }
}

/// Proposal statistics for honesty score
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Default)]
pub struct ProposalStats {
    pub total: u32,
    pub approved: u32,
}

// ================================
// Endorsement Types
// ================================

/// Types of peer endorsements
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum EndorsementType {
    /// General positive contribution
    GeneralContribution,
    
    /// Excellence in specific areas
    GovernanceLeadership,
    CommunityService,
    EducationalImpact,
    EnvironmentalStewardship,
    TechnicalContribution,
    
    /// Cultural and social
    CulturalPreservation,
    YouthMentorship,
}

impl EndorsementType {
    pub fn as_u8(&self) -> u8 {
        match self {
            EndorsementType::GeneralContribution => 0,
            EndorsementType::GovernanceLeadership => 1,
            EndorsementType::CommunityService => 2,
            EndorsementType::EducationalImpact => 3,
            EndorsementType::EnvironmentalStewardship => 4,
            EndorsementType::TechnicalContribution => 5,
            EndorsementType::CulturalPreservation => 6,
            EndorsementType::YouthMentorship => 7,
        }
    }
    
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(EndorsementType::GeneralContribution),
            1 => Some(EndorsementType::GovernanceLeadership),
            2 => Some(EndorsementType::CommunityService),
            3 => Some(EndorsementType::EducationalImpact),
            4 => Some(EndorsementType::EnvironmentalStewardship),
            5 => Some(EndorsementType::TechnicalContribution),
            6 => Some(EndorsementType::CulturalPreservation),
            7 => Some(EndorsementType::YouthMentorship),
            _ => None,
        }
    }
}

// ================================
// Fee Exemption Types (Phase 2)
// ================================

/// Fee exemption data for zero-fee protocol
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct FeeExemptionData<BlockNumber, Balance> {
    /// Amount of fee exemption used this month
    pub used_this_month: Balance,
    
    /// Block number when usage was last reset
    pub last_reset_block: BlockNumber,
}

/// Fee exemption tiers
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum FeeExemptionTier {
    None,           // No exemption
    Basic,          // 100 dBZD/month (Bronze)
    Standard,       // 100 dBZD/month (Silver)
    Premium,        // 100 dBZD/month (Gold)
    Unlimited,      // Unlimited (Platinum+)
}

// ================================
// Community Proposal Types (Phase 3)
// ================================

/// Community proposal status
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum ProposalStatus {
    EthicsReview,   // Awaiting ethics council approval
    Active,         // Open for voting
    Approved,       // Passed and executed
    Rejected,       // Failed to pass
    Cancelled,      // Cancelled by proposer
}

/// Types of community proposals
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum CommunityProposalType {
    LocalProject,           // Community builder grants
    EducationModule,        // Learn-to-earn programs
    GreenInitiative,        // Sustainability projects
    CulturalPreservation,   // Garifuna/Creole/Maya heritage
    DisasterRelief,         // Crisis protocol payouts
    CommunityBounty,        // Task-based work
}

impl CommunityProposalType {
    pub fn as_u8(&self) -> u8 {
        match self {
            CommunityProposalType::LocalProject => 0,
            CommunityProposalType::EducationModule => 1,
            CommunityProposalType::GreenInitiative => 2,
            CommunityProposalType::CulturalPreservation => 3,
            CommunityProposalType::DisasterRelief => 4,
            CommunityProposalType::CommunityBounty => 5,
        }
    }
    
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(CommunityProposalType::LocalProject),
            1 => Some(CommunityProposalType::EducationModule),
            2 => Some(CommunityProposalType::GreenInitiative),
            3 => Some(CommunityProposalType::CulturalPreservation),
            4 => Some(CommunityProposalType::DisasterRelief),
            5 => Some(CommunityProposalType::CommunityBounty),
            _ => None,
        }
    }
}

// ================================
// Education Types (Phase 5)
// ================================

/// Green project types
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum GreenProjectType {
    CarbonOffset,
    CleanEnergy,
    Reforestation,
    WasteReduction,
    WildlifeProtection,
}

// ================================
// Ethics Filter Types (Phase 4)
// ================================

/// Proposal categories for ethics filtering
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum ProposalCategory {
    AddictiveProducts,      // Gambling, alcohol, tobacco
    SurveillanceTech,       // Privacy-invasive systems
    ExploitativePractices,  // Predatory lending, pyramid schemes
    EnvironmentalHarm,      // Projects damaging ecosystems
}

// ================================
// Phase 3: Community Proposal Data Structures
// ================================

/// Community proposal data
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(AccountId, Balance, BlockNumber))]
pub struct CommunityProposal<AccountId, Balance, BlockNumber> {
    pub proposer: AccountId,
    pub proposal_type: CommunityProposalType,
    pub beneficiary: AccountId,
    pub amount: Balance,
    pub title: BoundedVec<u8, ConstU32<128>>,
    pub description: BoundedVec<u8, ConstU32<1024>>,
    pub deposit: Balance,
    pub status: ProposalStatus,
    pub submission_block: BlockNumber,
    pub voting_deadline: BlockNumber,
    pub votes_for: u32,
    pub votes_against: u32,
    pub total_votes: u32,
}

/// Vote record
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct Vote {
    pub approve: bool,
    pub weight: u32,
}

// ================================
// Phase 4: Ethics Filter Types
// ================================

/// Ethics filter configuration
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(AccountId))]
pub struct EthicsConfig<AccountId> {
    /// Minimum honesty rating required (0-10,000 scale, default 5,000 = 50%)
    pub min_honesty_rating: u32,
    /// Require council review for large amounts
    pub council_review_threshold: u64,
    /// Minimum council approval votes (default 2 for 2/3 of 3-member council)
    pub min_council_votes: u32,
    /// Ethics council members
    pub council_members: BoundedVec<AccountId, ConstU32<10>>,
}

impl<AccountId> Default for EthicsConfig<AccountId> {
    fn default() -> Self {
        Self {
            min_honesty_rating: 5_000, // 50%
            council_review_threshold: 50_000, // 50,000 dBZD
            min_council_votes: 2, // 2/3 majority
            council_members: BoundedVec::default(),
        }
    }
}

/// Sanction status for an account
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SanctionStatus {
    pub active: bool,
    pub reason: BoundedVec<u8, ConstU32<128>>,
    pub sanctioned_at: u32, // Block number
}

// ==================== Phase 5: Incentive Programs ====================

/// Education module for learn-to-earn initiatives
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct EducationModule {
    pub id: u32,
    pub title: BoundedVec<u8, ConstU32<128>>,
    pub description: BoundedVec<u8, ConstU32<256>>,
    pub reward_amount: u64,
    pub total_completions: u32,
    pub max_completions: Option<u32>, // None = unlimited
    pub active: bool,
}

/// Record of education module completion
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct CompletionData {
    pub completed_at: u32, // Block number
    pub reward_claimed: bool,
}

/// Type of green/sustainability project
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum ProjectType {
    CarbonOffset,
    CleanEnergy,
    Reforestation,
    WasteReduction,
    WildlifeProtection,
}

/// Green project for sustainability contributions
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct GreenProject {
    pub id: u32,
    pub project_type: ProjectType,
    pub title: BoundedVec<u8, ConstU32<128>>,
    pub amount_contributed: u64,
    pub total_contributors: u32,
    pub active: bool,
}

/// Referral tracking data
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Default)]
pub struct ReferralData {
    pub total_referrals: u32,
    pub total_rewards_earned: u64,
}
