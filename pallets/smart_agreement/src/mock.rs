use crate as pallet_smart_agreement;
use frame_support::{parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system = 1,
		Balances: pallet_balances = 2,
		SmartAgreement: pallet_smart_agreement = 3,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sp_core::sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u32>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const NumVotesForApproval: u32 = 2;
	pub const MaxProposalsForUser: u32 = 2;
}

impl pallet_smart_agreement::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type NumVotesForApproval = NumVotesForApproval;
	type MaxProposalsForUser = MaxProposalsForUser;
	type EscrowPallet = ();
}

parameter_types! {
	pub const ExistentialDeposit: u32 = 1_000_000;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = u32;
	type DustRemoval = ();
	type Event = Event;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

// Test Externalities specific imports
use codec::Encode;
use frame_support::traits::GenesisBuild;
use sp_core::testing::SR25519;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use std::sync::Arc;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	// Generated using subkey
	const ALICE_PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	const BOB_PHRASE: &str =
		"monitor exhibit resource stumble subject nut valid furnace obscure misery satoshi assume";
	const EVE_PHRASE: &str =
		"rain matter permit differ deer master purchase galaxy avoid amused drink unit";

	let keystore = KeyStore::new();
	keystore.sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();
	keystore.sr25519_generate_new(SR25519, Some(BOB_PHRASE)).unwrap();
	keystore.sr25519_generate_new(SR25519, Some(EVE_PHRASE)).unwrap();

	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	GenesisBuild::<Test>::assimilate_storage(
		&pallet_smart_agreement::GenesisConfig { prefix: "GCP_BUCKET/agreements".encode() },
		&mut storage,
	)
	.unwrap();

	let mut ext = sp_io::TestExternalities::from(storage);
	ext.register_extension(KeystoreExt(Arc::new(keystore)));
	ext
}
