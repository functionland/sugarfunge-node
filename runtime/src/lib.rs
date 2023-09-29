#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use pallet_grandpa::AuthorityId as GrandpaId;

use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::traits::{
    AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, One, OpaqueKeys,
};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perbill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use sugarfunge_validator_set as validator_set;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstU128, ConstU32, ConstU64, ConstU8, Contains, EitherOfDiverse, EqualPrivilegeOnly,
        FindAuthor, KeyOwnerProofSystem, Nothing, Randomness,
    },
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        ConstantMultiplier, IdentityFee, Weight,
    },
    ConsensusEngineId, PalletId, StorageValue,
};
use frame_system::EnsureRoot;

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
#[cfg(any(feature = "std", test))]
pub use sugarfunge_bundle::Call as BundleCall;
#[cfg(any(feature = "std", test))]
pub use sugarfunge_market::Call as MarketCall;

#[cfg(any(feature = "std", test))]
pub use fula_pool::Call as FulaPoolCall;
#[cfg(any(feature = "std", test))]
pub use functionland_fula::Call as FulaCall;

/// Constant values used within the runtime.
mod constants;
pub use constants::{currency::*, time::*};
pub use sugarfunge_primitives::{
    AccountId, AccountIndex, Amount, AssetId, Balance, BlockNumber, ClassId, Hash, Index, Moment,
    Signature,
};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("sugarfunge-node"),
    impl_name: create_runtime_str!("sugarfunge-node"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// Maximum metadata size usually for JSON content
// SBP-M1 review: evaluate whether this is still feasible if becoming a parachain and adjust design accordingly
const METADATA_SIZE: u32 = 1024 * 4;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const BlockHashCount: BlockNumber = 2400;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = frame_support::traits::Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    /// The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxAuthorities: u32 = 32;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
    type MaxSetIdSessionEntries = ConstU64<0>;

    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = ();
    type MaxHolds = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // SBP-M1 review: consider what happens with fees and provide an OnUnbalanced handler to CurrencyAdapter accordingly
    type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    // SBP-M1 review: consider
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    // SBP-M1 review: consider updating to SlowAdjustingFeeUpdate as per https://github.com/paritytech/polkadot/blob/6f991987c0b4cbbd7d4badc9ef08d83da5fefbfd/runtime/polkadot/src/lib.rs#L317
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // SBP-M1 review: only used once, consider inlining
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        BlockWeights::get().max_block;
}

// SBP-M1 review: consider whether required
impl pallet_scheduler::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = ConstU32<50>;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    // SBP-M1 review: consider adding Preimage pallet as required
    type Preimages = ();
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 48 * HOURS;
    pub MaxProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = ConstU32<100>;
    type MaxMembers = ConstU32<100>;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type SetMembersOrigin = EnsureRoot<AccountId>;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type MaxProposalWeight = MaxProposalWeight;
}

type EnsureRootOrHalfCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;

// SBP-M1 review: no benchmarks, zero weights
impl validator_set::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddRemoveOrigin = EnsureRootOrHalfCouncil;
    type MinAuthorities = ConstU32<1>;
    type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = validator_set::ValidatorOf<Self>;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = ValidatorSet;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = opaque::SessionKeys;
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
}

// SBP-M1 review: no benchmarks, invalid static weights
impl sugarfunge_asset::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CreateAssetClassDeposit = ConstU128<{ 500 * MILLICENTS }>;
    type Currency = Balances;
    // SBP-M1 review: are these overkill for expected usage for the life of the chain?
    type AssetId = u64;
    type ClassId = u64;
    type MaxClassMetadata = ConstU32<{ METADATA_SIZE }>;
    type MaxAssetMetadata = ConstU32<{ METADATA_SIZE }>;
}

parameter_types! {
    pub const BundleModuleId: PalletId = PalletId(*b"sug/bndl");
    pub const BagModuleId: PalletId = PalletId(*b"sug/crow");
    pub const MarketModuleId: PalletId = PalletId(*b"sug/mrkt");
}

// SBP-M1 review: no benchmarks, invalid static weights
impl sugarfunge_bundle::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = BundleModuleId;
    type Currency = Balances;
    type MaxAssets = ConstU32<20>;
}

// SBP-M1 review: no benchmarks, invalid static weights
impl sugarfunge_bag::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = BagModuleId;
    type CreateBagDeposit = ConstU128<{ 500 * MILLICENTS }>;
    type Currency = Balances;
    type MaxHolders = ConstU32<20>;
    type MaxDepositClassAssets = ConstU32<100>;
    type MaxDepositTypeAssets = ConstU32<100>;
    type Asset = Asset;
}

// SBP-M1 review: no benchmarks, invalid static weights
impl sugarfunge_market::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = MarketModuleId;
    type MarketId = u64;
    type MarketRateId = u64;
    type MaxTransactions = ConstU32<20>;
    type MaxMetadata = ConstU32<256>;
    type Asset = Asset;
}

impl functionland_fula::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxManifestMetadata = ConstU32<128>;
    type MaxCID = ConstU32<128>;
    type Pool = Pool;
}

impl fula_pool::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = ConstU32<128>;
    type MaxPoolParticipants = ConstU32<200>;
}

impl pallet_node_authorization::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxWellKnownNodes = ConstU32<8>;
    type MaxPeerIdLength = ConstU32<128>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

construct_runtime!(
    pub struct Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // SBP-M1 review: simplify syntax https://github.com/paritytech/substrate/blob/ff24c60ac7d9f87727ecdd0ded9a80c56e4f4b65/bin/node-template/runtime/src/lib.rs#L284
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>},
        Sudo: pallet_sudo,
        Scheduler: pallet_scheduler,
        Council: pallet_collective::<Instance1>,
        NodeAuthorization: pallet_node_authorization,
        ValidatorSet: validator_set,
        Session: pallet_session,

        // SugarFunge pallets
        Asset: sugarfunge_asset,
        Bundle: sugarfunge_bundle,
        Bag: sugarfunge_bag,
        Market: sugarfunge_market,

        // Functionland pallets
        Fula: functionland_fula,
        Pool: fula_pool,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;
#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
        // SBP-M1 review: add all other pallets used once benchmarking implemented and then run benchmarks
    );
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }

        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;
            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);
            let storage_info = AllPalletsWithSystem::storage_info();
            (list, storage_info)
        }

        // SBP-M1 review: outdated implementation, align with https://github.com/paritytech/substrate/blob/ff24c60ac7d9f87727ecdd0ded9a80c56e4f4b65/bin/node-template/runtime/src/lib.rs#L530
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}


            use frame_support::traits::WhitelistedStorageKeys;
            let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

            // SBP-M1 review: as noted above, benchmarks are now read from add_benchmarks! and dont need to be manually defined here anymore
            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::traits::WhitelistedStorageKeys;
    use sp_core::hexdisplay::HexDisplay;
    use std::collections::HashSet;

    #[test]
    fn check_whitelist() {
        let whitelist: HashSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
            .iter()
            .map(|e| HexDisplay::from(&e.key).to_string())
            .collect();

        // Block Number
        assert!(
            whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
        );
        // Total Issuance
        assert!(
            whitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
        );
        // Execution Phase
        assert!(
            whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
        );
        // Event Count
        assert!(
            whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
        );
        // System Events
        assert!(
            whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
        );
    }
}
