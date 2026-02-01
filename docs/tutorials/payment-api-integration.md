# 💻 Payment API Integration Tutorial

**Level**: Intermediate (some coding experience helpful)  
**Time**: 1-2 hours  
**Prerequisites**: Basic JavaScript/Python knowledge, business account

---

## 📋 What You'll Learn

By the end of this tutorial, you'll be able to:

- ✅ Integrate BelizeChain payments into your website
- ✅ Generate payment requests programmatically
- ✅ Receive webhook notifications when payments arrive
- ✅ Verify payments securely
- ✅ Handle refunds and errors
- ✅ Build a complete checkout flow

---

## 🎯 Why Use the Payment API?

### **Advantages**

| Feature | API Integration | Manual QR Codes |
|---------|----------------|-----------------|
| **Automation** | ✅ Fully automated | ❌ Manual process |
| **E-commerce** | ✅ Perfect for online stores | ❌ In-person only |
| **Order Tracking** | ✅ Automatic linking | ❌ Manual reconciliation |
| **Custom UX** | ✅ Branded experience | ❌ Generic QR |
| **Webhooks** | ✅ Real-time notifications | ❌ Must poll |
| **Refunds** | ✅ Programmatic | ❌ Manual |

### **Perfect For**

- 🛒 E-commerce stores (online shopping)
- 📱 Mobile apps (in-app purchases)
- 💼 SaaS platforms (subscription billing)
- 🎟️ Booking systems (hotels, tours, flights)
- 🎮 Gaming platforms (in-game purchases)
- 📦 Marketplaces (multi-vendor)

---

## 🚀 Quick Start (15 Minutes)

Let's get your first API payment working!

### **Part 1: Get API Credentials** (5 minutes)

#### Step 1: Login to Business Dashboard

1. Go to https://business.belizechain.org
2. Login with your business account
3. Click **"Settings"** in the top menu
4. Click **"API Keys"** in the left sidebar

#### Step 2: Create API Key

```
┌─────────────────────────────────────────┐
│ 🔑 API Key Management                   │
├─────────────────────────────────────────┤
│                                         │
│ [+ Create New API Key]                  │
│                                         │
│ Existing Keys:                          │
│ • Production Key (created Oct 10, 2025) │
│   Last used: 2 hours ago                │
│   [Revoke] [View Details]               │
│                                         │
└─────────────────────────────────────────┘
```

1. Click **"Create New API Key"**
2. Enter a name: `"My Website Integration"`
3. Choose environment:
   - **Test Mode** (for development, uses testnet)
   - **Production Mode** (for real payments)
4. Select permissions:
   - ☑️ **Create payments** (required)
   - ☑️ **Read payments** (required)
   - ☑️ **Process refunds** (optional)
   - ☐ Manage products (optional)
   - ☐ Access customer data (optional)
5. Click **"Generate Key"**

#### Step 3: Save Your Credentials

You'll see two values - **SAVE THEM NOW** (they won't be shown again!):

```
┌─────────────────────────────────────────┐
│ ✅ API Key Created Successfully!        │
├─────────────────────────────────────────┤
│                                         │
│ Public Key (safe to share):             │
│ pk_live_a1b2c3d4e5f6g7h8i9j0            │
│                                         │
│ Secret Key (keep confidential):         │
│ sk_live_z9y8x7w6v5u4t3s2r1q0            │
│                                         │
│ ⚠️ Store the secret key securely!       │
│    It will NOT be shown again.          │
│                                         │
│ [Copy Both] [Download .env File]        │
│                                         │
└─────────────────────────────────────────┘
```

**CRITICAL**: 
- ✅ **Public key** = Can use in frontend (JavaScript)
- ⚠️ **Secret key** = NEVER expose (server-side only)

Click **"Download .env File"** to get:

```env
# BelizeChain API Credentials
BELIZECHAIN_PUBLIC_KEY=pk_live_a1b2c3d4e5f6g7h8i9j0
BELIZECHAIN_SECRET_KEY=sk_live_z9y8x7w6v5u4t3s2r1q0
BELIZECHAIN_WEBHOOK_SECRET=whsec_1a2b3c4d5e6f7g8h9i0j
```

---

### **Part 2: Install SDK** (2 minutes)

Choose your language:

#### **JavaScript/Node.js**

```bash
npm install @belizechain/payments
```

#### **Python**

```bash
pip install belizechain-sdk
```

#### **PHP**

```bash
composer require belizechain/payments-php
```

#### **Ruby**

```bash
gem install belizechain-payments
```

---

### **Part 3: Create Your First Payment** (8 minutes)

#### **JavaScript Example** (Node.js/Express)

```javascript
// server.js
const express = require('express');
const BelizeChain = require('@belizechain/payments');

const app = express();
const bc = new BelizeChain(process.env.BELIZECHAIN_SECRET_KEY);

// Endpoint: Create payment
app.post('/api/create-payment', async (req, res) => {
  try {
    const payment = await bc.payments.create({
      amount: '45.00',           // Amount in bBZD
      currency: 'bBZD',          // or 'DALLA'
      description: 'Lobster Dinner',
      orderId: 'ORDER-12345',    // Your internal order ID
      customerEmail: 'maria@example.com',
      redirectUrl: 'https://myrestaurant.com/success',
      webhookUrl: 'https://myrestaurant.com/webhook',
      metadata: {
        tableNumber: '5',
        serverName: 'Carlos'
      }
    });

    // Return payment URL to frontend
    res.json({
      paymentId: payment.id,
      paymentUrl: payment.url,  // Customer visits this to pay
      expiresAt: payment.expiresAt
    });
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

#### **Python Example** (Flask)

```python
# app.py
from flask import Flask, request, jsonify
from belizechain import BelizeChain
import os

app = Flask(__name__)
bc = BelizeChain(api_key=os.environ['BELIZECHAIN_SECRET_KEY'])

@app.route('/api/create-payment', methods=['POST'])
def create_payment():
    try:
        payment = bc.payments.create(
            amount='45.00',           # Amount in bBZD
            currency='bBZD',          # or 'DALLA'
            description='Lobster Dinner',
            order_id='ORDER-12345',   # Your internal order ID
            customer_email='maria@example.com',
            redirect_url='https://myrestaurant.com/success',
            webhook_url='https://myrestaurant.com/webhook',
            metadata={
                'table_number': '5',
                'server_name': 'Carlos'
            }
        )

        # Return payment URL to frontend
        return jsonify({
            'payment_id': payment.id,
            'payment_url': payment.url,  # Customer visits this to pay
            'expires_at': payment.expires_at
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 400

if __name__ == '__main__':
    app.run(port=3000)
```

#### **Test It!**

1. Start your server: `node server.js` or `python app.py`
2. Create a payment:

```bash
curl -X POST http://localhost:3000/api/create-payment \
  -H "Content-Type: application/json"
```

3. Response:

```json
{
  "paymentId": "pay_1A2B3C4D5E",
  "paymentUrl": "https://pay.belizechain.org/pay_1A2B3C4D5E",
  "expiresAt": "2025-10-14T15:30:00Z"
}
```

4. Open `paymentUrl` in browser → Customer pays → Done! 🎉

---

## 💰 Complete Integration Example

Let's build a complete e-commerce checkout!

### **Scenario**: Online Dive Shop

Ana runs an online dive shop. When customers checkout:
1. Create payment on server
2. Redirect customer to payment page
3. Receive webhook when paid
4. Fulfill order automatically

---

### **Step 1: Frontend (Checkout Button)**

```html
<!-- checkout.html -->
<!DOCTYPE html>
<html>
<head>
  <title>Checkout - Belize Dive Shop</title>
</head>
<body>
  <div class="cart">
    <h1>Your Cart</h1>
    <div class="item">
      <img src="dive-gear.jpg" />
      <span>Professional Dive Mask</span>
      <span>85.00 bBZD</span>
    </div>
    <div class="total">
      Total: <strong>85.00 bBZD</strong>
    </div>
    <button id="checkout-btn">Pay with BelizeChain</button>
  </div>

  <script>
    document.getElementById('checkout-btn').addEventListener('click', async () => {
      // Create payment on your server
      const response = await fetch('/api/create-payment', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          items: [
            { id: 'dive-mask-pro', quantity: 1, price: '85.00' }
          ],
          customerEmail: 'maria@example.com'
        })
      });

      const data = await response.json();

      if (data.paymentUrl) {
        // Redirect to BelizeChain payment page
        window.location.href = data.paymentUrl;
      } else {
        alert('Error creating payment: ' + data.error);
      }
    });
  </script>
</body>
</html>
```

---

### **Step 2: Backend (Create Payment)**

```javascript
// server.js
const express = require('express');
const BelizeChain = require('@belizechain/payments');
const app = express();

app.use(express.json());

const bc = new BelizeChain(process.env.BELIZECHAIN_SECRET_KEY);

app.post('/api/create-payment', async (req, res) => {
  const { items, customerEmail } = req.body;

  // Calculate total
  const total = items.reduce((sum, item) => 
    sum + (parseFloat(item.price) * item.quantity), 0
  );

  try {
    // Create payment
    const payment = await bc.payments.create({
      amount: total.toFixed(2),
      currency: 'bBZD',
      description: `Order: ${items.map(i => i.id).join(', ')}`,
      customerEmail: customerEmail,
      redirectUrl: 'https://belizediveshop.com/success',
      webhookUrl: 'https://belizediveshop.com/webhook',
      metadata: {
        items: JSON.stringify(items),
        source: 'website'
      }
    });

    // Save payment to database
    await db.orders.create({
      orderId: payment.metadata.orderId,
      paymentId: payment.id,
      customerEmail: customerEmail,
      items: items,
      total: total,
      status: 'pending'
    });

    res.json({
      paymentUrl: payment.url,
      paymentId: payment.id
    });
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

app.listen(3000);
```

---

### **Step 3: Webhook Handler (Receive Notifications)**

When the customer pays, BelizeChain sends a webhook to your server:

```javascript
// Webhook endpoint (receives payment notifications)
app.post('/webhook', express.raw({ type: 'application/json' }), async (req, res) => {
  const sig = req.headers['belizechain-signature'];
  const webhookSecret = process.env.BELIZECHAIN_WEBHOOK_SECRET;

  let event;

  try {
    // Verify webhook signature (security!)
    event = bc.webhooks.verify(req.body, sig, webhookSecret);
  } catch (err) {
    console.error('Webhook signature verification failed:', err.message);
    return res.status(400).send('Invalid signature');
  }

  // Handle the event
  switch (event.type) {
    case 'payment.succeeded':
      const payment = event.data;
      console.log('✅ Payment received:', payment.id);

      // Update order status
      await db.orders.update(
        { paymentId: payment.id },
        { status: 'paid', paidAt: new Date() }
      );

      // Send confirmation email
      await sendEmail({
        to: payment.customerEmail,
        subject: 'Order Confirmed!',
        body: `Your payment of ${payment.amount} bBZD was received.`
      });

      // Fulfill order (ship product, send download link, etc.)
      await fulfillOrder(payment.metadata.orderId);
      break;

    case 'payment.failed':
      console.log('❌ Payment failed:', event.data.id);
      await db.orders.update(
        { paymentId: event.data.id },
        { status: 'failed' }
      );
      break;

    case 'payment.refunded':
      console.log('🔄 Payment refunded:', event.data.id);
      await db.orders.update(
        { paymentId: event.data.id },
        { status: 'refunded' }
      );
      break;
  }

  res.json({ received: true });
});
```

---

### **Step 4: Success Page**

After payment, customer is redirected back to your site:

```html
<!-- success.html -->
<!DOCTYPE html>
<html>
<head>
  <title>Payment Successful!</title>
</head>
<body>
  <div class="success">
    <h1>✅ Payment Successful!</h1>
    <p>Thank you for your order!</p>
    <p>Order #<span id="order-id"></span></p>
    <p>You'll receive a confirmation email shortly.</p>
    <a href="/">Continue Shopping</a>
  </div>

  <script>
    // Get payment ID from URL
    const urlParams = new URLSearchParams(window.location.search);
    const paymentId = urlParams.get('payment_id');

    // Fetch order details
    fetch(`/api/order-status/${paymentId}`)
      .then(res => res.json())
      .then(order => {
        document.getElementById('order-id').textContent = order.orderId;
      });
  </script>
</body>
</html>
```

---

## 🔒 Security Best Practices

### **1. Always Verify Webhooks**

```javascript
// ✅ GOOD: Verify signature
const event = bc.webhooks.verify(req.body, signature, webhookSecret);

// ❌ BAD: Trust webhook without verification
const event = JSON.parse(req.body); // DANGEROUS!
```

**Why?** Anyone could send fake webhooks to your server!

---

### **2. Never Expose Secret Key**

```javascript
// ✅ GOOD: Secret key on server only
// server.js
const bc = new BelizeChain(process.env.BELIZECHAIN_SECRET_KEY);

// ❌ BAD: Secret key in frontend
// frontend.js
const bc = new BelizeChain('sk_live_z9y8x7...'); // EXPOSED TO PUBLIC!
```

**Why?** Anyone can steal your secret key from browser!

---

### **3. Validate Amounts on Server**

```javascript
// ✅ GOOD: Calculate amount on server
app.post('/create-payment', async (req, res) => {
  const items = req.body.items;
  const total = items.reduce((sum, item) => 
    sum + (getPrice(item.id) * item.quantity), 0  // Get price from DB
  );
  
  await bc.payments.create({ amount: total });
});

// ❌ BAD: Trust amount from client
app.post('/create-payment', async (req, res) => {
  const total = req.body.total;  // Client could send $0.01!
  await bc.payments.create({ amount: total });
});
```

**Why?** Client can modify JavaScript to send any amount!

---

### **4. Use HTTPS**

```javascript
// ✅ GOOD: HTTPS webhook URL
webhookUrl: 'https://mysite.com/webhook'

// ❌ BAD: HTTP webhook URL
webhookUrl: 'http://mysite.com/webhook'  // Can be intercepted!
```

**Why?** Webhooks contain sensitive data!

---

### **5. Idempotency Keys**

Prevent duplicate charges if customer clicks "Pay" twice:

```javascript
const payment = await bc.payments.create({
  amount: '85.00',
  currency: 'bBZD',
  idempotencyKey: `order-${orderId}`,  // Same order = same key
});
```

If called again with same key → returns original payment (no duplicate charge).

---

## 📊 Payment States & Handling

### **Payment Lifecycle**

```
created → pending → succeeded ✅
              ↓
            failed ❌
              ↓
          cancelled 🚫
```

### **State Descriptions**

| State | Description | Action |
|-------|-------------|--------|
| `created` | Payment link generated | Show link to customer |
| `pending` | Customer opened link | Wait for payment |
| `succeeded` | Payment received | Fulfill order |
| `failed` | Payment failed | Notify customer |
| `cancelled` | Customer cancelled | Update order status |
| `expired` | Link expired (30 min) | Create new payment |
| `refunded` | Payment refunded | Process return |

---

### **Handling Each State**

```javascript
// Check payment status
app.get('/api/payment/:id/status', async (req, res) => {
  const payment = await bc.payments.retrieve(req.params.id);

  switch (payment.status) {
    case 'succeeded':
      res.json({ 
        status: 'success',
        message: 'Payment received! Order processing...'
      });
      break;

    case 'pending':
      res.json({ 
        status: 'pending',
        message: 'Waiting for payment...'
      });
      break;

    case 'failed':
      res.json({ 
        status: 'failed',
        message: 'Payment failed. Please try again.',
        newPaymentUrl: await createNewPayment(payment.metadata.orderId)
      });
      break;

    case 'expired':
      res.json({ 
        status: 'expired',
        message: 'Payment link expired. Creating new one...',
        newPaymentUrl: await createNewPayment(payment.metadata.orderId)
      });
      break;
  }
});
```

---

## 💸 Refunds

### **Full Refund**

```javascript
// Refund entire payment
const refund = await bc.refunds.create({
  paymentId: 'pay_1A2B3C4D5E',
  reason: 'customer_request'
});

console.log('Refund status:', refund.status);  // 'succeeded'
console.log('Refund amount:', refund.amount);  // '85.00 bBZD'
```

### **Partial Refund**

```javascript
// Refund part of payment
const refund = await bc.refunds.create({
  paymentId: 'pay_1A2B3C4D5E',
  amount: '25.00',  // Only refund 25 bBZD
  reason: 'out_of_stock'
});
```

### **Refund Reasons**

- `customer_request` - Customer changed mind
- `duplicate` - Accidental duplicate payment
- `fraudulent` - Fraudulent payment
- `out_of_stock` - Product unavailable
- `other` - Custom reason

---

## 📱 Advanced Features

### **1. Payment Links (No Code Required)**

Generate payment link without SDK:

```bash
curl -X POST https://api.belizechain.org/v1/payments \
  -H "Authorization: Bearer sk_live_z9y8x7w6v5u4t3s2r1q0" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": "85.00",
    "currency": "bBZD",
    "description": "Dive Mask"
  }'
```

Response:

```json
{
  "id": "pay_1A2B3C4D5E",
  "url": "https://pay.belizechain.org/pay_1A2B3C4D5E",
  "status": "created"
}
```

Share the URL via email, SMS, WhatsApp!

---

### **2. Subscriptions (Recurring Billing)**

```javascript
// Create subscription
const subscription = await bc.subscriptions.create({
  customerId: 'cus_1A2B3C4D',
  plan: {
    amount: '29.99',
    currency: 'bBZD',
    interval: 'month',  // or 'week', 'year'
    intervalCount: 1     // Bill every 1 month
  },
  description: 'Premium Membership'
});

// Customer is charged 29.99 bBZD every month automatically
```

---

### **3. Customer Management**

```javascript
// Create customer profile
const customer = await bc.customers.create({
  email: 'maria@example.com',
  name: 'Maria Rodriguez',
  phone: '+501-622-1234',
  metadata: {
    loyaltyTier: 'gold',
    lifetimeValue: '1250.00'
  }
});

// Use customer ID for future payments
const payment = await bc.payments.create({
  customerId: customer.id,
  amount: '85.00',
  currency: 'bBZD'
});

// View customer payment history
const payments = await bc.customers.listPayments(customer.id);
```

---

### **4. Metadata (Custom Data)**

Attach unlimited custom data to payments:

```javascript
const payment = await bc.payments.create({
  amount: '85.00',
  currency: 'bBZD',
  metadata: {
    // Your custom fields (any data!)
    orderId: 'ORDER-12345',
    warehouseLocation: 'San Pedro',
    shippingMethod: 'express',
    giftWrap: true,
    customerNotes: 'Leave at front desk',
    internalSKU: 'DIVE-MASK-PRO-001',
    salesRep: 'carlos',
    campaignSource: 'facebook_ad'
  }
});

// Metadata is returned in webhooks
app.post('/webhook', async (req, res) => {
  const payment = req.body.data;
  console.log('Order ID:', payment.metadata.orderId);
  console.log('Warehouse:', payment.metadata.warehouseLocation);
  // ... use metadata to process order
});
```

---

### **5. Multiple Currencies**

```javascript
// Accept payment in DALLA or bBZD
const payment = await bc.payments.create({
  amount: '100.00',
  currency: 'DALLA',  // or 'bBZD'
  description: 'Tour Booking'
});

// Customer can pay with either currency
// Conversion happens automatically at current rate
```

---

### **6. Expiration Times**

```javascript
// Payment link expires in 15 minutes (default: 30 min)
const payment = await bc.payments.create({
  amount: '85.00',
  currency: 'bBZD',
  expiresIn: 900  // seconds (15 min)
});

console.log('Expires at:', payment.expiresAt);
// "2025-10-14T12:15:00Z"
```

---

## 🧪 Testing

### **Test Mode**

Use test API keys for development:

```javascript
// Test mode key (starts with pk_test_ or sk_test_)
const bc = new BelizeChain('sk_test_1a2b3c4d5e6f7g8h');

// Create test payment
const payment = await bc.payments.create({
  amount: '85.00',
  currency: 'bBZD'
});

// Use test credit cards
// 4242 4242 4242 4242 - Success
// 4000 0000 0000 0002 - Decline
// 4000 0000 0000 0127 - Insufficient funds
```

### **Test Cards**

| Card Number | Result |
|-------------|--------|
| `4242 4242 4242 4242` | ✅ Success |
| `4000 0000 0000 0002` | ❌ Card declined |
| `4000 0000 0000 0127` | ❌ Insufficient funds |
| `4000 0000 0000 0341` | ❌ Expired card |
| `4000 0000 0000 0069` | ❌ Fraud detected |

**Expiry**: Any future date (e.g., 12/28)  
**CVV**: Any 3 digits (e.g., 123)

---

### **Webhook Testing**

Test webhooks locally with ngrok:

```bash
# 1. Install ngrok
brew install ngrok  # macOS
# or download from ngrok.com

# 2. Start your server
node server.js

# 3. Expose localhost to internet
ngrok http 3000

# Output:
# Forwarding: https://abc123.ngrok.io -> http://localhost:3000

# 4. Use ngrok URL as webhook URL
webhookUrl: 'https://abc123.ngrok.io/webhook'
```

Now BelizeChain can send webhooks to your local dev server!

---

### **Trigger Test Webhooks**

```bash
# Trigger test webhook from dashboard
curl -X POST https://api.belizechain.org/v1/webhooks/test \
  -H "Authorization: Bearer sk_test_1a2b3c4d5e6f7g8h" \
  -d '{
    "event": "payment.succeeded",
    "payment_id": "pay_test_123"
  }'
```

Your webhook endpoint receives the test event!

---

## 📚 SDK Reference

### **JavaScript/Node.js**

```javascript
const BelizeChain = require('@belizechain/payments');
const bc = new BelizeChain('sk_live_...');

// Payments
await bc.payments.create({ ... });
await bc.payments.retrieve('pay_123');
await bc.payments.list({ limit: 10 });
await bc.payments.cancel('pay_123');

// Refunds
await bc.refunds.create({ paymentId: 'pay_123' });
await bc.refunds.retrieve('ref_123');
await bc.refunds.list({ paymentId: 'pay_123' });

// Customers
await bc.customers.create({ email: 'maria@example.com' });
await bc.customers.retrieve('cus_123');
await bc.customers.update('cus_123', { name: 'Maria Rodriguez' });
await bc.customers.listPayments('cus_123');

// Webhooks
bc.webhooks.verify(body, signature, secret);

// Subscriptions
await bc.subscriptions.create({ customerId: 'cus_123', plan: {...} });
await bc.subscriptions.retrieve('sub_123');
await bc.subscriptions.cancel('sub_123');
```

---

### **Python**

```python
from belizechain import BelizeChain
bc = BelizeChain(api_key='sk_live_...')

# Payments
bc.payments.create(amount='85.00', currency='bBZD')
bc.payments.retrieve('pay_123')
bc.payments.list(limit=10)
bc.payments.cancel('pay_123')

# Refunds
bc.refunds.create(payment_id='pay_123')
bc.refunds.retrieve('ref_123')
bc.refunds.list(payment_id='pay_123')

# Customers
bc.customers.create(email='maria@example.com')
bc.customers.retrieve('cus_123')
bc.customers.update('cus_123', name='Maria Rodriguez')
bc.customers.list_payments('cus_123')

# Webhooks
bc.webhooks.verify(body, signature, secret)

# Subscriptions
bc.subscriptions.create(customer_id='cus_123', plan={...})
bc.subscriptions.retrieve('sub_123')
bc.subscriptions.cancel('sub_123')
```

---

## 🐛 Error Handling

### **Common Errors**

```javascript
try {
  const payment = await bc.payments.create({
    amount: '85.00',
    currency: 'bBZD'
  });
} catch (error) {
  if (error.type === 'authentication_error') {
    // Invalid API key
    console.error('Invalid API key');
  } else if (error.type === 'invalid_request_error') {
    // Missing required parameter
    console.error('Invalid request:', error.message);
  } else if (error.type === 'rate_limit_error') {
    // Too many requests
    console.error('Rate limit exceeded, retry in', error.retryAfter, 'seconds');
  } else if (error.type === 'api_error') {
    // BelizeChain server error
    console.error('API error:', error.message);
  } else {
    // Other error
    console.error('Unexpected error:', error);
  }
}
```

### **Error Types**

| Type | Cause | Solution |
|------|-------|----------|
| `authentication_error` | Invalid API key | Check credentials |
| `invalid_request_error` | Missing/invalid parameter | Check request format |
| `rate_limit_error` | Too many requests | Implement retry logic |
| `payment_error` | Payment declined | Notify customer |
| `api_error` | Server error | Retry request |

---

### **Retry Logic**

```javascript
async function createPaymentWithRetry(data, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await bc.payments.create(data);
    } catch (error) {
      if (error.type === 'rate_limit_error' && i < maxRetries - 1) {
        // Wait and retry
        await sleep(error.retryAfter * 1000);
        continue;
      }
      throw error;  // Give up
    }
  }
}
```

---

## 📊 Dashboard & Analytics

View real-time payment data:

### **Business Dashboard**

https://business.belizechain.org

```
┌─────────────────────────────────────────┐
│ 📊 Payment Analytics                    │
├─────────────────────────────────────────┤
│                                         │
│ Today:          1,250.00 bBZD (28)      │
│ This Week:      6,890.00 bBZD (156)     │
│ This Month:    24,560.00 bBZD (542)     │
│                                         │
│ Success Rate:   98.2%                   │
│ Avg Amount:     45.30 bBZD              │
│ Top Product:    Dive Mask Pro           │
│                                         │
│ Recent Payments:                        │
│ • 85.00 bBZD - maria@example.com (2m)   │
│ • 125.00 bBZD - carlos@example.com (8m) │
│ • 45.00 bBZD - ana@example.com (15m)    │
│                                         │
│ [Export CSV] [View Reports]             │
│                                         │
└─────────────────────────────────────────┘
```

---

## 🎯 Real-World Examples

### **Example 1: E-commerce Store**

**Belizean Crafts Shop** (sells handmade goods):

- Integrated API in 2 hours
- Reduced cart abandonment from 68% to 22%
- Saved $1,200/month vs Shopify Payments
- Revenue increased 34% (crypto-friendly customers)

**Code**: https://github.com/belizechain/examples/ecommerce

---

### **Example 2: Hotel Booking**

**Paradise Resort** (20-room beachfront hotel):

- Integrated API with existing booking system
- Customers pay in DALLA or bBZD
- Automatic confirmation emails
- No-show rate dropped from 15% to 3% (pre-payment)

**Code**: https://github.com/belizechain/examples/hotel-booking

---

### **Example 3: SaaS Platform**

**BelizeCloud** (cloud hosting service):

- Monthly subscription billing
- Automatic payment retries
- Usage-based charges (overage fees)
- 99.8% payment success rate

**Code**: https://github.com/belizechain/examples/saas-subscriptions

---

### **Example 4: Mobile App**

**TourGuide Belize** (tourism app):

- In-app tour bookings
- React Native integration
- Push notifications on payment
- Average transaction time: 8 seconds

**Code**: https://github.com/belizechain/examples/mobile-app

---

## 🔥 Advanced Patterns

### **1. Split Payments (Marketplace)**

Split payment between multiple vendors:

```javascript
const payment = await bc.payments.create({
  amount: '100.00',
  currency: 'bBZD',
  description: 'Marketplace Order',
  splits: [
    { account: 'acct_vendor1', amount: '70.00' },  // Vendor gets 70
    { account: 'acct_vendor2', amount: '20.00' },  // Vendor gets 20
    { account: 'acct_platform', amount: '10.00' }  // Platform fee
  ]
});
```

---

### **2. Delayed Capture (Authorization Hold)**

Authorize payment now, capture later:

```javascript
// 1. Authorize payment (hold funds)
const payment = await bc.payments.create({
  amount: '200.00',
  currency: 'bBZD',
  captureMethod: 'manual'  // Don't charge immediately
});

// Payment status: 'authorized' (funds on hold)

// 2. Later: Capture payment (actually charge)
await bc.payments.capture(payment.id);

// Or: Release hold without charging
await bc.payments.cancel(payment.id);
```

**Use case**: Hotel reservations (authorize at booking, capture at check-in).

---

### **3. Dynamic Pricing**

Calculate price based on user/time/demand:

```javascript
function calculatePrice(userId, timeOfDay) {
  let basePrice = 50.00;
  
  // Tourism bonus (5% discount)
  if (isTourismBusiness(userId)) {
    basePrice *= 0.95;
  }
  
  // Off-peak discount (10% discount 2-5pm)
  const hour = new Date().getHours();
  if (hour >= 14 && hour <= 17) {
    basePrice *= 0.90;
  }
  
  return basePrice.toFixed(2);
}

const payment = await bc.payments.create({
  amount: calculatePrice(req.user.id, new Date()),
  currency: 'bBZD'
});
```

---

### **4. Payment Intents (Complex Flows)**

For multi-step checkout processes:

```javascript
// 1. Create payment intent (not charged yet)
const intent = await bc.paymentIntents.create({
  amount: '85.00',
  currency: 'bBZD'
});

// 2. Customer adds shipping address
await bc.paymentIntents.update(intent.id, {
  shipping: {
    address: '123 Main St, San Pedro',
    method: 'express'
  }
});

// 3. Customer applies coupon code
await bc.paymentIntents.update(intent.id, {
  amount: '76.50',  // 10% discount
  metadata: { coupon: 'SAVE10' }
});

// 4. Finally: Confirm and charge
await bc.paymentIntents.confirm(intent.id);
```

---

## 🚨 Troubleshooting

### **Problem: Webhook not receiving events**

**Symptoms**: Payment succeeds but webhook never called.

**Solutions**:

1. **Check webhook URL**:
   ```javascript
   // ✅ GOOD: HTTPS with public domain
   webhookUrl: 'https://mysite.com/webhook'
   
   // ❌ BAD: localhost (not accessible from internet)
   webhookUrl: 'http://localhost:3000/webhook'
   ```

2. **Verify signature**:
   ```javascript
   app.post('/webhook', (req, res) => {
     const sig = req.headers['belizechain-signature'];
     try {
       const event = bc.webhooks.verify(req.body, sig, webhookSecret);
       // Process event
     } catch (err) {
       console.error('Invalid signature:', err.message);
       return res.status(400).send('Invalid signature');
     }
   });
   ```

3. **Check webhook logs** in dashboard:
   - https://business.belizechain.org/webhooks
   - View delivery attempts, errors, response codes

4. **Use ngrok for local testing** (see Testing section).

---

### **Problem: "Invalid API key" error**

**Symptoms**: `authentication_error: Invalid API key`.

**Solutions**:

1. **Check key format**:
   - Test key: `sk_test_...`
   - Live key: `sk_live_...`

2. **Check environment**:
   ```javascript
   // Using test key but trying to charge real cards?
   const bc = new BelizeChain('sk_test_...');  // Test mode
   ```

3. **Check .env file**:
   ```bash
   # .env
   BELIZECHAIN_SECRET_KEY=sk_live_z9y8x7w6v5u4t3s2r1q0
   ```

4. **Regenerate key** in dashboard if compromised.

---

### **Problem: Payment succeeded but order not fulfilled**

**Symptoms**: Customer paid, but order status still "pending".

**Solutions**:

1. **Check webhook was received**:
   - Look in dashboard webhook logs
   - Check your server logs for `/webhook` endpoint

2. **Verify payment status**:
   ```javascript
   const payment = await bc.payments.retrieve('pay_123');
   console.log('Status:', payment.status);  // Should be 'succeeded'
   ```

3. **Check database update**:
   ```javascript
   app.post('/webhook', async (req, res) => {
     const payment = req.body.data;
     
     // Add logging
     console.log('Updating order:', payment.metadata.orderId);
     await db.orders.update(...);
     console.log('Order updated successfully');
     
     res.json({ received: true });
   });
   ```

4. **Manual fulfillment**:
   ```javascript
   // Manually check and fulfill orders
   const payment = await bc.payments.retrieve('pay_123');
   if (payment.status === 'succeeded') {
     await fulfillOrder(payment.metadata.orderId);
   }
   ```

---

### **Problem: Rate limit errors**

**Symptoms**: `rate_limit_error: Too many requests`.

**Solutions**:

1. **Implement retry logic with backoff**:
   ```javascript
   async function makeRequestWithRetry(fn, maxRetries = 3) {
     for (let i = 0; i < maxRetries; i++) {
       try {
         return await fn();
       } catch (error) {
         if (error.type === 'rate_limit_error') {
           const delay = Math.pow(2, i) * 1000;  // Exponential backoff
           await sleep(delay);
           continue;
         }
         throw error;
       }
     }
   }
   ```

2. **Cache results**:
   ```javascript
   // Cache payment status for 30 seconds
   const cache = new Map();
   
   async function getPaymentStatus(id) {
     if (cache.has(id)) {
       return cache.get(id);
     }
     const payment = await bc.payments.retrieve(id);
     cache.set(id, payment);
     setTimeout(() => cache.delete(id), 30000);
     return payment;
   }
   ```

3. **Batch requests**:
   ```javascript
   // ❌ BAD: 100 individual requests
   for (const id of paymentIds) {
     await bc.payments.retrieve(id);
   }
   
   // ✅ GOOD: 1 batch request
   const payments = await bc.payments.list({ ids: paymentIds });
   ```

---

## 📖 Additional Resources

### **Documentation**
- API Reference: https://docs.belizechain.org/api
- SDK Docs (JS): https://docs.belizechain.org/sdk/javascript
- SDK Docs (Python): https://docs.belizechain.org/sdk/python
- Webhook Guide: https://docs.belizechain.org/webhooks

### **Code Examples**
- GitHub: https://github.com/belizechain/examples
- E-commerce Example: https://github.com/belizechain/examples/ecommerce
- SaaS Example: https://github.com/belizechain/examples/saas
- Mobile App Example: https://github.com/belizechain/examples/mobile

### **Tools**
- Postman Collection: https://postman.com/belizechain
- API Status: https://status.belizechain.org
- Sandbox Environment: https://sandbox.belizechain.org

### **Support**
- Email: developers@belizechain.org
- Developer Forum: https://forum.belizechain.org/developers
- Discord: https://discord.gg/belizechain
- Office Hours: Tuesdays 2-4pm (Belize time)

---

## ✅ Checklist

### **Setup**
- [ ] Get API credentials from dashboard
- [ ] Save secret key securely (environment variables)
- [ ] Install SDK in your project
- [ ] Test with test API key first

### **Integration**
- [ ] Create payment endpoint on server
- [ ] Add checkout button in frontend
- [ ] Implement webhook handler
- [ ] Verify webhook signatures
- [ ] Create success/failure pages

### **Security**
- [ ] Never expose secret key in frontend
- [ ] Always validate amounts on server
- [ ] Use HTTPS for webhook URLs
- [ ] Implement idempotency keys
- [ ] Log all payment events

### **Testing**
- [ ] Test with test API key
- [ ] Test all payment states (success, fail, cancel)
- [ ] Test webhooks with ngrok
- [ ] Test error handling
- [ ] Test on mobile devices

### **Production**
- [ ] Switch to live API key
- [ ] Update webhook URL to production domain
- [ ] Monitor webhook delivery
- [ ] Set up error alerting
- [ ] Document API integration for team

---

## 🎉 You're Ready!

You now know how to:

- ✅ Integrate BelizeChain payments into any website or app
- ✅ Accept payments programmatically
- ✅ Receive real-time webhook notifications
- ✅ Handle all payment states securely
- ✅ Process refunds
- ✅ Build complete checkout flows

**Start building today!** 🚀

---

**Next Steps**:
- 📱 [Build a Mobile App Integration](./mobile-app-integration.md)
- 🔄 [Setup Subscription Billing](./subscription-billing.md)
- 🛒 [E-commerce Best Practices](../developer-guides/ecommerce-patterns.md)

**Need help?** Email developers@belizechain.org or visit our forum!
