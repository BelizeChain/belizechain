# BNS Decentralized Web Hosting

**Host Websites on BelizeChain Using DAG Storage**

Complete guide to hosting static websites on .bz domains using Pakit's sovereign DAG storage system.

---

## Hosting Overview

BNS provides decentralized web hosting with:
- **DAG Storage**: Content-addressed, sovereign storage (via Pakit)
- **No External Dependencies**: 100% BelizeChain infrastructure
- **Version Control**: Full content history with rollback
- **CDN Caching**: Fast global content delivery (Pro/Enterprise tiers)
- **SSL/TLS Support**: HTTPS encryption for all hosted sites
- **Monthly Subscriptions**: Pay-as-you-go hosting

---

## Hosting Tiers

| Tier | Storage | Bandwidth | Price | Features |
|------|---------|-----------|-------|----------|
| **Free** | 100 MB | Basic | 0 DALLA/mo | DAG storage, basic CDN |
| **Basic** | 1 GB | Standard | 10 DALLA/mo | DAG + CDN caching |
| **Pro** | 10 GB | Enhanced | 50 DALLA/mo | Full CDN, compression |
| **Enterprise** | 100 GB | Premium | 200 DALLA/mo | Priority CDN, analytics |

**Billing Cycle:** 30 days (432,000 blocks)  
**Auto-Renewal:** Optional (enabled by default)  
**Payment Method:** DALLA tokens only

---

## Quick Start

### Prerequisites

1. **Registered Domain**: Own a .bz domain
2. **Website Files**: Static HTML/CSS/JS files ready
3. **DALLA Balance**: Tier fee + gas costs
4. **Pakit CLI**: Install for content upload

---

### 1. Install Pakit CLI

```bash
# Install Pakit CLI tools
npm install -g @belizechain/pakit-cli

# Or with Yarn
yarn global add @belizechain/pakit-cli

# Verify installation
pakit --version
# Output: @belizechain/pakit-cli v1.2.0
```

---

### 2. Prepare Website Files

**Directory Structure:**
```
my-website/
├── index.html       # Homepage (required)
├── about.html
├── contact.html
├── css/
│   └── style.css
├── js/
│   └── script.js
└── images/
    ├── logo.png
    └── hero.jpg
```

**Important:**
- Must have `index.html` at root
- All paths relative to root
- No server-side code (PHP, Node.js, Python)
- Static files only (HTML, CSS, JS, images, fonts)

---

### 3. Upload Website to DAG Storage

```bash
# Navigate to website directory
cd my-website

# Upload to Pakit DAG storage
pakit upload --dir . --manifest

# Output:
# Uploading files to DAG storage...
# ✅ 15 files uploaded
# ✅ Manifest created
# Content Hash: 0x1a2b3c4d5e6f7890abcdef...
# Total Size: 2.4 MB
# Blocks Created: 23
```

**Content Hash:**
- 32-byte DAG root hash
- Content-addressed (immutable)
- Used for domain resolution

---

### 4. Activate Hosting

**Via Web UI:**
```
1. Visit https://bns.belizechain.org/hosting
2. Select your domain
3. Click "Activate Hosting"
4. Choose hosting tier:
   - Free (100 MB)
   - Basic (1 GB) - 10 DALLA/month
   - Pro (10 GB) - 50 DALLA/month
   - Enterprise (100 GB) - 200 DALLA/month
5. Paste content hash from upload
6. Enable/disable auto-renewal
7. Review payment schedule
8. Click "Activate"
9. Sign transaction
```

**Via SDK:**
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

async function activateHosting() {
    const sdk = new GemSDK('wss://rpc.belizechain.org');
    await sdk.connect();

    const alice = sdk.getAccount(process.env.SEED_PHRASE);
    
    // Content hash from pakit upload
    const contentHash = '0x1a2b3c4d5e6f7890abcdef...';
    
    // Activate Basic tier hosting (10 DALLA/month)
    await sdk.api.tx.bns.activateHosting(
        'alice',           // Domain name
        1,                 // Tier: 0=Free, 1=Basic, 2=Pro, 3=Enterprise
        contentHash,       // DAG content hash
        true               // Auto-renewal enabled
    ).signAndSend(alice, ({ status, events }) => {
        if (status.isInBlock) {
            console.log('✅ Hosting activated!');
            console.log('Visit: https://alice.bz');
            
            events.forEach(({ event }) => {
                if (event.method === 'HostingActivated') {
                    const [domain, owner, tier, hash] = event.data;
                    console.log('Domain:', domain.toHuman());
                    console.log('Tier:', tier.toNumber());
                    console.log('Content:', hash.toHuman());
                }
            });
        }
    });
}

activateHosting().catch(console.error);
```

---

### 5. Access Your Website

Once activated, your website is live at:

```
https://alice.bz
```

**Resolution Flow:**
```
User visits https://alice.bz
    ↓
Browser queries BNS pallet for content hash
    ↓
BNS returns DAG root hash: 0x1a2b...
    ↓
Browser fetches content from Pakit DAG storage
    ↓
Files retrieved via CDN (if tier supports)
    ↓
Website rendered in browser
```

---

## Managing Hosted Content

### Update Website

```bash
# Make changes to your website
nano index.html

# Upload new version to DAG
pakit upload --dir . --manifest

# Output:
# Content Hash: 0xfedcba0987654321... (NEW)

# Update domain content
```

**Via SDK:**
```javascript
// Update to new content hash
await sdk.api.tx.bns.updateContent(
    'alice',      // Domain
    newContentHash  // New DAG hash
).signAndSend(alice);

// Event: ContentUpdated { domain, version: 2, content_hash }
```

**Version Tracking:**
- Every update creates new version
- Version history preserved on-chain
- Rollback to previous versions anytime
- Full audit trail

---

### Rollback to Previous Version

```javascript
// Get content version history
const versions = await sdk.api.query.bns.contentHistory.entries('alice');

console.log('Version History:');
versions.forEach(([key, version]) => {
    const versionNum = key.args[1].toNumber();
    const data = version.unwrap();
    console.log(`Version ${versionNum}:`);
    console.log('  Hash:', data.contentHash.toHex());
    console.log('  Uploaded:', data.uploadedAt.toNumber());
    console.log('  Size:', data.sizeBytes.toNumber(), 'bytes');
});

// Rollback to version 1
await sdk.api.tx.bns.rollbackContent(
    'alice',  // Domain
    1         // Version number
).signAndSend(alice);

// Event: ContentRolledBack { domain, version: 1, content_hash }
```

---

### Pause Hosting

```javascript
// Pause hosting (content preserved, stops billing)
await sdk.api.tx.bns.pauseHosting('alice').signAndSend(alice);

// Resume hosting (restarts billing)
await sdk.api.tx.bns.resumeHosting('alice', true).signAndSend(alice);  // auto-renew
```

---

## Hosting Payments

### Monthly Billing

**Automatic Deduction:**
- Every 30 days (432,000 blocks)
- Deducted from domain owner's wallet
- Hosting fee based on tier

**Payment Schedule:**
```javascript
// Check next payment date
const hosting = await sdk.api.query.bns.hostedWebsites('alice');
if (hosting.isSome) {
    const data = hosting.unwrap();
    const currentBlock = await sdk.api.query.system.number();
    const blocksUntilPayment = data.expiresAt.toNumber() - currentBlock.toNumber();
    const daysUntilPayment = (blocksUntilPayment * 6) / 86400;
    
    console.log(`Next payment in ${daysUntilPayment.toFixed(1)} days`);
    console.log(`Amount: ${data.monthlyFee.toNumber() / 1e12} DALLA`);
}
```

---

### Manual Renewal

```javascript
// Manually renew for 3 months
await sdk.api.tx.bns.renewHosting(
    'alice',  // Domain
    3         // Months to extend
).signAndSend(alice);

// Deducts: monthlyFee * 3
// Extends expiry by: 432,000 * 3 blocks
```

---

### Auto-Renewal

**Enable:**
```javascript
await sdk.api.tx.bns.setAutoRenewal('alice', true).signAndSend(alice);
```

**Disable:**
```javascript
await sdk.api.tx.bns.setAutoRenewal('alice', false).signAndSend(alice);
```

**How It Works:**
- Automatically deducts monthly fee on expiry
- Continues until disabled or insufficient balance
- Email notification 7 days before payment (if email set)

---

## External Domain Hosting

Host websites on your existing (.com, .org, .net) domains using BelizeChain.

### 1. Register External Domain

```javascript
// Register example.com for hosting
await sdk.api.tx.bns.registerExternal(
    'example.com',     // External domain
    'alice',           // Link to alice.bz
    1                  // Tier: 1=Basic (10 DALLA/month)
).signAndSend(alice);

// Output:
// Verification Token: 0xabc123def456...
```

---

### 2. Verify DNS Ownership

Add TXT record to your domain's DNS:

```
Record Type: TXT
Name: _belizechain-verification.example.com
Value: bz-verify-abc123def456...
TTL: 3600
```

**DNS Providers:**
- GoDaddy: DNS Management → TXT Record → Add
- Cloudflare: DNS → Add Record → TXT
- Namecheap: Advanced DNS → Add New Record

---

### 3. Verify on BelizeChain

```javascript
// Trigger verification check
await sdk.api.tx.bns.verifyExternal('example.com').signAndSend(alice);

// BelizeChain queries DNS for TXT record
// If match: domain verified
// If no match: verification fails (can retry)
```

**Verification Status:**
```javascript
const status = await sdk.api.query.bns.domainVerification('example.com');
if (status.isSome) {
    const data = status.unwrap();
    console.log('Verified:', data.verified);
    console.log('Attempts:', data.attempts);
}
```

---

### 4. Host Website

Once verified, host content same as .bz domains:

```javascript
// Upload content to DAG
pakit upload --dir ./example-com-website --manifest
// Content Hash: 0x9876543210...

// Update external domain content
await sdk.api.tx.bns.updateExternalContent(
    'example.com',
    '0x9876543210...'
).signAndSend(alice);
```

**Access:**
- Configure CNAME: `example.com → gateway.belizechain.org`
- Users visit `https://example.com`
- Content served from BelizeChain DAG storage

---

## SSL/TLS Certificates

### Automatic HTTPS

All hosted sites get free SSL/TLS certificates:
- Let's Encrypt integration
- Auto-renewal every 90 days
- HTTPS enforced (HTTP redirects)

**Certificate Info:**
```javascript
const ssl = await sdk.api.query.bns.sslCertificates('alice');
if (ssl.isSome) {
    const cert = ssl.unwrap();
    console.log('Issued:', cert.issuedAt.toNumber());
    console.log('Expires:', cert.expiresAt.toNumber());
    console.log('Issuer:', cert.issuer.toString());
}
```

---

### Custom Certificates

For enterprise customers:

```javascript
// Upload custom SSL cert (Enterprise tier only)
await sdk.api.tx.bns.uploadCustomCert(
    'alice',
    certHash,      // SHA-256 hash of cert
    serialNumber,  // Cert serial number
    expiryBlock    // When cert expires
).signAndSend(alice);
```

---

## Performance Optimization

### CDN Caching (Pro/Enterprise)

**Cache Settings:**
- Static assets: 30-day cache
- HTML files: 1-hour cache
- API responses: No cache
- Custom cache headers supported

**Purge Cache:**
```javascript
// Force CDN cache refresh
await sdk.api.tx.bns.purgeCdnCache('alice').signAndSend(alice);

// All CDN nodes refresh within 60 seconds
```

---

### Compression

**Automatic:**
- Brotli compression for text files
- GZIP fallback for legacy browsers
- Image optimization (WebP conversion)

**Manual Control:**
```javascript
// Enable/disable compression
await sdk.api.tx.bns.setCompression(
    'alice',
    true,     // Enable
    'brotli'  // Algorithm: 'brotli', 'gzip', 'zstd'
).signAndSend(alice);
```

---

## Analytics (Enterprise Tier)

**Metrics Tracked:**
- Page views
- Unique visitors
- Bandwidth usage
- Popular pages
- Referrers
- Geographic distribution

**Query Analytics:**
```javascript
const analytics = await sdk.api.query.bns.hostingAnalytics('alice');

console.log('Last 30 Days:');
console.log('  Page Views:', analytics.pageViews.toNumber());
console.log('  Unique Visitors:', analytics.uniqueVisitors.toNumber());
console.log('  Bandwidth:', analytics.bandwidthBytes.toNumber() / 1e9, 'GB');
console.log('  Top Page:', analytics.topPage.toString());
```

---

## Troubleshooting

### Site Not Loading

**Check hosting status:**
```javascript
const hosting = await sdk.api.query.bns.hostedWebsites('alice');
if (hosting.isNone) {
    console.log('❌ Hosting not activated');
} else {
    const data = hosting.unwrap();
    if (data.expiresAt < currentBlock) {
        console.log('❌ Hosting expired - renew payment');
    } else {
        console.log('✅ Hosting active');
    }
}
```

---

### Content Not Updating

**Clear browser cache:**
```bash
# Force reload: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)
```

**Purge CDN cache:**
```javascript
await sdk.api.tx.bns.purgeCdnCache('alice').signAndSend(alice);
```

---

### SSL Certificate Issues

**Check certificate validity:**
```javascript
const ssl = await sdk.api.query.bns.sslCertificates('alice');
const cert = ssl.unwrap();

if (cert.expiresAt < currentBlock) {
    console.log('❌ Certificate expired');
    // Auto-renewal should trigger within 24 hours
} else {
    console.log('✅ Certificate valid');
}
```

---

## Best Practices

**✅ Do:**
- Optimize images before upload (reduce file sizes)
- Use semantic HTML for better SEO
- Minify CSS/JS files
- Enable auto-renewal
- Monitor bandwidth usage (upgrade tier if needed)
- Version your content (easy rollback)

**❌ Don't:**
- Upload server-side code (won't execute)
- Exceed tier storage limits
- Use absolute URLs (breaks on DAG)
- Forget to test locally first
- Skip compression (wastes bandwidth)

---

## Migration from Traditional Hosting

**Steps:**
1. Export static files from current host
2. Remove server-side code (use static site generator if needed)
3. Upload to Pakit DAG
4. Activate BNS hosting
5. Update DNS records
6. Monitor for issues
7. Cancel old hosting

**Static Site Generators:**
- Hugo, Jekyll, Gatsby, Next.js (SSG mode)
- Export to static HTML/CSS/JS
- Deploy to BelizeChain

---

## Related Documentation

- [BNS Overview](./bns-overview.md)
- [Domain Registration](./bns-domain-registry.md)
- [Marketplace](./bns-marketplace.md)
- [Pricing Details](./bns-pricing.md)
- [API Reference](./bns-api-reference.md)
- [DAG Storage Architecture](../architecture/dag-storage-design.md)

---

## Support

- **Hosting Portal**: https://bns.belizechain.org/hosting
- **Discord**: #hosting-support
- **Status Page**: https://status.belizechain.org
- **Pakit CLI Docs**: https://docs.belizechain.org/pakit/cli
