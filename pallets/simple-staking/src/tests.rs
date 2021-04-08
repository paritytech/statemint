use super::*;
use crate::{mock::*, Error, AuthorInfo};
use frame_support::{assert_noop, assert_ok};
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
		assert_ok!(SimpleStaking::set_invulnerables(Origin::root(), new_handlers.clone()));
		// assert_eq!(
		// 	last_event(),
		// 	mock::Event::simple_staking(crate::Event::NewInvulnerables(new_handlers.clone())),
		// );
		assert_noop!(SimpleStaking::set_invulnerables(Origin::signed(1), new_handlers.clone()), BadOrigin);
		assert_eq!(SimpleStaking::invulnerables(), Some(new_handlers));
	});
}

#[test]
fn set_max_author_count() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleStaking::set_max_author_count(Origin::root(), 7));
		assert_noop!(SimpleStaking::set_max_author_count(Origin::signed(1), 8), BadOrigin);
		assert_eq!(SimpleStaking::max_authors(), 7);
	});
}

#[test]
fn author_intent() {
	new_test_ext().execute_with(|| {
		assert_noop!(SimpleStaking::author_intent(Origin::signed(1), 10), Error::<Test>::MaxAuthors);
		
		assert_ok!(SimpleStaking::set_max_author_count(Origin::root(), 1));
		assert_ok!(SimpleStaking::author_intent(Origin::signed(1), 10));

		let addition = AuthorInfo {
			who: 1, 
			deposit: 10, 
			last_block: 0
		};
		assert_eq!(Balances::free_balance(1), 90);
		assert_noop!(SimpleStaking::author_intent(Origin::signed(1), 10), Error::<Test>::MaxAuthors);
		
		assert_ok!(SimpleStaking::set_max_author_count(Origin::root(), 5));		
		assert_noop!(SimpleStaking::author_intent(Origin::signed(1), 10), Error::<Test>::AlreadyAuthor);
		assert_noop!(SimpleStaking::author_intent(Origin::signed(3), 10), BalancesError::<Test>::InsufficientBalance);
		assert_eq!(SimpleStaking::authors(), vec![addition]);
	});
}

#[test]
fn leave_intent() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleStaking::set_max_author_count(Origin::root(), 1));
		assert_ok!(SimpleStaking::author_intent(Origin::signed(1), 10));
		assert_noop!(SimpleStaking::leave_intent(Origin::signed(3)), Error::<Test>::NotAuthor);
		assert_ok!(SimpleStaking::leave_intent(Origin::signed(1)));
		assert_eq!(SimpleStaking::authors(), vec![]);
		assert_eq!(Balances::free_balance(1), 100);
	});
}