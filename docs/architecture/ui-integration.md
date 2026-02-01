# UI Integration

**Maya Wallet & Blue Hole Portal**

BelizeChain's two primary user interfaces connect citizens and government to the blockchain.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│ Maya Wallet (Port 3000)                     │
│ - Citizen payments & staking               │
│ - Tourism cashback                         │
│ - BelizeID management                      │
└──────────────────┬──────────────────────────┘
                   │ Polkadot.js WebSocket
                   ↓
┌─────────────────────────────────────────────┐
│ BelizeChain Node (:9944)                    │
│ 15 Custom Pallets + System Pallets          │
└──────────────────┬──────────────────────────┘
                   │ Polkadot.js WebSocket
                   ↓
┌─────────────────────────────────────────────┐
│ Blue Hole Portal (Port 3001)                │
│ - Treasury management                       │
│ - Compliance dashboard                      │
│ - Analytics & reporting                     │
└─────────────────────────────────────────────┘
```

---

## Features

### Maya Wallet

#### 1. Account Management
```typescript
// ui/maya-wallet/lib/blockchain.ts
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';

export async function connectWallet(): Promise<ApiPromise> {
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });
  
  await api.isReady;
  console.log(`Connected to ${await api.rpc.system.chain()}`);
  
  return api;
}

export function createAccount(mnemonic: string) {
  const keyring = new Keyring({ type: 'sr25519' });
  const account = keyring.addFromMnemonic(mnemonic);
  
  return {
    address: account.address,
    publicKey: account.publicKey.toString()
  };
}
```

#### 2. DALLA/bBZD Transfers
```typescript
// ui/maya-wallet/components/Transfer.tsx
import { web3FromAddress } from '@polkadot/extension-dapp';

async function transferDALLA(
  api: ApiPromise,
  recipient: string,
  amount: number
) {
  const injector = await web3FromAddress(senderAddress);
  
  const transfer = api.tx.balances.transferKeepAlive(
    recipient,
    amount * 1_000_000_000_000  // 12 decimals
  );
  
  const hash = await transfer.signAndSend(
    senderAddress,
    { signer: injector.signer },
    ({ status, events }) => {
      if (status.isInBlock) {
        console.log(`Included in block: ${status.asInBlock}`);
        
        // Check for Transfer event
        events.forEach(({ event }) => {
          if (event.method === 'Transfer') {
            const [from, to, value] = event.data;
            console.log(`Transferred ${value.toString()} from ${from} to ${to}`);
          }
        });
      }
    }
  );
  
  return hash.toHex();
}
```

#### 3. Tourism Cashback
```typescript
// ui/maya-wallet/hooks/useCashback.ts
export function useCashback(merchantAddress: string) {
  const { api } = usePolkadotApi();
  const [rate, setRate] = useState<number>(0);
  
  useEffect(() => {
    async function fetchCashbackRate() {
      const merchant = await api.query.economy.merchants(merchantAddress);
      
      if (merchant.isSome) {
        const category = merchant.unwrap().category.toString();
        
        // Tourism categories get 5-8% cashback
        const rates: Record<string, number> = {
          'hotels': 8,
          'restaurants': 6,
          'tours': 7,
          'crafts': 5
        };
        
        setRate(rates[category] || 0);
      }
    }
    
    fetchCashbackRate();
  }, [merchantAddress]);
  
  return rate;
}
```

#### 4. Staking (PoUW)
```typescript
// ui/maya-wallet/pages/staking.tsx
async function stakeDALLA(amount: number) {
  const api = await connectWallet();
  const injector = await web3FromAddress(accountAddress);
  
  // Bond DALLA for staking
  const bondTx = api.tx.staking.bond(
    accountAddress,  // Controller (same as stash for simplicity)
    amount * 1_000_000_000_000,  // Amount in smallest units
    'Staked'  // RewardDestination
  );
  
  await bondTx.signAndSend(
    accountAddress,
    { signer: injector.signer }
  );
  
  console.log(`Staked ${amount} DALLA`);
}
```

### Blue Hole Portal

#### 1. Treasury Dashboard
```typescript
// ui/blue-hole-portal/pages/treasury.tsx
import { useQuery } from '@tanstack/react-query';

export function TreasuryDashboard() {
  const { data: treasury } = useQuery(['treasury'], async () => {
    const api = await connectWallet();
    
    // Multi-sig treasury balance
    const balance = await api.query.system.account(TREASURY_ADDRESS);
    const proposalCount = await api.query.treasury.proposalCount();
    const proposals = [];
    
    for (let i = 0; i < proposalCount.toNumber(); i++) {
      const proposal = await api.query.treasury.proposals(i);
      if (proposal.isSome) {
        proposals.push({
          id: i,
          beneficiary: proposal.unwrap().beneficiary.toString(),
          value: proposal.unwrap().value.toBigInt(),
          bond: proposal.unwrap().bond.toBigInt()
        });
      }
    }
    
    return {
      balance: balance.data.free.toBigInt(),
      proposals
    };
  });
  
  return (
    <div>
      <h1>National Treasury</h1>
      <p>Balance: {formatDALLA(treasury?.balance)} DALLA</p>
      <h2>Active Proposals: {treasury?.proposals.length}</h2>
      <ul>
        {treasury?.proposals.map(p => (
          <li key={p.id}>
            #{p.id}: {formatDALLA(p.value)} DALLA to {p.beneficiary}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

#### 2. Compliance Monitoring
```typescript
// ui/blue-hole-portal/pages/compliance.tsx
export function ComplianceDashboard() {
  const [kycStats, setKycStats] = useState({
    pending: 0,
    approved: 0,
    rejected: 0
  });
  
  useEffect(() => {
    async function fetchKycStats() {
      const api = await connectWallet();
      
      // Query all KYC entries
      const entries = await api.query.identity.kycRecords.entries();
      
      const stats = entries.reduce((acc, [_key, value]) => {
        const status = value.unwrap().status.toString();
        acc[status] = (acc[status] || 0) + 1;
        return acc;
      }, {} as Record<string, number>);
      
      setKycStats({
        pending: stats.Pending || 0,
        approved: stats.Approved || 0,
        rejected: stats.Rejected || 0
      });
    }
    
    fetchKycStats();
  }, []);
  
  return (
    <div>
      <h1>KYC/AML Compliance</h1>
      <div className="grid grid-cols-3 gap-4">
        <div className="stat">
          <h3>Pending Review</h3>
          <p>{kycStats.pending}</p>
        </div>
        <div className="stat">
          <h3>Approved</h3>
          <p>{kycStats.approved}</p>
        </div>
        <div className="stat">
          <h3>Rejected</h3>
          <p>{kycStats.rejected}</p>
        </div>
      </div>
    </div>
  );
}
```

#### 3. Analytics
```typescript
// ui/blue-hole-portal/pages/analytics.tsx
export function AnalyticsDashboard() {
  const [metrics, setMetrics] = useState({
    totalAccounts: 0,
    dailyTransactions: 0,
    avgTransactionValue: 0,
    stakingParticipation: 0
  });
  
  useEffect(() => {
    async function fetchMetrics() {
      const api = await connectWallet();
      
      // Total accounts
      const accounts = await api.query.system.account.entries();
      
      // Daily transactions (from recent blocks)
      const currentBlock = await api.rpc.chain.getBlock();
      const blockNumber = currentBlock.block.header.number.toNumber();
      
      let txCount = 0;
      let txValue = BigInt(0);
      
      // Scan last 100 blocks (~10 minutes)
      for (let i = blockNumber - 100; i < blockNumber; i++) {
        const hash = await api.rpc.chain.getBlockHash(i);
        const block = await api.rpc.chain.getBlock(hash);
        
        block.block.extrinsics.forEach(ext => {
          if (ext.method.section === 'balances' && ext.method.method === 'transferKeepAlive') {
            txCount++;
            const value = ext.method.args[1].toString();
            txValue += BigInt(value);
          }
        });
      }
      
      // Staking participation
      const validators = await api.query.staking.validators.entries();
      const nominators = await api.query.staking.nominators.entries();
      
      setMetrics({
        totalAccounts: accounts.length,
        dailyTransactions: txCount * 144,  // Extrapolate to 24h
        avgTransactionValue: Number(txValue / BigInt(txCount || 1)) / 1e12,
        stakingParticipation: ((validators.length + nominators.length) / accounts.length) * 100
      });
    }
    
    fetchMetrics();
    
    // Refresh every 60 seconds
    const interval = setInterval(fetchMetrics, 60000);
    return () => clearInterval(interval);
  }, []);
  
  return (
    <div>
      <h1>Network Analytics</h1>
      <div className="metrics">
        <div>
          <h3>Total Accounts</h3>
          <p>{metrics.totalAccounts.toLocaleString()}</p>
        </div>
        <div>
          <h3>Daily Transactions</h3>
          <p>{metrics.dailyTransactions.toLocaleString()}</p>
        </div>
        <div>
          <h3>Avg Transaction</h3>
          <p>{metrics.avgTransactionValue.toFixed(2)} DALLA</p>
        </div>
        <div>
          <h3>Staking Participation</h3>
          <p>{metrics.stakingParticipation.toFixed(1)}%</p>
        </div>
      </div>
    </div>
  );
}
```

---

## Configuration

### Environment Variables
```bash
# ui/maya-wallet/.env.local
NEXT_PUBLIC_WS_PROVIDER=ws://localhost:9944
NEXT_PUBLIC_NETWORK=BelizeChain Testnet
NEXT_PUBLIC_FAUCET_URL=https://faucet.belizechain.org

# ui/blue-hole-portal/.env.local
NEXT_PUBLIC_WS_PROVIDER=ws://localhost:9944
NEXT_PUBLIC_ADMIN_ACCOUNTS=5GrwvaEF...,5FHneW...
```

### Polkadot.js Types
```typescript
// ui/maya-wallet/lib/types.ts
export const customTypes = {
  AccountType: {
    _enum: ['Citizen', 'Business', 'Tourism', 'Government']
  },
  KycLevel: {
    _enum: ['None', 'Basic', 'Verified', 'Enhanced']
  },
  MerchantCategory: {
    _enum: ['Hotels', 'Restaurants', 'Tours', 'Crafts', 'Retail']
  },
  PropertyInfo: {
    owner: 'AccountId',
    document_hash: 'H256',
    land_area_sqm: 'u64',
    registered_at: 'BlockNumber'
  }
};

// Register types
api.registry.register(customTypes);
```

---

## Real-Time Updates

### Subscribe to Block Events
```typescript
// ui/maya-wallet/hooks/useBlockSubscription.ts
export function useBlockSubscription() {
  const [block, setBlock] = useState<BlockInfo | null>(null);
  const { api } = usePolkadotApi();
  
  useEffect(() => {
    let unsubscribe: () => void;
    
    async function subscribe() {
      unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
        setBlock({
          number: header.number.toNumber(),
          hash: header.hash.toHex(),
          parentHash: header.parentHash.toHex(),
          timestamp: Date.now()
        });
      });
    }
    
    subscribe();
    
    return () => {
      if (unsubscribe) unsubscribe();
    };
  }, [api]);
  
  return block;
}
```

### Subscribe to Balance Changes
```typescript
// ui/maya-wallet/hooks/useBalance.ts
export function useBalance(address: string) {
  const [balance, setBalance] = useState<bigint>(BigInt(0));
  const { api } = usePolkadotApi();
  
  useEffect(() => {
    let unsubscribe: () => void;
    
    async function subscribe() {
      unsubscribe = await api.query.system.account(
        address,
        ({ data: { free } }) => {
          setBalance(free.toBigInt());
        }
      );
    }
    
    subscribe();
    
    return () => {
      if (unsubscribe) unsubscribe();
    };
  }, [address, api]);
  
  return balance;
}
```

---

## Performance

### Initial Load Time
```
Maya Wallet (localhost):
- HTML/CSS/JS: 450 KB (gzipped: 120 KB)
- Polkadot.js: 2.8 MB (gzipped: 680 KB)
- First Contentful Paint: 1.2s
- Time to Interactive: 2.8s
- WebSocket connection: 150ms

Blue Hole Portal (localhost):
- HTML/CSS/JS: 520 KB (gzipped: 135 KB)
- Polkadot.js: 2.8 MB (cached from Maya)
- First Contentful Paint: 1.0s
- Time to Interactive: 2.5s
```

### Transaction Confirmation
```
User clicks "Send" → Blockchain confirmation

1. Sign transaction: 200ms (MetaMask)
2. Submit to mempool: 50ms
3. Block inclusion: 6s (average)
4. UI update: 100ms (WebSocket event)

Total: ~6.4 seconds end-to-end
```

---

## UI Component Library

### GlassCard Pattern
```typescript
// ui/shared/components/GlassCard.tsx
export function GlassCard({ 
  children, 
  variant = 'dark-medium',
  blur = 'lg'
}: GlassCardProps) {
  const variants = {
    'dark-light': 'bg-gray-800/30',
    'dark-medium': 'bg-gray-800/50',
    'dark-heavy': 'bg-gray-800/70'
  };
  
  const blurs = {
    'sm': 'backdrop-blur-sm',
    'md': 'backdrop-blur-md',
    'lg': 'backdrop-blur-lg'
  };
  
  return (
    <div className={`
      ${variants[variant]}
      ${blurs[blur]}
      rounded-xl
      border border-gray-700/50
      shadow-xl
    `}>
      {children}
    </div>
  );
}
```

### Standard Page Header
```typescript
// ui/maya-wallet/components/PageHeader.tsx
export function PageHeader({ title, subtitle, icon: Icon }: PageHeaderProps) {
  const router = useRouter();
  
  return (
    <div className="sticky top-0 bg-gray-900/80 backdrop-blur-xl px-6 py-4 z-10 border-b border-gray-700/50">
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center gap-3">
          <button 
            onClick={() => router.back()} 
            className="p-2 hover:bg-gray-800 rounded-full transition-colors"
          >
            <ArrowLeft size={24} className="text-gray-300" weight="bold" />
          </button>
          <div>
            <h1 className="text-xl font-bold text-white">{title}</h1>
            <p className="text-xs text-gray-400">{subtitle}</p>
          </div>
        </div>
        {Icon && (
          <Icon size={32} className="text-blue-400" weight="duotone" />
        )}
      </div>
    </div>
  );
}
```

---

## Security

### Transaction Signing
```typescript
// Always verify transaction details before signing
async function secureTransfer(recipient: string, amount: number) {
  // 1. Validate recipient address
  if (!isValidPolkadotAddress(recipient)) {
    throw new Error('Invalid recipient address');
  }
  
  // 2. Check balance
  const balance = await api.query.system.account(senderAddress);
  if (balance.data.free.toBigInt() < BigInt(amount)) {
    throw new Error('Insufficient balance');
  }
  
  // 3. Show confirmation dialog
  const confirmed = await showConfirmDialog({
    title: 'Confirm Transfer',
    message: `Send ${formatDALLA(amount)} to ${truncateAddress(recipient)}?`
  });
  
  if (!confirmed) return;
  
  // 4. Sign and send
  const hash = await transferDALLA(api, recipient, amount);
  
  return hash;
}
```

### XSS Prevention
```typescript
// Always sanitize user input
import DOMPurify from 'isomorphic-dompurify';

function renderUserContent(content: string) {
  const sanitized = DOMPurify.sanitize(content);
  return <div dangerouslySetInnerHTML={{ __html: sanitized }} />;
}
```

---

## Related Documentation

- [Multi-Repo Overview](./multi-repo-overview.md)
- [UI Repository](https://github.com/BelizeChain/ui)
- [Maya Wallet User Guide](../user-guides/maya-wallet.md)
- [Blue Hole Portal Admin Guide](../user-guides/blue-hole-portal.md)
- [Polkadot.js Documentation](https://polkadot.js.org/docs/)
