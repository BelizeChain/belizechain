# 🚀 Quick Start Testing Guide

**Ready to Test!** Your blockchain and UI are running successfully.

---

## 📋 Pre-Configured Test Accounts

The development blockchain has these pre-funded accounts (each with **1,000,000 DALLA**):

### Test Accounts Available:
| Account | Seed Phrase | Balance |
|---------|-------------|---------|
| **Alice** | `//Alice` | 1,000,000 DALLA |
| **Bob** | `//Bob` | 1,000,000 DALLA |
| **Charlie** | `//Charlie` | 1,000,000 DALLA (testnet only) |
| **Dave** | `//Dave` | 1,000,000 DALLA (testnet only) |

**Note:** Since you're running `--dev` mode, only Alice and Bob are pre-funded.

---

## 🔧 Step-by-Step: Start Testing NOW

### **Step 1: Install Polkadot.js Extension** (2 minutes)
1. **Chrome/Brave:** https://chrome.google.com/webstore/detail/polkadot%7Bjs%7D-extension/mopnmbcafieddcagagdcbnhejhlodfdd
2. **Firefox:** https://addons.mozilla.org/en-US/firefox/addon/polkadot-js-extension/

### **Step 2: Import Test Accounts** (3 minutes)
1. Open Polkadot.js extension
2. Click **"+"** → **"Import account from pre-existing seed"**
3. **For Alice:**
   - Seed: `//Alice`
   - Name: `Alice (Test)`
   - Click **"Add the account with the generated seed"**
4. **Repeat for Bob:**
   - Seed: `//Bob`
   - Name: `Bob (Test)`

### **Step 3: Connect to Blockchain** (1 minute)
1. Open: http://localhost:3001
2. Click **"Connect Wallet"** (or wallet icon)
3. Select **"Alice (Test)"**
4. Click **"Connect"**
5. ✅ You should see your balance: **1,000,000 DALLA**

### **Step 4: Quick Page Tour** (5 minutes)
Click through these pages to verify they load:

✅ **Home** → See balance cards  
✅ **Staking** → View validators (Alice should be listed)  
✅ **Governance** → View proposals  
✅ **Trade** → View liquidity pools  
✅ **BelizeID** → View identity (may be empty initially)  

### **Step 5: Your First Transaction** (5 minutes)

#### **Option A: Stake DALLA** (Recommended)
1. Go to: http://localhost:3001/staking
2. Find **Alice** in the validator list
3. Click **"Stake with this Validator"**
4. Enter amount: **100**
5. Click **"Stake"**
6. **Sign transaction** in Polkadot.js popup
7. ✅ Wait ~6 seconds for block confirmation
8. Refresh page → See your staked balance

#### **Option B: Swap Tokens**
1. Go to: http://localhost:3001/trade
2. Select: **DALLA → bBZD**
3. Enter amount: **50**
4. Click **"Get Quote"**
5. Click **"Execute Swap"**
6. **Sign transaction** in Polkadot.js popup
7. ✅ See updated balances

---

## 🧪 What to Test Next

### **Critical Tests** (15 minutes)
- [ ] Send DALLA to Bob (use /send page)
- [ ] Vote on a governance proposal
- [ ] Check transaction history
- [ ] Test wallet switching (Alice ↔ Bob)
- [ ] Verify auto-refresh (wait 30 seconds on any page)

### **Advanced Tests** (30 minutes)
- [ ] Register .bz domain (BNS page)
- [ ] List domain for sale
- [ ] Create governance proposal
- [ ] Bridge transfer (may need testnet tokens)
- [ ] Test all error states (disconnect wallet, etc.)

### **Visual Tests**
- [ ] Test mobile view (resize browser)
- [ ] Test dark mode (should be default)
- [ ] Test animations and transitions
- [ ] Test bottom navigation on mobile

---

## 📊 Expected Results

### **Balance Display**
- **Initial:** 1,000,000 DALLA
- **After 100 stake:** 999,900 DALLA (minus tiny gas fee)
- **Staked:** 100 DALLA

### **Blockchain Status**
- **Block height:** Increasing every ~6 seconds
- **Finalized blocks:** 2-3 blocks behind current
- **RPC:** ws://127.0.0.1:9944

### **Page Load Times**
- **First load:** 1-3 seconds
- **Subsequent:** < 1 second
- **Auto-refresh:** Every 30 seconds

---

## 🐛 Troubleshooting

### **"Wallet not connected" error**
→ Make sure Polkadot.js extension is installed and account imported

### **"Insufficient balance" error**
→ You may need gas fees (keep at least 1 DALLA for fees)

### **"Transaction failed" error**
→ Check browser console for details (F12)

### **Page loading forever**
→ Verify blockchain is running: `ps aux | grep belizechain-node`

### **Data not showing**
→ Wait for auto-refresh (30s) or manually refresh browser

---

## 🎯 Success Criteria

Your testing is successful if:
- ✅ Wallet connects and shows correct balance
- ✅ At least 3 pages load data from blockchain
- ✅ At least 1 transaction completes successfully  
- ✅ Auto-refresh works (data updates every 30s)
- ✅ No critical console errors

---

## 📝 Report Template

After testing, please note:

**Browser:** _________________  
**Test Date:** _________________  
**Blockchain Block #:** _________________

### ✅ Working Features:
1. _________________
2. _________________
3. _________________

### ❌ Issues Found:
1. _________________
2. _________________

### 💡 Suggestions:
1. _________________
2. _________________

---

## 🚨 Need Help?

**Blockchain not running?**
```bash
cd /home/wicked/belizechain-belizechain
./target/release/belizechain-node --dev --tmp
```

**UI not running?**
```bash
cd /home/wicked/belizechain-belizechain/ui/maya-wallet
npm run dev
```

**Check services:**
```bash
# Blockchain
curl -s http://127.0.0.1:9944 && echo "✅ Blockchain running"

# UI  
curl -s http://localhost:3001 && echo "✅ UI running"
```

**View logs:**
```bash
# Blockchain logs - check the terminal running the node

# UI logs - check browser console (F12)
```

---

## 📚 Next Steps After Manual Testing

1. **All tests pass?** → Deploy to testnet
2. **Found bugs?** → Create GitHub issues
3. **Need features?** → Update requirements doc
4. **Ready for production?** → Run load tests

---

**🎉 Happy Testing! Everything is configured and ready to go!**
