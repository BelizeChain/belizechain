# 🏗️ Build Your First dApp on BelizeChain

**Complete tutorial: Build a decentralized voting application from scratch**

---

## 📋 What You'll Build

A **Community Voting dApp** where Belizean neighborhoods can propose and vote on local improvements:

**Features**:
- 📝 Create proposals (street repairs, park improvements, etc.)
- 🗳️ Cast votes (yes/no/abstain)
- 💰 Fund approved proposals from community treasury
- 📊 Real-time results visualization
- 🔒 Wallet-based authentication (no passwords!)

**Tech Stack**:
- **Blockchain**: BelizeChain (Substrate)
- **Smart Contract**: Governance pallet (Rust)
- **Frontend**: Next.js + React + TypeScript
- **Styling**: Tailwind CSS
- **Wallet**: Polkadot.js extension

**Time Required**: 2-3 hours  
**Skill Level**: Intermediate  
**Prerequisites**: [Development environment set up](setup-environment.md)

---

## 🎯 Why Build This?

This tutorial teaches you:
- ✅ Blockchain interaction (read/write operations)
- ✅ Wallet connection and transaction signing
- ✅ Real-time blockchain data subscription
- ✅ Error handling and user feedback
- ✅ Professional UI/UX patterns
- ✅ Production-ready code structure

**Real-world applications**:
- Community governance
- DAO (Decentralized Autonomous Organization)
- Crowdfunding platforms
- Voting systems
- Grant management

---

## 🚀 Quick Start (30 Minutes)

### **Step 1: Start Local Blockchain** (2 minutes)

```bash
# Navigate to BelizeChain directory
cd ~/projects/belizechain

# Start development blockchain
./scripts/start_blockchain.sh
```

**Wait for this output**:
```
✅ BelizeChain node started
   RPC: http://localhost:9933
   WebSocket: ws://localhost:9944
```

**Keep this terminal open!** The blockchain must run throughout development.

---

### **Step 2: Create dApp Project** (3 minutes)

```bash
# Navigate to UI workspace
cd ~/projects/belizechain/ui

# Create new Next.js app
pnpm create next-app@latest community-voting \
  --typescript \
  --tailwind \
  --app \
  --src-dir \
  --import-alias "@/*"

# Navigate into project
cd community-voting
```

**Project structure created**:
```
community-voting/
├── src/
│   ├── app/           # Next.js 14 app router
│   ├── components/    # React components
│   └── lib/           # Utilities
├── public/            # Static assets
├── package.json
└── tailwind.config.ts
```

---

### **Step 3: Install Blockchain Dependencies** (2 minutes)

```bash
# Install Polkadot.js libraries
pnpm add @polkadot/api @polkadot/extension-dapp @polkadot/util @polkadot/util-crypto

# Install UI libraries
pnpm add lucide-react date-fns
```

**What we installed**:
- `@polkadot/api`: Connect to blockchain, read/write data
- `@polkadot/extension-dapp`: Connect to wallet browser extension
- `@polkadot/util`: Utility functions (encoding, decoding)
- `lucide-react`: Icon library
- `date-fns`: Date formatting

---

### **Step 4: Create Blockchain Connection** (5 minutes)

Create `src/lib/blockchain.ts`:

```typescript
// Blockchain connection and API utilities

import { ApiPromise, WsProvider } from '@polkadot/api';
import { web3Accounts, web3Enable, web3FromAddress } from '@polkadot/extension-dapp';

// Blockchain connection URL
const WS_PROVIDER = 'ws://localhost:9944';

let api: ApiPromise | null = null;

/**
 * Connect to BelizeChain blockchain
 */
export async function connectToBlockchain(): Promise<ApiPromise> {
  if (api) return api;

  console.log('🔗 Connecting to BelizeChain...');
  
  const provider = new WsProvider(WS_PROVIDER);
  api = await ApiPromise.create({ provider });

  await api.isReady;
  
  console.log('✅ Connected to BelizeChain');
  console.log(`   Chain: ${await api.rpc.system.chain()}`);
  console.log(`   Version: ${await api.rpc.system.version()}`);
  
  return api;
}

/**
 * Enable wallet and get accounts
 */
export async function enableWallet(): Promise<any[]> {
  // Request access to wallet extension
  const extensions = await web3Enable('Community Voting dApp');
  
  if (extensions.length === 0) {
    throw new Error('No wallet extension found. Please install Polkadot.js extension.');
  }
  
  // Get all accounts from wallet
  const accounts = await web3Accounts();
  
  if (accounts.length === 0) {
    throw new Error('No accounts found in wallet. Please create an account.');
  }
  
  console.log(`✅ Found ${accounts.length} wallet account(s)`);
  
  return accounts;
}

/**
 * Get injector for signing transactions
 */
export async function getInjector(address: string) {
  const injector = await web3FromAddress(address);
  return injector;
}

/**
 * Format balance from blockchain units to human-readable
 */
export function formatBalance(balance: any, decimals: number = 12): string {
  const value = balance.toString();
  const whole = value.slice(0, -decimals) || '0';
  const fraction = value.slice(-decimals).padStart(decimals, '0').slice(0, 4);
  return `${whole}.${fraction}`;
}

/**
 * Parse human-readable balance to blockchain units
 */
export function parseBalance(balance: string, decimals: number = 12): string {
  const [whole, fraction = '0'] = balance.split('.');
  const paddedFraction = fraction.padEnd(decimals, '0');
  return whole + paddedFraction;
}
```

---

### **Step 5: Create Proposal Type** (2 minutes)

Create `src/lib/types.ts`:

```typescript
// TypeScript types for our dApp

export interface Proposal {
  id: number;
  proposer: string;
  title: string;
  description: string;
  amount: string;
  beneficiary: string;
  votesFor: number;
  votesAgainst: number;
  votesAbstain: number;
  status: 'Active' | 'Approved' | 'Rejected' | 'Executed';
  createdAt: number;
  endsAt: number;
}

export interface Vote {
  voter: string;
  proposalId: number;
  vote: 'Yes' | 'No' | 'Abstain';
  timestamp: number;
}

export interface Account {
  address: string;
  meta: {
    name?: string;
    source: string;
  };
}
```

---

### **Step 6: Create Wallet Connection Component** (8 minutes)

Create `src/components/WalletConnect.tsx`:

```typescript
'use client';

import { useState, useEffect } from 'react';
import { Wallet, LogOut } from 'lucide-react';
import { enableWallet } from '@/lib/blockchain';
import type { Account } from '@/lib/types';

interface WalletConnectProps {
  onAccountChange: (account: Account | null) => void;
}

export default function WalletConnect({ onAccountChange }: WalletConnectProps) {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Connect to wallet
  const handleConnect = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const walletAccounts = await enableWallet();
      setAccounts(walletAccounts);
      
      // Auto-select first account
      if (walletAccounts.length > 0) {
        const firstAccount = walletAccounts[0];
        setSelectedAccount(firstAccount);
        onAccountChange(firstAccount);
      }
    } catch (err: any) {
      setError(err.message);
      console.error('Wallet connection error:', err);
    } finally {
      setIsLoading(false);
    }
  };

  // Disconnect wallet
  const handleDisconnect = () => {
    setSelectedAccount(null);
    setAccounts([]);
    onAccountChange(null);
  };

  // Switch account
  const handleAccountChange = (account: Account) => {
    setSelectedAccount(account);
    onAccountChange(account);
  };

  // Format address (shorten for display)
  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  return (
    <div className="flex items-center gap-4">
      {selectedAccount ? (
        <>
          {/* Account dropdown */}
          <div className="flex items-center gap-2 bg-white px-4 py-2 rounded-lg border border-gray-200">
            <Wallet className="w-5 h-5 text-blue-500" />
            <select
              value={selectedAccount.address}
              onChange={(e) => {
                const account = accounts.find(a => a.address === e.target.value);
                if (account) handleAccountChange(account);
              }}
              className="bg-transparent outline-none cursor-pointer"
            >
              {accounts.map((account) => (
                <option key={account.address} value={account.address}>
                  {account.meta.name || formatAddress(account.address)}
                </option>
              ))}
            </select>
          </div>
          
          {/* Disconnect button */}
          <button
            onClick={handleDisconnect}
            className="flex items-center gap-2 px-4 py-2 bg-red-50 text-red-600 rounded-lg hover:bg-red-100 transition"
          >
            <LogOut className="w-4 h-4" />
            Disconnect
          </button>
        </>
      ) : (
        <>
          {/* Connect button */}
          <button
            onClick={handleConnect}
            disabled={isLoading}
            className="flex items-center gap-2 px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Wallet className="w-5 h-5" />
            {isLoading ? 'Connecting...' : 'Connect Wallet'}
          </button>
          
          {/* Error message */}
          {error && (
            <div className="text-sm text-red-600 bg-red-50 px-4 py-2 rounded-lg">
              {error}
            </div>
          )}
        </>
      )}
    </div>
  );
}
```

---

### **Step 7: Create Main Application** (8 minutes)

Update `src/app/page.tsx`:

```typescript
'use client';

import { useState, useEffect } from 'react';
import { Vote as VoteIcon, Plus, TrendingUp } from 'lucide-react';
import WalletConnect from '@/components/WalletConnect';
import { connectToBlockchain, formatBalance } from '@/lib/blockchain';
import type { Account, Proposal } from '@/lib/types';

export default function Home() {
  const [account, setAccount] = useState<Account | null>(null);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [blockNumber, setBlockNumber] = useState(0);

  // Connect to blockchain on mount
  useEffect(() => {
    initBlockchain();
  }, []);

  const initBlockchain = async () => {
    try {
      const api = await connectToBlockchain();
      
      // Subscribe to new blocks
      api.rpc.chain.subscribeNewHeads((header) => {
        setBlockNumber(header.number.toNumber());
      });
      
      // Load proposals
      await loadProposals();
      
      setIsLoading(false);
    } catch (error) {
      console.error('Blockchain init error:', error);
      setIsLoading(false);
    }
  };

  const loadProposals = async () => {
    const api = await connectToBlockchain();
    
    // Query governance pallet for proposals
    const proposalCount = await api.query.governance.proposalCount();
    const count = proposalCount.toNumber();
    
    const proposalList: Proposal[] = [];
    
    for (let i = 0; i < count; i++) {
      const proposalData = await api.query.governance.proposals(i);
      
      if (proposalData.isSome) {
        const proposal = proposalData.unwrap();
        
        proposalList.push({
          id: i,
          proposer: proposal.proposer.toString(),
          title: proposal.title.toUtf8(),
          description: proposal.description.toUtf8(),
          amount: formatBalance(proposal.amount),
          beneficiary: proposal.beneficiary.toString(),
          votesFor: proposal.votesFor.toNumber(),
          votesAgainst: proposal.votesAgainst.toNumber(),
          votesAbstain: proposal.votesAbstain.toNumber(),
          status: proposal.status.toString(),
          createdAt: proposal.createdAt.toNumber(),
          endsAt: proposal.endsAt.toNumber(),
        });
      }
    }
    
    setProposals(proposalList);
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600">Connecting to BelizeChain...</p>
        </div>
      </div>
    );
  }

  return (
    <main className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <VoteIcon className="w-8 h-8 text-blue-500" />
              <div>
                <h1 className="text-2xl font-bold text-gray-900">
                  Community Voting
                </h1>
                <p className="text-sm text-gray-500">
                  Decentralized decision making for Belize
                </p>
              </div>
            </div>
            
            <div className="flex items-center gap-4">
              {/* Block number */}
              <div className="flex items-center gap-2 text-sm text-gray-600">
                <TrendingUp className="w-4 h-4" />
                Block #{blockNumber.toLocaleString()}
              </div>
              
              {/* Wallet connection */}
              <WalletConnect onAccountChange={setAccount} />
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {account ? (
          <>
            {/* Create proposal button */}
            <div className="mb-8">
              <button className="flex items-center gap-2 px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition">
                <Plus className="w-5 h-5" />
                Create Proposal
              </button>
            </div>

            {/* Proposals list */}
            <div className="grid gap-6">
              {proposals.length === 0 ? (
                <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
                  <VoteIcon className="w-12 h-12 text-gray-300 mx-auto mb-4" />
                  <p className="text-gray-600">No proposals yet</p>
                  <p className="text-sm text-gray-400 mt-2">
                    Be the first to create a community proposal!
                  </p>
                </div>
              ) : (
                proposals.map((proposal) => (
                  <ProposalCard
                    key={proposal.id}
                    proposal={proposal}
                    currentAccount={account.address}
                    onVote={() => loadProposals()}
                  />
                ))
              )}
            </div>
          </>
        ) : (
          <div className="text-center py-16 bg-white rounded-lg border border-gray-200">
            <Wallet className="w-16 h-16 text-gray-300 mx-auto mb-4" />
            <h2 className="text-2xl font-semibold text-gray-900 mb-2">
              Connect Your Wallet
            </h2>
            <p className="text-gray-600 max-w-md mx-auto">
              Connect your wallet to view and vote on community proposals.
              Don't have a wallet? Install the Polkadot.js extension.
            </p>
          </div>
        )}
      </div>
    </main>
  );
}

// Proposal card component (we'll implement this next)
function ProposalCard({ proposal, currentAccount, onVote }: any) {
  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6">
      <h3 className="text-xl font-semibold mb-2">{proposal.title}</h3>
      <p className="text-gray-600 mb-4">{proposal.description}</p>
      <div className="flex items-center justify-between">
        <span className="text-sm text-gray-500">Status: {proposal.status}</span>
        <span className="text-sm font-medium">{proposal.amount} DALLA</span>
      </div>
    </div>
  );
}
```

---

### **Step 8: Run Your dApp!** (2 minutes)

```bash
# Start development server
pnpm dev
```

**Open browser**: http://localhost:3000

**You should see**:
- ✅ Community Voting header
- ✅ "Connect Wallet" button
- ✅ Real-time block number updating
- ✅ Professional UI with Tailwind styling

**🎉 Your first dApp is running!**

---

## 🔨 Complete Implementation (90 Minutes)

Now let's build out the full functionality...

### **Step 9: Create Proposal Form** (20 minutes)

Create `src/components/CreateProposalForm.tsx`:

```typescript
'use client';

import { useState } from 'react';
import { X, Send } from 'lucide-react';
import { connectToBlockchain, getInjector, parseBalance } from '@/lib/blockchain';

interface CreateProposalFormProps {
  account: string;
  onClose: () => void;
  onSuccess: () => void;
}

export default function CreateProposalForm({ account, onClose, onSuccess }: CreateProposalFormProps) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [amount, setAmount] = useState('');
  const [beneficiary, setBeneficiary] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setError(null);

    try {
      const api = await connectToBlockchain();
      const injector = await getInjector(account);

      // Convert amount to blockchain units
      const amountInUnits = parseBalance(amount);

      // Create proposal transaction
      const tx = api.tx.governance.propose(
        title,
        description,
        amountInUnits,
        beneficiary
      );

      // Sign and send transaction
      await tx.signAndSend(
        account,
        { signer: injector.signer },
        ({ status, events }) => {
          if (status.isInBlock) {
            console.log(`✅ Proposal included in block ${status.asInBlock}`);
            
            // Check for errors
            events.forEach(({ event }) => {
              if (api.events.system.ExtrinsicFailed.is(event)) {
                setError('Transaction failed');
              } else if (api.events.governance.ProposalCreated.is(event)) {
                console.log('✅ Proposal created successfully');
                onSuccess();
                onClose();
              }
            });
          }
        }
      );
    } catch (err: any) {
      console.error('Proposal creation error:', err);
      setError(err.message);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg max-w-2xl w-full p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold">Create Proposal</h2>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 rounded-lg transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Title */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Title
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Fix potholes on Main Street"
              required
              maxLength={100}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="The potholes on Main Street between..."
              required
              rows={4}
              maxLength={1000}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Amount */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Amount (DALLA)
            </label>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="5000"
              required
              min="0"
              step="0.01"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Beneficiary */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Beneficiary Address
            </label>
            <input
              type="text"
              value={beneficiary}
              onChange={(e) => setBeneficiary(e.target.value)}
              placeholder="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
              required
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
            />
            <p className="text-xs text-gray-500 mt-1">
              Who will receive the funds if this proposal passes
            </p>
          </div>

          {/* Error message */}
          {error && (
            <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-600">{error}</p>
            </div>
          )}

          {/* Buttons */}
          <div className="flex items-center gap-3 pt-4">
            <button
              type="submit"
              disabled={isSubmitting}
              className="flex items-center gap-2 px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Send className="w-4 h-4" />
              {isSubmitting ? 'Creating...' : 'Create Proposal'}
            </button>
            
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="px-6 py-3 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
```

---

### **Step 10: Create Voting Component** (20 minutes)

Create `src/components/ProposalCard.tsx`:

```typescript
'use client';

import { useState } from 'react';
import { ThumbsUp, ThumbsDown, MinusCircle, Clock, User, DollarSign } from 'lucide-react';
import { format } from 'date-fns';
import { connectToBlockchain, getInjector } from '@/lib/blockchain';
import type { Proposal } from '@/lib/types';

interface ProposalCardProps {
  proposal: Proposal;
  currentAccount: string;
  onVote: () => void;
}

export default function ProposalCard({ proposal, currentAccount, onVote }: ProposalCardProps) {
  const [isVoting, setIsVoting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleVote = async (voteType: 'Yes' | 'No' | 'Abstain') => {
    setIsVoting(true);
    setError(null);

    try {
      const api = await connectToBlockchain();
      const injector = await getInjector(currentAccount);

      // Create vote transaction
      const tx = api.tx.governance.vote(proposal.id, voteType);

      // Sign and send
      await tx.signAndSend(
        currentAccount,
        { signer: injector.signer },
        ({ status, events }) => {
          if (status.isInBlock) {
            events.forEach(({ event }) => {
              if (api.events.system.ExtrinsicFailed.is(event)) {
                setError('Vote failed');
              } else if (api.events.governance.VoteCast.is(event)) {
                console.log('✅ Vote cast successfully');
                onVote(); // Refresh proposals
              }
            });
          }
        }
      );
    } catch (err: any) {
      console.error('Voting error:', err);
      setError(err.message);
    } finally {
      setIsVoting(false);
    }
  };

  // Calculate total votes
  const totalVotes = proposal.votesFor + proposal.votesAgainst + proposal.votesAbstain;
  
  // Calculate percentages
  const forPercent = totalVotes > 0 ? (proposal.votesFor / totalVotes) * 100 : 0;
  const againstPercent = totalVotes > 0 ? (proposal.votesAgainst / totalVotes) * 100 : 0;
  const abstainPercent = totalVotes > 0 ? (proposal.votesAbstain / totalVotes) * 100 : 0;

  // Format dates
  const createdDate = new Date(proposal.createdAt * 1000);
  const endsDate = new Date(proposal.endsAt * 1000);
  const now = Date.now();
  const hasEnded = now > proposal.endsAt * 1000;

  // Status styling
  const statusColors = {
    Active: 'bg-blue-100 text-blue-700',
    Approved: 'bg-green-100 text-green-700',
    Rejected: 'bg-red-100 text-red-700',
    Executed: 'bg-purple-100 text-purple-700',
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6 hover:shadow-lg transition">
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <h3 className="text-xl font-semibold text-gray-900 mb-2">
            {proposal.title}
          </h3>
          <p className="text-gray-600">{proposal.description}</p>
        </div>
        
        <span className={`px-3 py-1 rounded-full text-sm font-medium ${statusColors[proposal.status]}`}>
          {proposal.status}
        </span>
      </div>

      {/* Metadata */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6 p-4 bg-gray-50 rounded-lg">
        <div className="flex items-center gap-2">
          <User className="w-4 h-4 text-gray-400" />
          <div>
            <div className="text-xs text-gray-500">Proposer</div>
            <div className="text-sm font-medium text-gray-900">
              {proposal.proposer.slice(0, 6)}...{proposal.proposer.slice(-4)}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <DollarSign className="w-4 h-4 text-gray-400" />
          <div>
            <div className="text-xs text-gray-500">Amount</div>
            <div className="text-sm font-medium text-gray-900">
              {proposal.amount} DALLA
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Clock className="w-4 h-4 text-gray-400" />
          <div>
            <div className="text-xs text-gray-500">Created</div>
            <div className="text-sm font-medium text-gray-900">
              {format(createdDate, 'MMM d, yyyy')}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Clock className="w-4 h-4 text-gray-400" />
          <div>
            <div className="text-xs text-gray-500">{hasEnded ? 'Ended' : 'Ends'}</div>
            <div className="text-sm font-medium text-gray-900">
              {format(endsDate, 'MMM d, yyyy')}
            </div>
          </div>
        </div>
      </div>

      {/* Voting results */}
      <div className="space-y-3 mb-6">
        {/* For votes */}
        <div>
          <div className="flex items-center justify-between text-sm mb-1">
            <span className="text-gray-600">For</span>
            <span className="font-medium text-green-600">
              {proposal.votesFor} ({forPercent.toFixed(1)}%)
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-green-500 h-2 rounded-full transition-all"
              style={{ width: `${forPercent}%` }}
            />
          </div>
        </div>

        {/* Against votes */}
        <div>
          <div className="flex items-center justify-between text-sm mb-1">
            <span className="text-gray-600">Against</span>
            <span className="font-medium text-red-600">
              {proposal.votesAgainst} ({againstPercent.toFixed(1)}%)
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-red-500 h-2 rounded-full transition-all"
              style={{ width: `${againstPercent}%` }}
            />
          </div>
        </div>

        {/* Abstain votes */}
        <div>
          <div className="flex items-center justify-between text-sm mb-1">
            <span className="text-gray-600">Abstain</span>
            <span className="font-medium text-gray-600">
              {proposal.votesAbstain} ({abstainPercent.toFixed(1)}%)
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-gray-400 h-2 rounded-full transition-all"
              style={{ width: `${abstainPercent}%` }}
            />
          </div>
        </div>
      </div>

      {/* Voting buttons */}
      {proposal.status === 'Active' && !hasEnded && (
        <div className="flex items-center gap-3">
          <button
            onClick={() => handleVote('Yes')}
            disabled={isVoting}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-green-50 text-green-700 rounded-lg hover:bg-green-100 transition disabled:opacity-50"
          >
            <ThumbsUp className="w-4 h-4" />
            Vote For
          </button>

          <button
            onClick={() => handleVote('No')}
            disabled={isVoting}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-red-50 text-red-700 rounded-lg hover:bg-red-100 transition disabled:opacity-50"
          >
            <ThumbsDown className="w-4 h-4" />
            Vote Against
          </button>

          <button
            onClick={() => handleVote('Abstain')}
            disabled={isVoting}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-gray-50 text-gray-700 rounded-lg hover:bg-gray-100 transition disabled:opacity-50"
          >
            <MinusCircle className="w-4 h-4" />
            Abstain
          </button>
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="mt-3 p-3 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-sm text-red-600">{error}</p>
        </div>
      )}
    </div>
  );
}
```

---

### **Step 11: Update Main Page** (10 minutes)

Update `src/app/page.tsx` to use our new components:

```typescript
'use client';

import { useState, useEffect } from 'react';
import { Vote as VoteIcon, Plus, TrendingUp } from 'lucide-react';
import WalletConnect from '@/components/WalletConnect';
import CreateProposalForm from '@/components/CreateProposalForm';
import ProposalCard from '@/components/ProposalCard';
import { connectToBlockchain, formatBalance } from '@/lib/blockchain';
import type { Account, Proposal } from '@/lib/types';

export default function Home() {
  const [account, setAccount] = useState<Account | null>(null);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [blockNumber, setBlockNumber] = useState(0);
  const [showCreateForm, setShowCreateForm] = useState(false);

  // Connect to blockchain on mount
  useEffect(() => {
    initBlockchain();
  }, []);

  const initBlockchain = async () => {
    try {
      const api = await connectToBlockchain();
      
      // Subscribe to new blocks
      api.rpc.chain.subscribeNewHeads((header) => {
        setBlockNumber(header.number.toNumber());
      });
      
      // Load proposals
      await loadProposals();
      
      setIsLoading(false);
    } catch (error) {
      console.error('Blockchain init error:', error);
      setIsLoading(false);
    }
  };

  const loadProposals = async () => {
    const api = await connectToBlockchain();
    
    // Query governance pallet for proposals
    const proposalCount = await api.query.governance.proposalCount();
    const count = proposalCount.toNumber();
    
    const proposalList: Proposal[] = [];
    
    for (let i = 0; i < count; i++) {
      const proposalData = await api.query.governance.proposals(i);
      
      if (proposalData.isSome) {
        const proposal = proposalData.unwrap();
        
        proposalList.push({
          id: i,
          proposer: proposal.proposer.toString(),
          title: proposal.title.toUtf8(),
          description: proposal.description.toUtf8(),
          amount: formatBalance(proposal.amount),
          beneficiary: proposal.beneficiary.toString(),
          votesFor: proposal.votesFor.toNumber(),
          votesAgainst: proposal.votesAgainst.toNumber(),
          votesAbstain: proposal.votesAbstain.toNumber(),
          status: proposal.status.toString(),
          createdAt: proposal.createdAt.toNumber(),
          endsAt: proposal.endsAt.toNumber(),
        });
      }
    }
    
    setProposals(proposalList);
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600">Connecting to BelizeChain...</p>
        </div>
      </div>
    );
  }

  return (
    <main className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <VoteIcon className="w-8 h-8 text-blue-500" />
              <div>
                <h1 className="text-2xl font-bold text-gray-900">
                  Community Voting
                </h1>
                <p className="text-sm text-gray-500">
                  Decentralized decision making for Belize
                </p>
              </div>
            </div>
            
            <div className="flex items-center gap-4">
              {/* Block number */}
              <div className="flex items-center gap-2 text-sm text-gray-600">
                <TrendingUp className="w-4 h-4" />
                Block #{blockNumber.toLocaleString()}
              </div>
              
              {/* Wallet connection */}
              <WalletConnect onAccountChange={setAccount} />
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {account ? (
          <>
            {/* Create proposal button */}
            <div className="mb-8">
              <button
                onClick={() => setShowCreateForm(true)}
                className="flex items-center gap-2 px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
              >
                <Plus className="w-5 h-5" />
                Create Proposal
              </button>
            </div>

            {/* Proposals list */}
            <div className="grid gap-6">
              {proposals.length === 0 ? (
                <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
                  <VoteIcon className="w-12 h-12 text-gray-300 mx-auto mb-4" />
                  <p className="text-gray-600">No proposals yet</p>
                  <p className="text-sm text-gray-400 mt-2">
                    Be the first to create a community proposal!
                  </p>
                </div>
              ) : (
                proposals.map((proposal) => (
                  <ProposalCard
                    key={proposal.id}
                    proposal={proposal}
                    currentAccount={account.address}
                    onVote={loadProposals}
                  />
                ))
              )}
            </div>

            {/* Create proposal form modal */}
            {showCreateForm && (
              <CreateProposalForm
                account={account.address}
                onClose={() => setShowCreateForm(false)}
                onSuccess={loadProposals}
              />
            )}
          </>
        ) : (
          <div className="text-center py-16 bg-white rounded-lg border border-gray-200">
            <Wallet className="w-16 h-16 text-gray-300 mx-auto mb-4" />
            <h2 className="text-2xl font-semibold text-gray-900 mb-2">
              Connect Your Wallet
            </h2>
            <p className="text-gray-600 max-w-md mx-auto">
              Connect your wallet to view and vote on community proposals.
              Don't have a wallet? Install the Polkadot.js extension.
            </p>
          </div>
        )}
      </div>
    </main>
  );
}
```

---

### **Step 12: Add Loading States and Error Handling** (15 minutes)

Create `src/components/LoadingSpinner.tsx`:

```typescript
export default function LoadingSpinner({ size = 'md' }: { size?: 'sm' | 'md' | 'lg' }) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  };

  return (
    <div className={`animate-spin rounded-full border-b-2 border-blue-500 ${sizeClasses[size]}`} />
  );
}
```

Create `src/components/ErrorMessage.tsx`:

```typescript
import { AlertCircle } from 'lucide-react';

interface ErrorMessageProps {
  title?: string;
  message: string;
  onRetry?: () => void;
}

export default function ErrorMessage({ title = 'Error', message, onRetry }: ErrorMessageProps) {
  return (
    <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
      <div className="flex items-start gap-3">
        <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
        <div className="flex-1">
          <h3 className="text-sm font-medium text-red-800">{title}</h3>
          <p className="text-sm text-red-600 mt-1">{message}</p>
          {onRetry && (
            <button
              onClick={onRetry}
              className="mt-2 text-sm font-medium text-red-700 hover:text-red-800 underline"
            >
              Try again
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
```

---

### **Step 13: Test Complete Application** (15 minutes)

**Test 1: Wallet Connection**
```bash
# Make sure blockchain is running
# Make sure Polkadot.js extension is installed
# Browser: chrome.google.com/webstore -> search "Polkadot.js extension"
```

1. Open http://localhost:3000
2. Click "Connect Wallet"
3. Approve connection in extension
4. Should see your account address

**Test 2: Create Proposal**
1. Click "Create Proposal"
2. Fill form:
   - Title: "Fix Main Street Potholes"
   - Description: "Repair potholes on Main Street between 1st and 5th"
   - Amount: "5000"
   - Beneficiary: `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty` (Bob's address)
3. Click "Create Proposal"
4. Sign transaction in wallet
5. Wait for confirmation
6. Proposal appears in list!

**Test 3: Vote on Proposal**
1. Click "Vote For" on your proposal
2. Sign transaction
3. Wait for confirmation
4. Vote count updates!
5. Progress bar shows 100% For

**Test 4: Real-time Updates**
1. Open dApp in two browser windows
2. Vote in one window
3. Other window updates automatically!
4. Block number updates every ~6 seconds

---

## 🚀 Production Deployment (30 Minutes)

### **Step 14: Build for Production**

```bash
# Build optimized production bundle
cd ~/projects/belizechain/ui/community-voting
pnpm build

# Output will be in .next/ directory
```

---

### **Step 15: Deploy to Vercel** (Recommended)

```bash
# Install Vercel CLI
pnpm add -g vercel

# Deploy
vercel

# Follow prompts:
# - Setup and deploy? Yes
# - Which scope? Your account
# - Link to existing project? No
# - Project name? community-voting
# - Directory? ./
# - Override settings? No

# Production URL: https://community-voting-abc123.vercel.app
```

**Configure environment variables**:
1. Go to Vercel dashboard
2. Project settings → Environment Variables
3. Add: `NEXT_PUBLIC_WS_PROVIDER=wss://mainnet.belizechain.org`

---

### **Alternative: Deploy to Your Own Server**

```bash
# Build application
pnpm build

# Copy to server
scp -r .next public package.json user@your-server.com:/var/www/community-voting/

# On server:
cd /var/www/community-voting
pnpm install --production
pnpm start

# Configure nginx reverse proxy
sudo nano /etc/nginx/sites-available/community-voting

# Add:
server {
    listen 80;
    server_name voting.belizechain.org;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}

# Enable site
sudo ln -s /etc/nginx/sites-available/community-voting /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Setup SSL with Let's Encrypt
sudo certbot --nginx -d voting.belizechain.org
```

---

## 📚 What You Learned

### **Blockchain Concepts**
- ✅ Connecting to blockchain nodes (WebSocket)
- ✅ Querying blockchain state (storage queries)
- ✅ Submitting transactions (extrinsics)
- ✅ Transaction signing (cryptographic signatures)
- ✅ Real-time subscriptions (new blocks, events)
- ✅ Error handling and retry logic

### **Frontend Development**
- ✅ Next.js 14 App Router
- ✅ TypeScript for type safety
- ✅ React hooks (useState, useEffect)
- ✅ Tailwind CSS for styling
- ✅ Component composition
- ✅ Form handling and validation

### **User Experience**
- ✅ Wallet integration (Polkadot.js extension)
- ✅ Loading states and skeletons
- ✅ Error messages and retry
- ✅ Real-time updates
- ✅ Responsive design
- ✅ Accessibility (ARIA labels, keyboard navigation)

---

## 🎨 Customization Ideas

### **Feature Enhancements**
1. **Delegation**: Let users delegate voting power
2. **Comments**: Discussion threads on proposals
3. **Notifications**: Email/push when proposal created
4. **Analytics**: Charts showing voting trends
5. **Filters**: Filter by status, amount, date
6. **Search**: Search proposals by title/description
7. **Categories**: Tag proposals (infrastructure, education, etc.)
8. **Quadratic Voting**: More sophisticated voting mechanism

### **UI Improvements**
1. **Dark Mode**: Toggle between light/dark theme
2. **Animations**: Smooth transitions and loading states
3. **Mobile App**: React Native version
4. **Multiple Languages**: i18n support (English, Spanish)
5. **Social Sharing**: Share proposals on social media
6. **PDF Export**: Generate proposal PDFs

### **Advanced Features**
1. **Multi-sig Proposals**: Require multiple approvers
2. **Time-locked Execution**: Delay execution of approved proposals
3. **Recurring Proposals**: Monthly/quarterly proposals
4. **Proposal Templates**: Pre-filled forms for common proposals
5. **Reputation System**: Weight votes by contribution history
6. **NFT Rewards**: Issue NFTs to active voters

---

## 🐛 Troubleshooting

### **Issue 1: "Cannot connect to wallet"**

**Problem**: Polkadot.js extension not installed or not giving permission.

**Solution**:
1. Install Polkadot.js extension from Chrome Web Store
2. Create account in extension
3. Refresh dApp page
4. Click "Connect Wallet" again
5. Approve connection in extension popup

---

### **Issue 2: "Transaction failed"**

**Problem**: Insufficient balance or invalid parameters.

**Solution**:
```bash
# Check account balance
# In browser console:
const api = await connectToBlockchain();
const balance = await api.query.system.account('YOUR_ADDRESS');
console.log('Balance:', balance.data.free.toString());

# Make sure you have test tokens
# For local dev chain, use Alice/Bob accounts (pre-funded)
```

---

### **Issue 3: "WebSocket connection failed"**

**Problem**: Blockchain node not running or wrong URL.

**Solution**:
```bash
# Check if blockchain is running
lsof -i :9944

# If not running, start it:
cd ~/projects/belizechain
./scripts/start_blockchain.sh

# Update WS_PROVIDER in lib/blockchain.ts if needed
const WS_PROVIDER = 'ws://localhost:9944';
```

---

### **Issue 4: "Module not found: @polkadot/api"**

**Problem**: Dependencies not installed.

**Solution**:
```bash
cd ~/projects/belizechain/ui/community-voting
pnpm install
```

---

## 📚 Next Steps

**Expand Your dApp**:
1. **[Develop Custom Pallets →](smart-pallets.md)**  
   Build custom blockchain logic in Rust

2. **[API Reference →](api-reference.md)**  
   Explore all available blockchain APIs

3. **[Testing Guide →](testing.md)**  
   Write comprehensive tests

**Deploy to Production**:
1. **[Deployment Guide →](../deployment/README.md)**  
   Deploy to mainnet

2. **[Security Best Practices →](../deployment/security.md)**  
   Secure your dApp

**Join Community**:
- Discord: https://discord.gg/belizechain
- Forum: https://forum.belizechain.org
- GitHub: https://github.com/BelizeChain/belizechain

---

**🎉 Congratulations!** You've built a complete decentralized application on BelizeChain!

**Questions?** Join our [Discord](https://discord.gg/belizechain) for support.

**Happy building!** 🚀🇧🇿
