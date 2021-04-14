use super::*;
use crate::{mock::*, Error, AuthorInfo};
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};
use super::Event as StakingEvent;
use sp_runtime::traits::BadOrigin;
use frame_system::EventRecord;
use pallet_balances::Error as BalancesError;


fn last_event() -> mock::Event {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}

#[test]
fn it_should_set_invulnerables() {
	new_test_ext().execute_with(|| {
		let new_handlers = vec![1, 2, 3, 4];
		assert_ok!(SimpleStaking::set_invulnerables(Origin::signed(15276289921735352792), new_handlers.clone()));
		// assert_eq!(
		// 	last_event(),
		// 	mock::Event::simple_staking(crate::Event::NewInvulnerables(new_handlers.clone())),
		// );
		assert_noop!(SimpleStaking::set_invulnerables(Origin::signed(1), new_handlers.clone()), BadOrigin);
		assert_eq!(SimpleStaking::invulnerables(), new_handlers);
	});
}

#[test]
fn set_max_author_count() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleStaking::set_max_author_count(Origin::signed(15276289921735352792), 7));
		assert_noop!(SimpleStaking::set_max_author_count(Origin::signed(1), 8), BadOrigin);
		assert_eq!(SimpleStaking::max_authors(), 7);
	});
}

#[test]
fn set_author_bond() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleStaking::set_author_bond(Origin::signed(15276289921735352792), 7));
		assert_noop!(SimpleStaking::set_author_bond(Origin::signed(1), 8), BadOrigin);
		assert_eq!(SimpleStaking::author_bond(), 7);
	});
}

#[test]
fn register_as_author() {
	new_test_ext().execute_with(|| {
		assert_noop!(SimpleStaking::register_as_author(Origin::signed(1)), Error::<Test>::MaxAuthors);

		assert_ok!(SimpleStaking::set_author_bond(Origin::signed(15276289921735352792), 10));
		assert_ok!(SimpleStaking::set_max_author_count(Origin::signed(15276289921735352792), 1));
		assert_ok!(SimpleStaking::register_as_author(Origin::signed(1)));

		let addition = AuthorInfo {
			who: 1,
			deposit: 10,
			last_block: None
		};
		assert_eq!(Balances::free_balance(1), 90);
		assert_noop!(SimpleStaking::register_as_author(Origin::signed(1)), Error::<Test>::MaxAuthors);

		assert_ok!(SimpleStaking::set_max_author_count(Origin::signed(15276289921735352792), 5));
		assert_noop!(SimpleStaking::register_as_author(Origin::signed(1)), Error::<Test>::AlreadyAuthor);
		assert_noop!(SimpleStaking::register_as_author(Origin::signed(15276289921735352792)), BalancesError::<Test>::InsufficientBalance);
		assert_eq!(SimpleStaking::authors(), vec![addition]);
	});
}

#[test]
fn leave_intent() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleStaking::set_max_author_count(Origin::signed(15276289921735352792), 1));
		assert_ok!(SimpleStaking::set_author_bond(Origin::signed(15276289921735352792), 10));
		assert_ok!(SimpleStaking::register_as_author(Origin::signed(1)));
		assert_noop!(SimpleStaking::leave_intent(Origin::signed(15276289921735352792)), Error::<Test>::NotAuthor);
		assert_ok!(SimpleStaking::leave_intent(Origin::signed(1)));
		assert_eq!(SimpleStaking::authors(), vec![]);
		assert_eq!(Balances::free_balance(1), 100);
	});
}

#[test]
fn on_init() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(4), 0);
		assert_eq!(Balances::free_balance(5), 100);
		Authorship::on_initialize(1);
		assert_eq!(Balances::free_balance(4), 50);
		assert_eq!(Balances::free_balance(5), 50);
	});
}

#[test]
fn on_genesis() {
	new_test_ext().execute_with(|| {
		assert_eq!(SimpleStaking::invulnerables(), vec![1,2,3]);
		assert_eq!(SimpleStaking::invulnerables().len(), 3);
	});
}