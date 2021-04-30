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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod impls;

use codec::Decode;
use cumulus_primitives_core::{DownwardMessageHandler, InboundDownwardMessage};
use frame_support::{traits::{Filter, Get}, weights::Weight};
use sp_std::{convert::TryFrom, marker::PhantomData};
use xcm::{VersionedXcm, v0::{BodyId, ExecuteXcm, Junction, MultiLocation, Xcm}};
use xcm_executor::traits::ShouldExecute;

pub struct UnqueuedDmpAsParent<MaxWeight, XcmExecutor, Call>(
	PhantomData<(MaxWeight, XcmExecutor, Call)>
);
impl<
	MaxWeight: Get<Weight>,
	XcmExecutor: ExecuteXcm<Call>,
	Call,
> DownwardMessageHandler for UnqueuedDmpAsParent<MaxWeight, XcmExecutor, Call> {
	fn handle_downward_message(msg: InboundDownwardMessage) -> Weight {
		let msg = VersionedXcm::<Call>::decode(&mut &msg.msg[..])
			.map(Xcm::<Call>::try_from);
		match msg {
			Ok(Ok(x)) => {
                log::debug!(target: "runtime::xcm::downward", "downward message: {:?}", x);
				let weight_limit = MaxWeight::get();
				XcmExecutor::execute_xcm(Junction::Parent.into(), x, weight_limit).weight_used()
			}
			Ok(Err(..)) => 0,
			Err(..) => 0,
		}
	}
}

pub struct IsMajorityOf<Prefix, Body>(PhantomData<(Prefix, Body)>);
impl<Prefix: Get<MultiLocation>, Body: Get<BodyId>> Filter<MultiLocation> for IsMajorityOf<Prefix, Body> {
	fn filter(l: &MultiLocation) -> bool {
		let maybe_suffix = l.match_and_split(&Prefix::get());
		log::debug!(target: "runtime::xcm::filter", "suffix: {:?}", maybe_suffix);
		let does_match = matches!(maybe_suffix, Some(Junction::Plurality { id, part }) if id == &Body::get() && part.is_majority());
		log::debug!(target: "runtime::xcm::filter", "matches: {:?}", does_match);
		does_match
	}
}

pub struct AllowUnpaidExecutionFromPlurality;
impl ShouldExecute for AllowUnpaidExecutionFromPlurality {
	fn should_execute<Call>(
		origin: &MultiLocation,
		_top_level: bool,
		_message: &xcm::v0::Xcm<Call>,
		_shallow_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		log::debug!(target: "runtime::should_execute","origin: {:?}", origin);
		frame_support::ensure!(
			matches!(origin, MultiLocation::X2(Junction::Parent, Junction::Plurality { .. })),
			()
		);
		log::debug!(target: "runtime::should_execute", "yes");
		Ok(())
	}
}