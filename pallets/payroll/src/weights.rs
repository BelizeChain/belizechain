//! Hand-estimated weights for pallet-belize-payroll
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_payroll using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: Employees (r:1 w:1), NextEmployeeId (r:1 w:1)
    fn add_employee() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Employees (r:1 w:1)
    fn remove_employee() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: Employees (r:1 w:1)
    fn update_salary() -> Weight {
        Weight::from_parts(15_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: Employees (r:1 w:0), Currency::transfer (r:2 w:2), PaymentRecords (r:0 w:1)
    fn execute_payment() -> Weight {
        Weight::from_parts(50_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// O(N) — iterates over N employees
    /// Storage: Employees (r:N w:0), Currency::transfer (r:2*N w:2*N), PaymentRecords (r:0 w:N)
    fn batch_payment(n: u32) -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(Weight::from_parts(50_000_000u64.saturating_mul(n as u64), 0))
            .saturating_add(T::DbWeight::get().reads(3u64.saturating_mul(n as u64)))
            .saturating_add(T::DbWeight::get().writes(3u64.saturating_mul(n as u64)))
    }

    /// Storage: Schedules (r:1 w:1), NextScheduleId (r:1 w:1)
    fn create_schedule() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Schedules (r:1 w:1)
    fn update_schedule() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
