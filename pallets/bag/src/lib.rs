#![cfg_attr(not(feature = "std"), no_std)]

use codec::EncodeLike;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
    PalletId,
};
use scale_info::TypeInfo;
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_runtime::{traits::AccountIdConversion, BoundedVec, RuntimeDebug};
use sp_std::fmt::Debug;
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// SBP-M1 review: add doc comment for struct
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BagClass<AccountId, ClassId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting claims
    pub class_id: ClassId,
}

// SBP-M1 review: add doc comment for struct
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
    // SBP-M1 review: loose pallet coupling preferable, can sugarfunge_asset dependency be brought in via trait (associated type on this pallets Config trait)?
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
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
        type CreateBagDeposit: Get<BalanceOf<Self>>;

        /// Max instances of class metadata
        #[pallet::constant]
        type MaxClassMetadata: Get<u32>;

        /// Max quantity of asset classes an account can deposit
        #[pallet::constant]
        type MaxDepositClassAssets: Get<u32>;

        /// Max quantity of different assets
        #[pallet::constant]
        type MaxDepositTypeAssets: Get<u32>;

        type Balance: Copy + TypeInfo + Debug + Eq + EncodeLike + Encode + Decode + MaxEncodedLen;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ClassMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxClassMetadata>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type BagClasses<T: Config> =
        StorageMap<_, Blake2_128, T::ClassId, BagClass<T::AccountId, T::ClassId>>;

    #[pallet::storage]
    pub(super) type Bags<T: Config> = StorageMap<
        _,
        Blake2_128,
        T::AccountId,
        Bag<T::Balance, T::AccountId, T::ClassId, T::AssetId>,
    >;

    #[pallet::storage]
    pub(super) type NextBagId<T: Config> = StorageMap<_, Blake2_128, T::ClassId, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A class for Bags was successfully created
        Register {
            who: T::AccountId,
            class_id: T::ClassId,
        },
        /// A Bag was successfully created
        Created {
            bag: T::AccountId,
            who: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
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
            class_id: T::ClassId,
            metadata: ClassMetadataOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_register_class(&who, class_id, metadata)
        }

        // SBP-M1 review: whilst a deposit is taken for a bag, anyone can call this to create bags and mint tokens. Consider whether this should perhaps be limited to the class owner or governance?
        /// Create a Bag of a given class_id
        #[pallet::call_index(1)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create_bag(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            holders: BoundedVec<(T::AccountId, T::Balance), <T as Config>::MaxHolders>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_create_bag(&who, class_id, holders)
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
                    T::ClassId,
                    BoundedVec<(T::AssetId, T::Balance), <T as Config>::MaxDepositTypeAssets>,
                ),
                <T as Config>::MaxDepositClassAssets,
            >,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_deposit(&who, &bag, assets)
        }

        // Withdraw from a given bag
        #[pallet::call_index(3)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn sweep(origin: OriginFor<T>, bag: T::AccountId, to: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_sweep(&who, &to, &bag)
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_register_class(
        who: &T::AccountId,
        class_id: T::ClassId,
        // SBP-M1 review: not consumed, can pass by reference
        metadata: ClassMetadataOf<T>,
    ) -> DispatchResult {
        ensure!(
            !BagClasses::<T>::contains_key(class_id),
            Error::<T>::BagClassExists
        );

        // SBP-M1 review: refactor into account_id() function
        let owner = <T as Config>::PalletId::get().into_account_truncating();
        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : CreateClass
        // SBP-M1 review: unnecessary clone and needless borrows
        sugarfunge_asset::Pallet::<T>::do_create_class(&who, &owner, class_id, metadata.clone())?;

        let bag_class = BagClass {
            operator: who.clone(),
            class_id,
        };

        // SBP-M1 review: BagClasses are never removed, consider mechanism for avoiding state bloat
        // SBP-M1 review: BagClasses removal should also remove NextBagId entry
        BagClasses::<T>::insert(class_id, &bag_class);

        Self::deposit_event(Event::Register {
            // SBP-M1 review: unnecessary clone
            who: who.clone(),
            class_id,
        });

        Ok(())
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_create_bag(
        who: &T::AccountId,
        class_id: T::ClassId,
        holders: BoundedVec<(T::AccountId, T::Balance), <T as Config>::MaxHolders>,
    ) -> DispatchResult {
        ensure!(
            // SBP-M1 review: needless borrow
            BagClasses::<T>::contains_key(&class_id),
            Error::<T>::InvalidBagClass
        );

        // SBP-M1 review: try_mutate unnecessary as no error returned, use .mutate(). Safe math fix will require try_mutate however, returning ArithmeticError::Overflow depending on id value
        // SBP-M1 review: needless borrow
        let bag_id = NextBagId::<T>::try_mutate(&class_id, |id| -> Result<u64, DispatchError> {
            let current_id = *id;
            // SBP-M1 review: use safe math
            *id = *id + 1;
            Ok(current_id)
        })?;

        // SBP-M1 review: convert directly into u64 to save a cast
        let block_number: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let sub = vec![block_number as u64, class_id.into(), bag_id];
        let bag = <T as Config>::PalletId::get().into_sub_account_truncating(sub);

        ensure!(!Bags::<T>::contains_key(&bag), Error::<T>::BagExists);

        // SBP-M1 review: consider whether bag deposit should increase based on number of shares. A static deposit value may not account for very large share structures.
        let deposit = T::CreateBagDeposit::get();
        <T as Config>::Currency::transfer(who, &bag, deposit, AllowDeath)?;

        let asset_id: T::AssetId = bag_id.into();

        // SBP-M1 review: refactor into account_id() function
        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Mint shares for each owner
        // SBP-M1 review: iteration should be bounded to complete within block limits. Each iteration is adding state and should be benchmarked accordingly.
        for (idx, owner) in owners.iter().enumerate() {
            // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : CreateClass + Mint
            sugarfunge_asset::Pallet::<T>::do_mint(
                &operator,
                // SBP-M1 review: needless borrow
                &owner,
                class_id,
                asset_id,
                // SBP-M1 review: indexing may panic, .get() preferred. Eliminated by using tuple as suggested above
                shares[idx],
            )?;
        }

        let new_bag = Bag {
            // SBP-M1 review: unnecessary clone
            operator: operator.clone(),
            class_id,
            asset_id,
            total_shares: shares.iter().sum(),
        };

        // SBP-M1 review: Bags are never removed, consider mechanism for avoiding state bloat
        // SBP-M1 review: corresponding bag accounts need cleanup when bag removed, with deposit returned
        Bags::<T>::insert(&bag, &new_bag);

        // SBP-M1 review: unnecessary clones
        Self::deposit_event(Event::Created {
            bag: bag.clone(),
            who: who.clone(),
            class_id,
            asset_id,
            owners: owners.clone(),
        });

        // SBP-M1 review: unnecessary clone and return type not used
        Ok(bag.clone())
    }

    pub fn do_deposit(
        who: &T::AccountId,
        bag: &T::AccountId,
        assets: BoundedVec<
            (
                T::ClassId,
                BoundedVec<(T::AssetId, T::Balance), <T as Config>::MaxDepositTypeAssets>,
            ),
            <T as Config>::MaxDepositClassAssets,
        >,
    ) -> DispatchResult {
        // SBP-M1 review: needless borrow
        ensure!(Bags::<T>::contains_key(&bag), Error::<T>::InvalidBag);

        // SBP-M1 review: iteration should be bounded to complete within block limits. Each iteration is changing state and should be benchmarked accordingly.
        for (idx, class_id) in class_ids.iter().enumerate() {
            // SBP-M1 review: improve parameter data type to avoid this
            ensure!(
                // SBP-M1 review: indexing may panic, .get() preferred. Can be eliminated by improving parameter type
                asset_ids[idx].len() == amounts[idx].len(),
                Error::<T>::InvalidArrayLength
            );

            // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : CreateClass + Mint + BatchTransfer
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                // SBP-M1 review: needless borrows
                &who,
                &who,
                &bag,
                *class_id,
                // SBP-M1 review: indexing may panic, .get() preferred. Can be eliminated by improving parameter type
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        // SBP-M1 review: unnecessary clones
        Self::deposit_event(Event::Deposit {
            bag: bag.clone(),
            who: who.clone(),
        });

        // SBP-M1 review: unnecessary .into()
        Ok(().into())
    }

    // SBP-M1 review: should sweep not remove bag from Bags storage item?
    // SBP-M1 review: too many lines, refactor
    pub fn do_sweep(who: &T::AccountId, to: &T::AccountId, bag: &T::AccountId) -> DispatchResult {
        let bag_info = Bags::<T>::get(bag).ok_or(Error::<T>::InvalidBag)?;

        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : Inspect
        let shares =
            sugarfunge_asset::Pallet::<T>::balance_of(who, bag_info.class_id, bag_info.asset_id);
        ensure!(
            // SBP-M1 review: shares >= bag_info.total_shares ?
            shares == bag_info.total_shares,
            Error::<T>::InsufficientShares
        );

        // SBP-M1 review: refactor into account_id() function
        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Burn bag shares
        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : Burn
        sugarfunge_asset::Pallet::<T>::do_burn(
            &operator,
            who,
            bag_info.class_id,
            bag_info.asset_id,
            bag_info.total_shares,
        )?;

        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : Inspect
        // SBP-M1 review: needless borrow
        let balances = sugarfunge_asset::Pallet::<T>::balances_of_owner(&bag)?;
        // SBP-M1 review: iteration should be bounded to complete within block limits.
        // SBP-M1 review: destructure into clearer variable names
        let balances = balances.iter().fold(
            (
                // SBP-M1 review: simplify using tuple
                Vec::<T::ClassId>::new(),
                Vec::<Vec<T::AssetId>>::new(),
                Vec::<Vec<T::Balance>>::new(),
            ),
            |(mut class_ids, mut asset_ids, mut balances), (class_id, asset_id, balance)| {
                // SBP-M1 review: use .map_or_else()
                let class_idx = if let Some(class_idx) =
                    class_ids.iter().position(|class| *class == *class_id)
                {
                    class_idx
                } else {
                    let class_idx = class_ids.len();
                    class_ids.push(*class_id);
                    class_idx
                };
                if asset_ids.len() <= class_idx {
                    // SBP-M1 review: use safe math
                    asset_ids.resize(class_idx + 1, vec![]);
                }
                // SBP-M1 review: indexing may panic, prefer .get()
                asset_ids[class_idx].push(*asset_id);
                if balances.len() <= class_idx {
                    // SBP-M1 review: use safe math
                    balances.resize(class_idx + 1, vec![]);
                }
                // SBP-M1 review: indexing may panic, prefer .get()
                balances[class_idx].push(*balance);
                (class_ids, asset_ids, balances)
            },
        );

        // SBP-M1 review: iteration should be bounded to complete within block limits.
        for (idx, class_id) in balances.0.iter().enumerate() {
            // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : BatchTransfer
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                // SBP-M1 review: needless borrows
                &bag,
                &bag,
                to,
                *class_id,
                // SBP-M1 review: indexing may panic, prefer .get()
                balances.1[idx].clone(),
                balances.2[idx].clone(),
            )?;
        }

        // SBP-M1 review: unnecessary clones
        Self::deposit_event(Event::Sweep {
            bag: bag.clone(),
            who: who.clone(),
            to: to.clone(),
        });

        // SBP-M1 review: return type not used anywhere
        Ok(balances)
    }
}
