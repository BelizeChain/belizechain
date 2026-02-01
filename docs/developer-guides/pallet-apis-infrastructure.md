# Infrastructure Pallet APIs

**Oracle • Interoperability • Consensus • Quantum**

Comprehensive API reference for BelizeChain's infrastructure pallets.

---

## Oracle Pallet

**Merchant verification ONLY (bBZD peg is 1:1 fixed, no price Oracle needed)**

### Extrinsics

#### `register_merchant_verification`
Register merchant for cashback program (NOT a price feed)

```rust
pub fn register_merchant_verification(
    origin: OriginFor<T>,
    merchant: T::AccountId,
    category: MerchantCategory,
    verification_proof: BoundedVec<u8, ConstU32<256>>
) -> DispatchResult
```

**Purpose:** Verify merchants for tourism cashback eligibility, **NOT** exchange rates

**Events:**
- `MerchantVerified(AccountId, MerchantCategory)`

**Weights:** 40M + 2 reads + 1 write

```javascript
// JavaScript example
await api.tx.oracle.registerMerchantVerification(
  merchantAddress,
  'Hotels',  // Category for 8% cashback
  verificationProof
).signAndSend(verifierAccount);
```

#### `update_merchant_status`
Update merchant verification status

```rust
pub fn update_merchant_status(
    origin: OriginFor<T>,
    merchant: T::AccountId,
    status: VerificationStatus
) -> DispatchResult
```

**Status values:**
- `Pending`: Awaiting verification
- `Verified`: Active for cashback
- `Suspended`: Temporarily inactive
- `Revoked`: Permanently removed

**Events:**
- `MerchantStatusUpdated(AccountId, VerificationStatus)`

**Weights:** 35M + 2 reads + 1 write

### Storage

#### `MerchantVerifications`
```rust
pub type MerchantVerifications<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    MerchantVerification<T::BlockNumber>,
    OptionQuery
>;

pub struct MerchantVerification<BlockNumber> {
    pub category: MerchantCategory,
    pub status: VerificationStatus,
    pub verified_at: BlockNumber,
    pub verifier: AccountId,
    pub expiry: BlockNumber
}
```

**NOTE:** This pallet does NOT provide exchange rates. bBZD is pegged 1:1 to BZD by Central Bank governance, not Oracle data.

---

## Interoperability Pallet

**Cross-chain bridges to Ethereum and Polkadot ecosystem**

### Extrinsics

#### `initiate_bridge_transfer`
Transfer assets to Ethereum or Polkadot

```rust
pub fn initiate_bridge_transfer(
    origin: OriginFor<T>,
    target_chain: ChainId,
    asset: AssetId,
    amount: BalanceOf<T>,
    recipient: BoundedVec<u8, ConstU32<64>>  // External address
) -> DispatchResult
```

**Supported chains:**
- `Ethereum`: ERC20 wrapped DALLA/bBZD
- `Polkadot`: XCM to relay chain
- `Kusama`: XCM to Kusama relay
- `Moonbeam`: EVM parachain bridge

**Events:**
- `BridgeTransferInitiated(AccountId, ChainId, AssetId, Balance, Vec<u8>)`

**Weights:** 90M + 4 reads + 3 writes

```typescript
// Bridge 1000 DALLA to Ethereum
await api.tx.interoperability.initiateBridgeTransfer(
  'Ethereum',
  'DALLA',
  1000_000_000_000_000n,
  '0x742d35Cc6634C0532925a3b844Bc454e4438f44e'  // ETH address
).signAndSend(sender);
```

#### `confirm_bridge_transfer`
Confirm incoming bridge transfer (relayer only)

```rust
pub fn confirm_bridge_transfer(
    origin: OriginFor<T>,
    transfer_id: H256,
    source_chain: ChainId,
    proof: BridgeProof
) -> DispatchResult
```

**Requirements:**
- Valid Merkle proof from source chain
- Relayer authorization
- Sufficient liquidity in bridge pool

**Events:**
- `BridgeTransferConfirmed(H256, AccountId, Balance)`

**Weights:** 100M + 5 reads + 4 writes

#### `submit_xcm_message`
Send XCM message to Polkadot parachain

```rust
pub fn submit_xcm_message(
    origin: OriginFor<T>,
    destination: MultiLocation,
    message: Xcm<()>
) -> DispatchResult
```

**XCM capabilities:**
- Asset transfers (ReserveAssetDeposited)
- Remote execution (Transact)
- Staking delegation (Bond)

**Events:**
- `XcmMessageSent(MultiLocation, XcmHash)`

**Weights:** 120M + 3 reads + 2 writes

```rust
// XCM example: Transfer to Polkadot relay chain
use xcm::latest::prelude::*;

let destination = MultiLocation {
    parents: 1,
    interior: X1(AccountId32 {
        network: NetworkId::Polkadot,
        id: recipient.into()
    })
};

let message = Xcm(vec![
    WithdrawAsset((Here, 1000 * DALLA).into()),
    InitiateReserveWithdraw {
        assets: All.into(),
        reserve: destination.clone(),
        xcm: Xcm(vec![DepositAsset {
            assets: All.into(),
            max_assets: 1,
            beneficiary: destination
        }])
    }
]);
```

### Storage

#### `BridgeTransfers`
```rust
pub type BridgeTransfers<T> = StorageMap<
    _,
    Blake2_128Concat,
    H256,  // Transfer ID
    BridgeTransfer<T::AccountId, BalanceOf<T>, T::BlockNumber>,
    OptionQuery
>;

pub struct BridgeTransfer<AccountId, Balance, BlockNumber> {
    pub sender: AccountId,
    pub target_chain: ChainId,
    pub asset: AssetId,
    pub amount: Balance,
    pub recipient: BoundedVec<u8, ConstU32<64>>,
    pub status: TransferStatus,
    pub initiated_at: BlockNumber
}
```

#### `BridgeLiquidity`
```rust
pub type BridgeLiquidity<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, ChainId,
    Blake2_128Concat, AssetId,
    BalanceOf<T>,
    ValueQuery
>;
```

Liquidity pools for each chain/asset pair

---

## Consensus Pallet

**Proof of Useful Work with quantum work integration**

### Extrinsics

#### `submit_quantum_work`
Submit Proof of Quantum Work (Kinich integration)

```rust
pub fn submit_quantum_work(
    origin: OriginFor<T>,
    problem_id: u64,
    quantum_solution: BoundedVec<u8, ConstU32<1024>>,
    proof: QuantumProof
) -> DispatchResult
```

**Verification:**
- Quantum signature validation
- Solution correctness check
- Problem difficulty verification

**Rewards:** 50 DALLA base + up to 200 DALLA for complex problems

**Events:**
- `QuantumWorkSubmitted(AccountId, u64, QuantumProof)`
- `QuantumRewardIssued(AccountId, Balance)`

**Weights:** 150M + 6 reads + 4 writes

```python
# Python example (Kinich → Blockchain)
from kinich import QuantumNode

node = QuantumNode()
solution = node.solve_optimization_problem(problem_id)

receipt = substrate.compose_call(
    call_module='Consensus',
    call_function='submit_quantum_work',
    call_params={
        'problem_id': problem_id,
        'quantum_solution': solution.encode(),
        'proof': generate_quantum_proof(solution)
    }
)
```

#### `register_validator_node`
Register node as validator

```rust
pub fn register_validator_node(
    origin: OriginFor<T>,
    node_type: ValidatorNodeType,
    endpoint: BoundedVec<u8, ConstU32<128>>
) -> DispatchResult
```

**Node types:**
- `Standard`: Basic block production
- `Nawal`: Federated learning capability
- `Kinich`: Quantum work capability
- `Full`: All capabilities (highest rewards)

**Requirements:**
- Minimum stake: 10,000 DALLA
- KYC level: Verified
- Uptime SLA: 99.5%

**Events:**
- `ValidatorNodeRegistered(AccountId, ValidatorNodeType)`

**Weights:** 50M + 3 reads + 2 writes

### Storage

#### `QuantumWorkSubmissions`
```rust
pub type QuantumWorkSubmissions<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, u64,  // Problem ID
    Blake2_128Concat, T::AccountId,
    QuantumSubmission<T::BlockNumber>,
    OptionQuery
>;

pub struct QuantumSubmission<BlockNumber> {
    pub solution: BoundedVec<u8, ConstU32<1024>>,
    pub submitted_at: BlockNumber,
    pub verified: bool,
    pub reward: Balance
}
```

#### `ValidatorNodes`
```rust
pub type ValidatorNodes<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    ValidatorNodeInfo<T::BlockNumber>,
    OptionQuery
>;

pub struct ValidatorNodeInfo<BlockNumber> {
    pub node_type: ValidatorNodeType,
    pub endpoint: BoundedVec<u8, ConstU32<128>>,
    pub registered_at: BlockNumber,
    pub blocks_produced: u64,
    pub uptime_percentage: u8
}
```

---

## Quantum Pallet

**Quantum workload orchestration and compression**

### Extrinsics

#### `request_quantum_compression`
Request quantum compression for large data (Kinich/Pakit integration)

```rust
pub fn request_quantum_compression(
    origin: OriginFor<T>,
    content_hash: H256,
    target_ratio: u8  // Desired compression ratio (2-10x)
) -> DispatchResult
```

**Process:**
1. Fetch content from Pakit DAG storage
2. Submit to Kinich quantum node
3. Store compressed result back to Pakit
4. Update on-chain proof with new hash

**Cost:** ~$10 per GB (Azure Quantum compute)

**Events:**
- `QuantumCompressionRequested(AccountId, H256, u8)`
- `QuantumCompressionCompleted(H256, H256, u8)` - (original, compressed, actual_ratio)

**Weights:** 70M + 4 reads + 2 writes

```javascript
// Request 6x compression for archival data
await api.tx.quantum.requestQuantumCompression(
  contentHash,
  6  // Target 6x ratio
).signAndSend(requester);
```

#### `register_quantum_backend`
Register quantum computing backend (admin only)

```rust
pub fn register_quantum_backend(
    origin: OriginFor<T>,
    backend_id: BoundedVec<u8, ConstU32<32>>,
    provider: QuantumProvider,
    qubits: u16,
    gate_fidelity: Permill
) -> DispatchResult
```

**Providers:**
- `AzureIonQ`: 25 qubits, 99.5% fidelity (primary)
- `AzureQuantinuum`: 20 qubits, 99.9% fidelity
- `IBMQuantum`: 127 qubits, 99.2% fidelity (fallback)
- `Rigetti`: 32 qubits, 98.5% fidelity

**Events:**
- `QuantumBackendRegistered(Vec<u8>, QuantumProvider, u16)`

**Weights:** 45M + 2 reads + 1 write

### Storage

#### `QuantumCompressionJobs`
```rust
pub type QuantumCompressionJobs<T> = StorageMap<
    _,
    Blake2_128Concat,
    H256,  // Content hash
    CompressionJob<T::AccountId, T::BlockNumber>,
    OptionQuery
>;

pub struct CompressionJob<AccountId, BlockNumber> {
    pub requester: AccountId,
    pub original_hash: H256,
    pub target_ratio: u8,
    pub status: JobStatus,
    pub submitted_at: BlockNumber,
    pub compressed_hash: Option<H256>,
    pub actual_ratio: Option<u8>
}
```

#### `QuantumBackends`
```rust
pub type QuantumBackends<T> = StorageMap<
    _,
    Blake2_128Concat,
    BoundedVec<u8, ConstU32<32>>,  // Backend ID
    QuantumBackend,
    OptionQuery
>;

pub struct QuantumBackend {
    pub provider: QuantumProvider,
    pub qubits: u16,
    pub gate_fidelity: Permill,
    pub enabled: bool
}
```

---

## Type Definitions

```rust
// Interoperability
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ChainId {
    Ethereum,
    Polkadot,
    Kusama,
    Moonbeam,
    Acala
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TransferStatus {
    Initiated,
    Confirmed,
    Failed,
    Cancelled
}

// Consensus
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ValidatorNodeType {
    Standard,
    Nawal,
    Kinich,
    Full
}

// Quantum
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum QuantumProvider {
    AzureIonQ,
    AzureQuantinuum,
    IBMQuantum,
    Rigetti
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed
}
```

---

## Performance Benchmarks

### Bridge Operations
```
Ethereum bridge: 150 seconds average (15 block confirmations)
Polkadot XCM: 12 seconds (2 relay chain blocks)
Moonbeam EVM: 18 seconds (3 parachain blocks)

Cost per transfer:
- Ethereum: ~$5 (gas fees)
- Polkadot XCM: ~$0.10
- Internal: ~$0.001 (DALLA fees)
```

### Quantum Operations
```
Compression request: 70M gas
Backend registration: 45M gas

Quantum job latency:
- IonQ: 28 seconds average
- Quantinuum: 45 seconds
- IBM: 120 seconds (queue time)

Cost per compression:
- 1 GB data: ~$10 (1000 shots × 50 circuits)
- Savings: 6.8x ratio → 85% storage reduction
```

---

## Related Documentation

- [Core Pallet APIs](./pallet-apis-core.md)
- [Financial Pallet APIs](./pallet-apis-financial.md)
- [Services Pallet APIs](./pallet-apis-services.md)
- [Bridge Documentation](../bridges/ethereum-bridge.md)
- [Kinich Quantum Integration](../architecture/kinich-integration.md)
