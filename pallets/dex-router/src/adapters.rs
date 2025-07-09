//! AMM adapter implementations for the DEX router.

use crate::traits::{FeeCollector, AMM};
use alloc::{boxed::Box, vec};
use core::marker::PhantomData;
use frame::prelude::*;
use polkadot_sdk::{pallet_asset_conversion, pallet_balances};

/// XYK AMM adapter that wraps pallet-asset-conversion.
pub struct XYKAdapter<T> {
  _phantom: PhantomData<T>,
}

impl<T> Default for XYKAdapter<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> XYKAdapter<T> {
  pub fn new() -> Self {
    Self {
      _phantom: PhantomData,
    }
  }
}

impl<T, AssetKind, Balance, AccountId> AMM<AssetKind, Balance, AccountId> for XYKAdapter<T>
where
  T: pallet_asset_conversion::Config<AssetKind = AssetKind, Balance = Balance, AccountId = AccountId>
    + frame_system::Config<AccountId = AccountId>,
  AssetKind: Clone + Copy,
  Balance: Zero + From<u32> + Copy + PartialOrd,
  AccountId: Clone,
{
  type Error = DispatchError;

  fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
    // Check if a pool exists for this pair by trying to get a quote
    let test_amount = Balance::from(1u32);
    pallet_asset_conversion::Pallet::<T>::quote_price_exact_tokens_for_tokens(
      *asset_in,
      *asset_out,
      test_amount,
      true,
    )
    .is_some()
  }

  fn quote_price(
    &self,
    asset_in: &AssetKind,
    asset_out: &AssetKind,
    amount_in: Balance,
  ) -> Option<Balance> {
    pallet_asset_conversion::Pallet::<T>::quote_price_exact_tokens_for_tokens(
      *asset_in, *asset_out, amount_in, true, // include fees
    )
  }

  fn execute_swap(
    &self,
    who: &AccountId,
    asset_in: AssetKind,
    asset_out: AssetKind,
    amount_in: Balance,
    min_amount_out: Balance,
  ) -> Result<Balance, Self::Error> {
    // Create path for the swap - pallet-asset-conversion expects Vec<Box<AssetKind>>
    let path = vec![Box::new(asset_in), Box::new(asset_out)];

    // Get quote before swap to return actual amount out
    let expected_amount_out =
      pallet_asset_conversion::Pallet::<T>::quote_price_exact_tokens_for_tokens(
        asset_in, asset_out, amount_in, true,
      )
      .ok_or(DispatchError::Other("No liquidity available"))?;

    // Check if quote meets minimum requirements
    if expected_amount_out < min_amount_out {
      return Err(DispatchError::Other("Insufficient output amount"));
    }

    // Execute the actual swap using pallet-asset-conversion
    pallet_asset_conversion::Pallet::<T>::swap_exact_tokens_for_tokens(
      frame_system::RawOrigin::Signed(who.clone()).into(),
      path,
      amount_in,
      min_amount_out,
      who.clone(),
      false, // keep_alive
    )?;

    Ok(expected_amount_out)
  }

  fn name(&self) -> &'static str {
    "XYK"
  }
}

/// Default fee collector implementation.
pub struct DefaultFeeCollector<T, AccountId> {
  fee_collector: AccountId,
  _phantom: PhantomData<T>,
}

impl<T, AccountId> DefaultFeeCollector<T, AccountId> {
  pub fn new(fee_collector: AccountId) -> Self {
    Self {
      fee_collector,
      _phantom: PhantomData,
    }
  }
}

impl<T, AssetKind, Balance, AccountId> FeeCollector<AssetKind, Balance, AccountId>
  for DefaultFeeCollector<T, AccountId>
where
  T: pallet_balances::Config<Balance = Balance, AccountId = AccountId>
    + frame_system::Config<AccountId = AccountId>,
  Balance: Zero,
  AccountId: Clone,
{
  fn collect_fee(&self, from: &AccountId, _asset: &AssetKind, amount: Balance) -> DispatchResult {
    if amount.is_zero() {
      return Ok(());
    }

    // For now, assume we're dealing with native tokens
    // Use transfer_allow_death to avoid NotExpendable errors
    pallet_balances::Pallet::<T>::transfer_allow_death(
      frame_system::RawOrigin::Signed(from.clone()).into(),
      T::Lookup::unlookup(self.fee_collector.clone()),
      amount,
    )?;

    Ok(())
  }
}
