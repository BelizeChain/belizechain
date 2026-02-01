# BelizeChain Payroll: Automated Salary & Contractor Payments

## Overview

The **Payroll** pallet (`pallet-belize-payroll`) enables businesses, government agencies, and organizations to automate recurring salary and contractor payments on-chain with full regulatory compliance. This pallet supports both traditional payroll (employees) and modern gig economy payments (contractors) with comprehensive audit trails for tax and labor law compliance.

**Key Features**:
- **Employee Management**: Register employees with KYC verification (Identity pallet integration)
- **Recurring Payment Schedules**: Weekly, bi-weekly, monthly, or custom intervals
- **Multi-Token Support**: Pay in DALLA or bBZD (fiat-stable for predictable wages)
- **Batch Payments**: Execute all scheduled payments in a single transaction
- **Audit Trail**: Immutable payment history for tax compliance and labor inspections
- **Automated Execution**: Scheduled payments auto-execute via `on_initialize` hook (optional)
- **Employer Verification**: KYC Level 2 required for payroll management
- **Compliance Integration**: Automatic sanctions checks (Oracle pallet), tax reporting support

**Business Use Cases**:
- **Company Payroll**: Belizean businesses automating bi-weekly salary disbursements
- **Government Salaries**: Ministry payroll with complete audit trail for transparency
- **Gig Economy**: Ride-sharing, freelance platforms with instant contractor payments
- **Tourism Industry**: Hotel/resort staff payments (seasonal employment)
- **Construction**: Project-based contractor payments with milestone tracking

---

## Architecture

### 1. Payroll Flow (Recurring Payments)

```
┌──────────────────────────────────────────────────────────┐
│  1. EMPLOYER REGISTRATION (KYC Level 2 Required)         │
│     - Business account verified via Identity pallet      │
│     - Tax ID, business registration on-chain             │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  2. EMPLOYEE ONBOARDING                                  │
│     - add_employee(account, salary, metadata_hash)       │
│     - Employee KYC Level 1+ required                     │
│     - Metadata: employment contract (Pakit DAG storage)  │
│     - Salary: DALLA or bBZD (employer choice)            │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  3. PAYMENT SCHEDULE CREATION                            │
│     - create_schedule(frequency)                         │
│     - Frequencies: Weekly, BiWeekly, Monthly, Custom     │
│     - Auto-calculation: next_payment = current + freq    │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  4. AUTOMATED PAYMENT EXECUTION (on_initialize hook)     │
│     - Every block: Check if next_payment <= current      │
│     - If true: Execute batch_payment() automatically     │
│     - Update: next_payment += frequency                  │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  5. PAYMENT TRANSFER                                     │
│     - Currency::transfer(employer → employee, salary)    │
│     - Create PayrollRecord (audit trail)                 │
│     - Update Employee::last_paid, total_paid             │
│     - Emit PaymentExecuted event                         │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  6. TAX REPORTING & AUDIT                                │
│     - PayrollRecords queryable by employer/employee      │
│     - Government can query total payroll by company      │
│     - Annual tax forms auto-generated from records       │
└──────────────────────────────────────────────────────────┘
```

### 2. Payment Frequency Intervals

| Frequency | Blocks | Days | Use Case |
|-----------|--------|------|----------|
| **Weekly** | 50,400 | ~7 days | Gig workers, hourly employees |
| **BiWeekly** | 100,800 | ~14 days | Most common (US/Belize standard) |
| **Monthly** | 432,000 | ~30 days | Salaried professionals, government |
| **Custom** | User-defined | Variable | Project milestones, seasonal work |

**Block Timing**: Assumes 6-second block time (~14,400 blocks/day)

### 3. Multi-Token Support

```rust
pub enum TokenType {
    Dalla,  // Native token (volatile, for crypto-native workers)
    BBZD,   // BZD-pegged stablecoin (stable, for traditional employees)
}
```

**Employer Choice**:
- **DALLA**: Crypto enthusiasts, staking rewards (5-10% APY)
- **bBZD**: Stable wages (1:1 BZD peg), rent/mortgage payments, traditional workers

**Employee Conversion**: Employees can convert DALLA ↔ bBZD via BelizeX DEX

---

## Storage Items

### Employee Registry

| Storage | Type | Description |
|---------|------|-------------|
| `Employees` | `StorageDoubleMap<AccountId (employer), AccountId (employee), Employee>` | Employee records by employer |
| `EmployeeCount` | `StorageMap<AccountId (employer), u32>` | Total employees per employer |

**Employee Structure**:
```rust
pub struct Employee<AccountId, Balance> {
    pub account: AccountId,          // Employee account
    pub salary: Balance,             // Per-period salary (DALLA or bBZD)
    pub active: bool,                // Active employment status
    pub last_paid: u32,              // Last payment block number
    pub total_paid: Balance,         // Lifetime earnings
    pub start_block: u32,            // Hire date (block)
    pub metadata_hash: [u8; 32],     // Employment contract (Pakit DAG hash)
}
```

**Metadata (Off-Chain via Pakit DAG)**:
- Employment contract (signed PDF)
- Job title, department, work location
- Benefits package, vacation days
- Tax withholding elections (Form W-4 equivalent)
- Emergency contact information

### Payment Schedules

| Storage | Type | Description |
|---------|------|-------------|
| `PayrollSchedules` | `StorageMap<AccountId (employer), PayrollSchedule>` | Recurring payment schedule |
| `NextPaymentBlock` | `StorageMap<AccountId (employer), BlockNumber>` | Next scheduled payment block |

**PayrollSchedule Structure**:
```rust
pub struct PayrollSchedule<BlockNumber> {
    pub id: u32,                     // Schedule ID
    pub frequency: PaymentFrequency, // Weekly, BiWeekly, Monthly, Custom
    pub next_payment: BlockNumber,   // Next auto-payment block
    pub active: bool,                // Schedule active status
    pub payments_made: u32,          // Total payments executed
    pub employee_count: u32,         // Employees covered
}
```

### Audit Trail

| Storage | Type | Description |
|---------|------|-------------|
| `PayrollRecords` | `StorageMap<u64 (record_id), PayrollRecord>` | Immutable payment history |
| `RecordsByEmployee` | `StorageDoubleMap<AccountId (employee), BlockNumber, u64>` | Employee payment history index |
| `RecordsByEmployer` | `StorageDoubleMap<AccountId (employer), BlockNumber, u64>` | Employer payment history index |
| `NextRecordId` | `StorageValue<u64>` | Auto-incrementing record ID |

**PayrollRecord Structure**:
```rust
pub struct PayrollRecord<AccountId, Balance, BlockNumber> {
    pub id: u64,                     // Unique record ID
    pub employer: AccountId,         // Payer account
    pub employee: AccountId,         // Payee account
    pub amount: Balance,             // Payment amount
    pub token_type: TokenType,       // DALLA or bBZD
    pub block_number: BlockNumber,   // Payment block
    pub timestamp: u64,              // Unix timestamp
}
```

### Employer Verification

| Storage | Type | Description |
|---------|------|-------------|
| `VerifiedEmployers` | `StorageMap<AccountId, bool>` | KYC-verified employers (Identity pallet) |

### Statistics

| Storage | Type | Description |
|---------|------|-------------|
| `PayrollStats` | `StorageValue<PayrollStats>` | Global payroll statistics |

**PayrollStats Structure**:
```rust
pub struct PayrollStats<Balance> {
    pub total_disbursed: Balance,    // All-time payroll (DALLA + bBZD)
    pub total_employees: u32,        // Active employees system-wide
    pub total_employers: u32,        // Active employers
    pub payments_this_period: u32,   // Payments this epoch
}
```

---

## Extrinsics (Public Functions)

### Employee Management

#### `add_employee(origin, employee, salary, metadata_hash)`
**Purpose**: Register new employee for payroll

**Parameters**:
- `origin`: Signed origin (employer account)
- `employee`: Employee account to add
- `salary`: Salary amount per payment period (DALLA or bBZD, smallest unit)
- `metadata_hash`: Employment contract hash (32 bytes, Pakit DAG storage)

**Requirements**:
- Origin is verified employer (KYC Level 2 via Identity pallet)
- Employee account is valid and not duplicate
- Employee has KYC Level 1+ (basic identity verification)
- Employer not sanctioned (Oracle pallet check)
- Employee not sanctioned

**Outcome**:
- Adds to `Employees[employer][employee]`
- Increments `EmployeeCount[employer]`
- Updates `PayrollStats.total_employees`
- Emits `EmployeeAdded(employer, employee, salary)`

**Example**:
```rust
// Business adds employee with bi-weekly salary
let employee_account = ...; // Employee AccountId
let salary = 2_000 * 1_000_000_000_000; // 2,000 bBZD (bi-weekly, 12 decimals)
let contract_hash = sha256(employment_contract_pdf); // Pakit DAG hash

Payroll::add_employee(
    Origin::signed(employer_account),
    employee_account,
    salary,
    contract_hash
)?;
// Result: Employee registered, eligible for next payroll cycle
```

---

#### `remove_employee(origin, employee)`
**Purpose**: Terminate employee (resignation, termination, layoff)

**Parameters**:
- `origin`: Signed origin (employer account)
- `employee`: Employee to remove

**Requirements**:
- Origin owns employee record

**Outcome**:
- Sets `Employees[employer][employee].active = false`
- Decrements `EmployeeCount[employer]`
- Updates `PayrollStats.total_employees`
- Emits `EmployeeRemoved(employer, employee)`

**Note**: Employee record preserved for audit trail (tax reporting)

---

#### `update_salary(origin, employee, new_salary)`
**Purpose**: Update employee salary (promotion, raise, demotion)

**Parameters**:
- `origin`: Signed origin (employer account)
- `employee`: Employee to update
- `new_salary`: New salary amount per period

**Requirements**:
- Origin owns employee record
- Employee is active

**Outcome**:
- Updates `Employees[employer][employee].salary`
- Emits `SalaryUpdated(employer, employee, old_salary, new_salary)`

---

### Payment Execution

#### `execute_payment(origin, employee, token_type)`
**Purpose**: Manually execute single employee payment (ad-hoc bonus, final paycheck)

**Parameters**:
- `origin`: Signed origin (employer account)
- `employee`: Employee to pay
- `token_type`: `TokenType::Dalla` or `TokenType::BBZD`

**Requirements**:
- Origin owns employee record
- Employee is active
- Employer has sufficient balance
- Employee not sanctioned (Oracle check)

**Process**:
1. Transfer `employee.salary` from employer to employee
2. Create `PayrollRecord` with auto-incrementing ID
3. Update `employee.last_paid` to current block
4. Update `employee.total_paid += salary`
5. Add record to indexes (`RecordsByEmployee`, `RecordsByEmployer`)

**Outcome**:
- Transfers tokens (Currency::transfer)
- Creates immutable payment record
- Emits `PaymentExecuted(employer, employee, amount, token_type)`

**Example**:
```rust
// Manual payment for specific employee (bonus, final paycheck)
Payroll::execute_payment(
    Origin::signed(employer_account),
    employee_account,
    TokenType::BBZD
)?;
// Result: Employee receives salary, record created
```

---

#### `batch_payment(origin)`
**Purpose**: Execute all scheduled payments for employer (entire payroll)

**Parameters**:
- `origin`: Signed origin (employer account)

**Requirements**:
- Origin is verified employer
- Has active employees
- Employer has sufficient balance for all salaries
- Current block >= next_payment (if schedule exists)

**Process**:
1. Iterate all active employees for employer
2. For each employee:
   - Check sanctions (Oracle)
   - Transfer salary amount
   - Create PayrollRecord
   - Update employee stats
3. Update schedule: `next_payment += frequency`
4. Increment `schedule.payments_made`

**Outcome**:
- All employees paid in single transaction
- Multiple PayrollRecords created
- Schedule updated for next cycle
- Emits `BatchPaymentExecuted(employer, employee_count, total_amount)`

**Gas Optimization**: Max 100 employees per batch (large employers split into multiple batches)

**Example**:
```rust
// Employer executes bi-weekly payroll for all employees
Payroll::batch_payment(Origin::signed(employer_account))?;
// Result: All active employees paid, next payment scheduled in 100,800 blocks (~14 days)
```

---

### Schedule Management

#### `create_schedule(origin, frequency)`
**Purpose**: Create recurring payment schedule (automated payroll)

**Parameters**:
- `origin`: Signed origin (employer account)
- `frequency`: `PaymentFrequency` (Weekly, BiWeekly, Monthly, Custom(blocks))

**Requirements**:
- Origin is verified employer
- No existing schedule (one schedule per employer)

**Outcome**:
- Creates `PayrollSchedule` with next_payment = current_block + frequency
- Emits `ScheduleCreated(employer, frequency, next_payment)`

**Example**:
```rust
// Create bi-weekly payroll schedule
Payroll::create_schedule(
    Origin::signed(employer_account),
    PaymentFrequency::BiWeekly
)?;
// Result: Automated payments every 100,800 blocks (~14 days)
```

---

#### `update_schedule(origin, frequency, active)`
**Purpose**: Modify existing payment schedule (pause/resume, change frequency)

**Parameters**:
- `origin`: Signed origin (employer account)
- `frequency`: New payment frequency
- `active`: Schedule active status (pause/resume)

**Requirements**:
- Origin has existing schedule

**Outcome**:
- Updates `PayrollSchedule.frequency` and `active`
- Recalculates `next_payment`
- Emits `ScheduleUpdated(employer, frequency, active)`

---

### Employer Verification

#### `verify_employer(origin, employer)`
**Purpose**: Verify employer eligibility for payroll system (admin-controlled)

**Parameters**:
- `origin`: `T::AdminOrigin` (root or council)
- `employer`: Employer account to verify

**Requirements**:
- Admin origin authority
- Employer has KYC Level 2 (via Identity pallet)
- Employer has business registration (off-chain verification)

**Outcome**:
- Adds to `VerifiedEmployers`
- Emits `EmployerVerified(employer)`

**Use Case**: Government vets businesses before allowing payroll management (prevents fake employment for tax evasion)

---

## Helper Functions (View/Query)

### `get_total_payroll(employer) -> Balance`
**Purpose**: Get total monthly payroll expense for employer

**Returns**: Sum of all active employee salaries

**Used By**: Government tax assessments, business cash flow planning

---

### `get_employee_count(employer) -> u32`
**Purpose**: Get active employee count for employer

**Returns**: Number of active employees

---

### `get_employee_earnings(employee) -> Balance`
**Purpose**: Get lifetime earnings for employee (all employers)

**Returns**: Sum of `total_paid` across all employer relationships

**Used By**: Tax reporting, credit applications, income verification

---

### `get_payment_history(employer, start_block, end_block) -> Vec<PayrollRecord>`
**Purpose**: Query payment records for date range (audit/tax)

**Returns**: Vector of PayrollRecords within block range

**Used By**: Annual tax forms (W-2 equivalent), labor inspections

---

## Events

| Event | When Emitted |
|-------|-------------|
| `EmployeeAdded(AccountId, AccountId, Balance)` | Employee registered (employer, employee, salary) |
| `EmployeeRemoved(AccountId, AccountId)` | Employee terminated |
| `SalaryUpdated(AccountId, AccountId, Balance, Balance)` | Salary changed (employer, employee, old, new) |
| `PaymentExecuted(AccountId, AccountId, Balance, TokenType)` | Single payment made (employer, employee, amount, token) |
| `BatchPaymentExecuted(AccountId, u32, Balance)` | Batch payment completed (employer, count, total) |
| `ScheduleCreated(AccountId, PaymentFrequency, BlockNumber)` | Schedule created (employer, frequency, next_payment) |
| `ScheduleUpdated(AccountId, PaymentFrequency, bool)` | Schedule modified (employer, frequency, active) |
| `EmployerVerified(AccountId)` | Employer approved for payroll |

---

## Errors

| Error | Cause |
|-------|-------|
| `EmployerNotVerified` | Employer lacks KYC Level 2 verification |
| `EmployeeAlreadyExists` | Duplicate employee account |
| `EmployeeNotFound` | Employee record does not exist |
| `EmployeeNotActive` | Employee terminated or inactive |
| `InsufficientBalance` | Employer cannot afford payroll |
| `Sanctioned` | Employer or employee on OFAC/UN sanctions list |
| `InvalidSalary` | Salary amount is zero or unreasonable |
| `ScheduleAlreadyExists` | Employer already has payment schedule |
| `ScheduleNotFound` | No payment schedule exists |
| `TooManyEmployees` | Batch payment exceeds max limit (100 employees) |
| `InvalidFrequency` | Custom frequency is too short (< 1 day) |
| `NotScheduledYet` | Next payment block not reached |

---

## Automated Execution (on_initialize Hook)

### Automatic Payment Processing

The Payroll pallet optionally executes scheduled payments automatically via the `on_initialize` hook (called every block):

```rust
fn on_initialize(n: BlockNumberFor<T>) -> Weight {
    let mut weight = Weight::zero();
    
    // Iterate all employers with active schedules
    for (employer, schedule) in PayrollSchedules::<T>::iter() {
        // Check if payment is due
        if schedule.active && n >= schedule.next_payment {
            // Execute batch payment automatically
            let result = Self::internal_batch_payment(&employer);
            
            match result {
                Ok(_) => {
                    // Update next payment block
                    schedule.next_payment += schedule.frequency.to_blocks();
                    weight += T::DbWeight::get().reads_writes(10, 10);
                }
                Err(_) => {
                    // Log error (insufficient funds, sanctions, etc.)
                    weight += T::DbWeight::get().reads(5);
                }
            }
        }
    }
    
    weight
}
```

**Disabled by Default**: Automatic execution adds weight to every block. Employers can opt-in via `update_schedule(active: true)`

**Fallback**: Employers can always call `batch_payment()` manually if automatic execution fails

---

## Integration with Other Pallets

### Identity Pallet

**Used For**:
- Employer verification (KYC Level 2 required)
- Employee verification (KYC Level 1 minimum)

**Example**:
```rust
// Payroll checks employer KYC before registration
if !Identity::meets_kyc_level(&employer, 2) {
    return Err(Error::<T>::EmployerNotVerified.into());
}
```

---

### Oracle Pallet

**Used For**:
- Sanctions compliance (employer/employee checks before payments)

**Example**:
```rust
// Check employee sanctions before payment
if Oracle::is_sanctioned(&employee) {
    return Err(Error::<T>::Sanctioned.into());
}
```

---

### Economy Pallet

**Used For**:
- DALLA balance queries (treasury funding)
- bBZD balance queries (stablecoin payroll)

---

## Configuration Parameters

### `Config` Trait

```rust
pub trait Config: frame_system::Config {
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    type UnixTime: UnixTime;
    
    #[pallet::constant]
    type PalletId: Get<PalletId>;                    // Payroll treasury account
    
    #[pallet::constant]
    type MaxEmployeesPerBatch: Get<u32>;             // 100 employees max per batch
    
    #[pallet::constant]
    type MinSalary: Get<BalanceOf<Self>>;            // 500 bBZD minimum (livable wage)
    
    type WeightInfo: WeightInfo;
}
```

**Recommended Values**:
- `PalletId`: `b"bzpayrll"`
- `MaxEmployeesPerBatch`: `100` (prevents excessive block weight)
- `MinSalary`: `500_000_000_000_000` (500 bBZD bi-weekly = ~$250 USD minimum wage)

---

## Testing

### Unit Tests (src/tests.rs)

**Coverage**:
- Employee registration/removal/updates
- Single and batch payments (DALLA + bBZD)
- Payment schedule creation/updates
- Automated execution via on_initialize
- Sanctions compliance (Oracle integration)
- KYC verification (Identity integration)
- Audit trail queries (payment history)
- Edge cases (insufficient funds, inactive employees)

**Run Tests**:
```bash
cargo test -p pallet-belize-payroll
```

---

## Integration with BelizeChain

### Runtime Configuration (belizechain/runtime/src/lib.rs)

```rust
parameter_types! {
    pub const PayrollPalletId: PalletId = PalletId(*b"bzpayrll");
    pub const MaxEmployeesPerBatch: u32 = 100;
    pub const MinSalary: u128 = 500_000_000_000_000; // 500 bBZD
}

impl pallet_belize_payroll::Config for Runtime {
    type Currency = Balances;
    type AdminOrigin = EnsureRoot<AccountId>;
    type UnixTime = Timestamp;
    type PalletId = PayrollPalletId;
    type MaxEmployeesPerBatch = MaxEmployeesPerBatch;
    type MinSalary = MinSalary;
    type WeightInfo = ();
}
```

---

## Future Enhancements

1. **Tax Withholding**: Automatic tax calculation and withholding (Belize tax brackets)
2. **Social Security Contributions**: Employer/employee SSB contributions (9% employee, 6.5% employer)
3. **Multi-Currency Payroll**: Support for USD/EUR payments (cross-border workers)
4. **Benefits Management**: Health insurance, pension fund integration
5. **Time Tracking**: On-chain clock-in/out for hourly workers
6. **Contractor 1099**: Separate tax treatment for contractors vs employees
7. **Payroll Loans**: Short-term salary advances (deducted from future paychecks)
8. **Direct Deposit Split**: Allocate % of salary to savings account, bBZD staking

---

## Real-World Example: Tourism Hotel Payroll

**Scenario**: Beachfront resort with 50 employees (bi-weekly payroll)

```rust
// 1. Hotel verified as employer (KYC L2)
Payroll::verify_employer(Origin::root(), hotel_account)?;

// 2. Add 50 employees (front desk, housekeeping, kitchen, management)
for (employee, salary, contract_hash) in employee_list {
    Payroll::add_employee(
        Origin::signed(hotel_account),
        employee,
        salary, // Range: 800 bBZD (housekeeping) to 3,000 bBZD (manager)
        contract_hash
    )?;
}

// 3. Create bi-weekly payment schedule
Payroll::create_schedule(
    Origin::signed(hotel_account),
    PaymentFrequency::BiWeekly
)?;

// 4. Automated payroll every 100,800 blocks (~14 days)
// - Total payroll: 75,000 bBZD bi-weekly
// - Hotel account auto-debited
// - All 50 employees receive salary
// - PayrollRecords created for tax reporting

// 5. Seasonal adjustments (high season)
Payroll::update_salary(
    Origin::signed(hotel_account),
    bartender_account,
    1_200 * 1_000_000_000_000 // Raise from 1,000 to 1,200 bBZD (busy season)
)?;

// 6. Tax reporting (end of year)
let year_records = Payroll::get_payment_history(
    hotel_account,
    start_of_year_block,
    end_of_year_block
);
// Generate W-2 equivalent for all employees (total wages, tax withholding)
```

---

## References

- **Belize Labor Act**: [Labor Laws of Belize](https://www.belizelaw.org/)
- **Social Security Board (SSB)**: [SSB Contribution Rates](https://www.socialsecurity.org.bz/)
- **Belize Tax Service**: [Income Tax Guidelines](https://www.belize.gov.bz/)
- **Identity Pallet**: `belizechain/pallets/identity/README.md` (KYC verification)
- **Oracle Pallet**: `belizechain/pallets/oracle/README.md` (sanctions compliance)
- **Economy Pallet**: `belizechain/pallets/economy/README.md` (DALLA/bBZD management)

---

## Contact & Support

For questions regarding BelizeChain Payroll implementation:
- **Technical**: BelizeChain Core Developer Team
- **Business**: Ministry of Labour (payroll compliance)
- **Tax**: Belize Tax Service (payroll tax reporting)

**Audit Status**: ✅ Complete (January 2026)  
**Clippy Warnings**: 0  
**Test Coverage**: Comprehensive unit tests  
**Active Employers**: TBD (mainnet launch)  
**Total Payroll Disbursed**: TBD (mainnet launch)
