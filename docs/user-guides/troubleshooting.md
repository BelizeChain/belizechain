# 🆘 Troubleshooting Guide

**Fix common BelizeChain problems**

---

## 📋 Quick Problem Finder

| Symptom | Go To |
|---------|-------|
| Can't install wallet | [Installation Issues](#installation-issues) |
| Can't connect wallet | [Connection Problems](#connection-problems) |
| Transaction failed | [Transaction Errors](#transaction-errors) |
| Wrong balance | [Balance Issues](#balance-issues) |
| Lost password/PIN | [Access Problems](#access-problems) |
| App crashes | [App Problems](#app-problems) |
| Sent to wrong address | [Irreversible Transactions](#irreversible-transactions) |
| Can't find transaction | [Missing Transactions](#missing-transactions) |

---

## Installation Issues

### iPhone: "Cannot Download App"

**Causes**:
- Not enough storage
- App Store region mismatch
- iOS version too old

**Solutions**:
1. **Check storage**:
   - Settings → General → iPhone Storage
   - Need 200MB free
   - Delete unused apps if needed

2. **Check iOS version**:
   - Settings → General → About → Software Version
   - Need iOS 13+ (iPhone 6S or newer)
   - Update iOS if available

3. **Check region**:
   - Settings → [Your Name] → Media & Purchases → View Account
   - Country must be Belize or allow BelizeChain
   - Change region if needed

4. **Restart iPhone**:
   - Hold Power + Volume button
   - Slide to power off
   - Wait 30 seconds
   - Power back on
   - Try download again

---

### Android: "App Not Available in Your Region"

**Solution**:
1. Enable unknown sources (if using APK):
   - Settings → Security → Unknown Sources → ON
2. Download APK from official site:
   - https://maya.belizechain.org/apk
3. Install manually

**OR** change Google Play region:
1. Open Play Store
2. Menu → Account → Country
3. Add Belize payment method
4. Region updates in 48 hours

---

### Windows: "Windows Protected Your PC"

**This is normal!** Windows SmartScreen checks new apps.

**Solution**:
1. Click **"More info"**
2. Click **"Run anyway"**
3. Click **"Yes"** when asked for permission
4. Installation continues

**OR** disable SmartScreen temporarily:
1. Windows Security → App & Browser Control
2. Reputation-based protection settings
3. Turn off "Check apps and files"
4. Install Maya Wallet
5. Turn back ON after install

---

### Mac: "Cannot Open Because Developer Cannot Be Verified"

**Solution**:
1. Control-click (or right-click) Maya Wallet app
2. Click **"Open"**
3. Click **"Open"** again to confirm
4. ✅ App opens (only needed first time)

**OR** allow in Security Settings:
1. System Preferences → Security & Privacy
2. Click lock and enter password
3. Under "Allow apps downloaded from:"
4. Click **"Open Anyway"** next to Maya Wallet
5. Try opening app again

---

## Connection Problems

### "Cannot Connect to Network"

**Symptoms**: Red "Offline" indicator, can't send/receive

**Quick Fixes**:
1. **Check internet**:
   - Open browser
   - Visit google.com
   - Works? Internet is fine
   - Doesn't work? Fix internet first

2. **Restart app**:
   - Force close Maya Wallet
   - Reopen
   - Wait 30 seconds for connection

3. **Try different network**:
   - Switch WiFi → Mobile Data
   - Or switch to different WiFi
   - Connection issues might be network-specific

4. **Check firewall** (Desktop):
   - Allow Maya Wallet through firewall
   - Windows: Windows Security → Firewall → Allow an app
   - Mac: System Preferences → Security → Firewall Options

5. **Update app**:
   - Check for Maya Wallet updates
   - Old versions may have connection bugs

---

### "Connecting..." Forever

**Causes**: Server overload, network issues, outdated app

**Solutions**:
1. **Wait 2 minutes**: Might be temporary server issue
2. **Check status**: https://status.belizechain.org
3. **Change RPC endpoint**:
   - Settings → Advanced → Network
   - Tap "RPC Endpoint"
   - Select different server:
     - rpc1.belizechain.org
     - rpc2.belizechain.org
     - rpc3.belizechain.org
   - Save and restart

4. **Clear cache**:
   - Settings → Advanced → Clear Cache
   - Restart app

5. **Reinstall app** (last resort):
   - Uninstall Maya Wallet
   - Reinstall from official source
   - Restore with 12 words
   - ✅ Connection should work

---

### "Network Error: Timeout"

**Quick fix**:
- Enable "Fallback Servers" in Settings → Advanced
- App automatically tries backup servers

---

## Transaction Errors

### "Insufficient Balance"

**Meaning**: Not enough money for amount + fee

**Example**:
- Balance: 50.00 bBZD
- Trying to send: 50.00 bBZD
- Fee: 0.01 bBZD
- **Need**: 50.01 bBZD total
- **Have**: 50.00 bBZD
- ❌ **Short by**: 0.01 bBZD

**Solution**:
- Send less: 49.99 bBZD
- **OR** add more funds first

**Tip**: Use "Max" button - automatically calculates correct amount

---

### "Transaction Failed: Invalid Address"

**Causes**:
- Typo in address
- Wrong format
- Address for different blockchain

**BelizeChain addresses**:
- ✅ Start with "5"
- ✅ Exactly 48 characters
- ✅ Example: `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`

**Solutions**:
1. **Don't type addresses** - always copy/paste or scan QR
2. **Verify address** with recipient before sending
3. **Test with $1 first** before sending large amounts

---

### "Transaction Pending Forever"

**Normal**: Should complete in 6-12 seconds  
**Problem**: Still pending after 5 minutes

**Solutions**:
1. **Wait and refresh**:
   - Pull down to refresh
   - Transaction might complete shortly

2. **Check block explorer**:
   - Copy transaction ID
   - Visit explorer.belizechain.org
   - Paste transaction ID
   - See real status

3. **Check internet**:
   - Transaction might be stuck due to connection
   - Ensure stable internet

4. **Contact support** if:
   - Pending > 30 minutes
   - Money deducted from balance
   - Can't find on block explorer

**Note**: If transaction fails, money returns to your wallet automatically

---

### "Nonce Too Low"

**Technical error** - usually from sending multiple transactions too fast

**Solution**:
1. Wait 30 seconds
2. Try transaction again
3. Should work now

**Prevention**: Wait for previous transaction to complete before sending next

---

## Balance Issues

### "Balance Shows $0 But I Have Money"

**Causes**:
1. Wrong network (testnet vs mainnet)
2. Wrong account selected
3. App not synced

**Solutions**:
1. **Check network**:
   - Settings → Advanced → Network
   - Should be "Mainnet" (not Testnet)
   - Switch if wrong

2. **Check account**:
   - Tap account name at top
   - See all accounts and balances
   - Switch to correct account

3. **Refresh**:
   - Pull down on main screen to refresh
   - Wait 10 seconds
   - Balance should update

4. **Check on block explorer**:
   - Copy your address
   - Visit explorer.belizechain.org
   - Search your address
   - See real balance on blockchain
   - If explorer shows balance but app doesn't:
     - Clear cache: Settings → Advanced → Clear Cache
     - Restart app

---

### "Balance Different in App vs Explorer"

**Possible reasons**:
1. **Pending transactions**: Explorer shows confirmed only
2. **Multiple accounts**: Check you're viewing same address
3. **Sync delay**: App might be few seconds behind

**Solution**:
- Wait 1 minute and refresh
- Check all accounts (tap account name)
- Verify you're comparing same address

---

### "Missing Funds After Transaction"

**Don't panic!** Money doesn't disappear on blockchain.

**Check these**:
1. **Transaction status**:
   - History → Find transaction
   - Status = Complete? ✅ It went through
   - Status = Failed? ❌ Money returned to you

2. **Block explorer confirmation**:
   - Copy transaction ID
   - Search on explorer
   - See where money went

3. **Wrong address?**
   - Check if you sent to correct address
   - Ask recipient to confirm they received
   - If wrong address: money is gone 😢 (see below)

4. **Account confusion**:
   - Check all accounts (you might have sent from different account)
   - Check recipient address is in correct account

---

## Access Problems

### "Forgot My PIN"

**If biometric enabled**:
- Use Face ID / Touch ID / Fingerprint
- ✅ Works! Change PIN in Settings → Security

**If no biometric**:
- Tap "Forgot PIN?"
- Only option: **Restore wallet with 12 words**
- ⚠️ **If you don't have 12 words**: Money is lost forever

**To restore**:
1. Tap "Forgot PIN?" → "Restore Wallet"
2. Uninstall and reinstall app
3. Choose "Restore Existing Wallet"
4. Enter your 12 words in order
5. Set new PIN
6. ✅ Wallet restored!

---

### "Lost My 12 Words"

**If wallet still on your device**:
1. Open Maya Wallet (use PIN/biometric)
2. Settings → Security → Backup Wallet
3. Enter PIN
4. **Write down the 12 words NOW**
5. Hide paper somewhere safe
6. ✅ Crisis averted!

**If wallet deleted and no 12 words**:
- 😢 **Your money is lost forever**
- Nobody can recover it (not even BelizeChain support)
- This is why backup is SO important

**Prevention**:
- Write backup phrase on paper TODAY
- Keep paper in safe place
- Make a second copy at parent's/sibling's house

---

### "Biometric Not Working"

**Solutions**:
1. **Re-register biometric**:
   - Settings → Security → Biometric Login
   - Toggle OFF then ON
   - Authenticate again

2. **Check phone settings**:
   - Ensure Face ID/Touch ID/Fingerprint enabled in phone settings
   - Might need to re-register in phone settings

3. **Use PIN instead**:
   - PIN always works as backup
   - Enter PIN manually

4. **Restart phone**:
   - Power off completely
   - Wait 30 seconds
   - Power back on
   - Try biometric again

---

## App Problems

### App Crashes on Startup

**Solutions (try in order)**:
1. **Restart phone**: Simple but often works

2. **Update app**:
   - Check App Store/Play Store for updates
   - Install latest version

3. **Clear cache**:
   - iOS: Settings → General → iPhone Storage → Maya Wallet → Delete App Data
   - Android: Settings → Apps → Maya Wallet → Storage → Clear Cache

4. **Reinstall app**:
   - Uninstall Maya Wallet
   - Restart phone
   - Reinstall from official source
   - Restore with 12 words
   - ✅ Should work now

5. **Contact support** with:
   - Phone model
   - OS version
   - When crash happens
   - Screenshots of error

---

### App Freezes / Unresponsive

**Quick fix**:
1. Force close app:
   - iOS: Swipe up from bottom, swipe Maya Wallet up
   - Android: Recent Apps button, swipe Maya Wallet away
2. Wait 10 seconds
3. Reopen app

**If happens repeatedly**:
- Clear cache (Settings → Advanced → Clear Cache)
- Check phone storage (need 1GB+ free)
- Update to latest app version

---

### "App Won't Update"

**iOS**:
1. App Store → Updates tab
2. Pull down to refresh
3. If no update shown but available:
   - Sign out of App Store
   - Sign back in
   - Check updates again

**Android**:
1. Play Store → Menu → My Apps & Games
2. Find Maya Wallet
3. Tap "Update"
4. If no update button:
   - Uninstall old version
   - Install new version fresh

---

## Irreversible Transactions

### "I Sent to Wrong Address!"

**Bad news**: 😢 **Blockchain transactions cannot be reversed**

**Why?** Blockchain is like physical cash:
- Hand someone $20 → it's theirs forever
- Send crypto → same thing

**What you CAN do**:
1. **Contact recipient** (if you know them):
   - Politely ask them to return funds
   - Some people will, some won't
   - No guarantee

2. **Check if address exists**:
   - Search address on explorer.belizechain.org
   - If address has no activity: might be typo (no one controls it)
   - If typo: money is lost in the void 😢

3. **Report to authorities** (if large amount):
   - File police report
   - Blockchain records are evidence
   - Might help if it was fraud

**Prevention** (Learn for next time):
- Always verify address with recipient first
- Send $1 test payment before large amounts
- Use QR codes (no typos)
- Add frequent recipients as contacts
- Double-check address before confirming

---

## Missing Transactions

### "Someone Said They Sent But I Didn't Receive"

**Steps to investigate**:

1. **Ask sender for transaction ID**:
   - They should have confirmation
   - Copy the TX ID

2. **Check block explorer**:
   - Go to explorer.belizechain.org
   - Paste transaction ID
   - See:
     - ✅ Status: Complete?
     - 📍 To address: Is it your address?
     - 💰 Amount: Correct?

3. **Verify address**:
   - Compare "To" address in explorer with YOUR address
   - Even 1 character different = wrong address!

4. **Check all accounts**:
   - Maybe they sent to different account
   - Check all your accounts

5. **Refresh your wallet**:
   - Pull down to refresh
   - Wait for sync

**Possible outcomes**:
- ✅ TX shows your address → Should appear in wallet (refresh)
- ❌ TX shows different address → Sent to wrong place
- ❌ TX not found → Sender didn't actually send (or wrong TX ID)

---

## Device-Specific Issues

### iPhone Storage Full

**Problem**: Can't update app, can't sync

**Solution**:
1. Settings → General → iPhone Storage
2. Delete unused apps
3. Delete photos/videos (back up first!)
4. Offload unused apps
5. Need 1GB+ free for wallet to work well

---

### Android Battery Optimization Killing App

**Problem**: Wallet closes when phone sleeps

**Solution**:
1. Settings → Apps → Maya Wallet
2. Battery → Battery Optimization
3. Select "Don't Optimize"
4. ✅ App stays running

---

### Mac Security Blocking App

See [Installation Issues](#installation-issues) → Mac section

---

## When to Contact Support

### Contact support if:
- ✅ Money missing > 1 hour
- ✅ Transaction pending > 30 minutes
- ✅ App crashes repeatedly
- ✅ Can't recover wallet with correct 12 words
- ✅ Security concern (suspicious activity)
- ✅ Bug/error message

### Don't need support for:
- ❌ Forgot PIN (use biometric or restore with 12 words)
- ❌ How to use features (check this guide or Maya Wallet Guide)
- ❌ Sent to wrong address (irreversible, we can't help)
- ❌ Price questions (market determines price)

---

## Contact Information

### Support Channels

**Phone** (fastest):
- +501-CHAIN (24246)
- Available 24/7
- English and Spanish

**Email**:
- support@belizechain.org
- Response within 1-4 hours
- Send screenshots if possible

**Live Chat**:
- In app: Menu → Help & Support → Chat
- On website: https://maya.belizechain.org
- 24/7 available

**In-Person Support Centers**:
- 📍 Belize City: 123 Marine Parade Blvd (8am-5pm)
- 📍 San Pedro: Island Plaza, Ground Floor (9am-4pm)
- 📍 Belmopan: Government Complex, Building B (8am-5pm)
- 📍 Orange Walk: Main Street, next to Scotia Bank (9am-4pm)

**Community**:
- Forum: forum.belizechain.org
- Discord: discord.gg/belizechain
- Telegram: t.me/belizechain

---

## Information to Provide Support

**When contacting support, include**:

1. **Your setup**:
   - Device: "iPhone 12" or "Samsung Galaxy S21"
   - OS: "iOS 16" or "Android 12"
   - App version: Settings → About → Version

2. **The problem**:
   - What happened
   - When it happened (date/time)
   - What you were trying to do

3. **Screenshots** (if possible):
   - Error messages
   - Transaction details
   - Account screen

4. **Transaction ID** (if relevant):
   - Copy from transaction history
   - Starts with `0x...`

5. **Your address** (safe to share):
   - From main screen
   - For balance checking

**DON'T share**:
- ❌ Your 12-word backup phrase
- ❌ Your PIN
- ❌ Your password
- ❌ Private keys

Support will NEVER ask for these!

---

## Prevention Checklist

### Keep your wallet healthy:

- [ ] ✅ Backup phrase written on paper (not digital!)
- [ ] ✅ Backup paper hidden safely
- [ ] ✅ Biometric enabled
- [ ] ✅ Auto-lock enabled (5-15 minutes)
- [ ] ✅ App updated to latest version
- [ ] ✅ Phone OS updated
- [ ] ✅ 1GB+ free storage on device
- [ ] ✅ Strong PIN (not 123456!)
- [ ] ✅ Frequent contacts saved
- [ ] ✅ Transaction limits set for large amounts
- [ ] ✅ Test small amount before large sends

---

## Emergency Procedures

### Phone Stolen / Lost

**Immediate actions**:
1. ✅ **Don't panic** - money is safe if you have backup phrase
2. ✅ **Get new phone**
3. ✅ **Install Maya Wallet**
4. ✅ **Restore with 12 words**
5. ✅ **Change PIN immediately**
6. ✅ **Check transaction history** for unauthorized activity

**If you had PIN/biometric**:
- Thief can't access wallet without PIN or your face/fingerprint
- You have time to restore on new device

**If no PIN/biometric** (you ignored our warnings):
- Thief might access wallet 😢
- Restore quickly and move funds immediately

---

### Suspicious Activity

**If you see transactions you didn't make**:

1. **Immediately**:
   - Create new wallet (different 12 words)
   - Send all funds to new wallet
   - Abandon compromised wallet

2. **Investigate**:
   - Did someone see your 12 words?
   - Did you enter words on a fake website?
   - Did you click a phishing link?

3. **Report**:
   - Contact support: support@belizechain.org
   - File police report if large amount
   - Report scam website to authorities

---

**Still stuck?** Contact support - we're here to help! 🤝

**Back to**: [User Guides](./README.md)
