# 🔒 Security Guide

**Comprehensive security hardening for BelizeChain nodes**

---

## 🎯 Security Principles

1. **Defense in Depth**: Multiple layers of security
2. **Least Privilege**: Minimal necessary permissions
3. **Separation of Concerns**: Isolate components
4. **Regular Updates**: Stay current with patches
5. **Monitoring**: Detect and respond to threats

---

## ⚡ Quick Security Checklist

**Critical** (do these first!):
- [ ] Change default passwords
- [ ] Configure firewall (UFW)
- [ ] Disable root SSH login
- [ ] Set up SSH keys (disable password auth)
- [ ] Enable automatic security updates
- [ ] Backup validator keys

**Important**:
- [ ] Configure fail2ban
- [ ] Set up HTTPS for RPC (if public)
- [ ] Enable two-factor authentication
- [ ] Regular security audits
- [ ] Monitor access logs

---

## 🔥 Firewall Configuration

### **UFW (Uncomplicated Firewall)**

```bash
# Install UFW
sudo apt install ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (IMPORTANT: Do this first!)
sudo ufw allow 22/tcp comment 'SSH'

# Allow P2P (required for blockchain)
sudo ufw allow 30333/tcp comment 'BelizeChain P2P'

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

**Output**:
```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere                   # SSH
30333/tcp                  ALLOW       Anywhere                   # BelizeChain P2P
```

---

### **For RPC Nodes** (public API):

```bash
# Allow RPC only from specific IPs
sudo ufw allow from 203.0.113.0/24 to any port 9933 comment 'RPC whitelist'
sudo ufw allow from 203.0.113.0/24 to any port 9944 comment 'WebSocket whitelist'

# OR use reverse proxy (recommended)
# See "Reverse Proxy" section below
```

---

### **For Validators** (NO public RPC):

```bash
# Only allow P2P and SSH
# Do NOT open 9933, 9944 to public!

# Allow Prometheus from monitoring server
sudo ufw allow from 10.0.1.100 to any port 9615 comment 'Prometheus'
```

---

## 🔐 SSH Hardening

### **Step 1: Create SSH Key Pair** (on local machine)

```bash
# Generate key (use strong passphrase!)
ssh-keygen -t ed25519 -C "belizechain-validator"

# Copy to server
ssh-copy-id -i ~/.ssh/id_ed25519.pub user@server
```

---

### **Step 2: Configure SSH Server**

Edit `/etc/ssh/sshd_config`:

```bash
sudo nano /etc/ssh/sshd_config
```

**Secure settings**:

```ini
# Disable root login
PermitRootLogin no

# Disable password authentication (use keys only)
PasswordAuthentication no
PubkeyAuthentication yes

# Disable empty passwords
PermitEmptyPasswords no

# Limit authentication attempts
MaxAuthTries 3

# Disconnect idle sessions (5 minutes)
ClientAliveInterval 300
ClientAliveCountMax 2

# Only allow specific users
AllowUsers belizechain admin

# Use strong ciphers
Ciphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com
MACs hmac-sha2-512-etm@openssh.com,hmac-sha2-256-etm@openssh.com
KexAlgorithms curve25519-sha256,curve25519-sha256@libssh.org

# Disable X11 forwarding
X11Forwarding no

# Change SSH port (optional, security through obscurity)
# Port 2222
```

**Restart SSH**:

```bash
# Test config first!
sudo sshd -t

# If no errors:
sudo systemctl restart sshd
```

**⚠️ Important**: Test SSH in new session before closing current session!

---

## 🛡️ Fail2Ban (Brute Force Protection)

**Install and configure**:

```bash
# Install
sudo apt install fail2ban

# Create local config
sudo cp /etc/fail2ban/jail.conf /etc/fail2ban/jail.local
sudo nano /etc/fail2ban/jail.local
```

**jail.local**:

```ini
[DEFAULT]
# Ban for 1 hour
bantime = 3600

# Look back 10 minutes
findtime = 600

# Ban after 5 failed attempts
maxretry = 5

# Ignore local IPs
ignoreip = 127.0.0.1/8 ::1 10.0.0.0/8

[sshd]
enabled = true
port = ssh
logpath = /var/log/auth.log
maxretry = 3
bantime = 86400  # 24 hours
```

**Start fail2ban**:

```bash
sudo systemctl enable fail2ban
sudo systemctl start fail2ban

# Check status
sudo fail2ban-client status sshd
```

---

## 🔒 Key Management

### **Validator Keys (CRITICAL)**

**⚠️ NEVER store validator keys on multiple machines!**

**Best practices**:

1. **Generate on secure machine** (air-gapped if possible)
2. **Encrypt before transfer**
3. **Store backups in multiple secure locations**
4. **Never commit to git**
5. **Use hardware security module (HSM)** for production

---

### **Backup Validator Keys**

```bash
# Backup keystore
sudo tar -czf validator-keys-$(date +%Y%m%d).tar.gz \
  /var/lib/belizechain/chains/belizechain/keystore/

# Encrypt backup
gpg --symmetric --cipher-algo AES256 \
  validator-keys-$(date +%Y%m%d).tar.gz

# Result: validator-keys-20251014.tar.gz.gpg
# Store this in 3+ locations:
# 1. Encrypted USB drive (offline)
# 2. Encrypted cloud storage (AWS S3, Google Drive)
# 3. Hardware security module (YubiKey, Ledger)
```

---

### **Restore Keys** (emergency)

```bash
# Decrypt backup
gpg --decrypt validator-keys-20251014.tar.gz.gpg > validator-keys.tar.gz

# Stop node
sudo systemctl stop belizechain

# Restore keys
sudo tar -xzf validator-keys.tar.gz -C /

# Fix permissions
sudo chown -R belizechain:belizechain /var/lib/belizechain

# Start node
sudo systemctl start belizechain
```

---

## 🌐 Reverse Proxy (RPC Nodes)

### **Nginx with SSL**

**Install Nginx and Certbot**:

```bash
sudo apt install nginx certbot python3-certbot-nginx
```

---

**Configure Nginx** (`/etc/nginx/sites-available/belizechain-rpc`):

```nginx
# Rate limiting
limit_req_zone $binary_remote_addr zone=rpc_limit:10m rate=10r/s;
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

upstream belizechain_http {
    server 127.0.0.1:9933;
}

upstream belizechain_ws {
    server 127.0.0.1:9944;
}

# HTTP redirect to HTTPS
server {
    listen 80;
    server_name rpc.belizechain.org;
    return 301 https://$server_name$request_uri;
}

# HTTPS RPC server
server {
    listen 443 ssl http2;
    server_name rpc.belizechain.org;

    # SSL certificates (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/rpc.belizechain.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.belizechain.org/privkey.pem;

    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Rate limiting
    limit_req zone=rpc_limit burst=20 nodelay;
    limit_conn conn_limit 10;

    # HTTP RPC
    location / {
        proxy_pass http://belizechain_http;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Timeout
        proxy_read_timeout 60s;
        proxy_send_timeout 60s;
    }

    # WebSocket RPC
    location /ws {
        proxy_pass http://belizechain_ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # Longer timeout for WebSocket
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}
```

**Enable site**:

```bash
sudo ln -s /etc/nginx/sites-available/belizechain-rpc /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

**Get SSL certificate**:

```bash
sudo certbot --nginx -d rpc.belizechain.org

# Auto-renewal (certbot installs cron job automatically)
sudo certbot renew --dry-run
```

---

## 🔐 System Hardening

### **Automatic Security Updates**

```bash
# Install unattended-upgrades
sudo apt install unattended-upgrades

# Configure
sudo dpkg-reconfigure -plow unattended-upgrades

# Edit config
sudo nano /etc/apt/apt.conf.d/50unattended-upgrades
```

**50unattended-upgrades**:

```
Unattended-Upgrade::Allowed-Origins {
    "${distro_id}:${distro_codename}-security";
    "${distro_id}ESMApps:${distro_codename}-apps-security";
};

Unattended-Upgrade::AutoFixInterruptedDpkg "true";
Unattended-Upgrade::Remove-Unused-Dependencies "true";
Unattended-Upgrade::Automatic-Reboot "false";  # Manual reboot for validators
```

---

### **Disable Unnecessary Services**

```bash
# List running services
systemctl list-units --type=service --state=running

# Disable unnecessary services
sudo systemctl disable bluetooth
sudo systemctl disable cups  # Printing
sudo systemctl disable avahi-daemon  # Network discovery
```

---

### **Kernel Hardening** (sysctl)

Edit `/etc/sysctl.conf`:

```bash
sudo nano /etc/sysctl.conf
```

**Security settings**:

```ini
# IP forwarding (disable if not routing)
net.ipv4.ip_forward = 0

# Ignore ICMP redirects
net.ipv4.conf.all.accept_redirects = 0
net.ipv6.conf.all.accept_redirects = 0

# Ignore source-routed packets
net.ipv4.conf.all.accept_source_route = 0
net.ipv6.conf.all.accept_source_route = 0

# Log martians (packets with impossible addresses)
net.ipv4.conf.all.log_martians = 1

# Ignore ICMP ping
net.ipv4.icmp_echo_ignore_all = 1

# SYN flood protection
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_max_syn_backlog = 2048
net.ipv4.tcp_synack_retries = 2

# Increase connection tracking
net.netfilter.nf_conntrack_max = 1000000
```

**Apply changes**:

```bash
sudo sysctl -p
```

---

### **File System Security**

```bash
# Set restrictive permissions on keystore
sudo chmod 700 /var/lib/belizechain/chains/belizechain/keystore
sudo chown -R belizechain:belizechain /var/lib/belizechain

# Protect config files
sudo chmod 600 /etc/belizechain/config.toml
sudo chown root:root /etc/belizechain/config.toml
```

---

## 🕵️ Intrusion Detection

### **AIDE (File Integrity Monitoring)**

```bash
# Install AIDE
sudo apt install aide

# Initialize database
sudo aideinit

# Move database
sudo mv /var/lib/aide/aide.db.new /var/lib/aide/aide.db

# Run check
sudo aide --check

# Schedule daily checks
echo "0 3 * * * root /usr/bin/aide --check | mail -s 'AIDE Report' admin@example.com" | sudo tee -a /etc/crontab
```

---

## 🔍 Security Monitoring

### **Monitor Authentication Logs**

```bash
# Watch authentication attempts
sudo tail -f /var/log/auth.log

# Check failed SSH attempts
sudo grep "Failed password" /var/log/auth.log | tail -20

# Check successful logins
sudo grep "Accepted publickey" /var/log/auth.log | tail -20
```

---

### **Monitor Firewall Logs**

```bash
# Enable UFW logging
sudo ufw logging on

# Check logs
sudo tail -f /var/log/ufw.log
```

---

### **Rootkit Detection**

```bash
# Install rkhunter
sudo apt install rkhunter

# Update database
sudo rkhunter --update

# Run scan
sudo rkhunter --check --skip-keypress

# Schedule weekly scans
echo "0 3 * * 0 root /usr/bin/rkhunter --check --skip-keypress --report-warnings-only | mail -s 'rkhunter Report' admin@example.com" | sudo tee -a /etc/crontab
```

---

## 🌐 DDoS Protection

### **Rate Limiting (Nginx)**

Already configured in reverse proxy above:
- 10 requests/second per IP
- Max 10 concurrent connections per IP

---

### **CloudFlare** (for RPC nodes)

**Advantages**:
- Free DDoS protection
- Global CDN
- Rate limiting
- Bot protection

**Setup**:
1. Add domain to CloudFlare
2. Point DNS to your server
3. Enable "I'm Under Attack" mode (if needed)
4. Configure rate limiting rules

---

### **Fail2Ban for RPC Abuse**

**Create custom filter** (`/etc/fail2ban/filter.d/belizechain-rpc.conf`):

```ini
[Definition]
failregex = ^<HOST> .* "POST / HTTP.*" (4|5)\d\d
ignoreregex =
```

**Add jail** (`/etc/fail2ban/jail.local`):

```ini
[belizechain-rpc]
enabled = true
port = http,https
logpath = /var/log/nginx/access.log
maxretry = 100
findtime = 60
bantime = 3600
```

---

## 🚨 Incident Response

### **Suspected Compromise**

**Immediate actions**:

1. **Isolate node**:
   ```bash
   sudo ufw default deny incoming
   sudo ufw default deny outgoing
   ```

2. **Stop validator** (prevent slashing):
   ```bash
   sudo systemctl stop belizechain
   ```

3. **Check for unauthorized access**:
   ```bash
   sudo last -f /var/log/wtmp | head -20
   sudo grep "Accepted" /var/log/auth.log
   ```

4. **Check running processes**:
   ```bash
   ps aux | grep -v "\[" | sort -rnk 3,3 | head
   ```

5. **Check network connections**:
   ```bash
   sudo netstat -tunap
   ```

6. **Preserve evidence**:
   ```bash
   sudo mkdir /tmp/incident-$(date +%Y%m%d-%H%M)
   sudo cp /var/log/auth.log /tmp/incident-*/
   sudo cp /var/log/syslog /tmp/incident-*/
   ```

---

### **Recovery Steps**

1. **Restore from clean backup**
2. **Regenerate all keys**
3. **Update all software**
4. **Review and harden security**
5. **Monitor closely for 48 hours**

---

## ✅ Security Audit Checklist

**Monthly audit**:

### **System Security**:
- [ ] All software up to date
- [ ] No unnecessary services running
- [ ] Firewall rules correct
- [ ] SSH configuration hardened
- [ ] Fail2ban working
- [ ] Automatic updates enabled

### **Node Security**:
- [ ] Validator keys backed up (3+ locations)
- [ ] RPC not exposed (validators)
- [ ] Session keys rotated (if needed)
- [ ] Monitoring alerts working
- [ ] Logs reviewed for anomalies

### **Network Security**:
- [ ] SSL certificates valid (if RPC)
- [ ] Rate limiting working
- [ ] DDoS protection active (if public)
- [ ] No open ports except necessary

### **Access Control**:
- [ ] SSH keys only (no passwords)
- [ ] No root login
- [ ] Minimal user accounts
- [ ] Strong passphrases
- [ ] Two-factor authentication (if applicable)

---

## 🎓 Security Best Practices

### **Operational Security (OpSec)**

1. **Separation of duties**:
   - Operations team (daily maintenance)
   - Security team (access control)
   - Never one person with all access

2. **Multi-signature operations**:
   - Treasury withdrawals require 2+ signatures
   - Critical updates require approval

3. **Regular drills**:
   - Practice key recovery
   - Test backup restoration
   - Simulate compromise scenarios

---

### **Physical Security** (for validators)

- Lock server room
- Video surveillance
- Alarm system
- Backup power (UPS)
- Fire suppression
- Environmental monitoring (temperature, humidity)

---

### **Social Engineering Defense**

- Never share keys/seeds with anyone
- Verify all requests (especially urgent ones)
- Be suspicious of support requests
- Official team will NEVER ask for keys
- Use official communication channels only

---

## 🆘 Security Resources

**Report vulnerabilities**:
- Email: security@belizechain.org
- Bug bounty: https://belizechain.org/bug-bounty

**Security updates**:
- Subscribe: https://belizechain.org/security-advisories
- Discord: #security-announcements

**Community**:
- Security Discord: https://discord.gg/belizechain-security
- Weekly security calls (Fridays 3pm UTC)

---

## 🚀 Next Steps

1. **Complete checklist** above
2. **Test backup restoration** (quarterly)
3. **Review logs** (daily)
4. **Update software** (monthly)
5. **Security audit** (quarterly)

**Remember**: Security is a process, not a destination! 🔒

---

**Questions?** Join [Security Discord](https://discord.gg/belizechain-security) for expert help! 🔒🇧🇿
