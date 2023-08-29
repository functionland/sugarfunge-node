use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
// SBP-M1 review: should amount not be a u128 rather than signed?
use sugarfunge_primitives::Amount;

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
pub enum RateAction<ClassId, AssetId> {
    Transfer(Amount),
    MarketTransfer(AMM, ClassId, AssetId),
    Mint(Amount),
    Burn(Amount),
    Has(AmountOp, Amount),
}

impl<ClassId, AssetId> RateAction<ClassId, AssetId> {
    /// Amount of currency of the respective transaction
    pub fn get_amount(&self) -> Amount {
        match *self {
            Self::Burn(amount) | Self::Mint(amount) | Self::Transfer(amount) => amount,
            Self::MarketTransfer(..) | Self::Has(..) => 0,
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
pub struct Transaction<AccountId, ClassId, AssetId> {
    pub class_id: ClassId,
    pub asset_id: AssetId,
    pub action: RateAction<ClassId, AssetId>,
    pub from: RateAccount<AccountId>,
    pub to: RateAccount<AccountId>,
}

/// Represents a Map that correlates transactions with their respective amount handle
pub type TransactionBalances<T> = BTreeMap<
    Transaction<
        <T as frame_system::Config>::AccountId,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    >,
    Amount,
>;

/// Represents a Map that correlates transactions parameters with their respective amount handle
pub type TransactionBalancesWIthRateAccount<T> = BTreeMap<
    (
        RateAccount<<T as frame_system::Config>::AccountId>,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    ),
    Amount,
>;

/// Represents a vector of transactions given the config types
pub type Transactions<T> = BoundedVec<
    Transaction<
        <T as frame_system::Config>::AccountId,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    >,
    <T as Config>::MaxTransactions,
>;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
/// Represents the transaction with the respetive balance
pub struct TransactionBalance<AccountId, ClassId, AssetId> {
    pub transaction: Transaction<AccountId, ClassId, AssetId>,
    pub balance: Amount,
}
