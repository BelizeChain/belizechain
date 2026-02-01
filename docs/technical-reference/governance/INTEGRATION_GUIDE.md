# Governance Pallet Integration Guide

## Table of Contents

1. [Runtime Integration](#runtime-integration)
2. [Config Trait Implementation](#config-trait-implementation)
3. [Integration with Economy Pallet](#integration-with-economy-pallet)
4. [Integration with Compliance Pallet](#integration-with-compliance-pallet)
5. [Integration with Identity Pallet](#integration-with-identity-pallet)
6. [Integration with Staking Pallet](#integration-with-staking-pallet)
7. [Testing Integration](#testing-integration)
8. [Common Integration Issues](#common-integration-issues)

---

## Runtime Integration

### Step 1: Add Dependencies

Add to your runtime's `Cargo.toml`:

```toml
[dependencies]
pallet-belize-governance = { path = "../pallets/governance", default-features = false }
pallet-belize-compliance = { path = "../pallets/compliance", default-features = false }
pallet-belize-economy = { path = "../pallets/economy", default-features = false }

[features]
std = [
    # ... existing pallets ...
    "pallet-belize-governance/std",
    "pallet-belize-compliance/std",
    "pallet-belize-economy/std",
]
```

### Step 2: Implement Config Trait

Add to your `runtime/src/lib.rs`:

```rust
use frame_support::traits::EnsureOrigin;

parameter_types! {
    pub const GovernancePalletId: PalletId = PalletId(*b"blz/govr");
    pub const MinimumDeposit: Balance = 100 * DOLLARS; // 100 DALLA
    pub const LaunchPeriod: BlockNumber = 28_800; // ~2 days at 6s/block
    pub const VotingPeriod: BlockNumber = 50_400; // ~7 days
    pub const MinimumParticipationRate: u8 = 33; // 33% quorum
    pub const CouncilSize: u32 = 12; // 6 districts * 2 seats
}

impl pallet_belize_governance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = pallet_belize_governance::weights::SubstrateWeight<Runtime>;
    type PalletId = GovernancePalletId;
    
    // Timing parameters
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type MinimumDeposit = MinimumDeposit;
    
    // Voting requirements
    type MinimumParticipationRate = MinimumParticipationRate;
    type CouncilSize = CouncilSize;
    
    // Compliance integration
    type ComplianceProvider = BelizeCompliance;
    
    // Origins for privileged operations
    type FSCOrigin = EnsureRoot<AccountId>; // TODO: Replace with FSC multi-sig
    type CouncilOrigin = EnsureRoot<AccountId>; // TODO: Replace with council collective
    
    // Randomness for elections
    type Randomness = RandomnessCollectiveFlip;
}
```

### Step 3: Add to Runtime Construction

Add to the `construct_runtime!` macro:

```rust
construct_runtime!(
    pub struct Runtime {
        // ... existing pallets ...
        
        BelizeCompliance: pallet_belize_compliance,
        BelizeGovernance: pallet_belize_governance,
        BelizeEconomy: pallet_belize_economy,
    }
);
```

### Step 4: Genesis Configuration

Add to your chain spec (`node/src/chain_spec.rs`):

```rust
fn genesis_config() -> GenesisConfig {
    GenesisConfig {
        // ... existing pallets ...
        
        belize_governance: BelizeGovernanceConfig {
            initial_council: vec![
                (alice_account(), BoardRole::Founder),
                (bob_account(), BoardRole::FSCRepresentative),
                (charlie_account(), BoardRole::TechnicalSteward),
            ],
            governance_parameters: vec![
                (GovernanceParameter::VotingPeriodBlocks, 50_400),
                (GovernanceParameter::MinProposalDeposit, 100_000_000_000_000), // 100 DALLA
                (GovernanceParameter::MinParticipationRate, 33),
            ],
        },
    }
}
```

---

## Config Trait Implementation

### Essential Types

```rust
/// Required traits for governance functionality
pub trait Config: frame_system::Config {
    /// Event type for governance events
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    /// Currency for deposits and treasury operations
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    
    /// Weight information for extrinsics
    type WeightInfo: WeightInfo;
    
    /// Pallet identifier for treasury management
    type PalletId: Get<PalletId>;
    
    /// Compliance provider for KYC verification
    type ComplianceProvider: ComplianceCheck<Self::AccountId>;
    
    /// Randomness source for council elections
    type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
    
    // ... parameter types ...
}
```

### Custom Origin Implementation

For production, replace `EnsureRoot` with proper multi-sig origins:

```rust
/// FSC multi-signature origin (4-of-7 signature requirement)
pub struct EnsureFSC;
impl EnsureOrigin<RuntimeOrigin> for EnsureFSC {
    type Success = AccountId;
    
    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        // Check if origin has FSC multi-sig approval
        match o.into() {
            Ok(RawOrigin::Signed(who)) => {
                if FSCMultisig::has_approval(&who) {
                    Ok(who)
                } else {
                    Err(RuntimeOrigin::from(RawOrigin::Signed(who)))
                }
            }
            r => Err(RuntimeOrigin::from(r)),
        }
    }
}

/// Council collective origin
pub struct EnsureCouncil;
impl EnsureOrigin<RuntimeOrigin> for EnsureCouncil {
    type Success = AccountId;
    
    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        match o.into() {
            Ok(RawOrigin::Signed(who)) => {
                if BelizeGovernance::is_council_member(&who) {
                    Ok(who)
                } else {
                    Err(RuntimeOrigin::from(RawOrigin::Signed(who)))
                }
            }
            r => Err(RuntimeOrigin::from(r)),
        }
    }
}
```

---

## Integration with Economy Pallet

The governance pallet interacts with the economy pallet for:
- **Treasury management**: Proposal-based spending
- **DALLA rewards**: Participation incentives
- **Multi-signature operations**: Secure treasury access

### Treasury Integration

```rust
// In pallet-belize-economy
pub trait TreasuryManager<AccountId, Balance> {
    /// Get current treasury balance
    fn treasury_balance() -> Balance;
    
    /// Transfer from treasury (requires governance approval)
    fn transfer_from_treasury(
        recipient: &AccountId,
        amount: Balance,
    ) -> DispatchResult;
    
    /// Reserve funds for proposal deposits
    fn reserve_proposal_deposit(
        account: &AccountId,
        amount: Balance,
    ) -> DispatchResult;
}

// Governance calls treasury
impl<T: Config> Pallet<T> {
    fn execute_treasury_transfer(
        recipient: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        T::Treasury::transfer_from_treasury(recipient, amount)?;
        Ok(())
    }
}
```

### DALLA Rewards Integration

```rust
// Claim participation rewards
pub fn claim_participation_reward(
    origin: OriginFor<T>,
    reward_type: u8,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    let reward_amount = match reward_type {
        0 => 10_000_000_000_000u128,  // 10 DALLA for voting
        1 => 100_000_000_000_000u128, // 100 DALLA for proposal
        2 => 500_000_000_000_000u128, // 500 DALLA for council service
        _ => return Err(Error::<T>::InvalidRewardType.into()),
    };
    
    // Verify participation
    ensure!(
        Self::has_participated(&who, reward_type),
        Error::<T>::NoRewardAvailable
    );
    
    // Transfer from economy pallet
    T::Economy::distribute_reward(&who, reward_amount)?;
    
    Ok(())
}
```

---

## Integration with Compliance Pallet

All governance operations require KYC verification through the compliance pallet.

### Compliance Provider Implementation

```rust
// In pallet-belize-compliance
pub trait ComplianceCheck<AccountId> {
    /// Check if account meets minimum governance requirements
    fn can_participate_in_governance(account: &AccountId) -> bool;
    
    /// Get verification level (0-4)
    fn get_verification_level(account: &AccountId) -> u8;
    
    /// Check specific tier requirement
    fn has_verification_tier(account: &AccountId, tier: u8) -> bool;
}

impl<T: Config> ComplianceCheck<T::AccountId> for Pallet<T> {
    fn can_participate_in_governance(account: &T::AccountId) -> bool {
        Self::verification_levels(account) >= 1 // Minimum: Basic KYC
    }
    
    fn get_verification_level(account: &T::AccountId) -> u8 {
        Self::verification_levels(account)
    }
    
    fn has_verification_tier(account: &T::AccountId, min_tier: u8) -> bool {
        Self::verification_levels(account) >= min_tier
    }
}
```

### Participation Tier Mapping

```rust
// Automatic tier assignment based on KYC level
pub fn get_participation_tier(account: &T::AccountId) -> ParticipationTier {
    let level = T::ComplianceProvider::get_verification_level(account);
    ParticipationTier::from_verification_level(level)
}

// Usage in extrinsics
pub fn submit_proposal(origin: OriginFor<T>, ...) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // Check compliance
    ensure!(
        T::ComplianceProvider::can_participate_in_governance(&who),
        Error::<T>::InsufficientCompliance
    );
    
    // Check tier for proposal creation (requires Contributor)
    let tier = Self::get_participation_tier(&who);
    ensure!(
        matches!(tier, ParticipationTier::Contributor | ParticipationTier::Validator),
        Error::<T>::InsufficientVerificationTier
    );
    
    // Continue with proposal creation...
}
```

---

## Integration with Identity Pallet

The identity pallet provides BelizeID credentials for governance participants.

### Identity Integration Points

```rust
// Check if account has valid BelizeID
pub fn verify_identity(account: &T::AccountId) -> DispatchResult {
    ensure!(
        T::Identity::has_identity(account),
        Error::<T>::NoIdentity
    );
    
    // Get identity data
    let identity = T::Identity::get_identity(account)?;
    
    // Verify citizenship or residency for governance
    ensure!(
        identity.citizenship_status.is_eligible_for_governance(),
        Error::<T>::NotEligibleForGovernance
    );
    
    Ok(())
}

// Link council members to BelizeID
pub fn register_council_member_identity(
    council_member: &T::AccountId,
) -> DispatchResult {
    // Verify Enhanced KYC through identity pallet
    ensure!(
        T::Identity::has_enhanced_verification(council_member),
        Error::<T>::RequiresEnhancedKYC
    );
    
    // Store council member identity link
    CouncilIdentities::<T>::insert(council_member, identity);
    
    Ok(())
}
```

---

## Integration with Staking Pallet

Governance uses Proof of Useful Work (PoUW) contributions for voting weight.

### PoUW Integration

```rust
// Get staking contribution for voting weight
pub fn calculate_voting_weight(account: &T::AccountId) -> u32 {
    let community_rank = CommunityRanks::<T>::get(account).unwrap_or(100);
    let pouw_contribution = T::Staking::get_pouw_contribution(account);
    
    // Base weight + PoUW bonus
    community_rank.saturating_add(pouw_contribution)
}

// Staking pallet trait
pub trait PoUWContribution<AccountId> {
    /// Get account's PoUW contribution score
    fn get_pouw_contribution(account: &AccountId) -> u32;
    
    /// Check if account is active validator
    fn is_active_validator(account: &AccountId) -> bool;
}

// Usage in voting
pub fn cast_vote(
    origin: OriginFor<T>,
    proposal_id: u32,
    vote_choice: VoteChoice,
    conviction: u8,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // Calculate weight including PoUW
    let base_weight = Self::calculate_voting_weight(&who);
    let final_weight = base_weight.saturating_mul(conviction as u32);
    
    // Record vote with PoUW-enhanced weight
    Votes::<T>::insert(proposal_id, &who, Vote {
        voter: who.clone(),
        vote_choice,
        weight: final_weight,
        conviction,
        block_number: current_block,
    });
    
    Ok(())
}
```

---

## Testing Integration

### Mock Setup

```rust
// tests/mock.rs
use frame_support::{parameter_types, construct_runtime};

// Mock compliance provider
impl pallet_belize_compliance::ComplianceCheck<AccountId> for MockCompliance {
    fn can_participate_in_governance(account: &AccountId) -> bool {
        // Test accounts have Basic KYC by default
        *account != RESTRICTED_ACCOUNT
    }
    
    fn get_verification_level(account: &AccountId) -> u8 {
        match account {
            ALICE => 3, // Enhanced (Validator tier)
            BOB => 2,   // Standard (Contributor tier)
            CHARLIE => 1, // Basic (Observer tier)
            _ => 0,     // None (restricted)
        }
    }
}

// Mock runtime
impl pallet_belize_governance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
    type PalletId = GovernancePalletId;
    type ComplianceProvider = MockCompliance;
    type Randomness = TestRandomness;
    // ... other types ...
}
```

### Integration Tests

```rust
#[test]
fn full_proposal_lifecycle_with_economy() {
    new_test_ext().execute_with(|| {
        // Setup: Alice has Validator tier, treasury has funds
        assert_eq!(MockCompliance::get_verification_level(&ALICE), 3);
        assert_eq!(Economy::treasury_balance(), 1_000_000);
        
        // 1. Create treasury spending proposal
        assert_ok!(Governance::submit_proposal(
            Origin::signed(ALICE),
            b"Road Repair".to_vec(),
            b"Allocate 50,000 DALLA".to_vec(),
            1, // Economic proposal
            0, // Simple majority
            false,
        ));
        
        // 2. Vote on proposal (multiple voters with PoUW weight)
        run_to_block(28_801); // After launch period
        assert_ok!(Governance::cast_vote(Origin::signed(ALICE), 0, 0, 1));
        assert_ok!(Governance::cast_vote(Origin::signed(BOB), 0, 0, 1));
        
        // 3. Check voting weight includes PoUW
        let alice_weight = Governance::calculate_voting_weight(&ALICE);
        assert_eq!(alice_weight, 150); // base 100 + 50 PoUW
        
        // 4. Execute approved proposal
        run_to_block(79_201); // After voting period
        assert_ok!(Governance::execute_proposal(Origin::signed(ALICE), 0));
        
        // 5. Verify treasury transfer occurred
        assert_eq!(Economy::treasury_balance(), 950_000);
        assert_eq!(Balances::free_balance(&RECIPIENT), 50_000);
        
        // 6. Claim participation rewards
        assert_ok!(Governance::claim_participation_reward(
            Origin::signed(ALICE),
            1, // Proposal reward (100 DALLA)
        ));
        assert_eq!(Balances::free_balance(&ALICE), original_balance + 100);
    });
}
```

---

## Common Integration Issues

### Issue 1: Compliance Provider Not Available

**Error**: `ComplianceProvider not found`

**Solution**: Ensure compliance pallet is included in runtime before governance:

```rust
construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Balances: pallet_balances,
        BelizeCompliance: pallet_belize_compliance, // Must come first
        BelizeGovernance: pallet_belize_governance, // Uses compliance
    }
);
```

### Issue 2: Origin Type Mismatch

**Error**: `expected EnsureOrigin, found EnsureRoot`

**Solution**: Use proper origin wrappers:

```rust
// Development (quick start)
type FSCOrigin = EnsureRoot<AccountId>;

// Production (recommended)
type FSCOrigin = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, FSCCollective, 4, 7>
>;
```

### Issue 3: Balance Type Mismatch

**Error**: `expected BalanceOf<T>, found u128`

**Solution**: Use proper type conversions:

```rust
// Correct
let amount: BalanceOf<T> = 100u128.saturated_into();
T::Currency::transfer(&from, &to, amount, ExistenceRequirement::KeepAlive)?;

// Incorrect
let amount = 100u128;
T::Currency::transfer(&from, &to, amount, ExistenceRequirement::KeepAlive)?; // Error!
```

### Issue 4: Weight Function Missing

**Error**: `WeightInfo trait not implemented`

**Solution**: Either use default weights or generate custom:

```rust
// Option 1: Use default weights (development)
type WeightInfo = ();

// Option 2: Use substrate weights (production)
type WeightInfo = pallet_belize_governance::weights::SubstrateWeight<Runtime>;

// Option 3: Custom weights (after benchmarking)
type WeightInfo = CustomWeights;
```

### Issue 5: Genesis Configuration Error

**Error**: `InitialCouncil not set`

**Solution**: Provide genesis configuration in chain spec:

```rust
belize_governance: BelizeGovernanceConfig {
    initial_council: vec![
        (root_account, BoardRole::Founder),
        (fsc_account, BoardRole::FSCRepresentative),
    ],
    governance_parameters: vec![
        (GovernanceParameter::VotingPeriodBlocks, 50_400),
        (GovernanceParameter::MinProposalDeposit, 100_000_000_000_000),
    ],
}
```

---

## Production Deployment Checklist

- [ ] **Origins Configured**: Replace `EnsureRoot` with proper multi-sig origins
- [ ] **Weights Calibrated**: Run benchmarks and update weight functions
- [ ] **Genesis Set**: Configure initial council and parameters
- [ ] **Compliance Active**: Ensure compliance pallet is operational
- [ ] **Treasury Funded**: Initialize governance treasury with sufficient DALLA
- [ ] **Monitoring Setup**: Deploy event monitoring for governance actions
- [ ] **Backup Procedures**: Document proposal recovery procedures
- [ ] **Security Audit**: Complete third-party security review
- [ ] **Documentation**: Deploy rustdoc and integration guides
- [ ] **Migration Plan**: Prepare for future runtime upgrades

---

## Support & Resources

- **Rustdoc**: Run `cargo doc --open -p pallet-belize-governance`
- **Example Runtime**: See `runtime/src/lib.rs` for complete integration
- **Test Suite**: 137 integration tests in `src/tests_*.rs`
- **Discord**: #governance-dev channel
- **GitHub Issues**: Report integration problems

For more details, see the [main governance documentation](./README.md) and [implementation plan](../../GOVERNANCE_IMPLEMENTATION_PLAN.md).
