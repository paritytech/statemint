//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as SimpleStaking;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, Zero};
use frame_system::{RawOrigin, EventRecord};
use frame_support::{traits::Currency, dispatch::DispatchResult};

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn set_authors<T: Config>(who: T::AccountId, deposit: BalanceOf<T>) -> DispatchResult {
	let addition = AuthorInfo {
		who,
		deposit,
		last_block: None
	};
	Authors::<T>::try_mutate(|authors| -> DispatchResult {
		authors.push(addition);
		Ok(())
	})
}

benchmarks! {
	set_invulnerables {
		let b in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();
		let new_invulnerables = vec![caller.clone(); b as usize];

	}: _(RawOrigin::Signed(caller), new_invulnerables.clone())
	verify {
		assert_last_event::<T>(Event::NewInvulnerables(new_invulnerables).into());
	}

	set_max_author_count {
		let b in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();

	}: _(RawOrigin::Signed(caller), b.clone())
	verify {
		assert_last_event::<T>(Event::NewMaxAuthorCount(b).into());
	}

	set_author_bond {
		let b in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();
		let new: BalanceOf<T> = b.into();

	}: _(RawOrigin::Signed(caller), new.clone())
	verify {
		assert_last_event::<T>(Event::NewAuthorBond(new).into());
	}

	register_as_author {
		let caller: T::AccountId = whitelisted_caller();
		SimpleStaking::<T>::set_author_bond(RawOrigin::Signed(caller.clone()).into(), 10u32.into());
		SimpleStaking::<T>::set_max_author_count(RawOrigin::Signed(caller.clone()).into(), 1);
		T::Currency::make_free_balance_be(&caller, 100_000u32.into());

	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::AuthorAdded(caller, 10u32.into()).into());
	}

	leave_intent {
		let caller: T::AccountId = whitelisted_caller();
		set_authors::<T>(caller.clone(), 10u32.into())?;

	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::AuthorRemoved(caller).into());
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn set_invulnerables() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_invulnerables::<Test>());
		});
	}

	#[test]
	fn set_max_author_count() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_max_author_count::<Test>());
		});
	}

	#[test]
	fn set_author_bond() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_author_bond::<Test>());
		});
	}

	#[test]
	fn register_as_author() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_register_as_author::<Test>());
		});
	}
	#[test]
	fn leave_intent() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_leave_intent::<Test>());
		});
	}
}

impl_benchmark_test_suite!(SimpleStaking, crate::mock::new_test_ext(), crate::mock::Test,);
