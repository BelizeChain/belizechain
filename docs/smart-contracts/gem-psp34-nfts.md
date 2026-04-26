# PSP34 Non-Fungible Tokens - BeLi NFT Standard

PSP34 is BelizeChain's NFT standard, equivalent to Ethereum's ERC-721. The BeLi NFT contract showcases Belizean cultural artifacts and landmarks.

## Standard Overview

PSP34 defines a common interface for non-fungible tokens:

```rust
pub trait PSP34 {
    fn collection_id(&self) -> Id;
    fn balance_of(&self, owner: AccountId) -> u32;
    fn owner_of(&self, id: Id) -> Option<AccountId>;
    fn total_supply(&self) -> u128;
    fn transfer(&mut self, to: AccountId, id: Id) -> Result<()>;
    fn approve(&mut self, to: AccountId, id: Option<Id>) -> Result<()>;
}
```

## BeLi NFT Implementation

### Contract Details

- **Collection Name**: BeLi (Belizean Cultural Heritage)
- **Symbol**: BELI
- **Token Type**: PSP34 (NFT)
- **Supply**: Dynamic (mintable by owner)
- **Address**: `5Ho6Ks...iFQL7` (testnet)

### Featured NFTs

1. **Great Blue Hole** (#1) - UNESCO World Heritage dive site
2. **Xunantunich Temple** (#2) - Ancient Maya pyramid
3. **Belize Barrier Reef** (#3) - Second-largest barrier reef globally
4. **Jaguar** (#4) - National animal of Belize
5. **Toucan** (#5) - National bird of Belize

### Key Features

#### 1. Minting NFTs
```rust
// Mint new NFT with metadata URI
pub fn mint(&mut self, to: AccountId, token_uri: String) -> Result<TokenId>
```

**Example:**
```javascript
const { GemSDK } = require('@belizechain/gem-sdk');

// Mint Great Blue Hole NFT
const tokenId = await sdk.beliMint(
    contractAddress,
    owner,
    bob.address,
    'ipfs://QmXYZ.../great-blue-hole.json' // Metadata URI
);

console.log(`Minted NFT #${tokenId}`);
```

**Gas Cost**: ~45,000 units (~0.000045 DALLA)

#### 2. NFT Transfers
```rust
// Transfer NFT from caller to recipient
pub fn transfer(&mut self, to: AccountId, id: TokenId) -> Result<()>
```

**Example:**
```javascript
// Transfer NFT #1 to Alice
await sdk.beliTransfer(
    contractAddress,
    bob,
    alice.address,
    1 // Token ID
);
```

**Gas Cost**: ~18,000 units

#### 3. Approval System
```rust
// Approve address to manage specific NFT
pub fn approve(&mut self, to: AccountId, id: TokenId) -> Result<()>

// Approve operator for all NFTs
pub fn set_approval_for_all(&mut self, operator: AccountId, approved: bool) -> Result<()>
```

**Example:**
```javascript
// Approve marketplace to sell NFT #1
await sdk.beliApprove(
    contractAddress,
    alice,
    marketplaceAddress,
    1
);

// Approve operator for all Alice's NFTs
await sdk.beliSetApprovalForAll(
    contractAddress,
    alice,
    operatorAddress,
    true
);
```

#### 4. Metadata & Enumeration
```rust
// Get token metadata URI
pub fn token_uri(&self, id: TokenId) -> Option<String>

// Get token owner
pub fn owner_of(&self, id: TokenId) -> Option<AccountId>

// Get total NFT count
pub fn total_supply(&self) -> u32

// Get NFTs owned by account
pub fn balance_of(&self, owner: AccountId) -> u32
```

**Example:**
```javascript
// Get NFT metadata
const uri = await sdk.beliTokenURI(contractAddress, 1);
const metadata = await fetch(uri).then(r => r.json());

console.log(metadata);
// {
//   "name": "Great Blue Hole",
//   "description": "UNESCO World Heritage dive site...",
//   "image": "ipfs://QmABC.../great-blue-hole.png",
//   "attributes": [
//     { "trait_type": "Location", "value": "Lighthouse Reef" },
//     { "trait_type": "Type", "value": "Natural Wonder" },
//     { "trait_type": "Rarity", "value": "Legendary" }
//   ]
// }

// Get all Alice's NFTs
const balance = await sdk.beliBalanceOf(contractAddress, alice.address);
console.log(`Alice owns ${balance} BeLi NFTs`);
```

### Events

```rust
#[ink(event)]
pub struct Transfer {
    #[ink(topic)]
    from: Option<AccountId>,  // None for minting
    #[ink(topic)]
    to: Option<AccountId>,    // None for burning
    #[ink(topic)]
    id: TokenId,
}

#[ink(event)]
pub struct Approval {
    #[ink(topic)]
    owner: AccountId,
    #[ink(topic)]
    approved: AccountId,
    #[ink(topic)]
    id: TokenId,
}

#[ink(event)]
pub struct ApprovalForAll {
    #[ink(topic)]
    owner: AccountId,
    #[ink(topic)]
    operator: AccountId,
    approved: bool,
}
```

## Metadata Standard

BeLi NFTs follow the ERC-721 metadata schema:

```json
{
  "name": "Great Blue Hole",
  "description": "A giant marine sinkhole off the coast of Belize, part of the Lighthouse Reef system. UNESCO World Heritage Site and one of the world's most spectacular dive sites.",
  "image": "ipfs://QmXYZ.../great-blue-hole.png",
  "external_url": "https://explorer.belizechain.org/nft/beli/1",
  "attributes": [
    {
      "trait_type": "Location",
      "value": "Lighthouse Reef, Belize"
    },
    {
      "trait_type": "Type",
      "value": "Natural Wonder"
    },
    {
      "trait_type": "Rarity",
      "value": "Legendary"
    },
    {
      "trait_type": "Year Discovered",
      "value": "1971",
      "display_type": "number"
    },
    {
      "trait_type": "Depth",
      "value": "124 meters",
      "display_type": "string"
    }
  ]
}
```

**Storage**: Metadata stored on IPFS with Pakit DAG backup for sovereignty.

## Integration Patterns

### 1. NFT Marketplace
```rust
#[ink::contract]
mod marketplace {
    use beli_nft::BeliNft;
    
    #[ink(storage)]
    pub struct Marketplace {
        beli_contract: AccountId,
        listings: Mapping<TokenId, Listing>,
    }
    
    #[derive(scale::Encode, scale::Decode)]
    pub struct Listing {
        seller: AccountId,
        price: u128,
        active: bool,
    }
    
    impl Marketplace {
        #[ink(message)]
        pub fn list_nft(&mut self, token_id: TokenId, price: u128) -> Result<()> {
            let beli: BeliNft = self.get_beli_contract();
            let caller = self.env().caller();
            
            // Verify caller owns NFT
            ensure!(beli.owner_of(token_id) == Some(caller), Error::NotOwner);
            
            // Create listing
            self.listings.insert(token_id, &Listing {
                seller: caller,
                price,
                active: true,
            });
            
            Ok(())
        }
        
        #[ink(message, payable)]
        pub fn buy_nft(&mut self, token_id: TokenId) -> Result<()> {
            let listing = self.listings.get(token_id).ok_or(Error::NotListed)?;
            ensure!(listing.active, Error::ListingInactive);
            
            let payment = self.env().transferred_value();
            ensure!(payment >= listing.price, Error::InsufficientPayment);
            
            // Transfer NFT to buyer
            let beli: BeliNft = self.get_beli_contract();
            beli.transfer(self.env().caller(), token_id)?;
            
            // Transfer payment to seller
            self.env().transfer(listing.seller, listing.price)?;
            
            // Mark listing as sold
            self.listings.remove(token_id);
            
            Ok(())
        }
    }
}
```

### 2. Land Title Registry (BelizeChain LandLedger)
```rust
#[ink(message)]
pub fn mint_land_title(
    &mut self,
    owner: AccountId,
    coordinates: String,
    area_sqm: u64,
) -> Result<TokenId> {
    self.ensure_government_authority()?;
    
    // Create metadata JSON
    let metadata = format!(
        r#"{{"name":"Land Parcel","coordinates":"{}","area":{},"type":"residential"}}"#,
        coordinates, area_sqm
    );
    
    // Store on IPFS via Pakit
    let ipfs_uri = self.pakit_upload(metadata.as_bytes())?;
    
    // Mint NFT representing land title
    let token_id = self.next_token_id;
    self.mint_impl(owner, ipfs_uri)?;
    
    Ok(token_id)
}
```

### 3. Gaming Assets
```rust
#[ink(message)]
pub fn mint_game_item(
    &mut self,
    player: AccountId,
    item_type: ItemType,
    rarity: Rarity,
) -> Result<TokenId> {
    let metadata = GameMetadata {
        name: item_type.name(),
        rarity,
        stats: self.generate_stats(item_type, rarity),
    };
    
    let uri = self.upload_metadata(metadata)?;
    self.mint(player, uri)
}
```

## Deployment Guide

### 1. Build Contract
```bash
cd gem/beli_nft
cargo contract build --release
```

### 2. Deploy via CLI
```bash
cargo contract instantiate \
    --suri //Alice \
    --constructor new \
    --args "BeLi" "BELI" \  # name, symbol
    --url wss://<current-public-testnet-rpc>
```

### 3. Deploy via SDK
```javascript
const { address } = await sdk.deployContract(
    wasmCode,
    metadata,
    {
        constructor: 'new',
        args: ['BeLi', 'BELI'],
    },
    alice
);
```

## Security Considerations

### 1. Ownership Verification
```rust
fn ensure_owner_or_approved(&self, token_id: TokenId) -> Result<()> {
    let caller = self.env().caller();
    let owner = self.token_owner.get(token_id).ok_or(Error::TokenNotFound)?;
    
    if caller == owner {
        return Ok(());
    }
    
    if self.token_approvals.get(token_id) == Some(caller) {
        return Ok(());
    }
    
    if self.operator_approvals.get((owner, caller)).is_some() {
        return Ok(());
    }
    
    Err(Error::NotAuthorized)
}
```

### 2. Safe Transfers
```rust
pub fn safe_transfer(&mut self, to: AccountId, id: TokenId) -> Result<()> {
    // Prevent transfers to zero address
    ensure!(to != AccountId::from([0u8; 32]), Error::InvalidRecipient);
    
    // Prevent transfers to contract address (could lock NFT)
    ensure!(to != self.env().account_id(), Error::InvalidRecipient);
    
    self.transfer(to, id)
}
```

### 3. Metadata Immutability
Store metadata on IPFS (content-addressed):
- URI: `ipfs://Qm...` (hash-based, immutable)
- Backup: Pakit DAG storage for sovereignty
- Never use mutable HTTP URLs

## Testing

```rust
#[cfg(test)]
mod tests {
    #[ink::test]
    fn minting_works() {
        let mut nft = BeliNft::new("BeLi".to_string(), "BELI".to_string());
        let accounts = ink::env::test::default_accounts::<Environment>();
        
        let uri = "ipfs://QmXYZ.../metadata.json".to_string();
        let token_id = nft.mint(accounts.bob, uri.clone()).unwrap();
        
        assert_eq!(nft.owner_of(token_id), Some(accounts.bob));
        assert_eq!(nft.token_uri(token_id), Some(uri));
        assert_eq!(nft.total_supply(), 1);
    }
    
    #[ink::test]
    fn transfer_works() {
        let mut nft = BeliNft::new("BeLi".to_string(), "BELI".to_string());
        let accounts = ink::env::test::default_accounts::<Environment>();
        
        let token_id = nft.mint(accounts.alice, "ipfs://...".to_string()).unwrap();
        
        // Alice transfers to Bob
        assert_eq!(nft.transfer(accounts.bob, token_id), Ok(()));
        
        assert_eq!(nft.owner_of(token_id), Some(accounts.bob));
        assert_eq!(nft.balance_of(accounts.alice), 0);
        assert_eq!(nft.balance_of(accounts.bob), 1);
    }
}
```

## Resources

- **Contract Source**: [github.com/BelizeChain/gem/tree/main/beli_nft](https://github.com/BelizeChain/gem/tree/main/beli_nft)
- **PSP34 Standard**: [github.com/w3f/PSPs/blob/master/PSPs/psp-34.md](https://github.com/w3f/PSPs/blob/master/PSPs/psp-34.md)
- **Metadata Guide**: [ERC-721 Metadata JSON Schema](https://eips.ethereum.org/EIPS/eip-721)
- **Live Contract**: `5Ho6Ks...iFQL7` on testnet
