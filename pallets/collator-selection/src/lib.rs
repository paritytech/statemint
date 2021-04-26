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

//! Collator Selection pallet.
//!
//! A pallet to manage collators in a parachain.
//!
//! ## Overview
//!
//! The Collator Selection pallet manages the collators of a parachain. **Collation is _not_ a
//! secure activity** and this pallet does not implement any game-theoretic mechanisms to meet BFT
//! safety assumptions of the chosen set.
//!
//! ## Terminology
//!
//! - Collator: A parachain block producer.
//! - Bond: An amount of `Balance` _reserved_ for candidate registration.
//! - Invulnerable: An account guaranteed to be in the collator set.
//!
//! ## Implementation
//!
//! The final [`Collators`] are aggregated from two individual lists:
//!
//! 1. [`Invulnerables`]: a set of collators appointed by governance. These accounts will always be
//!    collators.
//! 2. [`Candidates`]: these are *candidates to the collation task* and may or may not be elected as
//!    a final collator.
//!
//! The current implementation resolves congestion of [`Candidates`] in a first-come-first-serve
//! manner.
//!
//! ### Rewards
//!
//! The Collator Selection pallet maintains an on-chain account (the "Pot"). In each block, the
//! collator who authored it receives:
//!
//! - Half the value of the Pot.
//! - Half the value of the transaction fees within the block. The other half of the transaction
//!   fees are deposited into the Pot.
//!
//! Note: Eventually the Pot distribution may be modified as discussed in
//! [this issue](https://github.com/paritytech/statemint/issues/21#issuecomment-810481073).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		inherent::Vec,
		traits::{
			Currency, ReservableCurrency, EnsureOrigin, ExistenceRequirement::KeepAlive,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use frame_system::Config as SystemConfig;
	use frame_support::{
		sp_runtime::{
			RuntimeDebug,
			traits::AccountIdConversion,
		},
		weights::DispatchClass,
	};
	use core::ops::Div;
	use pallet_session::SessionManager;
	use sp_staking::SessionIndex;
	pub use crate::weights::WeightInfo;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	/// A convertor from collators id. Since this pallet does not have stash/controller, this is
	/// just identity.
	pub struct IdentityCollator;
	impl<T> sp_runtime::traits::Convert<T, Option<T>> for IdentityCollator {
		fn convert(t: T) -> Option<T> {
			Some(t)
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Origin that can dictate updating parameters of this pallet.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Account Identifier from which the internal Pot is generated.
		type PotId: Get<PalletId>;

		/// Maximum number of candidates that we should have. This is used for benchmarking and is not
		/// enforced.
		///
		/// This does not take into account the invulnerables.
		type MaxCandidates: Get<u32>;

		/// Maximum number of invulnerables.
		///
		/// Used only for benchmarking.
		type MaxInvulnerables: Get<u32>;

		/// The weight information of this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Basic information about a collation candidate.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
	pub struct CandidateInfo<AccountId, Balance, BlockNumber> {
		/// Account identifier.
		pub who: AccountId,
		/// Reserved deposit.
		pub deposit: Balance,
		/// Last block at which they authored a block.
		pub last_block: BlockNumber,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The invulnerable, fixed collators.
	#[pallet::storage]
	#[pallet::getter(fn invulnerables)]
	pub type Invulnerables<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// The (community, limited) collation candidates.
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> = StorageValue<
		_,
		Vec<CandidateInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>,
		ValueQuery,
	>;

	/// Desired number of candidates.
	///
	/// This should ideally always be less than [`Config::MaxCandidates`] for weights to be correct.
	#[pallet::storage]
	#[pallet::getter(fn desired_candidates)]
	pub type DesiredCandidates<T> = StorageValue<_, u32, ValueQuery>;

	/// Fixed deposit bond for each candidate.
	#[pallet::storage]
	#[pallet::getter(fn candidacy_bond)]
	pub type CandidacyBond<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;


	/// Desired blocks until check if bootable.
	#[pallet::storage]
	#[pallet::getter(fn boot_block)]
	pub type BootBlock<T> = StorageValue<_, <T as frame_system::Config>::BlockNumber, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub invulnerables: Vec<T::AccountId>,
		pub candidacy_bond: BalanceOf<T>,
		pub desired_candidates: u32,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				invulnerables: Default::default(),
				candidacy_bond: Default::default(),
				desired_candidates: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {

			let duplicate_invulnerables = self.invulnerables.iter().collect::<std::collections::BTreeSet<_>>();
			assert!(duplicate_invulnerables.len()  == self.invulnerables.len(), "duplicate invulnerables in genesis.");

			assert!(
				T::MaxInvulnerables::get() >= (self.invulnerables.len() as u32),
				"genesis invulnerables are more than T::MaxInvulnerables",
			);
			assert!(
				T::MaxCandidates::get() >= self.desired_candidates,
				"genesis desired_candidates are more than T::MaxCandidates",
			);

			<DesiredCandidates<T>>::put(&self.desired_candidates);
			<CandidacyBond<T>>::put(&self.candidacy_bond);
			<Invulnerables<T>>::put(&self.invulnerables);
		}
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewInvulnerables(Vec<T::AccountId>),
		NewDesiredCandidates(u32),
		NewCandidacyBond(BalanceOf<T>),
		CandidateAdded(T::AccountId, BalanceOf<T>),
		CandidateRemoved(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		TooManyCandidates,
		Unknown,
		Permission,
		AlreadyCandidate,
		NotCandidate,
		AlreadyInvulnerable,
		InvalidProof,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::set_invulnerables(new.len() as u32))]
		pub fn set_invulnerables(
			origin: OriginFor<T>,
			new: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			// we trust origin calls, this is just a for more accurate benchmarking
			if (new.len() as u32) > T::MaxInvulnerables::get() {
				log::warn!(
					"invulnerables > T::MaxInvulnerables; you might need to run benchmarks again"
				);
			}
			<Invulnerables<T>>::put(&new);
			Self::deposit_event(Event::NewInvulnerables(new));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::set_desired_candidates())]
		pub fn set_desired_candidates(origin: OriginFor<T>, max: u32) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			// we trust origin calls, this is just a for more accurate benchmarking
			if max > T::MaxCandidates::get() {
				log::warn!(
					"max > T::MaxCandidates; you might need to run benchmarks again"
				);
			}
			<DesiredCandidates<T>>::put(&max);
			Self::deposit_event(Event::NewDesiredCandidates(max));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::set_candidacy_bond())]
		pub fn set_candidacy_bond(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			<CandidacyBond<T>>::put(&bond);
			Self::deposit_event(Event::NewCandidacyBond(bond));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::register_as_candidate(T::MaxCandidates::get()))]
		pub fn register_as_candidate(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// ensure we are below limit.
			let length = <Candidates<T>>::decode_len().unwrap_or_default();
			ensure!((length as u32) < Self::desired_candidates(), Error::<T>::TooManyCandidates);
			ensure!(!Self::invulnerables().contains(&who), Error::<T>::AlreadyInvulnerable);

			let deposit = Self::candidacy_bond();
			let incoming = CandidateInfo { who: who.clone(), deposit, last_block: frame_system::Pallet::<T>::block_number() };

			let current_count =
				<Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
					if candidates.into_iter().any(|candidate| candidate.who == who) {
						Err(Error::<T>::AlreadyCandidate)?
					} else {
						T::Currency::reserve(&who, deposit)?;
						candidates.push(incoming);
						Ok(candidates.len())
					}
				})?;

			Self::deposit_event(Event::CandidateAdded(who, deposit));
			Ok(Some(T::WeightInfo::register_as_candidate(current_count as u32)).into())
		}

		#[pallet::weight(T::WeightInfo::leave_intent(T::MaxCandidates::get()))]
		pub fn leave_intent(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let current_count = Self::try_remove_candidate(&who)?;

			Ok(Some(T::WeightInfo::leave_intent(current_count as u32)).into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get a unique, inaccessible account id from the `PotId`.
		pub fn account_id() -> T::AccountId {
			T::PotId::get().into_account()
		}

		pub fn try_remove_candidate(who: &T::AccountId) -> Result<usize, DispatchError> {
			let current_count = <Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
				let index = candidates.iter().position(|candidate| candidate.who == *who).ok_or(Error::<T>::NotCandidate)?;
				T::Currency::unreserve(&who, candidates[index].deposit);
				candidates.remove(index);
				Ok(candidates.len())
			});
			Self::deposit_event(Event::CandidateRemoved(who.clone()));
			current_count
		}

		/// Assemble the current set of candidates and invulnerables into the next collator set.
		///
		/// This is done on the fly, as frequent as we are told to do so, as the session manager.
		pub fn assemble_collators() -> Vec<T::AccountId> {
			let mut collators = Self::invulnerables();
			let boot_block = Self::boot_block();
			collators.extend(
				Self::candidates().into_iter().filter_map(|c| {
					if c.last_block > boot_block {
						Some(c.who)
					} else {
						let _ = Self::try_remove_candidate(&c.who);
						None
					}

				}).collect::<Vec<_>>(),
			);
			<BootBlock<T>>::put(frame_system::Pallet::<T>::block_number());
			collators
		}
	}

	/// Keep track of number of authored blocks per authority, uncles are counted as well since
	/// they're a valid proof of being online.
	impl<T: Config + pallet_authorship::Config>
		pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
	{
		fn note_author(author: T::AccountId) {
			let treasury = Self::account_id();
			let reward = T::Currency::free_balance(&treasury).div(2u32.into());

			// `reward` is half of treasury account, this should never fail.
			let _success = T::Currency::transfer(&treasury, &author, reward, KeepAlive);
			debug_assert!(_success.is_ok());
			<Candidates<T>>::mutate(|candidates| {
				if candidates.len() > 0 {
				let index: usize = candidates.iter().position(|candidate| candidate.who == author).unwrap_or_default();
					candidates[index].last_block = frame_system::Pallet::<T>::block_number();
				}
			});
			let candidates_len = Self::candidates().len();
			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::WeightInfo::note_author(candidates_len as u32),
				DispatchClass::Mandatory,
			);
		}

		fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {
			//TODO can we ignore this?
		}
	}

	/// Play the role of the session manager.
	impl<T: Config> SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log::info!(
				"assembling new collators for new session {} at #{:?}",
				index,
				<frame_system::Pallet<T>>::block_number(),
			);
			let result = Some(Self::assemble_collators());
			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::WeightInfo::new_session(candidates_len as u32),
				DispatchClass::Mandatory,
			);
			result
		}
		fn start_session(_: SessionIndex) {
			// we don't care.
		}
		fn end_session(_: SessionIndex) {
			// we don't care.
		}
	}
}
