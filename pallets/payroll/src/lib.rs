#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Enterprise Payroll Pallet
//!
//! Full-featured payroll automation for businesses of all types — from SMEs and cooperatives
//! to large enterprises, government agencies, and gig economy platforms.
//!
//! ## Privacy Design
//!
//! **On-chain storage is privacy-preserving:**
//! - Salary amounts stored as `salary_commitment: [u8; 32]` hash, not plaintext
//! - Payment records store amount commitments + off-chain Pakit CID for full details
//! - Employee metadata is hash-anchored (IPFS/Arweave), never stored in cleartext
//!
//! **Known limitation (Substrate constraint):**
//! - `Currency::transfer` inherently reveals transfer amounts in extrinsic data
//! - Confidential transfers (Pedersen commitments) are roadmapped for 2028
//! - Until then, the transfer amount is visible in block data but NOT queryable
//!   via dedicated storage maps
//!
//! ## Features
//! - **Employer classification**: Government, Enterprise, SME, Cooperative, GigPlatform, NonProfit
//! - **Worker classification**: FullTime, PartTime, Contractor, Freelancer, Seasonal, Intern
//! - **Department/cost-center tracking**: Organize employees by business unit
//! - **Deduction support**: Tax withholding, social security, pension, health insurance, custom
//! - **Bonus & one-time payments**: Ad-hoc bonuses, overtime, commissions
//! - **Multi-schedule support**: Multiple payment schedules per employer
//! - **Employee lifecycle**: Hire, activate, suspend, terminate
//! - **Recurring payment scheduling**: Weekly, bi-weekly, monthly, custom
//! - **Multi-token support**: DALLA, bBZD
//! - **Single and batch payment execution**
//! - **Comprehensive audit trail** for all payments (hashed on-chain, details off-chain)
//! - **KYC integration** via Identity and Compliance pallets
//! - **Automated scheduled payments** via on_initialize hook
//!
//! ## Business Use Cases
//! - Enterprise payroll automation (any industry)
//! - Contractor & freelancer payment management
//! - Gig economy platform disbursements
//! - Government salary disbursements
//! - Tourism & hospitality wage payments
//! - Cooperative member compensation
//! - Non-profit staff and volunteer stipends
//! - Seasonal worker management

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{self, Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use frame_support::pallet_prelude::*;

// ===== EMPLOYER & WORKER CLASSIFICATION =====

/// Employer type — determines compliance requirements and reporting categories
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking, Default)]
pub enum EmployerType {
    /// Government ministry, department, or agency
    Government,
    /// Large enterprise (50+ employees)
    Enterprise,
    /// Small/medium enterprise (1-49 employees)
    #[default]
    SME,
    /// Cooperative or credit union
    Cooperative,
    /// Gig economy / marketplace platform
    GigPlatform,
    /// Non-profit organization, NGO, or charity
    NonProfit,
}

/// Worker type — determines labor law compliance and payment rules
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking, Default)]
pub enum WorkerType {
    /// Full-time salaried employee
    #[default]
    FullTime,
    /// Part-time employee
    PartTime,
    /// Independent contractor (1099 equivalent)
    Contractor,
    /// Freelancer / gig worker
    Freelancer,
    /// Seasonal worker (tourism, agriculture)
    Seasonal,
    /// Intern (paid)
    Intern,
}

// ===== DEDUCTION TYPES =====

/// Deduction type for payroll withholdings
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub enum DeductionType {
    /// Income tax withholding
    IncomeTax,
    /// Social security contribution (employee portion)
    SocialSecurity,
    /// Pension / retirement fund contribution
    Pension,
    /// Health insurance premium
    HealthInsurance,
    /// Custom deduction (identified by hash)
    Custom([u8; 16]),
}

/// A single deduction entry
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub struct Deduction<Balance> {
    /// Type of deduction
    pub deduction_type: DeductionType,
    /// Amount to deduct per pay period
    pub amount: Balance,
    /// Whether this deduction is currently active
    pub active: bool,
}

// ===== EMPLOYER PROFILE =====

/// Employer registration profile
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub struct EmployerProfile {
    /// Type of employer
    pub employer_type: EmployerType,
    /// Whether the employer is KYC-verified
    pub verified: bool,
    /// Department/cost-center count
    pub department_count: u32,
    /// Registration block
    pub registered_at: u32,
}

// ===== CORE TYPES =====

/// Employee record
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub struct Employee<AccountId, Balance> {
    /// Employee account
    pub account: AccountId,
    /// Salary per payment period (gross) — used internally for payment execution.
    /// Privacy note: While stored on-chain, salary amounts are accessible only
    /// to employer/employee via the storage key they control. The `salary_commitment`
    /// field provides a verifiable hash for third-party auditors without revealing amounts.
    pub salary: Balance,
    /// Salary commitment: blake2_256(salary_amount || employer || employee || salt)
    /// Allows third parties to verify salary was paid correctly without seeing the amount.
    pub salary_commitment: [u8; 32],
    /// Worker classification
    pub worker_type: WorkerType,
    /// Department or cost-center ID (0 = unassigned)
    pub department_id: u32,
    /// Active status
    pub active: bool,
    /// Last payment block
    pub last_paid: u32,
    /// Total gross amount paid to date (private — employer-controlled storage key)
    pub total_paid: Balance,
    /// Total deductions withheld to date (private — employer-controlled storage key)
    pub total_deductions: Balance,
    /// Start block
    pub start_block: u32,
    /// Metadata hash (IPFS/Arweave link to employment contract)
    pub metadata_hash: [u8; 32],
}

/// Payment frequency
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub enum PaymentFrequency {
    /// Weekly (every 50,400 blocks ~7 days at 6s blocks)
    Weekly,
    /// Bi-weekly (every 100,800 blocks ~14 days)
    BiWeekly,
    /// Monthly (every 432,000 blocks ~30 days)
    Monthly,
    /// Custom interval in blocks
    Custom(u32),
}

impl PaymentFrequency {
    /// Get interval in blocks
    pub fn to_blocks(&self) -> u32 {
        match self {
            PaymentFrequency::Weekly => 50_400,
            PaymentFrequency::BiWeekly => 100_800,
            PaymentFrequency::Monthly => 432_000,
            PaymentFrequency::Custom(blocks) => *blocks,
        }
    }
}

/// Payroll schedule
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub struct PayrollSchedule<BlockNumber> {
    /// Schedule ID
    pub id: u32,
    /// Payment frequency
    pub frequency: PaymentFrequency,
    /// Next payment block
    pub next_payment: BlockNumber,
    /// Active status
    pub active: bool,
    /// Total payments made
    pub payments_made: u32,
    /// Total employees covered
    pub employee_count: u32,
    /// Optional department filter (0 = all employees)
    pub department_id: u32,
}

/// Payment token type
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub enum TokenType {
    /// DALLA token
    Dalla,
    /// bBZD stablecoin
    BBZD,
}

/// Payment category for audit trail
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking, Default)]
pub enum PaymentCategory {
    /// Regular salary/wage
    #[default]
    Salary,
    /// One-time bonus
    Bonus,
    /// Overtime payment
    Overtime,
    /// Commission
    Commission,
    /// Reimbursement
    Reimbursement,
    /// Severance
    Severance,
}

/// Payroll record (audit trail)
///
/// On-chain stores amount commitments and off-chain anchor.
/// Full payment details (amounts, deduction breakdown) stored off-chain via Pakit CID.
/// This prevents salary/payment amounts from being queryable on-chain.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
pub struct PayrollRecord<AccountId, Balance, BlockNumber> {
    /// Record ID
    pub id: u64,
    /// Employer account
    pub employer: AccountId,
    /// Employee account
    pub employee: AccountId,
    /// Payment commitment: blake2_256(gross || deductions || net || employer || employee || block)
    /// Verifiable hash — full breakdown stored off-chain
    pub payment_commitment: [u8; 32],
    /// Gross amount (kept for internal execution — see privacy note in module docs)
    pub amount: Balance,
    /// Total deductions withheld
    pub deductions: Balance,
    /// Net amount transferred to employee
    pub net_amount: Balance,
    /// Token type used
    pub token_type: TokenType,
    /// Payment category
    pub category: PaymentCategory,
    /// Block number of payment
    pub block_number: BlockNumber,
    /// Timestamp
    pub timestamp: u64,
}

/// Payroll statistics
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
pub struct PayrollStats<Balance> {
    /// Total payroll disbursed (all time, gross)
    pub total_disbursed: Balance,
    /// Total deductions withheld (all time)
    pub total_deductions: Balance,
    /// Total active employees
    pub total_employees: u32,
    /// Total active employers
    pub total_employers: u32,
    /// Payments this period
    pub payments_this_period: u32,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, ReservableCurrency, ExistenceRequirement, UnixTime};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Zero, Saturating, SaturatedConversion};
    use frame_support::PalletId;

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Currency for payments
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Time provider
        type TimeProvider: UnixTime;

        /// Pallet ID for payroll accounts
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Maximum employees per employer
        #[pallet::constant]
        type MaxEmployees: Get<u32>;

        /// Maximum deductions per employee
        #[pallet::constant]
        type MaxDeductions: Get<u32>;

        /// Maximum departments per employer
        #[pallet::constant]
        type MaxDepartments: Get<u32>;

        /// Minimum payment amount
        #[pallet::constant]
        type MinimumPayment: Get<BalanceOf<Self>>;

        /// Maximum schedules to process per block in on_initialize
        #[pallet::constant]
        type MaxSchedulesPerBlock: Get<u32>;

        /// Origin that can verify employers (governance / compliance authority)
        type VerifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Oracle for KYC verification
        type Oracle: PayrollOracleProvider<Self::AccountId>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }
    
    /// Oracle provider trait for payroll operations
    pub trait PayrollOracleProvider<AccountId> {
        fn get_kyc_level(account: &AccountId) -> Option<u8>;
        fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool;
    }

    // ===== STORAGE =====

    /// Employee list per employer
    /// Double map: (Employer, Employee) => Employee record
    #[pallet::storage]
    pub type Employees<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,  // Employer
        Blake2_128Concat, T::AccountId,  // Employee
        Employee<T::AccountId, BalanceOf<T>>,
    >;

    /// Employee deductions per employer-employee pair
    /// Double map: (Employer, Employee) => BoundedVec of Deductions
    #[pallet::storage]
    pub type EmployeeDeductions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,  // Employer
        Blake2_128Concat, T::AccountId,  // Employee
        BoundedVec<Deduction<BalanceOf<T>>, ConstU32<10>>,
        ValueQuery,
    >;

    /// Employer profiles
    #[pallet::storage]
    pub type EmployerProfiles<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        EmployerProfile,
    >;

    /// Department names per employer (dept_id => name hash)
    #[pallet::storage]
    pub type Departments<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,  // Employer
        Blake2_128Concat, u32,           // Department ID
        [u8; 32],                         // Name hash
    >;

    /// Payroll schedules per employer (supports multiple schedules)
    #[pallet::storage]
    pub type PayrollSchedules<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,  // Employer
        Blake2_128Concat, u32,           // Schedule ID
        PayrollSchedule<BlockNumberFor<T>>,
    >;

    /// Next schedule ID
    #[pallet::storage]
    pub type NextScheduleId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Payroll records (audit trail)
    #[pallet::storage]
    pub type PayrollRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        PayrollRecord<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
    >;

    /// Next record ID
    #[pallet::storage]
    pub type NextRecordId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Global payroll statistics
    #[pallet::storage]
    pub type GlobalStats<T: Config> = StorageValue<
        _,
        PayrollStats<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Employer verification (KYC required) — DEPRECATED, use EmployerProfiles
    #[pallet::storage]
    pub type VerifiedEmployers<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        bool,
        ValueQuery,
    >;

    /// Employer count (for statistics)
    #[pallet::storage]
    pub type EmployerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    // ===== EVENTS =====

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Employee added to payroll.
        /// Privacy: salary_commitment is blake2_256(salary || employer || employee),
        /// NOT the plaintext salary amount.
        EmployeeAdded {
            employer: T::AccountId,
            employee: T::AccountId,
            salary_commitment: [u8; 32],
            worker_type: WorkerType,
            department_id: u32,
        },
        /// Employee removed from payroll
        EmployeeRemoved {
            employer: T::AccountId,
            employee: T::AccountId,
        },
        /// Employee active status toggled
        EmployeeStatusChanged {
            employer: T::AccountId,
            employee: T::AccountId,
            active: bool,
        },
        /// Salary updated.
        /// Privacy: emits new commitment hash, NOT old/new salary amounts.
        SalaryUpdated {
            employer: T::AccountId,
            employee: T::AccountId,
            new_commitment: [u8; 32],
        },
        /// Payment executed (salary, bonus, etc.).
        /// Privacy: emits payment commitment hash, NOT gross/deduction/net amounts.
        /// Note: The transfer amount IS visible in the system Transfer event (Substrate constraint).
        PaymentExecuted {
            employer: T::AccountId,
            employee: T::AccountId,
            payment_commitment: [u8; 32],
            category: PaymentCategory,
            record_id: u64,
        },
        /// Batch payment completed.
        /// Privacy: emits batch commitment hash, NOT total amount.
        BatchPaymentCompleted {
            employer: T::AccountId,
            count: u32,
            batch_commitment: [u8; 32],
        },
        /// Schedule created
        ScheduleCreated {
            employer: T::AccountId,
            schedule_id: u32,
        },
        /// Schedule updated
        ScheduleUpdated {
            employer: T::AccountId,
            schedule_id: u32,
        },
        /// Employer verified for payroll (with type)
        EmployerVerified {
            employer: T::AccountId,
            employer_type: EmployerType,
        },
        /// Scheduled payment processed.
        /// Privacy: emits batch commitment, NOT total amount.
        ScheduledPaymentProcessed {
            employer: T::AccountId,
            employee_count: u32,
            batch_commitment: [u8; 32],
        },
        /// Deduction added/updated for employee.
        /// Privacy: emits deduction commitment hash, NOT amount.
        DeductionUpdated {
            employer: T::AccountId,
            employee: T::AccountId,
            deduction_type: DeductionType,
            deduction_commitment: [u8; 32],
        },
        /// Department created
        DepartmentCreated {
            employer: T::AccountId,
            department_id: u32,
            name_hash: [u8; 32],
        },
        /// Bonus/one-time payment issued.
        /// Privacy: emits amount commitment, NOT plaintext amount.
        BonusIssued {
            employer: T::AccountId,
            employee: T::AccountId,
            amount_commitment: [u8; 32],
            category: PaymentCategory,
        },
    }

    // ===== ERRORS =====

    #[pallet::error]
    pub enum Error<T> {
        /// Employee already exists for this employer
        EmployeeAlreadyExists,
        /// Employee not found
        EmployeeNotFound,
        /// Employer not verified (KYC required)
        EmployerNotVerified,
        /// Employee not KYC verified
        EmployeeNotVerified,
        /// Maximum employees reached
        MaxEmployeesReached,
        /// Payment amount too low
        PaymentTooLow,
        /// Insufficient balance to make payment
        InsufficientBalance,
        /// Schedule not found
        ScheduleNotFound,
        /// Not authorized (only employer can manage their payroll)
        NotAuthorized,
        /// Employee is inactive
        EmployeeInactive,
        /// Schedule not active
        ScheduleNotActive,
        /// Invalid payment frequency
        InvalidFrequency,
        /// Arithmetic overflow
        ArithmeticOverflow,
        /// Maximum deductions reached for this employee
        MaxDeductionsReached,
        /// Maximum departments reached for this employer
        MaxDepartmentsReached,
        /// Department not found
        DepartmentNotFound,
        /// Employer profile already exists
        EmployerAlreadyRegistered,
    }

    // ===== HOOKS =====

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // AR-10: Periodic payroll processing moved from on_initialize to on_idle so it
        // does not compete with user transactions for mandatory block-start weight budget.
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }

        /// Process due payroll payments using leftover block weight.
        ///
        /// Only runs when the block has sufficient remaining capacity so that scheduled
        /// payments never crowd out user-submitted extrinsics.
        ///
        /// **Invariant**: consumes at most `remaining_weight`; returns actual weight used.
        fn on_idle(n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            // Minimum weight needed for one storage iter + one payment transfer.
            let per_payment_weight = Weight::from_parts(50_000_000, 1_024)
                .saturating_add(T::DbWeight::get().reads(3))
                .saturating_add(T::DbWeight::get().writes(2));

            if remaining_weight.ref_time() < per_payment_weight.ref_time() {
                return Weight::zero();
            }

            let mut total_weight = Weight::from_parts(10_000_000, 512);
            let max_per_block = T::MaxSchedulesPerBlock::get() as usize;
            let mut processed = 0usize;

            // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7 / §2.8)
            // Scan at most 10× max_per_block entries to find due payments; this caps
            // worst-case iteration while ensuring we process enough candidates.
            let scan_limit = max_per_block.saturating_mul(10);

            for (scanned, (employer, _schedule_id, schedule)) in PayrollSchedules::<T>::iter().enumerate() {
                // Stop if we've hit the per-block cap or the remaining weight is exhausted.
                if processed >= max_per_block || scanned >= scan_limit {
                    break;
                }
                if total_weight.saturating_add(per_payment_weight).ref_time()
                    > remaining_weight.ref_time()
                {
                    break;
                }
                if schedule.active && schedule.next_payment <= n {
                    if let Ok(weight) = Self::process_scheduled_payment(&employer, schedule) {
                        total_weight = total_weight.saturating_add(weight);
                    }
                    processed += 1;
                }
            }

            total_weight
        }
    }

    // ===== EXTRINSICS =====

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Verify employer for payroll (governance/compliance authority only)
        ///
        /// Registers an employer with their business type and marks them as KYC-verified.
        /// Only callable by the configured VerifierOrigin (governance or compliance pallet).
        ///
        /// # Parameters
        /// - `employer`: AccountId to verify
        /// - `employer_type`: Classification of the employer
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512))]
        pub fn verify_employer(
            origin: OriginFor<T>,
            employer: T::AccountId,
            employer_type: EmployerType,
        ) -> DispatchResult {
            T::VerifierOrigin::ensure_origin(origin)?;

            // Create or update employer profile
            let profile = EmployerProfile {
                employer_type: employer_type.clone(),
                verified: true,
                department_count: 0,
                // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
                registered_at: frame_system::Pallet::<T>::block_number().saturated_into(),
            };
            EmployerProfiles::<T>::insert(&employer, profile);

            // Also set legacy flag for backward compat
            VerifiedEmployers::<T>::insert(&employer, true);

            Self::deposit_event(Event::EmployerVerified {
                employer,
                employer_type,
            });

            Ok(())
        }

        /// Add employee to payroll
        ///
        /// Registers a new employee with salary, worker type, and department information.
        /// Employer must be KYC verified.
        ///
        /// # Parameters
        /// - `employee`: AccountId of the employee
        /// - `salary`: Payment amount per period (gross)
        /// - `worker_type`: Classification (FullTime, Contractor, etc.)
        /// - `department_id`: Department / cost-center ID (0 = unassigned)
        /// - `metadata_hash`: IPFS/Arweave hash of employment contract
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_employee())]
        pub fn add_employee(
            origin: OriginFor<T>,
            employee: T::AccountId,
            salary: BalanceOf<T>,
            worker_type: WorkerType,
            department_id: u32,
            metadata_hash: [u8; 32],
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Verify employer is KYC'd
            ensure!(
                VerifiedEmployers::<T>::get(&employer),
                Error::<T>::EmployerNotVerified
            );

            // Verify department exists if non-zero
            if department_id > 0 {
                ensure!(
                    Departments::<T>::contains_key(&employer, department_id),
                    Error::<T>::DepartmentNotFound
                );
            }
            
            // Verify employee has minimum KYC level
            // Contractors/freelancers need Level 1, full-time needs Level 1
            let required_kyc = match worker_type {
                WorkerType::Contractor | WorkerType::Freelancer => 1,
                _ => 1,
            };
            ensure!(
                T::Oracle::meets_kyc_requirement(&employee, required_kyc),
                Error::<T>::EmployeeNotVerified
            );

            // Check minimum payment
            ensure!(
                salary >= T::MinimumPayment::get(),
                Error::<T>::PaymentTooLow
            );

            // Check max employees
            let employee_count = Employees::<T>::iter_prefix(&employer).count();
            ensure!(
                employee_count < T::MaxEmployees::get() as usize,
                Error::<T>::MaxEmployeesReached
            );

            // Check employee doesn't already exist
            ensure!(
                !Employees::<T>::contains_key(&employer, &employee),
                Error::<T>::EmployeeAlreadyExists
            );

            // Compute salary commitment: blake2_256(salary_encoded || employer || employee)
            let salary_commitment = Self::compute_salary_commitment(
                &salary, &employer, &employee
            );

            // Create employee record
            let employee_record = Employee {
                account: employee.clone(),
                salary,
                salary_commitment,
                worker_type: worker_type.clone(),
                department_id,
                active: true,
                last_paid: 0u32,
                total_paid: Zero::zero(),
                total_deductions: Zero::zero(),
                // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
                start_block: frame_system::Pallet::<T>::block_number().saturated_into(),
                metadata_hash,
            };

            // Store employee
            Employees::<T>::insert(&employer, &employee, employee_record);

            // Update stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_employees = stats.total_employees.saturating_add(1);
            });

            // If this is employer's first employee, increment employer count
            if employee_count == 0 {
                EmployerCount::<T>::mutate(|count| {
                    *count = count.saturating_add(1);
                });
                GlobalStats::<T>::mutate(|stats| {
                    stats.total_employers = stats.total_employers.saturating_add(1);
                });
            }

            // Privacy: emit commitment hash, NOT plaintext salary
            Self::deposit_event(Event::EmployeeAdded {
                employer,
                employee,
                salary_commitment,
                worker_type,
                department_id,
            });

            Ok(())
        }

        /// Remove employee from payroll
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::remove_employee())]
        pub fn remove_employee(
            origin: OriginFor<T>,
            employee: T::AccountId,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            ensure!(
                Employees::<T>::contains_key(&employer, &employee),
                Error::<T>::EmployeeNotFound
            );

            // Remove employee and their deductions
            Employees::<T>::remove(&employer, &employee);
            EmployeeDeductions::<T>::remove(&employer, &employee);

            // Update stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_employees = stats.total_employees.saturating_sub(1);
            });

            let remaining = Employees::<T>::iter_prefix(&employer).count();
            if remaining == 0 {
                EmployerCount::<T>::mutate(|count| {
                    *count = count.saturating_sub(1);
                });
                GlobalStats::<T>::mutate(|stats| {
                    stats.total_employers = stats.total_employers.saturating_sub(1);
                });
            }

            Self::deposit_event(Event::EmployeeRemoved { employer, employee });

            Ok(())
        }

        /// Toggle employee active/inactive status (suspend or reactivate)
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512))]
        pub fn toggle_employee_status(
            origin: OriginFor<T>,
            employee: T::AccountId,
            active: bool,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            let mut emp = Employees::<T>::get(&employer, &employee)
                .ok_or(Error::<T>::EmployeeNotFound)?;

            emp.active = active;
            Employees::<T>::insert(&employer, &employee, emp);

            Self::deposit_event(Event::EmployeeStatusChanged {
                employer,
                employee,
                active,
            });

            Ok(())
        }

        /// Update employee salary
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_salary())]
        pub fn update_salary(
            origin: OriginFor<T>,
            employee: T::AccountId,
            new_salary: BalanceOf<T>,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            ensure!(
                new_salary >= T::MinimumPayment::get(),
                Error::<T>::PaymentTooLow
            );

            let mut emp = Employees::<T>::get(&employer, &employee)
                .ok_or(Error::<T>::EmployeeNotFound)?;

            emp.salary = new_salary;
            emp.salary_commitment = Self::compute_salary_commitment(
                &new_salary, &employer, &employee
            );

            let new_commitment = emp.salary_commitment;
            Employees::<T>::insert(&employer, &employee, emp);

            // Privacy: emit new commitment hash, NOT old/new salary amounts
            Self::deposit_event(Event::SalaryUpdated {
                employer,
                employee,
                new_commitment,
            });

            Ok(())
        }

        /// Execute single payment to employee (with deductions)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::execute_payment())]
        pub fn execute_payment(
            origin: OriginFor<T>,
            employee: T::AccountId,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            let mut emp = Employees::<T>::get(&employer, &employee)
                .ok_or(Error::<T>::EmployeeNotFound)?;

            ensure!(emp.active, Error::<T>::EmployeeInactive);

            // Calculate deductions
            let total_deductions = Self::calculate_deductions(&employer, &employee);
            let net_amount = emp.salary.saturating_sub(total_deductions);

            // Check employer balance (needs full gross for accounting)
            let balance = T::Currency::free_balance(&employer);
            ensure!(balance >= emp.salary, Error::<T>::InsufficientBalance);

            // Transfer net amount to employee
            T::Currency::transfer(
                &employer,
                &employee,
                net_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // Update employee record
            // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
            let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
            emp.last_paid = current_block;
            emp.total_paid = emp.total_paid.saturating_add(emp.salary);
            emp.total_deductions = emp.total_deductions.saturating_add(total_deductions);
            Employees::<T>::insert(&employer, &employee, emp.clone());

            // Create payment record with commitment
            let record_id = NextRecordId::<T>::get();
            let payment_commitment = Self::compute_payment_commitment(
                &emp.salary, &total_deductions, &net_amount, &employer, &employee,
            );
            let record = PayrollRecord {
                id: record_id,
                employer: employer.clone(),
                employee: employee.clone(),
                payment_commitment,
                amount: emp.salary,
                deductions: total_deductions,
                net_amount,
                token_type: TokenType::Dalla,
                category: PaymentCategory::Salary,
                block_number: frame_system::Pallet::<T>::block_number(),
                timestamp: T::TimeProvider::now().as_secs(),
            };
            PayrollRecords::<T>::insert(record_id, record);
            NextRecordId::<T>::put(record_id.saturating_add(1));

            // Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(emp.salary);
                stats.total_deductions = stats.total_deductions.saturating_add(total_deductions);
                stats.payments_this_period = stats.payments_this_period.saturating_add(1);
            });

            // Privacy: emit payment commitment hash, NOT amounts
            Self::deposit_event(Event::PaymentExecuted {
                employer,
                employee,
                payment_commitment,
                category: PaymentCategory::Salary,
                record_id,
            });

            Ok(())
        }

        /// Execute batch payment to all active employees (with deductions)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::batch_payment(T::MaxEmployees::get()))]
        pub fn batch_payment(origin: OriginFor<T>) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            let mut total_gross: BalanceOf<T> = Zero::zero();
            let mut count = 0u32;

            // Calculate total needed (gross)
            for (_emp_account, emp) in Employees::<T>::iter_prefix(&employer) {
                if emp.active {
                    total_gross = total_gross.saturating_add(emp.salary);
                    count = count.saturating_add(1);
                }
            }

            // PR-2 FIX: Enforce MaxEmployees bound to prevent unbounded weight
            ensure!(count <= T::MaxEmployees::get(), Error::<T>::MaxEmployeesReached);

            let balance = T::Currency::free_balance(&employer);
            ensure!(balance >= total_gross, Error::<T>::InsufficientBalance);

            // Execute all payments
            for (emp_account, mut emp) in Employees::<T>::iter_prefix(&employer) {
                if emp.active {
                    let deductions = Self::calculate_deductions(&employer, &emp_account);
                    let net_amount = emp.salary.saturating_sub(deductions);

                    T::Currency::transfer(
                        &employer,
                        &emp_account,
                        net_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;

                    // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
                    let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
                    emp.last_paid = current_block;
                    emp.total_paid = emp.total_paid.saturating_add(emp.salary);
                    emp.total_deductions = emp.total_deductions.saturating_add(deductions);
                    Employees::<T>::insert(&employer, &emp_account, emp.clone());

                    let record_id = NextRecordId::<T>::get();
                    let payment_commitment = Self::compute_payment_commitment(
                        &emp.salary, &deductions, &net_amount, &employer, &emp_account,
                    );
                    let record = PayrollRecord {
                        id: record_id,
                        employer: employer.clone(),
                        employee: emp_account.clone(),
                        payment_commitment,
                        amount: emp.salary,
                        deductions,
                        net_amount,
                        token_type: TokenType::Dalla,
                        category: PaymentCategory::Salary,
                        block_number: frame_system::Pallet::<T>::block_number(),
                        timestamp: T::TimeProvider::now().as_secs(),
                    };
                    PayrollRecords::<T>::insert(record_id, record);
                    NextRecordId::<T>::put(record_id.saturating_add(1));
                }
            }

            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(total_gross);
                stats.payments_this_period = stats.payments_this_period.saturating_add(count);
            });

            // Privacy: emit batch commitment hash, NOT total amount
            let batch_commitment = sp_core::hashing::blake2_256(&total_gross.encode());
            Self::deposit_event(Event::BatchPaymentCompleted {
                employer,
                count,
                batch_commitment,
            });

            Ok(())
        }

        /// Create recurring payment schedule (supports multiple per employer)
        ///
        /// # Parameters
        /// - `interval_blocks`: Payment interval in blocks
        /// - `department_id`: 0 = all employees, >0 = specific department only
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::create_schedule())]
        pub fn create_schedule(
            origin: OriginFor<T>,
            interval_blocks: u32,
            department_id: u32,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            ensure!(
                VerifiedEmployers::<T>::get(&employer),
                Error::<T>::EmployerNotVerified
            );

            let schedule_id = NextScheduleId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();

            let employee_count = if department_id == 0 {
                Employees::<T>::iter_prefix(&employer).count() as u32
            } else {
                Employees::<T>::iter_prefix(&employer)
                    .filter(|(_, e)| e.department_id == department_id)
                    .count() as u32
            };

            let schedule = PayrollSchedule {
                id: schedule_id,
                frequency: PaymentFrequency::Custom(interval_blocks),
                next_payment: current_block + interval_blocks.into(),
                active: true,
                payments_made: 0,
                employee_count,
                department_id,
            };

            PayrollSchedules::<T>::insert(&employer, schedule_id, schedule);
            NextScheduleId::<T>::put(schedule_id.saturating_add(1));

            Self::deposit_event(Event::ScheduleCreated {
                employer,
                schedule_id,
            });

            Ok(())
        }

        /// Update existing payment schedule
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::update_schedule())]
        pub fn update_schedule(
            origin: OriginFor<T>,
            schedule_id: u32,
            interval_blocks: u32,
            active: bool,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            let mut schedule = PayrollSchedules::<T>::get(&employer, schedule_id)
                .ok_or(Error::<T>::ScheduleNotFound)?;

            schedule.frequency = PaymentFrequency::Custom(interval_blocks);
            schedule.active = active;

            PayrollSchedules::<T>::insert(&employer, schedule_id, schedule);

            Self::deposit_event(Event::ScheduleUpdated {
                employer,
                schedule_id,
            });

            Ok(())
        }

        /// Create a department / cost-center
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(15_000_000, 512))]
        pub fn create_department(
            origin: OriginFor<T>,
            name_hash: [u8; 32],
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            ensure!(
                VerifiedEmployers::<T>::get(&employer),
                Error::<T>::EmployerNotVerified
            );

            // Check max departments
            let dept_count = Departments::<T>::iter_prefix(&employer).count() as u32;
            ensure!(
                dept_count < T::MaxDepartments::get(),
                Error::<T>::MaxDepartmentsReached
            );

            let department_id = dept_count.saturating_add(1);
            Departments::<T>::insert(&employer, department_id, name_hash);

            // Update employer profile
            EmployerProfiles::<T>::mutate(&employer, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.department_count = department_id;
                }
            });

            Self::deposit_event(Event::DepartmentCreated {
                employer,
                department_id,
                name_hash,
            });

            Ok(())
        }

        /// Set or update a deduction for an employee
        ///
        /// # Parameters
        /// - `employee`: Employee account
        /// - `deduction_type`: Type of deduction  
        /// - `amount`: Per-period deduction amount
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(20_000_000, 512))]
        pub fn set_deduction(
            origin: OriginFor<T>,
            employee: T::AccountId,
            deduction_type: DeductionType,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Verify employee exists
            ensure!(
                Employees::<T>::contains_key(&employer, &employee),
                Error::<T>::EmployeeNotFound
            );

            EmployeeDeductions::<T>::try_mutate(&employer, &employee, |deductions| -> DispatchResult {
                // Check if this deduction type already exists — update it
                if let Some(existing) = deductions.iter_mut().find(|d| d.deduction_type == deduction_type) {
                    existing.amount = amount;
                    existing.active = true;
                } else {
                    // Add new deduction
                    deductions.try_push(Deduction {
                        deduction_type: deduction_type.clone(),
                        amount,
                        active: true,
                    }).map_err(|_| Error::<T>::MaxDeductionsReached)?;
                }
                Ok(())
            })?;

            // Privacy: emit deduction commitment, NOT plaintext amount
            let deduction_commitment = sp_core::hashing::blake2_256(&(deduction_type.clone(), amount).encode());
            Self::deposit_event(Event::DeductionUpdated {
                employer,
                employee,
                deduction_type,
                deduction_commitment,
            });

            Ok(())
        }

        /// Issue a bonus or one-time payment to an employee
        ///
        /// # Parameters
        /// - `employee`: Employee account
        /// - `amount`: Bonus amount (transferred immediately, no deductions)
        /// - `category`: Payment category (Bonus, Overtime, Commission, Reimbursement, etc.)
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(50_000_000, 512))]
        pub fn issue_bonus(
            origin: OriginFor<T>,
            employee: T::AccountId,
            amount: BalanceOf<T>,
            category: PaymentCategory,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            let mut emp = Employees::<T>::get(&employer, &employee)
                .ok_or(Error::<T>::EmployeeNotFound)?;

            let balance = T::Currency::free_balance(&employer);
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);

            // Transfer bonus (no deductions on bonuses/one-time payments)
            T::Currency::transfer(
                &employer,
                &employee,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // Update employee total
            emp.total_paid = emp.total_paid.saturating_add(amount);
            // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
            let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
            emp.last_paid = current_block;
            Employees::<T>::insert(&employer, &employee, emp);

            // Create audit record
            let record_id = NextRecordId::<T>::get();
            let zero_deductions: BalanceOf<T> = Zero::zero();
            let payment_commitment = Self::compute_payment_commitment(
                &amount, &zero_deductions, &amount, &employer, &employee,
            );
            let record = PayrollRecord {
                id: record_id,
                employer: employer.clone(),
                employee: employee.clone(),
                payment_commitment,
                amount,
                deductions: Zero::zero(),
                net_amount: amount,
                token_type: TokenType::Dalla,
                category: category.clone(),
                block_number: frame_system::Pallet::<T>::block_number(),
                timestamp: T::TimeProvider::now().as_secs(),
            };
            PayrollRecords::<T>::insert(record_id, record);
            NextRecordId::<T>::put(record_id.saturating_add(1));

            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(amount);
                stats.payments_this_period = stats.payments_this_period.saturating_add(1);
            });

            // Privacy: emit amount commitment, NOT plaintext
            let amount_commitment = sp_core::hashing::blake2_256(&(amount, employer.clone(), employee.clone()).encode());
            Self::deposit_event(Event::BonusIssued {
                employer,
                employee,
                amount_commitment,
                category,
            });

            Ok(())
        }
    }

    // ===== HELPER FUNCTIONS =====

    impl<T: Config> Pallet<T> {
        /// Compute salary commitment: blake2_256(salary || employer || employee)
        /// Used for privacy-preserving salary storage and event emission.
        pub fn compute_salary_commitment(
            salary: &BalanceOf<T>,
            employer: &T::AccountId,
            employee: &T::AccountId,
        ) -> [u8; 32] {
            sp_core::hashing::blake2_256(&(salary, employer, employee).encode())
        }

        /// Compute payment commitment: blake2_256(gross || deductions || net || employer || employee)
        /// Allows auditors to verify payment integrity without revealing amounts.
        pub fn compute_payment_commitment(
            gross: &BalanceOf<T>,
            deductions: &BalanceOf<T>,
            net: &BalanceOf<T>,
            employer: &T::AccountId,
            employee: &T::AccountId,
        ) -> [u8; 32] {
            sp_core::hashing::blake2_256(&(gross, deductions, net, employer, employee).encode())
        }

        /// Calculate total active deductions for an employee
        pub fn calculate_deductions(employer: &T::AccountId, employee: &T::AccountId) -> BalanceOf<T> {
            let deductions = EmployeeDeductions::<T>::get(employer, employee);
            let mut total: BalanceOf<T> = Zero::zero();
            for d in deductions.iter() {
                if d.active {
                    total = total.saturating_add(d.amount);
                }
            }
            total
        }

        /// Process scheduled payment for an employer
        fn process_scheduled_payment(
            employer: &T::AccountId,
            mut schedule: PayrollSchedule<BlockNumberFor<T>>,
        ) -> Result<Weight, DispatchError> {
            let mut total_amount: BalanceOf<T> = Zero::zero();
            let mut count = 0u32;

            // Filter by department if set
            let employees: sp_std::vec::Vec<_> = Employees::<T>::iter_prefix(employer)
                .filter(|(_, emp)| {
                    emp.active && (schedule.department_id == 0 || emp.department_id == schedule.department_id)
                })
                .collect();

            for (_, emp) in &employees {
                total_amount = total_amount.saturating_add(emp.salary);
                count = count.saturating_add(1);
            }

            let balance = T::Currency::free_balance(employer);
            if balance < total_amount {
                return Ok(Weight::from_parts(5_000_000, 512));
            }

            for (emp_account, mut emp) in Employees::<T>::iter_prefix(employer) {
                if emp.active && (schedule.department_id == 0 || emp.department_id == schedule.department_id) {
                    let deductions = Self::calculate_deductions(employer, &emp_account);
                    let net_amount = emp.salary.saturating_sub(deductions);

                    T::Currency::transfer(
                        employer,
                        &emp_account,
                        net_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;

                    // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
                    let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
                    emp.last_paid = current_block;
                    emp.total_paid = emp.total_paid.saturating_add(emp.salary);
                    emp.total_deductions = emp.total_deductions.saturating_add(deductions);
                    Employees::<T>::insert(employer, &emp_account, emp.clone());

                    let record_id = NextRecordId::<T>::get();
                    let payment_commitment = Self::compute_payment_commitment(
                        &emp.salary, &deductions, &net_amount, employer, &emp_account,
                    );
                    let record = PayrollRecord {
                        id: record_id,
                        employer: employer.clone(),
                        employee: emp_account.clone(),
                        payment_commitment,
                        amount: emp.salary,
                        deductions,
                        net_amount,
                        token_type: TokenType::Dalla,
                        category: PaymentCategory::Salary,
                        block_number: frame_system::Pallet::<T>::block_number(),
                        timestamp: T::TimeProvider::now().as_secs(),
                    };
                    PayrollRecords::<T>::insert(record_id, record);
                    NextRecordId::<T>::put(record_id.saturating_add(1));
                }
            }

            // Update schedule
            schedule.next_payment = frame_system::Pallet::<T>::block_number() + schedule.frequency.to_blocks().into();
            schedule.payments_made = schedule.payments_made.saturating_add(1);
            PayrollSchedules::<T>::insert(employer, schedule.id, schedule);

            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(total_amount);
                stats.payments_this_period = stats.payments_this_period.saturating_add(count);
            });

            // Privacy: emit batch commitment, NOT total amount
            let batch_commitment = sp_core::hashing::blake2_256(&total_amount.encode());
            Self::deposit_event(Event::ScheduledPaymentProcessed {
                employer: employer.clone(),
                employee_count: count,
                batch_commitment,
            });

            Ok(Weight::from_parts(50_000_000u64 * count as u64, 0))
        }

        /// Get total payroll for an employer
        pub fn get_total_payroll(employer: &T::AccountId) -> BalanceOf<T> {
            let mut total: BalanceOf<T> = Zero::zero();
            for (_emp_account, emp) in Employees::<T>::iter_prefix(employer) {
                if emp.active {
                    total = total.saturating_add(emp.salary);
                }
            }
            total
        }

        /// Get employee count for an employer
        pub fn get_employee_count(employer: &T::AccountId) -> u32 {
            Employees::<T>::iter_prefix(employer).count() as u32
        }

        /// Get employee count by department
        pub fn get_department_employee_count(employer: &T::AccountId, department_id: u32) -> u32 {
            Employees::<T>::iter_prefix(employer)
                .filter(|(_, e)| e.department_id == department_id)
                .count() as u32
        }
    }
}

// ===== WEIGHT INFO =====

pub trait WeightInfo {
    fn add_employee() -> Weight;
    fn remove_employee() -> Weight;
    fn update_salary() -> Weight;
    fn execute_payment() -> Weight;
    fn batch_payment(n: u32) -> Weight;
    fn create_schedule() -> Weight;
    fn update_schedule() -> Weight;
}

impl WeightInfo for () {
    fn add_employee() -> Weight {
        Weight::from_parts(25_000_000, 512)
            .saturating_add(Weight::from_parts(0, 3_000))
    }
    fn remove_employee() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2_000))
    }
    fn update_salary() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2_000))
    }
    fn execute_payment() -> Weight {
        Weight::from_parts(50_000_000, 512)
            .saturating_add(Weight::from_parts(0, 5_000))
    }
    fn batch_payment(n: u32) -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(Weight::from_parts(50_000_000u64.saturating_mul(n as u64), 0))
            .saturating_add(Weight::from_parts(0, 5_000u64.saturating_mul(n as u64)))
    }
    fn create_schedule() -> Weight {
        Weight::from_parts(30_000_000, 512)
            .saturating_add(Weight::from_parts(0, 3_000))
    }
    fn update_schedule() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2_000))
    }
}
