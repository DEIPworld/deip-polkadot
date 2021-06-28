#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate as pallet_deip_org;
use super::{*, Event as RawEvent, Call as RawCall};

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
		DeipOrg: pallet_deip_org::{Module, Call, Storage, Event<T>, Config},
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

fn with_test_ext<R>(t: impl FnOnce() -> R) -> R {
    ExtBuilder::build().execute_with(t)
}

use frame_support::{assert_noop, assert_ok};
use crate::org::*;
use sp_std::str::FromStr;
use frame_system::RawOrigin;

fn last_event() -> Event {
	frame_system::Module::<TestRuntime>::events().pop().map(|e| e.event).expect("Event expected")
}

fn expect_event<E: Into<Event>>(e: E) {
	assert_eq!(last_event(), e.into());
}

fn plain_key_source(who: u64) -> InputKeySource<u64> {
    InputKeySource { signatories: vec![who], threshold: 0 }
}

#[test]
#[ignore]
fn fake_test_example() {
    with_test_ext(|| {
        // ...test conditions...
    })
}

#[test]
fn org_create() {
    with_test_ext(|| {
        System::set_block_number(1);
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        assert_ok!(DeipOrg::create(Origin::signed(who), name, plain_key_source(who)));
        assert!(matches!(
            last_event(),
            Event::pallet_deip_org(RawEvent::OrgCreate(org))
            if org.key() == &who && org.name() == &name
        ));
    })
}

#[test]
fn org_create_exists() {
    with_test_ext(|| {
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        DeipOrg::create(Origin::signed(who), name, plain_key_source(who)).expect("create OK");
        assert_noop!(
            DeipOrg::create(Origin::signed(who), name, plain_key_source(who)),
            Error::<TestRuntime>::Exists,
        );
    })
}

#[test]
fn org_transfer_ownership() {
    with_test_ext(|| {
        System::set_block_number(1);
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        DeipOrg::create(Origin::signed(who), name, plain_key_source(who)).expect("create OK");
        let transfer_to = 2;
        assert_ok!(DeipOrg::transfer_ownership(Origin::signed(who), name, transfer_to, plain_key_source(transfer_to)));
        assert!(matches!(
            last_event(),
            Event::pallet_deip_org(RawEvent::OrgTransferOwnership(org))
            if org.key() == &transfer_to && org.name() == &name
        ));
    })
}

#[test]
fn org_transfer_ownership_not_found() {
    with_test_ext(|| {
        System::set_block_number(1);
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        let transfer_to = 2;
        assert_noop!(
            DeipOrg::transfer_ownership(Origin::signed(who), name, transfer_to, plain_key_source(who)),
            Error::<TestRuntime>::NotFound,
        );
    })
}

#[test]
fn org_transfer_ownership_forbidden() {
    with_test_ext(|| {
        System::set_block_number(1);
        let owner = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        DeipOrg::create(Origin::signed(owner), name, plain_key_source(owner)).expect("create OK");
        let transfer_to = 2;
        let other = 3;
        assert_noop!(
            DeipOrg::transfer_ownership(Origin::signed(transfer_to), name, transfer_to, plain_key_source(transfer_to)),
            Error::<TestRuntime>::Forbidden,
        );
        assert_noop!(
            DeipOrg::transfer_ownership(Origin::signed(other), name, transfer_to, plain_key_source(transfer_to)),
            Error::<TestRuntime>::Forbidden,
        );
    })
}

#[test]
fn org_on_behalf() {
    with_test_ext(|| {
        System::set_block_number(1);
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        DeipOrg::create(
            Origin::signed(who),
            name,
            plain_key_source(who)
        ).expect("create OK");
        let transfer_to = 2;
        assert_ok!(
            DeipOrg::on_behalf(
                Origin::signed(who),
                name,
                Box::new(Call::DeipOrg(RawCall::transfer_ownership(
                    name,
                    transfer_to,
                    plain_key_source(transfer_to)
                )))
            )
        );
    })
}

#[test]
fn org_on_behalf_not_found() {
    with_test_ext(|| {
        System::set_block_number(1);
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        let transfer_to = 2;
        assert_noop!(
            DeipOrg::on_behalf(
                Origin::signed(who),
                name,
                Box::new(Call::DeipOrg(RawCall::transfer_ownership(
                    name,
                    transfer_to,
                    plain_key_source(transfer_to)
                )))
            ),
            Error::<TestRuntime>::NotFound,
        );
    })
}

#[test]
fn org_on_behalf_forbidden() {
    with_test_ext(|| {
        System::set_block_number(1);
        let who = 1;
        let name = OrgName::from_slice("test_org\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
        DeipOrg::create(
            Origin::signed(who),
            name,
            plain_key_source(who)
        ).expect("create OK");
        let transfer_to = 2;
        assert_noop!(
            DeipOrg::on_behalf(
                Origin::signed(transfer_to),
                name,
                Box::new(Call::DeipOrg(RawCall::transfer_ownership(
                    name,
                    transfer_to,
                    plain_key_source(transfer_to)
                )))
            ),
            Error::<TestRuntime>::Forbidden,
        );
    })
}
