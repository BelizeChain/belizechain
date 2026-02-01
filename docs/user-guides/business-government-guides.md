# Business & Government User Guides

**Merchant Registration • Payment Acceptance • Payroll • Blue Hole Portal**

Comprehensive guides for businesses and government operations.

---

## Part 1: Merchant Registration

### Prerequisites

- Maya Wallet with **Verified KYC** (minimum)
- Business license (upload to Pakit storage)
- Tax ID (SSN for sole proprietors, EIN for corporations)

### Registration Steps

```
1. Maya Wallet → Business → Register as Merchant
2. Select category:
   - Hotel
   - Restaurant
   - Tour Operator
   - Crafts/Retail
   - Other
3. Upload documents:
   - Business license (PDF)
   - Tax registration certificate
   - Proof of address (business location)
4. Submit application (500 DALLA fee)
5. Wait 3-5 business days for FSC approval
6. ✅ Merchant status approved
```

**Benefits:**
- Accept DALLA/bBZD payments (0.01 DALLA fee vs 3% credit card)
- Eligible for tourism cashback program (customers earn 5-8%, you attract more business)
- BelizeX DEX listing (optional, for selling products/services)
- Payroll integration (if you have employees)

---

## Part 2: Payment Acceptance

### Point of Sale (POS) Setup

#### Option A: QR Code (Free)

```
1. Maya Wallet → Business → Generate QR Code
2. Enter amount (e.g., 25 DALLA for $7.50 meal)
3. Customer scans QR
4. Payment confirmed in 6 seconds
5. ✅ Funds in your wallet
```

**Best for:** Small businesses, market vendors, food trucks

#### Option B: BelizeChain POS Terminal (Hardware)

```
Purchase BelizeChain POS device ($299 one-time):
- NFC card reader
- QR scanner
- 7-inch touchscreen
- Receipt printer

Setup:
1. Connect to WiFi
2. Login with merchant account
3. Accept payments via:
   - QR code scan
   - NFC card tap
   - Manual address entry
```

**Best for:** Restaurants, hotels, retail stores

#### Option C: Web Integration (API)

```javascript
// Install BelizeChain SDK
npm install @belizechain/sdk

// Create checkout page
const { BelizeChainSDK } = require('@belizechain/sdk');

const sdk = new BelizeChainSDK({
  network: 'mainnet',
  merchantAccount: '5GrwvaEF...'
});

// Generate payment request
app.post('/checkout', async (req, res) => {
  const { amount, orderId } = req.body;
  
  const payment = await sdk.payments.create({
    amount: amount * 1e12,  // Convert to DALLA (12 decimals)
    currency: 'DALLA',
    orderId: orderId,
    redirectUrl: 'https://yourstore.com/success'
  });
  
  res.json({
    qrCode: payment.qrCodeData,
    address: payment.address,
    expiresAt: payment.expiresAt
  });
});

// Listen for payment confirmation
sdk.payments.onConfirmed(orderId, (payment) => {
  console.log(`✅ Payment received: ${payment.amount} DALLA`);
  // Fulfill order
});
```

**Best for:** E-commerce websites, online booking systems

---

## Part 3: Payroll Management

### Register as Employer

```
1. Blue Hole Portal → Payroll → Register Employer
2. Select type:
   - Government entity
   - Private sector
3. Enter tax ID
4. Upload business license
5. Submit application (1,000 DALLA fee for private, free for government)
6. ✅ Employer registered
```

### Add Employees

```
1. Blue Hole Portal → Payroll → Employees → Add Employee
2. Enter employee details:
   - Full name
   - BelizeID or SSN
   - Maya Wallet address
   - Pay schedule: Weekly / Biweekly / Monthly
   - Gross salary (e.g., 5,000 bBZD/month)
3. Auto-calculated deductions:
   - Social Security: 8% employee + 8% employer = 800 bBZD
   - Income tax: Progressive (0-25%) = ~625 bBZD (for 5K salary)
   - GST withholding: 12.5% (if applicable)
   - Net pay: ~3,575 bBZD
4. Save employee
```

### Process Payroll (Monthly Example)

```
1. Blue Hole Portal → Payroll → Process Payroll
2. Select pay period: January 1-31, 2026
3. Review employee list (15 employees)

Employee Summary:
┌─────────────────┬──────────┬──────────┬─────────┬──────────┐
│ Employee        │ Gross    │ SS       │ Tax     │ Net Pay  │
├─────────────────┼──────────┼──────────┼─────────┼──────────┤
│ John Doe        │ 5,000    │ 400      │ 625     │ 3,975    │
│ Jane Smith      │ 3,500    │ 280      │ 287     │ 2,933    │
│ ... (13 more)   │ ...      │ ...      │ ...     │ ...      │
├─────────────────┼──────────┼──────────┼─────────┼──────────┤
│ **Total**       │ 62,500   │ 5,000    │ 7,812   │ 49,688   │
│ Employer SS     │          │ +5,000   │         │          │
│ **Grand Total** │ **62,500**│ **10,000**│ **7,812**│ **49,688**│
└─────────────────┴──────────┴──────────┴─────────┴──────────┘

3. Confirm payroll (requires Enhanced KYC for >10K total)
4. Funds automatically transferred:
   - Employees: 49,688 bBZD (direct to Maya Wallets)
   - Social Security: 10,000 bBZD (to government account)
   - Tax Board: 7,812 bBZD (withholding tax)
5. ✅ Payroll processed, all employees notified
```

**Transaction fee:** 0.01 DALLA per employee (0.15 DALLA for 15 employees ≈ $0.045)

---

## Part 4: BNS Domain Setup (.bz Domains)

### Register Domain

```
1. Maya Wallet → BNS → Register Domain
2. Search for domain (e.g., "myhotel.bz")
3. Check availability
4. Select tier:
   - Standard: 100 DALLA/year (most businesses)
   - Premium: 1,000 DALLA/year (high-value domains like "hotel.bz")
   - Verified: 500 DALLA/year (for verified businesses, includes checkmark)
5. Pay registration fee
6. ✅ Domain registered (instant)
```

### Host Website on Pakit

```
1. Upload website files to Pakit:
   pakit upload ./website/ --domain myhotel.bz

2. Pakit returns content hash:
   QmXyZ123... (DAG root)

3. Update domain content:
   Maya Wallet → BNS → My Domains → myhotel.bz → Update Content
   - Enter Pakit hash: QmXyZ123...
   - ✅ Domain now points to website

4. Access website:
   https://myhotel.bz (via BelizeChain gateway)
   ipfs://QmXyZ123... (direct Pakit access)
```

**Hosting cost:** Free (already paid in domain fee)

---

## Part 5: Blue Hole Portal (Government Dashboard)

### Access Portal

**URL:** https://blueholeportal.belizechain.org

**Login:**
1. Enter government official wallet address
2. Sign message with Maya Wallet
3. ✅ Logged in (role-based access)

### Dashboard Overview

**For District Council Members:**
```
Blue Hole Portal → Dashboard

Key Metrics:
- District population: 95,000 (Belize District)
- Active DALLA wallets: 42,350 (44.6%)
- Monthly transaction volume: 8.2M DALLA
- Tourism cashback claimed: 125,000 bBZD
- District treasury: 2.5M DALLA

Actions:
- Approve treasury proposals
- View KYC applications (FSC officers only)
- Review compliance reports
- Monitor validator performance
```

**For FSC Officers:**
```
Blue Hole Portal → Compliance

Pending KYC Applications: 127
Flagged Accounts: 5 (AML review)
Monthly Approvals: 1,832 (Basic: 1,676, Verified: 156)

Actions:
- Approve/reject KYC applications
- Freeze/unfreeze accounts
- Generate compliance reports
- Export transaction data (for audits)
```

**For Central Bank Officials:**
```
Blue Hole Portal → Treasury

bBZD Supply: 45.2M (backed by 45.2M BZD in reserves)
DALLA Supply: 102.5M (inflation: 3.2% annual)
Monthly Treasury Income: 118.6K DALLA
  - TX fees: 80% = 94.9K
  - BNS fees: 5% = 5.9K
  - Inflation: 25% = 29.6K
  - Slashing: ~2K

Actions:
- Approve bBZD mint/burn requests (4-of-7 multi-sig)
- Review treasury proposals
- Monitor reserve ratios
- Generate financial reports
```

---

## Part 6: Compliance Requirements

### For Merchants

**Monthly:**
- Report sales tax (12.5% GST) if revenue >$75,000 BZD/year
- Submit AML report if transactions >$50,000 BZD/month

**Annually:**
- Renew business license (via Blue Hole Portal)
- File income tax return
- Audit if revenue >$500,000 BZD/year

### For Employers

**Monthly:**
- Process payroll by 1st of month
- Remit Social Security contributions (16% total: 8% employee + 8% employer)
- Remit withholding tax (progressive 0-25%)

**Quarterly:**
- Submit payroll report to Tax Board
- GST filing (if applicable)

**Annually:**
- Employee W-2 equivalents (via Blue Hole Portal, auto-generated)
- Reconcile tax withholding

---

## Troubleshooting

### Payment Not Received

**Check:**
1. Blue Hole Portal → Transactions → Search by transaction ID
2. Verify on block explorer: https://explorer.belizechain.org
3. If confirmed but not in wallet: Contact support with transaction hash

### Payroll Failed

**Common causes:**
- Insufficient funds (check treasury balance)
- Employee wallet address incorrect (verify with employee)
- Enhanced KYC not approved (required for >10K/month payroll)

**Solution:**
1. Blue Hole Portal → Payroll → Failed Transactions
2. Review error message
3. Fix issue and retry

### Domain Not Loading

**Check:**
1. BNS → My Domains → myhotel.bz → View Content Hash
2. Verify Pakit hash is correct
3. If hash correct, wait 5 minutes for propagation
4. If still not loading: Re-upload to Pakit and update domain

---

## Related Documentation

- [Economy Pallet API](../developer-guides/pallet-apis-core.md#economy-pallet)
- [Payroll Pallet API](../developer-guides/pallet-apis-services.md#payroll-pallet)
- [BNS Documentation](../services/bns-overview.md)
- [KYC/AML Procedures](../security/kyc-aml-procedures.md)
- [Maya Wallet Guide](./maya-wallet-guide.md)
