// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Benchmarking setup for pallet-parachain-staking

use super::*;

#[allow(unused)]
use crate::Pallet as ParachainStaking;
use sp_std::prelude::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, account};
use frame_system::{RawOrigin, EventRecord};
use frame_support::{
	assert_ok,
	traits::{Currency, Get, EnsureOrigin},
};
use pallet_authorship::EventHandler;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const SEED: u32 = 0;

// TODO: remove if this is given in substrate commit.
macro_rules! whitelist {
	($acc:ident) => {
		frame_benchmarking::benchmarking::add_to_whitelist(
			frame_system::Account::<T>::hashed_key_for(&$acc).into()
		);
	};
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn register_candidates<T: Config>(count: u32) {
	let candidates = (0..count).map(|c| account("candidate", c, SEED)).collect::<Vec<_>>();
	assert!(<CandidacyBond<T>>::get() > 0u32.into(), "Bond cannot be zero!");
	for who in candidates {
		T::Currency::make_free_balance_be(&who, <CandidacyBond<T>>::get() * 2u32.into());
		<ParachainStaking<T>>::register_as_candidate(RawOrigin::Signed(who).into()).unwrap();
	}
}

benchmarks! {
	where_clause { where T: pallet_authorship::Config }

	set_invulnerables {
		let b in 1 .. T::MaxInvulnerables::get();
		let new_invulnerables = (0..b).map(|c| account("candidate", c, SEED)).collect::<Vec<_>>();
		let origin = T::UpdateOrigin::successful_origin();
	}: {
		assert_ok!(
			<ParachainStaking<T>>::set_invulnerables(origin, new_invulnerables.clone())
		);
	}
	verify {
		assert_last_event::<T>(Event::NewInvulnerables(new_invulnerables).into());
	}

	aet_max_candidate {
		let max: u32 = 999;
		let origin = T::UpdateOrigin::successful_origin();
	}: {
		assert_ok!(
			<ParachainStaking<T>>::set_desired_candidates(origin, max.clone())
		);
	}
	verify {
		assert_last_event::<T>(Event::NewDesiredCandidates(max).into());
	}

	set_candidacy_bond {
		let bond: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let origin = T::UpdateOrigin::successful_origin();
	}: {
		assert_ok!(
			<ParachainStaking<T>>::set_candidacy_bond(origin, bond.clone())
		);
	}
	verify {
		assert_last_event::<T>(Event::NewCandidacyBond(bond).into());
	}

	// worse case is when we have all the max-candidate slots filled except one, and we fill that
	// one.
	register_as_candidate {
		let c in 1 .. T::MaxCandidates::get();

		<CandidacyBond<T>>::put(T::Currency::minimum_balance());
		<DesiredCandidates<T>>::put(c + 1);
		register_candidates::<T>(c);

		let caller: T::AccountId = whitelisted_caller();
		let bond: BalanceOf<T> = T::Currency::minimum_balance() * 2u32.into();
		T::Currency::make_free_balance_be(&caller, bond.clone());

	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::CandidateAdded(caller, bond / 2u32.into()).into());
	}

	// worse case is the last candidate leaving.
	leave_intent {
		let c in 1 .. T::MaxCandidates::get();
		<CandidacyBond<T>>::put(T::Currency::minimum_balance());
		<DesiredCandidates<T>>::put(c);
		register_candidates::<T>(c);

		let leaving = <Candidates<T>>::get().last().unwrap().who.clone();
		whitelist!(leaving);
	}: _(RawOrigin::Signed(leaving.clone()))
	verify {
		assert_last_event::<T>(Event::CandidateRemoved(leaving).into());
	}

	// worse case is paying a non-existing candidate account.
	note_author {
		T::Currency::make_free_balance_be(
			&<ParachainStaking<T>>::account_id(),
			T::Currency::minimum_balance() * 2u32.into(),
		);
		let author = account("author", 0, SEED);
		assert!(T::Currency::free_balance(&author) == 0u32.into());
	}: {
		<ParachainStaking<T> as EventHandler<_, _>>::note_author(author.clone())
	} verify {
		assert!(T::Currency::free_balance(&author) > 0u32.into());
	}
}

impl_benchmark_test_suite!(ParachainStaking, crate::mock::new_test_ext(), crate::mock::Test,);
