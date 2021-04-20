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

//! Parachain staking pallet.
//!
//! A pallet to manage collators in a parachain.
//!
//! The final [`Collators`] are aggregated from two individual lists. 1. [`Invulnerables`], a set of
//! governance controlled collators. These accounts will always be collators. 2. [`Candidates`]:
//! these are *candidates to the collation task* and may or may not be elected as a final collator.
//!
//! The current implementation resolves congestion of [`Candidates`] by a first-come-first-serve
//! manner.
//!
//! All active collators are rewarded equally for being the block author, by receiving half of the
//! block's transaction fees.

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
		traits::{Currency, ReservableCurrency, EnsureOrigin, ExistenceRequirement::KeepAlive},
		PalletId
	};
	use frame_system::pallet_prelude::*;
	use frame_system::Config as SystemConfig;
	use frame_support::{
		sp_runtime::{RuntimeDebug, traits::{AccountIdConversion, Zero, One}},
		weights::DispatchClass,
	};
	use core::ops::Div;
	pub use crate::weights::WeightInfo;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Origin that can dictate updating parameters of this pallet.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Account Identifier using which the internal treasury account is generated.
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

		/// Length of an epoch. Each epoch is merely a fixed period of time during which the
		/// [`Collators`] will not change. At each block where `block % epoch == 0`, based on the
		/// logic of the pallet, a new collator set is created from [`Candidates`] and
		/// [`Invulnerables`].
		type Epoch: Get<Self::BlockNumber>;

		/// The weight information of this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Basic information about a collation candidate.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
	pub struct CandidateInfo<AccountId, Balance, BlockNumber> {
		/// Account id
		pub who: AccountId,
		/// Reserved deposit.
		pub deposit: Balance,
		/// Last block at which they authored a block.
		pub last_block: Option<BlockNumber>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage

	/// The invulnerable, fixed collators.
	#[pallet::storage]
	#[pallet::getter(fn invulnerables)]
	pub type Invulnerables<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// The (community, limited) collation candidates.
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> = StorageValue<_, Vec<CandidateInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>, ValueQuery>;

	/// Desired number of candidates.
	///
	/// This should ideally always be less than [`Config::MaxCandidates`] for weights to be correct.
	#[pallet::storage]
	#[pallet::getter(fn desired_candidates)]
	pub type DesiredCandidate<T> = StorageValue<_, u32, ValueQuery>;

	/// Fixed deposit bond for each candidate.
	#[pallet::storage]
	#[pallet::getter(fn candidacy_bond)]
	pub type CandidacyBond<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Final collator set that this pallet computes. Change only every [`Self::Epoch`].
	// #[pallet::storage]
	// #[pallet::getter(fn collators)]
	// pub type Collators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;
	// TODO: @kianenigma

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub invulnerables: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				invulnerables: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				T::MaxInvulnerables::get() >= (self.invulnerables.len() as u32),
				"genesis invulnerables are more than T::MaxInvulnerables",
			);
			<Invulnerables<T>>::put(&self.invulnerables);
			// TODO: @kianenigma
			// <Collators<T>>::put(&self.invulnerables);
		}
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewInvulnerables(Vec<T::AccountId>),
		NewDesiredCandidate(u32),
		NewCandidacyBond(BalanceOf<T>),
		CandidateAdded(T::AccountId, BalanceOf<T>),
		CandidateRemoved(T::AccountId)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		TooManyCandidates,
		Unknown,
		Permission,
		AlreadyCandidate,
		NotCandidate,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// TODO: @kianenigma
		// fn on_initialize(n: T::BlockNumber) -> Weight {
		// 	if (n % T::Epoch::get().max(One::one())).is_zero() {
		// 		let mut collators = Self::invulnerables();
		// 		collators.extend(Self::candidates().into_iter().map(|c| c.who).collect::<Vec<_>>());
		// 		<Collators<T>>::put(collators);
		// 		T::DbWeight::get().reads_writes(2, 1)
		// 	} else {
		// 		Zero::zero()
		// 	}
		// }
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
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

		#[pallet::weight(T::WeightInfo::set_max_candidates())]
		pub fn set_max_candidates(origin: OriginFor<T>, max: u32) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			// we trust origin calls, this is just a for more accurate benchmarking
			if max > T::MaxCandidates::get() {
				log::warn!(
					"max > T::MaxCandidates; you might need to run benchmarks again"
				);
			}
			<DesiredCandidate<T>>::put(&max);
			Self::deposit_event(Event::NewDesiredCandidate(max));
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

			let deposit = Self::candidacy_bond();
			let incoming = CandidateInfo { who: who.clone(), deposit, last_block: None };

			let current_count = <Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
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

			let current_count = <Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
				let index = candidates.iter().position(|candidate| candidate.who == who).ok_or(Error::<T>::NotCandidate)?;
				T::Currency::unreserve(&who, candidates[index].deposit);
				candidates.remove(index);
				Ok(candidates.len())
			})?;

			Self::deposit_event(Event::CandidateRemoved(who));
			Ok(Some(T::WeightInfo::leave_intent(current_count as u32)).into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get a unique, inaccessible account id from the `PotId`.
		pub fn account_id() -> T::AccountId {
			T::PotId::get().into_account()
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

			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::WeightInfo::note_author(),
				DispatchClass::Mandatory,
			);
		}

		fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {
			//TODO can we ignore this?
		}
	}
}
