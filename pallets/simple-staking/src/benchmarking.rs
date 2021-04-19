//! Benchmarking setup for pallet-simple-staking

use super::*;

#[allow(unused)]
use crate::Pallet as SimpleStaking;
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

fn register_authors<T: Config>(count: u32) {
	let authors = (0..count).map(|c| account("author", c, SEED)).collect::<Vec<_>>();
	assert!(<AuthorBond<T>>::get() > 0u32.into(), "Bond cannot be zero!");
	for who in authors {
		T::Currency::make_free_balance_be(&who, <AuthorBond<T>>::get() * 2u32.into());
		<SimpleStaking<T>>::register_as_author(RawOrigin::Signed(who).into()).unwrap();
	}
}

benchmarks! {
	where_clause { where T: pallet_authorship::Config }

	set_invulnerables {
		let b in 1 .. T::MaxInvulnerables::get();
		let new_invulnerables = (0..b).map(|c| account("author", c, SEED)).collect::<Vec<_>>();
		let origin = T::UpdateOrigin::successful_origin();
		// whitelist!(RootAccount); // TODO
	}: {
		assert_ok!(
			<SimpleStaking<T>>::set_invulnerables(origin, new_invulnerables.clone())
		);
	}
	verify {
		assert_last_event::<T>(Event::NewInvulnerables(new_invulnerables).into());
	}

	set_allowed_author_count {
		let max: u32 = 999;
		let origin = T::UpdateOrigin::successful_origin();
		// whitelist!(caller);
	}: {
		assert_ok!(
			<SimpleStaking<T>>::set_allowed_author_count(origin, max.clone())
		);
	}
	verify {
		assert_last_event::<T>(Event::NewAllowedAuthorCount(max).into());
	}

	set_author_bond {
		let bond: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let origin = T::UpdateOrigin::successful_origin();
		// whitelist!(caller);
	}: {
		assert_ok!(
			<SimpleStaking<T>>::set_author_bond(origin, bond.clone())
		);
	}
	verify {
		assert_last_event::<T>(Event::NewAuthorBond(bond).into());
	}

	// worse case is when we have all the max-author slots filled except one, and we fill that one.
	register_as_author {
		let c in 1 .. T::MaxAuthors::get();

		<AuthorBond<T>>::put(T::Currency::minimum_balance());
		<AllowedAuthors<T>>::put(c + 1);
		register_authors::<T>(c);

		let caller: T::AccountId = whitelisted_caller();
		let bond: BalanceOf<T> = T::Currency::minimum_balance() * 2u32.into();
		T::Currency::make_free_balance_be(&caller, bond.clone());

	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::AuthorAdded(caller, bond / 2u32.into()).into());
	}

	// worse case is the last author leaving.
	leave_intent {
		let c in 1 .. T::MaxAuthors::get();
		<AuthorBond<T>>::put(T::Currency::minimum_balance());
		<AllowedAuthors<T>>::put(c);
		register_authors::<T>(c);

		let leaving = <Authors<T>>::get().last().unwrap().who.clone();
		whitelist!(leaving);
	}: _(RawOrigin::Signed(leaving.clone()))
	verify {
		assert_last_event::<T>(Event::AuthorRemoved(leaving).into());
	}

	// worse case is paying a non-existing author account.
	note_author {
		T::Currency::make_free_balance_be(
			&<SimpleStaking<T>>::account_id(),
			T::Currency::minimum_balance() * 2u32.into(),
		);
		let author = account("author", 0, SEED);
		assert!(T::Currency::free_balance(&author) == 0u32.into());
	}: {
		<SimpleStaking<T> as EventHandler<_, _>>::note_author(author.clone())
	} verify {
		assert!(T::Currency::free_balance(&author) > 0u32.into());
	}
}

impl_benchmark_test_suite!(SimpleStaking, crate::mock::new_test_ext(), crate::mock::Test,);
