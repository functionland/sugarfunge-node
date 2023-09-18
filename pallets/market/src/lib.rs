// SBP-M1 review: use cargo fmt
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{HasCompact, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    BoundedVec, PalletId,
};
use sp_arithmetic::ArithmeticError;
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Zero};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
use sugarfunge_asset::AssetInterface;
use types::*;

pub use pallet::*;

mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Identifier of the vault account
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        // Identifier of the markets created
        type MarketId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>
            + MaxEncodedLen;

        /// Identifier of a transaction
        type MarketRateId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>
            + MaxEncodedLen;

        /// Max number of Transactions per market_rate
        #[pallet::constant]
        type MaxTransactions: Get<u32>;

        /// Max metadata size
        #[pallet::constant]
        type MaxMetadata: Get<u32>;

        /// The interface to execute the sugarfunge-asset calls
        type Asset: AssetInterface<AccountId = Self::AccountId>;
    }

    pub type ClassId<T> = <<T as Config>::Asset as AssetInterface>::ClassId;

    pub type AssetId<T> = <<T as Config>::Asset as AssetInterface>::AssetId;

    pub type Balance<T> = <<T as Config>::Asset as AssetInterface>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn markets)]
    pub(super) type Markets<T: Config> =
        StorageMap<_, Blake2_128, T::MarketId, Market<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn market_rates)]
    pub(super) type MarketRates<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::MarketId>,
            NMapKey<Blake2_128Concat, T::MarketRateId>,
        ),
        Transactions<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn market_rates_metadata)]
    pub(super) type MarketRatesMetadata<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::MarketId>,
            NMapKey<Blake2_128Concat, T::MarketRateId>,
        ),
        BoundedVec<u8, <T as Config>::MaxMetadata>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Market created
        Created {
            market_id: T::MarketId,
            who: T::AccountId,
        },
        /// Transaction created
        RateCreated {
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            who: T::AccountId,
        },
        /// Liquidity was added to the market
        LiquidityAdded {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            class_ids: Vec<ClassId<T>>,
            asset_ids: Vec<Vec<AssetId<T>>>,
            amounts: Vec<Vec<Balance<T>>>,
        },
        /// Liquidity was removed from the market
        LiquidityRemoved {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            class_ids: Vec<ClassId<T>>,
            asset_ids: Vec<Vec<AssetId<T>>>,
            amounts: Vec<Vec<Balance<T>>>,
        },
        /// Deposit made to a market
        Deposit {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance<T>,
            balances: Vec<TransactionBalance<T::AccountId, ClassId<T>, AssetId<T>, Balance<T>>>,
            success: bool,
        },
        /// Withdraw from the market
        Exchanged {
            buyer: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance<T>,
            balances: Vec<TransactionBalance<T::AccountId, ClassId<T>, AssetId<T>, Balance<T>>>,
            success: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient amount to make the transaction
        InsufficientAmount,
        /// Insufficient liquidity to make the transaction
        InsufficientLiquidity,
        /// The given market doesn't exists, create the market first
        InvalidMarket,
        /// The market rate doesn't exists, create the market rate first
        InvalidMarketRate,
        /// The account is not the market rate owner
        InvalidMarketOwner,
        /// The market is already created
        MarketExists,
        /// The market rate is already created
        MarketRateExists,
        /// The value is not in the available prices to burn
        InvalidBurnPrice,
        /// The value is not in the available balances
        InvalidBurnBalance,
        /// The value is not in the available prices to transfer
        InvalidTransferPrice,
        /// The value is not in the available balances
        InvalidTransferBalance,
        /// The acccount given is the Owner or Vault account
        InvalidBuyer,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new market
        #[pallet::call_index(0)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create_market(origin: OriginFor<T>, market_id: T::MarketId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: how are markets removed? Can a deposit be taken to create a market to incentivize owner to remove market when no longer required?
            // SBP-M1 review: market rate removal also needs consideration
            Self::do_create_market(who, market_id)
        }

        /// Create a market rate for a given market
        #[pallet::call_index(1)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create_market_rate(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            transactions: Transactions<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_create_market_rate(who, market_id, market_rate_id, &transactions)
        }

        /// Deposit a given amount to an specific market rate
        #[pallet::call_index(2)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn deposit(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_deposit(&who, market_id, market_rate_id, amount)
        }

        // Exchange a given amount to withdraw from an specific market rate
        #[pallet::call_index(3)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn exchange_assets(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_exchange_assets(&who, market_id, market_rate_id, amount)
        }
    }
}

// SBP-M1 review: verify pub visibility on all functions. Wrap in traits to signal if they should be accessible to other pallets
impl<T: Config> Pallet<T> {
    // SBP-M1 review: probably doesnt need to be public, consider pub(crate), pub(super)...
    pub fn do_create_market(who: T::AccountId, market_id: T::MarketId) -> DispatchResult {
        ensure!(
            !Markets::<T>::contains_key(market_id),
            Error::<T>::MarketExists
        );

        // SBP-M1 review: vault account may require existential deposit (especially for native currency), could take from owner here at creation and return once market removed/closed
        // SBP-M1 review: may apply for each asset depending on implementation in asset pallet
        let vault: T::AccountId =
            <T as Config>::PalletId::get().into_sub_account_truncating(market_id);

        Markets::<T>::insert(
            market_id,
            Market {
                owner: who.clone(),
                vault,
            },
        );

        Self::deposit_event(Event::Created { market_id, who });

        Ok(())
    }

    pub fn do_create_market_rate(
        who: T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        transactions: &Transactions<T>,
    ) -> DispatchResult {
        // SBP-M1 review: can market not be auto-created if it doesnt exist? Currently requires one transaction to create market and then another to submit rates
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            !MarketRates::<T>::contains_key((market_id, market_rate_id)),
            Error::<T>::MarketRateExists
        );

        // Ensure rates are valid
        for asset_rate in transactions.iter() {
            // SBP-M1 review: duplicate code, use asset_rate.action.get_amount()
            let amount = match asset_rate.action {
                // SBP-M1 review: merge arms > RateAction::Burn(amount) | RateAction::Mint(amount) ...
                RateAction::Burn(amount) => amount,
                RateAction::Mint(amount) => amount,
                RateAction::Transfer(amount) => amount,
                // SBP-M1 review: wildcard match will also match any future variants, prefer being explicit
                _ => 0.into(),
            };
        }

        MarketRates::<T>::insert((market_id, market_rate_id), transactions);

        Self::deposit_event(Event::RateCreated {
            market_id,
            market_rate_id,
            who,
        });

        Ok(())
    }

    // SBP-M1 review: too many lines, refactor
    // SBP-M1 review: who uses this? Appears to be no usage in project. Wrap in a well documented trait so other pallets can call via trait
    pub fn add_liquidity(
        who: T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        // SBP-M1 review: vectors should be bounded
        // SBP-M1 review: use Vec<(ClassId<T>, Vec<AssetId<T>>, Vec<Balance<T>>)> to avoid needing to check sizes of each vector match
        class_ids: Vec<ClassId<T>>,
        asset_ids: Vec<Vec<AssetId<T>>>,
        amounts: Vec<Vec<Balance<T>>>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used, used .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        // SBP-M1 review: move above market rate check to prevent unnecessary read if caller is not market owner
        ensure!(who == market.owner, Error::<T>::InvalidMarketOwner);

        for (idx, class_id) in class_ids.iter().enumerate() {
            // SBP-M1 review: should use trait, defined as associated type on Config trait of pallet (loose coupling)
            T::Asset::batch_transfer_from(
                market.owner,
                market.owner,
                market.vault,
                *class_id,
                // SBP-M1 review: indexing may panic, consider .get() instead
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::LiquidityAdded {
            who,
            market_id,
            market_rate_id,
            class_ids,
            asset_ids,
            amounts,
        });

        Ok(())
    }

    // SBP-M1 review: too many lines, refactor
    // SBP-M1 review: who uses this? Appears to be no usage in project. Wrap in a well documented trait so other pallets can call via trait
    pub fn remove_liquidity(
        who: T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        // SBP-M1 review: vectors should be bounded
        // SBP-M1 review: use Vec<(ClassId<T>, Vec<AssetId<T>>, Vec<Balance<T>>)> to avoid needing to check sizes of each vector match
        class_ids: Vec<ClassId<T>>,
        asset_ids: Vec<Vec<AssetId<T>>>,
        amounts: Vec<Vec<Balance<T>>>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(who == market.owner, Error::<T>::InvalidMarketOwner);
        // SBP-M1 review: value not used, used .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        for (idx, class_id) in class_ids.iter().enumerate() {
            // SBP-M1 review: should use trait, defined as associated type on Config trait of pallet (loose coupling)
            T::Asset::batch_transfer_from(
                market.owner,
                market.vault,
                market.owner,
                *class_id,
                // SBP-M1 review: indexing may panic, consider .get() instead
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::LiquidityRemoved {
            who: who.clone(),
            market_id,
            market_rate_id,
            class_ids,
            asset_ids,
            amounts,
        });
        Ok(())
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_quote_deposit(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance<T>,
        // SBP-M1 review: consider changing to Result<RateBalances<T>, DispatchError> and simply returning error if !can_do_deposit
    ) -> Result<(bool, TransactionBalances<T>), DispatchError> {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);
        let rates = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        let mut deposit_balances = BTreeMap::new();

        // SBP-M1 review: consider grouping each 'phase' of algorithm into blocks: aggregate, compute burns, compute transfers
        // RateAction::Transfer|Burn - Aggregate transferable prices and balances

        let mut balances: TransactionBalancesWIthRateAccount<T> = BTreeMap::new();

        let mut prices: TransactionBalancesWIthRateAccount<T> = BTreeMap::new();

        // SBP-M1 review: why use a signed integer?
        let total_amount: u128 = amount.try_into().map_err(|_| ArithmeticError::Overflow)?;

        // SBP-M1 review: filter and collect in single statement > rates.into_iter().filter(..).collect();
        let asset_rates = rates.into_iter().filter(|asset_rate| {
            // SBP-M1 review: refactor into .quotable() function on enum to simplify to asset_rate.from == RateAccount::Market && asset_rate.action.quotable()
            // SBP-M1 review: alternatively use matches!
            let quotable = match asset_rate.action {
                RateAction::Transfer(_) | RateAction::Burn(_) => true,
                // SBP-M1 review: wildcard will match future added variants
                _ => false,
            };
            asset_rate.from == RateAccount::Market && quotable
        });

        let asset_rates: Vec<Transaction<T::AccountId, ClassId<T>, AssetId<T>, Balance<T>>> =
            asset_rates.collect();

        for asset_rate in &asset_rates {
            let balance: u128 =
                T::Asset::balance_of(market.owner, asset_rate.class_id, asset_rate.asset_id)
                    .try_into()
                    .map_err(|_| ArithmeticError::Overflow)?;
            balances.insert(
                (
                    asset_rate.from.clone(),
                    asset_rate.class_id,
                    asset_rate.asset_id,
                ),
                balance.into(),
            );
            let mut amount_value: u128 = asset_rate.action.get_amount().into();
            amount_value = amount_value
                .checked_mul(total_amount)
                .ok_or(ArithmeticError::Overflow)?;
            // SBP-M1 review: duplicate code, refactor into reusable function
            // SBP-M1 review: consider BTreeMap.entry() api
            if let Some(price) = prices.get_mut(&(
                asset_rate.from.clone(),
                asset_rate.class_id,
                asset_rate.asset_id,
            )) {
                let mut price_value: u128 = (*price).into();
                price_value = price_value
                    .checked_add(amount_value)
                    .ok_or(ArithmeticError::Overflow)?;
                *price = price_value.into()
            } else {
                prices.insert(
                    (
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ),
                    amount_value.into(),
                );
            }
        }

        // RateAction::Burn - Compute total burns

        for asset_rate in &asset_rates {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Burn(_) = asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                // SBP-M1 review: consider .entry() api
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnBalance)?;
                let mut balance_value: u128 = (*balance).into();
                balance_value = balance_value
                    .checked_sub((*price).into())
                    .ok_or(ArithmeticError::Overflow)?;
                if balance_value < 0 {
                    deposit_balances.insert(asset_rate.clone(), balance_value.into());
                } else {
                    deposit_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Transfer - Compute total transfers

        for asset_rate in &asset_rates {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Transfer(_) = asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferPrice)?;
                // SBP-M1 review: consider .entry() api
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferBalance)?;
                let mut balance_value: u128 = (*balance).into();
                balance_value = balance_value
                    .checked_sub((*price).into())
                    .ok_or(ArithmeticError::Overflow)?;
                if balance_value < 0 {
                    deposit_balances.insert(asset_rate.clone(), balance_value.into());
                } else {
                    deposit_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        let mut can_do_deposit = true;

        // SBP-M1 review: use deposit_balances.values()
        for (_, deposit_balance) in &deposit_balances {
            if (*deposit_balance).into() < 0 {
                // SBP-M1 review: consider return Err(Error::CannotDeposit);
                can_do_deposit = false;
                break;
            }
        }

        // SBP-M1 review: consider Ok(deposit_balances)
        Ok((can_do_deposit, deposit_balances))
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_deposit(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance<T>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used therefore inefficient, use .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        let (can_do_deposit, deposit_balances) =
            Self::do_quote_deposit(who, market_id, market_rate_id, amount)?;

        // SBP-M1 review: consider returning an error rather than signalling that caller can not deposit via success attribute of Deposit event
        if can_do_deposit {
            // SBP-M1 review: each transfer request results in separate state change, consider grouping by class/asset if applicable
            for (asset_rate, amount) in &deposit_balances {
                let amount: u128 = (*amount)
                    .try_into()
                    .map_err(|_| ArithmeticError::Overflow)?;
                T::Asset::transfer_from(
                    market.owner,
                    market.owner,
                    market.vault,
                    asset_rate.class_id,
                    asset_rate.asset_id,
                    amount.into(),
                )?
            }
        }

        let balances = deposit_balances
            .iter()
            .map(|(rate, balance)| TransactionBalance {
                transaction: rate.clone(),
                balance: *balance,
            })
            .collect();

        // SBP-M1 review: consider emitting an error if the deposit could not be performed rather than success: can_do_deposit
        Self::deposit_event(Event::Deposit {
            // SBP-M1 review: unnecessary clone()
            who: who.clone(),
            market_id,
            market_rate_id,
            amount,
            balances,
            success: can_do_deposit,
        });

        Ok(())
    }

    pub fn get_vault(market_id: T::MarketId) -> Option<T::AccountId> {
        Markets::<T>::get(market_id).map(|market| market.vault)
    }

    pub fn balance(
        market: &Market<T::AccountId>,
        class_id: ClassId<T>,
        asset_id: AssetId<T>,
    ) -> Balance<T> {
        T::Asset::balance_of(market.vault, class_id, asset_id)
    }

    /// Pricing function used for converting between outgoing asset to incomming asset.
    ///
    /// - `amount_out`: Amount of outgoing asset being bought.
    /// - `reserve_in`: Amount of incomming asset in reserves.
    /// - `reserve_out`: Amount of outgoing asset in reserves.
    /// Return the price Amount of incoming asset to send to vault.

    pub fn get_buy_price(
        amount_out: Balance<T>,
        reserve_in: Balance<T>,
        reserve_out: Balance<T>,
    ) -> Result<Balance<T>, DispatchError> {
        ensure!(
            reserve_in > Zero::zero() && reserve_out > Zero::zero(),
            Error::<T>::InsufficientLiquidity
        );

        let numerator: U256 = U256::from(reserve_in)
            .saturating_mul(U256::from(amount_out))
            .saturating_mul(U256::from(1000u128));
        let denominator: U256 = (U256::from(reserve_out).saturating_sub(U256::from(amount_out)))
            .saturating_mul(U256::from(995u128));

        let amount_in = numerator
            .checked_div(denominator)
            .and_then(|r| r.checked_add(U256::one())) // add 1 to correct possible losses caused by remainder discard
            .and_then(|n| TryInto::<Balance<T>>::try_into(n).ok())
            .unwrap_or_else(Zero::zero);

        Ok(amount_in)
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_quote_exchange(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance<T>,
        // SBP-M1 review: consider changing to Result<RateBalances<T>, DispatchError> and simply returning error if !can_do_exchange
    ) -> Result<(bool, TransactionBalances<T>), DispatchError> {
        ensure!(amount.into() > 0, Error::<T>::InsufficientAmount);

        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let rates = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        let mut exchange_balances = BTreeMap::new();

        let mut can_do_exchange = true;

        // SBP-M1 review: consider grouping each 'phase' of algorithm into blocks: prove, aggregate, burn, transfer, mint
        // RateAction::Has - Prove parties possess non-transferable assets

        for asset_rate in rates.iter() {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Has(op, amount) = asset_rate.action {
                // SBP-M1 review: consider .target_account() helper function on enum
                let target_account = match &asset_rate.from {
                    RateAccount::Account(account) => account,
                    RateAccount::Buyer => buyer,
                    RateAccount::Market => &market.vault,
                };
                let balance: u128 = T::Asset::balance_of(
                    target_account.clone(),
                    asset_rate.class_id,
                    asset_rate.asset_id,
                )
                .try_into()
                .map_err(|_| ArithmeticError::Overflow)?;
                // SBP-M1 review: refactor into helper function - e.g. op.evaluate(balance, amount)
                let amount = match op {
                    AmountOp::Equal => {
                        if balance == amount.into() {
                            amount.into()
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            balance - amount.into()
                        }
                    }
                    AmountOp::GreaterEqualThan => {
                        if balance >= amount.into() {
                            amount.into()
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            balance - amount.into()
                        }
                    }
                    AmountOp::GreaterThan => {
                        if balance > amount.into() {
                            amount.into()
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            balance - amount.into()
                        }
                    }
                    AmountOp::LessEqualThan => {
                        if balance <= amount.into() {
                            amount.into()
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            amount.into() - balance
                        }
                    }
                    AmountOp::LessThan => {
                        if balance < amount.into() {
                            amount.into()
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            amount.into() - balance
                        }
                    }
                };
                exchange_balances.insert(asset_rate.clone(), amount.into());
            }
        }

        // RateAction::Transfer|Burn - Aggregate transferable prices and balances

        let mut balances: TransactionBalancesWIthRateAccount<T> = BTreeMap::new();

        let mut prices: TransactionBalancesWIthRateAccount<T> = BTreeMap::new();

        let total_amount: u128 = amount.try_into().map_err(|_| ArithmeticError::Overflow)?;

        for asset_rate in rates.iter() {
            let balance = match &asset_rate.action {
                RateAction::Transfer(_) | RateAction::Burn(_) => {
                    // SBP-M1 review: refactor into .target_account() helper function on enum
                    let target_account = match &asset_rate.from {
                        RateAccount::Account(account) => account,
                        RateAccount::Buyer => buyer,
                        RateAccount::Market => &market.vault,
                    };
                    let balance: u128 = T::Asset::balance_of(
                        target_account.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    )
                    .try_into()
                    .map_err(|_| ArithmeticError::Overflow)?;
                    Some(balance)
                }
                // SBP-M1 review: prefer explict matches, wildcard will match any future added variants
                _ => None,
            };
            if let Some(balance) = balance {
                balances.insert(
                    (
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ),
                    balance.into(),
                );
            }
            let mut amount_value: u128 = asset_rate.action.get_amount().into();
            amount_value = amount_value
                .checked_mul(total_amount)
                .ok_or(ArithmeticError::Overflow)?;
            // SBP-M1 review: duplicate code, refactor into reusable function
            // SBP-M1 review: consider BTreeMap.entry() api
            if let Some(price) = prices.get_mut(&(
                asset_rate.from.clone(),
                asset_rate.class_id,
                asset_rate.asset_id,
            )) {
                let mut price_value: u128 = (*price).into();
                price_value = price_value
                    .checked_add(amount_value)
                    .ok_or(ArithmeticError::Overflow)?;
                *price = price_value.into()
            } else {
                prices.insert(
                    (
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ),
                    amount_value.into(),
                );
            }
        }

        // RateAction::Burn - Compute total burns

        for asset_rate in rates.iter() {
            // SBP-M1 review: use let-else continue to reduce nesting
            if let RateAction::Burn(_) = &asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnBalance)?;
                let mut balance_value: u128 = (*balance).into();
                balance_value = balance_value
                    .checked_sub((*price).into())
                    .ok_or(ArithmeticError::Overflow)?;
                if balance_value < 0 {
                    exchange_balances.insert(asset_rate.clone(), balance_value.into());
                } else {
                    exchange_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Transfer - Compute total transfers

        for asset_rate in rates.iter() {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Transfer(_) = &asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferPrice)?;
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferBalance)?;
                let mut balance_value: u128 = (*balance).into();
                balance_value = balance_value
                    .checked_sub((*price).into())
                    .ok_or(ArithmeticError::Overflow)?;
                if balance_value < 0 {
                    exchange_balances.insert(asset_rate.clone(), balance_value.into());
                } else {
                    exchange_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Mint - Compute total mints

        for asset_rate in rates.iter() {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Mint(_) = &asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                exchange_balances.insert(asset_rate.clone(), *price);
            }
        }

        // SBP-M1 review: consider Ok(exchange_balances), returning error early once !can_do_exchange
        Ok((can_do_exchange, exchange_balances))
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_exchange_assets(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance<T>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used therefore inefficient, use .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        // SBP-M1 review: move before market rate check to avoid unnecessary read if buyer is owner/vault
        ensure!(*buyer != market.owner, Error::<T>::InvalidBuyer);
        ensure!(*buyer != market.vault, Error::<T>::InvalidBuyer);

        let (can_do_exchange, exchange_balances) =
            Self::do_quote_exchange(buyer, market_id, market_rate_id, amount)?;

        // SBP-M1 review: consider returning an error rather than signalling that caller can not exchange via success attribute of Exchanged event
        if can_do_exchange {
            for (asset_rate, amount) in &exchange_balances {
                let amount: u128 = (*amount)
                    .try_into()
                    .map_err(|_| ArithmeticError::Overflow)?;
                // SBP-M1 review: use .from() helper method on enum
                let from = match &asset_rate.from {
                    RateAccount::Account(account) => account,
                    RateAccount::Buyer => buyer,
                    RateAccount::Market => &market.vault,
                };
                // SBP-M1 review: use .to() helper method on enum
                let to = match &asset_rate.to {
                    RateAccount::Account(account) => account,
                    RateAccount::Buyer => buyer,
                    RateAccount::Market => &market.vault,
                };
                match asset_rate.action {
                    RateAction::Transfer(_) => T::Asset::transfer_from(
                        market.owner,
                        from.clone(),
                        to.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                        amount.into(),
                    )?,
                    RateAction::Burn(_) => T::Asset::burn(
                        market.owner,
                        from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                        amount.into(),
                    )?,
                    RateAction::Mint(_) => T::Asset::mint(
                        market.owner,
                        to.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                        amount.into(),
                    )?,
                    // SBP-M1 review: use actual enum variants, wildcard will match future added variants
                    _ => (),
                }
            }
        }

        let balances = exchange_balances
            .iter()
            .map(|(rate, balance)| TransactionBalance {
                transaction: rate.clone(),
                balance: *balance,
            })
            .collect();

        // SBP-M1 review: consider emitting an error if the exchange could not be performed rather than success: can_do_exchange
        Self::deposit_event(Event::Exchanged {
            buyer: buyer.clone(),
            market_id,
            market_rate_id,
            amount,
            balances,
            success: can_do_exchange,
        });
        Ok(())
    }
}
