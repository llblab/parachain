//! Core traits for DEX router functionality

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame::prelude::*;
use scale_info::TypeInfo;

/// Main trait for Automated Market Makers (AMMs)
pub trait AMM<AssetKind, Balance, AccountId> {
  type Error: Into<DispatchError>;

  /// Check if this AMM can handle the given asset pair
  fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool;

  /// Get a price quote for swapping tokens
  /// Returns the amount of `asset_out` tokens that would be received for `amount_in` of `asset_in`
  fn quote_price(
    &self,
    asset_in: &AssetKind,
    asset_out: &AssetKind,
    amount_in: Balance,
  ) -> Option<Balance>;

  /// Execute a token swap
  /// Note: This receives `amount_in` after router fee has been deducted
  fn execute_swap(
    &self,
    who: &AccountId,
    asset_in: AssetKind,
    asset_out: AssetKind,
    amount_in: Balance,
    min_amount_out: Balance,
  ) -> Result<Balance, Self::Error>;

  /// Get the name of this AMM for logging purposes
  fn name(&self) -> &'static str;
}

/// AMM types supported by the router
#[derive(
  Clone, Copy, Debug, Decode, DecodeWithMemTracking, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo,
)]
pub enum AMMType {
  /// Uniswap V2-style constant product AMM
  XYK,
  /// Token Bonding Curve AMM
  TBC,
}

/// Trait for collecting router fees
pub trait FeeCollector<AssetKind, Balance, AccountId> {
  /// Collect a fee from the specified account
  fn collect_fee(&self, from: &AccountId, asset: &AssetKind, amount: Balance) -> DispatchResult;
}

/// Trait for routing strategies
pub trait RoutingStrategy<AssetKind, Balance> {
  /// Select the best AMM from available quotes
  fn select_best_amm(
    &self,
    quotes: Vec<(AMMType, Balance)>,
    asset_in: &AssetKind,
    asset_out: &AssetKind,
  ) -> Option<AMMType>;
}

/// Simple best-price routing strategy
pub struct BestPriceStrategy;

impl<AssetKind, Balance> RoutingStrategy<AssetKind, Balance> for BestPriceStrategy
where
  Balance: Ord + Copy,
{
  fn select_best_amm(
    &self,
    quotes: Vec<(AMMType, Balance)>,
    _asset_in: &AssetKind,
    _asset_out: &AssetKind,
  ) -> Option<AMMType> {
    quotes
      .into_iter()
      .max_by_key(|(_, quote)| *quote)
      .map(|(amm_type, _)| amm_type)
  }
}
