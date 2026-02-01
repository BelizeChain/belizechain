# 🧪 Testing Guide

**Comprehensive testing strategies for BelizeChain applications**

---

## 📋 Overview

**Testing Levels**:
1. **Unit Tests**: Test individual functions/components
2. **Integration Tests**: Test pallet interactions
3. **End-to-End Tests**: Test complete user flows
4. **Benchmarking**: Measure performance

**Time Required**: 1-2 hours to learn  
**Skill Level**: Intermediate  
**Prerequisites**: [Development environment setup](setup-environment.md)

---

## 🚀 Quick Start

### **Run All Tests**

```bash
# Rust tests (blockchain)
cargo test --workspace

# JavaScript tests (UI)
cd ui && pnpm test

# Python tests (AI/Quantum)
cd nawal && pytest
```

### DEX Dev Seeding & Oracle Rate (Integration)

```bash
# Start a dev node with default DEX pairs and auto-seeded WUSDC/BBZD liquidity
./target/release/belizechain-node --dev --tmp

# In another terminal, run guarded swap tests
pytest -q tests/integration/belizex/test_wusdc_bbzd_guard.py -k TestWusdcBbzdOracleGuard

# To set USD/BZD oracle rate used by the guard (example 2.0)
# Tests include a sudo helper that calls Oracle.update_exchange_rate
```

### One-shot helper for dev swaps

```bash
# Optional: use the helper to set rate, KYC, add liquidity, and trade
python scripts/dex_dev_helper.py all --rate 2.0 --ws ws://localhost:9944

# Or run individual steps
python scripts/dex_dev_helper.py set-rate --rate 2.0
python scripts/dex_dev_helper.py verify-kyc
python scripts/dex_dev_helper.py add-liquidity --base 10000000000000 --quote 20000000000000
python scripts/dex_dev_helper.py trade --amount 1000000000000
```

### **Run Specific Tests**

```bash
# Single pallet
cargo test -p pallet-belize-economy

# Single test
cargo test -p pallet-belize-economy create_account_works

# With output
cargo test -p pallet-belize-economy -- --nocapture
```

---

## 🦀 Rust Unit Tests

### **Basic Test Structure**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop};
    
    #[test]
    fn test_name() {
        // Arrange: Set up test data
        let value = 42;
        
        // Act: Execute function
        let result = some_function(value);
        
        // Assert: Verify result
        assert_eq!(result, expected);
    }
}
```

### **Pallet Test Setup**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop, parameter_types};
    use sp_core::H256;
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    // Configure mock runtime
    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Balances: pallet_balances,
            MyPallet: crate,
        }
    );

    // System config
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
    }

    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Nonce = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Block = Block;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = pallet_balances::AccountData<u128>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
    }

    // Balances config
    parameter_types! {
        pub const ExistentialDeposit: u128 = 1;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ();
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type Balance = u128;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = ();
        type MaxFreezes = ();
        type RuntimeHoldReason = ();
        type MaxHolds = ();
    }

    // Your pallet config
    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
    }

    // Test helper: Create test environment
    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();

        // Fund test accounts
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 1000),  // Alice: 1000 tokens
                (2, 500),   // Bob: 500 tokens
                (3, 100),   // Charlie: 100 tokens
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }

    // Tests start here...
    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Test implementation
        });
    }
}
```

### **Common Test Patterns**

#### **Pattern 1: Happy Path**

```rust
#[test]
fn create_account_works() {
    new_test_ext().execute_with(|| {
        // Create account
        assert_ok!(MyPallet::create_account(
            RuntimeOrigin::signed(1),
            AccountType::Citizen
        ));

        // Verify account exists
        assert!(MyPallet::accounts(1).is_some());

        // Verify account data
        let account = MyPallet::accounts(1).unwrap();
        assert_eq!(account.account_type, AccountType::Citizen);

        // Verify event emitted
        System::assert_has_event(RuntimeEvent::MyPallet(
            Event::AccountCreated { who: 1 }
        ));
    });
}
```

#### **Pattern 2: Error Cases**

```rust
#[test]
fn insufficient_balance_error() {
    new_test_ext().execute_with(|| {
        // Try to transfer more than balance
        assert_noop!(
            MyPallet::transfer(RuntimeOrigin::signed(1), 2, 10000),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn unauthorized_error() {
    new_test_ext().execute_with(|| {
        // Create account as Alice
        assert_ok!(MyPallet::create_account(RuntimeOrigin::signed(1), AccountType::Citizen));

        // Try to delete as Bob (should fail)
        assert_noop!(
            MyPallet::delete_account(RuntimeOrigin::signed(2), 1),
            Error::<Test>::NotAuthorized
        );
    });
}
```

#### **Pattern 3: State Changes**

```rust
#[test]
fn balance_updates_correctly() {
    new_test_ext().execute_with(|| {
        // Initial balances
        assert_eq!(Balances::free_balance(1), 1000);
        assert_eq!(Balances::free_balance(2), 500);

        // Transfer
        assert_ok!(MyPallet::transfer(RuntimeOrigin::signed(1), 2, 100));

        // Verify new balances
        assert_eq!(Balances::free_balance(1), 900);
        assert_eq!(Balances::free_balance(2), 600);
    });
}
```

#### **Pattern 4: Time-Based Logic**

```rust
#[test]
fn proposal_expires_after_deadline() {
    new_test_ext().execute_with(|| {
        // Create proposal with 10-block deadline
        assert_ok!(Governance::propose(
            RuntimeOrigin::signed(1),
            "Proposal".into(),
            "Description".into(),
            1000,
            2,
            10  // Deadline: block 10
        ));

        // Advance to block 5 (before deadline)
        System::set_block_number(5);
        let proposal = Governance::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Active);

        // Advance to block 11 (after deadline)
        System::set_block_number(11);
        assert_ok!(Governance::finalize_proposal(RuntimeOrigin::signed(1), 0));
        
        let proposal = Governance::proposals(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Expired);
    });
}
```

#### **Pattern 5: Event Verification**

```rust
#[test]
fn events_emitted_correctly() {
    new_test_ext().execute_with(|| {
        // Perform actions
        assert_ok!(MyPallet::create_account(RuntimeOrigin::signed(1), AccountType::Citizen));
        assert_ok!(MyPallet::transfer(RuntimeOrigin::signed(1), 2, 100));

        // Verify events in order
        let events = System::events();
        assert_eq!(events.len(), 2);

        // First event: AccountCreated
        assert_eq!(
            events[0].event,
            RuntimeEvent::MyPallet(Event::AccountCreated { who: 1 })
        );

        // Second event: Transfer
        assert_eq!(
            events[1].event,
            RuntimeEvent::MyPallet(Event::Transfer {
                from: 1,
                to: 2,
                amount: 100
            })
        );
    });
}
```

---

## 🔗 Integration Tests

### **Testing Pallet Interactions**

```rust
#[test]
fn governance_economy_integration() {
    new_test_ext().execute_with(|| {
        // 1. Create governance proposal
        assert_ok!(Governance::propose(
            RuntimeOrigin::signed(1),
            "Fund project".into(),
            "Description".into(),
            5000,  // Request 5000 DALLA from treasury
            2,     // Beneficiary
            100    // Deadline
        ));

        // 2. Vote for proposal
        assert_ok!(Governance::vote(RuntimeOrigin::signed(1), 0, Vote::Yes));
        assert_ok!(Governance::vote(RuntimeOrigin::signed(2), 0, Vote::Yes));
        assert_ok!(Governance::vote(RuntimeOrigin::signed(3), 0, Vote::Yes));

        // 3. Finalize (approve) proposal
        System::set_block_number(101);
        assert_ok!(Governance::finalize_proposal(RuntimeOrigin::signed(1), 0));

        // 4. Execute proposal (transfers funds from treasury)
        assert_ok!(Governance::execute_proposal(RuntimeOrigin::signed(1), 0));

        // 5. Verify funds transferred
        let initial_balance = 500;
        let expected_balance = initial_balance + 5000;
        assert_eq!(Balances::free_balance(2), expected_balance);

        // 6. Verify treasury balance decreased
        let treasury = Treasury::account_id();
        assert_eq!(Balances::free_balance(treasury), /* treasury balance - 5000 */);
    });
}
```

---

## ⚛️ React Component Tests

### **Setup (Jest + React Testing Library)**

```bash
cd ui/maya-wallet
pnpm add -D @testing-library/react @testing-library/jest-dom @testing-library/user-event
```

`jest.config.js`:
```javascript
module.exports = {
  testEnvironment: 'jsdom',
  setupFilesAfterEnv: ['<rootDir>/jest.setup.js'],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
};
```

`jest.setup.js`:
```javascript
import '@testing-library/jest-dom';
```

### **Component Test Example**

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import WalletConnect from '@/components/WalletConnect';

describe('WalletConnect', () => {
  it('renders connect button', () => {
    render(<WalletConnect onAccountChange={jest.fn()} />);
    
    const button = screen.getByText('Connect Wallet');
    expect(button).toBeInTheDocument();
  });

  it('calls onAccountChange when wallet connected', async () => {
    const onAccountChange = jest.fn();
    
    // Mock wallet
    global.enableWallet = jest.fn().mockResolvedValue([
      { address: '5GrwvaEF...', meta: { name: 'Alice' } }
    ]);

    render(<WalletConnect onAccountChange={onAccountChange} />);
    
    const button = screen.getByText('Connect Wallet');
    fireEvent.click(button);

    await waitFor(() => {
      expect(onAccountChange).toHaveBeenCalledWith({
        address: '5GrwvaEF...',
        meta: { name: 'Alice' }
      });
    });
  });

  it('shows error when wallet not found', async () => {
    global.enableWallet = jest.fn().mockRejectedValue(
      new Error('No wallet extension found')
    );

    render(<WalletConnect onAccountChange={jest.fn()} />);
    
    const button = screen.getByText('Connect Wallet');
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/No wallet extension found/i)).toBeInTheDocument();
    });
  });
});
```

### **Hook Testing**

```typescript
import { renderHook, act } from '@testing-library/react';
import useBlockchain from '@/hooks/useBlockchain';

describe('useBlockchain', () => {
  it('connects to blockchain', async () => {
    const { result } = renderHook(() => useBlockchain());

    expect(result.current.isConnected).toBe(false);

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.isConnected).toBe(true);
    expect(result.current.api).toBeDefined();
  });

  it('subscribes to new blocks', async () => {
    const { result } = renderHook(() => useBlockchain());

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.blockNumber).toBeGreaterThan(0);
  });
});
```

---

## 🐍 Python Tests

### **Setup (pytest)**

```bash
pip install pytest pytest-asyncio
```

### **Test Example**

```python
import pytest
from nawal.client.train import FederatedLearningClient

@pytest.fixture
def fl_client():
    """Create FL client for testing."""
    return FederatedLearningClient(
        server_url="http://localhost:8080",
        dataset_path="test_data.csv"
    )

def test_client_initialization(fl_client):
    """Test client initializes correctly."""
    assert fl_client.server_url == "http://localhost:8080"
    assert fl_client.dataset_path == "test_data.csv"

def test_load_dataset(fl_client):
    """Test dataset loading."""
    fl_client.load_dataset()
    
    assert fl_client.train_data is not None
    assert len(fl_client.train_data) > 0

@pytest.mark.asyncio
async def test_train_model(fl_client):
    """Test model training."""
    fl_client.load_dataset()
    
    initial_accuracy = 0.5
    metrics = await fl_client.train_model(epochs=5)
    
    assert metrics['accuracy'] > initial_accuracy
    assert 'loss' in metrics
    assert 'f1_score' in metrics

def test_model_encryption(fl_client):
    """Test model encryption."""
    fl_client.load_dataset()
    fl_client.train_model(epochs=1)
    
    encrypted = fl_client.encrypt_model()
    
    assert encrypted is not None
    assert len(encrypted) > 0
```

---

## 🔄 End-to-End Tests

### **Setup (Playwright)**

```bash
cd ui/maya-wallet
pnpm add -D @playwright/test
npx playwright install
```

`playwright.config.ts`:
```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  use: {
    baseURL: 'http://localhost:3000',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  webServer: {
    command: 'pnpm dev',
    port: 3000,
    reuseExistingServer: !process.env.CI,
  },
});
```

### **E2E Test Example**

```typescript
import { test, expect } from '@playwright/test';

test.describe('Wallet Flow', () => {
  test('complete transaction flow', async ({ page }) => {
    // 1. Navigate to app
    await page.goto('/');

    // 2. Connect wallet
    await page.click('text=Connect Wallet');
    
    // Mock wallet extension
    await page.evaluate(() => {
      window.injectedWeb3 = {
        'polkadot-js': {
          enable: () => Promise.resolve({
            accounts: {
              get: () => Promise.resolve([
                { address: '5GrwvaEF...', name: 'Alice' }
              ])
            },
            signer: {}
          })
        }
      };
    });

    // 3. Verify wallet connected
    await expect(page.locator('text=Alice')).toBeVisible();

    // 4. Navigate to send
    await page.click('text=Send');

    // 5. Fill form
    await page.fill('input[name=recipient]', '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty');
    await page.fill('input[name=amount]', '10');

    // 6. Submit transaction
    await page.click('button:has-text("Send")');

    // 7. Confirm in modal
    await page.click('button:has-text("Confirm")');

    // 8. Wait for success message
    await expect(page.locator('text=Transaction successful')).toBeVisible({ timeout: 10000 });

    // 9. Verify balance updated
    const balance = await page.textContent('[data-testid=balance]');
    expect(parseFloat(balance)).toBeLessThan(1000); // Started with 1000
  });

  test('governance proposal flow', async ({ page }) => {
    await page.goto('/governance');

    // Connect wallet
    await page.click('text=Connect Wallet');

    // Create proposal
    await page.click('text=Create Proposal');
    await page.fill('input[name=title]', 'Test Proposal');
    await page.fill('textarea[name=description]', 'This is a test');
    await page.fill('input[name=amount]', '5000');
    await page.click('button:has-text("Submit")');

    // Wait for proposal to appear
    await expect(page.locator('text=Test Proposal')).toBeVisible();

    // Vote on proposal
    await page.click('button:has-text("Vote For")');
    await expect(page.locator('text=Vote recorded')).toBeVisible();
  });
});
```

---

## 📊 Performance Testing

### **Benchmarking Rust Code**

```bash
# Run benchmarks
cargo bench -p pallet-belize-economy
```

`benches/benchmarks.rs`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pallet_belize_economy::*;

fn transfer_benchmark(c: &mut Criterion) {
    c.bench_function("transfer", |b| {
        b.iter(|| {
            // Benchmark transfer function
            transfer(
                black_box(sender),
                black_box(recipient),
                black_box(amount)
            )
        });
    });
}

criterion_group!(benches, transfer_benchmark);
criterion_main!(benches);
```

### **Load Testing (Artillery)**

```bash
npm install -g artillery
```

`load-test.yml`:
```yaml
config:
  target: 'https://api.belizechain.org'
  phases:
    - duration: 60
      arrivalRate: 10
      name: Warm up
    - duration: 300
      arrivalRate: 100
      name: Sustained load
    - duration: 60
      arrivalRate: 200
      name: Spike test

scenarios:
  - name: Query balance
    flow:
      - post:
          url: "/rpc"
          json:
            jsonrpc: "2.0"
            method: "system_health"
            id: 1
          capture:
            - json: "$.result.peers"
              as: "peers"
  
  - name: Submit transaction
    flow:
      - post:
          url: "/rpc"
          json:
            jsonrpc: "2.0"
            method: "author_submitExtrinsic"
            params: ["{{ $randomString }}"]
            id: 1
```

Run test:
```bash
artillery run load-test.yml
```

---

## 🎯 Test Coverage

### **Rust Coverage**

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage

# Open report
open coverage/index.html
```

### **JavaScript Coverage**

```bash
# Run tests with coverage
cd ui/maya-wallet
pnpm test --coverage

# Coverage report in coverage/
open coverage/lcov-report/index.html
```

### **Coverage Goals**

- **Unit tests**: 80%+ coverage
- **Integration tests**: 70%+ coverage
- **Critical paths**: 100% coverage
- **Error handling**: 100% coverage

---

## ✅ Testing Checklist

Before deploying:

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All E2E tests pass
- [ ] 80%+ code coverage
- [ ] No linter warnings
- [ ] Security audit passed
- [ ] Performance benchmarks met
- [ ] Load testing completed
- [ ] Error scenarios tested
- [ ] Edge cases covered

---

## 📚 Resources

**Testing Frameworks**:
- Rust: https://doc.rust-lang.org/book/ch11-00-testing.html
- Jest: https://jestjs.io/
- Playwright: https://playwright.dev/
- pytest: https://pytest.org/

**BelizeChain**:
- Discord: https://discord.gg/belizechain
- GitHub: https://github.com/BelizeChain/belizechain

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) for testing support! 🚀🇧🇿
