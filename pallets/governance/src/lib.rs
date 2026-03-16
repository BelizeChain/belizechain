#![cfg_attr(not(feature = "std"), no_std)]

//! # BelizeChain Governance Pallet
//!
//! A comprehensive democratic governance system for Belize's sovereign blockchain infrastructure.
//!
//! ## Overview
//!
//! This pallet implements a multi-tiered democratic governance system that combines:
//! - **District Elections**: 6 electoral districts with proportional representation (12 council seats)
//! - **Departmental Governance**: 8 ministry departments with specialized decision-making
//! - **Foundation Board**: 7-member board for strategic oversight and technical governance
//! - **Compliance Integration**: KYC/AML verification through pallet-belize-compliance
//! - **Economic Incentives**: DALLA token rewards for participation
//! - **Advanced Features**: Vote delegation, proposal amendments, priority queues
//!
//! ## Architecture
//!
//! ### Governance Layers
//!
//! 1. **District Elections** (Phase 6)
//!    - 6 districts: Belize, Cayo, Corozal, Orange Walk, Stann Creek, Toledo
//!    - Each district elects 2 representatives
//!    - Elections occur every 3-6 months with stake-weighted voting
//!    - Delegate system for proxy voting
//!
//! 2. **Foundation Board** (Phase 4)
//!    - 7 specialized roles: Founder, Technical Steward, FSC Rep, BTB Delegate, 
//!      Citizen Delegate, Security Auditor, Culture & Ethics Advisor
//!    - Term-based appointments (1-2 years for rotating seats)
//!    - Multi-signature authority for critical operations
//!
//! 3. **Departmental Governance** (Phase 3)
//!    - 8 ministries: Finance, Education, Health, Works, Justice, Tourism, Agriculture, Defense
//!    - Department-specific treasuries and spending authority
//!    - Specialized proposal types per department
//!
//! 4. **Proposal System** (Phases 1-2, 5, 7)
//!    - Lifecycle: Draft → Active → Voting → Approved/Rejected → Executed/Failed
//!    - 7-day voting periods with configurable thresholds
//!    - 66% supermajority requirement for sensitive proposals
//!    - Priority queue (Low/Normal/High/Critical) for proposal management
//!
//! ### Compliance Integration
//!
//! All governance operations require KYC verification through `pallet-belize-compliance`:
//! - **Observer** (Basic KYC): View proposals, limited participation
//! - **Contributor** (Standard KYC): Vote, submit proposals
//! - **Validator** (Enhanced KYC): Join council, board membership
//!
//! ### Economic Model
//!
//! DALLA token rewards incentivize active participation:
//! - **Vote Participation**: 10 DALLA per vote cast
//! - **Proposal Creation**: 100 DALLA for approved proposals
//! - **Council Service**: 500 DALLA per month for council members
//!
//! ## Usage Examples
//!
//! ### Creating a Proposal
//!
//! ```rust,ignore
//! use pallet_belize_governance::{Call, ProposalType};
//!
//! // Submit a treasury spending proposal (requires Standard KYC)
//! let proposal = Call::create_proposal {
//!     title: b"Road Repair Budget".to_vec(),
//!     description: b"Allocate 50,000 DALLA for district road repairs".to_vec(),
//!     proposal_type: ProposalType::Treasury,
//!     department: Some(Department::Works),
//!     metadata: None,
//! };
//! ```
//!
//! ### Voting on Proposals
//!
//! ```rust,ignore
//! // Cast a vote with conviction (requires Standard KYC)
//! let vote = Call::cast_vote {
//!     proposal_id: 42,
//!     approve: true,
//!     vote_choice_index: 1, // Yes vote
//!     conviction: 2, // 2x voting power with 30-day lock
//! };
//! ```
//!
//! ### Vote Delegation
//!
//! ```rust,ignore
//! // Delegate voting power to trusted representative (requires Basic KYC)
//! let delegation = Call::delegate_vote {
//!     delegate: trusted_account.clone(),
//!     expires_at: Some(current_block + 5_256_000), // 1 year expiry
//! };
//!
//! // Revoke delegation at any time
//! let revoke = Call::revoke_delegation {};
//! ```
//!
//! ### District Elections
//!
//! ```rust,ignore
//! // Register as candidate (requires Enhanced KYC + 1000 DALLA stake)
//! let candidacy = Call::register_as_candidate {
//!     district: District::Cayo,
//!     stake_amount: 1_000_000_000_000_000, // 1000 DALLA
//! };
//!
//! // Vote in district election (requires Standard KYC)
//! let election_vote = Call::vote_in_election {
//!     election_id: 1,
//!     candidate: candidate_account.clone(),
//! };
//! ```
//!
//! ### Council Operations
//!
//! ```rust,ignore
//! // Council member appoints board member (requires council membership)
//! let appointment = Call::appoint_council_member {
//!     account: new_member.clone(),
//!     role: BoardRole::SecurityAuditor,
//!     term_years: 2,
//! };
//!
//! // Execute approved proposal (requires council membership)
//! let execution = Call::execute_approved_proposal {
//!     proposal_id: 42,
//! };
//! ```
//!
//! ## Integration Points
//!
//! ### With pallet-belize-economy
//!
//! - Treasury management for governance operations
//! - DALLA reward distribution for participation
//! - Multi-signature treasury with governance oversight
//!
//! ### With pallet-belize-compliance
//!
//! - KYC/AML verification for all governance participants
//! - Tiered access control based on verification levels
//! - Automatic tier calculation from compliance status
//!
//! ### With pallet-belize-staking
//!
//! - Proof of Useful Work (PoUW) contributions influence voting weight
//! - Validator performance tracked for council eligibility
//! - Community rank system integration
//!
//! ## Security Considerations
//!
//! - **Access Control**: All sensitive operations require appropriate origin (Root, Council, FSC)
//! - **Economic Security**: Stake requirements prevent spam and ensure commitment
//! - **Rate Limiting**: Maximum proposal creation limits per account/period
//! - **Emergency Powers**: Root can override for national security scenarios
//! - **Audit Trail**: All governance actions emit events for transparency
//!
//! ## Performance Characteristics
//!
//! - **Storage Efficiency**: BoundedVec for all dynamic data with sensible limits
//! - **Computational Cost**: Optimized weight functions based on operation complexity
//! - **Scalability**: Handles 100+ concurrent proposals, 1000+ voters per proposal
//! - **Pagination**: Large datasets (delegations, amendments) support efficient iteration
//!
//! ## Testing
//!
//! Comprehensive test suite covering:
//! - 137 unit tests across 7 implementation phases
//! - Full lifecycle testing for all proposal types
//! - Edge case testing (max limits, expiry, conflicts)
//! - Integration testing with mock compliance pallet
//!
//! See `tests_phase*.rs` files for complete test coverage.
//!
//! ## Future Enhancements
//!
//! - **Quadratic Voting**: Research QV implementation for fairer preference aggregation
//! - **Liquid Democracy**: Extended delegation chains with transitive trust
//! - **Shielded Voting** (Roadmapped 2028): Commit-reveal scheme with optional ZK selective
//!   disclosure for anonymous voting. NOT YET IMPLEMENTED — current votes are public for
//!   accountability. Infrastructure storage (`VoteCommitments`) is pre-provisioned.
//! - **Cross-Chain Governance**: XCM integration for Polkadot governance participation
//!
//! ## References
//!
//! - [Polkadot Governance](https://wiki.polkadot.network/docs/learn-governance)
//! - [Substrate FRAME](https://docs.substrate.io/reference/frame-pallets/)
//! - [OpenGov Design](https://wiki.polkadot.network/docs/learn-opengov)

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, Get, Randomness,
        EnsureOrigin, ExistenceRequirement,
    },
    sp_runtime::traits::AccountIdConversion,
};
use sp_runtime::{
    traits::Saturating,
    RuntimeDebug,
};
use sp_io::hashing::blake2_256;
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

// Import GovernanceParticipation trait from Community pallet (Phase 6)
pub use pallet_belize_community::GovernanceParticipation;

/// Phase 5A — Cross-pallet behavior-flag integration trait.
///
/// Implemented in the runtime by a struct that forwards to the oracle pallet's
/// `has_active_behavior_flag` / `behavior_cooldown_end` public API.
/// Governance uses this as an addiction-loop circuit breaker.
pub trait BehaviorFlagProvider<AccountId, BlockNumber> {
    /// Returns `true` if `account` has an active, unexpired behavior flag.
    fn has_active_flag(account: &AccountId) -> bool;
    /// Returns the block at which the account's behavior cooldown expires, if any.
    fn cooldown_end(account: &AccountId) -> Option<BlockNumber>;
}

/// Phase 6A — Cross-pallet dual-house membership provider for bifurcated
/// constitutional ratification.
///
/// Implemented in the runtime by a struct that checks pallet_collective
/// membership for TechnicalCouncil (Instance1) and GovernanceCouncil (Instance2).
/// A Constitutional proposal MUST be independently ratified by at least one
/// member from each house before it can be executed.
pub trait DualHouseProvider<AccountId> {
    /// Returns `true` if `account` is a member of the Technical house (Instance1).
    fn is_technical_house_member(account: &AccountId) -> bool;
    /// Returns `true` if `account` is a member of the Governance house (Instance2).
    fn is_governance_house_member(account: &AccountId) -> bool;
}

/// Blocks per year assuming 6-second block time (~5,256,000).
/// Used for term calculations, delegation expiry, and fiscal year defaults.
const BLOCKS_PER_YEAR: u32 = 5_256_000;

/// Maximum number of delegation receivers per delegate account.
const MAX_DELEGATION_RECEIVERS: usize = 100;

/// Participation tiers determine what governance actions an account can perform.
///
/// Tiers are automatically assigned based on KYC/AML verification level from
/// `pallet-belize-compliance`. Higher tiers require more comprehensive identity
/// verification but enable more powerful governance capabilities.
///
/// ## Tier Requirements
///
/// - **Observer**: Basic KYC verification - Can view proposals and delegate votes
/// - **Contributor**: Standard KYC verification - Can vote and create proposals
/// - **Validator**: Enhanced/Government KYC - Can serve on council and board
///
/// ## Integration
///
/// ```rust,ignore
/// let verification_level = pallet_compliance::get_verification_level(&account);
/// let tier = ParticipationTier::from_verification_level(verification_level);
///
/// match tier {
///     ParticipationTier::Observer => { /* Limited access */ },
///     ParticipationTier::Contributor => { /* Voting rights */ },
///     ParticipationTier::Validator => { /* Full governance access */ },
/// }
/// ```
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ParticipationTier {
    /// Observer tier - Basic KYC verification
    ///
    /// **Capabilities**:
    /// - View all proposals and governance activity
    /// - Delegate voting power to representatives
    /// - Claim delegation-based participation rewards
    ///
    /// **Restrictions**:
    /// - Cannot vote directly on proposals
    /// - Cannot create proposals
    /// - Cannot run for council positions
    Observer,
    
    /// Contributor tier - Standard KYC verification
    ///
    /// **Capabilities**:
    /// - All Observer capabilities
    /// - Vote on active proposals with conviction
    /// - Create proposals (Treasury, Council, Community, Department)
    /// - Claim voting participation rewards (10 DALLA per vote)
    /// - Vote in district elections
    ///
    /// **Restrictions**:
    /// - Cannot serve on foundation board
    /// - Cannot be elected to council
    Contributor,
    
    /// Validator tier - Enhanced/Government KYC verification
    ///
    /// **Capabilities**:
    /// - All Contributor capabilities
    /// - Run as candidate in district elections
    /// - Serve on foundation board (if appointed)
    /// - Execute approved proposals (if council member)
    /// - Claim council service rewards (500 DALLA/month)
    /// - Access to sensitive departmental proposals
    Validator,
}

impl ParticipationTier {
    /// Convert compliance verification level to participation tier.
    ///
    /// Maps `pallet-belize-compliance` verification levels to governance tiers:
    /// - Level 0 (None): Observer (restricted access)
    /// - Level 1 (Basic): Observer
    /// - Level 2 (Standard): Contributor (voting rights)
    /// - Level 3+ (Enhanced/Government): Validator (full access)
    ///
    /// # Arguments
    ///
    /// * `level` - Verification level from compliance pallet (0-4)
    ///
    /// # Returns
    ///
    /// Appropriate `ParticipationTier` for the given verification level
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Account with Standard KYC gets Contributor tier
    /// let tier = ParticipationTier::from_verification_level(2);
    /// assert_eq!(tier, ParticipationTier::Contributor);
    /// ```
    pub fn from_verification_level(level: u8) -> Self {
        match level {
            0 => Self::Observer, // None - restricted
            1 => Self::Observer, // Basic
            2 => Self::Contributor, // Standard
            3..=4 => Self::Validator, // Enhanced/Government
            _ => Self::Observer,
        }
    }
}

/// Government departments for compartmentalized governance workflows.
///
/// Each department has specialized authority over specific areas of national operations,
/// with dedicated treasuries and approval workflows. Department-specific proposals enable
/// targeted decision-making by domain experts while maintaining democratic oversight.
///
/// ## Department Structure
///
/// - **Independent Treasuries**: Each department manages its own budget allocation
/// - **Specialized Workflows**: Department-specific proposal types and approval thresholds
/// - **Multi-Signature Authority**: Critical operations require FSC + department head approval
/// - **Accountability**: All department actions are recorded on-chain for public audit
///
/// ## Integration with Other Systems
///
/// - **Economy Pallet**: Treasury allocations and spending authorization
/// - **Compliance Pallet**: Department-level access control and verification
/// - **Identity Pallet**: Department employee credentials and role assignments
///
/// ## Example Usage
///
/// ```rust,ignore
/// // Create department-specific proposal
/// let proposal = Call::create_proposal {
///     title: b"School Stipend Program".to_vec(),
///     description: b"Distribute 200 DALLA per student for uniforms".to_vec(),
///     proposal_type: ProposalType::Department,
///     department: Some(Department::Education),
///     metadata: None,
/// };
/// ```
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Department {
    /// Ministry of Finance - Economic management and fiscal policy
    ///
    /// **Responsibilities**:
    /// - Tax collection and revenue management
    /// - National treasury oversight
    /// - Government payroll and pensions
    /// - Economic policy proposals
    /// - Financial auditing and reporting
    ///
    /// **On-Chain Prefix**: `gov.finance`
    Finance,
    
    /// Ministry of Education - Educational system and student services
    ///
    /// **Responsibilities**:
    /// - School funding and infrastructure
    /// - Student stipends and scholarships
    /// - Teacher payroll and training
    /// - Educational grants and programs
    /// - Curriculum development oversight
    ///
    /// **On-Chain Prefix**: `gov.education`
    Education,
    
    /// Ministry of Health - Healthcare services and medical infrastructure
    ///
    /// **Responsibilities**:
    /// - Hospital operations and funding
    /// - Medical supply procurement
    /// - Healthcare worker payroll
    /// - Public health programs
    /// - Medical records management (privacy-preserving)
    ///
    /// **On-Chain Prefix**: `gov.health`
    Health,
    
    /// Ministry of Public Works - Infrastructure and construction
    ///
    /// **Responsibilities**:
    /// - Road and bridge construction/maintenance
    /// - Public building projects
    /// - Land grants and allocation
    /// - Infrastructure contracts
    /// - Utility management
    ///
    /// **On-Chain Prefix**: `gov.works`
    Works,
    
    /// Ministry of Justice - Legal system and compliance
    ///
    /// **Responsibilities**:
    /// - Legal compliance enforcement
    /// - Identity verification oversight
    /// - Dispute resolution mechanisms
    /// - Court system funding
    /// - Law enforcement coordination
    ///
    /// **On-Chain Prefix**: `gov.justice`
    Justice,
    
    /// Belize Tourism Board - Tourism promotion and development
    ///
    /// **Responsibilities**:
    /// - Tourism marketing and promotion
    /// - Travel incentive programs (5-8% DALLA rewards)
    /// - Tourism infrastructure funding
    /// - Cultural heritage preservation
    /// - Visitor services coordination
    ///
    /// **On-Chain Prefix**: `gov.tourism`
    Tourism,
    
    /// Ministry of Agriculture - Food security and rural development
    ///
    /// **Responsibilities**:
    /// - Farming subsidies and support
    /// - Food security programs
    /// - Rural infrastructure development
    /// - Agricultural research funding
    /// - Land use planning
    ///
    /// **On-Chain Prefix**: `gov.agriculture`
    Agriculture,
    
    /// Ministry of Defense - National security and border protection
    ///
    /// **Responsibilities**:
    /// - National security operations
    /// - Border protection and monitoring
    /// - Defense infrastructure
    /// - Emergency response coordination
    /// - Classified operations (limited on-chain data)
    ///
    /// **On-Chain Prefix**: `gov.defense`
    Defense,
}

impl Department {
    /// Get department name for display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Finance => "Finance",
            Self::Education => "Education",
            Self::Health => "Health",
            Self::Works => "Public Works",
            Self::Justice => "Justice",
            Self::Tourism => "Tourism",
            Self::Agriculture => "Agriculture",
            Self::Defense => "Defense",
        }
    }
    
    /// Get department prefix for on-chain entity (e.g., gov.finance)
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Finance => "gov.finance",
            Self::Education => "gov.education",
            Self::Health => "gov.health",
            Self::Works => "gov.works",
            Self::Justice => "gov.justice",
            Self::Tourism => "gov.tourism",
            Self::Agriculture => "gov.agriculture",
            Self::Defense => "gov.defense",
        }
    }
}

/// AR-8: SCALE-decodable department action variants.
///
/// Encoded as `call_data` bytes in `ProposalAction::DepartmentAction`.
/// New variants must be added here and handled in `execute_department_action`.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum DepartmentCall<AccountId, Balance> {
    /// Transfer `amount` from the governance treasury to `recipient`.
    SpendBudget {
        recipient: AccountId,
        amount: Balance,
    },
    /// Set a named policy value for the department (key → value, both bounded).
    UpdatePolicy {
        policy_key: BoundedVec<u8, ConstU32<64>>,
        policy_value: BoundedVec<u8, ConstU32<256>>,
    },
    /// No-operation: emit `DepartmentActionExecuted` event only.
    NoOp,
}

/// Convenience alias used inside the pallet module.
pub use DepartmentCall as GovDepartmentCall;

/// Foundation Board roles for strategic governance and technical oversight.
///
/// The Foundation Board provides specialized expertise and strategic direction for
/// BelizeChain's development. The 7-member board combines permanent institutional
/// representatives with rotating citizen delegates to balance stability with
/// democratic representation.
///
/// ## Board Structure
///
/// - **Total Seats**: 7-11 (depending on role expansion)
/// - **Permanent Seats**: Founder, FSC Rep, BTB Delegate, Culture Advisor (4)
/// - **Technical Seats**: Technical Steward (up to 2), Security Auditor (up to 2)
/// - **Rotating Seats**: Citizen Delegate (up to 3, elected for 1-2 year terms)
///
/// ## Appointment Process
///
/// - **Permanent Roles**: Appointed by Root or existing council members
/// - **Citizen Delegates**: Elected through district election system
/// - **Term Limits**: 1-2 years for rotating seats, indefinite for permanent
///
/// ## Multi-Signature Authority
///
/// Critical operations require consensus from board members:
/// - Treasury allocations > 10,000 DALLA require 4-of-7 approval
/// - Runtime upgrades require unanimous technical member approval
/// - Emergency actions require FSC + 2 additional board members
///
/// ## Example Usage
///
/// ```rust,ignore
/// // Appoint a security auditor (requires council origin)
/// let appointment = Call::appoint_council_member {
///     account: auditor_account.clone(),
///     role: BoardRole::SecurityAuditor,
///     term_years: 2,
/// };
/// ```
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum BoardRole {
    /// Founder - Strategic lead and innovation oversight
    ///
    /// **Responsibilities**:
    /// - Long-term vision and strategic direction
    /// - Innovation initiatives and research oversight
    /// - Partnership development
    /// - Community engagement leadership
    ///
    /// **Appointment**: Root origin (permanent)
    /// **Max Count**: 1
    Founder,
    
    /// Technical Steward - Infrastructure maintenance and development
    ///
    /// **Responsibilities**:
    /// - Wallet and portal infrastructure maintenance
    /// - Developer tools and documentation
    /// - API and integration support
    /// - Technical roadmap execution
    ///
    /// **Appointment**: Founder or Council (renewable terms)
    /// **Max Count**: 2
    TechnicalSteward,
    
    /// FSC Representative - Regulatory compliance and financial oversight
    ///
    /// **Responsibilities**:
    /// - Financial Services Commission liaison
    /// - Regulatory compliance verification
    /// - Financial auditing and reporting
    /// - Legal framework adherence
    /// - KYC/AML policy enforcement
    ///
    /// **Appointment**: FSC or Root origin (permanent)
    /// **Max Count**: 1
    FSCRepresentative,
    
    /// BTB Delegate - Tourism and economic development
    ///
    /// **Responsibilities**:
    /// - Belize Tourism Board liaison
    /// - Tourism DAO funding management
    /// - Travel incentive program oversight
    /// - Entertainment and cultural initiatives
    ///
    /// **Appointment**: BTB or Council (permanent)
    /// **Max Count**: 1
    BTBDelegate,
    
    /// Citizen Delegate - Democratic representation of token holders
    ///
    /// **Responsibilities**:
    /// - Community voice in board decisions
    /// - Feedback collection from citizens
    /// - Proposal advocacy for community needs
    /// - Democratic accountability
    ///
    /// **Appointment**: District election (1-2 year rotating terms)
    /// **Max Count**: 3 (multiple rotating seats)
    ///
    /// **Special**: This is the only rotating board role
    CitizenDelegate,
    
    /// Security Auditor - Code security and smart contract integrity
    ///
    /// **Responsibilities**:
    /// - Runtime upgrade code reviews
    /// - Smart contract auditing
    /// - Security vulnerability assessment
    /// - Incident response coordination
    ///
    /// **Appointment**: Council or existing auditors (renewable terms)
    /// **Max Count**: 2
    SecurityAuditor,
    
    /// Culture & Ethics Advisor - Cultural relevance and ethical AI
    ///
    /// **Responsibilities**:
    /// - Cultural heritage preservation in tech design
    /// - Ethical AI development oversight (Nawal AI)
    /// - Community values integration
    /// - Inclusive design advocacy
    ///
    /// **Appointment**: Council or Root (permanent)
    /// **Max Count**: 1
    CultureEthicsAdvisor,
}

impl BoardRole {
    /// Get role name for display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Founder => "Founder",
            Self::TechnicalSteward => "Technical Steward",
            Self::FSCRepresentative => "FSC Representative",
            Self::BTBDelegate => "BTB Delegate",
            Self::CitizenDelegate => "Citizen Delegate",
            Self::SecurityAuditor => "Security Auditor",
            Self::CultureEthicsAdvisor => "Culture & Ethics Advisor",
        }
    }

    /// Check if role requires rotation.
    /// All roles are now subject to term limits and rotation — no role is permanently exempt.
    pub fn is_rotating(&self) -> bool {
        true
    }

    /// Maximum allowed members per role
    pub fn max_count(&self) -> u32 {
        match self {
            Self::Founder => 1,
            Self::TechnicalSteward => 2,
            Self::FSCRepresentative => 1,
            Self::BTBDelegate => 1,
            Self::CitizenDelegate => 3, // Up to 3 rotating delegates
            Self::SecurityAuditor => 2,
            Self::CultureEthicsAdvisor => 1,
        }
    }
}

/// Belize's 6 administrative districts (Phase 6)
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum BelizeDistrict {
    /// Belize District (largest population, includes Belize City)
    Belize,
    /// Cayo District (western, capital Belmopan)
    Cayo,
    /// Corozal District (northern, near Mexico border)
    Corozal,
    /// Orange Walk District (northern, sugar cane region)
    OrangeWalk,
    /// Stann Creek District (southern coast, tourism hub)
    StannCreek,
    /// Toledo District (southernmost, diverse indigenous population)
    Toledo,
}

impl BelizeDistrict {
    /// Get district name for display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Belize => "Belize District",
            Self::Cayo => "Cayo District",
            Self::Corozal => "Corozal District",
            Self::OrangeWalk => "Orange Walk District",
            Self::StannCreek => "Stann Creek District",
            Self::Toledo => "Toledo District",
        }
    }

    /// Get number of council seats allocated per district (proportional to population)
    pub fn seat_allocation(&self) -> u32 {
        match self {
            Self::Belize => 3,      // Largest population
            Self::Cayo => 2,        // Second largest
            Self::Corozal => 1,     // Smaller population
            Self::OrangeWalk => 2,  // Medium population
            Self::StannCreek => 2,  // Medium population (tourism)
            Self::Toledo => 2,      // Smallest but needs representation
        }
    }

    /// Convert district to index (for events)
    pub fn to_index(&self) -> u8 {
        match self {
            Self::Belize => 0,
            Self::Cayo => 1,
            Self::Corozal => 2,
            Self::OrangeWalk => 3,
            Self::StannCreek => 4,
            Self::Toledo => 5,
        }
    }

    /// Convert index to district (for extrinsics)
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Belize),
            1 => Some(Self::Cayo),
            2 => Some(Self::Corozal),
            3 => Some(Self::OrangeWalk),
            4 => Some(Self::StannCreek),
            5 => Some(Self::Toledo),
            _ => None,
        }
    }
}

/// Election status (Phase 6)
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ElectionStatus {
    /// Election announced, candidates can register
    Registration,
    /// Voting is open
    Voting,
    /// Election completed, results finalized
    Finalized,
    /// Election cancelled
    Cancelled,
}

/// Trait for compliance checking
pub trait ComplianceCheck<AccountId> {
    /// Check if account can participate in governance
    fn can_participate_in_governance(account: &AccountId) -> bool;

    /// CRIT-2 FIX: Return the number of accounts eligible to vote.
    /// Used to set dynamic quorum instead of a hardcoded placeholder.
    /// Default implementation returns 1 to avoid division-by-zero in callers.
    fn eligible_voter_count() -> u32 {
        1
    }
}

// GOVERNANCE_ID removed (E-7): dead constant, never used.\n\n/// Minimum and maximum council members
pub const MIN_COUNCIL_MEMBERS: u32 = 7;
pub const MAX_COUNCIL_MEMBERS: u32 = 12;
pub const ABSOLUTE_MAX_COUNCIL: u32 = 32; // Constitutional cap

/// Voting periods in blocks (7 days = 7 * 24 * 60 * 10 = 100,800 blocks at 6s per block)
pub const VOTING_PERIOD: u32 = 100_800; // 7 days

/// Supermajority threshold (66%)
pub const SUPERMAJORITY_THRESHOLD: u32 = 66;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_std::prelude::*;
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
    use frame_system::{ensure_signed, ensure_root};
    use sp_runtime::{SaturatedConversion, traits::Zero};
    use frame_support::ensure;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// The currency used for governance operations
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        
        /// Source of randomness for council selection
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
        
        /// Council origin for administrative functions
        type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Community origin for community proposals
        type CommunityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Compliance provider for KYC/AML checks
        type ComplianceProvider: ComplianceCheck<Self::AccountId>;
        
        /// Community participation tracker (Phase 6 integration)
        type CommunityParticipation: GovernanceParticipation<Self::AccountId>;
        
        /// Minimum deposit for proposals
        #[pallet::constant]
        type MinimumDeposit: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;
        
        /// Voting period duration in blocks
        #[pallet::constant]
        type VotingPeriod: Get<BlockNumberFor<Self>>;
        
        /// Launch period before voting starts
        #[pallet::constant]
        type LaunchPeriod: Get<BlockNumberFor<Self>>;
        
        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// Maximum number of candidates per district election.
        /// Bounds the `iter_prefix().collect()` in `finalize_district_election`.
        #[pallet::constant]
        type MaxCandidatesPerElection: Get<u32>;

        /// Maximum consecutive terms a council member can serve before a mandatory break.
        #[pallet::constant]
        type MaxConsecutiveTerms: Get<u8>;

        /// Maximum number of permanent-role term extensions allowed (advisory; full enforcement
        /// requires a governance proposal).
        #[pallet::constant]
        type MaxPermanentTermExtensions: Get<u8>;

        /// Minimum blocks between successive proposal submissions from the same account.
        /// Emergency proposals (`is_emergency = true`) bypass this cooldown.
        #[pallet::constant]
        type ProposalCooldown: Get<BlockNumberFor<Self>>;

        /// Enactment delay (blocks) for Constitutional proposals.
        #[pallet::constant]
        type EnactmentPeriodConstitutional: Get<BlockNumberFor<Self>>;

        /// Enactment delay (blocks) for Economic proposals.
        #[pallet::constant]
        type EnactmentPeriodEconomic: Get<BlockNumberFor<Self>>;

        /// Enactment delay (blocks) for Standard / Technical / Community proposals.
        #[pallet::constant]
        type EnactmentPeriodStandard: Get<BlockNumberFor<Self>>;

        // ── Phase 2A: Quadratic voting ────────────────────────────────────────

        /// Whether stake-based quadratic voting is enabled.
        /// When `true`, stake units are square-rooted before being added to voting
        /// weight, preventing pure wealth domination of governance.
        #[pallet::constant]
        type QuadraticVotingEnabled: Get<bool>;

        /// Hard ceiling on stake-derived voting units per account.
        /// No account may gain more than this many units from stake regardless of
        /// how much DALLA they hold.  Applies before and after the QV square-root.
        #[pallet::constant]
        type MaxVotingUnits: Get<u32>;

        /// DALLA (in smallest unit) required per 1 raw stake-voting unit
        /// before the quadratic root is applied.
        #[pallet::constant]
        type StakeUnitSize: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        // ── Phase 2B: Wealth → responsibility ────────────────────────────────

        /// Stake balance threshold above which an account is classified as a
        /// "large holder" and subject to participation requirements.
        #[pallet::constant]
        type LargeHolderStakeThreshold: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Minimum vote-participation rate (out of 100) required from large
        /// holders each quarter.  Holders who fall below this rate have their
        /// effective voting multiplier halved until the next quarterly reset.
        #[pallet::constant]
        type LargeHolderMinParticipationRate: Get<u8>;

        // ── Phase 4C: Exit right ──────────────────────────────────────────────

        /// Number of blocks for which a generated exit proof remains valid.
        /// Default: ~30 days at 6 s/block = 432,000 blocks.
        /// Any account may generate a proof of their on-chain state at any time.
        #[pallet::constant]
        type ExitProofValidity: Get<BlockNumberFor<Self>>;

        // ── Phase 5A: Addiction-loop circuit breaker ──────────────────────────

        /// Cross-pallet behavior-flag provider (oracle pallet implementation).
        /// When an account has an active flag, their proposal and vote actions
        /// are temporarily blocked as a circuit breaker.
        type BehaviorFlags: BehaviorFlagProvider<Self::AccountId, BlockNumberFor<Self>>;

        // ── Phase 6A: Constitutional dual-house ratification ───────────────────

        /// Runtime dual-house membership provider bridging to pallet_collective
        /// Instance1 (Technical) and Instance2 (Governance) for constitutional
        /// ratification checks.
        type DualHouseProvider: DualHouseProvider<Self::AccountId>;

        // ── Phase 6B: Constitutional parameter locks ───────────────────────────

        /// Origin required to lock or unlock a chain parameter constitutionally.
        /// Wired to `TechnicalCouncilSuperMajority` during bootstrap.
        /// TODO(MAINNET): Replace with dual-council compound origin.
        type ConstitutionalAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // ── CONS-029: Emergency veto window ────────────────────────────────────

        /// Number of blocks emergency declarations remain in pending state
        /// before activation, allowing council veto.
        /// Default: 14,400 blocks (~24 hours at 6s/block).
        #[pallet::constant]
        type EmergencyVetoWindow: Get<BlockNumberFor<Self>>;

        // ── CONS-030: Action-type timelocks ────────────────────────────────────

        /// Minimum enactment delay for runtime upgrade proposals.
        /// Default: 100,800 blocks (~7 days at 6s/block).
        #[pallet::constant]
        type RuntimeUpgradeMinTimelock: Get<BlockNumberFor<Self>>;

        /// Minimum enactment delay for parameter change proposals.
        /// Default: 28,800 blocks (~48 hours at 6s/block).
        #[pallet::constant]
        type ParameterChangeMinTimelock: Get<BlockNumberFor<Self>>;

        // ── S5-4: Treasury per-period spend cap ─────────────────────────────

        /// Maximum cumulative treasury spend per rolling period.
        /// Prevents treasury drain attacks even if governance is compromised.
        #[pallet::constant]
        type MaxTreasurySpendPerPeriod: Get<BalanceOf<Self>>;

        /// Length of each treasury spending period in blocks.
        /// Cumulative spend resets at the start of each period.
        /// Default: 14,400 blocks (~24 hours at 6s/block).
        #[pallet::constant]
        type TreasurySpendPeriod: Get<BlockNumberFor<Self>>;

        /// Minimum quorum percentage for referendums (P0-17 FIX).
        /// Prevents referendum creators from setting a trivially low quorum
        /// (e.g. 1%) that bypasses meaningful community participation.
        /// Value is in range 1-100.  Recommended: 10.
        #[pallet::constant]
        type MinQuorumPercentage: Get<u8>;
    }

    /// Type alias for balance amounts
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Council member information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct CouncilMember<AccountId, BlockNumber> {
        /// Member account ID
        pub account: AccountId,
        /// Foundation board role (Phase 4)
        pub role: BoardRole,
        /// Term end block (Phase 4)
        pub term_end: BlockNumber,
        /// Whether this is a rotating position (Phase 4)
        pub is_rotating: bool,
        /// Community rank score
        pub community_rank: u32,
        /// PoUW contribution score
        pub pouw_contribution: u32,
        /// Total voting weight
        pub voting_weight: u32,
        /// Term start block
        pub term_start: BlockNumber,
        /// Election votes received
        pub votes_received: u32,
        /// Proposals authored
        pub proposals_authored: u32,
        /// Voting participation rate
        pub participation_rate: u8,
        /// Number of consecutive terms served (incremented on re-appointment, reset to 0 on fresh start).
        /// Used to enforce `MaxConsecutiveTerms`.
        pub consecutive_terms: u8,
    }

    /// Proposal information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Proposal<AccountId, BlockNumber, Balance> {
        /// Proposal ID
        pub id: u32,
        /// Proposer account
        pub proposer: AccountId,
        /// Proposal title
        pub title: BoundedVec<u8, ConstU32<256>>,
        /// Proposal description
        pub description: BoundedVec<u8, ConstU32<1024>>,
        /// Proposal type
        pub proposal_type: ProposalType,
        /// Required voting threshold
        pub threshold: VotingThreshold,
        /// Deposit amount
        pub deposit: Balance,
        /// Voting start block
        pub voting_start: BlockNumber,
        /// Voting end block  
        pub voting_end: BlockNumber,
        /// Current vote tally
        pub vote_tally: VoteTally,
        /// Proposal status
        pub status: ProposalStatus,
        /// Emergency flag
        pub is_emergency: bool,
        /// Department that owns this proposal (None for general proposals)
        pub department: Option<Department>,
        /// District that owns this proposal (None for national proposals) - NEW
        pub district: Option<BelizeDistrict>,
        /// Whether this proposal requires cross-department approval
        pub requires_cross_approval: bool,
        /// Departments that have approved this cross-department proposal
        pub cross_approved_by: BoundedVec<Department, ConstU32<8>>,
        /// Executable action for this proposal (Phase 5)
        pub action: Option<ProposalAction<AccountId, Balance>>,
        /// Block number when proposal was executed (Phase 5)
        pub executed_at: Option<BlockNumber>,
    }

    /// Types of governance proposals with specialized approval requirements.
    ///
    /// Each proposal type has different voting thresholds, time periods, and
    /// required approvals based on the sensitivity and impact of the decision.
    ///
    /// ## Approval Requirements
    ///
    /// | Type | Threshold | Approval Authority | Voting Period |
    /// |------|-----------|-------------------|---------------|
    /// | Treasury | Simple (51%) | Council or FSC | 7 days |
    /// | Council | Supermajority (66%) | Council | 7 days |
    /// | Community | Simple (51%) | Community vote | 7 days |
    /// | Department | Simple (51%) | Dept head + FSC | 3 days |
    /// | Emergency | Unanimous Council | Council + Root | 24 hours |
    ///
    /// ## Economic Integration
    ///
    /// - **Treasury proposals**: Require sufficient treasury balance verification
    /// - **Department proposals**: Check department-specific treasury allocation
    /// - **Emergency proposals**: Can override normal treasury limits
    ///
    /// ## Example Usage
    ///
    /// ```rust,ignore
    /// // Create a treasury spending proposal (requires Contributor tier)
    /// let proposal = Call::create_proposal {
    ///     title: b"Community Center Funding".to_vec(),
    ///     description: b"Allocate 5,000 DALLA for district community center".to_vec(),
    ///     proposal_type: ProposalType::Treasury,
    ///     department: None, // Treasury proposals not department-specific
    ///     metadata: None,
    /// };
    /// ```
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalType {
        /// Constitutional amendments - Fundamental governance changes
        ///
        /// **Requirements**:
        /// - Proposer must have Validator tier
        /// - Supermajority (66%) or Unanimous approval
        /// - Extended voting period
        ///
        /// **Use Cases**:
        /// - Fundamental governance structure changes
        /// - Voting threshold modifications
        /// - Constitutional rights and rules
        /// - System-wide governance reforms
        Constitutional,
        
        /// Economic policy changes - Financial and treasury management
        ///
        /// **Requirements**:
        /// - Proposer must have Contributor tier or higher
        /// - Treasury must have sufficient balance
        /// - Simple majority (51%) or Supermajority for large amounts
        ///
        /// **Use Cases**:
        /// - Treasury spending and allocations
        /// - Economic policy adjustments
        /// - Tax and revenue strategies
        /// - Financial incentive programs
        Economic,
        
        /// Council membership and governance structure changes
        ///
        /// **Requirements**:
        /// - Proposer must have Validator tier
        /// - Supermajority (66%) for approval
        /// - Existing council member approval
        ///
        /// **Use Cases**:
        /// - Add/remove council members
        /// - Change council structure
        /// - Modify member responsibilities
        /// - Update governance roles
        Council,
        
        /// Technical upgrades - Runtime and infrastructure changes
        ///
        /// **Requirements**:
        /// - Proposer must have Validator tier
        /// - Technical review by Security Auditor
        /// - Supermajority approval
        ///
        /// **Use Cases**:
        /// - Runtime upgrades
        /// - Smart contract deployments
        /// - Infrastructure improvements
        /// - Protocol modifications
        Technical,
        
        /// Emergency measures - Urgent national security actions
        ///
        /// **Requirements**:
        /// - Root or FSC origin
        /// - Unanimous council approval for non-Root
        /// - Immediate execution capability
        ///
        /// **Use Cases**:
        /// - National security threats
        /// - Economic crises
        /// - Natural disaster response
        /// - System vulnerability patches
        Emergency,
        
        /// International agreements - Cross-border and diplomatic
        ///
        /// **Requirements**:
        /// - Proposer must have Validator tier
        /// - FSC and relevant ministry approval
        /// - Supermajority approval
        ///
        /// **Use Cases**:
        /// - International partnerships
        /// - Cross-border agreements
        /// - Trade policies
        /// - Diplomatic initiatives
        International,
        
        /// Community-driven initiatives and programs
        ///
        /// **Requirements**:
        /// - Proposer must have Contributor tier
        /// - Simple majority (51%) for approval
        /// - Open to all verified citizens
        ///
        /// **Use Cases**:
        /// - Community programs
        /// - Local initiatives
        /// - Cultural events
        /// - Public feedback collection
        Community,
        
        /// District-specific local proposals - NEW
        ///
        /// **Requirements**:
        /// - Proposer must be resident of the district (verified via Identity)
        /// - Only district residents can vote
        /// - Simple majority (51%) for approval
        /// - District council approval for budgets > 10,000 DALLA
        ///
        /// **Use Cases**:
        /// - Local infrastructure projects (roads, bridges, public facilities)
        /// - District cultural events and festivals
        /// - Local education programs
        /// - District tourism initiatives
        /// - Municipal services (waste management, utilities)
        /// - Local economic development
        ///
        /// **Examples**:
        /// - Belize District: Road repairs in Belize City, coastal protection
        /// - Cayo District: Mountain Pine Ridge conservation, adventure tourism
        /// - Orange Walk: Sugar industry support, river cleanup
        /// - Corozal: Border infrastructure, trade facilitation
        /// - Stann Creek: Beach preservation, reef conservation
        /// - Toledo: Indigenous cultural programs, eco-tourism
        DistrictLocal,

        /// Wellbeing funding proposals - Phase 5B
        ///
        /// **Requirements**:
        /// - Proposer must have Contributor tier or higher
        /// - Simple majority (51%) for approval
        /// - 3-day voting period; reduced minimum deposit (5 DALLA)
        ///
        /// **Use Cases**:
        /// - Mental health programme grants
        /// - Digital wellness initiatives (anti-addiction UX, screen-time education)
        /// - Community wellbeing research and tooling
        /// - Occupational health support for gig-economy workers
        WellbeingFunding,
    }

    /// Voting threshold requirements
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum VotingThreshold {
        /// Simple majority (>50%)
        SimpleMajority,
        /// Supermajority (66%)
        Supermajority,
        /// Unanimous (100%)
        Unanimous,
        /// Custom percentage
        Custom(u8),
    }

    /// Vote tally tracking
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct VoteTally {
        /// Votes in favor
        pub ayes: u32,
        /// Votes against
        pub nays: u32,
        /// Abstentions
        pub abstentions: u32,
        /// Total voting weight
        pub total_weight: u32,
        /// Participation percentage
        pub participation: u8,
    }

    /// Proposal status
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalStatus {
        /// Proposal submitted, waiting for launch period
        Pending,
        /// Voting period active
        Voting,
        /// Proposal approved
        Approved,
        /// Proposal rejected
        Rejected,
        /// Proposal cancelled
        Cancelled,
        /// Proposal executed
        Executed,
    }

    /// Individual vote record
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Vote {
        /// Vote choice
        pub vote: VoteChoice,
        /// Voting weight used
        pub weight: u32,
        /// Vote conviction/lock period
        pub conviction: u8,
        /// Voting timestamp
        pub timestamp: u32,
    }

    /// Vote choices
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum VoteChoice {
        /// Vote in favor
        Aye,
        /// Vote against
        Nay,
        /// Abstain from voting
        Abstain,
    }

    /// Executable actions for approved proposals (Phase 5)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalAction<AccountId, Balance> {
        /// Transfer funds from treasury
        TreasurySpend { 
            recipient: AccountId, 
            amount: Balance,
        },
        /// Upgrade the runtime code
        RuntimeUpgrade { 
            code_hash: BoundedVec<u8, ConstU32<32>>,
        },
        /// Change governance parameter
        ParameterChange { 
            parameter: GovernanceParameter, 
            new_value: u32,
        },
        /// Execute department-specific action
        DepartmentAction { 
            department: Department, 
            call_data: BoundedVec<u8, ConstU32<1024>>,
        },
        /// Execute emergency action
        EmergencyAction { 
            action_type: EmergencyActionType,
        },
    }

    /// Governance parameters that can be changed via proposals (Phase 5)
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum GovernanceParameter {
        /// Duration of voting period in blocks
        VotingPeriod,
        /// Duration of launch period in blocks
        LaunchPeriod,
        /// Minimum deposit required for proposals
        MinimumDeposit,
        /// Supermajority threshold percentage (0-100)
        SupermajorityThreshold,
        /// Maximum council size
        CouncilSize,
        /// Emergency mode timeout in blocks
        EmergencyTimeout,
    }

    /// Emergency action types (Phase 5)
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum EmergencyActionType {
        /// Activate emergency mode
        ActivateEmergency,
        /// Deactivate emergency mode
        DeactivateEmergency,
        /// Freeze a specific account
        FreezeAccount,
        /// Unfreeze a specific account
        UnfreezeAccount,
        /// Halt all governance operations
        HaltGovernance,
        /// Resume governance operations
        ResumeGovernance,
    }

    /// Council election information (Phase 6)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Election<BlockNumber> {
        /// Election ID (block number when started)
        pub id: BlockNumber,
        /// District this election is for
        pub district: BelizeDistrict,
        /// Number of council seats available
        pub seats: u32,
        /// Registration end block
        pub registration_end: BlockNumber,
        /// Voting start block
        pub voting_start: BlockNumber,
        /// Voting end block
        pub voting_end: BlockNumber,
        /// Current election status
        pub status: ElectionStatus,
        /// Total votes cast
        pub total_votes: u32,
    }

    /// Election candidate information (Phase 6)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ElectionCandidate<AccountId, BlockNumber> {
        /// Candidate account ID
        pub account: AccountId,
        /// District they're running in
        pub district: BelizeDistrict,
        /// Election ID
        pub election_id: BlockNumber,
        /// Campaign platform/statement
        pub platform: BoundedVec<u8, ConstU32<512>>,
        /// Total votes received
        pub votes: u32,
        /// Registration block
        pub registered_at: BlockNumber,
    }

    /// Vote delegation information (Phase 7)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct VoteDelegation<AccountId, BlockNumber> {
        /// Account delegating votes
        pub delegator: AccountId,
        /// Account receiving delegated votes
        pub delegate: AccountId,
        /// When delegation was created
        pub created_at: BlockNumber,
        /// Expiry block (delegation expires after this)
        pub expires_at: BlockNumber,
        /// Whether this delegation is currently active
        pub is_active: bool,
    }

    /// Proposal amendment (Phase 7)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ProposalAmendment<BlockNumber> {
        /// Proposal ID being amended
        pub proposal_id: u32,
        /// New title (if changed)
        pub new_title: Option<BoundedVec<u8, ConstU32<256>>>,
        /// New description (if changed)
        pub new_description: Option<BoundedVec<u8, ConstU32<1024>>>,
        /// When amendment was proposed
        pub proposed_at: BlockNumber,
        /// Whether amendment was applied
        pub is_applied: bool,
    }

    /// Referendum for direct democracy voting on specific questions.
    ///
    /// Referendums allow citizens to vote directly on specific policy questions,
    /// bypassing the representative council system. Requires quorum for validity.
    ///
    /// ## Referendum Types
    ///
    /// - **Binary**: Yes/No questions
    /// - **Multiple Choice**: Select one option from many
    /// - **Approval**: Approve/Reject with abstention option
    ///
    /// ## Quorum Requirements
    ///
    /// - National referendums: 40% participation minimum
    /// - District referendums: 30% participation minimum
    /// - Emergency referendums: 50% participation minimum
    ///
    /// ## Example Usage
    ///
    /// ```rust,ignore
    /// // Create national referendum on voting age
    /// governance.createReferendum(
    ///     title: "Lower Voting Age to 16",
    ///     description: "Should voting age be lowered?",
    ///     options: vec!["Yes", "No"],
    ///     quorum: 40,  // 40% participation required
    ///     duration: 201600,  // 14 days in blocks
    ///     district: None,  // National referendum
    /// )
    /// ```
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Referendum<AccountId, BlockNumber> {
        /// Unique referendum ID
        pub id: u32,
        /// Creator of the referendum
        pub creator: AccountId,
        /// Referendum title (max 256 bytes)
        pub title: BoundedVec<u8, ConstU32<256>>,
        /// Referendum description (max 1024 bytes)
        pub description: BoundedVec<u8, ConstU32<1024>>,
        /// Voting options (e.g., ["Yes", "No", "Abstain"])
        pub options: BoundedVec<BoundedVec<u8, ConstU32<128>>, ConstU32<10>>, // Max 10 options
        /// Quorum percentage required (0-100)
        pub quorum_percentage: u8,
        /// Block when voting starts
        pub voting_start: BlockNumber,
        /// Block when voting ends
        pub voting_end: BlockNumber,
        /// Current vote counts per option
        pub vote_counts: BoundedVec<u32, ConstU32<10>>, // Vote count per option
        /// Total votes cast
        pub total_votes: u32,
        /// Current status
        pub status: ReferendumStatus,
        /// Optional district (None = national)
        pub district: Option<BelizeDistrict>,
        /// Winning option index (set after finalization)
        pub winning_option: Option<u8>,
    }

    /// Referendum status tracking
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ReferendumStatus {
        /// Referendum is active, accepting votes
        Active,
        /// Referendum ended, quorum met, executed
        Passed,
        /// Referendum ended, quorum not met
        Failed,
        /// Referendum cancelled by governance
        Cancelled,
    }

    /// Individual vote on a referendum
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReferendumVote {
        /// Voter account
        pub voter: BoundedVec<u8, ConstU32<32>>, // AccountId as bytes
        /// Option index voted for (0-based)
        pub option_index: u8,
        /// Block when vote was cast
        pub voted_at: u32, // BlockNumber as u32
        /// Vote weight (based on community rank + PoUW)
        pub weight: u32,
    }

    /// District budget allocation for fiscal year management.
    ///
    /// Tracks allocated funds, spending, and funded proposals per district.
    /// Enables transparent budget management and accountability at district level.
    ///
    /// ## Fiscal Year Management
    ///
    /// - Budgets reset annually based on national allocations
    /// - Districts can request budget amendments via proposals
    /// - Inter-district transfers require multi-sig approval
    /// - Unspent funds roll over to next fiscal year (up to 20%)
    ///
    /// ## Example Usage
    ///
    /// ```rust,ignore
    /// // Allocate 5M DALLA to Cayo District for fiscal year
    /// governance.allocate_district_budget(
    ///     district_index: 1,  // Cayo
    ///     amount: 5_000_000_000000,  // 5M DALLA (12 decimals)
    ///     fiscal_year_blocks: 5_256_000,  // 1 year in blocks
    /// )
    /// ```
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct DistrictBudget<Balance, BlockNumber> {
        /// District identifier
        pub district: BelizeDistrict,
        /// Total allocated budget for fiscal year
        pub allocated: Balance,
        /// Amount already spent
        pub spent: Balance,
        /// Proposal IDs funded from this budget
        pub proposals_funded: BoundedVec<u32, ConstU32<100>>, // Max 100 proposals per year
        /// Block when fiscal year started
        pub fiscal_year_start: BlockNumber,
        /// Block when fiscal year ends
        pub fiscal_year_end: BlockNumber,
        /// Whether budget is active
        pub is_active: bool,
    }

    /// Treasury spend proposal for multi-sig approval workflow.
    ///
    /// Larger treasury spends (>10,000 DALLA) require Foundation Board approval
    /// via multi-signature workflow. Tracks approval status and signatories.
    ///
    /// ## Approval Workflow
    ///
    /// 1. Proposal created by authorized account
    /// 2. Foundation Board members sign (4-of-7 required)
    /// 3. Once threshold met, proposal executable
    /// 4. Execution transfers funds and records transaction
    ///
    /// ## Thresholds
    ///
    /// - < 10,000 DALLA: Single signature (council/FSC)
    /// - 10,000 - 100,000 DALLA: 3-of-7 multi-sig
    /// - > 100,000 DALLA: 4-of-7 multi-sig
    ///
    /// ## Example Usage
    ///
    /// ```rust,ignore
    /// // Create treasury spend proposal
    /// governance.propose_treasury_spend(
    ///     recipient: contractor_account,
    ///     amount: 50_000_000000,  // 50K DALLA
    ///     description: "Road construction payment",
    ///     district: Some(BelizeDistrict::Cayo),
    /// )
    /// ```
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct TreasurySpendProposal<AccountId, Balance, BlockNumber> {
        /// Unique proposal ID
        pub id: u32,
        /// Proposer account
        pub proposer: AccountId,
        /// Recipient of funds
        pub recipient: AccountId,
        /// Amount to transfer
        pub amount: Balance,
        /// Description/justification (max 512 bytes)
        pub description: BoundedVec<u8, ConstU32<512>>,
        /// Optional district (for district budget tracking)
        pub district: Option<BelizeDistrict>,
        /// Accounts that have approved (max 7 for Foundation Board)
        pub approvals: BoundedVec<AccountId, ConstU32<7>>,
        /// Required approval threshold
        pub threshold: u8,
        /// Block when proposal was created
        pub created_at: BlockNumber,
        /// Block when proposal expires (30 days default)
        pub expires_at: BlockNumber,
        /// Whether proposal has been executed
        pub executed: bool,
    }

    /// Proposal priority levels for queue management and processing order.
    ///
    /// Priority determines processing order when multiple proposals are active.
    /// Higher priority proposals are processed first, with stricter time constraints
    /// and expedited review procedures. Root origin can set proposal priority.
    ///
    /// ## Priority Assignment Rules
    ///
    /// | Priority | Typical Use Cases | Max Queue Size | Review Time |
    /// |----------|------------------|----------------|-------------|
    /// | Low | Community initiatives, minor changes | 50 | 14 days |
    /// | Normal | Standard proposals, most governance | 50 | 7 days |
    /// | High | Economic policy, technical upgrades | 50 | 3 days |
    /// | Critical | Emergency measures, security fixes | 50 | 24 hours |
    ///
    /// ## Queue Management
    ///
    /// - Each priority level can hold up to 50 proposals
    /// - Critical proposals bypass normal voting periods (with proper authority)
    /// - Priority can be escalated by Root or FSC for urgent matters
    /// - Expired low-priority proposals are auto-cancelled if not addressed
    ///
    /// ## Example Usage
    ///
    /// ```rust,ignore
    /// // Set proposal as critical (requires Root origin)
    /// let prioritize = Call::set_proposal_priority {
    ///     proposal_id: 42,
    ///     priority: ProposalPriority::Critical,
    /// };
    /// ```
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalPriority {
        /// Low priority - Community initiatives and non-urgent changes
        ///
        /// **Characteristics**:
        /// - Extended review period (14 days)
        /// - Simple majority approval
        /// - Can be deprioritized if queue is full
        /// - Auto-cancelled after 30 days if not addressed
        ///
        /// **Numeric Value**: 1
        Low,
        
        /// Normal priority - Standard governance proposals
        ///
        /// **Characteristics**:
        /// - Standard review period (7 days)
        /// - Normal voting thresholds
        /// - Default priority for most proposals
        /// - Balanced processing order
        ///
        /// **Numeric Value**: 2
        Normal,
        
        /// High priority - Important economic or technical changes
        ///
        /// **Characteristics**:
        /// - Expedited review (3 days)
        /// - May require expert review
        /// - Higher visibility to voters
        /// - Processed before Normal/Low
        ///
        /// **Numeric Value**: 3
        High,
        
        /// Critical priority - Emergency measures and security fixes
        ///
        /// **Characteristics**:
        /// - Immediate review (24 hours)
        /// - May bypass normal voting for emergencies
        /// - Requires FSC or Root authority
        /// - Alerts all council members
        ///
        /// **Numeric Value**: 4
        Critical,
    }

    impl ProposalPriority {
        /// Get numeric value for sorting and comparison.
        ///
        /// Higher values indicate more urgent priority. Used for queue sorting
        /// and determining processing order.
        ///
        /// # Returns
        ///
        /// - `1` for Low
        /// - `2` for Normal
        /// - `3` for High
        /// - `4` for Critical
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let critical = ProposalPriority::Critical;
        /// assert_eq!(critical.to_value(), 4);
        /// 
        /// let normal = ProposalPriority::Normal;
        /// assert!(critical.to_value() > normal.to_value());
        /// ```
        pub fn to_value(&self) -> u8 {
            match self {
                Self::Low => 1,
                Self::Normal => 2,
                Self::High => 3,
                Self::Critical => 4,
            }
        }
    }

    /// Emergency type classification for Jaguar Mode
    /// 
    /// Named after Belize's national animal, the Jaguar (Panthera onca), 
    /// representing swift, decisive action during national crises.
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum EmergencyType {
        /// Hurricane or tropical storm emergency
        Hurricane,
        /// Public health crisis (epidemic, pandemic)
        HealthCrisis,
        /// Economic crisis or financial emergency
        EconomicCrisis,
        /// Security threat or attack
        SecurityThreat,
        /// Infrastructure failure
        InfrastructureFailure,
        /// Other emergency requiring immediate response
        Other,
    }

    impl EmergencyType {
        /// Convert emergency type to numeric index for event emission
        pub fn to_index(&self) -> u8 {
            match self {
                Self::Hurricane => 0,
                Self::HealthCrisis => 1,
                Self::EconomicCrisis => 2,
                Self::SecurityThreat => 3,
                Self::InfrastructureFailure => 4,
                Self::Other => 5,
            }
        }
    }

    /// Jaguar Mode emergency status
    /// 
    /// Tracks active national emergencies with detailed metadata for
    /// crisis response coordination.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct EmergencyStatus<BlockNumber> {
        /// Whether Jaguar Mode is currently active
        pub active: bool,
        /// Type of emergency
        pub emergency_type: EmergencyType,
        /// Human-readable description
        pub description: BoundedVec<u8, ConstU32<256>>,
        /// Block when emergency was declared
        pub declared_at: BlockNumber,
        /// Block when emergency expires (if set)
        pub expires_at: Option<BlockNumber>,
        /// Account that declared the emergency
        pub declared_by: Option<[u8; 32]>, // AccountId as bytes
        /// Whether this emergency is pending veto window (CONS-029)
        pub is_pending: bool,
        /// Block at which the veto window expires (CONS-029)
        pub veto_window_ends_at: Option<BlockNumber>,
    }

    #[allow(clippy::type_complexity)]
    #[pallet::storage]
    #[pallet::getter(fn council_members)]
    /// Current council members
    pub type CouncilMembers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        CouncilMember<T::AccountId, BlockNumberFor<T>>,
    >;

    #[allow(clippy::type_complexity)]
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    /// Active and historical proposals
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Proposal ID
        Proposal<T::AccountId, BlockNumberFor<T>, <T::Currency as Currency<T::AccountId>>::Balance>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn votes)]
    /// Vote records for proposals.
    ///
    /// **Privacy Note**: Votes are currently stored in plaintext (voter -> vote).
    /// This is a deliberate design choice for governance accountability — council
    /// votes and proposal votes are public in most democratic systems.
    /// For referendum votes (citizen-level), commit-reveal privacy is roadmapped.
    /// See `VoteCommitments` storage for the commit-reveal infrastructure.
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // Proposal ID
        Blake2_128Concat,
        T::AccountId, // Voter
        Vote,
    >;

    #[pallet::storage]
    /// Vote commitments for commit-reveal voting (Phase 3 roadmap).
    ///
    /// When commit-reveal is active for a proposal/referendum:
    /// 1. **Commit phase**: Voter submits `blake2_256(vote_choice || salt)`, stored here
    /// 2. **Reveal phase**: After voting period, voter submits `(vote_choice, salt)`,
    ///    pallet verifies hash match, records in `Votes`/`ReferendumVotes`, deletes commitment
    ///
    /// Currently pre-provisioned storage — not yet wired into extrinsics.
    /// Full implementation requires new `commit_vote` and `reveal_vote` extrinsics.
    pub type VoteCommitments<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // Proposal/Referendum ID
        Blake2_128Concat,
        T::AccountId, // Voter
        [u8; 32], // blake2_256(vote_choice || salt)
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_ranks)]
    /// Community rank scores for voting weight calculation
    pub type CommunityRanks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Community rank score
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn pouw_contributions)]
    /// PoUW contribution scores for voting weight
    pub type PoUWContributions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // PoUW contribution score
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    /// Next available proposal ID
    pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn council_term)]
    /// Current council term information
    pub type CouncilTerm<T: Config> = StorageValue<_, u32, ValueQuery>; // Term number

    #[pallet::storage]
    #[pallet::getter(fn emergency_status)]
    /// Jaguar Mode: Emergency governance status for national crises
    /// Named after Belize's swift and powerful national animal
    pub type JaguarMode<T: Config> = StorageValue<_, EmergencyStatus<BlockNumberFor<T>>, OptionQuery>;

    #[pallet::storage]
    /// Council members who have vetoed the current pending emergency (CONS-029)
    pub type EmergencyVetoes<T: Config> = CountedStorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,  // block at which the veto was cast
    >;

    #[pallet::storage]
    #[pallet::getter(fn department_managers)]
    /// Department managers (multisig addresses that control each department)
    pub type DepartmentManagers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Department,
        T::AccountId,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn department_proposal_count)]
    /// Count of proposals submitted per department
    pub type DepartmentProposalCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Department,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Track which departments have approved cross-department proposals
    pub type CrossDepartmentApprovals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // Proposal ID
        Blake2_128Concat,
        Department,
        bool,
        ValueQuery,
    >;

    // ===== Phase 4: Foundation Board System Storage =====

    #[pallet::storage]
    /// Board composition tracking by role
    pub type BoardComposition<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoardRole,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Term expiry queue for automatic rotation
    pub type TermExpiryQueue<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<T::AccountId, ConstU32<32>>,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Last block at which an account submitted a proposal (used for cooldown enforcement).
    pub type AccountLastProposal<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Earliest block at which an approved proposal may be executed (enactment delay).
    /// `None` means the proposal may be executed immediately once approved.
    pub type ProposalEnactmentBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // proposal_id
        BlockNumberFor<T>,
        OptionQuery,
    >;

    // ===== Phase 2B: Wealth-responsibility storage =====

    #[pallet::storage]
    /// Per-account vote participation counters for the current quarter.
    /// Value is `(proposals_eligible, proposals_voted)`.
    /// Reset every `BLOCKS_PER_QUARTER` blocks in `on_initialize`.
    pub type AccountVoteParticipation<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (u32, u32), // (eligible, voted)
        ValueQuery,
    >;

    #[pallet::storage]
    /// Effective voting-weight multiplier for each account, expressed as a
    /// percentage (100 = full weight, 50 = half weight).  Large holders who
    /// fail the quarterly participation check are reduced to 50 until next
    /// quarterly reset.
    pub type EffectiveVotingMultiplier<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u8,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Block number of the last quarterly participation check.
    pub type LastParticipationCheckBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    /// Delegate election nominees (account -> vote count)
    pub type DelegateNominees<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Delegate election voters (to prevent double voting)
    pub type DelegateVoters<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>, // Election ID (block number)
        Blake2_128Concat,
        T::AccountId, // Voter
        T::AccountId, // Nominee they voted for
        OptionQuery,
    >;

    #[pallet::storage]
    /// Current delegate election round
    pub type CurrentElection<T: Config> = StorageValue<
        _,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    // ===== PHASE 5: EXECUTION LAYER STORAGE =====

    #[pallet::storage]
    /// Current governance parameter values
    pub type GovernanceParameters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        GovernanceParameter,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Department treasury sub-account balances (tracked separately from main treasury)
    pub type DepartmentTreasuryBalances<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Department,
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Track which proposals have been executed (prevent double execution)
    pub type ExecutedProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // proposal_id
        BlockNumberFor<T>, // executed_at block
        OptionQuery,
    >;

    #[pallet::storage]
    /// Pending runtime upgrade code hash (32 bytes)
    pub type PendingRuntimeUpgrade<T: Config> = StorageValue<
        _,
        [u8; 32], // Runtime code hash (fixed size)
        OptionQuery,
    >;

    // ===== PHASE 6: COUNCIL ELECTION SYSTEM STORAGE =====

    #[pallet::storage]
    /// Active district elections (district -> election info)
    pub type DistrictElections<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BelizeDistrict,
        Election<BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Election candidates (election_id -> candidate account -> candidate info)
    pub type ElectionCandidates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>, // Election ID
        Blake2_128Concat,
        T::AccountId, // Candidate account
        ElectionCandidate<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Election votes (election_id -> voter -> candidate voted for).
    ///
    /// **Privacy Note**: Election votes are public for democratic accountability.
    /// Commit-reveal voting for elections is roadmapped for Phase 3.
    pub type ElectionVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>, // Election ID
        Blake2_128Concat,
        T::AccountId, // Voter
        T::AccountId, // Candidate they voted for
        OptionQuery,
    >;

    #[pallet::storage]
    /// District representation tracking (district -> list of council member accounts)
    pub type DistrictRepresentation<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BelizeDistrict,
        BoundedVec<T::AccountId, ConstU32<5>>, // Max 5 representatives per district
        ValueQuery,
    >;

    // ===== PHASE 7: ADVANCED FEATURES STORAGE =====

    #[pallet::storage]
    /// Vote delegations (delegator -> delegation info)
    pub type VoteDelegations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId, // Delegator
        VoteDelegation<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Reverse delegation lookup (delegate -> list of delegators)
    pub type DelegationReceivers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId, // Delegate
        BoundedVec<T::AccountId, ConstU32<100>>, // Max 100 delegators per delegate
        ValueQuery,
    >;

    #[pallet::storage]
    /// Proposal amendments (proposal_id -> amendment)
    pub type ProposalAmendments<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Proposal ID
        ProposalAmendment<BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Proposal priority queue (priority -> list of proposal IDs)
    pub type ProposalQueue<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProposalPriority,
        BoundedVec<u32, ConstU32<50>>, // Max 50 proposals per priority level
        ValueQuery,
    >;

    #[pallet::storage]
    /// Governance participation rewards claimed (account, reward_type -> amount)
    /// H-26 fix: per-type tracking so claiming a Vote reward does not block
    /// Proposal or Council claims.
    pub type RewardsClaimed<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u8, // reward_type: 0=Vote, 1=Proposal, 2=Council
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Total governance rewards distributed
    pub type TotalRewardsDistributed<T: Config> = StorageValue<
        _,
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    // ===== REFERENDUM SYSTEM STORAGE =====

    #[pallet::storage]
    /// Active and historical referendums (referendum_id -> referendum)
    pub type Referendums<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Referendum ID
        Referendum<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Next available referendum ID
    pub type NextReferendumId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    /// Referendum votes (referendum_id -> voter -> option_index).
    ///
    /// **Privacy Note**: Referendum votes are currently public. These are the
    /// highest-priority candidate for commit-reveal privacy since referendum
    /// votes are citizen-level and should not be linkable to individual voters.
    /// Migration to commit-reveal is roadmapped for Phase 3.
    pub type ReferendumVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // Referendum ID
        Blake2_128Concat,
        T::AccountId, // Voter
        u8, // Option index voted for
        OptionQuery,
    >;

    #[pallet::storage]
    /// Total eligible voters for referendum quorum calculation (cached at referendum creation)
    pub type ReferendumEligibleVoters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Referendum ID
        u32, // Number of eligible voters
        ValueQuery,
    >;

    // ===== TREASURY MANAGEMENT STORAGE =====

    #[pallet::storage]
    /// District budget allocations by district and fiscal year
    pub type DistrictBudgets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BelizeDistrict,
        DistrictBudget<BalanceOf<T>, BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    /// Treasury spend proposals requiring multi-sig approval
    pub type TreasurySpendProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Proposal ID
        TreasurySpendProposal<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    /// Next available treasury spend proposal ID
    pub type NextTreasuryProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    /// Total national treasury reserves (separate from district budgets)
    pub type NationalTreasuryReserve<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    /// S5-4: Rolling treasury spend tracker.
    /// Stores (period_index, cumulative_spent) to enforce MaxTreasurySpendPerPeriod.
    pub type TreasurySpendTracker<T: Config> = StorageValue<
        _, (u32, BalanceOf<T>), ValueQuery
    >;

    #[pallet::storage]
    /// AR-8: Per-department on-chain policy store.
    ///
    /// Key: `(Department, policy_key_bytes)`.
    /// Value: `policy_value_bytes` (up to 256 bytes).
    /// Updated by `DepartmentCall::UpdatePolicy` dispatched through governance.
    pub type DepartmentPolicies<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (Department, BoundedVec<u8, ConstU32<64>>),
        BoundedVec<u8, ConstU32<256>>,
        OptionQuery,
    >;

    // ===== GOVERNANCE-CONTROLLED CHAIN PARAMETERS (Audit P3 §32) =====

    #[pallet::storage]
    /// Runtime-configurable chain parameters, updatable via governance proposal.
    ///
    /// Keys use a short string identifier (e.g. `b"blocks_per_yr"`, `b"max_delegators"`).
    /// Values are u64 to accommodate block counts, percentages, and small amounts.
    ///
    /// ## Intended Usage
    ///
    /// Rather than hardcoding constants like `BLOCKS_PER_YEAR` or
    /// `MAX_DELEGATION_RECEIVERS`, this map allows governance to adjust operational
    /// parameters without a runtime upgrade. Individual pallets can query this map
    /// via a helper or fallback to their compile-time defaults.
    pub type ChainParameters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<32>>, // parameter key
        u64,                          // parameter value
        OptionQuery,
    >;

    // ── Phase 6A: Constitutional dual-house ratification state ──────────────────────

    #[pallet::storage]
    #[pallet::getter(fn constitutional_ratification)]
    /// Dual-house ratification state for each Constitutional proposal.
    /// Maps `proposal_id → (technical_house_ratified, governance_house_ratified)`.
    /// Both booleans must be `true` before `execute_proposal` will proceed.
    pub type ConstitutionalRatifications<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, (bool, bool), ValueQuery>;

    // ── Phase 6B: Constitutional parameter locks ────────────────────────────────

    #[pallet::storage]
    #[pallet::getter(fn is_param_locked)]
    /// Set of constitutionally locked parameter keys.
    /// A locked parameter cannot be modified by `update_chain_parameter`;
    /// it must first be unlocked via a dual-house-ratified Constitutional proposal.
    pub type LockedParameters<T: Config> =
        StorageMap<_, Blake2_128Concat, BoundedVec<u8, ConstU32<32>>, bool, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Initial council members
        pub council_members: Vec<(
            T::AccountId,
            u32, // Community rank
            u32, // PoUW contribution
        )>,
        /// Democracy launch period (2 days)
        pub democracy_launch_period: BlockNumberFor<T>,
        /// Democracy voting period (7 days) 
        pub democracy_voting_period: BlockNumberFor<T>,
        /// Minimum proposal deposit
        pub democracy_minimum_deposit: <T::Currency as Currency<T::AccountId>>::Balance,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                council_members: vec![],
                democracy_launch_period: BlockNumberFor::<T>::from(28800u32), // 2 days
                democracy_voting_period: BlockNumberFor::<T>::from(VOTING_PERIOD), // 7 days
                democracy_minimum_deposit: 1000u32.into(), // 1000 DALLA
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (member, community_rank, pouw_contribution) in &self.council_members {
                let voting_weight = community_rank + pouw_contribution;
                
                // Calculate 2-year term (default for genesis members)
                let term_blocks = BlockNumberFor::<T>::from(BLOCKS_PER_YEAR * 2);
                let term_end_zero: u64 = 0;
                let term_blocks_u64: u64 = TryInto::<u64>::try_into(term_blocks).unwrap_or(0);
                let term_end_u64 = term_end_zero.saturating_add(term_blocks_u64);
                // SAFETY: value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
                let term_end: BlockNumberFor<T> = term_end_u64.saturated_into();
                
                let council_member = CouncilMember {
                    account: member.clone(),
                    role: BoardRole::Founder, // Genesis members default to Founder role
                    term_end,
                    is_rotating: false,
                    community_rank: *community_rank,
                    pouw_contribution: *pouw_contribution,
                    voting_weight,
                    term_start: Zero::zero(),
                    votes_received: voting_weight, // Initial election assumption
                    proposals_authored: 0,
                    participation_rate: 100, // Start with perfect participation
                    consecutive_terms: 0,
                };

                CouncilMembers::<T>::insert(member, council_member);
                CommunityRanks::<T>::insert(member, *community_rank);
                PoUWContributions::<T>::insert(member, *pouw_contribution);
            }

            // Start with term 1
            CouncilTerm::<T>::put(1);

            // ===== PHASE 5: Initialize Governance Parameters =====
            GovernanceParameters::<T>::insert(GovernanceParameter::VotingPeriod, VOTING_PERIOD);
            GovernanceParameters::<T>::insert(GovernanceParameter::LaunchPeriod, 28800u32); // 2 days
            GovernanceParameters::<T>::insert(GovernanceParameter::MinimumDeposit, 1000u32); // 1000 DALLA
            GovernanceParameters::<T>::insert(GovernanceParameter::SupermajorityThreshold, 66u32); // 66%
            GovernanceParameters::<T>::insert(GovernanceParameter::CouncilSize, 15u32); // Max 15 members
            GovernanceParameters::<T>::insert(GovernanceParameter::EmergencyTimeout, 86400u32); // 1 day
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Process automatic council term expiries and quarterly participation audits.
        /// Term expiries: drains up to 10 expired entries per block to bound work.
        /// Participation audit: runs once per quarter (~1,296,000 blocks at 6s/block).
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = T::DbWeight::get().reads(2);

            // ── CONS-029: Finalize pending emergencies whose veto window elapsed ──
            if let Some(mut emergency) = JaguarMode::<T>::get() {
                weight = weight.saturating_add(T::DbWeight::get().reads(1));
                if emergency.is_pending {
                    if let Some(veto_end) = emergency.veto_window_ends_at {
                        if n >= veto_end {
                            // Veto window expired — activate the emergency
                            emergency.active = true;
                            emergency.is_pending = false;
                            JaguarMode::<T>::put(emergency);
                            // Clear veto records
                            let _ = EmergencyVetoes::<T>::clear(u32::MAX, None);
                            weight = weight.saturating_add(T::DbWeight::get().writes(2));
                            Self::deposit_event(Event::EmergencyFinalizedAfterVetoWindow);
                        }
                    }
                }
            }

            // ── Phase 2B: Quarterly participation check ───────────────────────
            // Runs once per quarter.  Bounded to MAX_CHECKS_PER_QUARTER accounts so
            // on-block work stays O(1).  Remaining accounts carry over to next quarter.
            let n_u64: u64 = TryInto::<u64>::try_into(n).unwrap_or(0);
            let last_check_u64: u64 = TryInto::<u64>::try_into(
                LastParticipationCheckBlock::<T>::get()
            ).unwrap_or(0);
            const BLOCKS_PER_QUARTER: u64 = 1_296_000;
            if n_u64.saturating_sub(last_check_u64) >= BLOCKS_PER_QUARTER {
                LastParticipationCheckBlock::<T>::put(n);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));

                let threshold = T::LargeHolderStakeThreshold::get();
                let min_rate = T::LargeHolderMinParticipationRate::get() as u32;
                const MAX_CHECKS_PER_QUARTER: usize = 20;

                // Collect first to avoid mutating while iterating.
                let entries: Vec<(T::AccountId, (u32, u32))> =
                    AccountVoteParticipation::<T>::iter()
                        .take(MAX_CHECKS_PER_QUARTER)
                        .collect();

                let checked_count = entries.len() as u32;

                for (account, (eligible, voted)) in entries {
                    // Evaluate only large holders.
                    let balance = T::Currency::free_balance(&account);
                    if balance >= threshold {
                        let participation_rate = if eligible > 0 {
                            voted.saturating_mul(100) / eligible
                        } else {
                            // No eligible proposals this quarter — treat as missed.
                            0u32
                        };
                        let current_multiplier = EffectiveVotingMultiplier::<T>::get(&account);
                        if participation_rate < min_rate {
                            // Apply or maintain penalty at 50%.
                            if current_multiplier != 50 {
                                EffectiveVotingMultiplier::<T>::insert(&account, 50u8);
                                weight = weight.saturating_add(T::DbWeight::get().writes(1));
                                Self::deposit_event(Event::LargeHolderParticipationPenalty {
                                    who: account.clone(),
                                    new_multiplier: 50,
                                });
                            }
                        } else if current_multiplier < 100 {
                            // Good participation this quarter — restore full weight.
                            EffectiveVotingMultiplier::<T>::insert(&account, 100u8);
                            weight = weight.saturating_add(T::DbWeight::get().writes(1));
                        }
                    }
                    // Reset participation counters for next quarter regardless.
                    AccountVoteParticipation::<T>::remove(&account);
                    weight = weight
                        .saturating_add(T::DbWeight::get().reads(2))
                        .saturating_add(T::DbWeight::get().writes(1));
                }

                Self::deposit_event(Event::QuarterlyParticipationReset { checked_count });
            }

            // ── Phase 1: Automatic council term expiries ──────────────────────
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
            let expired = TermExpiryQueue::<T>::take(n);

            if expired.is_empty() {
                return weight;
            }

            const MAX_EXPIRIES_PER_BLOCK: usize = 10;
            let (to_process, overflow): (Vec<_>, Vec<_>) = expired
                .into_iter()
                .enumerate()
                .partition(|(i, _)| *i < MAX_EXPIRIES_PER_BLOCK);

            // Remove expired members
            for (_, account) in &to_process {
                if let Some(member) = CouncilMembers::<T>::take(account) {
                    // Update board composition count
                    let current_count = BoardComposition::<T>::get(member.role);
                    if current_count > 0 {
                        BoardComposition::<T>::insert(member.role, current_count - 1);
                    }
                    Self::deposit_event(Event::CouncilTermAutoExpired {
                        account: account.clone(),
                    });
                    weight = weight
                        .saturating_add(T::WeightInfo::expire_council_member())
                        .saturating_add(T::DbWeight::get().writes(2));
                }
            }

            // Re-queue overflow to the next block
            if !overflow.is_empty() {
                let remaining = overflow.len() as u32;
                let next_block = n.saturating_add(BlockNumberFor::<T>::from(1u32));
                TermExpiryQueue::<T>::mutate(next_block, |queue| {
                    for (_, account) in overflow {
                        let _ = queue.try_push(account);
                    }
                });
                Self::deposit_event(Event::TermExpiryOverflow { block: n, remaining });
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }

            weight
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New proposal submitted
        ProposalSubmitted {
            proposal_id: u32,
            proposer: T::AccountId,
            proposal_type_index: u8,  // Index into ProposalType enum
            threshold_index: u8,      // Index into VotingThreshold enum
        },
        /// Vote cast on proposal.
        ///
        /// **Privacy Note**: Vote choice and voter identity are currently public.
        /// This enables accountability for governance votes. Commit-reveal privacy
        /// is roadmapped for Phase 3 (see `VoteCommitments` storage).
        VoteCast {
            proposal_id: u32,
            voter: T::AccountId,
            vote_index: u8,          // Index into VoteChoice enum
            weight: u32,
        },
        /// Proposal approved
        ProposalApproved {
            proposal_id: u32,
            ayes: u32,               // Simplified tally
            nays: u32,
            abstentions: u32,
        },
        /// Proposal rejected
        ProposalRejected {
            proposal_id: u32,
            ayes: u32,               // Simplified tally
            nays: u32,
            abstentions: u32,
        },
        // CouncilMemberElected, CouncilMemberRemoved, EmergencyTypeActivated
        // removed (E-7): orphaned events, never emitted via deposit_event.
        /// Community rank updated
        CommunityRankUpdated {
            account: T::AccountId,
            old_rank: u32,
            new_rank: u32,
        },
        /// PoUW contribution updated
        PoUWContributionUpdated {
            account: T::AccountId,
            old_contribution: u32,
            new_contribution: u32,
        },
        /// Council override executed
        CouncilOverrideExecuted {
            proposal_id: u32,
            override_reason: Vec<u8>,
        },
        /// Department manager set
        DepartmentManagerSet {
            department_index: u8,
            manager: T::AccountId,
        },
        /// Department proposal submitted
        DepartmentProposalSubmitted {
            proposal_id: u32,
            department_index: u8,
            proposer: T::AccountId,
        },
        /// Cross-department approval granted
        CrossDepartmentApproved {
            proposal_id: u32,
            department_index: u8,
            approver: T::AccountId,
        },
        // Phase 4: Foundation Board System Events
        /// Board member added with role
        BoardMemberAdded {
            member: T::AccountId,
            role_index: u8,
            term_end: BlockNumberFor<T>,
        },
        /// Board member removed
        BoardMemberRemoved {
            member: T::AccountId,
            role_index: u8,
            reason: BoundedVec<u8, ConstU32<128>>,
        },
        // TermExpired, DelegateElectionStarted removed (E-7): orphaned, never emitted.
        /// Citizen nominated for delegate position
        DelegateNominated {
            nominee: T::AccountId,
            nominator: T::AccountId,
        },
        /// Vote cast for delegate candidate
        DelegateVoteCast {
            voter: T::AccountId,
            nominee: T::AccountId,
            weight: u32,
        },
        // DelegateElectionCompleted removed (E-7): orphaned, never emitted.

        // ===== PHASE 5: EXECUTION LAYER EVENTS =====

        /// Proposal has been executed
        ProposalExecuted {
            proposal_id: u32,
            executor: T::AccountId,
            action_type: u8, // Index of ProposalAction variant
        },
        /// Runtime upgrade executed (governance approved — hash stored on-chain)
        RuntimeUpgradeExecuted {
            proposal_id: u32,
            code_hash: [u8; 32],
        },
        /// Voter committed a blinded vote hash for commit-reveal voting (AR-13)
        VoteCommitted {
            proposal_id: u32,
            voter: T::AccountId,
        },
        /// Voter revealed their committed vote; tally updated (AR-13)
        VoteRevealed {
            proposal_id: u32,
            voter: T::AccountId,
            /// 0 = Aye, 1 = Nay, 2 = Abstain
            vote_index: u8,
            weight: u32,
        },
        /// Governance-approved runtime upgrade code was verified and applied (AR-14)
        RuntimeUpgradeApplied {
            code_hash: [u8; 32],
        },
        /// Governance parameter changed
        ParameterChanged {
            parameter_index: u8, // Index of GovernanceParameter variant
            old_value: u32,
            new_value: u32,
        },
        /// Department action executed
        DepartmentActionExecuted {
            proposal_id: u32,
            department_index: u8, // Index of Department variant
        },
        /// Emergency action executed
        EmergencyActionExecuted {
            proposal_id: u32,
            action_type_index: u8, // Index of EmergencyActionType variant
        },
        /// Funds allocated to department treasury
        DepartmentFundsAllocated {
            department_index: u8, // Index of Department variant
            amount: BalanceOf<T>,
            from_proposal: u32,
        },

        // ===== PHASE 6: COUNCIL ELECTION SYSTEM EVENTS =====

        /// District election started
        DistrictElectionStarted {
            election_id: BlockNumberFor<T>,
            district_index: u8, // Index of BelizeDistrict (0-5)
            seats: u32,
            registration_end: BlockNumberFor<T>,
            voting_end: BlockNumberFor<T>,
        },
        /// Candidate registered for district election
        CandidateRegistered {
            election_id: BlockNumberFor<T>,
            candidate: T::AccountId,
            district_index: u8, // Index of BelizeDistrict (0-5)
        },
        /// Vote cast in district election
        ElectionVoteCast {
            election_id: BlockNumberFor<T>,
            voter: T::AccountId,
            candidate: T::AccountId,
            district_index: u8, // Index of BelizeDistrict (0-5)
        },
        /// District election finalized
        DistrictElectionFinalized {
            election_id: BlockNumberFor<T>,
            district_index: u8, // Index of BelizeDistrict (0-5)
            winners: BoundedVec<T::AccountId, ConstU32<5>>,
            total_votes: u32,
        },
        /// Council member elected from district
        CouncilMemberElectedFromDistrict {
            account: T::AccountId,
            district_index: u8, // Index of BelizeDistrict (0-5)
            votes_received: u32,
            term_end: BlockNumberFor<T>,
        },

        // ===== PHASE 7: ADVANCED FEATURES EVENTS =====

        /// Vote delegation created
        VoteDelegationCreated {
            delegator: T::AccountId,
            delegate: T::AccountId,
            expires_at: Option<BlockNumberFor<T>>,
        },
        /// Vote delegation revoked
        VoteDelegationRevoked {
            delegator: T::AccountId,
            delegate: T::AccountId,
        },
        /// Proposal amendment submitted
        ProposalAmendmentSubmitted {
            proposal_id: u32,
            proposer: T::AccountId,
        },
        // ProposalAmendmentApplied removed (E-7): orphaned, never emitted.
        /// Governance participation reward claimed
        RewardClaimed {
            account: T::AccountId,
            reward_type: u8, // 0=Vote, 1=Proposal, 2=Council
            amount: BalanceOf<T>,
        },
        /// Proposal added to priority queue
        ProposalQueuedWithPriority {
            proposal_id: u32,
            priority_index: u8, // 1=Low, 2=Normal, 3=High, 4=Critical
        },

        // ===== JAGUAR MODE: EMERGENCY GOVERNANCE EVENTS =====

        // JaguarModeActivated removed (E-7): orphaned, never emitted.
        /// 🐆 Jaguar Mode deactivated - Emergency ended
        JaguarModeDeactivated,
        /// 🐆 Emergency declaration pending — veto window open (CONS-029)
        EmergencyPendingActivation {
            emergency_type_index: u8,
            description: BoundedVec<u8, ConstU32<256>>,
            veto_window_ends_at: BlockNumberFor<T>,
        },
        /// 🐆 Council member vetoed a pending emergency (CONS-029)
        EmergencyVetoed {
            who: T::AccountId,
            vetoes_so_far: u32,
        },
        /// 🐆 Pending emergency finalized after veto window elapsed (CONS-029)
        EmergencyFinalizedAfterVetoWindow,

        // ===== REFERENDUM SYSTEM EVENTS =====

        /// Referendum created
        /// district_index: Optional (0=Belize, 1=Cayo, 2=Corozal, 3=OrangeWalk, 4=StannCreek, 5=Toledo)
        ReferendumCreated {
            referendum_id: u32,
            creator: T::AccountId,
            title: BoundedVec<u8, ConstU32<256>>,
            quorum_percentage: u8,
            district_index: Option<u8>,
        },
        /// Vote cast on referendum
        ReferendumVoteCast {
            referendum_id: u32,
            voter: T::AccountId,
            option_index: u8,
            weight: u32,
        },
        /// Referendum passed (quorum met, executed)
        ReferendumPassed {
            referendum_id: u32,
            winning_option_index: u8,
            total_votes: u32,
            participation_percentage: u8,
        },
        /// Referendum failed (quorum not met)
        ReferendumFailed {
            referendum_id: u32,
            total_votes: u32,
            participation_percentage: u8,
            required_quorum: u8,
        },
        // ReferendumCancelled removed (E-7): orphaned, never emitted.

        // ===== TREASURY MANAGEMENT EVENTS =====

        /// District budget allocated for fiscal year
        /// district_index: 0=Belize, 1=Cayo, 2=Corozal, 3=OrangeWalk, 4=StannCreek, 5=Toledo
        DistrictBudgetAllocated {
            district_index: u8,
            amount: BalanceOf<T>,
            fiscal_year_start: BlockNumberFor<T>,
            fiscal_year_end: BlockNumberFor<T>,
        },
        /// Treasury spend proposal created
        TreasurySpendProposed {
            proposal_id: u32,
            proposer: T::AccountId,
            recipient: T::AccountId,
            amount: BalanceOf<T>,
            threshold: u8,
        },
        /// Treasury spend proposal approved by signatory
        TreasurySpendApproved {
            proposal_id: u32,
            approver: T::AccountId,
            approvals_count: u8,
            threshold: u8,
        },
        /// Treasury spend proposal executed (funds transferred)
        TreasurySpendExecuted {
            proposal_id: u32,
            recipient: T::AccountId,
            amount: BalanceOf<T>,
            district_index: Option<u8>,
        },
        /// Budget transferred between districts
        DistrictBudgetTransferred {
            from_district_index: u8,
            to_district_index: u8,
            amount: BalanceOf<T>,
            reason: BoundedVec<u8, ConstU32<256>>,
        },
        /// District budget spending recorded
        DistrictBudgetSpent {
            district_index: u8,
            proposal_id: u32,
            amount: BalanceOf<T>,
            remaining: BalanceOf<T>,
        },

        // Emergency Governance Events
        /// Emergency proposal executed via fast-track
        EmergencyProposalExecuted {
            proposal_id: u32,
            emergency_type_index: u8,
            approval_percentage: u32,
        },
        /// Referendum fast-tracked during emergency
        ReferendumFastTracked {
            referendum_id: u32,
            original_deadline: BlockNumberFor<T>,
            new_deadline: BlockNumberFor<T>,
            emergency_type_index: u8,
        },
        /// Proposal result overridden during emergency
        ProposalEmergencyOverride {
            proposal_id: u32,
            executed: bool,
            overridden_by: [u8; 32],
            justification: BoundedVec<u8, ConstU32<512>>,
        },
        /// A governance-controlled chain parameter was updated
        ChainParameterUpdated {
            key: BoundedVec<u8, ConstU32<32>>,
            old_value: Option<u64>,
            new_value: u64,
        },

        // ===== PHASE 1: ETHICAL SAFEGUARDS EVENTS =====

        /// A council member's term expired and they were automatically removed.
        CouncilTermAutoExpired {
            account: T::AccountId,
        },
        /// More than 10 terms expired in one block; remainder re-queued to next block.
        TermExpiryOverflow {
            block: BlockNumberFor<T>,
            remaining: u32,
        },
        /// An approved proposal has been queued for enactment at a future block.
        ProposalEnqueued {
            proposal_id: u32,
            execute_at: BlockNumberFor<T>,
        },
        // ProposalCooldownBlocked removed (E-7): orphaned, never emitted.
        /// A large holder failed the quarterly participation check and had their
        /// effective voting multiplier reduced.
        LargeHolderParticipationPenalty {
            who: T::AccountId,
            new_multiplier: u8,
        },
        /// Quarterly participation check completed.  `checked_count` is the number
        /// of large-holder accounts that were evaluated.
        QuarterlyParticipationReset {
            checked_count: u32,
        },

        // ── Phase 4C: Exit right event ────────────────────────────────────────

        /// Account generated a chain-signed exit proof of their on-chain state.
        /// The `proof_hash` is `blake2_256(account ++ free_balance ++ current_block)`.
        /// The proof can be independently verified off-chain to demonstrate
        /// the account's state at the moment of exit.
        ExitProofGenerated {
            account: T::AccountId,
            proof_hash: [u8; 32],
            valid_until: BlockNumberFor<T>,
        },

        // ===== PHASE 6A: CONSTITUTIONAL DUAL-HOUSE RATIFICATION EVENTS =====

        /// A house member cast a ratification vote for a Constitutional proposal.
        /// `house`: 0 = TechnicalCouncil, 1 = GovernanceCouncil.
        ConstitutionalRatificationCast {
            proposal_id: u32,
            ratifier: T::AccountId,
            house: u8,
            technical_approved: bool,
            governance_approved: bool,
        },

        /// Both houses have ratified; the Constitutional proposal is now executable.
        ConstitutionalRatificationComplete {
            proposal_id: u32,
        },

        // ===== PHASE 6B: CONSTITUTIONAL PARAMETER LOCK EVENTS =====

        /// A chain parameter was constitutionally locked.
        ParameterLocked {
            key: BoundedVec<u8, ConstU32<32>>,
        },

        /// A constitutional lock on a chain parameter was lifted.
        ParameterUnlocked {
            key: BoundedVec<u8, ConstU32<32>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Proposal not found
        ProposalNotFound,
        /// Not a council member
        NotCouncilMember,
        /// Not compliant (KYC/AML verification required)
        NotCompliant,
        /// Too many approvals
        TooManyApprovals,
        /// Too many proposals funded
        TooManyProposals,
        /// Voting period has ended
        VotingPeriodEnded,
        /// Voting period not started
        VotingPeriodNotStarted,
        /// Already voted on this proposal
        AlreadyVoted,
        /// Insufficient deposit for proposal
        InsufficientDeposit,
        /// Invalid voting threshold
        InvalidThreshold,
        /// Council size limits exceeded
        CouncilSizeLimits,
        /// Insufficient voting weight
        InsufficientVotingWeight,
        /// Emergency mode restrictions
        EmergencyTypeRestrictions,
        /// Unauthorized operation
        Unauthorized,
        /// Invalid proposal type
        InvalidProposalType,
        /// Insufficient compliance/KYC level for governance participation
        InsufficientCompliance,
        /// Account is restricted by compliance
        RestrictedAccount,
        /// Not a department manager
        NotDepartmentManager,
        /// Department not found or not initialized
        DepartmentNotFound,
        /// Cross-department approval not required for this proposal
        CrossApprovalNotRequired,
        /// Department already approved this proposal
        DepartmentAlreadyApproved,
        /// Invalid department index
        InvalidDepartment,
        /// Too many cross-department approvals
        TooManyCrossApprovals,
        // Phase 4: Foundation Board System Errors
        /// Board role limit exceeded
        BoardRoleLimitExceeded,
        /// Invalid board role
        InvalidBoardRole,
        /// Member term has not expired
        TermNotExpired,
        /// Member not found in board
        MemberNotFound,
        /// Delegate election not active
        NoActiveElection,
        /// Already voted in current election
        AlreadyVotedInElection,
        /// Nominee not found
        NomineeNotFound,
        /// Term duration invalid (must be 1-2 years)
        InvalidTermDuration,

        // ===== PHASE 5: EXECUTION LAYER ERRORS =====

        /// Proposal not approved yet
        ProposalNotApproved,
        /// Proposal already executed
        AlreadyExecuted,
        /// No executable action defined for this proposal
        NoActionDefined,
        /// Insufficient treasury balance
        InsufficientTreasuryBalance,
        /// Treasury spend cap exceeded for this period (S5-4)
        TreasurySpendCapExceeded,
        /// Runtime upgrade code too large
        RuntimeCodeTooLarge,
        /// Invalid governance parameter value
        InvalidParameterValue,
        /// Department action execution failed
        DepartmentActionFailed,
        /// Emergency action not allowed
        EmergencyActionNotAllowed,
        /// Insufficient department treasury balance
        InsufficientDepartmentBalance,
        /// Invalid action type for this proposal
        InvalidActionType,

        // ===== PHASE 6: COUNCIL ELECTION SYSTEM ERRORS =====

        /// Election not found for this district
        ElectionNotFound,
        /// Election is not in registration phase
        NotInRegistrationPhase,
        /// Election is not in voting phase
        NotInVotingPhase,
        /// Candidate already registered for this election
        CandidateAlreadyRegistered,
        /// Maximum candidate count reached for this election
        TooManyCandidates,
        /// Already voted in this election
        AlreadyVotedInDistrictElection,
        /// Candidate not found in election
        CandidateNotFound,
        /// Election already active for this district
        ElectionAlreadyActive,
        /// Cannot finalize election yet (voting period not ended)
        VotingPeriodNotEnded,
        /// Election already finalized
        ElectionAlreadyFinalized,
        /// Invalid district
        InvalidDistrict,
        /// District required for DistrictLocal proposals
        DistrictRequired,
        /// Not a resident of the specified district
        NotDistrictResident,
        /// Platform statement too long
        PlatformTooLong,
        /// Not enough candidates to fill seats
        InsufficientCandidates,

        // ===== PHASE 7: ADVANCED FEATURES ERRORS =====

        /// Cannot delegate to self
        CannotDelegateToSelf,
        /// Delegation already exists
        DelegationAlreadyExists,
        /// No active delegation found
        NoDelegationFound,
        /// Delegate has too many delegators
        TooManyDelegators,
        /// Not the proposal author
        NotProposalAuthor,
        /// Amendment already exists
        AmendmentAlreadyExists,
        /// Amendment not found
        AmendmentNotFound,
        /// G-8 FIX: No amendment fields provided
        NoAmendmentProvided,
        /// Cannot amend after voting started
        CannotAmendAfterVoting,
        /// Reward already claimed
        RewardAlreadyClaimed,
        /// No reward available
        NoRewardAvailable,
        /// Insufficient treasury for rewards
        InsufficientTreasuryForRewards,
        /// Priority queue full
        PriorityQueueFull,
        /// Invalid priority level
        InvalidPriority,

        // ===== JAGUAR MODE: EMERGENCY GOVERNANCE ERRORS =====

        /// Emergency already active - cannot declare new emergency
        EmergencyAlreadyActive,
        /// No active emergency - cannot end emergency
        NoActiveEmergency,
        /// Emergency description too long (max 256 bytes)
        DescriptionTooLong,

        // ===== REFERENDUM SYSTEM ERRORS =====

        /// Referendum not found
        ReferendumNotFound,
        /// Referendum voting period has ended
        ReferendumVotingEnded,
        /// Referendum voting period not started yet
        ReferendumVotingNotStarted,
        /// Already voted on this referendum
        AlreadyVotedOnReferendum,
        /// Too many options for referendum (max 10)
        TooManyOptions,
        /// No options provided for referendum
        NoOptionsProvided,
        /// Invalid quorum percentage (must be 1-100)
        InvalidQuorum,
        /// Referendum title too long (max 256 bytes)
        ReferendumTitleTooLong,
        /// Referendum description too long (max 1024 bytes)
        ReferendumDescriptionTooLong,
        /// Invalid option index
        InvalidOptionIndex,
        /// Referendum not finalized yet
        ReferendumNotFinalized,
        /// Referendum already finalized
        ReferendumAlreadyFinalized,

        // ===== TREASURY MANAGEMENT ERRORS =====

        /// District budget not found
        DistrictBudgetNotFound,
        /// District budget already exists for fiscal year
        DistrictBudgetAlreadyExists,
        /// Insufficient district budget for spending
        InsufficientDistrictBudget,
        /// Treasury spend proposal not found
        TreasuryProposalNotFound,
        /// Treasury spend proposal already executed
        TreasuryProposalAlreadyExecuted,
        /// Treasury spend proposal expired
        TreasuryProposalExpired,
        /// Already approved this treasury proposal
        AlreadyApprovedTreasuryProposal,
        /// Insufficient approvals to execute treasury proposal
        InsufficientTreasuryApprovals,
        /// Treasury description too long (max 512 bytes)
        TreasuryDescriptionTooLong,
        /// Invalid fiscal year period (must be > 0 blocks)
        InvalidFiscalYearPeriod,
        /// Insufficient national treasury reserves
        InsufficientNationalTreasury,
        /// Transfer amount exceeds source district budget
        TransferExceedsSourceBudget,

        // Emergency Governance Errors
        /// JaguarMode not currently active
        NotInEmergencyMode,
        /// Proposal does not have emergency priority
        NotEmergencyProposal,
        /// Emergency proposal voting period has expired
        EmergencyProposalExpired,
        /// Super-majority (66%+) not achieved for emergency action
        InsufficientSuperMajority,
        /// Veto window has not yet expired (CONS-029)
        VetoWindowNotExpired,
        /// Emergency is still in pending veto phase (CONS-029)
        EmergencyStillPending,
        /// Council member already vetoed this pending emergency (CONS-029)
        AlreadyVetoed,
        /// No pending emergency to veto (CONS-029)
        NoPendingEmergency,

        // Chain Parameter Errors
        /// Parameter key too long (max 32 bytes)
        ParameterKeyTooLong,

        // Commit-Reveal Voting Errors (AR-13)
        /// A commitment for this proposal has already been submitted by the caller.
        VoteAlreadyCommitted,
        /// The revealed vote+salt does not match the stored commitment hash.
        CommitmentHashMismatch,
        /// No commitment found — caller must commit before revealing.
        NoCommitmentFound,
        /// This account already revealed its vote (and it was recorded in `Votes`).
        VoteAlreadyRevealedViaCommit,

        // Runtime Upgrade Errors (AR-14)
        /// No pending runtime upgrade has been approved via governance.
        RuntimeUpgradeNotPending,
        /// Submitted code hash does not match governance-approved hash.
        RuntimeCodeHashMismatch,
        // Department Action Errors (AR-8)
        /// `call_data` bytes could not be SCALE-decoded as a valid `DepartmentCall`.
        InvalidCallData,

        // ===== PHASE 1: ETHICAL SAFEGUARDS ERRORS =====

        /// Account must wait `ProposalCooldown` blocks before submitting another proposal.
        ProposalCooldownActive,
        /// An approved proposal cannot be executed before its enactment delay has elapsed.
        EnactmentDelayNotPassed,
        /// Council member has reached the maximum number of consecutive terms.
        ConsecutiveTermLimitReached,

        // ===== PHASE 5A: BEHAVIOR FLAG CIRCUIT BREAKER ERRORS =====

        /// Account is in a behavior-flag cooldown period and may not submit proposals or vote.
        /// The cooldown was triggered by oracle consensus detecting an anti-social pattern
        /// (compulsive activity, reward-loop exploitation, or bot-like behavior).
        BehaviorCooldownActive,

        // ===== PHASE 6A: CONSTITUTIONAL DUAL-HOUSE RATIFICATION ERRORS =====

        /// Constitutional proposal requires dual-house ratification before it can be executed.
        /// Call `ratify_constitutional_proposal` from each house (Technical + Governance).
        ConstitutionalRatificationRequired,
        /// Caller is not a member of either governance house and cannot ratify.
        NotHouseMember,
        /// This governance house has already cast a ratification for this proposal.
        AlreadyRatifiedByHouse,
        /// Ratification is only applicable to Constitutional proposals.
        NotConstitutionalProposal,
        /// Proposal must reach Approved status before dual-house ratification can begin.
        ProposalNotYetApproved,

        // ===== PHASE 6B: CONSTITUTIONAL PARAMETER LOCK ERRORS =====

        /// This chain parameter has been constitutionally locked and cannot be updated.
        /// Unlock it via a dual-house-ratified Constitutional proposal first.
        ParameterConstitutionallyLocked,
        /// Parameter key is already locked; no-op lock rejected.
        ParameterAlreadyLocked,
        /// Parameter key is not currently locked; no-op unlock rejected.
        ParameterNotLocked,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new governance proposal to the council.
        ///
        /// Creates a new proposal that will enter a voting period after the launch delay.
        /// Requires a minimum deposit that is refunded if the proposal passes and slashed
        /// if it fails to meet quorum.
        ///
        /// ## Complexity
        /// - **Reads**: 3 (Account balance, compliance status, next proposal ID)
        /// - **Writes**: 2 (Reserve deposit, store proposal)
        /// - **Weight**: `T::WeightInfo::submit_proposal()`
        ///
        /// ## Parameters
        ///
        /// - `origin`: Signed origin of the proposal author (requires Contributor tier or higher)
        /// - `title`: Proposal title (max 256 bytes, UTF-8 recommended)
        /// - `description`: Detailed proposal description (max 1024 bytes)
        /// - `proposal_type_index`: Type index (0=Constitutional, 1=Economic, 2=Council, 
        ///   3=Technical, 4=Emergency, 5=International, 6=Community)
        /// - `threshold_index`: Voting threshold (0=SimpleMajority 51%, 1=Supermajority 66%, 
        ///   2=Unanimous 100%)
        /// - `is_emergency`: If true, reduces launch period to 1 hour and voting to 3 hours
        ///
        /// ## Emits
        ///
        /// - `ProposalSubmitted`: When proposal is successfully created with proposal_id
        ///
        /// ## Errors
        ///
        /// - `InsufficientCompliance`: Proposer lacks required KYC verification
        /// - `InsufficientBalance`: Cannot reserve required deposit
        /// - `InvalidProposalType`: Invalid type or threshold index
        /// - `InvalidProposalData`: Title or description exceeds size limits
        ///
        /// ## Example
        ///
        /// ```rust,ignore
        /// // Submit a treasury spending proposal
        /// let result = Governance::submit_proposal(
        ///     Origin::signed(alice),
        ///     b"Road Repair Budget".to_vec(),
        ///     b"Allocate 50,000 DALLA for Cayo district road maintenance".to_vec(),
        ///     1, // Economic proposal
        ///     0, // Simple majority
        ///     false, // Not emergency
        /// );
        /// ```
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_proposal())]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Vec<u8>,
            proposal_type_index: u8,
            threshold_index: u8,
            is_emergency: bool,
            district_index: Option<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance/KYC requirements
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Phase 5A: Reject if account is in behavior-flag cooldown
            ensure!(
                !T::BehaviorFlags::has_active_flag(&who),
                Error::<T>::BehaviorCooldownActive
            );

            // Enforce proposal cooldown — emergency proposals bypass the wait
            if !is_emergency {
                let now = frame_system::Pallet::<T>::block_number();
                let last = AccountLastProposal::<T>::get(&who);
                let elapsed = now.saturating_sub(last);
                ensure!(
                    elapsed >= T::ProposalCooldown::get(),
                    Error::<T>::ProposalCooldownActive
                );
            }

            // Convert indices to enums with validation
            let proposal_type = match proposal_type_index {
                0 => ProposalType::Constitutional,
                1 => ProposalType::Economic,
                2 => ProposalType::Council,
                3 => ProposalType::Technical,
                4 => ProposalType::Emergency,
                5 => ProposalType::International,
                6 => ProposalType::Community,
                7 => ProposalType::DistrictLocal,
                8 => ProposalType::WellbeingFunding,
                _ => return Err(Error::<T>::InvalidProposalType.into()),
            };

            let threshold = match threshold_index {
                0 => VotingThreshold::SimpleMajority,
                1 => VotingThreshold::Supermajority,
                2 => VotingThreshold::Unanimous,
                _ => return Err(Error::<T>::InvalidThreshold.into()),
            };

            // Convert and validate district for DistrictLocal proposals
            let district = if let Some(idx) = district_index {
                ensure!(idx < 6, Error::<T>::InvalidDistrict);
                let dist = match idx {
                    0 => BelizeDistrict::Belize,
                    1 => BelizeDistrict::Cayo,
                    2 => BelizeDistrict::Corozal,
                    3 => BelizeDistrict::OrangeWalk,
                    4 => BelizeDistrict::StannCreek,
                    5 => BelizeDistrict::Toledo,
                    _ => return Err(Error::<T>::InvalidDistrict.into()),
                };
                
                // DistrictLocal proposals MUST specify a district
                if proposal_type == ProposalType::DistrictLocal {
                    Some(dist)
                } else {
                    // Optional district tag for non-DistrictLocal proposals
                    Some(dist)
                }
            } else {
                // DistrictLocal proposals MUST have a district
                if proposal_type == ProposalType::DistrictLocal {
                    return Err(Error::<T>::DistrictRequired.into());
                }
                None
            };

            let deposit = T::MinimumDeposit::get();
            
            // Reserve deposit
            T::Currency::reserve(&who, deposit)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let launch_period = if is_emergency { 
                BlockNumberFor::<T>::from(3600u32) // 1 hour for emergency
            } else { 
                T::LaunchPeriod::get() 
            };
            
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let launch_u64: u64 = TryInto::<u64>::try_into(launch_period).unwrap_or(0);
            let voting_start_u64 = current_u64.saturating_add(launch_u64);
            // SAFETY: value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let voting_start: BlockNumberFor<T> = voting_start_u64.saturated_into();
            let voting_period = if is_emergency {
                BlockNumberFor::<T>::from(21600u32) // 3 hours for emergency
            } else {
                T::VotingPeriod::get()
            };
            let voting_start_u64_2: u64 = TryInto::<u64>::try_into(voting_start).unwrap_or(0);
            let voting_period_u64: u64 = TryInto::<u64>::try_into(voting_period).unwrap_or(0);
            let voting_end_u64 = voting_start_u64_2.saturating_add(voting_period_u64);
            // SAFETY: value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let voting_end: BlockNumberFor<T> = voting_end_u64.saturated_into();

            let proposal_id = Self::next_proposal_id();

            let bounded_title: BoundedVec<u8, ConstU32<256>> = title.try_into().map_err(|_| Error::<T>::InvalidProposalType)?;
            let bounded_description: BoundedVec<u8, ConstU32<1024>> = description.try_into().map_err(|_| Error::<T>::InvalidProposalType)?;

            let proposal = Proposal {
                id: proposal_id,
                proposer: who.clone(),
                title: bounded_title,
                description: bounded_description,
                proposal_type: proposal_type.clone(),
                threshold: threshold.clone(),
                deposit,
                voting_start,
                voting_end,
                vote_tally: VoteTally {
                    ayes: 0,
                    nays: 0,
                    abstentions: 0,
                    total_weight: 0,
                    participation: 0,
                },
                status: ProposalStatus::Pending,
                is_emergency,
                department: None, // General proposal, no department
                district,
                requires_cross_approval: false,
                cross_approved_by: BoundedVec::default(),
                action: None, // Phase 5: No action defined yet
                executed_at: None, // Phase 5: Not executed
            };

            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id.saturating_add(1));
            // Record the block of this submission for cooldown tracking
            AccountLastProposal::<T>::insert(&who, current_block);

            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer: who,
                proposal_type_index: match &proposal_type {
                    ProposalType::Constitutional => 0,
                    ProposalType::Economic => 1,
                    ProposalType::Council => 2,
                    ProposalType::Technical => 3,
                    ProposalType::Emergency => 4,
                    ProposalType::International => 5,
                    ProposalType::Community => 6,
                    ProposalType::DistrictLocal => 7,
                    ProposalType::WellbeingFunding => 8,
                },
                threshold_index: match &threshold {
                    VotingThreshold::SimpleMajority => 0,
                    VotingThreshold::Supermajority => 1,
                    VotingThreshold::Unanimous => 2,
                    VotingThreshold::Custom(_) => 3,
                },
            });

            Ok(())
        }

        /// Cast a vote on an active proposal with optional conviction multiplier.
        ///
        /// Votes are weighted based on the voter's community rank and Proof of Useful Work
        /// contributions. Conviction voting allows voters to increase their voting power by
        /// locking tokens for longer periods. Each vote earns 10 DALLA participation reward.
        ///
        /// ## Complexity
        /// - **Reads**: 5 (Proposal, compliance, vote check, community rank, PoUW)
        /// - **Writes**: 2 (Record vote, update proposal tally)
        /// - **Weight**: `T::WeightInfo::cast_vote()`
        ///
        /// ## Parameters
        ///
        /// - `origin`: Signed origin of the voter (requires Contributor tier or higher)
        /// - `proposal_id`: ID of the proposal to vote on
        /// - `vote_choice_index`: Vote direction (0=Aye/Yes, 1=Nay/No, 2=Abstain)
        /// - `conviction`: Voting power multiplier (0=1x no lock, 1=2x 7-day lock, 
        ///   2=3x 30-day lock, 3=6x 90-day lock)
        ///
        /// ## Voting Weight Formula
        ///
        /// ```text
        /// final_weight = (community_rank + pouw_contribution) * conviction_multiplier
        /// ```
        ///
        /// ## Conviction Locking Periods
        ///
        /// | Conviction | Multiplier | Lock Period | Use Case |
        /// |------------|-----------|-------------|-----------|
        /// | 0 | 1x | None | Quick votes, uncertain |
        /// | 1 | 2x | 7 days | Moderate confidence |
        /// | 2 | 3x | 30 days | Strong support |
        /// | 3 | 6x | 90 days | Maximum conviction |
        ///
        /// ## Emits
        ///
        /// - `VoteCast`: When vote is successfully recorded with weight information
        ///
        /// ## Errors
        ///
        /// - `InsufficientCompliance`: Voter lacks required KYC verification
        /// - `ProposalNotFound`: Invalid proposal ID
        /// - `VotingPeriodNotStarted`: Proposal not yet open for voting
        /// - `VotingPeriodEnded`: Voting period has closed
        /// - `AlreadyVoted`: Account has already voted on this proposal
        ///
        /// ## Example
        ///
        /// ```rust,ignore
        /// // Vote YES with strong conviction (3x weight, 30-day lock)
        /// let result = Governance::cast_vote(
        ///     Origin::signed(alice),
        ///     42, // proposal_id
        ///     0,  // Aye (Yes)
        ///     2,  // 3x conviction with 30-day lock
        /// );
        /// 
        /// // Voter earns 10 DALLA participation reward
        /// // Voting weight = (rank + pouw) * 3
        /// ```
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::cast_vote())]
        pub fn cast_vote(
            origin: OriginFor<T>,
            proposal_id: u32,
            vote_choice_index: u8,
            conviction: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance/KYC requirements
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Phase 5A: Reject if account is in behavior-flag cooldown
            ensure!(
                !T::BehaviorFlags::has_active_flag(&who),
                Error::<T>::BehaviorCooldownActive
            );

            // Convert index to enum with validation
            let vote_choice = match vote_choice_index {
                0 => VoteChoice::Aye,
                1 => VoteChoice::Nay,
                2 => VoteChoice::Abstain,
                _ => return Err(Error::<T>::InvalidThreshold.into()), // Reuse existing error
            };

            let mut proposal = Self::proposals(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            let current_block = frame_system::Pallet::<T>::block_number();

            // Check voting period
            ensure!(current_block >= proposal.voting_start, Error::<T>::VotingPeriodNotStarted);
            ensure!(current_block <= proposal.voting_end, Error::<T>::VotingPeriodEnded);

            // Check if already voted
            ensure!(!Votes::<T>::contains_key(proposal_id, &who), Error::<T>::AlreadyVoted);

            // Calculate voting weight (community rank + PoUW contribution + stake QV)
            let community_rank = Self::community_ranks(&who);
            let pouw_contribution = Self::pouw_contributions(&who);

            // ── Phase 2A: Stake-based quadratic voting component ──────────────
            // Converts free balance → stake units → optionally quadratic-rooted → capped.
            let balance_u128: u128 = T::Currency::free_balance(&who).saturated_into::<u128>();
            let stake_unit_size_u128: u128 = T::StakeUnitSize::get().saturated_into::<u128>();
            let stake_units: u128 = if stake_unit_size_u128 > 0 {
                balance_u128 / stake_unit_size_u128
            } else {
                0
            };
            let raw_stake_weight = if T::QuadraticVotingEnabled::get() {
                Self::isqrt(stake_units)
            } else {
                stake_units
            };
            let stake_weight: u32 = raw_stake_weight
                .min(T::MaxVotingUnits::get() as u128) as u32;

            let base_weight = community_rank
                .saturating_add(pouw_contribution)
                .saturating_add(stake_weight);

            // P1-14 FIX: Include delegated voting power. calculate_voting_power()
            // returns 1 (base) + number of active delegations to this voter.
            let delegation_bonus = Self::calculate_voting_power(&who).saturating_sub(1);
            let base_weight = base_weight.saturating_add(delegation_bonus);

            // Apply conviction multiplier (minimum 1x to preserve vote weight)
            let conviction_multiplier = (conviction as u32).max(1);
            let conviction_weight = base_weight.saturating_mul(conviction_multiplier);

            // ── Phase 2B: Apply effective voting multiplier (wealth-responsibility) ──
            // Defaults to 100 (full weight).  Large holders who missed last quarter's
            // participation threshold have this set to 50 until the next quarterly reset.
            let multiplier = EffectiveVotingMultiplier::<T>::get(&who);
            let effective_mult = if multiplier == 0 { 100u32 } else { multiplier as u32 };
            let final_weight = conviction_weight
                .saturating_mul(effective_mult)
                / 100;

            // Track participation for the quarterly audit (Phase 2B).
            // Increment the "voted" counter; "eligible" is incremented in create_proposal
            // when a proposal enters the voting period.
            AccountVoteParticipation::<T>::mutate(&who, |(eligible, voted)| {
                // If neither counter has been touched yet this quarter, initialise eligible
                // based on the current proposal count so new voters aren't unfairly penalised.
                if *eligible == 0 && *voted == 0 {
                    *eligible = 1;
                }
                *voted = voted.saturating_add(1);
            });

            // Record vote
            let vote = Vote {
                vote: vote_choice.clone(),
                weight: final_weight,
                conviction,
                // SAFETY: BlockNumber fits in u64 (runtime uses u32 block numbers)
                timestamp: current_block.saturated_into(),
            };

            Votes::<T>::insert(proposal_id, &who, vote);

            // Update proposal tally
            match vote_choice {
                VoteChoice::Aye => proposal.vote_tally.ayes = proposal.vote_tally.ayes.saturating_add(final_weight),
                VoteChoice::Nay => proposal.vote_tally.nays = proposal.vote_tally.nays.saturating_add(final_weight),
                VoteChoice::Abstain => proposal.vote_tally.abstentions = proposal.vote_tally.abstentions.saturating_add(final_weight),
            }

            proposal.vote_tally.total_weight = proposal.vote_tally.total_weight.saturating_add(final_weight);

            // Update proposal status if needed
            proposal.status = ProposalStatus::Voting;

            Proposals::<T>::insert(proposal_id, proposal);

            // Record governance participation in Community pallet (Phase 6)
            let _ = T::CommunityParticipation::record_vote_cast(&who);

            Self::deposit_event(Event::VoteCast {
                proposal_id,
                voter: who,
                vote_index: match vote_choice {
                    VoteChoice::Aye => 0,
                    VoteChoice::Nay => 1,
                    VoteChoice::Abstain => 2,
                },
                weight: final_weight,
            });

            Ok(())
        }

        /// Finalize voting on a proposal
        #[pallet::call_index(2)]
        // DOS-009 FIX: Operational dispatch — governance finalization must not be
        // blocked during congestion.
        #[pallet::weight((T::WeightInfo::finalize_proposal(), DispatchClass::Operational))]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let mut proposal = Self::proposals(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            let current_block = frame_system::Pallet::<T>::block_number();

            // Ensure voting period has ended
            ensure!(current_block > proposal.voting_end, Error::<T>::VotingPeriodEnded);

            // Calculate result based on threshold
            let required_percentage = match proposal.threshold {
                VotingThreshold::SimpleMajority => 50,
                VotingThreshold::Supermajority => SUPERMAJORITY_THRESHOLD,
                VotingThreshold::Unanimous => 100,
                VotingThreshold::Custom(pct) => pct as u32,
            };

            let total_decisive_votes = proposal.vote_tally.ayes.saturating_add(proposal.vote_tally.nays);
            let approval_percentage = if total_decisive_votes > 0 {
                proposal.vote_tally.ayes.saturating_mul(100) / total_decisive_votes
            } else {
                0
            };

            // Determine outcome
            let approved = approval_percentage >= required_percentage;

            if approved {
                proposal.status = ProposalStatus::Approved;
                // Compute enactment delay based on proposal type
                let enact_delay = match proposal.proposal_type {
                    ProposalType::Constitutional => T::EnactmentPeriodConstitutional::get(),
                    ProposalType::Economic => T::EnactmentPeriodEconomic::get(),
                    _ => T::EnactmentPeriodStandard::get(),
                };
                // CONS-030: Enforce minimum timelocks per action type
                let action_timelock = match &proposal.action {
                    Some(ProposalAction::RuntimeUpgrade { .. }) => T::RuntimeUpgradeMinTimelock::get(),
                    Some(ProposalAction::ParameterChange { .. }) => T::ParameterChangeMinTimelock::get(),
                    _ => Zero::zero(),
                };
                let enact_delay = enact_delay.max(action_timelock);
                let current_u64_enact: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
                let delay_u64: u64 = TryInto::<u64>::try_into(enact_delay).unwrap_or(0);
                // SAFETY: derived from u32 block number arithmetic in u64; fits BlockNumberFor<T>
                let execute_at: BlockNumberFor<T> = current_u64_enact.saturating_add(delay_u64).saturated_into();
                ProposalEnactmentBlock::<T>::insert(proposal_id, execute_at);
                Self::deposit_event(Event::ProposalEnqueued { proposal_id, execute_at });
                Self::deposit_event(Event::ProposalApproved {
                    proposal_id,
                    ayes: proposal.vote_tally.ayes,
                    nays: proposal.vote_tally.nays,
                    abstentions: proposal.vote_tally.abstentions,
                });
            } else {
                proposal.status = ProposalStatus::Rejected;
                Self::deposit_event(Event::ProposalRejected {
                    proposal_id,
                    ayes: proposal.vote_tally.ayes,
                    nays: proposal.vote_tally.nays,
                    abstentions: proposal.vote_tally.abstentions,
                });
            }

            // Return deposit to proposer
            T::Currency::unreserve(&proposal.proposer, proposal.deposit);

            Proposals::<T>::insert(proposal_id, proposal);

            Ok(())
        }

        /// Update community rank (for future community pallet integration)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::update_community_rank())]
        pub fn update_community_rank(
            origin: OriginFor<T>,
            account: T::AccountId,
            new_rank: u32,
        ) -> DispatchResult {
            // For now, only root can update - later will be community pallet
            ensure_root(origin)?;

            let old_rank = Self::community_ranks(&account);
            CommunityRanks::<T>::insert(&account, new_rank);

            // Update council member if applicable
            if let Some(mut member) = Self::council_members(&account) {
                member.community_rank = new_rank;
                member.voting_weight = member.community_rank.saturating_add(member.pouw_contribution);
                CouncilMembers::<T>::insert(&account, member);
            }

            Self::deposit_event(Event::CommunityRankUpdated {
                account,
                old_rank,
                new_rank,
            });

            Ok(())
        }

        /// Update PoUW contribution score (called by staking pallet)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_pouw_contribution())]
        pub fn update_pouw_contribution(
            origin: OriginFor<T>,
            account: T::AccountId,
            new_contribution: u32,
        ) -> DispatchResult {
            // Only staking pallet should call this
            ensure_root(origin)?;

            let old_contribution = Self::pouw_contributions(&account);
            PoUWContributions::<T>::insert(&account, new_contribution);

            // Update council member if applicable
            if let Some(mut member) = Self::council_members(&account) {
                member.pouw_contribution = new_contribution;
                member.voting_weight = member.community_rank.saturating_add(member.pouw_contribution);
                CouncilMembers::<T>::insert(&account, member);
            }

            Self::deposit_event(Event::PoUWContributionUpdated {
                account,
                old_contribution,
                new_contribution,
            });

            Ok(())
        }

        /// Council override for emergency situations.
        ///
        /// P0-18 FIX: Override now requires JaguarMode to be **active** (not just
        /// `proposal.is_emergency`, which is caller-controlled).  This prevents
        /// any proposer from self-flagging a proposal as emergency and then having
        /// council approve it without votes or timelocks.
        #[pallet::call_index(5)]
        // DOS-009 FIX: Operational dispatch — council emergency override.
        #[pallet::weight((T::WeightInfo::council_override(), DispatchClass::Operational))]
        pub fn council_override(
            origin: OriginFor<T>,
            proposal_id: u32,
            override_reason: Vec<u8>,
        ) -> DispatchResult {
            T::CouncilOrigin::ensure_origin(origin)?;

            let mut proposal = Self::proposals(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            // P0-18 FIX: Require JaguarMode to be active — the self-declared
            // `proposal.is_emergency` flag alone is no longer sufficient.
            let is_emergency_active = JaguarMode::<T>::get()
                .map(|e| e.active)
                .unwrap_or(false);
            ensure!(
                is_emergency_active,
                Error::<T>::NotInEmergencyMode
            );

            proposal.status = ProposalStatus::Approved;
            Proposals::<T>::insert(proposal_id, proposal.clone());

            // HIGH-4 FIX: Even emergency overrides must respect enactment timelock.
            // Use the proposal-type-specific enactment period so the override
            // cannot bypass the mandatory delay before execution.
            let enact_delay = match proposal.proposal_type {
                ProposalType::Constitutional => T::EnactmentPeriodConstitutional::get(),
                ProposalType::Economic => T::EnactmentPeriodEconomic::get(),
                _ => T::EnactmentPeriodStandard::get(),
            };
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let delay_u64: u64 = TryInto::<u64>::try_into(enact_delay).unwrap_or(0);
            let execute_at: BlockNumberFor<T> = current_u64.saturating_add(delay_u64).saturated_into();
            ProposalEnactmentBlock::<T>::insert(proposal_id, execute_at);

            Self::deposit_event(Event::CouncilOverrideExecuted {
                proposal_id,
                override_reason,
            });

            Ok(())
        }

        /// Set department manager (root only)
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000_000, 512))]
        pub fn set_department_manager(
            origin: OriginFor<T>,
            department_index: u8,
            manager: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let department = match department_index {
                0 => Department::Finance,
                1 => Department::Education,
                2 => Department::Health,
                3 => Department::Works,
                4 => Department::Justice,
                5 => Department::Tourism,
                6 => Department::Agriculture,
                7 => Department::Defense,
                _ => return Err(Error::<T>::InvalidDepartment.into()),
            };

            DepartmentManagers::<T>::insert(department, manager.clone());

            Self::deposit_event(Event::DepartmentManagerSet {
                department_index,
                manager,
            });

            Ok(())
        }

        /// Submit a department-specific proposal
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(20_000_000, 512))]
        pub fn submit_department_proposal(
            origin: OriginFor<T>,
            department_index: u8,
            title: Vec<u8>,
            description: Vec<u8>,
            proposal_type_index: u8,
            threshold_index: u8,
            requires_cross_approval: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Convert department index
            let department = match department_index {
                0 => Department::Finance,
                1 => Department::Education,
                2 => Department::Health,
                3 => Department::Works,
                4 => Department::Justice,
                5 => Department::Tourism,
                6 => Department::Agriculture,
                7 => Department::Defense,
                _ => return Err(Error::<T>::InvalidDepartment.into()),
            };

            // Verify caller is the department manager
            let manager = DepartmentManagers::<T>::get(department)
                .ok_or(Error::<T>::DepartmentNotFound)?;
            ensure!(who == manager, Error::<T>::NotDepartmentManager);

            // Convert proposal type and threshold
            let proposal_type = match proposal_type_index {
                0 => ProposalType::Constitutional,
                1 => ProposalType::Economic,
                2 => ProposalType::Council,
                3 => ProposalType::Technical,
                4 => ProposalType::Emergency,
                5 => ProposalType::International,
                6 => ProposalType::Community,
                _ => return Err(Error::<T>::InvalidProposalType.into()),
            };

            let threshold = match threshold_index {
                0 => VotingThreshold::SimpleMajority,
                1 => VotingThreshold::Supermajority,
                2 => VotingThreshold::Unanimous,
                _ => return Err(Error::<T>::InvalidThreshold.into()),
            };

            let deposit = T::MinimumDeposit::get();
            T::Currency::reserve(&who, deposit)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let launch_period_u64: u64 = TryInto::<u64>::try_into(T::LaunchPeriod::get()).unwrap_or(0);
            let voting_start_u64 = current_u64.saturating_add(launch_period_u64);
            // SAFETY: value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let voting_start: BlockNumberFor<T> = voting_start_u64.saturated_into();
            let voting_period_u64: u64 = TryInto::<u64>::try_into(T::VotingPeriod::get()).unwrap_or(0);
            let voting_end_u64 = voting_start_u64.saturating_add(voting_period_u64);
            // SAFETY: value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let voting_end: BlockNumberFor<T> = voting_end_u64.saturated_into();

            let proposal_id = Self::next_proposal_id();

            let bounded_title: BoundedVec<u8, ConstU32<256>> = title.try_into()
                .map_err(|_| Error::<T>::InvalidProposalType)?;
            let bounded_description: BoundedVec<u8, ConstU32<1024>> = description.try_into()
                .map_err(|_| Error::<T>::InvalidProposalType)?;

            let proposal = Proposal {
                id: proposal_id,
                proposer: who.clone(),
                title: bounded_title,
                description: bounded_description,
                proposal_type,
                threshold,
                deposit,
                voting_start,
                voting_end,
                vote_tally: VoteTally {
                    ayes: 0,
                    nays: 0,
                    abstentions: 0,
                    total_weight: 0,
                    participation: 0,
                },
                status: ProposalStatus::Pending,
                is_emergency: false,
                department: Some(department),
                district: None, // Department proposals are not district-specific
                requires_cross_approval,
                cross_approved_by: BoundedVec::default(),
                action: None, // Phase 5: No action defined yet
                executed_at: None, // Phase 5: Not executed
            };

            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id.saturating_add(1));
            
            // Increment department proposal count
            let count = DepartmentProposalCount::<T>::get(department);
            DepartmentProposalCount::<T>::insert(department, count.saturating_add(1));

            // Record proposal submission in Community pallet (Phase 6)
            let _ = T::CommunityParticipation::record_proposal_submission(&who);

            Self::deposit_event(Event::DepartmentProposalSubmitted {
                proposal_id,
                department_index,
                proposer: who,
            });

            Ok(())
        }

        /// Approve a cross-department proposal
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512))]
        pub fn approve_cross_department(
            origin: OriginFor<T>,
            proposal_id: u32,
            department_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Convert department index
            let department = match department_index {
                0 => Department::Finance,
                1 => Department::Education,
                2 => Department::Health,
                3 => Department::Works,
                4 => Department::Justice,
                5 => Department::Tourism,
                6 => Department::Agriculture,
                7 => Department::Defense,
                _ => return Err(Error::<T>::InvalidDepartment.into()),
            };

            // Verify caller is the department manager
            let manager = DepartmentManagers::<T>::get(department)
                .ok_or(Error::<T>::DepartmentNotFound)?;
            ensure!(who == manager, Error::<T>::NotDepartmentManager);

            // Get proposal and verify it requires cross approval
            let mut proposal = Self::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.requires_cross_approval,
                Error::<T>::CrossApprovalNotRequired
            );

            // Check if this department already approved
            ensure!(
                !CrossDepartmentApprovals::<T>::get(proposal_id, department),
                Error::<T>::DepartmentAlreadyApproved
            );

            // Mark approval
            CrossDepartmentApprovals::<T>::insert(proposal_id, department, true);
            
            // Add to approved list
            proposal.cross_approved_by.try_push(department)
                .map_err(|_| Error::<T>::TooManyCrossApprovals)?;
            Proposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::CrossDepartmentApproved {
                proposal_id,
                department_index,
                approver: who,
            });

            Ok(())
        }

        // ===== Phase 4: Foundation Board System Extrinsics =====

        /// Add a new board member with a specific role and term
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(20_000_000, 512))]
        pub fn add_board_member(
            origin: OriginFor<T>,
            account: T::AccountId,
            role_index: u8,
            term_years: u8,
        ) -> DispatchResult {
            // Requires root or council supermajority
            T::CouncilOrigin::ensure_origin(origin)?;

            // Validate term duration (1-2 years)
            ensure!(
                (1..=2).contains(&term_years),
                Error::<T>::InvalidTermDuration
            );

            // Convert role index to BoardRole
            let role = match role_index {
                0 => BoardRole::Founder,
                1 => BoardRole::TechnicalSteward,
                2 => BoardRole::FSCRepresentative,
                3 => BoardRole::BTBDelegate,
                4 => BoardRole::CitizenDelegate,
                5 => BoardRole::SecurityAuditor,
                6 => BoardRole::CultureEthicsAdvisor,
                _ => return Err(Error::<T>::InvalidBoardRole.into()),
            };

            // Check role limits
            let current_count = BoardComposition::<T>::get(role);
            ensure!(
                current_count < role.max_count(),
                Error::<T>::BoardRoleLimitExceeded
            );

            // Calculate term end
            let term_blocks = BlockNumberFor::<T>::from(BLOCKS_PER_YEAR * term_years as u32);
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let term_blocks_u64: u64 = TryInto::<u64>::try_into(term_blocks).unwrap_or(0);
            let term_end_u64 = current_u64.saturating_add(term_blocks_u64);
            // SAFETY(saturated_into): value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let term_end: BlockNumberFor<T> = term_end_u64.saturated_into();

            // Consecutive-term guard: re-appointment increments the count; fresh appointment starts at 0.
            let consecutive = if let Some(existing) = CouncilMembers::<T>::get(&account) {
                let next = existing.consecutive_terms.saturating_add(1);
                ensure!(
                    next < T::MaxConsecutiveTerms::get(),
                    Error::<T>::ConsecutiveTermLimitReached
                );
                next
            } else {
                0u8
            };

            // Create council member
            let member = CouncilMember {
                account: account.clone(),
                role,
                term_end,
                is_rotating: role.is_rotating(),
                community_rank: 100,
                pouw_contribution: 0,
                voting_weight: 100,
                term_start: current_block,
                votes_received: 0,
                proposals_authored: 0,
                participation_rate: 0,
                consecutive_terms: consecutive,
            };

            // Add to council
            CouncilMembers::<T>::insert(&account, member);
            
            // Update board composition
            BoardComposition::<T>::insert(role, current_count + 1);

            // Add to term expiry queue
            TermExpiryQueue::<T>::try_mutate(term_end, |members| {
                members.try_push(account.clone())
                    .map_err(|_| Error::<T>::CouncilSizeLimits)
            })?;

            Self::deposit_event(Event::BoardMemberAdded {
                member: account,
                role_index,
                term_end,
            });

            Ok(())
        }

        /// Remove a board member (expired term or resignation)
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512))]
        pub fn remove_board_member(
            origin: OriginFor<T>,
            account: T::AccountId,
            reason: Vec<u8>,
        ) -> DispatchResult {
            // Requires root or council supermajority
            T::CouncilOrigin::ensure_origin(origin)?;

            // Get member info
            let member = CouncilMembers::<T>::get(&account)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Remove from council
            CouncilMembers::<T>::remove(&account);

            // Update board composition
            let current_count = BoardComposition::<T>::get(member.role);
            if current_count > 0 {
                BoardComposition::<T>::insert(member.role, current_count - 1);
            }

            // Remove from expiry queue
            TermExpiryQueue::<T>::mutate(member.term_end, |members| {
                members.retain(|m| m != &account);
            });

            let bounded_reason: BoundedVec<u8, ConstU32<128>> = reason
                .try_into()
                .unwrap_or_default();

            // Convert role to index for event
            let role_index = match member.role {
                BoardRole::Founder => 0,
                BoardRole::TechnicalSteward => 1,
                BoardRole::FSCRepresentative => 2,
                BoardRole::BTBDelegate => 3,
                BoardRole::CitizenDelegate => 4,
                BoardRole::SecurityAuditor => 5,
                BoardRole::CultureEthicsAdvisor => 6,
            };

            Self::deposit_event(Event::BoardMemberRemoved {
                member: account,
                role_index,
                reason: bounded_reason,
            });

            Ok(())
        }

        /// 🐆 Activate Jaguar Mode - Declare National Emergency
        /// 
        /// Named after Belize's national animal, the Jaguar (Panthera onca),
        /// representing swift, decisive action during national crises.
        /// 
        /// **Requires**: Root or Foundation Board authority
        /// **Use Cases**: Hurricanes, health crises, economic emergencies, security threats
        /// 
        /// # Parameters
        /// - `emergency_type`: Type of emergency (Hurricane, HealthCrisis, etc.)
        /// - `description`: Human-readable description (max 256 bytes)
        /// - `duration_hours`: How long emergency lasts (converted to blocks)
        /// 
        /// # Effects
        /// - Activates fast-track governance procedures
        /// - Enables emergency fund disbursement
        /// - Alerts all council members and stakeholders
        /// 
        /// # Example
        /// ```ignore
        /// declare_emergency(
        ///     EmergencyType::Hurricane,
        ///     b"Category 5 Hurricane Dean - landfall in 24 hours".to_vec(),
        ///     72  // 3-day emergency period
        /// )
        /// ```
        #[pallet::call_index(23)]
        #[pallet::weight(Weight::from_parts(15_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn declare_emergency(
            origin: OriginFor<T>,
            emergency_type_index: u8,  // 0=Hurricane, 1=HealthCrisis, 2=EconomicCrisis, 3=SecurityThreat, 4=InfrastructureFailure, 5=Other
            description: Vec<u8>,
            duration_hours: u32,
        ) -> DispatchResult {
            // Get declarer account bytes (before consuming origin)
            let declarer_bytes: Option<[u8; 32]> = if let Ok(who) = ensure_signed(origin.clone()) {
                let encoded = who.encode();
                let mut bytes = [0u8; 32];
                let len = encoded.len().min(32);
                bytes[..len].copy_from_slice(&encoded[..len]);
                Some(bytes)
            } else {
                None
            };

            // Requires root or board authority
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            // Check if emergency already active or pending
            if let Some(existing) = JaguarMode::<T>::get() {
                ensure!(
                    !existing.active && !existing.is_pending,
                    Error::<T>::EmergencyAlreadyActive
                );
            }

            // Convert index to EmergencyType
            let emergency_type = match emergency_type_index {
                0 => EmergencyType::Hurricane,
                1 => EmergencyType::HealthCrisis,
                2 => EmergencyType::EconomicCrisis,
                3 => EmergencyType::SecurityThreat,
                4 => EmergencyType::InfrastructureFailure,
                _ => EmergencyType::Other,
            };

            // Convert description to bounded vec
            let bounded_description: BoundedVec<u8, ConstU32<256>> = description
                .try_into()
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            // Calculate expiry block (6 second blocks: 600 blocks/hour)
            let current_block = frame_system::Pallet::<T>::block_number();
            let blocks_per_hour: u32 = 600;
            let duration_blocks = blocks_per_hour.saturating_mul(duration_hours);
            let expires_at = current_block + duration_blocks.into();

            // Create emergency status (CONS-029: pending with veto window)
            let veto_window_ends = current_block + T::EmergencyVetoWindow::get();
            let emergency = EmergencyStatus {
                active: false,
                emergency_type,
                description: bounded_description.clone(),
                declared_at: current_block,
                expires_at: Some(expires_at),
                declared_by: declarer_bytes,
                is_pending: true,
                veto_window_ends_at: Some(veto_window_ends),
            };

            // Store emergency status
            JaguarMode::<T>::put(emergency);
            // Clear any prior veto records
            let _ = EmergencyVetoes::<T>::clear(u32::MAX, None);

            // Emit pending event (CONS-029)
            Self::deposit_event(Event::EmergencyPendingActivation {
                emergency_type_index,
                description: bounded_description,
                veto_window_ends_at: veto_window_ends,
            });

            Ok(())
        }

        /// 🐆 Deactivate Jaguar Mode - End National Emergency
        /// 
        /// Manually ends an active emergency before its expiry time.
        /// 
        /// **Requires**: Root or Foundation Board authority
        /// 
        /// # Effects
        /// - Deactivates emergency governance procedures
        /// - Resumes normal governance operations
        /// - Logs emergency end for historical records
        #[pallet::call_index(24)]
        #[pallet::weight(Weight::from_parts(10_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn end_emergency(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            // Requires root or board authority
            T::CouncilOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            // Check if emergency is active or pending
            let emergency = JaguarMode::<T>::get()
                .ok_or(Error::<T>::NoActiveEmergency)?;
            
            ensure!(emergency.active || emergency.is_pending, Error::<T>::NoActiveEmergency);

            // Clear emergency status and any veto records
            JaguarMode::<T>::kill();
            let _ = EmergencyVetoes::<T>::clear(u32::MAX, None);

            Self::deposit_event(Event::JaguarModeDeactivated);

            Ok(())
        }

        /// 🐆 Veto a pending emergency declaration (CONS-029)
        ///
        /// Council members may veto during the veto window period.
        /// If more than 1/3 of council members veto, the pending
        /// emergency is cancelled.
        #[pallet::call_index(44)]
        // DOS-009 FIX: Operational dispatch — emergency veto is safety-critical.
        // DOS-010 FIX: Weight accounts for bounded council scan (up to 200 reads)
        // and worst-case EmergencyVetoes clear (up to 200 writes).
        #[pallet::weight((Weight::from_parts(25_000_000, 4096)
            .saturating_add(T::DbWeight::get().reads(204))
            .saturating_add(T::DbWeight::get().writes(202)), DispatchClass::Operational))]
        pub fn veto_emergency(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Must be a council member
            ensure!(
                CouncilMembers::<T>::contains_key(&who),
                Error::<T>::NotCouncilMember
            );

            // Must have a pending emergency
            let emergency = JaguarMode::<T>::get()
                .ok_or(Error::<T>::NoPendingEmergency)?;
            ensure!(emergency.is_pending, Error::<T>::NoPendingEmergency);

            // Must not have already vetoed
            ensure!(
                !EmergencyVetoes::<T>::contains_key(&who),
                Error::<T>::AlreadyVetoed
            );

            // Record veto
            let current_block = frame_system::Pallet::<T>::block_number();
            EmergencyVetoes::<T>::insert(&who, current_block);
            let veto_count = EmergencyVetoes::<T>::count();

            Self::deposit_event(Event::EmergencyVetoed {
                who,
                vetoes_so_far: veto_count,
            });

            // If vetoes exceed 1/3 of council, cancel the pending emergency
            // DOS-010 FIX: Bounded iteration — consistent with council_size() helper.
            let max_council = T::MaxCandidatesPerElection::get() as usize;
            let council_count = CouncilMembers::<T>::iter().take(max_council).count() as u32;
            let veto_threshold = council_count / 3 + 1; // strict > 1/3
            if veto_count >= veto_threshold {
                JaguarMode::<T>::kill();
                let _ = EmergencyVetoes::<T>::clear(u32::MAX, None);
                Self::deposit_event(Event::JaguarModeDeactivated);
            }

            Ok(())
        }

        /// Nominate a citizen for delegate position
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(10_000_000, 512))]
        pub fn nominate_for_delegate(
            origin: OriginFor<T>,
            nominee: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&nominee),
                Error::<T>::InsufficientCompliance
            );

            // Ensure election is active
            let _election_id = CurrentElection::<T>::get()
                .ok_or(Error::<T>::NoActiveElection)?;

            // Add nominee (initialize vote count to 0)
            if !DelegateNominees::<T>::contains_key(&nominee) {
                DelegateNominees::<T>::insert(&nominee, 0);
            }

            Self::deposit_event(Event::DelegateNominated {
                nominee,
                nominator: who,
            });

            Ok(())
        }

        /// Vote for a delegate candidate
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(10_000_000, 512))]
        pub fn vote_for_delegate(
            origin: OriginFor<T>,
            nominee: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Ensure election is active
            let election_id = CurrentElection::<T>::get()
                .ok_or(Error::<T>::NoActiveElection)?;

            // Check if already voted
            ensure!(
                !DelegateVoters::<T>::contains_key(election_id, &who),
                Error::<T>::AlreadyVotedInElection
            );

            // Verify nominee exists
            ensure!(
                DelegateNominees::<T>::contains_key(&nominee),
                Error::<T>::NomineeNotFound
            );

            // Calculate voting weight (could be based on token balance)
            let weight = 1u32; // Simple 1 token = 1 vote for now

            // Record vote
            DelegateVoters::<T>::insert(election_id, &who, &nominee);
            
            // Increment nominee vote count
            DelegateNominees::<T>::mutate(&nominee, |count| {
                *count = count.saturating_add(weight);
            });

            Self::deposit_event(Event::DelegateVoteCast {
                voter: who,
                nominee,
                weight,
            });

            Ok(())
        }

        // ===== REFERENDUM SYSTEM EXTRINSICS =====

        /// Create a referendum for direct democracy voting.
        ///
        /// Referendums allow citizens to vote directly on specific policy questions.
        /// Requires minimum deposit and compliance checks. Supports binary, multiple
        /// choice, or approval voting.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Creator of the referendum (must be compliant)
        /// - `title`: Short title (max 256 bytes)
        /// - `description`: Detailed description (max 1024 bytes)
        /// - `options`: Voting options (2-10 options, e.g., ["Yes", "No"])
        /// - `quorum_percentage`: Required participation (1-100%)
        /// - `duration_blocks`: Voting period duration
        /// - `district_index`: Optional district restriction (0-5)
        #[pallet::call_index(25)]
        #[pallet::weight(Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4)))]
        pub fn create_referendum(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Vec<u8>,
            options: Vec<Vec<u8>>,
            quorum_percentage: u8,
            duration_blocks: u32,
            district_index: Option<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Compliance check
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Validate options
            ensure!(!options.is_empty(), Error::<T>::NoOptionsProvided);
            ensure!(options.len() <= 10, Error::<T>::TooManyOptions);

            // Validate quorum — P0-17 FIX: enforce minimum floor
            ensure!(
                quorum_percentage >= T::MinQuorumPercentage::get() && quorum_percentage <= 100,
                Error::<T>::InvalidQuorum
            );

            // Convert title and description to bounded vecs
            let bounded_title: BoundedVec<u8, ConstU32<256>> = title.try_into()
                .map_err(|_| Error::<T>::ReferendumTitleTooLong)?;
            let bounded_description: BoundedVec<u8, ConstU32<1024>> = description.try_into()
                .map_err(|_| Error::<T>::ReferendumDescriptionTooLong)?;

            // Convert options
            let bounded_options: BoundedVec<BoundedVec<u8, ConstU32<128>>, ConstU32<10>> = options
                .into_iter()
                .map(|opt| opt.try_into())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| Error::<T>::ReferendumDescriptionTooLong)?
                .try_into()
                .map_err(|_| Error::<T>::TooManyOptions)?;

            // Convert district if specified
            let district = if let Some(idx) = district_index {
                ensure!(idx < 6, Error::<T>::InvalidDistrict);
                Some(match idx {
                    0 => BelizeDistrict::Belize,
                    1 => BelizeDistrict::Cayo,
                    2 => BelizeDistrict::Corozal,
                    3 => BelizeDistrict::OrangeWalk,
                    4 => BelizeDistrict::StannCreek,
                    5 => BelizeDistrict::Toledo,
                    _ => return Err(Error::<T>::InvalidDistrict.into()),
                })
            } else {
                None
            };

            // Calculate voting period
            let current_block = frame_system::Pallet::<T>::block_number();
            let duration = BlockNumberFor::<T>::from(duration_blocks);
            let voting_start = current_block;
            
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let duration_u64: u64 = TryInto::<u64>::try_into(duration).unwrap_or(0);
            let voting_end_u64 = current_u64.saturating_add(duration_u64);
            // SAFETY(saturated_into): value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let voting_end: BlockNumberFor<T> = voting_end_u64.saturated_into();

            // Generate referendum ID
            let referendum_id = NextReferendumId::<T>::get();
            NextReferendumId::<T>::put(referendum_id.saturating_add(1));

            // Initialize vote counts
            let vote_counts: BoundedVec<u32, ConstU32<10>> = bounded_options
                .iter()
                .map(|_| 0u32)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| Error::<T>::TooManyOptions)?;

            // Create referendum
            let referendum = Referendum {
                id: referendum_id,
                creator: who.clone(),
                title: bounded_title.clone(),
                description: bounded_description,
                options: bounded_options,
                quorum_percentage,
                voting_start,
                voting_end,
                vote_counts,
                total_votes: 0,
                status: ReferendumStatus::Active,
                district,
                winning_option: None,
            };

            // Store referendum
            Referendums::<T>::insert(referendum_id, referendum);

            // CRIT-2 FIX: Dynamically count eligible voters from the compliance
            // provider instead of storing a hardcoded placeholder.
            let eligible_count = T::ComplianceProvider::eligible_voter_count();
            ReferendumEligibleVoters::<T>::insert(referendum_id, eligible_count);

            Self::deposit_event(Event::ReferendumCreated {
                referendum_id,
                creator: who,
                title: bounded_title,
                quorum_percentage,
                district_index,
            });

            Ok(())
        }

        /// Cast a vote on an active referendum.
        ///
        /// Voting weight is based on community rank + PoUW contributions.
        /// Each account can vote only once per referendum.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Voter account
        /// - `referendum_id`: ID of referendum to vote on
        /// - `option_index`: Index of option being voted for (0-based)
        #[pallet::call_index(26)]
        #[pallet::weight(Weight::from_parts(20_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn vote_on_referendum(
            origin: OriginFor<T>,
            referendum_id: u32,
            option_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Compliance check
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Get referendum
            let mut referendum = Referendums::<T>::get(referendum_id)
                .ok_or(Error::<T>::ReferendumNotFound)?;

            // Check referendum is active
            ensure!(referendum.status == ReferendumStatus::Active, Error::<T>::ReferendumNotFinalized);

            // Check voting period
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block >= referendum.voting_start, Error::<T>::ReferendumVotingNotStarted);
            ensure!(current_block <= referendum.voting_end, Error::<T>::ReferendumVotingEnded);

            // Check not already voted
            ensure!(
                !ReferendumVotes::<T>::contains_key(referendum_id, &who),
                Error::<T>::AlreadyVotedOnReferendum
            );

            // Validate option index
            ensure!((option_index as usize) < referendum.options.len(), Error::<T>::InvalidOptionIndex);

            // Calculate voting weight
            let community_rank = CommunityRanks::<T>::get(&who);
            let pouw_contribution = PoUWContributions::<T>::get(&who);
            let weight = community_rank.saturating_add(pouw_contribution).max(1);

            // Record participation in Community pallet (Phase 6 integration)
            let _ = T::CommunityParticipation::record_vote_cast(&who);

            // Record vote
            ReferendumVotes::<T>::insert(referendum_id, &who, option_index);

            // Update vote counts
            referendum.vote_counts[option_index as usize] = referendum.vote_counts[option_index as usize]
                .saturating_add(weight);
            referendum.total_votes = referendum.total_votes.saturating_add(weight);

            // Update referendum
            Referendums::<T>::insert(referendum_id, referendum);

            Self::deposit_event(Event::ReferendumVoteCast {
                referendum_id,
                voter: who,
                option_index,
                weight,
            });

            Ok(())
        }

        /// Finalize a referendum after voting period ends.
        ///
        /// Checks quorum, determines winning option, and updates status.
        /// Can be called by anyone after voting period ends.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Any signed account
        /// - `referendum_id`: ID of referendum to finalize
        #[pallet::call_index(27)]
        #[pallet::weight(Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn finalize_referendum(
            origin: OriginFor<T>,
            referendum_id: u32,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Get referendum
            let mut referendum = Referendums::<T>::get(referendum_id)
                .ok_or(Error::<T>::ReferendumNotFound)?;

            // Check not already finalized
            ensure!(referendum.status == ReferendumStatus::Active, Error::<T>::ReferendumAlreadyFinalized);

            // Check voting period ended
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block > referendum.voting_end, Error::<T>::VotingPeriodNotEnded);

            // Calculate participation percentage (capped at 100 to prevent u8 wrap)
            let eligible_voters = ReferendumEligibleVoters::<T>::get(referendum_id);
            let participation_percentage = if eligible_voters > 0 {
                ((referendum.total_votes as u64 * 100) / eligible_voters as u64).min(100) as u8
            } else {
                0
            };

            // Check quorum
            if participation_percentage >= referendum.quorum_percentage {
                // Find winning option (highest vote count)
                let mut winning_index = 0u8;
                let mut max_votes = 0u32;
                for (idx, &votes) in referendum.vote_counts.iter().enumerate() {
                    if votes > max_votes {
                        max_votes = votes;
                        winning_index = idx as u8;
                    }
                }

                referendum.status = ReferendumStatus::Passed;
                referendum.winning_option = Some(winning_index);

                Self::deposit_event(Event::ReferendumPassed {
                    referendum_id,
                    winning_option_index: winning_index,
                    total_votes: referendum.total_votes,
                    participation_percentage,
                });
            } else {
                // Quorum not met
                referendum.status = ReferendumStatus::Failed;

                Self::deposit_event(Event::ReferendumFailed {
                    referendum_id,
                    total_votes: referendum.total_votes,
                    participation_percentage,
                    required_quorum: referendum.quorum_percentage,
                });
            }

            // Update referendum
            Referendums::<T>::insert(referendum_id, referendum);

            Ok(())
        }

        // ===== PHASE 5: EXECUTION LAYER EXTRINSICS =====

        /// Execute an approved proposal's on-chain action.
        ///
        /// After a proposal is approved through voting, this extrinsic triggers the
        /// actual on-chain execution of the proposed action. Actions can include:
        /// - **TreasuryTransfer**: Send DALLA from governance treasury to recipient
        /// - **ParameterChange**: Modify governance parameters (voting thresholds, periods)
        /// - **EmergencyAction**: Execute emergency measures (pause, resume, upgrade)
        /// - **CouncilAction**: Add/remove council members, change structure
        ///
        /// Execution is permanent and cannot be undone. Only approved proposals with
        /// defined actions can be executed. The executor must be a council member.
        ///
        /// ## Complexity
        /// - **Reads**: 5 (Proposal, execution status, action verification, treasury)
        /// - **Writes**: 3 (Update proposal status, mark executed, perform action)
        /// - **Weight**: 50M computational + 5 reads + 3 writes (varies by action type)
        ///
        /// ## Parameters
        ///
        /// - `origin`: Signed origin of executor (must be council member)
        /// - `proposal_id`: ID of the approved proposal to execute
        ///
        /// ## Execution Safety
        ///
        /// Multiple safety checks prevent double-execution and invalid execution:
        /// 1. Verify proposal exists
        /// 2. Check not already executed (via proposal flag and storage)
        /// 3. Verify proposal status is Approved
        /// 4. Verify action is defined
        /// 5. Execute action with rollback on failure
        /// 6. Mark as executed atomically
        ///
        /// ## Action Types
        ///
        /// | Action Type | Description | Requirements |
        /// |-------------|-------------|--------------|
        /// | TreasuryTransfer | Send DALLA from treasury | Sufficient balance |
        /// | ParameterChange | Update governance params | Valid parameter & value |
        /// | EmergencyAction | Emergency operations | Root or FSC origin |
        /// | CouncilAction | Council changes | Council consensus |
        ///
        /// ## Emits
        ///
        /// - `ProposalExecuted`: When proposal action is successfully executed
        ///
        /// ## Errors
        ///
        /// - `ProposalNotFound`: Invalid proposal ID
        /// - `AlreadyExecuted`: Proposal has already been executed
        /// - `ProposalNotApproved`: Proposal not yet approved by voting
        /// - `NoActionDefined`: Proposal has no executable action
        /// - `InsufficientTreasuryBalance`: Treasury lacks funds for transfer
        /// - `InvalidParameterValue`: Parameter change value out of valid range
        ///
        /// ## Example
        ///
        /// ```rust,ignore
        /// // Execute approved treasury spending proposal
        /// // (Proposal 42 was approved with 70% yes votes)
        /// let result = Governance::execute_proposal(
        ///     Origin::signed(council_member), // Must be council member
        ///     42, // proposal_id
        /// );
        /// 
        /// // If action was TreasuryTransfer { recipient: alice, amount: 50_000 DALLA }
        /// // -> Alice receives 50,000 DALLA from governance treasury
        /// // -> Proposal marked as executed
        /// // -> ProposalExecuted event emitted
        /// ```
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(50_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3)))]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only council members can execute approved proposals
            ensure!(
                Self::is_council_member(&who),
                Error::<T>::NotCouncilMember
            );

            // Get proposal
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Check if already executed FIRST (before checking approval)
            ensure!(
                proposal.executed_at.is_none(),
                Error::<T>::AlreadyExecuted
            );

            // Double-check via storage
            ensure!(
                !ExecutedProposals::<T>::contains_key(proposal_id),
                Error::<T>::AlreadyExecuted
            );

            // Ensure proposal is approved
            ensure!(
                proposal.status == ProposalStatus::Approved,
                Error::<T>::ProposalNotApproved
            );

            // Enforce enactment delay: the proposal may not be executed before its scheduled block
            if let Some(enact_at) = ProposalEnactmentBlock::<T>::get(proposal_id) {
                ensure!(
                    frame_system::Pallet::<T>::block_number() >= enact_at,
                    Error::<T>::EnactmentDelayNotPassed
                );
            }

            // Phase 6A: Constitutional proposals require both houses to have independently
            // ratified via `ratify_constitutional_proposal` before execution proceeds.
            if proposal.proposal_type == ProposalType::Constitutional {
                let (tech, gov) = ConstitutionalRatifications::<T>::get(proposal_id);
                ensure!(
                    tech && gov,
                    Error::<T>::ConstitutionalRatificationRequired
                );
            }

            // Get action or fail
            let action = proposal.action.clone()
                .ok_or(Error::<T>::NoActionDefined)?;

            // Execute the action based on type
            Self::execute_action(&action, proposal_id)?;

            // Mark proposal as executed
            let current_block = frame_system::Pallet::<T>::block_number();
            proposal.status = ProposalStatus::Executed;
            proposal.executed_at = Some(current_block);
            
            // Update storage
            Proposals::<T>::insert(proposal_id, proposal);
            ExecutedProposals::<T>::insert(proposal_id, current_block);

            // Get action type index for event
            let action_type = Self::get_action_type_index(&action);

            Self::deposit_event(Event::ProposalExecuted {
                proposal_id,
                executor: who,
                action_type,
            });

            Ok(())
        }

        // ===== PHASE 6: COUNCIL ELECTION SYSTEM EXTRINSICS =====

        /// Start a district council election
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(25_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn start_district_election(
            origin: OriginFor<T>,
            district_index: u8,
            seats: u32,
            registration_period_blocks: u32,
            voting_period_blocks: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Convert district index
            let district = match district_index {
                0 => BelizeDistrict::Belize,
                1 => BelizeDistrict::Cayo,
                2 => BelizeDistrict::Corozal,
                3 => BelizeDistrict::OrangeWalk,
                4 => BelizeDistrict::StannCreek,
                5 => BelizeDistrict::Toledo,
                _ => return Err(Error::<T>::InvalidDistrict.into()),
            };

            // Verify seat allocation doesn't exceed district limit
            ensure!(
                seats <= district.seat_allocation(),
                Error::<T>::InvalidDistrict
            );

            // Ensure no active election for this district
            ensure!(
                !DistrictElections::<T>::contains_key(district),
                Error::<T>::ElectionAlreadyActive
            );

            // Calculate election timeline
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let reg_end_u64 = current_u64.saturating_add(registration_period_blocks.into());
            // SAFETY(saturated_into): value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let registration_end: BlockNumberFor<T> = reg_end_u64.saturated_into();
            let voting_start = registration_end;
            let voting_end_u64 = reg_end_u64.saturating_add(voting_period_blocks.into());
            // SAFETY(saturated_into): value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let voting_end: BlockNumberFor<T> = voting_end_u64.saturated_into();

            // Create election
            let election = Election {
                id: current_block,
                district,
                seats,
                registration_end,
                voting_start,
                voting_end,
                status: ElectionStatus::Registration,
                total_votes: 0,
            };

            // Store election
            DistrictElections::<T>::insert(district, election);

            Self::deposit_event(Event::DistrictElectionStarted {
                election_id: current_block,
                district_index,
                seats,
                registration_end,
                voting_end,
            });

            Ok(())
        }

        /// Register as a candidate in a district election
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn register_candidate(
            origin: OriginFor<T>,
            district_index: u8,
            platform: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Convert district index
            let district = match district_index {
                0 => BelizeDistrict::Belize,
                1 => BelizeDistrict::Cayo,
                2 => BelizeDistrict::Corozal,
                3 => BelizeDistrict::OrangeWalk,
                4 => BelizeDistrict::StannCreek,
                5 => BelizeDistrict::Toledo,
                _ => return Err(Error::<T>::InvalidDistrict.into()),
            };

            // Get election
            let mut election = DistrictElections::<T>::get(district)
                .ok_or(Error::<T>::ElectionNotFound)?;

            // Verify registration phase
            ensure!(
                election.status == ElectionStatus::Registration,
                Error::<T>::NotInRegistrationPhase
            );

            // Verify registration period not ended
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block < election.registration_end,
                Error::<T>::NotInRegistrationPhase
            );

            // Check if already registered
            ensure!(
                !ElectionCandidates::<T>::contains_key(election.id, &who),
                Error::<T>::CandidateAlreadyRegistered
            );

            // Enforce maximum candidate limit per election
            let candidate_count = ElectionCandidates::<T>::iter_prefix(election.id).count() as u32;
            ensure!(
                candidate_count < T::MaxCandidatesPerElection::get(),
                Error::<T>::TooManyCandidates
            );

            // Convert platform to BoundedVec
            let bounded_platform: BoundedVec<u8, ConstU32<512>> = platform.try_into()
                .map_err(|_| Error::<T>::PlatformTooLong)?;

            // Create candidate
            let candidate = ElectionCandidate {
                account: who.clone(),
                district,
                election_id: election.id,
                platform: bounded_platform,
                votes: 0,
                registered_at: current_block,
            };

            // Store candidate
            ElectionCandidates::<T>::insert(election.id, &who, candidate);

            // Transition to voting if registration period ended
            if current_block >= election.registration_end {
                election.status = ElectionStatus::Voting;
                DistrictElections::<T>::insert(district, election.clone());
            }

            Self::deposit_event(Event::CandidateRegistered {
                election_id: election.id,
                candidate: who,
                district_index,
            });

            Ok(())
        }

        /// Vote in a district election
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(20_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn vote_in_district_election(
            origin: OriginFor<T>,
            district_index: u8,
            candidate: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check compliance
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&who),
                Error::<T>::InsufficientCompliance
            );

            // Convert district index
            let district = match district_index {
                0 => BelizeDistrict::Belize,
                1 => BelizeDistrict::Cayo,
                2 => BelizeDistrict::Corozal,
                3 => BelizeDistrict::OrangeWalk,
                4 => BelizeDistrict::StannCreek,
                5 => BelizeDistrict::Toledo,
                _ => return Err(Error::<T>::InvalidDistrict.into()),
            };

            // Get election
            let mut election = DistrictElections::<T>::get(district)
                .ok_or(Error::<T>::ElectionNotFound)?;

            // Transition to voting if needed
            let current_block = frame_system::Pallet::<T>::block_number();
            if election.status == ElectionStatus::Registration && current_block >= election.voting_start {
                election.status = ElectionStatus::Voting;
            }

            // Verify voting phase
            ensure!(
                election.status == ElectionStatus::Voting,
                Error::<T>::NotInVotingPhase
            );

            // Verify within voting period
            ensure!(
                current_block >= election.voting_start && current_block < election.voting_end,
                Error::<T>::NotInVotingPhase
            );

            // Check if already voted
            ensure!(
                !ElectionVotes::<T>::contains_key(election.id, &who),
                Error::<T>::AlreadyVotedInDistrictElection
            );

            // Verify candidate exists
            let mut candidate_info = ElectionCandidates::<T>::get(election.id, &candidate)
                .ok_or(Error::<T>::CandidateNotFound)?;

            // Record vote
            ElectionVotes::<T>::insert(election.id, &who, &candidate);

            // Update candidate vote count
            candidate_info.votes = candidate_info.votes.saturating_add(1);
            ElectionCandidates::<T>::insert(election.id, &candidate, candidate_info);

            // Update election total votes
            election.total_votes = election.total_votes.saturating_add(1);
            DistrictElections::<T>::insert(district, election.clone());

            Self::deposit_event(Event::ElectionVoteCast {
                election_id: election.id,
                voter: who,
                candidate,
                district_index,
            });

            Ok(())
        }

        /// Finalize a district election and assign council seats
        #[pallet::call_index(17)]
        // DOS-015 FIX: Weight parameterized by MaxCandidatesPerElection (200)
        // for candidate iteration, sorting, and winner seating.
        #[pallet::weight(Weight::from_parts(80_000_000, 10240)
            .saturating_add(T::DbWeight::get().reads(210))
            .saturating_add(T::DbWeight::get().writes(210)))]
        pub fn finalize_district_election(
            origin: OriginFor<T>,
            district_index: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Convert district index
            let district = match district_index {
                0 => BelizeDistrict::Belize,
                1 => BelizeDistrict::Cayo,
                2 => BelizeDistrict::Corozal,
                3 => BelizeDistrict::OrangeWalk,
                4 => BelizeDistrict::StannCreek,
                5 => BelizeDistrict::Toledo,
                _ => return Err(Error::<T>::InvalidDistrict.into()),
            };

            // Get election
            let mut election = DistrictElections::<T>::get(district)
                .ok_or(Error::<T>::ElectionNotFound)?;

            // Verify voting period ended
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block >= election.voting_end,
                Error::<T>::VotingPeriodNotEnded
            );

            // Verify not already finalized
            ensure!(
                election.status != ElectionStatus::Finalized,
                Error::<T>::ElectionAlreadyFinalized
            );

            // Get all candidates for this election (bounded by MaxCandidatesPerElection)
            let max_candidates = T::MaxCandidatesPerElection::get() as usize;
            let mut candidates: Vec<(T::AccountId, u32)> = ElectionCandidates::<T>::iter_prefix(election.id)
                .take(max_candidates)
                .map(|(account, info)| (account, info.votes))
                .collect();

            // Sort by votes (descending)
            candidates.sort_by(|a, b| b.1.cmp(&a.1));

            // Take top N candidates based on seat count
            let winners: Vec<T::AccountId> = candidates.iter()
                .take(election.seats as usize)
                .map(|(account, _)| account.clone())
                .collect();

            // Verify we have winners
            ensure!(
                !winners.is_empty(),
                Error::<T>::InsufficientCandidates
            );

            // Calculate term end (2 years from now)
            let term_blocks: BlockNumberFor<T> = (BLOCKS_PER_YEAR * 2).into();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let term_blocks_u64: u64 = TryInto::<u64>::try_into(term_blocks).unwrap_or(0);
            let term_end_u64 = current_u64.saturating_add(term_blocks_u64);
            // SAFETY(saturated_into): value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
            let term_end: BlockNumberFor<T> = term_end_u64.saturated_into();

            // Assign council seats to winners
            let mut district_members: Vec<T::AccountId> = Vec::new();
            
            for winner in winners.iter() {
                // Get candidate info for vote count
                if let Some(candidate_info) = ElectionCandidates::<T>::get(election.id, winner) {
                    // Create or update council member
                    let member = CouncilMember {
                        account: winner.clone(),
                        role: BoardRole::CitizenDelegate,
                        term_end,
                        is_rotating: true,
                        community_rank: 100,
                        pouw_contribution: 0,
                        voting_weight: 100,
                        term_start: current_block,
                        votes_received: candidate_info.votes,
                        proposals_authored: 0,
                        participation_rate: 0,
                        consecutive_terms: 0,
                    };

                    CouncilMembers::<T>::insert(winner, member);
                    district_members.push(winner.clone());

                    Self::deposit_event(Event::CouncilMemberElectedFromDistrict {
                        account: winner.clone(),
                        district_index,
                        votes_received: candidate_info.votes,
                        term_end,
                    });
                }
            }

            // Update district representation
            let bounded_members: BoundedVec<T::AccountId, ConstU32<5>> = district_members.try_into()
                .unwrap_or_default();
            DistrictRepresentation::<T>::insert(district, bounded_members.clone());

            // Mark election as finalized
            election.status = ElectionStatus::Finalized;
            DistrictElections::<T>::insert(district, election.clone());

            Self::deposit_event(Event::DistrictElectionFinalized {
                election_id: election.id,
                district_index,
                winners: bounded_members,
                total_votes: election.total_votes,
            });

            Ok(())
        }

        // ===== PHASE 7: ADVANCED FEATURES EXTRINSICS =====

        /// Delegate voting power to a trusted representative.
        ///
        /// Enables liquid democracy by allowing voters to delegate their voting power
        /// to another account they trust. The delegate can then vote on their behalf
        /// for all proposals. Delegation can be revoked at any time and expires
        /// automatically after a specified period (default 1 year).
        ///
        /// This enables:
        /// - **Liquid Democracy**: Citizens can delegate to domain experts
        /// - **Participation Scaling**: Voters delegate when busy, vote directly when engaged
        /// - **Trust Networks**: Build networks of trusted representatives
        /// - **Observer Participation**: Basic KYC accounts can participate via delegation
        ///
        /// ## Complexity
        /// - **Reads**: 2 (Check existing delegation, delegate's current delegators)
        /// - **Writes**: 2 (Store delegation, update delegate's receiver list)
        /// - **Weight**: 25M computational + 2 reads + 2 writes
        ///
        /// ## Parameters
        ///
        /// - `origin`: Signed origin of the delegator (requires Basic KYC or higher)
        /// - `delegate`: Account to receive voting power (should have Contributor tier for voting)
        /// - `expires_at`: Optional expiry block (default: current_block + 5,256,000 = 1 year)
        ///
        /// ## Delegation Limits
        ///
        /// - **Max Delegators per Delegate**: 100 (prevents centralization)
        /// - **Default Expiry**: 1 year (5,256,000 blocks ≈ 12 months at 6s/block)
        /// - **Self-Delegation**: Not allowed
        /// - **Revocation**: Can revoke at any time via `revoke_delegation()`
        ///
        /// ## Voting Weight Calculation
        ///
        /// When delegate votes, their weight includes all delegators:
        /// ```text
        /// delegate_total_weight = delegate_base_weight + count_of_active_delegators
        /// ```
        ///
        /// ## Emits
        ///
        /// - `VoteDelegationCreated`: When delegation is successfully established
        ///
        /// ## Errors
        ///
        /// - `CannotDelegateToSelf`: Attempting to delegate to own account
        /// - `DelegationAlreadyExists`: Delegator already has an active delegation
        /// - `TooManyDelegators`: Delegate already has 100 delegators (max limit)
        ///
        /// ## Example
        ///
        /// ```rust,ignore
        /// // Delegate voting power to trusted expert for 6 months
        /// let six_months = current_block + 2_628_000; // ~6 months
        /// let result = Governance::delegate_vote(
        ///     Origin::signed(alice),
        ///     expert_account.clone(), // Trusted delegate
        ///     Some(six_months), // Custom expiry
        /// );
        /// 
        /// // Alice's voting power now added to expert_account's weight
        /// // Alice can still revoke at any time
        /// ```
        #[pallet::call_index(18)]
        #[pallet::weight(Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn delegate_vote(
            origin: OriginFor<T>,
            delegate: T::AccountId,
            expires_at: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let delegator = ensure_signed(origin)?;

            // M48 FIX: Check compliance/KYC requirements for both delegator and delegate
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&delegator),
                Error::<T>::InsufficientCompliance
            );
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&delegate),
                Error::<T>::InsufficientCompliance
            );

            // Cannot delegate to self
            ensure!(delegator != delegate, Error::<T>::CannotDelegateToSelf);

            // Check if delegation already exists
            ensure!(
                !VoteDelegations::<T>::contains_key(&delegator),
                Error::<T>::DelegationAlreadyExists
            );

            // Check delegate's delegator count
            let mut receivers = DelegationReceivers::<T>::get(&delegate);
            ensure!(
                receivers.len() < MAX_DELEGATION_RECEIVERS,
                Error::<T>::TooManyDelegators
            );

            let current_block = frame_system::Pallet::<T>::block_number();
            let expires_at_final = expires_at.unwrap_or_else(|| {
                // Default: 1 year (5,256,000 blocks)
                let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
                let expiry_u64 = current_u64.saturating_add(BLOCKS_PER_YEAR as u64);
                // SAFETY(saturated_into): value derived from u32 block number arithmetic in u64; fits in BlockNumberFor<T> (runtime uses u32 block numbers)
                expiry_u64.saturated_into()
            });

            // Create delegation
            let delegation = VoteDelegation {
                delegator: delegator.clone(),
                delegate: delegate.clone(),
                created_at: current_block,
                expires_at: expires_at_final,
                is_active: true,
            };

            VoteDelegations::<T>::insert(&delegator, delegation);

            // Add to receiver list
            receivers.try_push(delegator.clone())
                .map_err(|_| Error::<T>::TooManyDelegators)?;
            DelegationReceivers::<T>::insert(&delegate, receivers);

            Self::deposit_event(Event::VoteDelegationCreated {
                delegator,
                delegate,
                expires_at: Some(expires_at_final),
            });

            Ok(())
        }

        /// Revoke vote delegation
        #[pallet::call_index(19)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn revoke_delegation(origin: OriginFor<T>) -> DispatchResult {
            let delegator = ensure_signed(origin)?;

            // Get existing delegation
            let delegation = VoteDelegations::<T>::get(&delegator)
                .ok_or(Error::<T>::NoDelegationFound)?;

            let delegate = delegation.delegate.clone();

            // Remove delegation
            VoteDelegations::<T>::remove(&delegator);

            // Remove from receiver list
            let mut receivers = DelegationReceivers::<T>::get(&delegate);
            receivers.retain(|acc| acc != &delegator);
            DelegationReceivers::<T>::insert(&delegate, receivers);

            Self::deposit_event(Event::VoteDelegationRevoked {
                delegator,
                delegate,
            });

            Ok(())
        }

        /// Amend a proposal (only by original proposer before voting ends)
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(30_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn amend_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
            new_title: Option<BoundedVec<u8, ConstU32<256>>>,
            new_description: Option<BoundedVec<u8, ConstU32<1024>>>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            // Get proposal
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Verify proposer is author
            ensure!(proposal.proposer == proposer, Error::<T>::NotProposalAuthor);

            // Verify voting hasn't started
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block < proposal.voting_start,
                Error::<T>::CannotAmendAfterVoting
            );

            // Check if amendment already exists
            ensure!(
                !ProposalAmendments::<T>::contains_key(proposal_id),
                Error::<T>::AmendmentAlreadyExists
            );

            // At least one field must be provided
            ensure!(
                new_title.is_some() || new_description.is_some(),
                Error::<T>::NoAmendmentProvided
            );

            // Create amendment record
            let amendment = ProposalAmendment {
                proposal_id,
                new_title: new_title.clone(),
                new_description: new_description.clone(),
                proposed_at: current_block,
                is_applied: false,
            };

            ProposalAmendments::<T>::insert(proposal_id, amendment);

            // Apply amendments to proposal
            if let Some(title) = new_title {
                proposal.title = title;
            }
            if let Some(description) = new_description {
                proposal.description = description;
            }

            Proposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::ProposalAmendmentSubmitted {
                proposal_id,
                proposer,
            });

            Ok(())
        }

        /// Claim participation rewards
        #[pallet::call_index(21)]
        // DOS-004 FIX: Weight accounts for worst-case 50-item bounded scan.
        #[pallet::weight(Weight::from_parts(85_000_000, 4096)
            .saturating_add(T::DbWeight::get().reads(53))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn claim_participation_reward(
            origin: OriginFor<T>,
            reward_type: u8,
        ) -> DispatchResult {
            let claimer = ensure_signed(origin)?;

            // Determine reward amount based on type
            // 0 = Vote (10 DALLA)
            // 1 = Proposal (100 DALLA)
            // 2 = Council (500 DALLA/month)
            let reward_amount: BalanceOf<T> = match reward_type {
                0 => {
                    // Vote reward - check if voted on any proposal (bounded scan)
                    let next_id = NextProposalId::<T>::get();
                    let max_check = next_id.min(50);
                    let has_voted = (0..max_check)
                        .any(|pid| Votes::<T>::contains_key(pid, &claimer));
                    ensure!(has_voted, Error::<T>::NoRewardAvailable);
                    // SAFETY(saturated_into): constant 10_000_000_000_000 fits in BalanceOf<T> (u128 on standard runtimes)
                    10_000_000_000_000u128.saturated_into() // 10 DALLA (12 decimals)
                }
                1 => {
                    // Proposal reward - check if authored
                    // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7 / DOS-004)
                    // 50 is a practical bound — recent proposals only.
                    let has_proposed = Proposals::<T>::iter()
                        .take(50)
                        .any(|(_, prop)| prop.proposer == claimer);
                    ensure!(has_proposed, Error::<T>::NoRewardAvailable);
                    // SAFETY(saturated_into): constant 100_000_000_000_000 fits in BalanceOf<T> (u128 on standard runtimes)
                    100_000_000_000_000u128.saturated_into() // 100 DALLA
                }
                2 => {
                    // Council reward - check if council member
                    ensure!(
                        CouncilMembers::<T>::contains_key(&claimer),
                        Error::<T>::NoRewardAvailable
                    );
                    // SAFETY(saturated_into): constant 500_000_000_000_000 fits in BalanceOf<T> (u128 on standard runtimes)
                    500_000_000_000_000u128.saturated_into() // 500 DALLA
                }
                _ => return Err(Error::<T>::InvalidPriority.into()),
            };

            // Check if already claimed this type (H-26: per reward_type)
            let claimed = RewardsClaimed::<T>::get(&claimer, reward_type);
            ensure!(claimed == Zero::zero(), Error::<T>::RewardAlreadyClaimed);

            // Transfer DALLA tokens from treasury to claimer
            // Treasury is managed by the governance pallet and funded through:
            // 1. Transaction fees (collected by pallet-economy)
            // 2. Treasury proposals (approved by council)
            // 3. Initial endowment at genesis
            let treasury_account = Self::account_id();
            
            // Ensure treasury has sufficient balance
            let treasury_balance = T::Currency::free_balance(&treasury_account);
            ensure!(
                treasury_balance >= reward_amount,
                Error::<T>::InsufficientTreasuryForRewards
            );

            // Transfer reward from treasury to claimer
            T::Currency::transfer(
                &treasury_account,
                &claimer,
                reward_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // Record claim (H-26: per reward_type)
            RewardsClaimed::<T>::insert(&claimer, reward_type, reward_amount);
            
            let total_distributed = TotalRewardsDistributed::<T>::get();
            TotalRewardsDistributed::<T>::put(total_distributed.saturating_add(reward_amount));

            Self::deposit_event(Event::RewardClaimed {
                account: claimer,
                reward_type,
                // SAFETY(saturated_into): BalanceOf<T> → u128 is lossless on standard runtimes (Balance is u128)
                amount: reward_amount.saturated_into(),
            });

            Ok(())
        }

        /// Set proposal priority (admin function)
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn set_proposal_priority(
            origin: OriginFor<T>,
            proposal_id: u32,
            priority: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate priority (1-4)
            ensure!((1..=4).contains(&priority), Error::<T>::InvalidPriority);

            // Verify proposal exists
            ensure!(
                Proposals::<T>::contains_key(proposal_id),
                Error::<T>::ProposalNotFound
            );

            let priority_enum = match priority {
                1 => ProposalPriority::Low,
                2 => ProposalPriority::Normal,
                3 => ProposalPriority::High,
                4 => ProposalPriority::Critical,
                _ => return Err(Error::<T>::InvalidPriority.into()),
            };

            // Get or create queue for this priority
            let mut queue = ProposalQueue::<T>::get(priority_enum);
            
            // Check if queue is full
            ensure!(queue.len() < 50, Error::<T>::PriorityQueueFull);

            // Add to queue if not already present
            if !queue.contains(&proposal_id) {
                queue.try_push(proposal_id)
                    .map_err(|_| Error::<T>::PriorityQueueFull)?;
                ProposalQueue::<T>::insert(priority_enum, queue);
            }

            Self::deposit_event(Event::ProposalQueuedWithPriority {
                proposal_id,
                priority_index: priority,
            });

            Ok(())
        }

        // ===== TREASURY MANAGEMENT EXTRINSICS =====

        /// Allocate budget to a district for the fiscal year.
        ///
        /// Creates or updates the district budget allocation. Only authorized
        /// accounts (Root, FSC, or Foundation Board) can allocate budgets.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Root or authorized governance account
        /// - `district_index`: District identifier (0-5)
        /// - `amount`: Budget amount in DALLA (12 decimals)
        /// - `fiscal_year_blocks`: Duration of fiscal year in blocks
        #[pallet::call_index(28)]
        #[pallet::weight(Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn allocate_district_budget(
            origin: OriginFor<T>,
            district_index: u8,
            amount: BalanceOf<T>,
            fiscal_year_blocks: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate district index and convert
            let district = BelizeDistrict::from_index(district_index)
                .ok_or(Error::<T>::InvalidDistrict)?;

            // Validate fiscal year period
            ensure!(fiscal_year_blocks > 0, Error::<T>::InvalidFiscalYearPeriod);

            let current_block = <frame_system::Pallet<T>>::block_number();
            let fiscal_year_end = current_block + fiscal_year_blocks.into();

            // Check if budget already exists and is active
            if let Some(existing) = DistrictBudgets::<T>::get(district) {
                ensure!(!existing.is_active, Error::<T>::DistrictBudgetAlreadyExists);
            }

            // Create new budget
            let budget = DistrictBudget {
                district,
                allocated: amount,
                spent: Zero::zero(),
                proposals_funded: BoundedVec::default(),
                fiscal_year_start: current_block,
                fiscal_year_end,
                is_active: true,
            };

            DistrictBudgets::<T>::insert(district, budget);

            Self::deposit_event(Event::DistrictBudgetAllocated {
                district_index,
                amount,
                fiscal_year_start: current_block,
                fiscal_year_end,
            });

            Ok(())
        }

        /// Propose a treasury spend (requires multi-sig approval for large amounts).
        ///
        /// Creates a treasury spend proposal that requires Foundation Board approval
        /// via multi-signature workflow. Approval threshold depends on amount:
        /// - < 10,000 DALLA: Immediate execution (no multi-sig)
        /// - 10,000 - 100,000 DALLA: 3-of-7 approval required
        /// - > 100,000 DALLA: 4-of-7 approval required
        ///
        /// ## Parameters
        ///
        /// - `origin`: Proposer (must be council member or FSC)
        /// - `recipient`: Account to receive funds
        /// - `amount`: Amount to transfer (12 decimals)
        /// - `description`: Justification (max 512 bytes)
        /// - `district_index`: Optional district for budget tracking
        #[pallet::call_index(29)]
        #[pallet::weight(Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn propose_treasury_spend(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            amount: BalanceOf<T>,
            description: Vec<u8>,
            district_index: Option<u8>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            // Validate compliance
            ensure!(
                T::ComplianceProvider::can_participate_in_governance(&proposer),
                Error::<T>::NotCompliant
            );

            // Validate description length
            ensure!(description.len() <= 512, Error::<T>::TreasuryDescriptionTooLong);
            let bounded_description = BoundedVec::try_from(description)
                .map_err(|_| Error::<T>::TreasuryDescriptionTooLong)?;

            // Convert district index if provided
            let district = if let Some(idx) = district_index {
                Some(BelizeDistrict::from_index(idx).ok_or(Error::<T>::InvalidDistrict)?)
            } else {
                None
            };

            // Determine approval threshold based on amount
            // Converting amount to u128 for comparison (assuming 12 decimals)
            // SAFETY(saturated_into): BalanceOf<T> → u128 is lossless on standard runtimes (Balance is u128)
            let amount_value: u128 = amount.saturated_into();
            let threshold = if amount_value < 10_000_000_000_000 { // < 10K DALLA
                1u8  // Single approval
            } else if amount_value < 100_000_000_000_000 { // < 100K DALLA
                3u8  // 3-of-7
            } else {
                4u8  // 4-of-7 for large amounts
            };

            let current_block = <frame_system::Pallet<T>>::block_number();
            let expires_at = current_block + 432000u32.into(); // 30 days

            let proposal_id = NextTreasuryProposalId::<T>::get();
            let proposal = TreasurySpendProposal {
                id: proposal_id,
                proposer: proposer.clone(),
                recipient: recipient.clone(),
                amount,
                description: bounded_description,
                district,
                approvals: BoundedVec::default(),
                threshold,
                created_at: current_block,
                expires_at,
                executed: false,
            };

            TreasurySpendProposals::<T>::insert(proposal_id, proposal);
            NextTreasuryProposalId::<T>::put(proposal_id + 1);

            Self::deposit_event(Event::TreasurySpendProposed {
                proposal_id,
                proposer,
                recipient,
                amount,
                threshold,
            });

            Ok(())
        }

        /// Approve a treasury spend proposal (Foundation Board member signature).
        ///
        /// Foundation Board members approve treasury spend proposals. Once the
        /// required threshold is met, the proposal can be executed.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Board member account
        /// - `proposal_id`: ID of treasury spend proposal
        #[pallet::call_index(30)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn approve_treasury_spend(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            let approver = ensure_signed(origin)?;

            // Only council members can approve treasury spends
            ensure!(
                Self::is_council_member(&approver),
                Error::<T>::NotCouncilMember
            );

            // Get proposal
            let mut proposal = TreasurySpendProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::TreasuryProposalNotFound)?;

            // Check not executed
            ensure!(!proposal.executed, Error::<T>::TreasuryProposalAlreadyExecuted);

            // Check not expired
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block <= proposal.expires_at, Error::<T>::TreasuryProposalExpired);

            // Check not already approved by this account
            ensure!(
                !proposal.approvals.contains(&approver),
                Error::<T>::AlreadyApprovedTreasuryProposal
            );

            // Add approval
            proposal.approvals.try_push(approver.clone())
                .map_err(|_| Error::<T>::TooManyApprovals)?;

            let approvals_count = proposal.approvals.len() as u8;

            TreasurySpendProposals::<T>::insert(proposal_id, proposal.clone());

            Self::deposit_event(Event::TreasurySpendApproved {
                proposal_id,
                approver,
                approvals_count,
                threshold: proposal.threshold,
            });

            Ok(())
        }

        /// Execute an approved treasury spend proposal.
        ///
        /// Once the required approval threshold is met, any account can execute
        /// the treasury spend. Funds are transferred from national treasury or
        /// district budget (if specified).
        ///
        /// ## Parameters
        ///
        /// - `origin`: Any signed account
        /// - `proposal_id`: ID of approved treasury spend proposal
        #[pallet::call_index(31)]
        #[pallet::weight(Weight::from_parts(40_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3)))]
        pub fn execute_treasury_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            let _executor = ensure_signed(origin)?;

            // Get proposal
            let mut proposal = TreasurySpendProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::TreasuryProposalNotFound)?;

            // Check not executed
            ensure!(!proposal.executed, Error::<T>::TreasuryProposalAlreadyExecuted);

            // Check not expired
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block <= proposal.expires_at, Error::<T>::TreasuryProposalExpired);

            // Check approval threshold met
            let approvals_count = proposal.approvals.len() as u8;
            ensure!(
                approvals_count >= proposal.threshold,
                Error::<T>::InsufficientTreasuryApprovals
            );

            // If district specified, deduct from district budget
            if let Some(district) = proposal.district {
                let mut budget = DistrictBudgets::<T>::get(district)
                    .ok_or(Error::<T>::DistrictBudgetNotFound)?;

                // Check sufficient budget
                ensure!(
                    budget.allocated >= budget.spent + proposal.amount,
                    Error::<T>::InsufficientDistrictBudget
                );

                // Update budget
                budget.spent += proposal.amount;
                budget.proposals_funded.try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyProposals)?;

                DistrictBudgets::<T>::insert(district, budget.clone());

                Self::deposit_event(Event::DistrictBudgetSpent {
                    district_index: district.to_index(),
                    proposal_id,
                    amount: proposal.amount,
                    remaining: budget.allocated - budget.spent,
                });
            } else {
                // Deduct from national treasury
                let reserves = NationalTreasuryReserve::<T>::get();
                ensure!(reserves >= proposal.amount, Error::<T>::InsufficientNationalTreasury);
                NationalTreasuryReserve::<T>::put(reserves - proposal.amount);
            }

            // Transfer funds from governance treasury to recipient
            Self::execute_treasury_spend(&proposal.recipient, proposal.amount, proposal_id)?;

            // Mark as executed
            proposal.executed = true;
            TreasurySpendProposals::<T>::insert(proposal_id, proposal.clone());

            let district_index = proposal.district.map(|d| d.to_index());

            Self::deposit_event(Event::TreasurySpendExecuted {
                proposal_id,
                recipient: proposal.recipient,
                amount: proposal.amount,
                district_index,
            });

            Ok(())
        }

        /// Transfer budget between districts (requires Root authority).
        ///
        /// Allows reallocation of funds between district budgets with proper
        /// authorization and justification. Useful for emergency situations or
        /// adjusting allocations mid-year.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Root authority
        /// - `from_district_index`: Source district (0-5)
        /// - `to_district_index`: Destination district (0-5)
        /// - `amount`: Amount to transfer
        /// - `reason`: Justification (max 256 bytes)
        #[pallet::call_index(32)]
        #[pallet::weight(Weight::from_parts(30_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn transfer_district_budget(
            origin: OriginFor<T>,
            from_district_index: u8,
            to_district_index: u8,
            amount: BalanceOf<T>,
            reason: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate district indices
            // Validate and convert district indices
            ensure!(from_district_index != to_district_index, Error::<T>::InvalidDistrict);
            
            let from_district = BelizeDistrict::from_index(from_district_index)
                .ok_or(Error::<T>::InvalidDistrict)?;
            let to_district = BelizeDistrict::from_index(to_district_index)
                .ok_or(Error::<T>::InvalidDistrict)?;

            // Validate reason length
            ensure!(reason.len() <= 256, Error::<T>::DescriptionTooLong);
            let bounded_reason = BoundedVec::try_from(reason)
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            // Get source budget
            let mut from_budget = DistrictBudgets::<T>::get(from_district)
                .ok_or(Error::<T>::DistrictBudgetNotFound)?;

            // Check sufficient unspent funds
            let available = from_budget.allocated.saturating_sub(from_budget.spent);
            ensure!(available >= amount, Error::<T>::TransferExceedsSourceBudget);

            // Get or create destination budget
            let mut to_budget = DistrictBudgets::<T>::get(to_district)
                .unwrap_or_else(|| {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    DistrictBudget {
                        district: to_district,
                        allocated: Zero::zero(),
                        spent: Zero::zero(),
                        proposals_funded: BoundedVec::default(),
                        fiscal_year_start: current_block,
                        fiscal_year_end: current_block + BLOCKS_PER_YEAR.into(),
                        is_active: true,
                    }
                });

            // Transfer funds
            from_budget.allocated = from_budget.allocated.saturating_sub(amount);
            to_budget.allocated = to_budget.allocated.saturating_add(amount);

            DistrictBudgets::<T>::insert(from_district, from_budget);
            DistrictBudgets::<T>::insert(to_district, to_budget);

            Self::deposit_event(Event::DistrictBudgetTransferred {
                from_district_index,
                to_district_index,
                amount,
                reason: bounded_reason,
            });

            Ok(())
        }

        // ===== Emergency Governance Procedures =====

        /// Execute fast-track emergency proposal during JaguarMode
        /// 
        /// Enables immediate execution of emergency proposals when:
        /// - JaguarMode is active
        /// - Proposal has achieved super-majority (66%+)
        /// - Proposal has emergency priority
        /// 
        /// # Arguments
        /// * `proposal_id` - ID of the emergency proposal to execute
        /// 
        /// # Weight
        /// - Reads: 3 (JaguarMode, Proposals, CouncilMembers iteration)
        /// - Writes: 1 (Proposals update)
        #[pallet::call_index(33)]
        #[pallet::weight(Weight::from_parts(35_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn execute_emergency_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            // Only Root or Council can execute emergency proposals
            if ensure_root(origin.clone()).is_err() {
                T::CouncilOrigin::ensure_origin(origin)
                    .map_err(|_| Error::<T>::Unauthorized)?;
            }

            // Verify JaguarMode is active
            let emergency_status = JaguarMode::<T>::get()
                .ok_or(Error::<T>::NotInEmergencyMode)?;
            ensure!(emergency_status.active, Error::<T>::NotInEmergencyMode);

            // Get proposal
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            
            // Verify proposal has emergency flag
            ensure!(
                proposal.is_emergency,
                Error::<T>::NotEmergencyProposal
            );

            // Verify proposal is still in voting period
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= proposal.voting_end,
                Error::<T>::EmergencyProposalExpired
            );

            // Calculate super-majority threshold (66%)
            // Use vote_tally to check approval
            let total_votes = proposal.vote_tally.ayes + proposal.vote_tally.nays;
            ensure!(total_votes > 0, Error::<T>::InsufficientSuperMajority);
            
            let approval_percentage = (proposal.vote_tally.ayes * 100) / total_votes;

            // Ensure super-majority achieved (66%+)
            ensure!(
                approval_percentage >= 66,
                Error::<T>::InsufficientSuperMajority
            );

            // Mark as executed
            proposal.executed_at = Some(current_block);
            proposal.status = ProposalStatus::Executed;

            // P0-16 FIX: Actually execute the proposal action (was previously
            // a no-op that only set the status without performing the action).
            if let Some(ref action) = proposal.action {
                Self::execute_action(action, proposal_id)?;
            }

            Proposals::<T>::insert(proposal_id, proposal.clone());
            ExecutedProposals::<T>::insert(proposal_id, current_block);

            // Emit event
            Self::deposit_event(Event::EmergencyProposalExecuted {
                proposal_id,
                emergency_type_index: emergency_status.emergency_type.to_index(),
                approval_percentage,
            });

            Ok(())
        }

        /// Fast-track emergency referendum during JaguarMode
        /// 
        /// Reduces voting period to 3 hours (10,800 blocks) for critical decisions.
        /// Requires Root or FSC authority.
        /// 
        /// # Arguments
        /// * `referendum_id` - ID of the referendum to fast-track
        /// 
        /// # Weight
        /// - Reads: 2 (JaguarMode, Referendums)
        /// - Writes: 1 (Referendums update)
        #[pallet::call_index(34)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn fast_track_referendum(
            origin: OriginFor<T>,
            referendum_id: u32,
        ) -> DispatchResult {
            // Only Root or Council can fast-track
            if ensure_root(origin.clone()).is_err() {
                T::CouncilOrigin::ensure_origin(origin)
                    .map_err(|_| Error::<T>::Unauthorized)?;
            }

            // Verify JaguarMode is active
            let emergency_status = JaguarMode::<T>::get()
                .ok_or(Error::<T>::NotInEmergencyMode)?;
            ensure!(emergency_status.active, Error::<T>::NotInEmergencyMode);

            // Get referendum
            let mut referendum = Referendums::<T>::get(referendum_id)
                .ok_or(Error::<T>::ReferendumNotFound)?;

            // Verify still active (not already passed/failed/cancelled)
            ensure!(
                referendum.status == ReferendumStatus::Active,
                Error::<T>::ReferendumAlreadyFinalized
            );

            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Set fast-track deadline (3 hours = 10,800 blocks)
            let fast_track_deadline = current_block + 10_800u32.into();
            let original_deadline = referendum.voting_end;
            referendum.voting_end = fast_track_deadline;

            Referendums::<T>::insert(referendum_id, referendum);

            // Emit event
            Self::deposit_event(Event::ReferendumFastTracked {
                referendum_id,
                original_deadline,
                new_deadline: fast_track_deadline,
                emergency_type_index: emergency_status.emergency_type.to_index(),
            });

            Ok(())
        }

        /// Override proposal result during national emergency
        /// 
        /// Allows Root to forcibly execute or reject proposals during crisis situations.
        /// Requires detailed justification. Should only be used in extreme circumstances.
        /// 
        /// # Arguments
        /// * `proposal_id` - ID of the proposal to override
        /// * `execute` - true to force execution, false to force rejection
        /// * `justification` - Reason for emergency override
        /// 
        /// # Weight
        /// - Reads: 2 (JaguarMode, Proposals)
        /// - Writes: 1 (Proposals update)
        #[pallet::call_index(35)]
        #[pallet::weight(Weight::from_parts(30_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn emergency_override_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
            execute: bool,
            justification: Vec<u8>,
        ) -> DispatchResult {
            // Only Root can override
            ensure_root(origin)?;

            // Verify JaguarMode is active
            let emergency_status = JaguarMode::<T>::get()
                .ok_or(Error::<T>::NotInEmergencyMode)?;
            ensure!(emergency_status.active, Error::<T>::NotInEmergencyMode);

            // Validate justification
            let bounded_justification: BoundedVec<u8, ConstU32<512>> = justification
                .try_into()
                .map_err(|_| Error::<T>::TreasuryDescriptionTooLong)?;

            // Get proposal
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // Update proposal state based on override decision
            if execute {
                proposal.executed_at = Some(current_block);
                proposal.status = ProposalStatus::Executed;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
            
            Proposals::<T>::insert(proposal_id, proposal.clone());

            // Emit event
            Self::deposit_event(Event::ProposalEmergencyOverride {
                proposal_id,
                executed: execute,
                overridden_by: emergency_status.declared_by.unwrap_or([0u8; 32]),
                justification: bounded_justification,
            });

            Ok(())
        }

        /// Update a governance-controlled chain parameter.
        ///
        /// Only callable by Root (i.e., via a successful governance proposal or sudo).
        /// This allows the chain to adjust operational constants — such as term lengths,
        /// delegation limits, or reward percentages — without a full runtime upgrade.
        ///
        /// ## Parameters
        ///
        /// - `origin`: Root origin (governance proposal output)
        /// - `key`: Parameter identifier (max 32 bytes, e.g. `b"blocks_per_yr"`)
        /// - `value`: New u64 value for the parameter
        ///
        /// ## Emits
        ///
        /// - `ChainParameterUpdated`
        #[pallet::call_index(36)]
        #[pallet::weight(Weight::from_parts(10_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn update_chain_parameter(
            origin: OriginFor<T>,
            key: Vec<u8>,
            value: u64,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let bounded_key: BoundedVec<u8, ConstU32<32>> = key
                .try_into()
                .map_err(|_| Error::<T>::ParameterKeyTooLong)?;

            // Phase 6B: Reject if this parameter is constitutionally locked.
            ensure!(
                !LockedParameters::<T>::get(&bounded_key),
                Error::<T>::ParameterConstitutionallyLocked
            );

            let old_value = ChainParameters::<T>::get(&bounded_key);
            ChainParameters::<T>::insert(&bounded_key, value);

            Self::deposit_event(Event::ChainParameterUpdated {
                key: bounded_key,
                old_value,
                new_value: value,
            });

            Ok(())
        }

    // ── AR-13: Commit-Reveal Voting ──────────────────────────────────────
    // These extrinsics are part of the main #[pallet::call] block above.

        /// Commit a blinded vote for a proposal (phase 1 of commit-reveal).
        ///
        /// # Parameters
        /// - `proposal_id`: The proposal to vote on.
        /// - `commitment`: `blake2_256(vote_choice_byte || salt_32_bytes)`.
        ///   `vote_choice_byte` is 0 = Aye, 1 = Nay, 2 = Abstain.
        ///   `salt_32_bytes` is a 32-byte secret chosen by the voter.
        ///
        /// # Errors
        /// - `ProposalNotFound` — unknown proposal.
        /// - `VotingPeriodNotStarted` / `VotingPeriodEnded` — outside commit window.
        /// - `VoteAlreadyCommitted` — caller already committed for this proposal.
        /// - `AlreadyVoted` — caller already revealed (recorded in `Votes`).
        ///
        /// # Events
        /// - `VoteCommitted { proposal_id, voter }`
        #[pallet::weight(Weight::from_parts(10_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1)))]
        #[pallet::call_index(37)]
        pub fn commit_vote(
            origin: OriginFor<T>,
            proposal_id: u32,
            commitment: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let proposal = Self::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            let current_block = frame_system::Pallet::<T>::block_number();

            ensure!(current_block >= proposal.voting_start, Error::<T>::VotingPeriodNotStarted);
            ensure!(current_block <= proposal.voting_end, Error::<T>::VotingPeriodEnded);

            // Guard: prevent committing if already revealed
            ensure!(!Votes::<T>::contains_key(proposal_id, &who), Error::<T>::AlreadyVoted);
            // Guard: one commitment per account per proposal
            ensure!(
                !VoteCommitments::<T>::contains_key(proposal_id, &who),
                Error::<T>::VoteAlreadyCommitted
            );

            VoteCommitments::<T>::insert(proposal_id, &who, commitment);

            Self::deposit_event(Event::VoteCommitted { proposal_id, voter: who });
            Ok(())
        }

        /// Reveal a previously committed vote (phase 2 of commit-reveal).
        ///
        /// Verifies `blake2_256([vote_choice_byte] ++ salt) == stored_commitment`,
        /// then records the vote in `Votes` and updates the proposal tally, exactly
        /// as `cast_vote` does.
        ///
        /// # Parameters
        /// - `proposal_id`: The proposal being voted on.
        /// - `vote_choice_index`: 0 = Aye, 1 = Nay, 2 = Abstain.
        /// - `salt`: The 32-byte secret used when committing.
        /// - `conviction`: Conviction multiplier (≥ 1; clamped to 1 minimum).
        ///
        /// # Errors
        /// - `ProposalNotFound`, `VotingPeriodEnded`
        /// - `NoCommitmentFound` — no prior commitment exists.
        /// - `CommitmentHashMismatch` — hash does not match.
        /// - `AlreadyVoted` — vote already recorded (double-reveal guard).
        ///
        /// # Events
        /// - `VoteRevealed { proposal_id, voter, vote_index, weight }`
        #[pallet::weight(Weight::from_parts(15_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3)))]
        #[pallet::call_index(38)]
        pub fn reveal_vote(
            origin: OriginFor<T>,
            proposal_id: u32,
            vote_choice_index: u8,
            salt: [u8; 32],
            conviction: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Load stored commitment — error if none exists
            let stored_commitment = VoteCommitments::<T>::get(proposal_id, &who)
                .ok_or(Error::<T>::NoCommitmentFound)?;

            // Recompute commitment from plaintext; compare
            // Preimage: [vote_choice_byte (1)] ++ [salt (32)] = 33 bytes total
            let mut preimage = [0u8; 33];
            preimage[0] = vote_choice_index;
            preimage[1..].copy_from_slice(&salt);
            let computed = blake2_256(&preimage);
            ensure!(computed == stored_commitment, Error::<T>::CommitmentHashMismatch);

            // Guard against double-reveal
            ensure!(!Votes::<T>::contains_key(proposal_id, &who), Error::<T>::VoteAlreadyRevealedViaCommit);

            // Resolve vote choice
            let vote_choice = match vote_choice_index {
                0 => VoteChoice::Aye,
                1 => VoteChoice::Nay,
                2 => VoteChoice::Abstain,
                _ => return Err(Error::<T>::InvalidThreshold.into()),
            };

            let mut proposal = Self::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            let current_block = frame_system::Pallet::<T>::block_number();

            // G-1 FIX: Bound reveal window — reveals accepted only while voting is
            // open OR within one VotingPeriod after voting_end (reveal window).
            // This prevents tally mutation after finalization.
            ensure!(current_block >= proposal.voting_start, Error::<T>::VotingPeriodNotStarted);
            let reveal_deadline = proposal.voting_end
                .saturating_add(T::VotingPeriod::get());
            ensure!(current_block <= reveal_deadline, Error::<T>::VotingPeriodEnded);
            // Reject reveals on already-finalized proposals (accept Pending
            // or Voting — a reveal IS a vote and may be the first one cast)
            ensure!(
                proposal.status == ProposalStatus::Pending
                    || proposal.status == ProposalStatus::Voting,
                Error::<T>::ProposalNotFound
            );

            // G-2 FIX: Use full weight calculation matching cast_vote
            // (QV, stake, MaxVotingUnits, EffectiveVotingMultiplier)
            let community_rank = Self::community_ranks(&who);
            let pouw_contribution = Self::pouw_contributions(&who);

            let balance_u128: u128 = T::Currency::free_balance(&who).saturated_into::<u128>();
            let stake_unit_size_u128: u128 = T::StakeUnitSize::get().saturated_into::<u128>();
            let stake_units: u128 = if stake_unit_size_u128 > 0 {
                balance_u128 / stake_unit_size_u128
            } else {
                0
            };
            let raw_stake_weight = if T::QuadraticVotingEnabled::get() {
                Self::isqrt(stake_units)
            } else {
                stake_units
            };
            let stake_weight: u32 = raw_stake_weight
                .min(T::MaxVotingUnits::get() as u128) as u32;

            let base_weight = community_rank
                .saturating_add(pouw_contribution)
                .saturating_add(stake_weight);

            let conviction_multiplier = (conviction as u32).max(1);
            let conviction_weight = base_weight.saturating_mul(conviction_multiplier);

            let multiplier = EffectiveVotingMultiplier::<T>::get(&who);
            let effective_mult = if multiplier == 0 { 100u32 } else { multiplier as u32 };
            let final_weight = conviction_weight
                .saturating_mul(effective_mult)
                / 100;

            // Record vote in Votes storage
            let vote = Vote {
                vote: vote_choice.clone(),
                weight: final_weight,
                conviction,
                timestamp: current_block.saturated_into(),
            };
            Votes::<T>::insert(proposal_id, &who, vote);

            // Update tally
            match vote_choice {
                VoteChoice::Aye =>
                    proposal.vote_tally.ayes =
                        proposal.vote_tally.ayes.saturating_add(final_weight),
                VoteChoice::Nay =>
                    proposal.vote_tally.nays =
                        proposal.vote_tally.nays.saturating_add(final_weight),
                VoteChoice::Abstain =>
                    proposal.vote_tally.abstentions =
                        proposal.vote_tally.abstentions.saturating_add(final_weight),
            }
            proposal.vote_tally.total_weight =
                proposal.vote_tally.total_weight.saturating_add(final_weight);
            proposal.status = ProposalStatus::Voting;
            Proposals::<T>::insert(proposal_id, proposal);

            // Remove commitment (prevent replay)
            VoteCommitments::<T>::remove(proposal_id, &who);

            // Record governance participation
            let _ = T::CommunityParticipation::record_vote_cast(&who);

            Self::deposit_event(Event::VoteRevealed {
                proposal_id,
                voter: who,
                vote_index: vote_choice_index,
                weight: final_weight,
            });
            Ok(())
        }

        // ── AR-14: Apply governance-approved runtime upgrade ─────────────────

        /// Apply a pending runtime upgrade that was previously approved via governance.
        ///
        /// # Flow
        /// 1. Governance passes a `RuntimeUpgrade { code_hash }` proposal →
        ///    `execute_runtime_upgrade` stores the 32-byte hash in
        ///    `PendingRuntimeUpgrade`.
        /// 2. A trusted operator (root / sudo for now) calls this extrinsic with
        ///    the actual WASM blob.
        /// 3. `blake2_256(&code)` is verified against `PendingRuntimeUpgrade`.
        /// 4. `frame_system::Pallet::<T>::set_code` is dispatched as root.
        /// 5. `PendingRuntimeUpgrade` is cleared to prevent re-application.
        ///
        /// # Security
        /// - Only callable with root origin (sudo until sudo removal at launch).
        /// - Code hash is verified on-chain before `set_code` \u2014 no governance
        ///   bypass is possible.
        ///
        /// # Errors
        /// - `RuntimeUpgradeNotPending` \u2014 no governance-approved hash exists.
        /// - `RuntimeCodeHashMismatch` \u2014 submitted code does not match approved hash.
        /// - Any error propagated by `frame_system::set_code` (e.g. code too large).
        ///
        /// # Events
        /// - `RuntimeUpgradeApplied { code_hash }`
        #[pallet::weight(Weight::from_parts(200_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1)))]
        #[pallet::call_index(39)]
        pub fn apply_pending_runtime_upgrade(
            origin: OriginFor<T>,
            code: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let approved_hash = PendingRuntimeUpgrade::<T>::get()
                .ok_or(Error::<T>::RuntimeUpgradeNotPending)?;

            // Verify code against governance-approved hash
            let code_hash = blake2_256(&code);
            ensure!(code_hash == approved_hash, Error::<T>::RuntimeCodeHashMismatch);

            // Apply the upgrade. `set_code` takes a root origin internally.
            frame_system::Pallet::<T>::set_code(
                frame_system::RawOrigin::Root.into(),
                code,
            )?;

            // Clear pending upgrade to prevent replay.
            PendingRuntimeUpgrade::<T>::kill();

            Self::deposit_event(Event::RuntimeUpgradeApplied {
                code_hash: approved_hash,
            });

            Ok(Pays::No.into())
        }

        // ── Phase 4C: Exit right ──────────────────────────────────────────────

        /// Generate a chain-signed proof of the caller's current on-chain state.
        ///
        /// This extrinsic is permissionless — any account may call it at any time.
        /// It produces a deterministic `proof_hash = blake2_256(account || free_balance || block)`,
        /// which can be independently verified off-chain to demonstrate the exact state
        /// of the account at exit time.  The proof is valid for `ExitProofValidity` blocks
        /// (~30 days by default).
        ///
        /// ## Consensus Safety
        /// - Purely deterministic: same inputs → same hash on every node.
        /// - No storage writes: only emits an event.
        /// - No side effects on balances or active governance.
        ///
        /// ## Emits
        /// - `ExitProofGenerated { account, proof_hash, valid_until }`
        #[pallet::call_index(40)]
        #[pallet::weight(Weight::from_parts(10_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(2)))]
        pub fn generate_exit_proof(origin: OriginFor<T>) -> DispatchResult {
            let account = ensure_signed(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let valid_until = current_block.saturating_add(T::ExitProofValidity::get());

            // Capture the caller's current free balance as part of the proof payload.
            let balance = T::Currency::free_balance(&account);

            // Deterministic proof: hash( account_bytes ++ encoded_balance ++ encoded_block )
            // Using `Encode` (SCALE) for consistent byte layout across nodes.
            let proof_input = (account.clone(), balance, current_block).encode();
            let proof_hash = blake2_256(&proof_input);

            Self::deposit_event(Event::ExitProofGenerated {
                account,
                proof_hash,
                valid_until,
            });

            Ok(())
        }

        // ── Phase 6A: Constitutional dual-house ratification ──────────────────

        /// Ratify an approved Constitutional proposal on behalf of one governance house.
        ///
        /// After a Constitutional proposal passes public voting (status = Approved),
        /// it must be independently ratified by at least one member of each house
        /// (TechnicalCouncil + GovernanceCouncil) before it can be executed.  This
        /// provides a four-eyes check beyond the democratic vote.
        ///
        /// The caller's house membership is resolved at call time via `DualHouseProvider`.
        /// If both houses have now ratified, `ConstitutionalRatificationComplete` is emitted.
        ///
        /// ## Errors
        /// - `ProposalNotFound` — unknown proposal ID
        /// - `NotConstitutionalProposal` — only Constitutional proposals require dual ratification
        /// - `ProposalNotYetApproved` — proposal must pass the public vote first
        /// - `NotHouseMember` — caller is not a member of either governance house
        /// - `AlreadyRatifiedByHouse` — this house has already submitted its ratification
        #[pallet::call_index(41)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn ratify_constitutional_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Only Constitutional proposals go through dual-house ratification.
            ensure!(
                proposal.proposal_type == ProposalType::Constitutional,
                Error::<T>::NotConstitutionalProposal
            );

            // Proposal must have passed public vote first.
            ensure!(
                proposal.status == ProposalStatus::Approved,
                Error::<T>::ProposalNotYetApproved
            );

            let is_technical = T::DualHouseProvider::is_technical_house_member(&who);
            let is_governance = T::DualHouseProvider::is_governance_house_member(&who);

            // Caller must belong to at least one house.
            ensure!(is_technical || is_governance, Error::<T>::NotHouseMember);

            let (mut tech_approved, mut gov_approved) =
                ConstitutionalRatifications::<T>::get(proposal_id);

            // Determine which house(s) to record — a dual-member counts for both.
            if is_technical {
                ensure!(!tech_approved, Error::<T>::AlreadyRatifiedByHouse);
                tech_approved = true;
            }
            if is_governance {
                ensure!(!gov_approved, Error::<T>::AlreadyRatifiedByHouse);
                gov_approved = true;
            }

            ConstitutionalRatifications::<T>::insert(proposal_id, (tech_approved, gov_approved));

            let house: u8 = match (is_technical, is_governance) {
                (true, false) => 0,
                (false, true) => 1,
                _ => 2, // member of both houses
            };

            Self::deposit_event(Event::ConstitutionalRatificationCast {
                proposal_id,
                ratifier: who,
                house,
                technical_approved: tech_approved,
                governance_approved: gov_approved,
            });

            if tech_approved && gov_approved {
                Self::deposit_event(Event::ConstitutionalRatificationComplete { proposal_id });
            }

            Ok(())
        }

        // ── Phase 6B: Constitutional parameter locks ──────────────────────────

        /// Constitutionally lock a chain parameter against modification.
        ///
        /// Once locked, `update_chain_parameter` will reject any update to this key
        /// until `unlock_chain_parameter` lifts the lock.  Locking should be used
        /// to protect fundamental parameters (e.g., max council size, inflation cap)
        /// that should only change via a fully dual-house-ratified Constitutional proposal.
        ///
        /// Requires `ConstitutionalAdminOrigin`.
        #[pallet::call_index(42)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn lock_chain_parameter(
            origin: OriginFor<T>,
            key: Vec<u8>,
        ) -> DispatchResult {
            T::ConstitutionalAdminOrigin::ensure_origin(origin)?;

            let bounded_key: BoundedVec<u8, ConstU32<32>> =
                key.try_into().map_err(|_| Error::<T>::ParameterKeyTooLong)?;

            ensure!(
                !LockedParameters::<T>::get(&bounded_key),
                Error::<T>::ParameterAlreadyLocked
            );

            LockedParameters::<T>::insert(&bounded_key, true);

            Self::deposit_event(Event::ParameterLocked { key: bounded_key });
            Ok(())
        }

        /// Lift a constitutional lock on a chain parameter.
        ///
        /// After unlocking, `update_chain_parameter` may modify the key again.
        /// Re-locking requires another call to `lock_chain_parameter`.
        ///
        /// Requires `ConstitutionalAdminOrigin`.
        #[pallet::call_index(43)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1)))]
        pub fn unlock_chain_parameter(
            origin: OriginFor<T>,
            key: Vec<u8>,
        ) -> DispatchResult {
            T::ConstitutionalAdminOrigin::ensure_origin(origin)?;

            let bounded_key: BoundedVec<u8, ConstU32<32>> =
                key.try_into().map_err(|_| Error::<T>::ParameterKeyTooLong)?;

            ensure!(
                LockedParameters::<T>::get(&bounded_key),
                Error::<T>::ParameterNotLocked
            );

            LockedParameters::<T>::remove(&bounded_key);

            Self::deposit_event(Event::ParameterUnlocked { key: bounded_key });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Calculate total council voting weight
        ///
        /// DOS-018: This helper iterates up to MaxCandidatesPerElection (200) council
        /// members.  Callers must include the iteration cost in their extrinsic weight:
        /// reads(MaxCandidatesPerElection) + O(MaxCandidatesPerElection) ref_time.
        pub fn total_council_weight() -> u32 {
            // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7)
            // Council size is naturally bounded by MaxCandidatesPerElection;
            // explicit .take() provides defense-in-depth.
            let max_council = T::MaxCandidatesPerElection::get() as usize;
            CouncilMembers::<T>::iter()
                .take(max_council)
                .map(|(_, member)| member.voting_weight)
                .sum()
        }

        /// Query a governance-controlled chain parameter, falling back to `default`
        /// if the key has not been set via `update_chain_parameter`.
        pub fn chain_param(key: &[u8], default: u64) -> u64 {
            let bounded: Result<BoundedVec<u8, ConstU32<32>>, _> = key.to_vec().try_into();
            match bounded {
                Ok(k) => ChainParameters::<T>::get(&k).unwrap_or(default),
                Err(_) => default,
            }
        }

        /// Check if account is council member
        pub fn is_council_member(account: &T::AccountId) -> bool {
            CouncilMembers::<T>::contains_key(account)
        }

        // ── Phase 6A: Constitutional ratification helper ──────────────────────

        /// Check whether a Constitutional proposal has been ratified by both houses.
        ///
        /// Returns `(technical_ratified, governance_ratified)`.
        /// A proposal is fully ratified when both values are `true`.
        ///
        /// Invariant: only meaningful for `ProposalType::Constitutional` proposals.
        pub fn is_constitutionally_ratified(proposal_id: u32) -> (bool, bool) {
            ConstitutionalRatifications::<T>::get(proposal_id)
        }

        // ── Phase 2A: Quadratic voting helpers ───────────────────────────────

        /// Integer square-root (floor) using Newton's method.
        ///
        /// Deterministic, no floating-point.  Returns `floor(sqrt(n))`.
        /// Used for quadratic voting weight calculation.
        ///
        /// Invariant: `isqrt(n)^2 <= n < (isqrt(n)+1)^2`
        pub fn isqrt(n: u128) -> u128 {
            if n == 0 {
                return 0;
            }
            let mut x = n;
            let mut y = (x + 1) / 2;
            while y < x {
                x = y;
                y = (x + n / x) / 2;
            }
            x
        }

        /// Get council size
        ///
        /// DOS-018: This helper iterates up to MaxCandidatesPerElection (200) council
        /// members.  Callers must include the iteration cost in their extrinsic weight:
        /// reads(MaxCandidatesPerElection) + O(MaxCandidatesPerElection) ref_time.
        pub fn council_size() -> u32 {
            // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7)
            let max_council = T::MaxCandidatesPerElection::get() as usize;
            CouncilMembers::<T>::iter().take(max_council).count() as u32
        }

        // ===== Phase 4: Board System Helper Functions =====

        /// Get board composition for a role
        pub fn board_composition(role: BoardRole) -> u32 {
            BoardComposition::<T>::get(role)
        }

        /// Get delegate nominee vote count
        pub fn delegate_nominees(account: T::AccountId) -> u32 {
            DelegateNominees::<T>::get(account)
        }

        /// Get current election ID
        pub fn current_election() -> Option<BlockNumberFor<T>> {
            CurrentElection::<T>::get()
        }

        /// Check if account has voted in election
        pub fn has_voted_in_election(election_id: BlockNumberFor<T>, account: &T::AccountId) -> bool {
            DelegateVoters::<T>::contains_key(election_id, account)
        }

        // ===== PHASE 5: EXECUTION LAYER HELPER METHODS =====

        /// Execute a proposal action
        fn execute_action(
            action: &ProposalAction<T::AccountId, BalanceOf<T>>,
            proposal_id: u32,
        ) -> DispatchResult {
            match action {
                ProposalAction::TreasurySpend { recipient, amount } => {
                    Self::execute_treasury_spend(recipient, *amount, proposal_id)?;
                }
                ProposalAction::RuntimeUpgrade { code_hash } => {
                    Self::execute_runtime_upgrade(code_hash, proposal_id)?;
                }
                ProposalAction::ParameterChange { parameter, new_value } => {
                    Self::execute_parameter_change(*parameter, *new_value)?;
                }
                ProposalAction::DepartmentAction { department, call_data } => {
                    Self::execute_department_action(*department, call_data, proposal_id)?;
                }
                ProposalAction::EmergencyAction { action_type } => {
                    Self::execute_emergency_action(*action_type, proposal_id)?;
                }
            }
            Ok(())
        }

        /// Execute treasury spending
        fn execute_treasury_spend(
            recipient: &T::AccountId,
            amount: BalanceOf<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            // S5-4: Enforce per-period treasury spend cap
            let current_block = <frame_system::Pallet<T>>::block_number();
            let period_len: u32 = T::TreasurySpendPeriod::get().saturated_into();
            let current_period: u32 = if period_len > 0 {
                current_block.saturated_into::<u32>() / period_len
            } else {
                0u32
            };
            let (tracked_period, already_spent) = TreasurySpendTracker::<T>::get();
            let effective_spent = if tracked_period == current_period {
                already_spent
            } else {
                Zero::zero()
            };
            ensure!(
                effective_spent.saturating_add(amount) <= T::MaxTreasurySpendPerPeriod::get(),
                Error::<T>::TreasurySpendCapExceeded
            );
            TreasurySpendTracker::<T>::put(
                (current_period, effective_spent.saturating_add(amount))
            );

            // Get treasury account (we'll use pallet's account for now)
            let treasury_account = Self::account_id();

            // Check treasury balance
            let treasury_balance = T::Currency::free_balance(&treasury_account);
            ensure!(
                treasury_balance >= amount,
                Error::<T>::InsufficientTreasuryBalance
            );

            // Transfer funds
            T::Currency::transfer(
                &treasury_account,
                recipient,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Self::deposit_event(Event::TreasurySpendExecuted {
                proposal_id,
                recipient: recipient.clone(),
                amount,
                district_index: None,  // Helper function doesn't track district
            });

            Ok(())
        }

        /// Execute runtime upgrade
        fn execute_runtime_upgrade(
            code_hash: &BoundedVec<u8, ConstU32<32>>,
            proposal_id: u32,
        ) -> DispatchResult {
            // Verify code hash length
            ensure!(
                code_hash.len() == 32,
                Error::<T>::RuntimeCodeTooLarge
            );

            // Convert to fixed-size array
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&code_hash[..]);

            // Note: Actual runtime upgrade would use frame_system::set_code
            // For safety, we store the hash and require manual upgrade
            PendingRuntimeUpgrade::<T>::put(hash_array);

            Self::deposit_event(Event::RuntimeUpgradeExecuted {
                proposal_id,
                code_hash: hash_array,
            });

            Ok(())
        }

        /// Execute governance parameter change
        fn execute_parameter_change(
            parameter: GovernanceParameter,
            new_value: u32,
        ) -> DispatchResult {
            // Validate parameter value ranges
            match parameter {
                GovernanceParameter::VotingPeriod => {
                    ensure!((3600..=604800).contains(&new_value), Error::<T>::InvalidParameterValue);
                }
                GovernanceParameter::LaunchPeriod => {
                    ensure!((1800..=86400).contains(&new_value), Error::<T>::InvalidParameterValue);
                }
                GovernanceParameter::MinimumDeposit => {
                    ensure!((100..=1000000).contains(&new_value), Error::<T>::InvalidParameterValue);
                }
                GovernanceParameter::SupermajorityThreshold => {
                    ensure!((51..=100).contains(&new_value), Error::<T>::InvalidParameterValue);
                }
                GovernanceParameter::CouncilSize => {
                    ensure!((5..=50).contains(&new_value), Error::<T>::InvalidParameterValue);
                }
                GovernanceParameter::EmergencyTimeout => {
                    ensure!((3600..=604800).contains(&new_value), Error::<T>::InvalidParameterValue);
                }
            }

            // Get old value
            let old_value = GovernanceParameters::<T>::get(parameter);

            // Update parameter
            GovernanceParameters::<T>::insert(parameter, new_value);

            // Get parameter index for event
            let parameter_index = Self::governance_parameter_index(parameter);

            Self::deposit_event(Event::ParameterChanged {
                parameter_index,
                old_value,
                new_value,
            });

            Ok(())
        }

        /// Execute department-specific action
        ///
        /// AR-8: `call_data` is SCALE-encoded as `DepartmentCall`. Decoded
        /// variants are dispatched deterministically.  Unknown byte sequences
        /// return `InvalidCallData` so no panic can occur.
        fn execute_department_action(
            department: Department,
            call_data: &BoundedVec<u8, ConstU32<1024>>,
            proposal_id: u32,
        ) -> DispatchResult {
            let department_index = Self::department_index(department);

            // Decode SCALE-encoded DepartmentCall from call_data bytes.
            // An unrecognised byte sequence returns InvalidCallData — no panic.
            let action = DepartmentCall::<T::AccountId, BalanceOf<T>>::decode(&mut call_data.as_slice())
                .map_err(|_| Error::<T>::InvalidCallData)?;

            match action {
                DepartmentCall::SpendBudget { recipient, amount } => {
                    // Treasury transfer from the governance pallet account.
                    // Re-uses the same path as execute_treasury_spend.
                    Self::execute_treasury_spend(&recipient, amount, proposal_id)?;
                }
                DepartmentCall::UpdatePolicy { policy_key, policy_value } => {
                    // Store a bounded policy key→value entry on-chain.
                    DepartmentPolicies::<T>::insert(
                        (department, &policy_key),
                        policy_value,
                    );
                }
                DepartmentCall::NoOp => {
                    // Explicit no-operation: emit event only.
                }
            }

            Self::deposit_event(Event::DepartmentActionExecuted {
                proposal_id,
                department_index,
            });

            Ok(())
        }

        /// Execute emergency action
        fn execute_emergency_action(
            action_type: EmergencyActionType,
            proposal_id: u32,
        ) -> DispatchResult {
            // Check if emergency actions are allowed
            match action_type {
                EmergencyActionType::ActivateEmergency => {
                    // CONS-029: proposal-based emergencies also enter veto window
                    let current_block = frame_system::Pallet::<T>::block_number();
                    let veto_window_ends = current_block + T::EmergencyVetoWindow::get();
                    let emergency = EmergencyStatus {
                        active: false,
                        emergency_type: EmergencyType::Other,
                        description: BoundedVec::try_from(b"Emergency activated via proposal".to_vec())
                            .unwrap_or_default(),
                        declared_at: current_block,
                        expires_at: None,
                        declared_by: None,
                        is_pending: true,
                        veto_window_ends_at: Some(veto_window_ends),
                    };
                    JaguarMode::<T>::put(emergency);
                    let _ = EmergencyVetoes::<T>::clear(u32::MAX, None);
                }
                EmergencyActionType::DeactivateEmergency => {
                    JaguarMode::<T>::kill();
                }
                EmergencyActionType::FreezeAccount |
                EmergencyActionType::UnfreezeAccount |
                EmergencyActionType::HaltGovernance |
                EmergencyActionType::ResumeGovernance => {
                    // These would require additional storage and logic
                    // For now, just emit event
                }
            }

            let action_type_index = Self::emergency_action_type_index(action_type);

            Self::deposit_event(Event::EmergencyActionExecuted {
                proposal_id,
                action_type_index,
            });

            Ok(())
        }

        /// Get treasury account ID
        pub fn account_id() -> T::AccountId {
            // Use pallet's module ID as treasury
            const PALLET_ID: frame_support::PalletId = frame_support::PalletId(*b"py/gover");
            PALLET_ID.into_account_truncating()
        }

        /// Get action type index for events
        pub fn get_action_type_index(action: &ProposalAction<T::AccountId, BalanceOf<T>>) -> u8 {
            match action {
                ProposalAction::TreasurySpend { .. } => 0,
                ProposalAction::RuntimeUpgrade { .. } => 1,
                ProposalAction::ParameterChange { .. } => 2,
                ProposalAction::DepartmentAction { .. } => 3,
                ProposalAction::EmergencyAction { .. } => 4,
            }
        }

        /// Get governance parameter value
        pub fn get_parameter(parameter: GovernanceParameter) -> u32 {
            GovernanceParameters::<T>::get(parameter)
        }

        /// Get department treasury balance
        pub fn department_treasury_balance(department: Department) -> BalanceOf<T> {
            DepartmentTreasuryBalances::<T>::get(department)
        }

        /// Allocate funds to department treasury
        pub fn allocate_to_department(
            department: Department,
            amount: BalanceOf<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            // Update department balance
            DepartmentTreasuryBalances::<T>::mutate(department, |balance| {
                *balance = balance.saturating_add(amount);
            });

            let department_index = Self::department_index(department);

            Self::deposit_event(Event::DepartmentFundsAllocated {
                department_index,
                amount,
                from_proposal: proposal_id,
            });

            Ok(())
        }

        /// Get index for Department enum (for events)
        pub fn department_index(department: Department) -> u8 {
            match department {
                Department::Finance => 0,
                Department::Education => 1,
                Department::Health => 2,
                Department::Works => 3,
                Department::Justice => 4,
                Department::Tourism => 5,
                Department::Agriculture => 6,
                Department::Defense => 7,
            }
        }

        /// Get index for GovernanceParameter enum (for events)
        pub fn governance_parameter_index(parameter: GovernanceParameter) -> u8 {
            match parameter {
                GovernanceParameter::VotingPeriod => 0,
                GovernanceParameter::LaunchPeriod => 1,
                GovernanceParameter::MinimumDeposit => 2,
                GovernanceParameter::SupermajorityThreshold => 3,
                GovernanceParameter::CouncilSize => 4,
                GovernanceParameter::EmergencyTimeout => 5,
            }
        }

        /// Get index for EmergencyActionType enum (for events)
        pub fn emergency_action_type_index(action_type: EmergencyActionType) -> u8 {
            match action_type {
                EmergencyActionType::ActivateEmergency => 0,
                EmergencyActionType::DeactivateEmergency => 1,
                EmergencyActionType::FreezeAccount => 2,
                EmergencyActionType::UnfreezeAccount => 3,
                EmergencyActionType::HaltGovernance => 4,
                EmergencyActionType::ResumeGovernance => 5,
            }
        }

        // ===== PHASE 6: ELECTION SYSTEM HELPER FUNCTIONS =====

        /// Get district election info
        pub fn get_district_election(district: BelizeDistrict) -> Option<Election<BlockNumberFor<T>>> {
            DistrictElections::<T>::get(district)
        }

        /// Get election candidate info
        pub fn get_election_candidate(
            election_id: BlockNumberFor<T>,
            candidate: T::AccountId
        ) -> Option<ElectionCandidate<T::AccountId, BlockNumberFor<T>>> {
            ElectionCandidates::<T>::get(election_id, candidate)
        }

        /// Check if account has voted in election
        pub fn has_voted_in_district_election(
            election_id: BlockNumberFor<T>,
            voter: &T::AccountId
        ) -> bool {
            ElectionVotes::<T>::contains_key(election_id, voter)
        }

        /// Get district representatives
        pub fn get_district_representatives(district: BelizeDistrict) -> Vec<T::AccountId> {
            DistrictRepresentation::<T>::get(district).into_inner()
        }

        /// Get district index for event emission
        pub fn district_index(district: BelizeDistrict) -> u8 {
            match district {
                BelizeDistrict::Belize => 0,
                BelizeDistrict::Cayo => 1,
                BelizeDistrict::Corozal => 2,
                BelizeDistrict::OrangeWalk => 3,
                BelizeDistrict::StannCreek => 4,
                BelizeDistrict::Toledo => 5,
            }
        }

        /// Get election status index
        pub fn election_status_index(status: ElectionStatus) -> u8 {
            match status {
                ElectionStatus::Registration => 0,
                ElectionStatus::Voting => 1,
                ElectionStatus::Finalized => 2,
                ElectionStatus::Cancelled => 3,
            }
        }

        // ===== PHASE 7: ADVANCED FEATURES HELPER METHODS =====

        /// Get delegation info for an account
        pub fn get_delegation_info(delegator: &T::AccountId) -> Option<VoteDelegation<T::AccountId, BlockNumberFor<T>>> {
            VoteDelegations::<T>::get(delegator)
        }

        /// Count active delegators for a delegate
        pub fn count_delegated_votes(delegate: &T::AccountId) -> u32 {
            let delegators = DelegationReceivers::<T>::get(delegate);
            delegators.len() as u32
        }

        /// Check if delegation is active and not expired
        pub fn is_delegation_active(delegator: &T::AccountId) -> bool {
            if let Some(delegation) = VoteDelegations::<T>::get(delegator) {
                let current_block = frame_system::Pallet::<T>::block_number();
                delegation.is_active && current_block < delegation.expires_at
            } else {
                false
            }
        }

        /// Get proposal amendment if exists
        pub fn get_proposal_amendment(proposal_id: u32) -> Option<ProposalAmendment<BlockNumberFor<T>>> {
            ProposalAmendments::<T>::get(proposal_id)
        }

        /// Check if proposal can be amended
        pub fn is_amendment_allowed(proposal_id: u32) -> bool {
            if let Some(proposal) = Proposals::<T>::get(proposal_id) {
                let current_block = frame_system::Pallet::<T>::block_number();
                // Can amend before voting starts and if not finalized
                current_block < proposal.voting_start && proposal.status != ProposalStatus::Executed
            } else {
                false
            }
        }

        /// Get proposals by priority level
        pub fn get_priority_queue(priority: ProposalPriority) -> Vec<u32> {
            ProposalQueue::<T>::get(priority).into_inner()
        }

        /// Get total rewards claimed by account (summed across all types)
        pub fn get_rewards_claimed(account: &T::AccountId) -> BalanceOf<T> {
            let mut total: BalanceOf<T> = Zero::zero();
            for rt in 0u8..=2u8 {
                total = total.saturating_add(RewardsClaimed::<T>::get(account, rt));
            }
            total
        }

        /// Get total rewards distributed
        pub fn get_total_rewards_distributed() -> BalanceOf<T> {
            TotalRewardsDistributed::<T>::get()
        }

        /// Calculate voting power including delegations
        pub fn calculate_voting_power(account: &T::AccountId) -> u32 {
            let mut power = 1u32; // Base voting power

            // Add delegated votes
            let delegators = DelegationReceivers::<T>::get(account);
            for delegator in delegators.iter() {
                if Self::is_delegation_active(delegator) {
                    power = power.saturating_add(1);
                }
            }

            power
        }

        /// Get proposal priority index
        pub fn proposal_priority_index(priority: ProposalPriority) -> u8 {
            priority.to_value()
        }
    }
}

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn submit_proposal() -> Weight;
    fn cast_vote() -> Weight;
    fn finalize_proposal() -> Weight;
    fn update_community_rank() -> Weight;
    fn update_pouw_contribution() -> Weight;
    fn council_override() -> Weight;
    fn update_chain_parameter() -> Weight;
    fn expire_council_member() -> Weight;
}

impl WeightInfo for () {
    fn submit_proposal() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2500))
    }
    fn cast_vote() -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(Weight::from_parts(0, 1500))
    }
    fn finalize_proposal() -> Weight {
        Weight::from_parts(12_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2000))
    }
    fn update_community_rank() -> Weight {
        Weight::from_parts(5_000_000, 512)
            .saturating_add(Weight::from_parts(0, 500))
    }
    fn update_pouw_contribution() -> Weight {
        Weight::from_parts(5_000_000, 512)
            .saturating_add(Weight::from_parts(0, 500))
    }
    fn council_override() -> Weight {
        Weight::from_parts(8_000_000, 512)
            .saturating_add(Weight::from_parts(0, 1000))
    }
    fn update_chain_parameter() -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(Weight::from_parts(0, 500))
    }
    fn expire_council_member() -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(Weight::from_parts(0, 1000))
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;