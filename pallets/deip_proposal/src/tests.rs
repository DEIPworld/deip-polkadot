#![cfg_attr(not(feature = "std"), no_std)]

use crate as pallet_deip_proposal;

use sp_std::prelude::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
    	// Utility: pallet_utility::{Module, Call, Event},
		// RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
		// Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		// Aura: pallet_aura::{Module, Config<T>},
		// Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
		// Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		// TransactionPayment: pallet_transaction_payment::{Module, Storage},
		// Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
		// // Include the custom logic from the template pallet in the runtime.
		// TemplateModule: pallet_template::{Module, Call, Storage, Event<T>},
		// Deip: pallet_deip::{Module, Call, Storage, Event<T>, Config},
		DeipProposal: pallet_deip_proposal::{Module, Call, Storage, Event<T>, Config},
		// Multisig: pallet_multisig::{Module, Call, Storage, Event<T>, Config},
	}
);

frame_support::parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = sp_core::H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
    type Header = sp_runtime::testing::Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

impl crate::Config for TestRuntime {
    type Event = Event;
    type Call = Call;
}

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
        sp_io::TestExternalities::from(storage)
    }
}

#[test]
fn fake_test_example() {
    ExtBuilder::build().execute_with(|| {
        // ...test conditions...
    })
}

#[test]
fn create_proposal() {
    ExtBuilder::build().execute_with(|| {
        Call::DeipProposal(crate::Call::propose(Vec::new()));
    })
}
