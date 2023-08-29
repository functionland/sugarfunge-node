use crate::{self as sugarfunge_bag};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{OnFinalize, OnInitialize},
    PalletId,
};
use sp_core::{ConstU128, ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sugarfunge_primitives::Balance;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const MILLICENTS: Balance = 10_000_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

parameter_types! {
    pub const CurrencyModuleId: PalletId = PalletId(*b"sug/curr");
    pub const BagModuleId: PalletId = PalletId(*b"sug/crow");
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
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
    type DbWeight = ();
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

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
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
    type CreateAssetClassDeposit = ConstU128<1>;
    type Currency = Balances;
    type AssetId = u64;
    type ClassId = u64;
    type MaxClassMetadata = ConstU32<1>;
    type MaxAssetMetadata = ConstU32<1>;
}

impl sugarfunge_bag::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = BagModuleId;
    type CreateBagDeposit = ConstU128<1>;
    type Currency = Balances;
    type MaxHolders = ConstU32<20>;
    type MaxDepositClassAssets = ConstU32<100>;
    type MaxDepositTypeAssets = ConstU32<100>;
    type Asset = Asset;
}

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Asset: sugarfunge_asset,
        Bag: sugarfunge_bag,
    }
);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 100_000_000 * DOLLARS), (2, 100_000_000 * DOLLARS)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Bag::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Bag::on_initialize(System::block_number());
    }
}
