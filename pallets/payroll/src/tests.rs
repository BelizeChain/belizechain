use crate::{mock::*, Error, Event, PaymentFrequency, WorkerType, EmployerType, PaymentCategory, DeductionType};
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    traits::OnIdle,
    weights::Weight,
};

// Helper: verify employer via root origin (governance)
fn verify_employer_as_root(employer: u64, employer_type: EmployerType) {
    assert_ok!(Payroll::verify_employer(RuntimeOrigin::root(), employer, employer_type));
}

// Helper: add employee with default worker type & department
fn add_employee_default(employer: u64, employee: u64, salary: u64) {
    assert_ok!(Payroll::add_employee(
        RuntimeOrigin::signed(employer),
        employee,
        salary,
        WorkerType::FullTime,
        0, // no department
        [1u8; 32],
    ));
}

// ===== EMPLOYER VERIFICATION TESTS =====

#[test]
fn verify_employer_works_with_root_origin() {
    new_test_ext().execute_with(|| {
        let employer = 1;

        verify_employer_as_root(employer, EmployerType::Enterprise);

        assert!(crate::VerifiedEmployers::<Test>::get(employer));

        // Check profile was created
        let profile = crate::EmployerProfiles::<Test>::get(employer).unwrap();
        assert_eq!(profile.employer_type, EmployerType::Enterprise);
        assert!(profile.verified);

        System::assert_has_event(
            Event::EmployerVerified {
                employer,
                employer_type: EmployerType::Enterprise,
            }.into()
        );
    });
}

#[test]
fn verify_employer_fails_with_signed_origin() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let non_root = 100;

        // Signed origin should fail — only root/governance can verify
        assert_noop!(
            Payroll::verify_employer(
                RuntimeOrigin::signed(non_root),
                employer,
                EmployerType::SME
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn verify_employer_different_types() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Government);
        verify_employer_as_root(2, EmployerType::GigPlatform);

        let p1 = crate::EmployerProfiles::<Test>::get(1).unwrap();
        let p2 = crate::EmployerProfiles::<Test>::get(2).unwrap();
        assert_eq!(p1.employer_type, EmployerType::Government);
        assert_eq!(p2.employer_type, EmployerType::GigPlatform);
    });
}

// ===== EMPLOYEE MANAGEMENT TESTS =====

#[test]
fn add_employee_works_with_worker_type() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;

        verify_employer_as_root(employer, EmployerType::Enterprise);

        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            WorkerType::Contractor,
            0,
            [1u8; 32],
        ));

        let emp = crate::Employees::<Test>::get(employer, employee).unwrap();
        assert_eq!(emp.salary, salary);
        assert_eq!(emp.worker_type, WorkerType::Contractor);
        assert_eq!(emp.department_id, 0);
        assert!(emp.active);
        assert_eq!(emp.total_paid, 0);
        assert_eq!(emp.total_deductions, 0);

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 1);
        assert_eq!(stats.total_employers, 1);

        System::assert_has_event(
            Event::EmployeeAdded {
                employer,
                employee,
                salary_commitment: Payroll::compute_salary_commitment(&salary, &employer, &employee),
                worker_type: WorkerType::Contractor,
                department_id: 0,
            }.into()
        );
    });
}

#[test]
fn add_employee_with_department() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;

        verify_employer_as_root(employer, EmployerType::Enterprise);

        // Create department first
        assert_ok!(Payroll::create_department(
            RuntimeOrigin::signed(employer),
            [0xAA; 32],
        ));

        // Add employee to department 1
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            WorkerType::FullTime,
            1,
            [1u8; 32],
        ));

        let emp = crate::Employees::<Test>::get(employer, employee).unwrap();
        assert_eq!(emp.department_id, 1);
    });
}

#[test]
fn add_employee_fails_invalid_department() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;

        verify_employer_as_root(employer, EmployerType::Enterprise);

        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(employer),
                employee,
                salary,
                WorkerType::FullTime,
                99,
                [1u8; 32],
            ),
            Error::<Test>::DepartmentNotFound
        );
    });
}

#[test]
fn add_employee_fails_not_verified() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(1),
                3,
                10_000_000_000,
                WorkerType::FullTime,
                0,
                [1u8; 32],
            ),
            Error::<Test>::EmployerNotVerified
        );
    });
}

#[test]
fn add_employee_fails_payment_too_low() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::SME);

        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(1),
                3,
                100, // Below minimum
                WorkerType::FullTime,
                0,
                [1u8; 32],
            ),
            Error::<Test>::PaymentTooLow
        );
    });
}

#[test]
fn add_employee_fails_already_exists() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(1),
                3,
                10_000_000_000,
                WorkerType::FullTime,
                0,
                [1u8; 32],
            ),
            Error::<Test>::EmployeeAlreadyExists
        );
    });
}

#[test]
fn remove_employee_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::remove_employee(RuntimeOrigin::signed(1), 3));
        assert!(!crate::Employees::<Test>::contains_key(1, 3));

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 0);
        assert_eq!(stats.total_employers, 0);

        System::assert_has_event(Event::EmployeeRemoved { employer: 1, employee: 3 }.into());
    });
}

#[test]
fn remove_employee_fails_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_noop!(
            Payroll::remove_employee(RuntimeOrigin::signed(1), 3),
            Error::<Test>::EmployeeNotFound
        );
    });
}

// ===== EMPLOYEE STATUS TOGGLE =====

#[test]
fn toggle_employee_status_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        // Suspend
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, false));
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert!(!emp.active);

        System::assert_has_event(
            Event::EmployeeStatusChanged { employer: 1, employee: 3, active: false }.into()
        );

        // Reactivate
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, true));
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert!(emp.active);
    });
}

#[test]
fn toggle_employee_status_fails_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_noop!(
            Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 99, false),
            Error::<Test>::EmployeeNotFound
        );
    });
}

// ===== SALARY TESTS =====

#[test]
fn update_salary_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::update_salary(RuntimeOrigin::signed(1), 3, 15_000_000_000));
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.salary, 15_000_000_000);

        System::assert_has_event(
            Event::SalaryUpdated {
                employer: 1, employee: 3,
                new_commitment: Payroll::compute_salary_commitment(&15_000_000_000, &1, &3),
            }.into()
        );
    });
}

#[test]
fn update_salary_fails_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_noop!(
            Payroll::update_salary(RuntimeOrigin::signed(1), 3, 15_000_000_000),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn update_salary_fails_too_low() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_noop!(
            Payroll::update_salary(RuntimeOrigin::signed(1), 3, 100),
            Error::<Test>::PaymentTooLow
        );
    });
}

// ===== PAYMENT EXECUTION TESTS =====

#[test]
fn execute_payment_works_with_deductions() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let tax = 1_000_000_000u64;
        let ss = 500_000_000u64;

        let employer_before = Balances::free_balance(1);
        let employee_before = Balances::free_balance(3);

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, tax));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::SocialSecurity, ss));

        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        let total_ded = tax + ss;
        let net = salary - total_ded;

        assert_eq!(Balances::free_balance(1), employer_before - net);
        assert_eq!(Balances::free_balance(3), employee_before + net);

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary);
        assert_eq!(emp.total_deductions, total_ded);

        let record = crate::PayrollRecords::<Test>::get(0).unwrap();
        assert_eq!(record.amount, salary);
        assert_eq!(record.deductions, total_ded);
        assert_eq!(record.net_amount, net);
        assert_eq!(record.category, PaymentCategory::Salary);

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_disbursed, salary);
        assert_eq!(stats.total_deductions, total_ded);
    });
}

#[test]
fn execute_payment_works_no_deductions() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;

        let employer_before = Balances::free_balance(1);
        let employee_before = Balances::free_balance(3);

        verify_employer_as_root(1, EmployerType::SME);
        add_employee_default(1, 3, salary);

        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));
        assert_eq!(Balances::free_balance(1), employer_before - salary);
        assert_eq!(Balances::free_balance(3), employee_before + salary);
    });
}

#[test]
fn execute_payment_fails_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_noop!(
            Payroll::execute_payment(RuntimeOrigin::signed(1), 3),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn execute_payment_fails_inactive() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, false));

        assert_noop!(
            Payroll::execute_payment(RuntimeOrigin::signed(1), 3),
            Error::<Test>::EmployeeInactive
        );
    });
}

#[test]
fn execute_payment_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        // Salary under MaxPaymentAmount but drain employer balance to trigger InsufficientBalance
        add_employee_default(1, 3, 500_000_000_000);
        // Transfer most of employer's balance away so payment fails
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), 1, 100));

        assert_noop!(
            Payroll::execute_payment(RuntimeOrigin::signed(1), 3),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn batch_payment_works() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let employer_before = Balances::free_balance(1);

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        add_employee_default(1, 4, salary);
        add_employee_default(1, 5, salary);

        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(1)));

        assert_eq!(Balances::free_balance(1), employer_before - (salary * 3));

        for emp_id in [3u64, 4, 5] {
            let emp = crate::Employees::<Test>::get(1, emp_id).unwrap();
            assert_eq!(emp.total_paid, salary);
        }

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_disbursed, salary * 3);
        assert_eq!(stats.payments_this_period, 3);
    });
}

#[test]
fn batch_payment_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 500_000_000_000);
        add_employee_default(1, 4, 500_000_000_000);
        // Drain employer balance so batch total exceeds it
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), 1, 100));

        assert_noop!(
            Payroll::batch_payment(RuntimeOrigin::signed(1)),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ===== SCHEDULE TESTS =====

#[test]
fn create_schedule_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 432_000, 0));

        let schedule = crate::PayrollSchedules::<Test>::get(1, 0).unwrap();
        assert_eq!(schedule.frequency, PaymentFrequency::Custom(432_000));
        assert!(schedule.active);
        assert_eq!(schedule.next_payment, 1 + 432_000);
        assert_eq!(schedule.department_id, 0);
    });
}

#[test]
fn create_multiple_schedules() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xBB; 32]));

        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50_400, 0));
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 432_000, 1));

        let s0 = crate::PayrollSchedules::<Test>::get(1, 0).unwrap();
        let s1 = crate::PayrollSchedules::<Test>::get(1, 1).unwrap();
        assert_eq!(s0.department_id, 0);
        assert_eq!(s1.department_id, 1);
    });
}

#[test]
fn create_schedule_fails_not_verified() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Payroll::create_schedule(RuntimeOrigin::signed(1), 50_400, 0),
            Error::<Test>::EmployerNotVerified
        );
    });
}

#[test]
fn update_schedule_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 432_000, 0));

        assert_ok!(Payroll::update_schedule(RuntimeOrigin::signed(1), 0, 100_800, true));

        let schedule = crate::PayrollSchedules::<Test>::get(1, 0).unwrap();
        assert_eq!(schedule.frequency, PaymentFrequency::Custom(100_800));
    });
}

#[test]
fn update_schedule_fails_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_noop!(
            Payroll::update_schedule(RuntimeOrigin::signed(1), 99, 50_400, true),
            Error::<Test>::ScheduleNotFound
        );
    });
}

#[test]
fn scheduled_payment_processed_automatically() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let employer_before = Balances::free_balance(1);

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        System::set_block_number(101);
        // Payments run in on_idle (not on_initialize) to avoid competing with user txs
        Payroll::on_idle(101, Weight::from_parts(u64::MAX, u64::MAX));

        assert_eq!(Balances::free_balance(1), employer_before - salary);

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary);

        let schedule = crate::PayrollSchedules::<Test>::get(1, 0).unwrap();
        assert_eq!(schedule.next_payment, 101 + 100);
        assert_eq!(schedule.payments_made, 1);
    });
}

// ===== DEPARTMENT TESTS =====

#[test]
fn create_department_works() {
    new_test_ext().execute_with(|| {
        let name_hash = [0xCC; 32];

        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), name_hash));

        assert_eq!(crate::Departments::<Test>::get(1, 1u32).unwrap(), name_hash);

        let profile = crate::EmployerProfiles::<Test>::get(1).unwrap();
        assert_eq!(profile.department_count, 1);

        System::assert_has_event(
            Event::DepartmentCreated { employer: 1, department_id: 1, name_hash }.into()
        );
    });
}

#[test]
fn create_multiple_departments() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xAA; 32]));
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xBB; 32]));
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xCC; 32]));

        let profile = crate::EmployerProfiles::<Test>::get(1).unwrap();
        assert_eq!(profile.department_count, 3);
    });
}

#[test]
fn create_department_fails_not_verified() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Payroll::create_department(RuntimeOrigin::signed(1), [0xAA; 32]),
            Error::<Test>::EmployerNotVerified
        );
    });
}

// ===== DEDUCTION TESTS =====

#[test]
fn set_deduction_works() {
    new_test_ext().execute_with(|| {
        let tax = 1_000_000_000u64;

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, tax));

        let deductions = crate::EmployeeDeductions::<Test>::get(1, 3);
        assert_eq!(deductions.len(), 1);
        assert_eq!(deductions[0].deduction_type, DeductionType::IncomeTax);
        assert_eq!(deductions[0].amount, tax);
        assert!(deductions[0].active);
    });
}

#[test]
fn set_deduction_updates_existing() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, 1_000_000_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, 2_000_000_000));

        let deductions = crate::EmployeeDeductions::<Test>::get(1, 3);
        assert_eq!(deductions.len(), 1);
        assert_eq!(deductions[0].amount, 2_000_000_000);
    });
}

#[test]
fn set_deduction_fails_employee_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_noop!(
            Payroll::set_deduction(RuntimeOrigin::signed(1), 99, DeductionType::IncomeTax, 1_000_000_000),
            Error::<Test>::EmployeeNotFound
        );
    });
}

// ===== BONUS / ONE-TIME PAYMENT TESTS =====

#[test]
fn issue_bonus_works() {
    new_test_ext().execute_with(|| {
        let bonus = 5_000_000_000u64;

        let employer_before = Balances::free_balance(1);
        let employee_before = Balances::free_balance(3);

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::issue_bonus(RuntimeOrigin::signed(1), 3, bonus, PaymentCategory::Bonus));

        assert_eq!(Balances::free_balance(1), employer_before - bonus);
        assert_eq!(Balances::free_balance(3), employee_before + bonus);

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, bonus);

        let record = crate::PayrollRecords::<Test>::get(0).unwrap();
        assert_eq!(record.amount, bonus);
        assert_eq!(record.deductions, 0);
        assert_eq!(record.category, PaymentCategory::Bonus);

        let amount_commitment = sp_core::hashing::blake2_256(&(bonus, 1u64, 3u64).encode());
        System::assert_has_event(
            Event::BonusIssued { employer: 1, employee: 3, amount_commitment, category: PaymentCategory::Bonus }.into()
        );
    });
}

#[test]
fn issue_overtime_payment() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::issue_bonus(RuntimeOrigin::signed(1), 3, 2_000_000_000, PaymentCategory::Overtime));

        let record = crate::PayrollRecords::<Test>::get(0).unwrap();
        assert_eq!(record.category, PaymentCategory::Overtime);
    });
}

#[test]
fn issue_bonus_fails_employee_not_found() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_noop!(
            Payroll::issue_bonus(RuntimeOrigin::signed(1), 99, 5_000_000_000, PaymentCategory::Bonus),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn issue_bonus_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);
        // Drain employer balance so bonus exceeds it
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), 1, 100));

        assert_noop!(
            Payroll::issue_bonus(RuntimeOrigin::signed(1), 3, 500_000_000_000, PaymentCategory::Bonus),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ===== HELPER FUNCTION TESTS =====

#[test]
fn get_total_payroll_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);
        add_employee_default(1, 4, 15_000_000_000);

        assert_eq!(Payroll::get_total_payroll(&1), 25_000_000_000);
    });
}

#[test]
fn get_employee_count_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);
        add_employee_default(1, 4, 10_000_000_000);
        add_employee_default(1, 5, 10_000_000_000);

        assert_eq!(Payroll::get_employee_count(&1), 3);
    });
}

#[test]
fn get_department_employee_count_works() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xAA; 32]));
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xBB; 32]));

        assert_ok!(Payroll::add_employee(RuntimeOrigin::signed(1), 3, 10_000_000_000, WorkerType::FullTime, 1, [1u8; 32]));
        assert_ok!(Payroll::add_employee(RuntimeOrigin::signed(1), 4, 10_000_000_000, WorkerType::FullTime, 1, [1u8; 32]));
        assert_ok!(Payroll::add_employee(RuntimeOrigin::signed(1), 5, 10_000_000_000, WorkerType::Contractor, 2, [1u8; 32]));

        assert_eq!(Payroll::get_department_employee_count(&1, 1), 2);
        assert_eq!(Payroll::get_department_employee_count(&1, 2), 1);
    });
}

// ===== MULTI-EMPLOYER TESTS =====

#[test]
fn multiple_employers_independent() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Government);
        verify_employer_as_root(2, EmployerType::SME);

        add_employee_default(1, 3, 10_000_000_000);
        add_employee_default(2, 4, 10_000_000_000);

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 2);
        assert_eq!(stats.total_employers, 2);

        let p1 = crate::EmployerProfiles::<Test>::get(1).unwrap();
        let p2 = crate::EmployerProfiles::<Test>::get(2).unwrap();
        assert_eq!(p1.employer_type, EmployerType::Government);
        assert_eq!(p2.employer_type, EmployerType::SME);

        assert_noop!(
            Payroll::remove_employee(RuntimeOrigin::signed(1), 4),
            Error::<Test>::EmployeeNotFound
        );
    });
}

// ===== CLEANUP TESTS =====

#[test]
fn remove_employee_cleans_up_deductions() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, 1_000_000_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Pension, 500_000_000));

        let deductions = crate::EmployeeDeductions::<Test>::get(1, 3);
        assert_eq!(deductions.len(), 2);

        assert_ok!(Payroll::remove_employee(RuntimeOrigin::signed(1), 3));

        let deductions = crate::EmployeeDeductions::<Test>::get(1, 3);
        assert_eq!(deductions.len(), 0);
    });
}

// ===== EVENT EMISSION TESTS (6 previously unverified events) =====

#[test]
fn execute_payment_emits_payment_executed_event() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);

        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        let payment_commitment = Payroll::compute_payment_commitment(
            &salary, &0u64, &salary, &1u64, &3u64,
        );
        System::assert_has_event(
            Event::PaymentExecuted {
                employer: 1,
                employee: 3,
                payment_commitment,
                category: PaymentCategory::Salary,
                record_id: 0,
            }.into()
        );
    });
}

#[test]
fn execute_payment_with_deductions_emits_event_with_correct_commitment() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let tax = 1_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, tax));

        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        let net = salary - tax;
        let payment_commitment = Payroll::compute_payment_commitment(
            &salary, &tax, &net, &1u64, &3u64,
        );
        System::assert_has_event(
            Event::PaymentExecuted {
                employer: 1,
                employee: 3,
                payment_commitment,
                category: PaymentCategory::Salary,
                record_id: 0,
            }.into()
        );
    });
}

#[test]
fn batch_payment_emits_batch_completed_event() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        add_employee_default(1, 4, salary);

        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(1)));

        let total_gross = salary * 2;
        let batch_commitment = sp_core::hashing::blake2_256(&total_gross.encode());
        System::assert_has_event(
            Event::BatchPaymentCompleted {
                employer: 1,
                count: 2,
                batch_commitment,
            }.into()
        );
    });
}

#[test]
fn create_schedule_emits_schedule_created_event() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        System::assert_has_event(
            Event::ScheduleCreated {
                employer: 1,
                schedule_id: 0,
            }.into()
        );
    });
}

#[test]
fn update_schedule_emits_schedule_updated_event() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        assert_ok!(Payroll::update_schedule(RuntimeOrigin::signed(1), 0, 200, true));

        System::assert_has_event(
            Event::ScheduleUpdated {
                employer: 1,
                schedule_id: 0,
            }.into()
        );
    });
}

#[test]
fn scheduled_payment_emits_processed_event() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 0));

        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        let batch_commitment = sp_core::hashing::blake2_256(&salary.encode());
        System::assert_has_event(
            Event::ScheduledPaymentProcessed {
                employer: 1,
                employee_count: 1,
                batch_commitment,
            }.into()
        );
    });
}

#[test]
fn set_deduction_emits_deduction_updated_event() {
    new_test_ext().execute_with(|| {
        let amount = 1_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, amount));

        let deduction_commitment = sp_core::hashing::blake2_256(
            &(DeductionType::IncomeTax, amount).encode()
        );
        System::assert_has_event(
            Event::DeductionUpdated {
                employer: 1,
                employee: 3,
                deduction_type: DeductionType::IncomeTax,
                deduction_commitment,
            }.into()
        );
    });
}

// ===== CROSS-EMPLOYER ISOLATION TESTS =====

#[test]
fn wrong_employer_cannot_remove_employee() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, 10_000_000_000);

        // Employer 2 tries to remove employer 1's employee
        assert_noop!(
            Payroll::remove_employee(RuntimeOrigin::signed(2), 3),
            Error::<Test>::EmployeeNotFound
        );

        // Original employer can still remove
        assert_ok!(Payroll::remove_employee(RuntimeOrigin::signed(1), 3));
    });
}

#[test]
fn wrong_employer_cannot_toggle_status() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, 10_000_000_000);

        assert_noop!(
            Payroll::toggle_employee_status(RuntimeOrigin::signed(2), 3, false),
            Error::<Test>::EmployeeNotFound
        );

        // Employee remains active under employer 1
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert!(emp.active);
    });
}

#[test]
fn wrong_employer_cannot_update_salary() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, 10_000_000_000);

        assert_noop!(
            Payroll::update_salary(RuntimeOrigin::signed(2), 3, 20_000_000_000),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn wrong_employer_cannot_execute_payment() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, 10_000_000_000);

        assert_noop!(
            Payroll::execute_payment(RuntimeOrigin::signed(2), 3),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn wrong_employer_cannot_update_schedule() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        // Employer 2 can't update employer 1's schedule
        assert_noop!(
            Payroll::update_schedule(RuntimeOrigin::signed(2), 0, 200, true),
            Error::<Test>::ScheduleNotFound
        );
    });
}

// ===== MAX REACHED ERROR PATHS =====

#[test]
fn max_deductions_reached() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        // BoundedVec<_, ConstU32<10>> — fill all 10 slots
        // 5 named variants + 5 Custom variants = 10 unique deduction types
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::SocialSecurity, 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Pension, 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::HealthInsurance, 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([1u8; 16]), 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([2u8; 16]), 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([3u8; 16]), 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([4u8; 16]), 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([5u8; 16]), 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([6u8; 16]), 100_000));

        // 11th should fail
        assert_noop!(
            Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Custom([7u8; 16]), 100_000),
            Error::<Test>::MaxDeductionsReached
        );
    });
}

#[test]
fn max_departments_reached() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        // MaxDepartments = 50, create exactly 50
        for i in 0..50u8 {
            assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [i; 32]));
        }

        // 51st should fail
        assert_noop!(
            Payroll::create_department(RuntimeOrigin::signed(1), [99u8; 32]),
            Error::<Test>::MaxDepartmentsReached
        );
    });
}

// ===== ON_IDLE EDGE CASES =====

#[test]
fn on_idle_returns_zero_when_weight_too_low() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 0));

        System::set_block_number(51);

        // Give zero weight — should return Weight::zero()
        let consumed = Payroll::on_idle(51, Weight::from_parts(0, 0));
        assert_eq!(consumed, Weight::zero());

        // Employee should NOT have been paid
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, 0);
    });
}

#[test]
fn on_idle_skips_schedule_not_yet_due() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let employer_before = Balances::free_balance(1);
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        // Schedule at block 1, next_payment = 1 + 100 = 101
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        // Advance to block 50 (before due)
        System::set_block_number(50);
        Payroll::on_idle(50, Weight::from_parts(u64::MAX, u64::MAX));

        // No payment should have occurred
        assert_eq!(Balances::free_balance(1), employer_before);
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, 0);
    });
}

#[test]
fn on_idle_skips_employer_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // Use employee account 5 as "employer" — only has 10K DALLA
        verify_employer_as_root(5, EmployerType::SME);
        // Salary of 50K exceeds employer's 10K balance
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(5),
            6,
            50_000_000_000,
            WorkerType::FullTime,
            0,
            [1u8; 32],
        ));
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(5), 50, 0));

        let emp_before = Balances::free_balance(6);
        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        // Employee should NOT have been paid (employer can't cover it)
        assert_eq!(Balances::free_balance(6), emp_before);
    });
}

#[test]
fn on_idle_department_filter_pays_only_department() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);

        // Create department 1
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xAA; 32]));

        // Employee 3 in department 1, employee 4 in department 0 (unassigned)
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(1), 3, salary, WorkerType::FullTime, 1, [1u8; 32],
        ));
        add_employee_default(1, 4, salary);

        // Schedule for department 1 only
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 1));

        let before_3 = Balances::free_balance(3);
        let before_4 = Balances::free_balance(4);

        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        // Only employee 3 (department 1) should be paid
        assert_eq!(Balances::free_balance(3), before_3 + salary);
        assert_eq!(Balances::free_balance(4), before_4); // unchanged
    });
}

#[test]
fn on_idle_processes_multiple_employer_schedules() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, salary);
        add_employee_default(2, 4, salary);

        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 0));
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(2), 50, 0));

        let before_3 = Balances::free_balance(3);
        let before_4 = Balances::free_balance(4);

        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        // Both employees should be paid
        assert_eq!(Balances::free_balance(3), before_3 + salary);
        assert_eq!(Balances::free_balance(4), before_4 + salary);
    });
}

#[test]
fn on_idle_inactive_schedule_not_processed() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 0));

        // Deactivate the schedule
        assert_ok!(Payroll::update_schedule(RuntimeOrigin::signed(1), 0, 50, false));

        let before_3 = Balances::free_balance(3);
        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        // No payment — schedule is inactive
        assert_eq!(Balances::free_balance(3), before_3);
    });
}

// ===== BATCH PAYMENT EDGE CASES =====

#[test]
fn batch_payment_skips_inactive_employees() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let employer_before = Balances::free_balance(1);

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        add_employee_default(1, 4, salary);
        add_employee_default(1, 5, salary);

        // Deactivate employee 4
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 4, false));

        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(1)));

        // Only 2 active employees paid
        assert_eq!(Balances::free_balance(1), employer_before - (salary * 2));

        let emp3 = crate::Employees::<Test>::get(1, 3).unwrap();
        let emp4 = crate::Employees::<Test>::get(1, 4).unwrap();
        let emp5 = crate::Employees::<Test>::get(1, 5).unwrap();
        assert_eq!(emp3.total_paid, salary);
        assert_eq!(emp4.total_paid, 0); // inactive, not paid
        assert_eq!(emp5.total_paid, salary);
    });
}

#[test]
fn batch_payment_with_deductions_applied() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let tax = 1_000_000_000u64;
        let employer_before = Balances::free_balance(1);

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        add_employee_default(1, 4, salary);

        // Set deductions on employee 3 only
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, tax));

        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(1)));

        // Employer pays net amounts: (salary - tax) + salary
        let net_3 = salary - tax;
        assert_eq!(Balances::free_balance(1), employer_before - net_3 - salary);

        let emp3 = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp3.total_paid, salary);
        assert_eq!(emp3.total_deductions, tax);

        let emp4 = crate::Employees::<Test>::get(1, 4).unwrap();
        assert_eq!(emp4.total_paid, salary);
        assert_eq!(emp4.total_deductions, 0);
    });
}

#[test]
fn batch_payment_no_active_employees() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        // Deactivate all employees
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, false));

        let employer_before = Balances::free_balance(1);
        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(1)));

        // No balance change
        assert_eq!(Balances::free_balance(1), employer_before);
    });
}

// ===== BONUS / PAYMENT EDGE CASES =====

#[test]
fn issue_bonus_to_inactive_employee_works() {
    // FINDING: issue_bonus does NOT check emp.active — bonuses can be paid to inactive employees.
    // This is arguably correct (e.g. severance pay to suspended employee).
    new_test_ext().execute_with(|| {
        let bonus = 5_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        // Deactivate employee
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, false));

        let before_3 = Balances::free_balance(3);
        assert_ok!(Payroll::issue_bonus(
            RuntimeOrigin::signed(1), 3, bonus, PaymentCategory::Severance
        ));

        assert_eq!(Balances::free_balance(3), before_3 + bonus);
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, bonus);
        assert!(!emp.active); // still inactive
    });
}

#[test]
fn deductions_exceed_salary_net_is_zero() {
    // When deductions >= salary, net_amount = salary.saturating_sub(deductions) = 0
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let huge_deduction = 15_000_000_000u64;

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::set_deduction(
            RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, huge_deduction
        ));

        let before_1 = Balances::free_balance(1);
        let before_3 = Balances::free_balance(3);

        // Payment should succeed — net transfer = 0
        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        // No actual transfer occurred (net = 0)
        assert_eq!(Balances::free_balance(1), before_1);
        assert_eq!(Balances::free_balance(3), before_3);

        // But total_paid still reflects gross salary
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary);
        assert_eq!(emp.total_deductions, huge_deduction);
    });
}

#[test]
fn all_payment_categories_recordable() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        let categories = vec![
            PaymentCategory::Bonus,
            PaymentCategory::Overtime,
            PaymentCategory::Commission,
            PaymentCategory::Reimbursement,
            PaymentCategory::Severance,
        ];

        for (i, category) in categories.into_iter().enumerate() {
            assert_ok!(Payroll::issue_bonus(
                RuntimeOrigin::signed(1), 3, 1_000_000, category.clone()
            ));

            let record = crate::PayrollRecords::<Test>::get(i as u64).unwrap();
            assert_eq!(record.category, category);
        }
    });
}

#[test]
fn payment_record_commitment_is_deterministic() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);

        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        let record = crate::PayrollRecords::<Test>::get(0u64).unwrap();
        let expected = Payroll::compute_payment_commitment(
            &salary, &0u64, &salary, &1u64, &3u64,
        );
        assert_eq!(record.payment_commitment, expected);
    });
}

// ===== WORKER TYPE AND EMPLOYER TYPE COVERAGE =====

#[test]
fn all_worker_types_addable() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        let worker_types = vec![
            (3, WorkerType::FullTime),
            (4, WorkerType::PartTime),
            (5, WorkerType::Contractor),
            (6, WorkerType::Freelancer),
        ];

        for (emp_id, wt) in worker_types {
            assert_ok!(Payroll::add_employee(
                RuntimeOrigin::signed(1), emp_id, 10_000_000_000,
                wt.clone(), 0, [1u8; 32],
            ));
            let emp = crate::Employees::<Test>::get(1, emp_id).unwrap();
            assert_eq!(emp.worker_type, wt);
        }
    });
}

#[test]
fn seasonal_and_intern_worker_types() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(1), 3, 10_000_000_000,
            WorkerType::Seasonal, 0, [1u8; 32],
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(1), 4, 10_000_000_000,
            WorkerType::Intern, 0, [1u8; 32],
        ));

        let emp3 = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp3.worker_type, WorkerType::Seasonal);

        let emp4 = crate::Employees::<Test>::get(1, 4).unwrap();
        assert_eq!(emp4.worker_type, WorkerType::Intern);
    });
}

#[test]
fn all_employer_types_verifiable() {
    new_test_ext().execute_with(|| {
        let types = vec![
            (10u64, EmployerType::Government),
            (11u64, EmployerType::Enterprise),
            (12u64, EmployerType::SME),
            (13u64, EmployerType::Cooperative),
            (14u64, EmployerType::GigPlatform),
            (15u64, EmployerType::NonProfit),
        ];

        for (account, et) in types {
            verify_employer_as_root(account, et.clone());
            let profile = crate::EmployerProfiles::<Test>::get(account).unwrap();
            assert_eq!(profile.employer_type, et);
            assert!(profile.verified);
        }
    });
}

// ===== SCHEDULE EDGE CASES =====

#[test]
fn create_schedule_small_interval_triggers_fast() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        // Interval of 1 block
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 1, 0));

        let schedule = crate::PayrollSchedules::<Test>::get(1, 0u32).unwrap();
        assert_eq!(schedule.next_payment, 2); // created at block 1, next at block 2

        let before_3 = Balances::free_balance(3);
        System::set_block_number(2);
        Payroll::on_idle(2, Weight::from_parts(u64::MAX, u64::MAX));

        assert_eq!(Balances::free_balance(3), before_3 + salary);
    });
}

#[test]
fn schedule_deactivation_prevents_processing() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 0));

        // Deactivate
        assert_ok!(Payroll::update_schedule(RuntimeOrigin::signed(1), 0, 50, false));

        let before_3 = Balances::free_balance(3);
        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        assert_eq!(Balances::free_balance(3), before_3); // not paid
    });
}

#[test]
fn schedule_advances_after_processing() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        // First trigger at block 101
        System::set_block_number(101);
        Payroll::on_idle(101, Weight::from_parts(u64::MAX, u64::MAX));

        let schedule = crate::PayrollSchedules::<Test>::get(1, 0u32).unwrap();
        assert_eq!(schedule.next_payment, 201); // 101 + 100
        assert_eq!(schedule.payments_made, 1);

        // Should NOT trigger again at block 150
        let _before = Balances::free_balance(3);
        System::set_block_number(150);
        Payroll::on_idle(150, Weight::from_parts(u64::MAX, u64::MAX));

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary); // only paid once
    });
}

#[test]
fn schedule_with_department_employee_count() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        assert_ok!(Payroll::create_department(RuntimeOrigin::signed(1), [0xAA; 32]));

        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(1), 3, 10_000_000_000, WorkerType::FullTime, 1, [1u8; 32],
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(1), 4, 10_000_000_000, WorkerType::FullTime, 1, [1u8; 32],
        ));
        add_employee_default(1, 5, 10_000_000_000); // department 0

        // Schedule for department 1
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 1));

        let schedule = crate::PayrollSchedules::<Test>::get(1, 0u32).unwrap();
        assert_eq!(schedule.employee_count, 2); // only dept 1 employees
        assert_eq!(schedule.department_id, 1);
    });
}

// ===== MISCELLANEOUS EDGE CASES =====

#[test]
fn verify_employer_re_verification_rejected() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        let p1 = crate::EmployerProfiles::<Test>::get(1).unwrap();
        assert_eq!(p1.employer_type, EmployerType::Enterprise);

        // CRIT-1: Re-verification of existing employer is rejected
        assert_noop!(
            Payroll::verify_employer(RuntimeOrigin::root(), 1, EmployerType::Government),
            Error::<Test>::EmployerAlreadyRegistered
        );

        // Profile unchanged
        let p2 = crate::EmployerProfiles::<Test>::get(1).unwrap();
        assert_eq!(p2.employer_type, EmployerType::Enterprise);
    });
}

#[test]
fn toggle_status_double_suspend_idempotent() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, false));
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, false));

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert!(!emp.active);
    });
}

#[test]
fn toggle_status_double_activate_idempotent() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        // Already active, setting active again
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 3, true));

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert!(emp.active);
    });
}

#[test]
fn global_stats_track_multiple_operations() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);

        add_employee_default(1, 3, salary);
        add_employee_default(1, 4, salary);
        add_employee_default(2, 5, salary);

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 3);
        assert_eq!(stats.total_employers, 2);

        // Execute payments
        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));
        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(2), 5));

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_disbursed, salary * 2);
        assert_eq!(stats.payments_this_period, 2);

        // Remove employee — stats update
        assert_ok!(Payroll::remove_employee(RuntimeOrigin::signed(1), 4));

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 2);
        assert_eq!(stats.total_employers, 2); // employer 1 still has employee 3
    });
}

#[test]
fn remove_last_employee_decrements_employer_count() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);
        add_employee_default(1, 4, 10_000_000_000);

        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employers, 1);

        assert_ok!(Payroll::remove_employee(RuntimeOrigin::signed(1), 3));
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employers, 1); // still has employee 4

        assert_ok!(Payroll::remove_employee(RuntimeOrigin::signed(1), 4));
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employers, 0); // last employee removed
        assert_eq!(stats.total_employees, 0);
    });
}

#[test]
fn multiple_deduction_types_calculated_correctly() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let tax = 1_000_000_000u64;
        let pension = 500_000_000u64;
        let health = 300_000_000u64;

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, tax));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::Pension, pension));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::HealthInsurance, health));

        let total_ded = Payroll::calculate_deductions(&1, &3);
        assert_eq!(total_ded, tax + pension + health);

        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        let net = salary - (tax + pension + health);
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary);
        assert_eq!(emp.total_deductions, tax + pension + health);

        // Employee received net amount
        let employee_balance = Balances::free_balance(3);
        assert_eq!(employee_balance, 10_000_000_000u64 + net); // initial + net
    });
}

#[test]
fn set_deduction_custom_type_with_different_ids() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        let custom1 = DeductionType::Custom([1u8; 16]);
        let custom2 = DeductionType::Custom([2u8; 16]);

        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, custom1.clone(), 100_000));
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, custom2.clone(), 200_000));

        let deductions = crate::EmployeeDeductions::<Test>::get(1, 3);
        assert_eq!(deductions.len(), 2);
        assert_eq!(deductions[0].deduction_type, custom1);
        assert_eq!(deductions[1].deduction_type, custom2);
        assert_eq!(deductions[0].amount, 100_000);
        assert_eq!(deductions[1].amount, 200_000);
    });
}

#[test]
fn execute_payment_updates_last_paid_block() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        System::set_block_number(42);
        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.last_paid, 42);
    });
}

#[test]
fn create_schedule_increments_schedule_id() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);

        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 200, 0));
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 300, 0));

        let s0 = crate::PayrollSchedules::<Test>::get(1, 0u32).unwrap();
        let s1 = crate::PayrollSchedules::<Test>::get(1, 1u32).unwrap();
        let s2 = crate::PayrollSchedules::<Test>::get(1, 2u32).unwrap();

        assert_eq!(s0.id, 0);
        assert_eq!(s1.id, 1);
        assert_eq!(s2.id, 2);

        assert_eq!(s0.frequency, PaymentFrequency::Custom(100));
        assert_eq!(s1.frequency, PaymentFrequency::Custom(200));
        assert_eq!(s2.frequency, PaymentFrequency::Custom(300));
    });
}

#[test]
fn payment_record_id_increments_across_operations() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, 10_000_000_000);

        // First payment
        assert_ok!(Payroll::execute_payment(RuntimeOrigin::signed(1), 3));
        let r0 = crate::PayrollRecords::<Test>::get(0u64).unwrap();
        assert_eq!(r0.id, 0);

        // Bonus
        assert_ok!(Payroll::issue_bonus(RuntimeOrigin::signed(1), 3, 1_000_000, PaymentCategory::Bonus));
        let r1 = crate::PayrollRecords::<Test>::get(1u64).unwrap();
        assert_eq!(r1.id, 1);
        assert_eq!(r1.category, PaymentCategory::Bonus);
    });
}

#[test]
fn set_deduction_on_nonexistent_employee_for_wrong_employer() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, 10_000_000_000);

        // Employer 2 tries to set deduction on employer 1's employee
        assert_noop!(
            Payroll::set_deduction(RuntimeOrigin::signed(2), 3, DeductionType::IncomeTax, 1_000_000),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn issue_bonus_wrong_employer() {
    new_test_ext().execute_with(|| {
        verify_employer_as_root(1, EmployerType::Enterprise);
        verify_employer_as_root(2, EmployerType::SME);
        add_employee_default(1, 3, 10_000_000_000);

        assert_noop!(
            Payroll::issue_bonus(RuntimeOrigin::signed(2), 3, 1_000_000, PaymentCategory::Bonus),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn batch_payment_emits_correct_count_with_mixed_status() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        add_employee_default(1, 4, salary);
        add_employee_default(1, 5, salary);

        // Deactivate employee 4
        assert_ok!(Payroll::toggle_employee_status(RuntimeOrigin::signed(1), 4, false));

        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(1)));

        // Batch event should show count=2 (only active employees)
        let total_gross = salary * 2;
        let batch_commitment = sp_core::hashing::blake2_256(&total_gross.encode());
        System::assert_has_event(
            Event::BatchPaymentCompleted {
                employer: 1,
                count: 2,
                batch_commitment,
            }.into()
        );
    });
}

#[test]
fn on_idle_with_deductions_applied_in_scheduled_payment() {
    new_test_ext().execute_with(|| {
        let salary = 10_000_000_000u64;
        let tax = 1_000_000_000u64;

        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::set_deduction(RuntimeOrigin::signed(1), 3, DeductionType::IncomeTax, tax));
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 50, 0));

        let before_3 = Balances::free_balance(3);
        System::set_block_number(51);
        Payroll::on_idle(51, Weight::from_parts(u64::MAX, u64::MAX));

        // Employee receives net (salary - tax)
        let net = salary - tax;
        assert_eq!(Balances::free_balance(3), before_3 + net);

        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary);
        assert_eq!(emp.total_deductions, tax);
    });
}

// ================================
// Regression Tests — Audit Fix Verification
// ================================

/// REGRESSION (PY-1): Repeated on_idle calls at the same block must NOT
/// double-pay employees. The schedule.next_payment guard must prevent re-entry.
#[test]
fn on_idle_does_not_double_pay() {
    new_test_ext().execute_with(|| {
        let salary = 5_000_000_000u64;
        verify_employer_as_root(1, EmployerType::Enterprise);
        add_employee_default(1, 3, salary);
        assert_ok!(Payroll::create_schedule(RuntimeOrigin::signed(1), 100, 0));

        // First trigger at block 101
        System::set_block_number(101);
        Payroll::on_idle(101, Weight::from_parts(u64::MAX, u64::MAX));

        let paid_once = Balances::free_balance(3);
        let emp = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp.total_paid, salary);

        // Second on_idle at same block — must be a no-op
        Payroll::on_idle(101, Weight::from_parts(u64::MAX, u64::MAX));
        assert_eq!(Balances::free_balance(3), paid_once);

        let emp2 = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp2.total_paid, salary); // still only paid once

        // Third on_idle at intermediate block — also no-op
        System::set_block_number(150);
        Payroll::on_idle(150, Weight::from_parts(u64::MAX, u64::MAX));
        assert_eq!(Balances::free_balance(3), paid_once);

        let emp3 = crate::Employees::<Test>::get(1, 3).unwrap();
        assert_eq!(emp3.total_paid, salary); // still only paid once
    });
}