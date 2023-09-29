use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
// SBP-M1 review: should amount not be a u128 rather than signed?
use crate::pallet::{AssetId, Balance, ClassId};

use crate::Config;

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
/// Enum to identify the different relations between amounts
pub enum AmountOp {
    Equal,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum AMM {
    Constant,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
/// Enum to identify the different Actions possibles in a market
pub enum RateAction<ClassId, AssetId, Balance> {
    Transfer(Balance),
    MarketTransfer(AMM, ClassId, AssetId),
    Mint(Balance),
    Burn(Balance),
    Has(AmountOp, Balance),
}

impl<ClassId, AssetId, Balance: From<u128> + Into<u128>> RateAction<ClassId, AssetId, Balance> {
    /// Amount of currency of the respective transaction
    pub fn get_amount(self) -> Balance {
        match self {
            Self::Burn(amount) | Self::Mint(amount) | Self::Transfer(amount) => amount,
            Self::MarketTransfer(..) | Self::Has(..) => 0.into(),
        }
    }
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
/// Enum to identify the different types of account in a market
pub enum RateAccount<AccountId> {
    Market,
    Account(AccountId),
    Buyer,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
/// Represents the transaction parameters
pub struct Transaction<AccountId, ClassId, AssetId, Balance> {
    pub class_id: ClassId,
    pub asset_id: AssetId,
    pub action: RateAction<ClassId, AssetId, Balance>,
    pub from: RateAccount<AccountId>,
    pub to: RateAccount<AccountId>,
}

/// Represents a Map that correlates transactions with their respective amount handle
pub type TransactionBalances<T> = BTreeMap<
    Transaction<<T as frame_system::Config>::AccountId, ClassId<T>, AssetId<T>, Balance<T>>,
    Balance<T>,
>;

/// Represents a Map that correlates transactions parameters with their respective amount handle
pub type TransactionBalancesWIthRateAccount<T> = BTreeMap<
    (
        RateAccount<<T as frame_system::Config>::AccountId>,
        ClassId<T>,
        AssetId<T>,
    ),
    Balance<T>,
>;

/// Represents a vector of transactions given the config types
pub type Transactions<T> = BoundedVec<
    Transaction<<T as frame_system::Config>::AccountId, ClassId<T>, AssetId<T>, Balance<T>>,
    <T as Config>::MaxTransactions,
>;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
/// Represents the transaction with the respetive balance
pub struct TransactionBalance<AccountId, ClassId, AssetId, Balance> {
    pub transaction: Transaction<AccountId, ClassId, AssetId, Balance>,
    pub balance: Balance,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Represents the accounts related to a market
pub struct Market<AccountId> {
    /// The owner of the market
    pub owner: AccountId,
    /// The fund account of the market
    pub vault: AccountId,
}
