#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
    PalletId,
};
use scale_info::TypeInfo;
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_arithmetic::ArithmeticError;
use sp_runtime::{traits::AccountIdConversion, BoundedVec, RuntimeDebug};
use sp_std::prelude::*;
use sugarfunge_asset::AssetInterface;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Store the parameters of a Bag Class
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BagClass<AccountId, ClassId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting claims
    pub class_id: ClassId,
}

/// Store the parameters of a Bag
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Bag<Balance, AccountId, ClassId, AssetId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting shares
    pub class_id: ClassId,
    /// The asset_id for minting shares
    pub asset_id: AssetId,
    /// Total number of shares
    pub total_shares: Balance,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]

    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// PalletId of the asset representing the Bag
        type PalletId: Get<PalletId>;

        /// Handles the transaction balance of accounts
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Max number of holders of a bag
        #[pallet::constant]
        type MaxHolders: Get<u32>;

        /// The minimum balance to create bag
        #[pallet::constant]
        type CreateBagDeposit: Get<CurrencyBalance<Self>>;

        /// Max quantity of asset classes an account can deposit
        #[pallet::constant]
        type MaxDepositClassAssets: Get<u32>;

        /// Max quantity of different assets
        #[pallet::constant]
        type MaxDepositTypeAssets: Get<u32>;

        type Asset: AssetInterface<AccountId = Self::AccountId>;
    }

    type CurrencyBalance<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ClassMetadataOf<T> = <<T as Config>::Asset as AssetInterface>::Metadata;

    pub type ClassId<T> = <<T as Config>::Asset as AssetInterface>::ClassId;

    pub type AssetId<T> = <<T as Config>::Asset as AssetInterface>::AssetId;

    pub type Balance<T> = <<T as Config>::Asset as AssetInterface>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type BagClasses<T: Config> =
        StorageMap<_, Blake2_128, ClassId<T>, BagClass<T::AccountId, ClassId<T>>>;

    #[pallet::storage]
    pub(super) type Bags<T: Config> = StorageMap<
        _,
        Blake2_128,
        T::AccountId,
        Bag<Balance<T>, T::AccountId, ClassId<T>, AssetId<T>>,
    >;

    #[pallet::storage]
    pub(super) type NextBagId<T: Config> = StorageMap<_, Blake2_128, ClassId<T>, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A class for Bags was successfully created
        Register {
            who: T::AccountId,
            class_id: ClassId<T>,
        },
        /// A Bag was successfully created
        Created {
            bag: T::AccountId,
            who: T::AccountId,
            class_id: ClassId<T>,
            asset_id: AssetId<T>,
            owners: Vec<T::AccountId>,
        },
        // A deposit was made into the refered bag
        Deposit {
            bag: T::AccountId,
            who: T::AccountId,
        },
        // An owner withdraw the currency from the bag to their personal account
        Sweep {
            bag: T::AccountId,
            who: T::AccountId,
            to: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The class_id used to register the bag already exists, try a different clas_id parameter
        BagClassExists,
        /// The asset_id used to create the bag already exists, try a different asset_id parameter
        BagExists,
        /// The class_id used doesn't exist, you must register it first
        InvalidBagClass,
        /// The bag_id used doesn't exist, you must create it first
        InvalidBag,
        /// Insufficient shares to make the transaction
        InsufficientShares,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new Bag class_id
        #[pallet::call_index(0)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn register_class(
            origin: OriginFor<T>,
            class_id: ClassId<T>,
            metadata: ClassMetadataOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_register_class(who, class_id, metadata)
        }

        // SBP-M1 review: whilst a deposit is taken for a bag, anyone can call this to create bags and mint tokens. Consider whether this should perhaps be limited to the class owner or governance?
        /// Create a Bag of a given class_id
        #[pallet::call_index(1)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create_bag(
            origin: OriginFor<T>,
            class_id: ClassId<T>,
            holders: BoundedVec<(T::AccountId, Balance<T>), <T as Config>::MaxHolders>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_create_bag(who, class_id, holders)
        }

        /// Deposit into a given bag
        #[pallet::call_index(2)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn deposit(
            origin: OriginFor<T>,
            bag: T::AccountId,
            assets: BoundedVec<
                (
                    ClassId<T>,
                    BoundedVec<(AssetId<T>, Balance<T>), <T as Config>::MaxDepositTypeAssets>,
                ),
                <T as Config>::MaxDepositClassAssets,
            >,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_deposit(who, bag, assets)
        }

        // Withdraw from a given bag
        #[pallet::call_index(3)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn sweep(origin: OriginFor<T>, bag: T::AccountId, to: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_sweep(who, to, bag)
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_register_class(
        who: T::AccountId,
        class_id: ClassId<T>,
        metadata: ClassMetadataOf<T>,
    ) -> DispatchResult {
        ensure!(
            !BagClasses::<T>::contains_key(class_id),
            Error::<T>::BagClassExists
        );

        let owner = <T as Config>::PalletId::get().into_account_truncating();

        let _ = T::Asset::create_class(who.clone(), owner, class_id, metadata);

        let bag_class = BagClass {
            operator: who.clone(),
            class_id,
        };

        // SBP-M1 review: BagClasses are never removed, consider mechanism for avoiding state bloat
        // SBP-M1 review: BagClasses removal should also remove NextBagId entry
        BagClasses::<T>::insert(class_id, &bag_class);

        Self::deposit_event(Event::Register { who, class_id });

        Ok(())
    }

    fn get_operator_account_id() -> <T as frame_system::Config>::AccountId {
        <T as Config>::PalletId::get().into_account_truncating()
    }

    pub fn do_create_bag(
        who: T::AccountId,
        class_id: ClassId<T>,
        holders: BoundedVec<(T::AccountId, Balance<T>), <T as Config>::MaxHolders>,
    ) -> DispatchResult {
        ensure!(
            BagClasses::<T>::contains_key(class_id),
            Error::<T>::InvalidBagClass
        );

        let bag_id = NextBagId::<T>::try_mutate(&class_id, |id| -> Result<u64, DispatchError> {
            let current_id = *id;
            ensure!((*id).checked_add(1).is_some(), ArithmeticError::Overflow);
            *id = *id + 1;
            Ok(current_id)
        })?;

        let block_number: u64 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let sub = vec![block_number, class_id.into(), bag_id];
        let bag = <T as Config>::PalletId::get().into_sub_account_truncating(sub);

        ensure!(!Bags::<T>::contains_key(&bag), Error::<T>::BagExists);

        // SBP-M1 review: consider whether bag deposit should increase based on number of shares. A static deposit value may not account for very large share structures.
        let deposit = T::CreateBagDeposit::get();
        <T as Config>::Currency::transfer(&who, &bag, deposit, AllowDeath)?;

        let asset_id: AssetId<T> = bag_id.into();

        let operator = Self::get_operator_account_id();

        // Mint shares for each owner
        // SBP-M1 review: Each iteration is adding state and should be benchmarked accordingly.
        for holder in holders.iter() {
            let _ = T::Asset::mint(
                operator.clone(),
                holder.0.clone(),
                class_id,
                asset_id,
                holder.1,
            );
        }

        let new_bag = Bag {
            operator,
            class_id,
            asset_id,
            total_shares: holders.iter().map(|x| x.1).sum(),
        };

        // SBP-M1 review: Bags are never removed, consider mechanism for avoiding state bloat
        // SBP-M1 review: corresponding bag accounts need cleanup when bag removed, with deposit returned
        Bags::<T>::insert(&bag, &new_bag);

        Self::deposit_event(Event::Created {
            bag,
            who,
            class_id,
            asset_id,
            owners: holders.iter().map(|x| x.0.clone()).collect(),
        });

        Ok(())
    }

    pub fn do_deposit(
        who: T::AccountId,
        bag: T::AccountId,
        assets: BoundedVec<
            (
                ClassId<T>,
                BoundedVec<(AssetId<T>, Balance<T>), <T as Config>::MaxDepositTypeAssets>,
            ),
            <T as Config>::MaxDepositClassAssets,
        >,
    ) -> DispatchResult {
        ensure!(Bags::<T>::contains_key(bag.clone()), Error::<T>::InvalidBag);

        // SBP-M1 review: Each iteration is changing state and should be benchmarked accordingly.
        for asset in assets.iter() {
            T::Asset::batch_transfer_from(
                who.clone(),
                who.clone(),
                bag.clone(),
                asset.0,
                asset.1.iter().map(|x| x.0).collect(),
                asset.1.iter().map(|x| x.1).collect(),
            )?;
        }
        Self::deposit_event(Event::Deposit { bag, who });

        Ok(())
    }

    // SBP-M1 review: should sweep not remove bag from Bags storage item?
    pub fn do_sweep(who: T::AccountId, to: T::AccountId, bag: T::AccountId) -> DispatchResult {
        let bag_info = Bags::<T>::get(bag.clone()).ok_or(Error::<T>::InvalidBag)?;

        let shares = T::Asset::balance_of(who.clone(), bag_info.class_id, bag_info.asset_id);
        ensure!(
            // SBP-M1 review: shares >= bag_info.total_shares ?
            shares == bag_info.total_shares,
            Error::<T>::InsufficientShares
        );

        let operator = Self::get_operator_account_id();

        // Burn bag shares
        T::Asset::burn(
            operator,
            who.clone(),
            bag_info.class_id,
            bag_info.asset_id,
            bag_info.total_shares,
        )?;

        let balances = T::Asset::balances_of_owner(bag.clone())?;
        // SBP-M1 review: iteration should be bounded to complete within block limits.
        // SBP-M1 review: destructure into clearer variable names
        let balances = balances.iter().fold(
            Vec::<(ClassId<T>, Vec<AssetId<T>>, Vec<Balance<T>>)>::new(),
            |mut assets, (class_id, asset_id, balance)| {
                let position =
                    if let Some(pos) = assets.iter().position(|asset| asset.0 == *class_id) {
                        pos
                    } else {
                        let pos = assets.len();
                        assets.push((*class_id, vec![], vec![]));
                        pos
                    };
                if let Some(asset) = assets.get_mut(position) {
                    asset.1.push(*asset_id);
                    asset.2.push(*balance);
                }
                assets
            },
        );

        for balance in balances.iter() {
            T::Asset::batch_transfer_from(
                bag.clone(),
                bag.clone(),
                to.clone(),
                balance.0,
                balance.1.clone(),
                balance.2.clone(),
            )?;
        }
        Self::deposit_event(Event::Sweep { bag, who, to });
        Ok(())
    }
}
