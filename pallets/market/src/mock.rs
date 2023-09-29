use crate as sugarfunge_market;
use frame_support::{
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
    PalletId,
};
use frame_system as system;
use sp_core::{ConstU128, ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sugarfunge_primitives::Balance;

pub const MILLICENTS: Balance = 10_000_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS;
pub const DOLLARS: Balance = 100 * CENTS;

parameter_types! {
    pub const BundleModuleId: PalletId = PalletId(*b"sug/bndl");
    pub const MarketModuleId: PalletId = PalletId(*b"sug/mrkt");
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxHolds = ();
    type MaxFreezes = ();
}

impl sugarfunge_asset::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CreateAssetClassDeposit = ConstU128<{ 500 * MILLICENTS }>;
    type Currency = Balances;
    type AssetId = u64;
    type ClassId = u64;
    type MaxClassMetadata = ConstU32<1>;
    type MaxAssetMetadata = ConstU32<1>;
}

// impl sugarfunge_bundle::Config for Test {
//     type RuntimeEvent = RuntimeEvent;
//     type PalletId = BundleModuleId;
//     type Currency = Balances;
//     type MaxAssets = ConstU32<20>;
// }

impl sugarfunge_market::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = MarketModuleId;
    type MarketId = u64;
    type MarketRateId = u64;
    type MaxTransactions = ConstU32<20>;
    type MaxMetadata = ConstU32<256>;
    type Asset = Asset;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Asset: sugarfunge_asset,
        Market: sugarfunge_market,
    }
);

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1000000 * DOLLARS), (2, 1000000 * DOLLARS)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn run_to_block(n: u64) {
    // SBP-M1 review: market pallet doesnt implement any hooks to these seem unnecessary?
    while System::block_number() < n {
        Market::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Market::on_initialize(System::block_number());
    }
}
