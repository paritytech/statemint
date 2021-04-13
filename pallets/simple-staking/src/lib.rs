#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, inherent::Vec, traits::{Currency, ReservableCurrency, EnsureOrigin}};
	use frame_system::{pallet_prelude::*, ensure_root};
	use frame_system::Config as SystemConfig;
	use frame_support::sp_runtime::{RuntimeDebug};
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		type UpdateOrigin: EnsureOrigin<Self::Origin>;
	}
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;


	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
	pub struct AuthorInfo<AccountId, Balance, BlockNumber> {
		pub who: AccountId,
		pub deposit: Balance,
		pub last_block: Option<BlockNumber>
	}
	pub type AuthorInfoOf<T> = AuthorInfo<<T as SystemConfig>::AccountId, <T as Currency<<T as SystemConfig>::AccountId>>::Balance, <T as frame_system::Config>::BlockNumber>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage

	#[pallet::storage]
	#[pallet::getter(fn invulnerables)]
	pub type Invulnerables<T: Config>= StorageValue<_, Vec<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn authors)]
	pub type Authors<T: Config> = StorageValue<_, Vec<AuthorInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_authors)]
	pub type MaxAuthors<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn author_bond)]
	pub type AuthorBond<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn authority_bond)]
	pub type AuthorityBond<T: Config>= StorageValue<_, BalanceOf<T>>;


	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewInvulnerables(Vec<T::AccountId>),
		NewMaxAuthorCount(u32),
		NewAuthorBond(BalanceOf<T>),
		AuthorAdded(T::AccountId, BalanceOf<T>),
		AuthorRemoved(T::AccountId)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MaxAuthors,
		Unknown,
		Permission,
		AlreadyAuthor,
		NotAuthor
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		//TODO on init (or finalize) add to aura set at next era
		//TODO split half the pot to the author per block
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(10_000)]
		pub fn set_invulnerables(origin: OriginFor<T>, new: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			<Invulnerables<T>>::put(&new);
			Self::deposit_event(Event::NewInvulnerables(new));
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn set_max_author_count(origin: OriginFor<T>, max_authors: u32) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			<MaxAuthors<T>>::put(&max_authors);
			Self::deposit_event(Event::NewMaxAuthorCount(max_authors));
			Ok(().into())

		}


		#[pallet::weight(10_000)]
		pub fn set_author_bond(origin: OriginFor<T>, author_bond: BalanceOf<T>) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			<AuthorBond<T>>::put(&author_bond);
			Self::deposit_event(Event::NewAuthorBond(author_bond));
			Ok(().into())

		}

		#[pallet::weight(10_000)]
		pub fn register_as_author(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// lock deposit to start or require min?
			let who = ensure_signed(origin)?;
			let deposit = <AuthorBond<T>>::get();
			let length = <Authors<T>>::decode_len().unwrap_or_default();
			ensure!((length as u32) < MaxAuthors::<T>::get(), Error::<T>::MaxAuthors);
			let new_author = AuthorInfo {
				who: who.clone(),
				deposit,
				last_block: None
			};
			<Authors<T>>::try_mutate(|authors| -> DispatchResult {
				if authors.into_iter().any(|author| author.who == who) {
					Err(Error::<T>::AlreadyAuthor)?
				} else {
					T::Currency::reserve(&who, deposit)?;
					authors.push(new_author);
					Self::deposit_event(Event::AuthorAdded(who, deposit));
					Ok(())
				}
			})?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn leave_intent(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			<Authors<T>>::try_mutate(|authors| -> DispatchResult {
				let index = authors.iter().position(|author| author.who == who).ok_or(Error::<T>::NotAuthor)?;
				T::Currency::unreserve(&who, authors[index].deposit);
				authors.remove(index);
				Ok(())
			})?;
			Self::deposit_event(Event::AuthorRemoved(who));
			Ok(().into())

		}
	}
}