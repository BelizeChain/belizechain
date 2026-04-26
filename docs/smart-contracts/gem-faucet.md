# Testnet Faucet - Free DALLA for Development

The GEM faucet provides free testnet DALLA tokens for smart contract development and testing.

Use the current operator-published public-testnet RPC and faucet URLs in the examples below. Legacy unpublished public-testnet hosts are intentionally no longer advertised here.

## Faucet Details

- **Drip Amount**: 1,000 DALLA per claim
- **Cooldown**: 100 blocks (~10 minutes at 6s block time)
- **Max Per Account**: 10,000 DALLA per day
- **Contract Address**: TBD (awaiting deployment)
- **Status**: ✅ Built, ready for testnet

## How to Use

### Method 1: Web Interface

1. Visit `https://<current-public-testnet-faucet>`
2. Connect Polkadot.js extension
3. Click "Request 1000 DALLA"
4. Sign transaction
5. Receive tokens in ~6 seconds (1 block)

### Method 2: CLI

```bash
# Install GEM CLI
npm install -g @belizechain/gem-cli

# Request tokens
gem faucet claim --account 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

# Output:
# ✓ Claimed 1,000 DALLA
# Transaction: 0xabcd1234...
# New balance: 1,000 DALLA
# Next claim available in 100 blocks (~10 min)
```

### Method 3: SDK

```javascript
const { GemSDK } = require('@belizechain/gem-sdk');
const { Keyring } = require('@polkadot/api');

async function claimFromFaucet() {
    const sdk = new GemSDK('wss://<current-public-testnet-rpc>');
    await sdk.connect();

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    // Claim tokens
    const result = await sdk.faucetClaim(
        'FAUCET_CONTRACT_ADDRESS',
        alice
    );

    console.log(`Claimed 1,000 DALLA`);
    console.log(`Tx: ${result.txHash}`);
    
    await sdk.disconnect();
}

claimFromFaucet();
```

## Contract Interface

### Extrinsics (Callable Functions)

#### `claim()`
Request 1,000 DALLA tokens.

**Parameters**: None

**Returns**: `Result<(), FaucetError>`

**Requirements**:
- Account has not claimed within last 100 blocks
- Account has not exceeded daily limit (10,000 DALLA)
- Faucet has sufficient balance

**Gas Cost**: ~8,000 units

#### `get_last_claim(account: AccountId) -> Option<BlockNumber>`
Query when account last claimed.

**Parameters**:
- `account`: Account to check

**Returns**: Block number of last claim, or `None` if never claimed

#### `get_claimable_amount(account: AccountId) -> u128`
Check how much account can claim now.

**Returns**:
- `1000000000000000` (1,000 DALLA) if eligible
- `0` if cooldown not expired

### Events

```rust
#[ink(event)]
pub struct Claimed {
    #[ink(topic)]
    recipient: AccountId,
    amount: u128,
    block: u32,
}

#[ink(event)]
pub struct DailyLimitReached {
    #[ink(topic)]
    account: AccountId,
    total_claimed_today: u128,
}
```

## Anti-Abuse Mechanisms

### 1. Cooldown Period

```rust
pub fn claim(&mut self) -> Result<(), FaucetError> {
    let caller = self.env().caller();
    let current_block = self.env().block_number();
    
    // Check last claim time
    if let Some(last_claim_block) = self.last_claims.get(caller) {
        let blocks_since_claim = current_block - last_claim_block;
        if blocks_since_claim < COOLDOWN_BLOCKS {
            return Err(FaucetError::CooldownNotExpired);
        }
    }
    
    // Process claim...
}
```

**Cooldown**: 100 blocks = ~10 minutes
**Purpose**: Prevent rapid successive claims

### 2. Daily Limit

```rust
const DAILY_LIMIT: u128 = 10_000_000_000_000_000; // 10K DALLA
const DAILY_BLOCKS: u32 = 14_400; // ~24 hours

pub fn check_daily_limit(&self, account: AccountId) -> Result<(), FaucetError> {
    let current_block = self.env().block_number();
    let claims = self.get_claims_in_window(account, DAILY_BLOCKS);
    
    let total_claimed: u128 = claims.iter().sum();
    
    if total_claimed >= DAILY_LIMIT {
        return Err(FaucetError::DailyLimitExceeded);
    }
    
    Ok(())
}
```

**Limit**: 10,000 DALLA per 24 hours
**Purpose**: Prevent account abuse

### 3. Balance Check

```rust
pub fn claim(&mut self) -> Result<(), FaucetError> {
    let faucet_balance = self.env().balance();
    
    if faucet_balance < DRIP_AMOUNT {
        return Err(FaucetError::InsufficientFaucetBalance);
    }
    
    // Process claim...
}
```

**Minimum**: Faucet must have ≥ 1,000 DALLA
**Purpose**: Prevent overdraft

## Faucet Management (Admin Only)

### Refill Faucet

```rust
#[ink(message, payable)]
pub fn refill(&mut self) -> Result<(), FaucetError> {
    self.ensure_admin()?;
    
    let amount = self.env().transferred_value();
    self.env().emit_event(Refilled {
        amount,
        new_balance: self.env().balance(),
    });
    
    Ok(())
}
```

**Usage**:
```bash
cargo contract call \
    --contract FAUCET_ADDRESS \
    --message refill \
    --value 1000000 \ # 1M DALLA
    --suri //Admin
```

### Update Parameters

```rust
#[ink(message)]
pub fn update_params(
    &mut self,
    drip_amount: Option<u128>,
    cooldown_blocks: Option<u32>,
    daily_limit: Option<u128>,
) -> Result<(), FaucetError>
```

**Example**:
```javascript
await sdk.faucetUpdateParams(
    faucetAddress,
    admin,
    {
        dripAmount: '2000000000000000', // 2K DALLA
        cooldownBlocks: 200, // ~20 min
        dailyLimit: '20000000000000000' // 20K DALLA
    }
);
```

### Emergency Pause

```rust
#[ink(message)]
pub fn pause(&mut self) -> Result<(), FaucetError> {
    self.ensure_admin()?;
    self.paused = true;
    Ok(())
}

#[ink(message)]
pub fn unpause(&mut self) -> Result<(), FaucetError> {
    self.ensure_admin()?;
    self.paused = false;
    Ok(())
}
```

## Error Handling

```rust
pub enum FaucetError {
    /// Cooldown period not expired
    CooldownNotExpired,
    /// Daily claim limit exceeded
    DailyLimitExceeded,
    /// Faucet balance too low
    InsufficientFaucetBalance,
    /// Contract is paused
    Paused,
    /// Caller is not admin
    NotAdmin,
}
```

**Client-side handling**:
```javascript
try {
    await sdk.faucetClaim(address, account);
} catch (error) {
    if (error.code === 'CooldownNotExpired') {
        const blocksRemaining = error.details.blocksUntilNextClaim;
        console.log(`Wait ${blocksRemaining} blocks (~${blocksRemaining * 6}s)`);
    } else if (error.code === 'DailyLimitExceeded') {
        console.log('Daily limit reached. Try again tomorrow.');
    }
}
```

## Integration Example: Wallet

```typescript
// Maya Wallet faucet integration
import { GemSDK } from '@belizechain/gem-sdk';

export class FaucetService {
    private sdk: GemSDK;
    private faucetAddress: string;

    async checkEligibility(account: string): Promise<{
        canClaim: boolean;
        blocksUntilNext: number;
        dailyRemaining: u128;
    }> {
        const lastClaim = await this.sdk.faucetGetLastClaim(
            this.faucetAddress,
            account
        );

        const currentBlock = await this.sdk.getCurrentBlock();
        const blocksSinceLastClaim = lastClaim 
            ? currentBlock - lastClaim 
            : 999999;

        const canClaim = blocksSinceLastClaim >= 100;
        const blocksUntilNext = canClaim ? 0 : (100 - blocksSinceLastClaim);

        const claimableAmount = await this.sdk.faucetGetClaimableAmount(
            this.faucetAddress,
            account
        );

        return {
            canClaim: claimableAmount > 0,
            blocksUntilNext,
            dailyRemaining: claimableAmount,
        };
    }

    async claimTokens(account: Signer): Promise<string> {
        const result = await this.sdk.faucetClaim(
            this.faucetAddress,
            account
        );

        return result.txHash;
    }
}
```

**UI Component**:
```tsx
function FaucetButton({ account }: { account: string }) {
    const [eligible, setEligible] = useState(false);
    const [countdown, setCountdown] = useState(0);

    useEffect(() => {
        async function check() {
            const status = await faucetService.checkEligibility(account);
            setEligible(status.canClaim);
            setCountdown(status.blocksUntilNext * 6); // seconds
        }
        check();
        const interval = setInterval(check, 10000); // Check every 10s
        return () => clearInterval(interval);
    }, [account]);

    const handleClaim = async () => {
        try {
            const txHash = await faucetService.claimTokens(account);
            toast.success(`Claimed 1,000 DALLA! Tx: ${txHash}`);
        } catch (error) {
            toast.error(error.message);
        }
    };

    return (
        <button 
            onClick={handleClaim} 
            disabled={!eligible}
        >
            {eligible 
                ? 'Claim 1,000 DALLA' 
                : `Wait ${countdown}s`
            }
        </button>
    );
}
```

## Monitoring & Analytics

### Faucet Stats Dashboard

```javascript
async function getFaucetStats() {
    const faucet = await sdk.getContractInstance(faucetAddress);

    // Query events
    const claimEvents = await sdk.getEvents(faucetAddress, 'Claimed');

    return {
        totalClaims: claimEvents.length,
        totalDistributed: claimEvents.reduce((sum, e) => sum + e.amount, 0n),
        uniqueClaimants: new Set(claimEvents.map(e => e.recipient)).size,
        currentBalance: await sdk.getBalance(faucetAddress),
        claimsToday: claimEvents.filter(e => 
            e.block > currentBlock - 14400
        ).length,
    };
}
```

**Example Output**:
```json
{
  "totalClaims": 12_543,
  "totalDistributed": "12543000000000000000", // 12.5M DALLA
  "uniqueClaimants": 3_421,
  "currentBalance": "5000000000000000000", // 5M DALLA remaining
  "claimsToday": 827
}
```

## Deployment

```bash
cd gem/faucet
cargo contract build --release

cargo contract instantiate \
    --suri //Admin \
    --constructor new \
    --args 1000000000000000 100 10000000000000000 \ # drip, cooldown, daily_limit
    --url wss://<current-public-testnet-rpc> \
    --value 10000000 # Initial 10M DALLA funding
```

## Testing

```rust
#[cfg(test)]
mod tests {
    #[ink::test]
    fn claim_works() {
        let mut faucet = Faucet::new(
            1000_000_000_000_000,  // 1K DALLA drip
            100,                    // 100 block cooldown
            10_000_000_000_000_000  // 10K daily limit
        );

        // Fund faucet
        ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(
            10_000_000_000_000_000
        );

        // Claim tokens
        assert_eq!(faucet.claim(), Ok(()));

        // Verify balance increased
        let balance = faucet.env().balance();
        assert_eq!(balance, 9_000_000_000_000_000);
    }

    #[ink::test]
    fn cooldown_enforced() {
        let mut faucet = Faucet::new(1000_000_000_000_000, 100, 10_000_000_000_000_000);

        faucet.claim().unwrap();

        // Try to claim again immediately
        assert_eq!(faucet.claim(), Err(FaucetError::CooldownNotExpired));

        // Advance blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        // ... (advance 100 blocks)

        // Should work now
        assert_eq!(faucet.claim(), Ok(()));
    }
}
```

## Alternatives

If faucet is depleted or unavailable:

1. **Discord Bot**: Request tokens in #faucet channel
2. **Developer Grant**: Apply for larger allocation
3. **Local Devnet**: Use `//Alice` account with unlimited DALLA

## Resources

- **Contract Source**: [github.com/BelizeChain/gem/tree/main/faucet](https://github.com/BelizeChain/gem/tree/main/faucet)
- **Web Faucet**: `https://<current-public-testnet-faucet>`
- **Discord**: [discord.belizechain.org](https://discord.belizechain.org)
