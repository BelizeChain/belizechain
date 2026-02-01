# 🧱 Developing Custom Pallets

**Complete guide to building custom blockchain business logic in Rust**

---

## 📋 What You'll Learn

By the end of this guide, you'll be able to:
- ✅ Understand Substrate pallet architecture
- ✅ Create custom pallets from scratch
- ✅ Implement storage, events, and errors
- ✅ Write dispatchable functions (extrinsics)
- ✅ Add weights and benchmarking
- ✅ Test pallets thoroughly
- ✅ Integrate pallets into runtime

**Time Required**: 3-4 hours  
**Skill Level**: Intermediate to Advanced  
**Prerequisites**: 
- [Development environment set up](setup-environment.md)
- Basic Rust knowledge
- Understanding of blockchain concepts

---

## 🎯 What is a Pallet?

A **pallet** is a Substrate module that encapsulates specific blockchain logic:

**Real-World Analogy**: Think of pallets like LEGO blocks:
- Each pallet is a self-contained building block
- Pallets can interact with each other
- You combine pallets to build a complete blockchain
- You can add/remove pallets without breaking everything

**BelizeChain Examples**:
- **Economy Pallet**: Handles DALLA and bBZD tokens
- **Governance Pallet**: Manages proposals and voting
- **Staking Pallet**: Validator staking and rewards
- **Identity Pallet**: BelizeID and credentials

---

## 🏗️ Pallet Architecture

### **Core Components**

```rust
#[frame_support::pallet]
pub mod pallet {
    // 1. Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Runtime types and constants
    }

    // 2. Storage items
    #[pallet::storage]
    pub type MyStorage<T> = StorageValue<_, u32, ValueQuery>;

    // 3. Events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SomethingHappened { who: T::AccountId, value: u32 },
    }

    // 4. Errors
    #[pallet::error]
    pub enum Error<T> {
        ValueTooLarge,
        NotAuthorized,
    }

    // 5. Callable functions (extrinsics)
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn do_something(origin: OriginFor<T>, value: u32) -> DispatchResult {
            // Function logic
            Ok(())
        }
    }
}
```

---

## 🚀 Quick Start: Your First Pallet (30 Minutes)

Let's build a **Task Manager** pallet where users can:
- Create tasks
- Mark tasks as complete
- Delete tasks

### **Step 1: Create Pallet Structure** (3 minutes)

```bash
# Navigate to pallets directory
cd ~/projects/belizechain/pallets

# Create new pallet directory
mkdir task-manager
cd task-manager

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "pallet-task-manager"
version = "1.0.0"
edition = "2021"

[dependencies]
codec = { package = "parity-scale-codec", version = "3.6.1", default-features = false, features = ["derive"] }
scale-info = { version = "2.5.0", default-features = false, features = ["derive"] }

# Substrate dependencies
frame-support = { git = "https://github.com/paritytech/substrate", branch = "polkadot-v1.0.0", default-features = false }
frame-system = { git = "https://github.com/paritytech/substrate", branch = "polkadot-v1.0.0", default-features = false }
sp-std = { git = "https://github.com/paritytech/substrate", branch = "polkadot-v1.0.0", default-features = false }
sp-runtime = { git = "https://github.com/paritytech/substrate", branch = "polkadot-v1.0.0", default-features = false }

[dev-dependencies]
sp-core = { git = "https://github.com/paritytech/substrate", branch = "polkadot-v1.0.0" }
sp-io = { git = "https://github.com/paritytech/substrate", branch = "polkadot-v1.0.0" }

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
    "sp-std/std",
    "sp-runtime/std",
]
EOF

# Create source directory
mkdir src
```

---

### **Step 2: Define Task Struct** (5 minutes)

Create `src/lib.rs`:

```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    /// Task structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Task<T: Config> {
        /// Task owner
        pub owner: T::AccountId,
        /// Task title (max 100 characters)
        pub title: BoundedVec<u8, ConstU32<100>>,
        /// Task description (max 500 characters)
        pub description: BoundedVec<u8, ConstU32<500>>,
        /// Is task completed?
        pub completed: bool,
        /// Creation timestamp (block number)
        pub created_at: T::BlockNumber,
    }

    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Maximum tasks per user
        type MaxTasksPerUser: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Storage: Tasks indexed by (AccountId, TaskId)
    #[pallet::storage]
    #[pallet::getter(fn tasks)]
    pub type Tasks<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,  // First key: User
        Blake2_128Concat, u32,           // Second key: Task ID
        Task<T>,                         // Value: Task struct
        OptionQuery
    >;

    /// Storage: Next task ID for each user
    #[pallet::storage]
    #[pallet::getter(fn next_task_id)]
    pub type NextTaskId<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        u32,
        ValueQuery  // Defaults to 0
    >;

    /// Events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Task created [owner, task_id]
        TaskCreated { owner: T::AccountId, task_id: u32 },
        /// Task completed [owner, task_id]
        TaskCompleted { owner: T::AccountId, task_id: u32 },
        /// Task deleted [owner, task_id]
        TaskDeleted { owner: T::AccountId, task_id: u32 },
    }

    /// Errors
    #[pallet::error]
    pub enum Error<T> {
        /// Task title too long
        TitleTooLong,
        /// Task description too long
        DescriptionTooLong,
        /// Maximum tasks reached
        TooManyTasks,
        /// Task does not exist
        TaskNotFound,
        /// Not the task owner
        NotTaskOwner,
        /// Task already completed
        AlreadyCompleted,
    }

    /// Dispatchable functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new task
        #[pallet::weight(10_000)]
        #[pallet::call_index(0)]
        pub fn create_task(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Vec<u8>,
        ) -> DispatchResult {
            // Verify signed origin
            let who = ensure_signed(origin)?;

            // Convert Vec to BoundedVec (with length checking)
            let bounded_title: BoundedVec<u8, ConstU32<100>> = 
                title.try_into().map_err(|_| Error::<T>::TitleTooLong)?;
            let bounded_description: BoundedVec<u8, ConstU32<500>> = 
                description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;

            // Get next task ID for this user
            let task_id = NextTaskId::<T>::get(&who);

            // Check max tasks limit
            ensure!(
                task_id < T::MaxTasksPerUser::get(),
                Error::<T>::TooManyTasks
            );

            // Create task
            let task = Task {
                owner: who.clone(),
                title: bounded_title,
                description: bounded_description,
                completed: false,
                created_at: frame_system::Pallet::<T>::block_number(),
            };

            // Store task
            Tasks::<T>::insert(&who, task_id, task);

            // Increment next task ID
            NextTaskId::<T>::insert(&who, task_id + 1);

            // Emit event
            Self::deposit_event(Event::TaskCreated {
                owner: who,
                task_id,
            });

            Ok(())
        }

        /// Mark task as completed
        #[pallet::weight(10_000)]
        #[pallet::call_index(1)]
        pub fn complete_task(
            origin: OriginFor<T>,
            task_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get task
            Tasks::<T>::try_mutate(&who, task_id, |task_opt| -> DispatchResult {
                let task = task_opt.as_mut().ok_or(Error::<T>::TaskNotFound)?;

                // Check if already completed
                ensure!(!task.completed, Error::<T>::AlreadyCompleted);

                // Mark as completed
                task.completed = true;

                // Emit event
                Self::deposit_event(Event::TaskCompleted {
                    owner: who.clone(),
                    task_id,
                });

                Ok(())
            })
        }

        /// Delete a task
        #[pallet::weight(10_000)]
        #[pallet::call_index(2)]
        pub fn delete_task(
            origin: OriginFor<T>,
            task_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Remove task (returns old value or None)
            let task = Tasks::<T>::take(&who, task_id)
                .ok_or(Error::<T>::TaskNotFound)?;

            // Verify ownership (already guaranteed by storage key, but good practice)
            ensure!(task.owner == who, Error::<T>::NotTaskOwner);

            // Emit event
            Self::deposit_event(Event::TaskDeleted {
                owner: who,
                task_id,
            });

            Ok(())
        }
    }
}
```

---

### **Step 3: Add Unit Tests** (12 minutes)

Add tests to `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, parameter_types};
    use sp_core::H256;
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    // Configure test runtime
    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            TaskManager: crate,
        }
    );

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
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
    }

    parameter_types! {
        pub const MaxTasksPerUser: u32 = 100;
    }

    impl Config for Test {
        type MaxTasksPerUser = MaxTasksPerUser;
    }

    // Test helper: create new test externalities
    fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();
        t.into()
    }

    #[test]
    fn create_task_works() {
        new_test_ext().execute_with(|| {
            // Create task
            assert_ok!(TaskManager::create_task(
                RuntimeOrigin::signed(1),
                b"Buy groceries".to_vec(),
                b"Milk, bread, eggs".to_vec(),
            ));

            // Check task exists
            let task = TaskManager::tasks(1, 0).unwrap();
            assert_eq!(task.owner, 1);
            assert_eq!(task.title.to_vec(), b"Buy groceries".to_vec());
            assert_eq!(task.completed, false);

            // Check next task ID incremented
            assert_eq!(TaskManager::next_task_id(1), 1);

            // Check event emitted
            System::assert_has_event(RuntimeEvent::TaskManager(
                Event::TaskCreated { owner: 1, task_id: 0 }
            ));
        });
    }

    #[test]
    fn complete_task_works() {
        new_test_ext().execute_with(|| {
            // Create task
            assert_ok!(TaskManager::create_task(
                RuntimeOrigin::signed(1),
                b"Clean room".to_vec(),
                b"Vacuum and dust".to_vec(),
            ));

            // Complete task
            assert_ok!(TaskManager::complete_task(RuntimeOrigin::signed(1), 0));

            // Check task is completed
            let task = TaskManager::tasks(1, 0).unwrap();
            assert_eq!(task.completed, true);

            // Check event emitted
            System::assert_has_event(RuntimeEvent::TaskManager(
                Event::TaskCompleted { owner: 1, task_id: 0 }
            ));
        });
    }

    #[test]
    fn delete_task_works() {
        new_test_ext().execute_with(|| {
            // Create task
            assert_ok!(TaskManager::create_task(
                RuntimeOrigin::signed(1),
                b"Delete me".to_vec(),
                b"This will be deleted".to_vec(),
            ));

            // Delete task
            assert_ok!(TaskManager::delete_task(RuntimeOrigin::signed(1), 0));

            // Check task no longer exists
            assert!(TaskManager::tasks(1, 0).is_none());

            // Check event emitted
            System::assert_has_event(RuntimeEvent::TaskManager(
                Event::TaskDeleted { owner: 1, task_id: 0 }
            ));
        });
    }

    #[test]
    fn task_not_found_error() {
        new_test_ext().execute_with(|| {
            // Try to complete non-existent task
            assert_noop!(
                TaskManager::complete_task(RuntimeOrigin::signed(1), 0),
                Error::<Test>::TaskNotFound
            );

            // Try to delete non-existent task
            assert_noop!(
                TaskManager::delete_task(RuntimeOrigin::signed(1), 0),
                Error::<Test>::TaskNotFound
            );
        });
    }

    #[test]
    fn already_completed_error() {
        new_test_ext().execute_with(|| {
            // Create and complete task
            assert_ok!(TaskManager::create_task(
                RuntimeOrigin::signed(1),
                b"Task".to_vec(),
                b"Description".to_vec(),
            ));
            assert_ok!(TaskManager::complete_task(RuntimeOrigin::signed(1), 0));

            // Try to complete again
            assert_noop!(
                TaskManager::complete_task(RuntimeOrigin::signed(1), 0),
                Error::<Test>::AlreadyCompleted
            );
        });
    }

    #[test]
    fn title_too_long_error() {
        new_test_ext().execute_with(|| {
            // Create task with title > 100 characters
            let long_title = vec![b'x'; 101];
            assert_noop!(
                TaskManager::create_task(
                    RuntimeOrigin::signed(1),
                    long_title,
                    b"Description".to_vec(),
                ),
                Error::<Test>::TitleTooLong
            );
        });
    }
}
```

---

### **Step 4: Test Your Pallet** (5 minutes)

```bash
# Run tests
cargo test -p pallet-task-manager

# Should see:
# running 6 tests
# test tests::create_task_works ... ok
# test tests::complete_task_works ... ok
# test tests::delete_task_works ... ok
# test tests::task_not_found_error ... ok
# test tests::already_completed_error ... ok
# test tests::title_too_long_error ... ok
#
# test result: ok. 6 passed; 0 failed
```

**🎉 Your first pallet is complete and tested!**

---

### **Step 5: Integrate into Runtime** (5 minutes)

Edit `belizechain/runtime/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...

# Add your pallet
pallet-task-manager = { path = "../pallets/task-manager", default-features = false }

[features]
std = [
    # ... existing features ...
    "pallet-task-manager/std",
]
```

Edit `belizechain/runtime/src/lib.rs`:

```rust
// Add configuration
parameter_types! {
    pub const MaxTasksPerUser: u32 = 100;
}

impl pallet_task_manager::Config for Runtime {
    type MaxTasksPerUser = MaxTasksPerUser;
}

// Add to construct_runtime! macro
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // ... existing pallets ...
        
        // Add your pallet
        TaskManager: pallet_task_manager,
    }
);
```

---

## 🔍 Deep Dive: Storage Patterns

### **1. StorageValue** - Single Global Value

```rust
/// Store a single value (e.g., total task count)
#[pallet::storage]
pub type TotalTasks<T> = StorageValue<_, u32, ValueQuery>;

// Usage:
let total = TotalTasks::<T>::get();
TotalTasks::<T>::put(100);
TotalTasks::<T>::mutate(|val| *val += 1);
```

**Use cases**: Counters, global settings, flags

---

### **2. StorageMap** - Key-Value Store

```rust
/// Map AccountId to balance
#[pallet::storage]
pub type Balances<T: Config> = StorageMap<
    _,
    Blake2_128Concat,      // Hasher
    T::AccountId,          // Key
    u128,                  // Value
    ValueQuery             // Default to 0
>;

// Usage:
let balance = Balances::<T>::get(&account);
Balances::<T>::insert(&account, 1000);
Balances::<T>::remove(&account);
```

**Use cases**: Account data, balances, ownership

---

### **3. StorageDoubleMap** - Two-Key Map

```rust
/// Map (AccountId, Token ID) to balance
#[pallet::storage]
pub type TokenBalances<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,  // Key 1
    Blake2_128Concat, u32,           // Key 2
    u128,                            // Value
    ValueQuery
>;

// Usage:
let balance = TokenBalances::<T>::get(&account, &token_id);
TokenBalances::<T>::insert(&account, &token_id, 100);
```

**Use cases**: Multi-dimensional data, relationships

---

### **4. StorageNMap** - N-Key Map

```rust
/// Map (AccountId, Token ID, Timestamp) to value
#[pallet::storage]
pub type MultiKeyData<T: Config> = StorageNMap<
    _,
    (
        NMapKey<Blake2_128Concat, T::AccountId>,
        NMapKey<Blake2_128Concat, u32>,
        NMapKey<Blake2_128Concat, u64>,
    ),
    u128,
    ValueQuery
>;
```

**Use cases**: Complex queries, historical data

---

## 🎯 Advanced Pallet Patterns

### **Pattern 1: Weighted Transactions**

Transactions consume computational resources. We must specify **weights**:

```rust
// Simple weight (computational cost in picoseconds)
#[pallet::weight(10_000)]
pub fn simple_function(origin: OriginFor<T>) -> DispatchResult {
    // ...
    Ok(())
}

// Weight with database reads/writes
#[pallet::weight(
    Weight::from_parts(25_000_000, 0)
        .saturating_add(T::DbWeight::get().reads(2))
        .saturating_add(T::DbWeight::get().writes(1))
)]
pub fn complex_function(origin: OriginFor<T>) -> DispatchResult {
    // Reads 2 storage items
    // Writes 1 storage item
    Ok(())
}
```

**Best practices**:
- Measure actual costs with benchmarking
- Overestimate slightly to be safe
- Use `T::DbWeight` for database operations
- Account for loops and conditionals

---

### **Pattern 2: Genesis Configuration**

Initialize storage at blockchain genesis:

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub initial_tasks: Vec<(T::AccountId, Vec<u8>, Vec<u8>)>,
}

#[cfg(feature = "std")]
impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
        Self {
            initial_tasks: vec![],
        }
    }
}

#[pallet::genesis_build]
impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
        for (owner, title, desc) in &self.initial_tasks {
            let bounded_title: BoundedVec<u8, ConstU32<100>> = 
                title.clone().try_into().expect("Title too long");
            let bounded_desc: BoundedVec<u8, ConstU32<500>> = 
                desc.clone().try_into().expect("Description too long");
            
            let task = Task {
                owner: owner.clone(),
                title: bounded_title,
                description: bounded_desc,
                completed: false,
                created_at: 0u32.into(),
            };
            
            Tasks::<T>::insert(owner, 0, task);
            NextTaskId::<T>::insert(owner, 1);
        }
    }
}
```

---

### **Pattern 3: Hooks (On Initialize/Finalize)**

Run code at specific points in block lifecycle:

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// Called at the beginning of every block
    fn on_initialize(n: T::BlockNumber) -> Weight {
        log::info!("Block #{:?} starting", n);
        
        // Example: Delete expired tasks
        // (implementation would iterate storage)
        
        Weight::from_parts(10_000, 0)
    }

    /// Called at the end of every block
    fn on_finalize(n: T::BlockNumber) {
        log::info!("Block #{:?} ending", n);
    }

    /// Called when runtime is upgraded
    fn on_runtime_upgrade() -> Weight {
        log::info!("Runtime upgrading - migrating storage");
        
        // Example: Migrate old storage format to new format
        
        Weight::from_parts(100_000, 0)
    }
}
```

---

### **Pattern 4: Inter-Pallet Communication**

Pallets can call other pallets:

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// Access to another pallet
    type Currency: Currency<Self::AccountId>;
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(10_000)]
    pub fn create_premium_task(origin: OriginFor<T>, title: Vec<u8>) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Charge fee using Currency pallet
        let fee = 100u32.into();
        T::Currency::withdraw(
            &who,
            fee,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )?;
        
        // Create task...
        
        Ok(())
    }
}
```

---

### **Pattern 5: Custom Origins**

Create special permission levels:

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// Origin that can delete any task (e.g., governance)
    type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// Admin-only function
    #[pallet::weight(10_000)]
    pub fn admin_delete_task(
        origin: OriginFor<T>,
        owner: T::AccountId,
        task_id: u32,
    ) -> DispatchResult {
        // Only admins can call this
        T::AdminOrigin::ensure_origin(origin)?;
        
        Tasks::<T>::remove(&owner, task_id);
        
        Ok(())
    }
}
```

---

## 📊 Real-World Example: Crowdfunding Pallet

Let's build a complete crowdfunding pallet:

```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{CheckedAdd, Saturating};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Campaign status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum CampaignStatus {
        Active,
        Successful,
        Failed,
        Cancelled,
    }

    /// Campaign struct
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Campaign<T: Config> {
        pub creator: T::AccountId,
        pub title: BoundedVec<u8, ConstU32<100>>,
        pub description: BoundedVec<u8, ConstU32<1000>>,
        pub goal: BalanceOf<T>,
        pub raised: BalanceOf<T>,
        pub deadline: T::BlockNumber,
        pub status: CampaignStatus,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: ReservableCurrency<Self::AccountId>;
        
        /// Minimum campaign goal
        #[pallet::constant]
        type MinimumGoal: Get<BalanceOf<Self>>;
        
        /// Minimum campaign duration (in blocks)
        #[pallet::constant]
        type MinimumDuration: Get<Self::BlockNumber>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Campaigns storage
    #[pallet::storage]
    pub type Campaigns<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // Campaign ID
        Campaign<T>,
        OptionQuery
    >;

    /// Next campaign ID
    #[pallet::storage]
    pub type NextCampaignId<T> = StorageValue<_, u32, ValueQuery>;

    /// Contributions: (CampaignId, Contributor) => Amount
    #[pallet::storage]
    pub type Contributions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u32,
        Blake2_128Concat, T::AccountId,
        BalanceOf<T>,
        ValueQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CampaignCreated { campaign_id: u32, creator: T::AccountId, goal: BalanceOf<T> },
        ContributionMade { campaign_id: u32, contributor: T::AccountId, amount: BalanceOf<T> },
        CampaignSuccessful { campaign_id: u32, total_raised: BalanceOf<T> },
        CampaignFailed { campaign_id: u32 },
        FundsWithdrawn { campaign_id: u32, recipient: T::AccountId, amount: BalanceOf<T> },
        RefundIssued { campaign_id: u32, contributor: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        GoalTooLow,
        DurationTooShort,
        CampaignNotFound,
        CampaignNotActive,
        CampaignStillActive,
        GoalNotReached,
        NotCampaignCreator,
        AlreadyWithdrawn,
        NoContribution,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new crowdfunding campaign
        #[pallet::weight(10_000)]
        #[pallet::call_index(0)]
        pub fn create_campaign(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Vec<u8>,
            goal: BalanceOf<T>,
            duration: T::BlockNumber,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            // Validate inputs
            ensure!(goal >= T::MinimumGoal::get(), Error::<T>::GoalTooLow);
            ensure!(duration >= T::MinimumDuration::get(), Error::<T>::DurationTooShort);

            let bounded_title: BoundedVec<u8, ConstU32<100>> = 
                title.try_into().map_err(|_| Error::<T>::GoalTooLow)?;  // Reuse error
            let bounded_description: BoundedVec<u8, ConstU32<1000>> = 
                description.try_into().map_err(|_| Error::<T>::GoalTooLow)?;

            // Calculate deadline
            let current_block = frame_system::Pallet::<T>::block_number();
            let deadline = current_block.saturating_add(duration);

            // Create campaign
            let campaign_id = NextCampaignId::<T>::get();
            let campaign = Campaign {
                creator: creator.clone(),
                title: bounded_title,
                description: bounded_description,
                goal,
                raised: 0u32.into(),
                deadline,
                status: CampaignStatus::Active,
            };

            Campaigns::<T>::insert(campaign_id, campaign);
            NextCampaignId::<T>::put(campaign_id + 1);

            Self::deposit_event(Event::CampaignCreated {
                campaign_id,
                creator,
                goal,
            });

            Ok(())
        }

        /// Contribute to a campaign
        #[pallet::weight(10_000)]
        #[pallet::call_index(1)]
        pub fn contribute(
            origin: OriginFor<T>,
            campaign_id: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let contributor = ensure_signed(origin)?;

            Campaigns::<T>::try_mutate(campaign_id, |campaign_opt| -> DispatchResult {
                let campaign = campaign_opt.as_mut().ok_or(Error::<T>::CampaignNotFound)?;

                // Check campaign is active
                ensure!(campaign.status == CampaignStatus::Active, Error::<T>::CampaignNotActive);

                // Check deadline not passed
                let current_block = frame_system::Pallet::<T>::block_number();
                ensure!(current_block < campaign.deadline, Error::<T>::CampaignNotActive);

                // Reserve funds (locks them until campaign ends)
                T::Currency::reserve(&contributor, amount)?;

                // Update raised amount
                campaign.raised = campaign.raised.checked_add(&amount)
                    .ok_or(Error::<T>::GoalTooLow)?;  // Reuse error

                // Track contribution
                Contributions::<T>::mutate(campaign_id, &contributor, |contrib| {
                    *contrib = contrib.saturating_add(amount);
                });

                Self::deposit_event(Event::ContributionMade {
                    campaign_id,
                    contributor,
                    amount,
                });

                Ok(())
            })
        }

        /// Finalize campaign (anyone can call after deadline)
        #[pallet::weight(10_000)]
        #[pallet::call_index(2)]
        pub fn finalize_campaign(
            origin: OriginFor<T>,
            campaign_id: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Campaigns::<T>::try_mutate(campaign_id, |campaign_opt| -> DispatchResult {
                let campaign = campaign_opt.as_mut().ok_or(Error::<T>::CampaignNotFound)?;

                // Check campaign is active
                ensure!(campaign.status == CampaignStatus::Active, Error::<T>::CampaignNotActive);

                // Check deadline passed
                let current_block = frame_system::Pallet::<T>::block_number();
                ensure!(current_block >= campaign.deadline, Error::<T>::CampaignStillActive);

                // Check if goal reached
                if campaign.raised >= campaign.goal {
                    campaign.status = CampaignStatus::Successful;
                    Self::deposit_event(Event::CampaignSuccessful {
                        campaign_id,
                        total_raised: campaign.raised,
                    });
                } else {
                    campaign.status = CampaignStatus::Failed;
                    Self::deposit_event(Event::CampaignFailed { campaign_id });
                }

                Ok(())
            })
        }

        /// Withdraw funds (creator, after successful campaign)
        #[pallet::weight(10_000)]
        #[pallet::call_index(3)]
        pub fn withdraw_funds(
            origin: OriginFor<T>,
            campaign_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let campaign = Campaigns::<T>::get(campaign_id)
                .ok_or(Error::<T>::CampaignNotFound)?;

            // Check creator
            ensure!(campaign.creator == who, Error::<T>::NotCampaignCreator);

            // Check campaign successful
            ensure!(campaign.status == CampaignStatus::Successful, Error::<T>::GoalNotReached);

            // Transfer all reserved funds to creator
            // (In reality, would iterate Contributions and unreserve + transfer)
            // Simplified for example
            
            Self::deposit_event(Event::FundsWithdrawn {
                campaign_id,
                recipient: who,
                amount: campaign.raised,
            });

            Ok(())
        }

        /// Claim refund (contributor, after failed campaign)
        #[pallet::weight(10_000)]
        #[pallet::call_index(4)]
        pub fn claim_refund(
            origin: OriginFor<T>,
            campaign_id: u32,
        ) -> DispatchResult {
            let contributor = ensure_signed(origin)?;

            let campaign = Campaigns::<T>::get(campaign_id)
                .ok_or(Error::<T>::CampaignNotFound)?;

            // Check campaign failed
            ensure!(campaign.status == CampaignStatus::Failed, Error::<T>::GoalNotReached);

            // Get contribution amount
            let amount = Contributions::<T>::take(campaign_id, &contributor);
            ensure!(amount > 0u32.into(), Error::<T>::NoContribution);

            // Unreserve funds (returns them to contributor)
            T::Currency::unreserve(&contributor, amount);

            Self::deposit_event(Event::RefundIssued {
                campaign_id,
                contributor,
                amount,
            });

            Ok(())
        }
    }
}
```

**This pallet demonstrates**:
- ✅ Complex storage (campaigns, contributions)
- ✅ Currency integration (reserving/unreserving funds)
- ✅ Time-based logic (deadlines)
- ✅ State machines (campaign status)
- ✅ Economic incentives (all-or-nothing funding)

---

## 🧪 Testing Best Practices

### **1. Test Coverage Checklist**

- [ ] **Happy path**: Normal operations work correctly
- [ ] **Edge cases**: Boundary conditions (max values, empty inputs)
- [ ] **Error cases**: All error types can be triggered
- [ ] **Permissions**: Unauthorized users can't call restricted functions
- [ ] **State transitions**: All status changes work correctly
- [ ] **Events**: All events are emitted correctly
- [ ] **Storage**: Data persists correctly across calls

### **2. Test Organization**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Group 1: Basic functionality
    mod basic_operations {
        use super::*;
        
        #[test]
        fn create_works() { /* ... */ }
        
        #[test]
        fn update_works() { /* ... */ }
        
        #[test]
        fn delete_works() { /* ... */ }
    }
    
    // Group 2: Error handling
    mod error_cases {
        use super::*;
        
        #[test]
        fn not_found_error() { /* ... */ }
        
        #[test]
        fn unauthorized_error() { /* ... */ }
    }
    
    // Group 3: Complex scenarios
    mod integration {
        use super::*;
        
        #[test]
        fn full_lifecycle() { /* ... */ }
    }
}
```

---

## 🚀 Production Checklist

Before deploying your pallet:

- [ ] **Code review**: At least 2 reviewers
- [ ] **100% test coverage**: All lines tested
- [ ] **Fuzzing**: Random input testing
- [ ] **Benchmarking**: Accurate weight functions
- [ ] **Documentation**: All public functions documented
- [ ] **Security audit**: Professional security review
- [ ] **Upgrade path**: Storage migration plan
- [ ] **Emergency stops**: Circuit breakers implemented
- [ ] **Monitoring**: Logging and metrics
- [ ] **User documentation**: End-user guides

---

## 📚 Next Steps

**Continue Learning**:
1. **[API Reference →](api-reference.md)**  
   Complete API documentation

2. **[Testing Guide →](testing.md)**  
   Advanced testing strategies

3. **[Benchmarking →](../technical-reference/benchmarking.md)**  
   Measure pallet performance

**Join Community**:
- Discord: https://discord.gg/belizechain
- Forum: https://forum.belizechain.org
- GitHub: https://github.com/BelizeChain/belizechain

---

**🎉 You're now a Substrate pallet developer!**

**Questions?** Join our [Discord](https://discord.gg/belizechain) for support.

**Happy building!** 🚀🇧🇿
