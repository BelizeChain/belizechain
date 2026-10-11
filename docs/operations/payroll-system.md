# Payroll System

BelizeChain's on-chain payroll system automates salary processing for both government and private sector employers, ensuring transparency, compliance with Belize labor laws, and seamless integration with the dual currency system (DALLA/bBZD).

## System Overview

### Key Features

- **Automated Processing**: Scheduled payments (weekly, biweekly, monthly)
- **Statutory Deductions**: Social Security, income tax, GST compliance
- **Multi-Currency**: Support for DALLA, bBZD, or hybrid payments
- **Audit Trail**: Complete on-chain transaction history
- **Digital Payslips**: Automatic PDF generation and distribution
- **Compliance Reporting**: FSC and tax authority integration

### Supported Employer Types

| Type | Requirements | Monthly Limit | Approval |
|------|-------------|---------------|----------|
| Government | Enhanced KYC + Treasury approval | Unlimited | Council majority (7 of 12 seats) |
| Private (Large) | Enhanced KYC + Tax ID | 500,000 DALLA | Automatic |
| Private (SME) | Verified KYC + Business license | 100,000 DALLA | 2-day review |
| NGO/Non-Profit | Verified KYC + Registration certificate | 50,000 DALLA | 5-day review |

## Employer Registration

### 1. Initial Setup

```javascript
// Employer registration is NOT self-service — there is no `payroll.registerEmployer`.
// An authorised verifier (T::VerifierOrigin) calls `verifyEmployer`, which creates
// the EmployerProfile. The rich company/tax/contact payload is off-chain onboarding
// data; the chain stores the account, its type, and its schedules.
await api.tx.payroll.verifyEmployer(
  employerAccount,
  'Enterprise'  // EmployerType: Government | Enterprise | SME | Cooperative | GigPlatform | NonProfit
).signAndSend(verifier);
```

### 2. KYC Requirements

**Enhanced KYC Checklist**:
- Certificate of Incorporation
- Business license (valid)
- Tax Identification Number (TIN)
- Social Security Employer ID
- Director IDs (BelizeID or passport)
- Proof of business address
- Bank account verification (for bBZD reconciliation)
- Audited financial statements (for large employers >500K/month)

**Processing Time**: 3-5 business days for Enhanced KYC approval.

### 3. Employee Onboarding

```javascript
// Add employees to payroll system
const employees = [
  {
    belizeId: 'BZ-ID-78901',
    fullName: 'Carlos Martinez',
    walletAddress: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    position: 'Production Manager',
    department: 'Operations',
    baseSalary: 4500, // bBZD per month
    paymentCurrency: 'bBZD',
    schedule: 'Biweekly',
    socialSecurityNumber: 'SSN-456789',
    taxBracket: 3, // Calculated automatically based on salary
    allowances: {
      housing: 500,
      transportation: 200,
      meals: 150
    },
    deductions: {
      loanRepayment: 300,
      healthInsurance: 80
    }
  }
];

// One call per employee — there is no `addEmployees` batch extrinsic.
// addEmployee(employee, salary, workerType, departmentId, metadataHash)
for (const e of employees) {
  await api.tx.payroll.addEmployee(
    e.account,
    e.salary,
    'FullTime',    // WorkerType: FullTime|PartTime|Contractor|Freelancer|Seasonal|Intern
    0,             // departmentId
    metadataHash   // [u8; 32] — blake2_256 of the off-chain record
  ).signAndSend(employerAccount);
}
```

## Payment Schedules

### Schedule Types

| Schedule | Pay Periods/Year | Common For | Processing Day |
|----------|------------------|------------|----------------|
| Weekly | 52 | Hourly workers, construction | Friday |
| Biweekly | 26 | Most private sector | Every other Friday |
| Monthly | 12 | Government, management | Last business day |

### Schedule Configuration

```rust
// Runtime pallet configuration
pub enum PaymentSchedule {
    Weekly,
    Biweekly,
    Monthly,
}

// Automated cron job triggers
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
        // Process payroll at scheduled intervals
        if Self::is_payday(n) {
            Self::process_scheduled_payroll();
        }
        Weight::from_parts(10_000_000, 0)
    }
}
```

## Statutory Deductions

### 1. Social Security Contributions

**Employee Contribution**: 8% of gross salary
**Employer Contribution**: 8% of gross salary

```javascript
// Calculation example
const grossSalary = 4500; // bBZD
const employeeSSContribution = grossSalary * 0.08; // 360 bBZD
const employerSSContribution = grossSalary * 0.08; // 360 bBZD

// Total to Social Security Board: 720 bBZD
```

**Salary Ceiling**: BZD 320/week (BZD 1,386.67/month) - contributions capped at this amount.

### 2. Income Tax (PAYE)

Belize uses a progressive tax system with annual brackets converted to monthly:

| Annual Income (BZD) | Monthly Income (BZD) | Tax Rate | Monthly Tax Calculation |
|---------------------|----------------------|----------|-------------------------|
| 0 - 26,000 | 0 - 2,166.67 | 0% | 0 |
| 26,001 - 27,000 | 2,166.68 - 2,250 | 25% on excess | (Income - 2,166.67) × 0.25 |
| 27,001+ | 2,250+ | 25% on next 1,000<br>30% on excess | 250 + (Income - 2,250) × 0.30 |

```javascript
// Tax calculation function
function calculateMonthlyTax(monthlyIncome) {
  if (monthlyIncome <= 2166.67) {
    return 0;
  } else if (monthlyIncome <= 2250) {
    return (monthlyIncome - 2166.67) * 0.25;
  } else {
    const tier1 = (2250 - 2166.67) * 0.25; // 20.83
    const tier2 = (monthlyIncome - 2250) * 0.30;
    return tier1 + tier2;
  }
}

// Example: 4,500 BZD/month salary
const tax = calculateMonthlyTax(4500);
// = 20.83 + (4500 - 2250) * 0.30
// = 20.83 + 675 = 695.83 bBZD
```

### 3. Goods and Services Tax (GST)

**Rate**: 12.5% on applicable goods and services

**Not applicable** to basic salary payments, but deducted on:
- Employer-provided goods (e.g., company vehicle, phone)
- Taxable allowances
- Non-cash benefits

```javascript
// GST calculation for taxable benefits
const taxableBenefits = {
  companyVehicle: 800, // Deemed benefit value
  phone: 50,
  internet: 40
};

const totalBenefits = Object.values(taxableBenefits).reduce((a, b) => a + b, 0);
const gstOnBenefits = totalBenefits * 0.125; // 111.25 bBZD
```

## Net Salary Calculation

### Complete Example

```javascript
// Employee: Carlos Martinez
const payrollData = {
  grossSalary: 4500,
  allowances: {
    housing: 500,
    transportation: 200,
    meals: 150
  },
  taxableIncome: 4500 + 500 + 200 + 150, // 5,350 bBZD
  
  // Statutory deductions
  socialSecurity: 4500 * 0.08, // 360 bBZD (capped if >1,386.67)
  incomeTax: calculateMonthlyTax(5350), // 950.83 bBZD
  
  // Voluntary deductions
  loanRepayment: 300,
  healthInsurance: 80,
  
  // Net calculation
  totalDeductions: 360 + 950.83 + 300 + 80, // 1,690.83
  netSalary: 5350 - 1690.83 // 3,659.17 bBZD
};

// On-chain transaction
// `executePayment` takes only the employee account — the amount and deductions
// already live on-chain (Employee.salary, EmployeeDeductions), set beforehand
// with `addEmployee` and `setDeduction`.
await api.tx.payroll.executePayment(employeeAccount).signAndSend(employerAccount);

// Deductions are configured separately, once per employee:
await api.tx.payroll.setDeduction(
  employeeAccount,
  'SocialSecurity',  // DeductionType: IncomeTax|SocialSecurity|Pension|HealthInsurance
  360
).signAndSend(employerAccount);
```

## Payslip Generation

### Automated PDF Generation

```javascript
// Payslip data structure
const payslip = {
  header: {
    employerName: 'Belize Cacao Company Ltd',
    employerAddress: '12 Hummingbird Highway, Belmopan',
    employerId: 'TIN-123456789',
    payPeriod: 'January 1-15, 2026',
    payDate: '2026-01-15',
    currency: 'bBZD'
  },
  employee: {
    name: 'Carlos Martinez',
    belizeId: 'BZ-ID-78901',
    ssn: 'SSN-456789',
    position: 'Production Manager',
    department: 'Operations'
  },
  earnings: {
    baseSalary: 4500.00,
    housingAllowance: 500.00,
    transportationAllowance: 200.00,
    mealsAllowance: 150.00,
    grossPay: 5350.00
  },
  deductions: {
    socialSecurity: 360.00,
    incomeTax: 950.83,
    loanRepayment: 300.00,
    healthInsurance: 80.00,
    totalDeductions: 1690.83
  },
  netPay: 3659.17,
  employerContributions: {
    socialSecurity: 360.00,
    pension: 225.00 // 5% employer contribution (optional)
  },
  paymentDetails: {
    walletAddress: '5GrwvaEF...NoHGKutQY',
    transactionHash: '0x1234...abcd',
    blockNumber: 1234567,
    timestamp: '2026-01-15 09:00:00 UTC'
  }
};

// There is no `api.rpc.payroll.*` namespace — the payroll pallet exposes no
// custom RPC. Payslips are rendered off-chain from chain state:
//   api.query.payroll.employees(account)         -> salary, workerType, dept
//   api.query.payroll.employeeDeductions(account)
//   api.query.payroll.payrollRecords(<recordId>) -> what was actually paid
// The rendered document can then be pinned to Pakit and its hash recorded in the
// employee's `metadataHash`.
```

### Distribution Methods

1. **Maya Wallet**: Automatic push notification with PDF link
2. **Email**: Sent to employee email (optional)
3. **Blue Hole Portal**: Accessible for government employees
4. **On-Chain Storage**: Pakit DAG hash for permanent retrieval

## Compliance & Reporting

### Monthly Reporting

```javascript
// There is no `api.rpc.payroll.generateMonthlyReport`, and no on-chain tax
// report. Aggregate from indexed chain data instead:
const records = await api.query.payroll.payrollRecords.entries();
const totalGross = records.reduce((sum, [, r]) => sum + r.amount, 0n);
```

// Report includes:
// - Total employees paid
// - Total gross salaries
// - Total income tax withheld
// - Total SS contributions (employee + employer)
// - GST on taxable benefits
// - Employee-by-employee breakdown
```

### Annual Reporting

**T4/T4A Equivalent** (Belize Statement of Remuneration):
- Total annual income
- Total deductions
- Net pay
- Employer contributions
- Filed with Income Tax Department by March 31

### Audit Trail

All payroll transactions are recorded on-chain with:
- **Immutability**: Cannot be altered after block finalization
- **Transparency**: Government auditors can verify payments
- **Privacy**: Employee details encrypted, only authorized parties see full data
- **Provenance**: Complete history of all salary adjustments

```rust
// On-chain storage
#[pallet::storage]
pub type PayrollHistory<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId, // Employee
    BoundedVec<PaymentRecord<T>, ConstU32<1000>>, // Last 1000 payments
    ValueQuery,
>;

pub struct PaymentRecord<T: Config> {
    pub employer: T::AccountId,
    pub payment_commitment: [u8; 32], // blake2_256(gross + net + employer + employee)
    pub deductions: BoundedVec<Deduction<T>, ConstU32<20>>,
    pub timestamp: T::BlockNumber,
    pub payslip_hash: [u8; 32], // Pakit DAG reference
    // NOTE: gross_amount and net_amount are no longer stored in plaintext.
    // Events emit commitment hashes; actual amounts are in the encrypted payslip.
}
```

## Advanced Features

### 1. Multi-Currency Payroll

Employers can split payments between DALLA and bBZD:

```javascript
// Hybrid payment: 60% bBZD (stable), 40% DALLA (growth potential)
const hybridPayment = {
  employeeId: 'BZ-ID-78901',
  netSalary: 3659.17,
  distribution: {
    bBZD: 3659.17 * 0.60, // 2,195.50 bBZD (stable living expenses)
    DALLA: 3659.17 * 0.40 // 1,463.67 DALLA (savings/staking)
  }
};

// Staking has no `stakeAndEarn` and no fixed APY constant — rewards come from
// the per-epoch pool (`staking.epochRewards`) and vary with PoUW performance.
await api.tx.staking.joinValidators(
  1463n * 10n ** 12n,  // stake
  100,                 // computeCapacity
  'Belize City'        // location
);
```

### 2. Tourism Sector Bonuses

Employees in tourism-verified businesses receive cashback benefits:

```javascript
// Tourism employee at hotel
const tourismEmployee = {
  employer: 'Paradise Resort Ltd',
  category: 'Hotels',
  cashbackEligible: true,
  monthlyCashback: netSalary * 0.08 // 8% cashback on DALLA spending
};
```

### 3. Government Payroll Coordination

Government departments use council-majority approval for batch payments:

```javascript
// Ministry of Health batch payroll
// `batchPayment()` takes no arguments: it pays every active employee of the
// calling employer in one extrinsic. Approval is enforced by the caller's
// origin (the treasury account must be the one authorised to spend).
await api.tx.payroll.batchPayment().signAndSend(treasuryAccount);
```

## Troubleshooting

### Common Issues

**1. Payment Failed - Insufficient Balance**
```javascript
// Solution: Top up employer treasury account
await api.tx.balances.transfer(employerTreasuryAccount, requiredAmount);
```

**2. Employee Not Registered**
```javascript
// Solution: Verify the BelizeID and wallet address
const identityId = await api.query.identity.identityOf(belizeId);
if (identityId.isNone) {
  // Employee must be KYC'd first
  console.error('Employee must register BelizeID');
}
```

**3. Tax Calculation Mismatch**
```javascript
// Solution: Ensure all allowances marked as taxable/non-taxable
const taxableAllowances = ['housing', 'transportation'];
const nonTaxableAllowances = ['uniform', 'safety-equipment'];
```

**4. Payslip PDF Generation Failed**
```javascript
// Solution: Check Pakit storage connectivity
const pakitStatus = await api.rpc.pakit.healthCheck();
if (!pakitStatus.healthy) {
  // Retry after Pakit node reconnects
}
```

## Integration Examples

### Maya Wallet - Employee View

```typescript
// React component for viewing payslips
import { usePolkadot } from '@/hooks/usePolkadot';

export function PayslipHistory() {
  const { api, account } = usePolkadot();
  const [payslips, setPayslips] = useState([]);

  useEffect(() => {
    async function fetchPayslips() {
      // Records are keyed by a u64 record id, not by account. Filter by owner.
      const records = await api.query.payroll.payrollRecords.entries();
      const mine = records
        .filter(([, r]) => r.employee.toString() === account.address)
        .map(([k, r]) => ({ id: k.args[0].toNumber(), ...r }));
      setPayslips(mine);
    }
    fetchPayslips();
  }, [account]);

  return (
    <div className="payslip-list">
      {payslips.map(payment => (
        <PayslipCard 
          key={payment.timestamp}
          paymentCommitment={payment.payment_commitment}
          deductions={payment.deductions}
          downloadHash={payment.payslip_hash}
          // NOTE: Decrypt full payslip from Pakit DAG for gross/net details
        />
      ))}
    </div>
  );
}
```

### Blue Hole Portal - Employer Dashboard

```typescript
// Government payroll dashboard
export function GovernmentPayrollDashboard() {
  const departments = [
    'Ministry of Health',
    'Ministry of Education',
    'Ministry of Agriculture',
    'Police Department',
    'Fire Services'
  ];

  return (
    <div className="grid grid-cols-2 gap-6">
      {departments.map(dept => (
        <DepartmentPayrollCard
          department={dept}
          employees={departmentData[dept].count}
          monthlyBudget={departmentData[dept].budget}
          pendingApprovals={departmentData[dept].pending}
        />
      ))}
    </div>
  );
}
```

## Security Considerations

1. **Access Control**: Only authorized signatories can approve payments
2. **Rate Limiting**: Maximum 1,000 employees processed per transaction
3. **Council-majority approval**: Government payrolls require a council majority (7 of 12 seats)
4. **Audit Logging**: All changes logged with timestamps and approver identity
5. **Data Privacy**: Employee SSNs encrypted at rest, only visible to authorized parties
6. **Compliance**: Automatic FSC reporting for large employers (>100 employees)

## API Reference

See [Payroll Pallet API](../developer-guides/pallet-apis-services.md#payroll-pallet) for complete RPC method documentation.
