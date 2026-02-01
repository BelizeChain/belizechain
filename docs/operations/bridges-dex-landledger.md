# Cross-Chain Bridges & Interoperability

**Ethereum Bridge • Polkadot XCM • Asset Transfers • Security**

Complete guide to cross-chain operations on BelizeChain.

---

## Supported Bridges

| Chain | Type | Assets | Confirmation Time | Fee |
|-------|------|--------|-------------------|-----|
| **Ethereum** | Lock & Mint | wDALLA, wbBZD, ERC-20 → BelizeChain | ~150 seconds (10 ETH blocks) | 0.01 ETH + 10 DALLA |
| **Polkadot** | XCM v3 | DOT, DALLA (relay chain) | ~12 seconds (2 blocks) | 5 DALLA |
| **Kusama** | XCM v3 | KSM, DALLA | ~6 seconds (1 block) | 2 DALLA |
| **Moonbeam** | XCM + EVM | GLMR, ERC-20 | ~18 seconds (3 blocks) | 3 DALLA |

---

## Ethereum Bridge

### Architecture

```
Ethereum                     BelizeChain
┌──────────────┐            ┌──────────────┐
│ User locks   │──────────► │ Merkle proof │
│ 100 DAI      │   Event    │ verification │
│              │            │              │
│ BridgeVault  │            │ Mint 100     │
│ Contract     │            │ wDAI tokens  │
└──────────────┘            └──────────────┘
                             ▲
                             │
                 Relayer monitors events
```

### Bridge DALLA to Ethereum (Get wDALLA)

**Step 1: Initiate bridge transfer on BelizeChain**

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function bridgeToEthereum() {
  const provider = new WsProvider('wss://rpc.belizechain.org');
  const api = await ApiPromise.create({ provider });
  
  const amount = 1000 * 1e12;  // 1,000 DALLA
  const ethRecipient = '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb';
  
  const tx = api.tx.interoperability.initiateBridgeTransfer(
    'Ethereum',  // Target chain
    ethRecipient,
    amount
  );
  
  await tx.signAndSend(alice, ({ status, events }) => {
    if (status.isInBlock) {
      console.log(`✅ Bridge transfer initiated in block ${status.asInBlock}`);
      console.log('Wait ~150 seconds for Ethereum confirmation...');
    }
    
    events.forEach(({ event }) => {
      if (event.method === 'BridgeTransferInitiated') {
        const [bridgeId, from, to, amount] = event.data;
        console.log(`Bridge ID: ${bridgeId}`);
        console.log(`Merkle proof will be available after 10 ETH blocks`);
      }
    });
  });
}
```

**Step 2: Wait for relayer to submit proof**

Relayer monitors BelizeChain events and submits Merkle proof to Ethereum bridge contract.

**Step 3: Claim wDALLA on Ethereum**

```solidity
// BelizeChain bridge contract on Ethereum
contract BelizeChainBridge {
    function claimBridgedTokens(
        uint256 bridgeId,
        uint256 amount,
        bytes32[] calldata merkleProof
    ) external {
        // Verify Merkle proof
        require(verifyMerkleProof(bridgeId, amount, msg.sender, merkleProof), "Invalid proof");
        
        // Mint wDALLA tokens
        wDALLA.mint(msg.sender, amount);
        
        emit TokensClaimed(bridgeId, msg.sender, amount);
    }
}
```

**Step 4: Verify wDALLA balance**

```javascript
// Check wDALLA balance on Ethereum
const Web3 = require('web3');
const web3 = new Web3('https://mainnet.infura.io/v3/YOUR_KEY');

const wDALLA_ADDRESS = '0x...';  // wDALLA token contract
const wDALLA_ABI = [/* ERC-20 ABI */];

const contract = new web3.eth.Contract(wDALLA_ABI, wDALLA_ADDRESS);
const balance = await contract.methods.balanceOf('0x742d35...').call();

console.log(`wDALLA balance: ${web3.utils.fromWei(balance, 'ether')}`);
// Expected: 1000 wDALLA
```

### Bridge wDALLA back to BelizeChain (Redeem DALLA)

**Step 1: Burn wDALLA on Ethereum**

```javascript
// Approve and burn wDALLA
const Web3 = require('web3');
const web3 = new Web3(window.ethereum);

const wDALLA = new web3.eth.Contract(wDALLA_ABI, wDALLA_ADDRESS);
const bridge = new web3.eth.Contract(BRIDGE_ABI, BRIDGE_ADDRESS);

// Approve bridge to burn tokens
await wDALLA.methods.approve(
  BRIDGE_ADDRESS,
  web3.utils.toWei('1000', 'ether')
).send({ from: userAddress });

// Burn and bridge back
const belizeChainRecipient = '5GrwvaEF...';  // SS58 address
await bridge.methods.burnAndBridge(
  web3.utils.toWei('1000', 'ether'),
  belizeChainRecipient
).send({ from: userAddress });

console.log('✅ wDALLA burned, wait ~150 seconds for BelizeChain confirmation');
```

**Step 2: Claim on BelizeChain**

```javascript
// Relayer submits Ethereum event proof to BelizeChain
const ethBlockHash = '0xabc123...';  // Block containing burn event
const merkleProof = [/* Merkle proof of burn event */];

await api.tx.interoperability.confirmBridgeTransfer(
  bridgeId,
  merkleProof
).signAndSend(alice);

// DALLA tokens minted to your BelizeChain address
```

---

## Polkadot XCM Integration

### XCM Message Format

```rust
// Transfer 100 DOT from Polkadot to BelizeChain
let message = Xcm(vec![
    WithdrawAsset((Here, 100 * DOT).into()),
    BuyExecution {
        fees: (Here, 1 * DOT).into(),  // Pay 1 DOT for execution
        weight_limit: Unlimited,
    },
    DepositAsset {
        assets: All.into(),
        beneficiary: Parachain(2000).into(),  // BelizeChain parachain ID
    },
]);
```

### Transfer DOT to BelizeChain

**Using Polkadot.js Apps:**

1. Navigate to https://polkadot.js.org/apps
2. Network → Switch to Polkadot relay chain
3. Extrinsics → xcmPallet → limitedReserveTransferAssets
4. Parameters:
   - dest: `{ V3: { parents: 0, interior: { X1: { Parachain: 2000 } } } }`
   - beneficiary: Your BelizeChain address (SS58 format)
   - assets: `{ V3: [{ id: { Concrete: { parents: 0, interior: Here } }, fun: { Fungible: 100000000000 } }] }`  (100 DOT, 10 decimals)
   - feeAssetItem: 0
   - weightLimit: Unlimited
5. Sign and submit
6. ✅ DOT arrives on BelizeChain in ~12 seconds

**Programmatic example:**

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function transferDotToBelizeChain() {
  // Connect to Polkadot relay chain
  const provider = new WsProvider('wss://rpc.polkadot.io');
  const api = await ApiPromise.create({ provider });
  
  const dest = {
    V3: {
      parents: 0,
      interior: { X1: { Parachain: 2000 } }  // BelizeChain parachain ID
    }
  };
  
  const beneficiary = {
    V3: {
      parents: 0,
      interior: { X1: { AccountId32: {
        network: null,
        id: '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d'  // Alice
      }}}
    }
  };
  
  const assets = {
    V3: [{
      id: { Concrete: { parents: 0, interior: 'Here' } },
      fun: { Fungible: 100 * 1e10 }  // 100 DOT
    }]
  };
  
  const tx = api.tx.xcmPallet.limitedReserveTransferAssets(
    dest,
    beneficiary,
    assets,
    0,  // feeAssetItem
    'Unlimited'
  );
  
  await tx.signAndSend(alice, ({ status }) => {
    if (status.isInBlock) {
      console.log('✅ XCM message sent, DOT will arrive in ~12 seconds');
    }
  });
}
```

---

## Moonbeam Bridge (EVM Compatibility)

### Transfer GLMR to BelizeChain

```javascript
// Using Moonbeam's precompiled XCM contract
const Web3 = require('web3');
const web3 = new Web3('https://rpc.api.moonbeam.network');

const XCM_PRECOMPILE = '0x0000000000000000000000000000000000000804';
const XCM_ABI = [/* Moonbeam XCM precompile ABI */];

const xcm = new web3.eth.Contract(XCM_ABI, XCM_PRECOMPILE);

// Transfer 100 GLMR to BelizeChain
await xcm.methods.transfer(
  '0x02000400',  // BelizeChain parachain ID (2000 in hex)
  '5GrwvaEF...',  // BelizeChain recipient (SS58 → hex)
  web3.utils.toWei('100', 'ether')  // 100 GLMR
).send({ from: userAddress, value: web3.utils.toWei('100', 'ether') });

console.log('✅ GLMR sent to BelizeChain via XCM');
```

---

## Security Considerations

### Bridge Liquidity Limits

```rust
// Maximum bridge transfer per transaction
pub const MAX_BRIDGE_AMOUNT: Balance = 1_000_000 * DALLA;  // 1M DALLA

// Daily bridge limit per account
pub const DAILY_BRIDGE_LIMIT: Balance = 10_000_000 * DALLA;  // 10M DALLA

// Total bridge liquidity pool
pub const BRIDGE_LIQUIDITY_POOL: Balance = 100_000_000 * DALLA;  // 100M DALLA
```

### Merkle Proof Verification

```rust
pub fn verify_merkle_proof(
    proof: &MerkleProof,
    root: &H256,
    leaf: &H256
) -> bool {
    let mut computed_hash = *leaf;
    
    for (index, sibling) in proof.siblings.iter().enumerate() {
        computed_hash = if proof.path[index] {
            // Sibling on left
            keccak256(&[sibling.as_bytes(), computed_hash.as_bytes()].concat())
        } else {
            // Sibling on right
            keccak256(&[computed_hash.as_bytes(), sibling.as_bytes()].concat())
        };
    }
    
    computed_hash == *root
}
```

### Relayer Incentives

**Relayers earn fees for submitting proofs:**
- Ethereum → BelizeChain: 10 DALLA per transfer
- BelizeChain → Ethereum: 0.005 ETH per transfer
- XCM transfers: 1 DALLA per transfer (subsidized by network)

---

## BelizeX DEX Trading & Liquidity

**Complete guide in Part 2 below**

---

## Part 2: BelizeX DEX (Decentralized Exchange)

### Trading Models

**BelizeX supports two trading models:**

1. **AMM (Automated Market Maker):** Constant product formula (x * y = k)
2. **Order Book:** Traditional limit orders with price-time priority

### Create Trading Pair (AMM Pool)

```javascript
// Create DALLA/bBZD trading pair
await api.tx.belizex.createTradingPair(
  'DALLA',
  'bBZD',
  30  // 0.3% fee (in basis points: 30/10000)
).signAndSend(creator);

// Expected: TradingPairCreated event with pair ID
```

### Add Liquidity (Become LP)

```javascript
// Add 10,000 DALLA + 5,000 bBZD liquidity
const dallaAmount = 10_000 * 1e12;
const bbzdAmount = 5_000 * 1e12;

await api.tx.belizex.addLiquidity(
  pairId,
  dallaAmount,
  bbzdAmount,
  9_500 * 1e12,  // minDallaAmount (5% slippage tolerance)
  4_750 * 1e12   // minbBzdAmount (5% slippage tolerance)
).signAndSend(liquidityProvider);

// Receive LP tokens representing your share of pool
```

**LP token calculation:**
```
First LP: LP_tokens = sqrt(DALLA_amount * bBZD_amount)
          = sqrt(10,000 * 5,000) = sqrt(50,000,000) = 7,071 LP tokens

Subsequent LPs: LP_tokens = min(
  (DALLA_added / DALLA_reserve) * total_LP_supply,
  (bBZD_added / bBZD_reserve) * total_LP_supply
)
```

### Swap Tokens (AMM)

```javascript
// Swap 100 DALLA for bBZD
const dallaIn = 100 * 1e12;

// Calculate expected output (constant product formula)
const reserves = await api.query.belizex.reserves(pairId);
const dallaReserve = reserves.unwrap()[0].toBigInt();
const bbzdReserve = reserves.unwrap()[1].toBigInt();

const dallaInWithFee = dallaIn * 997n;  // 0.3% fee = 0.997 multiplier
const numerator = dallaInWithFee * bbzdReserve;
const denominator = (dallaReserve * 1000n) + dallaInWithFee;
const bbzdOut = numerator / denominator;

console.log(`Expected output: ${bbzdOut / 1e12} bBZD`);

// Execute swap
await api.tx.belizex.swap(
  pairId,
  dallaIn,
  bbzdOut * 95n / 100n,  // minAmountOut with 5% slippage
  ['DALLA', 'bBZD']  // Path
).signAndSend(trader);
```

### Place Limit Order

```javascript
// Sell 1,000 DALLA at 0.52 bBZD per DALLA (limit sell)
await api.tx.belizex.placeLimitOrder(
  pairId,
  'Sell',
  1_000 * 1e12,  // Amount to sell
  520_000_000_000,  // Price (0.52 bBZD with 12 decimals)
  604800  // Expiry: 7 days (in seconds)
).signAndSend(trader);

// Order added to order book, filled when buy orders match price
```

### Remove Liquidity

```javascript
// Remove 1,000 LP tokens
await api.tx.belizex.removeLiquidity(
  pairId,
  1_000 * 1e12,  // LP tokens to burn
  9_500 * 1e12,  // minDallaAmount
  4_750 * 1e12   // minbBzdAmount
).signAndSend(liquidityProvider);

// Receive proportional DALLA + bBZD from pool
```

---

## Part 3: LandLedger (Property Registry)

### Register Property

**Requirements:**
- Enhanced KYC (mandatory for property ownership)
- Property deed or title (upload to Pakit)
- Government approval (Ministry of Natural Resources)

```javascript
// Register 5-acre beachfront property
const documentHash = 'QmXyZ123...';  // Pakit hash of property deed
const location = { latitude: 17.4959, longitude: -88.1883 };  // Placencia

await api.tx.landledger.registerProperty(
  documentHash,
  'Beachfront Lot 42, Placencia',
  5.0,  // Area in acres
  location,
  'Residential'  // Property type
).signAndSend(owner);

// Wait for government approval (Ministry official signs)
// Property NFT minted after approval
```

### Transfer Property

```javascript
// Sell property to buyer (requires government stamp duty: 5%)
const salePrice = 500_000 * 1e12;  // 500,000 DALLA (~$150,000)
const stampDuty = salePrice * 5n / 100n;  // 25,000 DALLA

await api.tx.landledger.transferProperty(
  propertyId,
  newOwner,
  salePrice
).signAndSend(currentOwner, { value: stampDuty });

// Government collects 5% stamp duty
// Property ownership transferred on-chain
```

### Verify Property Title

```javascript
// Anyone can verify property ownership on-chain
const property = await api.query.landledger.properties(propertyId);

console.log(`Owner: ${property.unwrap().owner}`);
console.log(`Document: ${property.unwrap().documentHash}`);
console.log(`Area: ${property.unwrap().area} acres`);
console.log(`Registered: ${new Date(property.unwrap().registeredAt * 1000)}`);

// Download deed from Pakit
const deed = await pakit.download(property.unwrap().documentHash);
```

---

## Related Documentation

- [Interoperability Pallet API](../developer-guides/pallet-apis-infrastructure.md#interoperability-pallet)
- [BelizeX Pallet API](../developer-guides/pallet-apis-financial.md#belizex-pallet)
- [LandLedger Pallet API](../developer-guides/pallet-apis-services.md#landledger-pallet)
- [XCM Documentation](https://wiki.polkadot.network/docs/learn-xcm)
