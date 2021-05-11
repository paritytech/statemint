// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Statemint is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Statemint is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Statemint.  If not, see <http://www.gnu.org/licenses/>.

//! Autogenerated weights for pallet_proxy
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-11, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// pallet_proxy
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --raw
// --heap-pages=4096
// --header=./file_header.txt
// --output
// ./runtime/statemint/src/weights/


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_proxy.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::WeightInfo for WeightInfo<T> {
	fn proxy(p: u32, ) -> Weight {
		(27_012_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((207_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn proxy_announced(a: u32, p: u32, ) -> Weight {
		(58_688_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((545_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 1_000
			.saturating_add((217_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_announcement(a: u32, p: u32, ) -> Weight {
		(37_488_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((551_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 1_000
			.saturating_add((26_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn reject_announcement(a: u32, p: u32, ) -> Weight {
		(37_701_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((547_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 1_000
			.saturating_add((24_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn announce(a: u32, p: u32, ) -> Weight {
		(51_982_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((561_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 1_000
			.saturating_add((228_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn add_proxy(p: u32, ) -> Weight {
		(36_225_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((226_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_proxy(p: u32, ) -> Weight {
		(35_667_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((248_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_proxies(p: u32, ) -> Weight {
		(34_455_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((213_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn anonymous(p: u32, ) -> Weight {
		(50_486_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((48_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn kill_anonymous(p: u32, ) -> Weight {
		(36_875_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((206_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
