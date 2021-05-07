// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! # Transaction Payment Module
//!
//! This module provides the basic logic needed to pay the absolute minimum amount needed for a
//! transaction to be included. This includes:
//!   - _base fee_: This is the minimum amount a user pays for a transaction. It is declared
//! 	as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
//!   - _weight fee_: A fee proportional to amount of weight a transaction consumes.
//!   - _length fee_: A fee proportional to the encoded length of the transaction.
//!   - _tip_: An optional tip. Tip increases the priority of the transaction, giving it a higher
//!     chance to be included by the transaction queue.
//!
//! The base fee and adjusted weight and length fees constitute the _inclusion fee_, which is
//! the minimum fee for a transaction to be included in a block.
//!
//! The formula of final fee:
//!   ```ignore
//!   inclusion_fee = base_fee + length_fee + [targeted_fee_adjustment * weight_fee];
//!   final_fee = inclusion_fee + tip;
//!   ```
//!
//!   - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on
//! 	the congestion of the network.
//!
//! Additionally, this module allows one to configure:
//!   - The mapping between one unit of weight to one unit of fee via [`Config::WeightToFee`].
//!   - A means of updating the fee for the next block, via defining a multiplier, based on the
//!     final state of the chain at the end of the previous block. This can be configured via
//!     [`Config::FeeMultiplierUpdate`]
//!   - How the fees are paid via [`Config::OnChargeTransaction`].

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode, EncodeLike};
use frame_support::{
	decl_storage, decl_module,
	DefaultNoBound,
	traits::Get,
	weights::{
		Weight, DispatchInfo, PostDispatchInfo, GetDispatchInfo, Pays, WeightToFeePolynomial,
		WeightToFeeCoefficient, DispatchClass,
	},
	dispatch::DispatchResult,
};
use sp_runtime::{
	FixedU128, FixedPointNumber, FixedPointOperand, Perquintill, RuntimeDebug,
	transaction_validity::{
		InvalidTransaction, TransactionPriority, ValidTransaction, TransactionValidityError, TransactionValidity,
	},
	traits::{
		Saturating, SignedExtension, SaturatedConversion, Convert, Dispatchable,
		DispatchInfoOf, PostDispatchInfoOf, Zero, One,
	},
};
use pallet_balances::NegativeImbalance;
use pallet_transaction_payment::OnChargeTransaction;
use frame_support::traits::tokens::{fungibles::{Balanced, Inspect, CreditOf}, WithdrawConsequence};

// mod payment;
// mod types;

// pub use payment::*;
// pub use types::{InclusionFee, FeeDetails, RuntimeDispatchInfo};

type BalanceOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;
type AssetBalanceOf<T> = <<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> = <<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type LiquidityInfoOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo;

#[derive(Encode, Decode, DefaultNoBound)]
pub enum InitialPayment<T: Config> {
	Nothing,
	Native(LiquidityInfoOf<T>),
	Asset(CreditOf<T::AccountId, T::Fungibles>),
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

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

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config + pallet_balances::Config + pallet_authorship::Config + pallet_assets::Config {
		type Fungibles: Balanced<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T>
{
	fn to_asset_balance(balance: BalanceOf<T>, asset_id: AssetIdOf<T>) -> Result<AssetBalanceOf<T>, ()> {
		// TODO: needs to be implemented in the assets pallet
		// let ed = <T as pallet_balances::Config>::ExistentialDeposit::get().max(One::one());
		// let asset = pallet_assets::Asset::<T>::get(asset_id).ok_or(|| ())?;
		// if asset.is_sufficient {
		// 	Ok(FixedU128::saturating_from_rational(asset.min_balance, ed).saturating_mul_int(balance).into())
		// } else {
		// 	Err(())
		// }
		Ok(One::one())
	}
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue. Allows paying via both `pallet_balances` as well as `pallet_assets`.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ChargeAssetTxPayment<T: Config>(#[codec(compact)] BalanceOf<T>, Option<AssetIdOf<T>>);

impl<T: Config> ChargeAssetTxPayment<T> where
	T::Call: Dispatchable<Info=DispatchInfo, PostInfo=PostDispatchInfo>,
	BalanceOf<T>: Send + Sync + FixedPointOperand,
	AssetIdOf<T>: Send + Sync,
	AssetBalanceOf<T>: Send + Sync,
{
	/// utility constructor. Used only in client/factory code.
	pub fn from(fee: BalanceOf<T>, asset_id: Option<AssetIdOf<T>>) -> Self {
		Self(fee, asset_id)
	}

	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &T::Call,
		info: &DispatchInfoOf<T::Call>,
		len: usize,
	) -> Result<
		(
			BalanceOf<T>,
			InitialPayment<T>,
		),
		TransactionValidityError,
	> {
		let tip = self.0;
		let fee = pallet_transaction_payment::Module::<T>::compute_fee(len as u32, info, tip);

		if fee.is_zero() {
			return Ok((fee, InitialPayment::Nothing));
		}

		let maybe_asset_id  = self.1;
		if let Some(asset_id) = maybe_asset_id {
			// TODO: implement fee payment via assets pallet
			let converted_fee = Pallet::<T>::to_asset_balance(fee, asset_id)
				.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?;
			let can_withdraw = <T::Fungibles as Inspect<T::AccountId>>::can_withdraw(asset_id, who, converted_fee);
			if !matches!(can_withdraw, WithdrawConsequence::Success) {
				return Err(InvalidTransaction::Payment.into());
			}
			<T::Fungibles as Balanced<T::AccountId>>::withdraw(asset_id, who, converted_fee)
				.map(|i| (fee, InitialPayment::Asset(i)))
				.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
		} else {
			<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::withdraw_fee(who, call, info, fee, tip)
				.map(|i| (fee, InitialPayment::Native(i)))
				.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
		}
	}

	/// Get an appropriate priority for a transaction with the given length and info.
	///
	/// This will try and optimise the `fee/weight` `fee/length`, whichever is consuming more of the
	/// maximum corresponding limit.
	///
	/// For example, if a transaction consumed 1/4th of the block length and half of the weight, its
	/// final priority is `fee * min(2, 4) = fee * 2`. If it consumed `1/4th` of the block length
	/// and the entire block weight `(1/1)`, its priority is `fee * min(1, 4) = fee * 1`. This means
	///  that the transaction which consumes more resources (either length or weight) with the same
	/// `fee` ends up having lower priority.
	fn get_priority(len: usize, info: &DispatchInfoOf<T::Call>, final_fee: BalanceOf<T>) -> TransactionPriority {
		let weight_saturation = T::BlockWeights::get().max_block / info.weight.max(1);
		let max_block_length = *T::BlockLength::get().max.get(DispatchClass::Normal);
		let len_saturation = max_block_length as u64 / (len as u64).max(1);
		let coefficient: BalanceOf<T> = weight_saturation.min(len_saturation).saturated_into::<BalanceOf<T>>();
		final_fee.saturating_mul(coefficient).saturated_into::<TransactionPriority>()
	}

	fn correct_and_deposit_fee(
		who: &T::AccountId,
		_dispatch_info: &DispatchInfoOf<T::Call>,
		_post_info: &PostDispatchInfoOf<T::Call>,
		corrected_fee: BalanceOf<T>,
		tip: BalanceOf<T>,
		paid: CreditOf<T::AccountId, T::Fungibles>,
	) -> Result<(), TransactionValidityError> {
		let converted_fee = Pallet::<T>::to_asset_balance(corrected_fee, paid.asset())
		.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?;
		// Calculate how much refund we should return
		let (refund, final_fee) = paid.split(converted_fee);
		// refund to the the account that paid the fees. If this fails, the
		// account might have dropped below the existential balance. In
		// that case we don't refund anything.
		// TODO: what to do in case this errors?
		let _res = <T::Fungibles as Balanced<T::AccountId>>::resolve(who, refund);
		
		let author = pallet_authorship::Module::<T>::author();
		// TODO: what to do in case paying the author fails (e.g. because `fee < min_balance`)
		<T::Fungibles as Balanced<T::AccountId>>::resolve(&author, final_fee)
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?;
		Ok(())
	}
}

impl<T: Config> sp_std::fmt::Debug for ChargeAssetTxPayment<T>
{
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "ChargeAssetTxPayment<{:?}, {:?}>", self.0, self.1.encode())
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config> SignedExtension for ChargeAssetTxPayment<T> where
	BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
	T::Call: Dispatchable<Info=DispatchInfo, PostInfo=PostDispatchInfo>,
	AssetIdOf<T>: Send + Sync,
	AssetBalanceOf<T>: Send + Sync,
{
	const IDENTIFIER: &'static str = "ChargeAssetTxPayment";
	type AccountId = T::AccountId;
	type Call = T::Call;
	type AdditionalSigned = ();
	type Pre = (
		// tip
		BalanceOf<T>,
		// who paid the fee
		Self::AccountId,
		// imbalance resulting from withdrawing the fee
		InitialPayment<T>,
	);
	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> { Ok(()) }

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		let (fee, _) = self.withdraw_fee(who, call, info, len)?;
		Ok(ValidTransaction {
			priority: Self::get_priority(len, info, fee),
			..Default::default()
		})
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize
	) -> Result<Self::Pre, TransactionValidityError> {
		let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;
		Ok((self.0, who.clone(), initial_payment))
	}

	fn post_dispatch(
		pre: Self::Pre,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		_result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		let (tip, who, initial_payment) = pre;
		let actual_fee = pallet_transaction_payment::Module::<T>::compute_actual_fee(
			len as u32,
			info,
			post_info,
			tip,
		);
		match initial_payment {
			InitialPayment::Native(imbalance) => {
				<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::correct_and_deposit_fee(&who, info, post_info, actual_fee, tip, imbalance)?;
			},
			// InitialPayment::Asset((asset_id, amount)) => {
			InitialPayment::Asset(credit) => {
				// let credit = <T::Fungibles as Balanced<T::AccountId>>::issue(asset_id, amount);
				Self::correct_and_deposit_fee(&who, info, post_info, actual_fee, tip, credit)?;
			},
			// TODO: just assert that actual_fee is also zero?
			InitialPayment::Nothing => {
				debug_assert!(actual_fee.is_zero(), "actual fee should be zero if initial fee was zero.");
			},
		}
		
		Ok(())
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use crate as pallet_transaction_payment;
// 	use frame_system as system;
// 	use codec::Encode;
// 	use frame_support::{
// 		assert_noop, assert_ok, parameter_types,
// 		weights::{
// 			DispatchClass, DispatchInfo, PostDispatchInfo, GetDispatchInfo, Weight,
// 			WeightToFeePolynomial, WeightToFeeCoefficients, WeightToFeeCoefficient,
// 		},
// 		traits::Currency,
// 	};
// 	use pallet_balances::Call as BalancesCall;
// 	use sp_core::H256;
// 	use sp_runtime::{
// 		testing::{Header, TestXt},
// 		traits::{BlakeTwo256, IdentityLookup, One},
// 		transaction_validity::InvalidTransaction,
// 		Perbill,
// 	};
// 	use std::cell::RefCell;
// 	use smallvec::smallvec;

// 	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
// 	type Block = frame_system::mocking::MockBlock<Runtime>;

// 	frame_support::construct_runtime!(
// 		pub enum Runtime where
// 			Block = Block,
// 			NodeBlock = Block,
// 			UncheckedExtrinsic = UncheckedExtrinsic,
// 		{
// 			System: system::{Pallet, Call, Config, Storage, Event<T>},
// 			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
// 			TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
// 		}
// 	);

// 	const CALL: &<Runtime as frame_system::Config>::Call =
// 		&Call::Balances(BalancesCall::transfer(2, 69));

// 	thread_local! {
// 		static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
// 	}

// 	pub struct BlockWeights;
// 	impl Get<frame_system::limits::BlockWeights> for BlockWeights {
// 		fn get() -> frame_system::limits::BlockWeights {
// 			frame_system::limits::BlockWeights::builder()
// 				.base_block(0)
// 				.for_class(DispatchClass::all(), |weights| {
// 					weights.base_extrinsic = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow()).into();
// 				})
// 				.for_class(DispatchClass::non_mandatory(), |weights| {
// 					weights.max_total = 1024.into();
// 				})
// 				.build_or_panic()
// 		}
// 	}

// 	parameter_types! {
// 		pub const BlockHashCount: u64 = 250;
// 		pub static TransactionByteFee: u64 = 1;
// 		pub static WeightToFee: u64 = 1;
// 	}

// 	impl frame_system::Config for Runtime {
// 		type BaseCallFilter = ();
// 		type BlockWeights = BlockWeights;
// 		type BlockLength = ();
// 		type DbWeight = ();
// 		type Origin = Origin;
// 		type Index = u64;
// 		type BlockNumber = u64;
// 		type Call = Call;
// 		type Hash = H256;
// 		type Hashing = BlakeTwo256;
// 		type AccountId = u64;
// 		type Lookup = IdentityLookup<Self::AccountId>;
// 		type Header = Header;
// 		type Event = Event;
// 		type BlockHashCount = BlockHashCount;
// 		type Version = ();
// 		type PalletInfo = PalletInfo;
// 		type AccountData = pallet_balances::AccountData<u64>;
// 		type OnNewAccount = ();
// 		type OnKilledAccount = ();
// 		type SystemWeightInfo = ();
// 		type SS58Prefix = ();
// 		type OnSetCode = ();
// 	}

// 	parameter_types! {
// 		pub const ExistentialDeposit: u64 = 1;
// 	}

// 	impl pallet_balances::Config for Runtime {
// 		type Balance = u64;
// 		type Event = Event;
// 		type DustRemoval = ();
// 		type ExistentialDeposit = ExistentialDeposit;
// 		type AccountStore = System;
// 		type MaxLocks = ();
// 		type WeightInfo = ();
// 	}

// 	impl WeightToFeePolynomial for WeightToFee {
// 		type Balance = u64;

// 		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
// 			smallvec![WeightToFeeCoefficient {
// 				degree: 1,
// 				coeff_frac: Perbill::zero(),
// 				coeff_integer: WEIGHT_TO_FEE.with(|v| *v.borrow()),
// 				negative: false,
// 			}]
// 		}
// 	}

// 	impl Config for Runtime {
// 		type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
// 		type TransactionByteFee = TransactionByteFee;
// 		type WeightToFee = WeightToFee;
// 		type FeeMultiplierUpdate = ();
// 	}

// 	pub struct ExtBuilder {
// 		balance_factor: u64,
// 		base_weight: u64,
// 		byte_fee: u64,
// 		weight_to_fee: u64
// 	}

// 	impl Default for ExtBuilder {
// 		fn default() -> Self {
// 			Self {
// 				balance_factor: 1,
// 				base_weight: 0,
// 				byte_fee: 1,
// 				weight_to_fee: 1,
// 			}
// 		}
// 	}

// 	impl ExtBuilder {
// 		pub fn base_weight(mut self, base_weight: u64) -> Self {
// 			self.base_weight = base_weight;
// 			self
// 		}
// 		pub fn byte_fee(mut self, byte_fee: u64) -> Self {
// 			self.byte_fee = byte_fee;
// 			self
// 		}
// 		pub fn weight_fee(mut self, weight_to_fee: u64) -> Self {
// 			self.weight_to_fee = weight_to_fee;
// 			self
// 		}
// 		pub fn balance_factor(mut self, factor: u64) -> Self {
// 			self.balance_factor = factor;
// 			self
// 		}
// 		fn set_constants(&self) {
// 			EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
// 			TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
// 			WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
// 		}
// 		pub fn build(self) -> sp_io::TestExternalities {
// 			self.set_constants();
// 			let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
// 			pallet_balances::GenesisConfig::<Runtime> {
// 				balances: if self.balance_factor > 0 {
// 					vec![
// 						(1, 10 * self.balance_factor),
// 						(2, 20 * self.balance_factor),
// 						(3, 30 * self.balance_factor),
// 						(4, 40 * self.balance_factor),
// 						(5, 50 * self.balance_factor),
// 						(6, 60 * self.balance_factor)
// 					]
// 				} else {
// 					vec![]
// 				},
// 			}.assimilate_storage(&mut t).unwrap();
// 			t.into()
// 		}
// 	}

// 	/// create a transaction info struct from weight. Handy to avoid building the whole struct.
// 	pub fn info_from_weight(w: Weight) -> DispatchInfo {
// 		// pays_fee: Pays::Yes -- class: DispatchClass::Normal
// 		DispatchInfo { weight: w, ..Default::default() }
// 	}

// 	fn post_info_from_weight(w: Weight) -> PostDispatchInfo {
// 		PostDispatchInfo {
// 			actual_weight: Some(w),
// 			pays_fee: Default::default(),
// 		}
// 	}

// 	fn post_info_from_pays(p: Pays) -> PostDispatchInfo {
// 		PostDispatchInfo {
// 			actual_weight: None,
// 			pays_fee: p,
// 		}
// 	}

// 	fn default_post_info() -> PostDispatchInfo {
// 		PostDispatchInfo {
// 			actual_weight: None,
// 			pays_fee: Default::default(),
// 		}
// 	}

// 	#[test]
// 	fn signed_extension_transaction_payment_work() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			let len = 10;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(0)
// 				.pre_dispatch(&1, CALL, &info_from_weight(5), len)
// 				.unwrap();
// 			assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(5), &default_post_info(), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 50 - 5);
// 		});
// 	}

// 	#[test]
// 	fn signed_extension_transaction_payment_multiplied_refund_works() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			let len = 10;
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));

// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			// 5 base fee, 10 byte fee, 3/2 * 100 weight fee, 5 tip
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 150 - 5);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
// 			);
// 			// 75 (3/2 of the returned 50 units of weight) is refunded
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 75 - 5);
// 		});
// 	}

// 	#[test]
// 	fn signed_extension_transaction_payment_is_bounded() {
// 		ExtBuilder::default()
// 			.balance_factor(1000)
// 			.byte_fee(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// maximum weight possible
// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>::from(0)
// 					.pre_dispatch(&1, CALL, &info_from_weight(Weight::max_value()), 10)
// 			);
// 			// fee will be proportional to what is the actual maximum weight in the runtime.
// 			assert_eq!(
// 				Balances::free_balance(&1),
// 				(10000 - <Runtime as frame_system::Config>::BlockWeights::get().max_block) as u64
// 			);
// 		});
// 	}

// 	#[test]
// 	fn signed_extension_allows_free_transactions() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// 1 ain't have a penny.
// 			assert_eq!(Balances::free_balance(1), 0);

// 			let len = 100;

// 			// This is a completely free (and thus wholly insecure/DoS-ridden) transaction.
// 			let operational_transaction = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::No,
// 			};
// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>::from(0)
// 					.validate(&1, CALL, &operational_transaction , len)
// 			);

// 			// like a InsecureFreeNormal
// 			let free_transaction = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Normal,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_noop!(
// 				ChargeTransactionPayment::<Runtime>::from(0)
// 					.validate(&1, CALL, &free_transaction , len),
// 				TransactionValidityError::Invalid(InvalidTransaction::Payment),
// 			);
// 		});
// 	}

// 	#[test]
// 	fn signed_ext_length_fee_is_also_updated_per_congestion() {
// 		ExtBuilder::default()
// 			.base_weight(5)
// 			.balance_factor(10)
// 			.build()
// 			.execute_with(||
// 		{
// 			// all fees should be x1.5
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));
// 			let len = 10;

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>::from(10) // tipped
// 					.pre_dispatch(&1, CALL, &info_from_weight(3), len)
// 			);
// 			assert_eq!(
// 				Balances::free_balance(1),
// 				100 // original
// 				- 10 // tip
// 				- 5 // base
// 				- 10 // len
// 				- (3 * 3 / 2) // adjusted weight
// 			);
// 		})
// 	}

// 	#[test]
// 	fn query_info_works() {
// 		let call = Call::Balances(BalancesCall::transfer(2, 69));
// 		let origin = 111111;
// 		let extra = ();
// 		let xt = TestXt::new(call, Some((origin, extra)));
// 		let info  = xt.get_dispatch_info();
// 		let ext = xt.encode();
// 		let len = ext.len() as u32;
// 		ExtBuilder::default()
// 			.base_weight(5)
// 			.weight_fee(2)
// 			.build()
// 			.execute_with(||
// 		{
// 			// all fees should be x1.5
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));

// 			assert_eq!(
// 				TransactionPayment::query_info(xt, len),
// 				RuntimeDispatchInfo {
// 					weight: info.weight,
// 					class: info.class,
// 					partial_fee:
// 						5 * 2 /* base * weight_fee */
// 						+ len as u64  /* len * 1 */
// 						+ info.weight.min(BlockWeights::get().max_block) as u64 * 2 * 3 / 2 /* weight */
// 				},
// 			);

// 		});
// 	}

// 	#[test]
// 	fn compute_fee_works_without_multiplier() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Next fee multiplier is zero
// 			assert_eq!(NextFeeMultiplier::get(), Multiplier::one());

// 			// Tip only, no fees works
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::No,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 10), 10);
// 			// No tip, only base fee works
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);
// 			// Tip + base fee works
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 69), 169);
// 			// Len (byte fee) + base fee works
// 			assert_eq!(Module::<Runtime>::compute_fee(42, &dispatch_info, 0), 520);
// 			// Weight fee + base fee works
// 			let dispatch_info = DispatchInfo {
// 				weight: 1000,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 1100);
// 		});
// 	}

// 	#[test]
// 	fn compute_fee_works_with_multiplier() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Add a next fee multiplier. Fees will be x3/2.
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));
// 			// Base fee is unaffected by multiplier
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

// 			// Everything works together :)
// 			let dispatch_info = DispatchInfo {
// 				weight: 123,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			// 123 weight, 456 length, 100 base
// 			assert_eq!(
// 				Module::<Runtime>::compute_fee(456, &dispatch_info, 789),
// 				100 + (3 * 123 / 2) + 4560 + 789,
// 			);
// 		});
// 	}

// 	#[test]
// 	fn compute_fee_works_with_negative_multiplier() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Add a next fee multiplier. All fees will be x1/2.
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(1, 2));

// 			// Base fee is unaffected by multiplier.
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

// 			// Everything works together.
// 			let dispatch_info = DispatchInfo {
// 				weight: 123,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			// 123 weight, 456 length, 100 base
// 			assert_eq!(
// 				Module::<Runtime>::compute_fee(456, &dispatch_info, 789),
// 				100 + (123 / 2) + 4560 + 789,
// 			);
// 		});
// 	}

// 	#[test]
// 	fn compute_fee_does_not_overflow() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Overflow is handled
// 			let dispatch_info = DispatchInfo {
// 				weight: Weight::max_value(),
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(
// 				Module::<Runtime>::compute_fee(
// 					<u32>::max_value(),
// 					&dispatch_info,
// 					<u64>::max_value()
// 				),
// 				<u64>::max_value()
// 			);
// 		});
// 	}

// 	#[test]
// 	fn refund_does_not_recreate_account() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			// So events are emitted
// 			System::set_block_number(10);
// 			let len = 10;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

// 			// kill the account between pre and post dispatch
// 			assert_ok!(Balances::transfer(Some(2).into(), 3, Balances::free_balance(2)));
// 			assert_eq!(Balances::free_balance(2), 0);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::free_balance(2), 0);
// 			// Transfer Event
// 			assert!(System::events().iter().any(|event| {
// 				event.event == Event::pallet_balances(pallet_balances::Event::Transfer(2, 3, 80))
// 			}));
// 			// Killed Event
// 			assert!(System::events().iter().any(|event| {
// 				event.event == Event::system(system::Event::KilledAccount(2))
// 			}));
// 		});
// 	}

// 	#[test]
// 	fn actual_weight_higher_than_max_refunds_nothing() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			let len = 10;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(101), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);
// 		});
// 	}

// 	#[test]
// 	fn zero_transfer_on_free_transaction() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			// So events are emitted
// 			System::set_block_number(10);
// 			let len = 10;
// 			let dispatch_info = DispatchInfo {
// 				weight: 100,
// 				pays_fee: Pays::No,
// 				class: DispatchClass::Normal,
// 			};
// 			let user = 69;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(0)
// 				.pre_dispatch(&user, CALL, &dispatch_info, len)
// 				.unwrap();
// 			assert_eq!(Balances::total_balance(&user), 0);
// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &dispatch_info, &default_post_info(), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::total_balance(&user), 0);
// 			// No events for such a scenario
// 			assert_eq!(System::events().len(), 0);
// 		});
// 	}

// 	#[test]
// 	fn refund_consistent_with_actual_weight() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(7)
// 			.build()
// 			.execute_with(||
// 		{
// 			let info = info_from_weight(100);
// 			let post_info = post_info_from_weight(33);
// 			let prev_balance = Balances::free_balance(2);
// 			let len = 10;
// 			let tip = 5;

// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(5, 4));

// 			let pre = ChargeTransactionPayment::<Runtime>::from(tip)
// 				.pre_dispatch(&2, CALL, &info, len)
// 				.unwrap();

// 			ChargeTransactionPayment::<Runtime>
// 				::post_dispatch(pre, &info, &post_info, len, &Ok(()))
// 				.unwrap();

// 			let refund_based_fee = prev_balance - Balances::free_balance(2);
// 			let actual_fee = Module::<Runtime>
// 				::compute_actual_fee(len as u32, &info, &post_info, tip);

// 			// 33 weight, 10 length, 7 base, 5 tip
// 			assert_eq!(actual_fee, 7 + 10 + (33 * 5 / 4) + 5);
// 			assert_eq!(refund_based_fee, actual_fee);
// 		});
// 	}

// 	#[test]
// 	fn post_info_can_change_pays_fee() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(7)
// 			.build()
// 			.execute_with(||
// 		{
// 			let info = info_from_weight(100);
// 			let post_info = post_info_from_pays(Pays::No);
// 			let prev_balance = Balances::free_balance(2);
// 			let len = 10;
// 			let tip = 5;

// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(5, 4));

// 			let pre = ChargeTransactionPayment::<Runtime>::from(tip)
// 				.pre_dispatch(&2, CALL, &info, len)
// 				.unwrap();

// 			ChargeTransactionPayment::<Runtime>
// 				::post_dispatch(pre, &info, &post_info, len, &Ok(()))
// 				.unwrap();

// 			let refund_based_fee = prev_balance - Balances::free_balance(2);
// 			let actual_fee = Module::<Runtime>
// 				::compute_actual_fee(len as u32, &info, &post_info, tip);

// 			// Only 5 tip is paid
// 			assert_eq!(actual_fee, 5);
// 			assert_eq!(refund_based_fee, actual_fee);
// 		});
// 	}
// }
