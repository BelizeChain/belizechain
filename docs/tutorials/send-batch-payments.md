# 💸 Send Batch Payments

**Pay multiple people at once and save 90% on fees!**

---

## 📋 What You'll Learn

After this tutorial, you will:

- ✅ Understand batch payments
- ✅ Create a recipient list
- ✅ Import recipients from CSV
- ✅ Preview and send batch payment
- ✅ Track all payment statuses
- ✅ Export payment records

**Perfect for**: Payroll, vendor payments, allowances, dividends

---

## ⏱️ Tutorial Info

| Info | Details |
|------|---------|
| **Time Required** | 20 minutes |
| **Difficulty** | ⭐ Easy |
| **Prerequisites** | Maya Wallet installed, sufficient balance |
| **Cost Example** | 50 payments = $0.05 (vs $0.50 individual) |
| **Savings** | 90% on transaction fees! |

---

## 🤔 Why Batch Payments?

### Individual Payments (Old Way)
```
Payment 1: $100.00 + $0.01 fee = $100.01
Payment 2: $150.00 + $0.01 fee = $150.01
Payment 3: $200.00 + $0.01 fee = $200.01
...
Payment 50: $125.00 + $0.01 fee = $125.01

Total fees: 50 × $0.01 = $0.50
Time: 50 clicks × 30 seconds = 25 minutes
```

### Batch Payment (New Way)
```
All 50 payments in ONE transaction

Total fees: 1 × $0.05 = $0.05
Time: 1 click = 10 seconds

SAVINGS: $0.45 + 24 minutes! 🎉
```

---

## 📚 Prerequisites

Before starting:

### 1. Have Maya Wallet Installed
- iOS: App Store
- Android: Google Play
- Desktop: maya.belizechain.org

**Need help?** See [Create Wallet](../getting-started/create-wallet.md)

### 2. Have Sufficient Balance
- Enough to cover all payments
- Plus 0.05 bBZD for batch fee

**Example**: Paying $5,000 total → Need $5,000.05 in wallet

### 3. Know Recipient Addresses
- Valid BelizeChain addresses
- Start with "5"
- Exactly 48 characters

**Tip**: Ask recipients to send you their addresses first

---

## 📝 Step 1: Prepare Recipient List

### Option A: Manual Entry (for < 10 people)

1. Open Maya Wallet
2. Tap **"Send"** → **"Batch Payment"**
3. Tap **"+ Add Recipient"**
4. Enter details:
   - **Name**: John Smith (for your reference)
   - **Address**: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
   - **Amount**: 1,200.00 bBZD
   - **Memo** (optional): Salary - January 2025
5. Tap **"Add"**
6. Repeat for each person

### Option B: CSV Import (for 10+ people) ⭐ Recommended

**Step 1: Download Template**

1. In Maya Wallet: **Batch Payment** → **"Import CSV"**
2. Tap **"Download Template"**
3. Save to your device

**Step 2: Fill Template**

Open the CSV file (Excel, Google Sheets, etc.):

```csv
name,address,amount,memo
John Smith,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty,1200.00,Salary - January
Maria Garcia,5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,1500.00,Salary - January
Carlos Martinez,5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL,1800.00,Salary - January
Ana Lopez,5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw,1350.00,Salary - January
```

**CSV Format Rules**:
- **name**: Any text (for your reference only)
- **address**: Valid BelizeChain address (48 characters)
- **amount**: Number with up to 2 decimals (e.g., 1200.00)
- **memo**: Optional message (max 64 characters)

**Step 3: Import CSV**

1. Save your CSV file
2. In Maya Wallet: **"Import CSV"** → **"Choose File"**
3. Select your CSV
4. Wait for validation

**Success message**: "✅ 50 recipients loaded"

---

## 🔍 Step 2: Review Recipients

After adding/importing, you'll see a list:

```
╔════════════════════════════════════════════╗
║  BATCH PAYMENT SUMMARY                     ║
╠════════════════════════════════════════════╣
║  Recipients: 50                            ║
║  Total Amount: 65,250.00 bBZD              ║
║  Batch Fee: 0.05 bBZD                      ║
║  ─────────────────────────────────────────║
║  Total Cost: 65,250.05 bBZD                ║
║                                            ║
║  Your Balance: 70,000.00 bBZD ✅           ║
╚════════════════════════════════════════════╝

Recipients:
1. John Smith
   5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
   1,200.00 bBZD
   "Salary - January"

2. Maria Garcia
   5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
   1,500.00 bBZD
   "Salary - January"

... (48 more)

[Edit] [Send Batch] [Cancel]
```

### Check Each Line
- ✅ Name correct
- ✅ Address starts with "5" (48 characters)
- ✅ Amount correct (no typos!)
- ✅ Memo makes sense

**⚠️ WARNING**: Double-check addresses! Blockchain transactions are irreversible!

### Edit if Needed
- Tap any recipient to edit
- Tap **"−"** to remove
- Tap **"+ Add More"** to include others

---

## ✅ Step 3: Verify and Send

### 1. Final Check

**Before sending, verify**:
- [ ] All addresses are correct
- [ ] All amounts are correct
- [ ] Total matches your records
- [ ] You have sufficient balance
- [ ] Memo explains the payment

### 2. Initiate Batch

Tap **"Send Batch"**

### 3. Review Transaction Details

```
╔════════════════════════════════════════════╗
║  CONFIRM BATCH PAYMENT                     ║
╠════════════════════════════════════════════╣
║  From: My Business Account                 ║
║  Recipients: 50 people                     ║
║  Total Amount: 65,250.00 bBZD              ║
║  Batch Fee: 0.05 bBZD                      ║
║  Total: 65,250.05 bBZD                     ║
║                                            ║
║  This transaction cannot be reversed!      ║
╚════════════════════════════════════════════╝

[Cancel] [Confirm]
```

### 4. Enter PIN

Enter your 6-digit PIN to authorize

### 5. Wait for Confirmation

```
Processing batch payment...
⏳ 10-15 seconds

✅ Batch payment sent!
Transaction ID: 0x7d4e...f3a1
```

**Time**: About 10-15 seconds for the blockchain to confirm

---

## 📊 Step 4: Track Payment Status

### View Batch Details

After sending, tap **"View Details"**

```
╔════════════════════════════════════════════╗
║  BATCH PAYMENT #12345                      ║
║  January 15, 2025 at 10:45 AM             ║
╠════════════════════════════════════════════╣
║  Status: ✅ Complete                        ║
║  Recipients: 50                            ║
║  Successful: 50                            ║
║  Failed: 0                                 ║
║  Total Paid: 65,250.00 bBZD                ║
╚════════════════════════════════════════════╝

Individual Payments:
✅ John Smith - 1,200.00 bBZD
✅ Maria Garcia - 1,500.00 bBZD
✅ Carlos Martinez - 1,800.00 bBZD
✅ Ana Lopez - 1,350.00 bBZD
... (46 more)

[Export Report] [Share] [Close]
```

### Payment Statuses

| Icon | Status | Meaning |
|------|--------|---------|
| ✅ | Complete | Payment delivered successfully |
| ⏳ | Pending | Still processing (wait 1 min) |
| ❌ | Failed | Payment failed (balance refunded) |

### If Payment Failed

**Common reasons**:
1. **Invalid address** - Address doesn't exist or wrong format
2. **Recipient wallet issue** - Rare, but possible

**What happens**:
- Failed amounts are automatically refunded
- Other payments still go through
- You're notified of failures

**To fix**:
1. Verify the failed address with recipient
2. Create new batch with just failed payments
3. Send again

---

## 📄 Step 5: Export Payment Records

For your accounting/records:

### Export Options

**Option 1: CSV**
1. Tap **"Export Report"**
2. Select **"CSV"**
3. Choose where to save
4. Opens in Excel, Google Sheets, etc.

**Option 2: PDF**
1. Tap **"Export Report"**
2. Select **"PDF"**
3. Professional format with:
   - Your business info
   - Payment date/time
   - All recipients
   - Totals
   - Transaction ID

**Use for**: Bookkeeping, tax records, audits

---

## 💡 Real-World Example

### Scenario: Tourism Restaurant Payroll

**Carlos runs "Barrier Reef Grill" in San Pedro**

**Problem**: Every Friday, pays 25 staff members by cash
- Time: 2+ hours (count cash, hand out, get signatures)
- Risk: Carrying $15,000 cash from bank
- Fees: $5 per employee to cash checks = $125/month

**Solution**: Batch payments with BelizeChain

**Process**:
1. **Monday**: Send staff an email: "Get Maya Wallet + send me your address"
2. **Tuesday**: Create CSV with all addresses
3. **Friday 12:00 PM**: One batch payment in 10 seconds!
4. **Friday 12:01 PM**: All 25 employees have money in Maya Wallet

**Results**:
- **Time saved**: 2 hours → 10 seconds (99% reduction!)
- **Fees saved**: $125 → $0.05 (99.96% reduction!)
- **Safety**: No cash handling
- **Convenience**: Staff can spend immediately with Maya Wallet

**Annual savings**: 
- Time: 104 hours
- Money: $1,500

---

## 🎯 Common Use Cases

### 1. Employee Payroll
- **Frequency**: Weekly/bi-weekly/monthly
- **Recipients**: 5-500 employees
- **Benefit**: Instant payment, no bank delays

### 2. Vendor Payments
- **Frequency**: As needed
- **Recipients**: Multiple suppliers
- **Benefit**: Clear memo for invoice tracking

### 3. Family Allowances
- **Frequency**: Weekly/monthly
- **Recipients**: Children, relatives
- **Benefit**: Set and forget with recurring

### 4. Dividends/Distributions
- **Frequency**: Quarterly/annually
- **Recipients**: Partners, investors
- **Benefit**: Transparent, verifiable on blockchain

### 5. Refunds
- **Frequency**: As needed
- **Recipients**: Customers
- **Benefit**: Fast refund processing

### 6. Scholarships/Grants
- **Frequency**: Semester/annually
- **Recipients**: Students, grantees
- **Benefit**: Government can track fund usage

---

## ⚠️ Important Safety Tips

### DO ✅

- ✅ **Double-check addresses** - Copy from recipient directly
- ✅ **Verify amounts** - Use calculator to sum totals
- ✅ **Test first** - Send $1 to each new recipient first time
- ✅ **Keep records** - Export reports for accounting
- ✅ **Use memos** - Helps with bookkeeping
- ✅ **Backup CSV** - Save your recipient list for next time

### DON'T ❌

- ❌ **Don't rush** - Take time to verify each line
- ❌ **Don't type addresses** - Always copy/paste (too easy to make mistake)
- ❌ **Don't send to unverified** - Ask recipient to confirm address first
- ❌ **Don't forget to check balance** - Ensure you have enough + fees
- ❌ **Don't batch without testing** - Send individual payment first time

---

## 🔧 Troubleshooting

### "Insufficient Balance"

**Problem**: Not enough funds

**Solution**:
1. Check total needed: Amount + 0.05 fee
2. Add funds to wallet
3. Try again

**Math example**:
- Sending: 10,000.00 bBZD
- Fee: 0.05 bBZD
- Need: 10,000.05 bBZD
- Have: 10,000.00 bBZD ❌
- **Add at least**: 0.05 bBZD

---

### "Invalid CSV Format"

**Problem**: CSV file not formatted correctly

**Solution**:
1. Re-download template
2. Don't change column headers (name, address, amount, memo)
3. Ensure addresses are 48 characters
4. Amounts are numbers (no commas, dollar signs)
5. Save as CSV (not XLSX)

**Correct CSV**:
```csv
name,address,amount,memo
John,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty,100.00,Payment
```

**Wrong CSV**:
```csv
Name,Wallet Address,$ Amount,Note  ❌ (wrong headers)
John,5FHn...694ty,$100.00,Payment  ❌ (shortened address, $ sign)
```

---

### "Invalid Address" Error

**Problem**: One or more addresses are wrong

**Solution**:
1. Maya Wallet shows which line has error
2. Check address:
   - Starts with "5"?
   - Exactly 48 characters?
   - No spaces or special characters?
3. Ask recipient to send address again
4. Fix and re-import

**Valid address**: `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty` ✅  
**Invalid address**: `5FHneW46xGXgs5` ❌ (too short)

---

### "Batch Too Large"

**Problem**: Too many recipients in one batch

**Limit**: 500 recipients per batch

**Solution**:
1. Split into multiple batches:
   - Recipients 1-500 (Batch #1)
   - Recipients 501-1000 (Batch #2)
2. Send separately (10 seconds each)

**Still saves fees!**
- Individual: 1,000 × $0.01 = $10.00
- Batched (2 batches): 2 × $0.05 = $0.10
- Savings: $9.90

---

### Some Payments Failed

**Problem**: Batch sent, but some marked ❌ Failed

**Why**:
- Address doesn't exist
- Address typo
- Network issue (rare)

**What happens**:
- ✅ Successful payments: Complete
- ❌ Failed payments: **Automatically refunded**

**To fix**:
1. Contact failed recipients
2. Verify correct addresses
3. Create new batch with only failed ones
4. Send again

---

## 📈 Advanced Tips

### 1. Save Recipient Lists

**Create templates for common payments**:

- `payroll_2025.csv` - Monthly employee list
- `vendors_weekly.csv` - Regular supplier payments
- `family_allowances.csv` - Weekly family payments

**Benefit**: Just update amounts each time, addresses stay the same

---

### 2. Schedule Batch Payments

**Coming soon**: Recurring batch payments!

**Example**:
- Set up once: "Pay these 50 employees $X every Friday"
- Maya Wallet automatically sends
- You get notification
- Review and export records

---

### 3. Batch Payment Categories

**Organize by type**:
- Payroll → Use "Salary - [Month]" in memo
- Vendors → Use "Invoice #[number]" in memo
- Refunds → Use "Refund - Order #[number]" in memo

**Benefit**: Easy to search and filter later

---

### 4. Multi-Currency Batches

**Send different currencies in same batch**:
- 30 employees: DALLA
- 20 contractors: bBZD

**How**:
1. Add column to CSV: `currency`
2. Specify for each recipient

```csv
name,address,amount,currency,memo
John,5FHne...,1200,bBZD,Salary
Maria,5Grwv...,5000,DALLA,Contractor
```

---

## 📚 Next Steps

### Related Tutorials

**After mastering batch payments, try**:

1. **[Setup Recurring Payments](./setup-recurring-payment.md)**
   - Automate batch payments
   - Set it and forget it
   - Never miss payroll again

2. **[Accept Crypto Payments](./accept-payments.md)**
   - For businesses
   - Collect payments from customers
   - Use batch payments for refunds

3. **[Complete KYC Verification](./kyc-verification.md)**
   - Required for business accounts
   - Higher limits
   - Compliance for large batches

---

### Practice Exercise

**Try this on testnet first** (free test tokens):

1. Get testnet Maya Wallet: https://testnet.maya.belizechain.org
2. Get free test tokens: https://faucet.belizechain.org
3. Create CSV with 5 test addresses
4. Send batch payment
5. Verify all succeeded
6. Export report

**Once comfortable**: Switch to mainnet for real payments!

---

## 🆘 Need Help?

### During Tutorial
- **Video guide**: https://youtube.com/watch?v=batch-payments
- **Live demo**: Try interactive demo at demo.belizechain.org
- **Forum**: forum.belizechain.org/tutorials/batch-payments

### Stuck or Have Questions?
- **Live Chat**: In Maya Wallet - Menu → Help
- **Phone**: +501-CHAIN (24246) - 24/7
- **Email**: support@belizechain.org
- **Discord**: #batch-payments channel

---

## ✅ Tutorial Complete!

**You now know how to**:
- ✅ Create batch payments
- ✅ Import CSV files
- ✅ Send to multiple recipients
- ✅ Track payment status
- ✅ Export records
- ✅ Save 90% on fees!

**Share your success**: Post on social media #BelizeChainBatch

---

**Time saved**: 99% reduction in payment processing time  
**Money saved**: 90% reduction in transaction fees  
**Stress saved**: Priceless! 😊

**Ready for next tutorial?** → [Setup Recurring Payments](./setup-recurring-payment.md)
