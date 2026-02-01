# 🌊 Blue Hole Explorer Guide

**Explore the BelizeChain blockchain with transparency**

---

## 📋 Overview

**Blue Hole Explorer** is BelizeChain's official block explorer - your window into the blockchain. See every transaction, block, account, and proposal in real-time with full transparency.

**Named after**: Belize's famous Great Blue Hole, a natural wonder where you can see deep into the crystal-clear waters. Similarly, Blue Hole Explorer lets you see deep into the blockchain!

---

## 🎯 What is a Block Explorer?

### Simple Explanation

Think of the blockchain as a **giant public ledger** (record book):
- Every transaction is recorded
- Every block is saved
- Everything is public
- Nothing can be deleted or changed

**Blue Hole Explorer** lets you read this ledger:
- Search for transactions
- Verify payments
- Track accounts
- Monitor proposals
- Explore blocks
- See statistics

### Why It's Important

**Transparency**: Anyone can verify anything  
**Security**: Catch errors or fraud  
**Curiosity**: Explore how blockchain works  
**Research**: Analyze data and trends

---

## 🌐 Access Blue Hole Explorer

### Website
**URL**: https://explorer.belizechain.org

**Works on**:
- 💻 Desktop computers (recommended)
- 📱 Mobile phones
- 📱 Tablets
- Any modern web browser

**No login required!** Public and free.

### Alternative Explorers

**Official**:
- https://explorer.belizechain.org (main)
- https://scan.belizechain.org (faster, simpler)

**Community**:
- https://belizechain.subscan.io (Subscan)
- https://belizechain.explorers.guru (Community)

---

## 🏠 Home Page Overview

### What You See

```
╔════════════════════════════════════════════════╗
║  🌊 BLUE HOLE EXPLORER                         ║
║  Explore BelizeChain with Transparency         ║
╠════════════════════════════════════════════════╣
║  [Search: Address, Tx Hash, Block...        🔍]║
╠════════════════════════════════════════════════╣
║  NETWORK STATISTICS                            ║
║  ┌──────────────────────────────────────────┐ ║
║  │ Current Block: #5,234,567                │ ║
║  │ Block Time: 6 seconds                    │ ║
║  │ Transactions: 15,234,891 total           │ ║
║  │ Active Accounts: 48,392                  │ ║
║  │ Market Cap: $85.2M                       │ ║
║  │ DALLA Price: $0.42 USD                   │ ║
║  └──────────────────────────────────────────┘ ║
║                                                ║
║  LATEST BLOCKS                                 ║
║  Block #5,234,567  6 sec ago  23 txs          ║
║  Block #5,234,566  12 sec ago  19 txs         ║
║  Block #5,234,565  18 sec ago  31 txs         ║
║  [View All Blocks]                            ║
║                                                ║
║  LATEST TRANSACTIONS                           ║
║  0x7d4e...  5Fhn... → 5Grw...  1,200 bBZD    ║
║  0x8a2f...  5Cip... → 5Hgj...  450 DALLA     ║
║  0x9b3c...  5Dkl... → 5Jmn...  2,800 bBZD    ║
║  [View All Transactions]                      ║
╚════════════════════════════════════════════════╝
```

### Key Sections

1. **Search Bar** - Find anything
2. **Network Stats** - Live blockchain data
3. **Latest Blocks** - Recently produced blocks
4. **Latest Transactions** - Recent transfers
5. **Charts** (scroll down) - Trends and analytics

---

## 🔍 Searching

### What You Can Search

| Search For | Example | What You'll Find |
|------------|---------|------------------|
| **Transaction** | 0x7d4e3f2a... | Transaction details |
| **Address** | 5FHneW46xGXgs... | Account info, balance, history |
| **Block Number** | 5234567 | Block details, transactions |
| **Block Hash** | 0x8a2f... | Same as block number |
| **Proposal** | #156 | Governance proposal details |

### How to Search

**Step 1**: Click search bar at top

**Step 2**: Paste or type:
- Transaction ID (starts with 0x)
- Address (starts with 5)
- Block number (just number)

**Step 3**: Press Enter or click 🔍

**Step 4**: View results!

---

## 💳 View Transaction Details

### Find a Transaction

**Method 1**: Search by transaction ID
```
Example: 0x7d4e3f2a1b5c8d9e0f1a2b3c4d5e6f7a8b9c0d1e
```

**Method 2**: Click from "Latest Transactions"

**Method 3**: View from your Maya Wallet history

### Transaction Page

```
╔════════════════════════════════════════════════╗
║  TRANSACTION DETAILS                           ║
╠════════════════════════════════════════════════╣
║  Status: ✅ SUCCESS                             ║
║  Tx Hash: 0x7d4e3f2a1b5c8d9e0f1a2b3c4d5e6f...  ║
║  Block: #5,234,567                             ║
║  Timestamp: Oct 13, 2025 at 2:45:30 PM        ║
║  Age: 15 minutes ago                           ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  FROM                                          ║
║  5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92Uhj   ║
║  Balance: 15,234.50 DALLA                      ║
║                                                ║
║  TO                                            ║
║  5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoH   ║
║  Balance: 8,921.32 DALLA                       ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  AMOUNT                                        ║
║  1,200.00 bBZD                                 ║
║  ($1,200.00 USD)                               ║
║                                                ║
║  FEE                                           ║
║  0.01 bBZD                                     ║
║  ($0.01 USD)                                   ║
║                                                ║
║  MEMO                                          ║
║  "Rent payment - October 2025"                 ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  DETAILS                                       ║
║  Nonce: 42                                     ║
║  Method: balances.transfer                     ║
║  Weight: 159,000,000                           ║
║  Events: 2 (Transfer, Fee Paid)                ║
║                                                ║
║  [View in Maya Wallet] [Copy Tx ID] [Share]   ║
╚════════════════════════════════════════════════╝
```

### Understanding Transaction Status

| Status | Icon | Meaning |
|--------|------|---------|
| **Success** | ✅ | Transaction completed |
| **Failed** | ❌ | Transaction rejected |
| **Pending** | ⏳ | Still processing (wait 1 min) |

### Transaction Fields Explained

**Tx Hash**: Unique transaction ID (like a receipt number)  
**Block**: Which block contains this transaction  
**Timestamp**: Exact date and time  
**From**: Sender's address  
**To**: Recipient's address  
**Amount**: How much was sent  
**Fee**: Cost to process (usually 0.01 bBZD)  
**Memo**: Optional message from sender  
**Nonce**: Transaction number for sender (prevents replay)  
**Events**: What happened (Transfer, Fee, etc.)

---

## 👤 View Account (Address)

### Find an Account

**Method 1**: Search by address
```
Example: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
```

**Method 2**: Click address from transaction

**Method 3**: Enter your own address from Maya Wallet

### Account Page

```
╔════════════════════════════════════════════════╗
║  ACCOUNT                                       ║
║  5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92Uhj    ║
╠════════════════════════════════════════════════╣
║  QR CODE                                       ║
║  [QR code image]                               ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  BALANCES                                      ║
║  💰 15,234.50 DALLA                            ║
║     ($6,398.49 USD)                            ║
║                                                ║
║  💵 2,450.00 bBZD                              ║
║     ($2,450.00 USD)                            ║
║                                                ║
║  Total Value: $8,848.49 USD                    ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  STATISTICS                                    ║
║  Total Transactions: 287                       ║
║  Sent: 189 transactions                        ║
║  Received: 98 transactions                     ║
║  First Activity: Jan 15, 2025                  ║
║  Last Activity: 15 minutes ago                 ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  STAKING (if validator)                        ║
║  Staked: 10,000 DALLA                          ║
║  Commission: 5%                                ║
║  Total Stakers: 234                            ║
║  Rewards Earned: 1,234.50 DALLA                ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  GOVERNANCE                                    ║
║  Votes Cast: 42                                ║
║  Proposals Created: 2                          ║
║  Last Vote: 3 days ago                         ║
╚════════════════════════════════════════════════╝

[Transaction History ▼]
```

### Transaction History

Scroll down to see all transactions:

```
TRANSACTION HISTORY (287 total)

Filters: [All] [Sent] [Received] [Date Range]

Oct 13, 2025
  ↗️ Sent 1,200.00 bBZD to 5Grwv...
     2:45 PM • Tx: 0x7d4e... • "Rent payment"
     
  ↙️ Received 450.00 DALLA from 5Cipp...
     10:23 AM • Tx: 0x8a2f... • "Payment for services"

Oct 12, 2025
  ↗️ Sent 2,800.00 bBZD to 5Dkl...
     6:15 PM • Tx: 0x9b3c... • "Invoice #1234"
     
... (284 more transactions)

[Load More] [Export CSV]
```

### Export Transaction History

**Click "Export CSV"** to download:
- All transactions
- Date, time, amount
- From/to addresses
- Memos
- **Use for**: Accounting, taxes, records

---

## 📦 View Block Details

### Find a Block

**Method 1**: Search by block number
```
Example: 5234567
```

**Method 2**: Click from "Latest Blocks"

**Method 3**: Browse block-by-block

### Block Page

```
╔════════════════════════════════════════════════╗
║  BLOCK #5,234,567                              ║
╠════════════════════════════════════════════════╣
║  Status: ✅ Finalized                           ║
║  Timestamp: Oct 13, 2025 at 2:45:24 PM        ║
║  Age: 15 minutes ago                           ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  BLOCK DETAILS                                 ║
║  Block Hash: 0x8a2f3b4c5d6e7f8a9b0c1d2e3f4...  ║
║  Parent Hash: 0x7d4e3f2a1b5c8d9e0f1a2b3c4d5... ║
║  Validator: Maria's Validator                  ║
║  Validator Address: 5Jklm...                   ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  CONTENT                                       ║
║  Transactions: 23                              ║
║  Extrinsics: 25 (23 txs + 2 system)           ║
║  Events: 48                                    ║
║  Block Weight: 875,234,192                     ║
║  Block Size: 15.2 KB                           ║
║                                                ║
║  ─────────────────────────────────────────────║
║                                                ║
║  TRANSACTIONS IN THIS BLOCK                    ║
║                                                ║
║  1. 0x7d4e... Transfer 1,200 bBZD ✅           ║
║     5Fhn... → 5Grw...                          ║
║                                                ║
║  2. 0x8a2f... Transfer 450 DALLA ✅            ║
║     5Cip... → 5Hgj...                          ║
║                                                ║
║  3. 0x9b3c... Vote on Proposal #156 ✅         ║
║     5Dkl...                                    ║
║                                                ║
║  ... (20 more transactions)                    ║
║                                                ║
║  [View All 23 Transactions]                    ║
╚════════════════════════════════════════════════╝

[← Previous Block] [Next Block →]
```

### Block Fields Explained

**Block Number**: Sequential number (starts at 0)  
**Block Hash**: Unique identifier for this block  
**Parent Hash**: Link to previous block (forms chain!)  
**Validator**: Who produced this block  
**Transactions**: How many payments/actions  
**Timestamp**: When block was created  
**Block Weight**: Computational cost  
**Block Size**: Data size in kilobytes

---

## 📊 Charts & Analytics

### Available Charts

**1. Transaction Volume**
```
Daily Transactions (Last 30 Days)

15,000 |                              ╱╲
        |                            ╱    ╲
10,000 |                          ╱        ╲
        |        ╱╲              ╱
 5,000 |      ╱    ╲          ╱
        |    ╱        ╲      ╱
     0 |_____________________╲_______________
        Oct 1              Oct 15          Oct 30

Peak: Oct 20 (14,823 txs) - Tourist season!
```

**2. DALLA Price History**
```
DALLA/USD Price (Last 90 Days)

$0.50 |                    ╱─────╲
      |                  ╱         ╲
$0.40 |    ╱───╲       ╱             ╲____
      |  ╱       ╲   ╱
$0.30 |╱           ╲╱
      |_____________________________________
      Jul          Aug       Sep       Oct

Current: $0.42 USD
Change: +12% (3 months)
```

**3. Active Addresses**
```
Daily Active Addresses

2,000 |              ╱╲
      |            ╱    ╲
1,500 |          ╱        ╲╱╲
      |        ╱              ╲
1,000 |      ╱                  ╲
      |    ╱                      ╲
  500 |  ╱
      |╱_________________________________
      Week 1   Week 2   Week 3   Week 4

Growing: +23% this month!
```

**4. Validator Performance**
```
Top 10 Validators by Blocks Produced

Maria's Validator     ████████████████ 8,234 blocks
Carlos Node          ███████████████ 7,891 blocks
Ana's Staking        ██████████████ 7,654 blocks
...
```

---

## 🏛️ Governance Explorer

### View Proposals

**Click "Governance" tab**:

```
╔════════════════════════════════════════════════╗
║  GOVERNANCE PROPOSALS                          ║
╠════════════════════════════════════════════════╣
║  Filters: [All] [Active] [Passed] [Rejected]  ║
║                                                ║
║  ACTIVE PROPOSALS (6)                          ║
║                                                ║
║  #156 Fund National Healthcare AI              ║
║  💰 500,000 bBZD                               ║
║  ⏰ 4 days left                                 ║
║  🟢 YES: 72% • 🔴 NO: 18% • ⚪ ABSTAIN: 10%    ║
║  [View Details]                                ║
║                                                ║
║  #157 Increase Block Size to 5MB               ║
║  ⚙️ Technical                                  ║
║  ⏰ 6 days left                                 ║
║  🟢 YES: 82% • 🔴 NO: 15% • ⚪ ABSTAIN: 3%     ║
║  [View Details]                                ║
║                                                ║
║  ... (4 more active proposals)                 ║
║                                                ║
║  RECENTLY PASSED (10)                          ║
║  RECENTLY REJECTED (3)                         ║
╚════════════════════════════════════════════════╝
```

### Proposal Details

**Click proposal to see**:
- Full description
- Vote breakdown
- Who voted how (if not anonymous)
- Discussion comments
- Execution status

---

## 🔔 Real-Time Updates

### Live Features

**Auto-refresh** (default: 10 seconds)
- Latest blocks appear
- New transactions show
- Balances update
- Price changes

**WebSocket connection**:
- Real-time data
- No page reload needed
- Instant updates

**Notifications** (if enabled):
- Your transactions confirmed
- Proposals you voted on end
- Blocks your validator produces

---

## 📱 Mobile Experience

### Mobile-Friendly

**Blue Hole Explorer works great on phones!**

**Features**:
- Responsive design
- Touch-friendly
- Fast loading
- All features available

**Tip**: Add to home screen for quick access!

**iOS**:
1. Open in Safari
2. Tap Share button
3. "Add to Home Screen"

**Android**:
1. Open in Chrome
2. Tap menu (⋮)
3. "Add to Home screen"

---

## 🔍 Advanced Search

### Search Tips

**Transaction ID**:
- Starts with `0x`
- 66 characters long
- Copy from wallet, don't type!

**Address**:
- Starts with `5`
- 48 characters long
- Case-sensitive!

**Block Number**:
- Just number: `5234567`
- Or with #: `#5234567`

**Partial Search**:
- Some explorers support partial
- Example: `5Fhn...` finds `5FHneW46xGXgs...`

### Search Shortcuts

| Shortcut | Goes To |
|----------|---------|
| `/` key | Focus search bar |
| `Escape` | Clear search |
| `Enter` | Search |
| `Ctrl+K` | Quick search (Mac: ⌘K) |

---

## 💡 Common Use Cases

### 1. Verify Payment Received

**Scenario**: Someone says they sent you money

**Steps**:
1. Get transaction ID from sender
2. Search on Blue Hole Explorer
3. Check:
   - ✅ Status: Success?
   - ✅ To address: Matches yours?
   - ✅ Amount: Correct?
   - ✅ Timestamp: Recent?

**If everything checks out**: Payment confirmed! ✅

---

### 2. Proof of Payment

**Scenario**: Need to prove you paid someone

**Steps**:
1. Find transaction in your Maya Wallet history
2. Copy transaction ID
3. Search on Blue Hole Explorer
4. Click **"Share"** button
5. Send link to recipient

**Link example**:
```
https://explorer.belizechain.org/tx/0x7d4e3f2a...
```

**Recipient can verify**:
- Who sent (your address)
- Amount sent
- Date/time
- Can't be faked! (on blockchain)

---

### 3. Check Account Balance (Public)

**Scenario**: Want to see someone else's balance (public addresses)

**Steps**:
1. Get their address
2. Search on Blue Hole Explorer
3. See their balance

**Privacy note**: All balances are public on blockchain!

**Use cases**:
- Check donation addresses
- Verify business has funds
- Public accountability (government addresses)

---

### 4. Monitor Validator

**Scenario**: You're staking with a validator, want to monitor

**Steps**:
1. Search validator address
2. Check:
   - Blocks produced
   - Commission rate
   - Total stakers
   - Uptime
   - Rewards distributed

**Red flags**:
- ⚠️ Many missed blocks (low uptime)
- ⚠️ Very high commission (>20%)
- ⚠️ No recent activity

---

### 5. Track Governance Proposal

**Scenario**: You voted on proposal, want to track progress

**Steps**:
1. Click "Governance" tab
2. Find your proposal
3. See real-time vote count
4. Check discussion
5. Get notified when ends

---

### 6. Tax Records / Accounting

**Scenario**: Need transaction history for taxes

**Steps**:
1. Search your address
2. View transaction history
3. Click **"Export CSV"**
4. Download file
5. Open in Excel/Google Sheets
6. Give to accountant

**CSV includes**:
- Date, time
- From, to
- Amount
- Fee
- Memo
- Transaction ID

---

## 📖 Understanding Blockchain Concepts

### What is a Block?

**Simple**: Container that holds transactions

**Like a page in a book**:
- Block 1 = Page 1
- Block 2 = Page 2
- Block 3 = Page 3

**Each block contains**:
- ~20-50 transactions
- Link to previous block
- Timestamp
- Validator signature

**New block every 6 seconds!**

### What is a Transaction?

**Simple**: Transfer of value or action

**Types**:
- Transfer DALLA/bBZD
- Vote on proposal
- Stake tokens
- Claim rewards
- Update identity

**Every transaction**:
- Has unique ID
- Costs small fee (0.01 bBZD)
- Is permanent (can't be reversed!)
- Is public (visible to all)

### What Makes it Secure?

**Cryptography**:
- Each block linked to previous (hash chain)
- Can't change past blocks
- Validators verify everything
- Public verification

**Analogy**: Writing in permanent ink, witnessed by 100 people, then locked in glass case!

---

## 🛠️ Blue Hole Explorer Tools

### Developer Tools

**API Access**:
```
https://api.explorer.belizechain.org

Endpoints:
  /blocks
  /transactions
  /accounts
  /proposals
```

**Use for**: Build your own apps, analyze data

---

### Export Options

**CSV Export**:
- Transaction history
- Account data
- Block data

**JSON Export**:
- Raw blockchain data
- For developers
- API responses

**PDF Export**:
- Transaction receipts
- Account statements
- Proof documents

---

## 🎨 Customization

### Settings

**Click ⚙️ (Settings)**:

```
EXPLORER SETTINGS

Display:
  ☑️ Dark Mode
  ☑️ Compact View
  ☐ Show Technical Details
  
Updates:
  Auto-refresh: [10 seconds ▼]
  ☑️ Enable WebSocket (real-time)
  ☑️ Notifications
  
Privacy:
  ☐ Anonymous Mode (hide your searches)
  ☑️ Remember Recent Searches
  
Language:
  [English ▼]
  Options: English, Spanish
```

---

## ⚠️ Privacy & Security

### What's Public

**On blockchain (anyone can see)**:
- All addresses
- All balances
- All transactions
- All votes
- All proposals

**NOT on blockchain**:
- Your name (unless you share)
- Your email
- Your location
- Your identity (unless KYC'd)

### Protect Your Privacy

**DO**:
- ✅ Use different addresses for different purposes
- ✅ Be careful sharing your address publicly
- ✅ Use VPN if concerned

**DON'T**:
- ❌ Post your address with your name on social media
- ❌ Reuse same address for everything
- ❌ Share transaction IDs publicly if private

**Remember**: Blockchain is PERMANENT and PUBLIC!

---

## 🆘 Troubleshooting

### "Transaction Not Found"

**Possible reasons**:
1. Still pending (wait 1 minute)
2. Wrong transaction ID
3. Wrong network (mainnet vs testnet)

**Solution**: Double-check ID, wait, try again

---

### "Address Invalid"

**Check**:
- Starts with `5`?
- Exactly 48 characters?
- No spaces?
- Correct capitalization?

---

### "Block Explorer Down"

**Alternatives**:
- Try: https://scan.belizechain.org
- Or: https://belizechain.subscan.io

---

### "Data Not Updating"

**Solutions**:
1. Refresh page (F5)
2. Clear cache
3. Check internet connection
4. Enable WebSocket in settings

---

## 📚 Related Resources

**Guides**:
- [Maya Wallet Guide](./maya-wallet.md) - Your personal wallet
- [Troubleshooting](./troubleshooting.md) - Fix common issues
- [Governance Guide](./governance.md) - Track proposals

**Tools**:
- **Blue Hole Explorer**: https://explorer.belizechain.org
- **API Docs**: https://docs.explorer.belizechain.org
- **Mobile App**: Search "Blue Hole Explorer" in app stores

**Support**:
- **Email**: explorer@belizechain.org
- **Discord**: #explorer-help channel
- **Forum**: forum.belizechain.org/explorer

---

## ✅ Explorer Checklist

Track your explorer skills:

### Beginner
- [ ] Search a transaction
- [ ] View an account balance
- [ ] Explore a block
- [ ] View latest transactions

### Intermediate
- [ ] Export transaction history
- [ ] Track a proposal
- [ ] Monitor a validator
- [ ] Verify a payment

### Advanced
- [ ] Use API
- [ ] Analyze charts
- [ ] Export data for accounting
- [ ] Build custom queries

---

## 🌊 Dive Deep into the Blockchain!

Blue Hole Explorer gives you **complete transparency** into BelizeChain.

Explore with confidence! 🔍

---

**Start exploring**: https://explorer.belizechain.org  
**Need help?** explorer@belizechain.org  
**Learn more**: [Technical Reference](../technical-reference/README.md)
