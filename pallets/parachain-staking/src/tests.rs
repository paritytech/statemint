// // Copyright (C) 2021 Parity Technologies (UK) Ltd.
// // SPDX-License-Identifier: Apache-2.0

// // Licensed under the Apache License, Version 2.0 (the "License");
// // you may not use this file except in compliance with the License.
// // You may obtain a copy of the License at
// //
// // 	http://www.apache.org/licenses/LICENSE-2.0
// //
// // Unless required by applicable law or agreed to in writing, software
// // distributed under the License is distributed on an "AS IS" BASIS,
// // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// // See the License for the specific language governing permissions and
// // limitations under the License.

// use crate::{mock::*, Error, CandidateInfo};
// use frame_support::{
// 	assert_noop, assert_ok,
// 	traits::{OnInitialize, Currency},
// };
// use sp_runtime::{traits::BadOrigin, testing::UintAuthorityId};
// use pallet_balances::Error as BalancesError;

// #[test]
// fn basic_setup_works() {
// 	new_test_ext().execute_with(|| {
// 		assert_eq!(ParachainStaking::desired_candidates(), 2);
// 		assert_eq!(ParachainStaking::candidacy_bond(), 10);

// 		assert_eq!(candidates(), vec![]);

// 		assert_eq!(ParachainStaking::invulnerables(), vec![1, 2]);
// 		assert_eq!(ParachainStaking::collators(), vec![1, 2]);
// 	});
// }

// #[test]
// fn it_should_set_invulnerables() {
// 	new_test_ext().execute_with(|| {
// 		let new_set = vec![1, 2, 3, 4];
// 		assert_ok!(ParachainStaking::set_invulnerables(
// 			Origin::signed(RootAccount::get()),
// 			new_set.clone()
// 		));
// 		assert_eq!(ParachainStaking::invulnerables(), new_set);

// 		// cannot set with non-root.
// 		assert_noop!(
// 			ParachainStaking::set_invulnerables(Origin::signed(1), new_set.clone()),
// 			BadOrigin
// 		);
// 	});
// }

// #[test]
// fn set_desired_candidates_works() {
// 	new_test_ext().execute_with(|| {
// 		// given
// 		assert_eq!(ParachainStaking::desired_candidates(), 2);

// 		// can set
// 		assert_ok!(ParachainStaking::set_desired_candidates(Origin::signed(RootAccount::get()), 7));
// 		assert_eq!(ParachainStaking::desired_candidates(), 7);

// 		// rejects bad origin
// 		assert_noop!(ParachainStaking::set_desired_candidates(Origin::signed(1), 8), BadOrigin);
// 	});
// }

// #[test]
// fn set_candidacy_bond() {
// 	new_test_ext().execute_with(|| {
// 		// given
// 		assert_eq!(ParachainStaking::candidacy_bond(), 10);

// 		// can set
// 		assert_ok!(ParachainStaking::set_candidacy_bond(Origin::signed(RootAccount::get()), 7));
// 		assert_eq!(ParachainStaking::candidacy_bond(), 7);

// 		// rejects bad origin.
// 		assert_noop!(ParachainStaking::set_candidacy_bond(Origin::signed(1), 8), BadOrigin);
// 	});
// }

// #[test]
// fn cannot_register_candidate_if_too_many() {
// 	new_test_ext().execute_with(|| {
// 		// reset desired candidates:
// 		<crate::DesiredCandidates<Test>>::put(0);

// 		// can't accept anyone anymore.
// 		assert_noop!(
// 			ParachainStaking::register_as_candidate(Origin::signed(3), UintAuthorityId(3).into(), vec![]),
// 			Error::<Test>::TooManyCandidates,
// 		);

// 		// reset desired candidates:
// 		<crate::DesiredCandidates<Test>>::put(1);
// 		assert_ok!(ParachainStaking::register_as_candidate(Origin::signed(4), UintAuthorityId(4).into(), vec![]));

// 		// but no more
// 		assert_noop!(
// 			ParachainStaking::register_as_candidate(Origin::signed(5), UintAuthorityId(5).into(), vec![]),
// 			Error::<Test>::TooManyCandidates,
// 		);
// 	})
// }

// #[test]
// fn cannot_register_as_candidate_if_invulnerable() {
// 	new_test_ext().execute_with(|| {
// 		assert_eq!(ParachainStaking::invulnerables(), vec![1, 2]);

// 		// can't 1 because it is invulnerable.
// 		assert_noop!(
// 			ParachainStaking::register_as_candidate(Origin::signed(1), UintAuthorityId(1).into(), vec![]),
// 			Error::<Test>::AlreadyInvulnerable,
// 		);
// 	})
// }

// #[test]
// fn cannot_register_dupe_candidate() {
// 	new_test_ext().execute_with(|| {
// 		// can add 3 as candidate
// 		assert_ok!(ParachainStaking::register_as_candidate(
// 			Origin::signed(3),
// 			UintAuthorityId(3).into(),
// 			vec![]
// 		));
// 		let addition = CandidateInfo { who: 3, deposit: 10, last_block: None };
// 		assert_eq!(ParachainStaking::candidates(), vec![addition]);
// 		assert_eq!(Balances::free_balance(3), 90);

// 		// but no more
// 		assert_noop!(
// 			ParachainStaking::register_as_candidate(
// 				Origin::signed(3),
// 				UintAuthorityId(3).into(),
// 				vec![]
// 			),
// 			Error::<Test>::AlreadyCandidate,
// 		);
// 	})
// }

// #[test]
// fn cannot_register_as_candidate_if_poor() {
// 	new_test_ext().execute_with(|| {
// 		assert_eq!(Balances::free_balance(&3), 100);
// 		assert_eq!(Balances::free_balance(&33), 0);

// 		// works
// 		assert_ok!(ParachainStaking::register_as_candidate(Origin::signed(3), UintAuthorityId(3).into(), vec![]));

// 		// poor
// 		assert_noop!(
// 			ParachainStaking::register_as_candidate(Origin::signed(33), UintAuthorityId(33).into(), vec![]),
// 			BalancesError::<Test>::InsufficientBalance,
// 		);
// 	});
// }

// #[test]
// fn register_as_candidate_works() {
// 	new_test_ext().execute_with(|| {
// 		// given
// 		assert_eq!(ParachainStaking::desired_candidates(), 2);
// 		assert_eq!(ParachainStaking::candidacy_bond(), 10);
// 		assert_eq!(ParachainStaking::candidates(), vec![]);
// 		assert_eq!(ParachainStaking::invulnerables(), vec![1, 2]);

// 		// take two endowed, non-invulnerables accounts.
// 		assert_eq!(Balances::free_balance(&3), 100);
// 		assert_eq!(Balances::free_balance(&4), 100);

// 		assert_ok!(ParachainStaking::register_as_candidate(Origin::signed(3), UintAuthorityId(3).into(), vec![]));
// 		assert_ok!(ParachainStaking::register_as_candidate(Origin::signed(4), UintAuthorityId(4).into(), vec![]));

// 		assert_eq!(Balances::free_balance(&3), 90);
// 		assert_eq!(Balances::free_balance(&4), 90);

// 		assert_eq!(ParachainStaking::candidates().len(), 2);
// 	});
// }

// #[test]
// fn leave_intent() {
// 	new_test_ext().execute_with(|| {
// 		// register a candidate.
// 		assert_ok!(ParachainStaking::register_as_candidate(
// 			Origin::signed(3),
// 			UintAuthorityId(3).into(),
// 			vec![]
// 		));
// 		assert_eq!(Balances::free_balance(3), 90);

// 		// cannot leave if not candidate.
// 		assert_noop!(
// 			ParachainStaking::leave_intent(Origin::signed(4)),
// 			Error::<Test>::NotCandidate
// 		);

// 		// bond is returned
// 		assert_ok!(ParachainStaking::leave_intent(Origin::signed(3)));
// 		assert_eq!(Balances::free_balance(3), 100);
// 	});
// }

// #[test]
// fn authorship_event_handler() {
// 	new_test_ext().execute_with(|| {
// 		// put some money into the pot
// 		Balances::make_free_balance_be(&ParachainStaking::account_id(), 100);

// 		// 4 is the default author.
// 		assert_eq!(Balances::free_balance(4), 100);

// 		// triggers `note_author`
// 		Authorship::on_initialize(1);

// 		// half of the pot goes to the collator who's the author (4 in tests).
// 		assert_eq!(Balances::free_balance(4), 150);
// 		// half stays.
// 		assert_eq!(Balances::free_balance(ParachainStaking::account_id()), 50);
// 	});
// }

// #[test]
// fn epoch_change_works() {
// 	new_test_ext().execute_with(|| {
// 		// initial collators from invulnerables.
// 		assert_eq!(ParachainStaking::collators(), vec![(1, UintAuthorityId(1)), (2, UintAuthorityId(2))]);

// 		// add two more
// 		assert_ok!(ParachainStaking::register_as_candidate(Origin::signed(3), UintAuthorityId(3).into(), vec![]));
// 		assert_ok!(ParachainStaking::register_as_candidate(Origin::signed(4), UintAuthorityId(4).into(), vec![]));

// 		// same collators.
// 		ParachainStaking::on_initialize(1);
// 		assert_eq!(ParachainStaking::collators(), vec![1, 2]);

// 		for i in 2..=10 { ParachainStaking::on_initialize(i); }

// 		// new epoch enacted.
// 		assert_eq!(ParachainStaking::collators(), vec![(1, UintAuthorityId(1)), (2, UintAuthorityId(2)), (3, UintAuthorityId(3)), (4, UintAuthorityId(4))]);
// 	})
// }
