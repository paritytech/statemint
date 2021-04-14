//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as SimpleStaking;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, Zero, account};
use frame_system::{RawOrigin, EventRecord};
use frame_support::{traits::Currency, dispatch::DispatchResult};

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn set_authors<T: Config>(authors: Vec<T::AccountId>, deposit: BalanceOf<T>) {
	for who in authors {
		let junk_addition = AuthorInfo {
			who,
			deposit,
			last_block: None
		};
		Authors::<T>::append(junk_addition);
	}
}

benchmarks! {
	set_invulnerables {
		let b in 1 .. 1000; // T::MaxInvulenrables::get()
		let caller: T::AccountId = whitelisted_caller();
		let new_invulnerables = vec![caller.clone(); b as usize];

	}: _(RawOrigin::Signed(caller), new_invulnerables.clone())
	verify {
		assert_last_event::<T>(Event::NewInvulnerables(new_invulnerables).into());
	}

	set_max_author_count {
		let max: u32 = 999;
		let caller: T::AccountId = whitelisted_caller();

	}: _(RawOrigin::Signed(caller), max.clone())
	verify {
		assert_last_event::<T>(Event::NewMaxAuthorCount(max).into());
	}

	set_author_bond {
		let bond: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let caller: T::AccountId = whitelisted_caller();

	}: _(RawOrigin::Signed(caller), bond.clone())
	verify {
		assert_last_event::<T>(Event::NewAuthorBond(bond).into());
	}

	register_as_author {
		let c in 0 .. T::MaxAuthors::get();
		
		let junk_account = account("junk", 0, SEED);
		let caller: T::AccountId = whitelisted_caller();
		let junk = vec![junk_account.clone(); c as usize];
		let bond: BalanceOf<T> = T::Currency::minimum_balance() * 2u32.into();
		set_authors::<T>(junk, bond);

		<AuthorBond<T>>::put(T::Currency::minimum_balance());
		<MaxAuthors<T>>::put(c + 1);

		T::Currency::make_free_balance_be(&caller, bond.clone());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::AuthorAdded(caller, bond).into());
	}

	leave_intent {
		let c in 0 .. 1000;
		let junk_account = account("junk", 0, SEED);

		let caller: T::AccountId = whitelisted_caller();
		let junk = vec![junk_account.clone(); c as usize];
		set_authors::<T>(caller.clone(), junk, 10u32.into());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::AuthorRemoved(caller).into());
	}
}

impl_benchmark_test_suite!(SimpleStaking, crate::mock::new_test_ext(), crate::mock::Test,);
