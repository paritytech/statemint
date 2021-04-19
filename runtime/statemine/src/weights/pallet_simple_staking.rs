
//! Autogenerated weights for pallet_simple_staking
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-04-16, STEPS: `[20, ]`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("statemint-dev"), DB CACHE: 128

// Executed Command:
// ./target/release/statemint
// benchmark
// --chain
// statemint-dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_simple_staking
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --raw
// --output
// ./


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_simple_staking.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_simple_staking::WeightInfo for WeightInfo<T> {
	fn set_invulnerables(b: u32, ) -> Weight {
		(28_060_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((118_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_allowed_author_count() -> Weight {
		(25_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_author_bond() -> Weight {
		(25_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn register_as_author(c: u32, ) -> Weight {
		(82_496_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((266_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn leave_intent(c: u32, ) -> Weight {
		(65_836_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((273_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn note_author() -> Weight {
		(97_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
