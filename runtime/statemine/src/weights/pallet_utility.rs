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

//! Autogenerated weights for pallet_utility
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-11, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("statemine-dev"), DB CACHE: 128

// Executed Command:
// ./target/release/statemint
// benchmark
// --chain
// statemine-dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_utility
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
// ./runtime/statemine/src/weights/


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_utility.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	fn batch(c: u32, ) -> Weight {
		(15_812_000 as Weight)
			// Standard Error: 0
			.saturating_add((4_578_000 as Weight).saturating_mul(c as Weight))
	}
	fn as_derivative() -> Weight {
		(7_659_000 as Weight)
	}
	fn batch_all(c: u32, ) -> Weight {
		(16_730_000 as Weight)
			// Standard Error: 0
			.saturating_add((4_573_000 as Weight).saturating_mul(c as Weight))
	}
}
