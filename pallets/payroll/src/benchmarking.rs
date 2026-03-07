//! Benchmarking for pallet-belize-payroll
//!
//! Provides machine-profiled weight functions for all extrinsics
//! exposed via the `WeightInfo` trait.
//!
//! NOTE: `add_employee` depends on cross-pallet KYC (identity/oracle).
//! If KYC fails at benchmark runtime, add a `#[cfg(feature = "runtime-benchmarks")]`
//! bypass to the `meets_kyc_requirement` check in the extrinsic.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Saturating, Zero};
use frame_support::traits::Currency;

/// Helper: set up a verified employer via the `verify_employer` extrinsic (Root origin).
fn setup_employer<T: Config>() -> T::AccountId {
    let employer: T::AccountId = account("employer", 0, 0);
    T::Currency::make_free_balance_be(&employer, BalanceOf::<T>::max_value() / 2u32.into());
    // verify_employer is gated by VerifierOrigin (= Root in runtime)
    Pallet::<T>::verify_employer(
        RawOrigin::Root.into(),
        employer.clone(),
        EmployerType::SME,
    )
    .expect("verify_employer should succeed");
    employer
}

/// Helper: directly insert an employee record into storage (bypasses KYC).
///
/// Avoids the cross-pallet KYC dependency that `add_employee` requires.
fn insert_employee<T: Config>(
    employer: &T::AccountId,
    seed: u32,
) -> T::AccountId {
    let employee: T::AccountId = account("employee", seed, 0);
    T::Currency::make_free_balance_be(&employee, BalanceOf::<T>::max_value() / 4u32.into());

    let salary: BalanceOf<T> = T::MinimumPayment::get().saturating_mul(10u32.into());
    let salary_commitment = Pallet::<T>::compute_salary_commitment(
        &salary, employer, &employee,
    );

    let record = Employee {
        account: employee.clone(),
        salary,
        salary_commitment,
        worker_type: WorkerType::FullTime,
        department_id: 0,
        active: true,
        last_paid: 0u32,
        total_paid: Zero::zero(),
        total_deductions: Zero::zero(),
        start_block: 0u32,
        metadata_hash: [0u8; 32],
    };
    Employees::<T>::insert(employer, &employee, record);

    // Update global stats
    GlobalStats::<T>::mutate(|stats| {
        stats.total_employees = stats.total_employees.saturating_add(1);
    });

    employee
}

#[benchmarks]
mod benchmarks {
    use super::*;

    // ── 1. add_employee ──────────────────────────────────────────────
    // NOTE: This benchmark calls the real extrinsic, which requires
    // T::Oracle::meets_kyc_requirement to return true for the employee.
    // If it fails at runtime due to KYC, add a runtime-benchmarks bypass
    // in the pallet KYC check.
    #[benchmark]
    fn add_employee() {
        let employer = setup_employer::<T>();
        let employee: T::AccountId = account("new_employee", 0, 0);
        T::Currency::make_free_balance_be(&employee, BalanceOf::<T>::max_value() / 4u32.into());
        let salary: BalanceOf<T> = T::MinimumPayment::get().saturating_mul(10u32.into());
        let metadata_hash = [1u8; 32];

        #[extrinsic_call]
        add_employee(
            RawOrigin::Signed(employer),
            employee,
            salary,
            WorkerType::FullTime,
            0u32,            // department_id = unassigned
            metadata_hash,
        );
    }

    // ── 2. remove_employee ───────────────────────────────────────────
    #[benchmark]
    fn remove_employee() {
        let employer = setup_employer::<T>();
        let employee = insert_employee::<T>(&employer, 0);

        #[extrinsic_call]
        remove_employee(RawOrigin::Signed(employer), employee);
    }

    // ── 3. update_salary ─────────────────────────────────────────────
    #[benchmark]
    fn update_salary() {
        let employer = setup_employer::<T>();
        let employee = insert_employee::<T>(&employer, 0);
        let new_salary: BalanceOf<T> = T::MinimumPayment::get().saturating_mul(20u32.into());

        #[extrinsic_call]
        update_salary(RawOrigin::Signed(employer), employee, new_salary);
    }

    // ── 4. execute_payment ───────────────────────────────────────────
    #[benchmark]
    fn execute_payment() {
        let employer = setup_employer::<T>();
        let employee = insert_employee::<T>(&employer, 0);
        // Employer already has large balance from setup_employer

        #[extrinsic_call]
        execute_payment(RawOrigin::Signed(employer), employee);
    }

    // ── 5. batch_payment(n) ──────────────────────────────────────────
    // Pre-populates `n` active employees and measures batch_payment cost.
    // The extrinsic itself takes no `n` parameter — it iterates all
    // active employees for the caller.  Weight scales with employee count.
    #[benchmark]
    fn batch_payment(n: Linear<1, 50>) {
        let employer = setup_employer::<T>();
        for i in 0..n {
            insert_employee::<T>(&employer, i);
        }

        #[extrinsic_call]
        batch_payment(RawOrigin::Signed(employer));
    }

    // ── 6. create_schedule ───────────────────────────────────────────
    #[benchmark]
    fn create_schedule() {
        let employer = setup_employer::<T>();
        // Add an employee so the schedule has meaningful data
        insert_employee::<T>(&employer, 0);
        let interval_blocks = 50_400u32; // ~weekly

        #[extrinsic_call]
        create_schedule(RawOrigin::Signed(employer), interval_blocks, 0u32);
    }

    // ── 7. update_schedule ───────────────────────────────────────────
    #[benchmark]
    fn update_schedule() {
        let employer = setup_employer::<T>();
        insert_employee::<T>(&employer, 0);

        // Pre-create a schedule
        Pallet::<T>::create_schedule(
            RawOrigin::Signed(employer.clone()).into(),
            50_400u32,
            0u32,
        )
        .expect("create_schedule should succeed");

        let schedule_id = 0u32; // First schedule ID

        #[extrinsic_call]
        update_schedule(
            RawOrigin::Signed(employer),
            schedule_id,
            100_800u32,  // new interval: ~bi-weekly
            true,        // keep active
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}
