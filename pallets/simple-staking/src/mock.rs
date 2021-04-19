use super::*;
use crate as simple_staking;
use sp_core::H256;
use frame_support::{
	parameter_types, ord_parameter_types,
	traits::{FindAuthor, GenesisBuild},
	PalletId
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	testing::{Header},
};
use sp_consensus_aura::sr25519::AuthorityId;
use frame_system::{EnsureSignedBy};
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Aura: pallet_aura::{Pallet, Call, Storage, Config<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		SimpleStaking: simple_staking::{Pallet, Call, Storage, Event<T>},
		Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
}

pub struct Author4;
impl FindAuthor<u64> for Author4 {
	fn find_author<'a, I>(_digests: I) -> Option<u64>
		where I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Some(4)
	}
}

impl pallet_authorship::Config for Test {
	type FindAuthor = Author4;
	type UncleGenerations = ();
	type FilterUncle = ();
	type EventHandler = SimpleStaking;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_aura::Config for Test {
	type AuthorityId = AuthorityId;
}

ord_parameter_types! {
	pub const RootAccount: u64 = 777;
}
parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxAuthors: u32 = 20;
	pub const MaxInvulnerables: u32 = 20;
}
impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = EnsureSignedBy<RootAccount, u64>;
	type PotId = PotId;
	type MaxAuthors = MaxAuthors;
	type MaxInvulnerables = MaxInvulnerables;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 100),
			(2, 200),
		],
	};
	let genesis_staking = simple_staking::GenesisConfig::<Test> {
		invulnerables: vec![
			1,
			2,
			3,
		],
	};
	genesis.assimilate_storage(&mut t).unwrap();
	genesis_staking.assimilate_storage(&mut t).unwrap();
	t.into()
}
