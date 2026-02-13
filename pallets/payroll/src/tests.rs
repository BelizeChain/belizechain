use crate::{mock::*, Error, Event, PaymentFrequency, WorkerType, EmployerType, PaymentCategory, DeductionType};
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    traits::OnInitialize,
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
        add_employee_default(1, 3, 2_000_000_000_000_000);

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
        add_employee_default(1, 3, 600_000_000_000_000);
        add_employee_default(1, 4, 600_000_000_000_000);

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
        Payroll::on_initialize(101);

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

        assert_noop!(
            Payroll::issue_bonus(RuntimeOrigin::signed(1), 3, 2_000_000_000_000_000, PaymentCategory::Bonus),
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