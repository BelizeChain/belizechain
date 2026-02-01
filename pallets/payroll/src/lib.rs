#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Payroll Pallet
//!
//! Enables businesses to automate salary and contractor payments on-chain with full compliance.
//! 
//! ## Features
//! - Employee list management with KYC verification
//! - Recurring payment scheduling (weekly, bi-weekly, monthly)
//! - Multi-token support (DALLA, bBZD)
//! - Single and batch payment execution
//! - Comprehensive audit trail for all payments
//! - Integration with Identity and Compliance pallets
//! - Automated scheduled payments via on_initialize hook
//!
//! ## Business Use Cases
//! - Company payroll automation
//! - Contractor payment management
//! - Gig economy platforms
//! - Government salary disbursements
//! - Tourism industry wage payments

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use frame_support::pallet_prelude::*;

/// Employee record
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Employee<AccountId, Balance> {
    /// Employee account
    pub account: AccountId,
    /// Salary per payment period
    pub salary: Balance,
    /// Active status
    pub active: bool,
    /// Last payment block
    pub last_paid: u32,
    /// Total amount paid to date
    pub total_paid: Balance,
    /// Start block
    pub start_block: u32,
    /// Metadata hash (IPFS/Arweave link to employment contract)
    pub metadata_hash: [u8; 32],
}

/// Payment frequency
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
}

/// Payment token type
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TokenType {
    /// DALLA token
    Dalla,
    /// bBZD stablecoin
    BBZD,
}

/// Payroll record (audit trail)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct PayrollRecord<AccountId, Balance, BlockNumber> {
    /// Record ID
    pub id: u64,
    /// Employer account
    pub employer: AccountId,
    /// Employee account
    pub employee: AccountId,
    /// Amount paid
    pub amount: Balance,
    /// Token type used
    pub token_type: TokenType,
    /// Block number of payment
    pub block_number: BlockNumber,
    /// Timestamp
    pub timestamp: u64,
}

/// Payroll statistics
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct PayrollStats<Balance> {
    /// Total payroll disbursed (all time)
    pub total_disbursed: Balance,
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
        /// The overarching event type
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

        /// Minimum payment amount
        #[pallet::constant]
        type MinimumPayment: Get<BalanceOf<Self>>;
        
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

    /// Payroll schedules per employer
    #[pallet::storage]
    pub type PayrollSchedules<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,  // Employer
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

    /// Employer verification (KYC required)
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
        /// Employee added to payroll
        EmployeeAdded {
            employer: T::AccountId,
            employee: T::AccountId,
            salary: BalanceOf<T>,
        },
        /// Employee removed from payroll
        EmployeeRemoved {
            employer: T::AccountId,
            employee: T::AccountId,
        },
        /// Salary updated
        SalaryUpdated {
            employer: T::AccountId,
            employee: T::AccountId,
            old_salary: BalanceOf<T>,
            new_salary: BalanceOf<T>,
        },
        /// Payment executed
        PaymentExecuted {
            employer: T::AccountId,
            employee: T::AccountId,
            amount: BalanceOf<T>,
            record_id: u64,
        },
        /// Batch payment completed
        BatchPaymentCompleted {
            employer: T::AccountId,
            count: u32,
            total_amount: BalanceOf<T>,
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
        /// Employer verified for payroll
        EmployerVerified {
            employer: T::AccountId,
        },
        /// Scheduled payment processed
        ScheduledPaymentProcessed {
            employer: T::AccountId,
            employee_count: u32,
            total_amount: BalanceOf<T>,
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
    }

    // ===== HOOKS =====

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut total_weight = Weight::from_parts(10_000_000, 0);

            // Process scheduled payments
            for (employer, schedule) in PayrollSchedules::<T>::iter() {
                if schedule.active && schedule.next_payment <= n {
                    // Execute scheduled payment
                    if let Ok(weight) = Self::process_scheduled_payment(&employer, schedule) {
                        total_weight = total_weight.saturating_add(weight);
                    }
                }
            }

            total_weight
        }
    }

    // ===== EXTRINSICS =====

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add employee to payroll
        ///
        /// Registers a new employee with salary information. Employer must be KYC verified.
        ///
        /// # Parameters
        /// - `employee`: AccountId of the employee
        /// - `salary`: Payment amount per period
        /// - `metadata_hash`: IPFS/Arweave hash of employment contract
        ///
        /// # Errors
        /// - `EmployerNotVerified`: Employer hasn't completed KYC
        /// - `PaymentTooLow`: Salary below minimum payment threshold
        /// - `MaxEmployeesReached`: Employer has too many employees
        /// - `EmployeeAlreadyExists`: Employee already registered
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_employee())]
        pub fn add_employee(
            origin: OriginFor<T>,
            employee: T::AccountId,
            salary: BalanceOf<T>,
            metadata_hash: [u8; 32],
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Verify employer is KYC'd
            ensure!(
                VerifiedEmployers::<T>::get(&employer),
                Error::<T>::EmployerNotVerified
            );
            
            // Verify employee has minimum KYC level (L1 - Basic identity verification)
            // Level 1 = SSN verification, sufficient for payroll
            ensure!(
                T::Oracle::meets_kyc_requirement(&employee, 1),
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

            // Create employee record
            let employee_record = Employee {
                account: employee.clone(),
                salary,
                active: true,
                last_paid: 0u32,
                total_paid: Zero::zero(),
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

            Self::deposit_event(Event::EmployeeAdded {
                employer,
                employee,
                salary,
            });

            Ok(())
        }

        /// Remove employee from payroll
        ///
        /// Removes an employee from the employer's payroll list.
        ///
        /// # Parameters
        /// - `employee`: AccountId of the employee to remove
        ///
        /// # Errors
        /// - `EmployeeNotFound`: Employee doesn't exist for this employer
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::remove_employee())]
        pub fn remove_employee(
            origin: OriginFor<T>,
            employee: T::AccountId,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Verify employee exists
            ensure!(
                Employees::<T>::contains_key(&employer, &employee),
                Error::<T>::EmployeeNotFound
            );

            // Remove employee
            Employees::<T>::remove(&employer, &employee);

            // Update stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_employees = stats.total_employees.saturating_sub(1);
            });

            // If employer has no more employees, decrement employer count
            let remaining = Employees::<T>::iter_prefix(&employer).count();
            if remaining == 0 {
                EmployerCount::<T>::mutate(|count| {
                    *count = count.saturating_sub(1);
                });
                GlobalStats::<T>::mutate(|stats| {
                    stats.total_employers = stats.total_employers.saturating_sub(1);
                });
            }

            Self::deposit_event(Event::EmployeeRemoved {
                employer,
                employee,
            });

            Ok(())
        }

        /// Update employee salary
        ///
        /// Changes the salary amount for an existing employee.
        ///
        /// # Parameters
        /// - `employee`: AccountId of the employee
        /// - `new_salary`: New salary amount
        ///
        /// # Errors
        /// - `EmployeeNotFound`: Employee doesn't exist
        /// - `PaymentTooLow`: New salary below minimum threshold
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_salary())]
        pub fn update_salary(
            origin: OriginFor<T>,
            employee: T::AccountId,
            new_salary: BalanceOf<T>,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Check minimum payment
            ensure!(
                new_salary >= T::MinimumPayment::get(),
                Error::<T>::PaymentTooLow
            );

            // Get employee
            let mut emp = Employees::<T>::get(&employer, &employee)
                .ok_or(Error::<T>::EmployeeNotFound)?;

            let old_salary = emp.salary;
            emp.salary = new_salary;

            // Update storage
            Employees::<T>::insert(&employer, &employee, emp);

            Self::deposit_event(Event::SalaryUpdated {
                employer,
                employee,
                old_salary,
                new_salary,
            });

            Ok(())
        }

        /// Execute single payment to employee
        ///
        /// Transfers salary payment from employer to employee immediately.
        ///
        /// # Parameters
        /// - `employee`: AccountId of the employee to pay
        ///
        /// # Errors
        /// - `EmployeeNotFound`: Employee doesn't exist
        /// - `EmployeeInactive`: Employee is marked inactive
        /// - `InsufficientBalance`: Employer doesn't have enough funds
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::execute_payment())]
        pub fn execute_payment(
            origin: OriginFor<T>,
            employee: T::AccountId,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Get employee
            let mut emp = Employees::<T>::get(&employer, &employee)
                .ok_or(Error::<T>::EmployeeNotFound)?;

            // Check employee is active
            ensure!(emp.active, Error::<T>::EmployeeInactive);

            // Check employer balance
            let balance = T::Currency::free_balance(&employer);
            ensure!(balance >= emp.salary, Error::<T>::InsufficientBalance);

            // Execute transfer
            T::Currency::transfer(
                &employer,
                &employee,
                emp.salary,
                ExistenceRequirement::KeepAlive,
            )?;

            // Update employee record
            let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
            emp.last_paid = current_block;
            emp.total_paid = emp.total_paid.saturating_add(emp.salary);
            Employees::<T>::insert(&employer, &employee, emp.clone());

            // Create payment record
            let record_id = NextRecordId::<T>::get();
            let record = PayrollRecord {
                id: record_id,
                employer: employer.clone(),
                employee: employee.clone(),
                amount: emp.salary,
                token_type: TokenType::Dalla,
                block_number: frame_system::Pallet::<T>::block_number(),
                timestamp: T::TimeProvider::now().as_secs(),
            };
            PayrollRecords::<T>::insert(record_id, record);
            NextRecordId::<T>::put(record_id.saturating_add(1));

            // Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(emp.salary);
                stats.payments_this_period = stats.payments_this_period.saturating_add(1);
            });

            Self::deposit_event(Event::PaymentExecuted {
                employer,
                employee,
                amount: emp.salary,
                record_id,
            });

            Ok(())
        }

        /// Execute batch payment to multiple employees
        ///
        /// Pays all active employees for this employer in a single transaction.
        ///
        /// # Errors
        /// - `InsufficientBalance`: Employer doesn't have enough funds for total payroll
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::batch_payment(T::MaxEmployees::get()))]
        pub fn batch_payment(origin: OriginFor<T>) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            let mut total_amount: BalanceOf<T> = Zero::zero();
            let mut count = 0u32;

            // Calculate total needed
            for (_emp_account, emp) in Employees::<T>::iter_prefix(&employer) {
                if emp.active {
                    total_amount = total_amount.saturating_add(emp.salary);
                    count = count.saturating_add(1);
                }
            }

            // Check employer has sufficient balance
            let balance = T::Currency::free_balance(&employer);
            ensure!(balance >= total_amount, Error::<T>::InsufficientBalance);

            // Execute all payments
            for (emp_account, mut emp) in Employees::<T>::iter_prefix(&employer) {
                if emp.active {
                    // Transfer payment
                    T::Currency::transfer(
                        &employer,
                        &emp_account,
                        emp.salary,
                        ExistenceRequirement::KeepAlive,
                    )?;

                    // Update employee record
                    let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
                    emp.last_paid = current_block;
                    emp.total_paid = emp.total_paid.saturating_add(emp.salary);
                    Employees::<T>::insert(&employer, &emp_account, emp.clone());

                    // Create payment record
                    let record_id = NextRecordId::<T>::get();
                    let record = PayrollRecord {
                        id: record_id,
                        employer: employer.clone(),
                        employee: emp_account.clone(),
                        amount: emp.salary,
                        token_type: TokenType::Dalla,
                        block_number: frame_system::Pallet::<T>::block_number(),
                        timestamp: T::TimeProvider::now().as_secs(),
                    };
                    PayrollRecords::<T>::insert(record_id, record);
                    NextRecordId::<T>::put(record_id.saturating_add(1));
                }
            }

            // Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(total_amount);
                stats.payments_this_period = stats.payments_this_period.saturating_add(count);
            });

            Self::deposit_event(Event::BatchPaymentCompleted {
                employer,
                count,
                total_amount,
            });

            Ok(())
        }

        /// Create recurring payment schedule
        ///
        /// Sets up automated payroll on a recurring basis.
        ///
        /// # Parameters
        /// - `interval_blocks`: Payment interval in blocks
        ///   - Weekly: 50,400 blocks (~7 days at 6s blocks)
        ///   - Bi-weekly: 100,800 blocks (~14 days)
        ///   - Monthly: 432,000 blocks (~30 days)
        ///
        /// # Errors
        /// - `EmployerNotVerified`: Employer hasn't completed KYC
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::create_schedule())]
        pub fn create_schedule(
            origin: OriginFor<T>,
            interval_blocks: u32,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Verify employer is KYC'd
            ensure!(
                VerifiedEmployers::<T>::get(&employer),
                Error::<T>::EmployerNotVerified
            );

            let schedule_id = NextScheduleId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();

            let employee_count = Employees::<T>::iter_prefix(&employer).count() as u32;

            let schedule = PayrollSchedule {
                id: schedule_id,
                frequency: PaymentFrequency::Custom(interval_blocks),
                next_payment: current_block + interval_blocks.into(),
                active: true,
                payments_made: 0,
                employee_count,
            };

            PayrollSchedules::<T>::insert(&employer, schedule);
            NextScheduleId::<T>::put(schedule_id.saturating_add(1));

            Self::deposit_event(Event::ScheduleCreated {
                employer,
                schedule_id,
            });

            Ok(())
        }

        /// Update existing payment schedule
        ///
        /// Modifies the interval or activates/deactivates a schedule.
        ///
        /// # Parameters
        /// - `interval_blocks`: New payment interval in blocks
        /// - `active`: Whether schedule is active
        ///
        /// # Errors
        /// - `ScheduleNotFound`: No schedule exists for this employer
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::update_schedule())]
        pub fn update_schedule(
            origin: OriginFor<T>,
            interval_blocks: u32,
            active: bool,
        ) -> DispatchResult {
            let employer = ensure_signed(origin)?;

            // Get existing schedule
            let mut schedule = PayrollSchedules::<T>::get(&employer)
                .ok_or(Error::<T>::ScheduleNotFound)?;

            schedule.frequency = PaymentFrequency::Custom(interval_blocks);
            schedule.active = active;

            PayrollSchedules::<T>::insert(&employer, schedule.clone());

            Self::deposit_event(Event::ScheduleUpdated {
                employer,
                schedule_id: schedule.id,
            });

            Ok(())
        }

        /// Verify employer for payroll (admin/governance function)
        ///
        /// Marks an employer as KYC verified, allowing them to use payroll features.
        /// This would typically be called by governance or compliance pallet.
        ///
        /// # Parameters
        /// - `employer`: AccountId to verify
        ///
        /// # Note
        /// In production, this should be restricted to governance/root origin
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000_000, 0))]
        pub fn verify_employer(
            origin: OriginFor<T>,
            employer: T::AccountId,
        ) -> DispatchResult {
            // For now, allow signed origin (in production, use ensure_root or governance)
            let _who = ensure_signed(origin)?;

            VerifiedEmployers::<T>::insert(&employer, true);

            Self::deposit_event(Event::EmployerVerified {
                employer,
            });

            Ok(())
        }
    }

    // ===== HELPER FUNCTIONS =====

    impl<T: Config> Pallet<T> {
        /// Process scheduled payment for an employer
        fn process_scheduled_payment(
            employer: &T::AccountId,
            mut schedule: PayrollSchedule<BlockNumberFor<T>>,
        ) -> Result<Weight, DispatchError> {
            let mut total_amount: BalanceOf<T> = Zero::zero();
            let mut count = 0u32;

            // Calculate total needed
            for (_emp_account, emp) in Employees::<T>::iter_prefix(employer) {
                if emp.active {
                    total_amount = total_amount.saturating_add(emp.salary);
                    count = count.saturating_add(1);
                }
            }

            // Check employer has sufficient balance
            let balance = T::Currency::free_balance(employer);
            if balance < total_amount {
                // Skip this payment, will try again next block
                return Ok(Weight::from_parts(5_000_000, 0));
            }

            // Execute all payments
            for (emp_account, mut emp) in Employees::<T>::iter_prefix(employer) {
                if emp.active {
                    // Transfer payment
                    T::Currency::transfer(
                        employer,
                        &emp_account,
                        emp.salary,
                        ExistenceRequirement::KeepAlive,
                    )?;

                    // Update employee record
                    let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
                    emp.last_paid = current_block;
                    emp.total_paid = emp.total_paid.saturating_add(emp.salary);
                    Employees::<T>::insert(employer, &emp_account, emp.clone());

                    // Create payment record
                    let record_id = NextRecordId::<T>::get();
                    let record = PayrollRecord {
                        id: record_id,
                        employer: employer.clone(),
                        employee: emp_account.clone(),
                        amount: emp.salary,
                        token_type: TokenType::Dalla,
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
            PayrollSchedules::<T>::insert(employer, schedule);

            // Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_disbursed = stats.total_disbursed.saturating_add(total_amount);
                stats.payments_this_period = stats.payments_this_period.saturating_add(count);
            });

            Self::deposit_event(Event::ScheduledPaymentProcessed {
                employer: employer.clone(),
                employee_count: count,
                total_amount,
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
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 3_000))
    }
    fn remove_employee() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 2_000))
    }
    fn update_salary() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(Weight::from_parts(0, 2_000))
    }
    fn execute_payment() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(Weight::from_parts(0, 5_000))
    }
    fn batch_payment(n: u32) -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(Weight::from_parts(50_000_000u64.saturating_mul(n as u64), 0))
            .saturating_add(Weight::from_parts(0, 5_000u64.saturating_mul(n as u64)))
    }
    fn create_schedule() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(Weight::from_parts(0, 3_000))
    }
    fn update_schedule() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 2_000))
    }
}
