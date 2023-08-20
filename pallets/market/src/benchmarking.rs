//! Benchmarking setup for sugarfunge-market
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Market;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

// SBP-M1 review: missing benchmarks for dispatchable functions
// SBP-M1 review: add ci to require successful benchmark tests before merging
#[benchmarks]
mod benchmarks {
    use super::*;

    impl_benchmark_test_suite!(Market, crate::mock::new_test_ext(), crate::mock::Test);
}
