use crate::{mock::*, Error, Event, PaymentFrequency};
use frame_support::{
    assert_noop, assert_ok,
    traits::OnInitialize,
};

// ===== EMPLOYER VERIFICATION TESTS =====

#[test]
fn verify_employer_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let admin = 100;

        // Verify employer
        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(admin), employer));

        // Check verification
        assert!(crate::VerifiedEmployers::<Test>::get(employer));

        // Check event
        System::assert_has_event(
            Event::EmployerVerified { employer }.into()
        );
    });
}

// ===== EMPLOYEE MANAGEMENT TESTS =====

#[test]
fn add_employee_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000; // 10K DALLA
        let metadata = [1u8; 32];

        // Verify employer first
        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));

        // Add employee
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            metadata
        ));

        // Check employee exists
        let emp = crate::Employees::<Test>::get(employer, employee).unwrap();
        assert_eq!(emp.salary, salary);
        assert!(emp.active);
        assert_eq!(emp.total_paid, 0);

        // Check stats
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 1);
        assert_eq!(stats.total_employers, 1);

        // Check event
        System::assert_has_event(
            Event::EmployeeAdded {
                employer,
                employee,
                salary,
            }.into()
        );
    });
}

#[test]
fn add_employee_fails_not_verified() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        // Try to add employee without verification
        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(employer),
                employee,
                salary,
                metadata
            ),
            Error::<Test>::EmployerNotVerified
        );
    });
}

#[test]
fn add_employee_fails_payment_too_low() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 100; // Below minimum (1_000_000)
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));

        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(employer),
                employee,
                salary,
                metadata
            ),
            Error::<Test>::PaymentTooLow
        );
    });
}

#[test]
fn add_employee_fails_already_exists() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            metadata
        ));

        // Try to add same employee again
        assert_noop!(
            Payroll::add_employee(
                RuntimeOrigin::signed(employer),
                employee,
                salary,
                metadata
            ),
            Error::<Test>::EmployeeAlreadyExists
        );
    });
}

#[test]
fn remove_employee_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            metadata
        ));

        // Remove employee
        assert_ok!(Payroll::remove_employee(
            RuntimeOrigin::signed(employer),
            employee
        ));

        // Check employee removed
        assert!(!crate::Employees::<Test>::contains_key(employer, employee));

        // Check stats
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 0);
        assert_eq!(stats.total_employers, 0);

        // Check event
        System::assert_has_event(
            Event::EmployeeRemoved {
                employer,
                employee,
            }.into()
        );
    });
}

#[test]
fn remove_employee_fails_not_found() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;

        assert_noop!(
            Payroll::remove_employee(
                RuntimeOrigin::signed(employer),
                employee
            ),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn update_salary_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let old_salary = 10_000_000_000;
        let new_salary = 15_000_000_000;
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            old_salary,
            metadata
        ));

        // Update salary
        assert_ok!(Payroll::update_salary(
            RuntimeOrigin::signed(employer),
            employee,
            new_salary
        ));

        // Check updated salary
        let emp = crate::Employees::<Test>::get(employer, employee).unwrap();
        assert_eq!(emp.salary, new_salary);

        // Check event
        System::assert_has_event(
            Event::SalaryUpdated {
                employer,
                employee,
                old_salary,
                new_salary,
            }.into()
        );
    });
}

#[test]
fn update_salary_fails_not_found() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let new_salary = 15_000_000_000;

        assert_noop!(
            Payroll::update_salary(
                RuntimeOrigin::signed(employer),
                employee,
                new_salary
            ),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn update_salary_fails_too_low() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let old_salary = 10_000_000_000;
        let new_salary = 100; // Below minimum
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            old_salary,
            metadata
        ));

        assert_noop!(
            Payroll::update_salary(
                RuntimeOrigin::signed(employer),
                employee,
                new_salary
            ),
            Error::<Test>::PaymentTooLow
        );
    });
}

// ===== PAYMENT EXECUTION TESTS =====

#[test]
fn execute_payment_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        let employer_balance_before = Balances::free_balance(employer);
        let employee_balance_before = Balances::free_balance(employee);

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            metadata
        ));

        // Execute payment
        assert_ok!(Payroll::execute_payment(
            RuntimeOrigin::signed(employer),
            employee
        ));

        // Check balances
        assert_eq!(
            Balances::free_balance(employer),
            employer_balance_before - salary
        );
        assert_eq!(
            Balances::free_balance(employee),
            employee_balance_before + salary
        );

        // Check employee record updated
        let emp = crate::Employees::<Test>::get(employer, employee).unwrap();
        assert_eq!(emp.total_paid, salary);
        assert_eq!(emp.last_paid, 1);

        // Check stats
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_disbursed, salary);
        assert_eq!(stats.payments_this_period, 1);

        // Check payment record created
        let record = crate::PayrollRecords::<Test>::get(0).unwrap();
        assert_eq!(record.employer, employer);
        assert_eq!(record.employee, employee);
        assert_eq!(record.amount, salary);
    });
}

#[test]
fn execute_payment_fails_not_found() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;

        assert_noop!(
            Payroll::execute_payment(
                RuntimeOrigin::signed(employer),
                employee
            ),
            Error::<Test>::EmployeeNotFound
        );
    });
}

#[test]
fn execute_payment_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 2_000_000_000_000_000; // More than employer has
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            metadata
        ));

        assert_noop!(
            Payroll::execute_payment(
                RuntimeOrigin::signed(employer),
                employee
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn batch_payment_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee1 = 3;
        let employee2 = 4;
        let employee3 = 5;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        let employer_balance_before = Balances::free_balance(employer);

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee1,
            salary,
            metadata
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee2,
            salary,
            metadata
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee3,
            salary,
            metadata
        ));

        // Execute batch payment
        assert_ok!(Payroll::batch_payment(RuntimeOrigin::signed(employer)));

        // Check employer balance
        assert_eq!(
            Balances::free_balance(employer),
            employer_balance_before - (salary * 3)
        );

        // Check all employees paid
        let emp1 = crate::Employees::<Test>::get(employer, employee1).unwrap();
        assert_eq!(emp1.total_paid, salary);
        let emp2 = crate::Employees::<Test>::get(employer, employee2).unwrap();
        assert_eq!(emp2.total_paid, salary);
        let emp3 = crate::Employees::<Test>::get(employer, employee3).unwrap();
        assert_eq!(emp3.total_paid, salary);

        // Check stats
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_disbursed, salary * 3);
        assert_eq!(stats.payments_this_period, 3);

        // Check event
        System::assert_has_event(
            Event::BatchPaymentCompleted {
                employer,
                count: 3,
                total_amount: salary * 3,
            }.into()
        );
    });
}

#[test]
fn batch_payment_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee1 = 3;
        let employee2 = 4;
        let salary = 600_000_000_000_000; // Total > employer balance
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee1,
            salary,
            metadata
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee2,
            salary,
            metadata
        ));

        assert_noop!(
            Payroll::batch_payment(RuntimeOrigin::signed(employer)),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ===== SCHEDULE TESTS =====

#[test]
fn create_schedule_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let interval = 432_000u32; // Monthly

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));

        // Create schedule
        assert_ok!(Payroll::create_schedule(
            RuntimeOrigin::signed(employer),
            interval
        ));

        // Check schedule exists
        let schedule = crate::PayrollSchedules::<Test>::get(employer).unwrap();
        assert_eq!(schedule.frequency, PaymentFrequency::Custom(interval));
        assert!(schedule.active);
        assert_eq!(schedule.next_payment, 1 + 432_000); // Current block + monthly interval

        // Check event
        System::assert_has_event(
            Event::ScheduleCreated {
                employer,
                schedule_id: 0,
            }.into()
        );
    });
}

#[test]
fn create_schedule_fails_not_verified() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let interval = 50_400u32; // Weekly

        assert_noop!(
            Payroll::create_schedule(
                RuntimeOrigin::signed(employer),
                interval
            ),
            Error::<Test>::EmployerNotVerified
        );
    });
}

#[test]
fn update_schedule_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let old_interval = 432_000u32; // Monthly
        let new_interval = 100_800u32; // BiWeekly

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::create_schedule(
            RuntimeOrigin::signed(employer),
            old_interval
        ));

        // Update schedule
        assert_ok!(Payroll::update_schedule(
            RuntimeOrigin::signed(employer),
            new_interval,
            true
        ));

        // Check schedule updated
        let schedule = crate::PayrollSchedules::<Test>::get(employer).unwrap();
        assert_eq!(schedule.frequency, PaymentFrequency::Custom(new_interval));
        assert!(schedule.active);
    });
}

#[test]
fn update_schedule_fails_not_found() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let interval = 50_400u32; // Weekly

        assert_noop!(
            Payroll::update_schedule(
                RuntimeOrigin::signed(employer),
                interval,
                true
            ),
            Error::<Test>::ScheduleNotFound
        );
    });
}

#[test]
fn scheduled_payment_processed_automatically() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee = 3;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];
        let interval = 100u32; // 100 blocks

        let employer_balance_before = Balances::free_balance(employer);

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee,
            salary,
            metadata
        ));
        assert_ok!(Payroll::create_schedule(
            RuntimeOrigin::signed(employer),
            interval
        ));

        // Advance to payment block
        System::set_block_number(101);

        // Trigger on_initialize
        Payroll::on_initialize(101);

        // Check payment was made
        assert_eq!(
            Balances::free_balance(employer),
            employer_balance_before - salary
        );

        // Check employee paid
        let emp = crate::Employees::<Test>::get(employer, employee).unwrap();
        assert_eq!(emp.total_paid, salary);

        // Check schedule updated
        let schedule = crate::PayrollSchedules::<Test>::get(employer).unwrap();
        assert_eq!(schedule.next_payment, 101 + 100);
        assert_eq!(schedule.payments_made, 1);
    });
}

// ===== HELPER FUNCTION TESTS =====

#[test]
fn get_total_payroll_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee1 = 3;
        let employee2 = 4;
        let salary1 = 10_000_000_000;
        let salary2 = 15_000_000_000;
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee1,
            salary1,
            metadata
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee2,
            salary2,
            metadata
        ));

        // Check total payroll
        let total = Payroll::get_total_payroll(&employer);
        assert_eq!(total, salary1 + salary2);
    });
}

#[test]
fn get_employee_count_works() {
    new_test_ext().execute_with(|| {
        let employer = 1;
        let employee1 = 3;
        let employee2 = 4;
        let employee3 = 5;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee1,
            salary,
            metadata
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee2,
            salary,
            metadata
        ));
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer),
            employee3,
            salary,
            metadata
        ));

        // Check employee count
        let count = Payroll::get_employee_count(&employer);
        assert_eq!(count, 3);
    });
}

// ===== MULTI-EMPLOYER TESTS =====

#[test]
fn multiple_employers_independent() {
    new_test_ext().execute_with(|| {
        let employer1 = 1;
        let employer2 = 2;
        let employee1 = 3;
        let employee2 = 4;
        let salary = 10_000_000_000;
        let metadata = [1u8; 32];

        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer1));
        assert_ok!(Payroll::verify_employer(RuntimeOrigin::signed(100), employer2));

        // Employer 1 adds employee 1
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer1),
            employee1,
            salary,
            metadata
        ));

        // Employer 2 adds employee 2
        assert_ok!(Payroll::add_employee(
            RuntimeOrigin::signed(employer2),
            employee2,
            salary,
            metadata
        ));

        // Check stats
        let stats = crate::GlobalStats::<Test>::get();
        assert_eq!(stats.total_employees, 2);
        assert_eq!(stats.total_employers, 2);

        // Employer 1 can't manage employer 2's employees
        assert_noop!(
            Payroll::remove_employee(
                RuntimeOrigin::signed(employer1),
                employee2
            ),
            Error::<Test>::EmployeeNotFound
        );
    });
}
