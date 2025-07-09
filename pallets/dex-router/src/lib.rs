//! # DEX Router Pallet
//!
//! A trait-based DEX router that aggregates multiple AMMs with built-in router fees.
//!
//! ## Overview
//!
//! This pallet provides a unified interface for interacting with multiple Automated Market Makers (AMMs)
//! while automatically selecting the best price and collecting router fees.
//!
//! ## Features
//!
//! - **Multi-AMM Support**: Supports XYK (pallet-asset-conversion) and future AMMs
//! - **Automatic Best Price Selection**: Compares quotes from all available AMMs
//! - **Built-in Router Fees**: Collects fees on all transactions
//! - **Trait-based Architecture**: Easy to extend with new AMMs

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame::prelude::*;
use polkadot_sdk::{pallet_asset_conversion, pallet_balances};

pub mod traits;
pub use traits::*;

pub mod adapters;
pub use adapters::*;

pub use pallet::*;

#[cfg(test)]
pub mod tests;

#[frame::pallet(dev_mode)]
pub mod pallet {
  use super::*;

  #[pallet::config]
  pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// The balance type used by the pallet.
    type Balance: Parameter
      + Member
      + Copy
      + Default
      + Zero
      + AtLeast32BitUnsigned
      + Saturating
      + CheckedSub
      + PartialOrd;

    /// The asset kind type used by the pallet.
    type AssetKind: Parameter + Member + Copy;

    /// Router fee percentage for buyback mechanism (e.g., 20 = 0.2%).
    /// This fee is used for buying back and burning the base network asset.
    #[pallet::constant]
    type RouterFee: Get<Permill>;

    /// Account that receives router fees for buyback and burning.
    /// This account should be configured to handle the buyback mechanism.
    #[pallet::constant]
    type RouterFeeCollector: Get<Self::AccountId>;

    /// Weight information for extrinsics.
    type WeightInfo: WeightInfo;

    /// Asset Conversion pallet for XYK AMM integration.
    type AssetConversion: pallet_asset_conversion::Config<
      AssetKind = Self::AssetKind,
      Balance = Self::Balance,
      AccountId = Self::AccountId,
    >;

    /// Balances pallet for fee collection.
    type Balances: pallet_balances::Config<Balance = Self::Balance, AccountId = Self::AccountId>;
  }

  #[pallet::pallet]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn something)]
  /// A storage item for the pallet.
  pub type Something<T> = StorageValue<_, u32, ValueQuery>;

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  impl<T: Config> Pallet<T> {
    /// Get the XYK adapter for Asset Conversion integration.
    fn get_xyk_adapter() -> XYKAdapter<T::AssetConversion> {
      XYKAdapter::new()
    }

    /// Get the default fee collector.
    fn get_fee_collector() -> DefaultFeeCollector<T::Balances, T::AccountId> {
      DefaultFeeCollector::new(T::RouterFeeCollector::get())
    }

    /// Get quote from available AMMs for the given asset pair.
    fn get_best_quote(
      asset_in: &T::AssetKind,
      asset_out: &T::AssetKind,
      amount_in: T::Balance,
    ) -> Option<T::Balance> {
      let xyk_adapter = Self::get_xyk_adapter();

      if xyk_adapter.can_handle_pair(asset_in, asset_out) {
        xyk_adapter.quote_price(asset_in, asset_out, amount_in)
      } else {
        None
      }
    }

    /// Execute swap using the best available AMM.
    fn execute_best_swap(
      who: &T::AccountId,
      asset_in: T::AssetKind,
      asset_out: T::AssetKind,
      amount_in: T::Balance,
      min_amount_out: T::Balance,
    ) -> Result<T::Balance, DispatchError> {
      let xyk_adapter = Self::get_xyk_adapter();

      if xyk_adapter.can_handle_pair(&asset_in, &asset_out) {
        xyk_adapter.execute_swap(who, asset_in, asset_out, amount_in, min_amount_out)
      } else {
        Err(Error::<T>::NoCompatibleAMM.into())
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// A swap was executed through the router with dual fee structure.
    SwapExecuted {
      /// The account that initiated the swap.
      who: T::AccountId,
      /// The input asset.
      asset_in: T::AssetKind,
      /// The output asset.
      asset_out: T::AssetKind,
      /// The amount of input asset (total user payment).
      amount_in: T::Balance,
      /// The amount of output asset received.
      amount_out: T::Balance,
      /// The router fee collected (0.2% for buyback mechanism).
      router_fee: T::Balance,
      /// The AMM that was used.
      amm_used: AMMType,
    },
  }

  #[pallet::error]
  pub enum Error<T> {
    /// No compatible AMM found for the given asset pair.
    NoCompatibleAMM,
    /// No liquidity available for the swap.
    NoLiquidityAvailable,
    /// Invalid swap path provided.
    InvalidPath,
    /// Fee calculation failed.
    FeeCalculationFailed,
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Execute a token swap through the best available AMM.
    #[pallet::call_index(0)]
    #[pallet::weight(T::WeightInfo::swap_exact_tokens_for_tokens())]
    pub fn swap_exact_tokens_for_tokens(
      origin: OriginFor<T>,
      path: BoundedVec<T::AssetKind, ConstU32<5>>,
      amount_in: T::Balance,
      amount_out_min: T::Balance,
      _send_to: T::AccountId,
      _keep_alive: bool,
    ) -> DispatchResult {
      let who = ensure_signed(origin)?;

      // Currently only support direct swaps (path length = 2)
      ensure!(path.len() == 2, Error::<T>::InvalidPath);

      let asset_in = path[0];
      let asset_out = path[1];

      // DUAL FEE STRUCTURE IMPLEMENTATION (according to tokenomics):
      //
      // 1. Router Fee (0.2%): Goes to buyback and burning of base network asset
      //    - Collected by DEX Router before passing to AssetConversion
      //    - Used for token buyback mechanism to support token price
      //
      // 2. XYK Pool Fee (0.3%): Goes to liquidity providers
      //    - Handled internally by AssetConversion pallet
      //    - Increases pool liquidity over time
      //
      // 3. Total User Cost: 0.5% (0.2% + 0.3%)
      //    - User pays full amount_in
      //    - Router takes 0.2% for buyback
      //    - Remaining amount goes to AssetConversion (which takes its own 0.3%)

      // Calculate router fee (0.2% for buyback mechanism)
      let router_fee = T::RouterFee::get().mul_floor(amount_in);
      let amount_after_router_fee = amount_in
        .checked_sub(&router_fee)
        .ok_or(Error::<T>::FeeCalculationFailed)?;

      // Get quote from available AMMs (using amount after router fee)
      // AssetConversion will apply its own 0.3% fee on top of this amount
      let quote = Self::get_best_quote(&asset_in, &asset_out, amount_after_router_fee)
        .ok_or(Error::<T>::NoLiquidityAvailable)?;

      // Ensure the quote meets minimum requirements
      ensure!(quote >= amount_out_min, Error::<T>::NoLiquidityAvailable);

      // Execute the swap through the best available AMM
      // AssetConversion will deduct its 0.3% fee from amount_after_router_fee
      let actual_amount_out = Self::execute_best_swap(
        &who,
        asset_in,
        asset_out,
        amount_after_router_fee,
        amount_out_min,
      )
      .map_err(|_| Error::<T>::NoLiquidityAvailable)?;

      // Collect router fees for buyback and burning mechanism (0.2%)
      // This fee is sent to the configured fee collector account
      if !router_fee.is_zero() {
        let fee_collector = Self::get_fee_collector();
        fee_collector
          .collect_fee(&who, &asset_in, router_fee)
          .map_err(|_| Error::<T>::FeeCalculationFailed)?;
      }

      // FEE DISTRIBUTION SUMMARY:
      // - User pays: amount_in (100%)
      // - Router takes: router_fee (0.2%) → buyback mechanism
      // - AssetConversion receives: amount_after_router_fee (99.8%)
      // - AssetConversion takes: 0.3% of amount_after_router_fee → liquidity providers
      // - Actual swap amount: ~99.5% of original amount_in
      // - Total effective fee: ~0.5% of amount_in

      // Emit event
      Self::deposit_event(Event::SwapExecuted {
        who,
        asset_in,
        asset_out,
        amount_in,
        amount_out: actual_amount_out,
        router_fee,
        amm_used: AMMType::XYK,
      });

      Ok(())
    }
  }
}

/// Weight information for pallet extrinsics.
pub trait WeightInfo {
  fn swap_exact_tokens_for_tokens() -> Weight;
}

/// Default weights for the pallet
pub mod weights {
  use super::*;

  pub trait WeightInfo {
    fn swap_exact_tokens_for_tokens() -> Weight;
  }

  pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
  impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn swap_exact_tokens_for_tokens() -> Weight {
      Weight::from_parts(10_000, 0)
    }
  }
}

/// Default weights implementation.
impl WeightInfo for () {
  fn swap_exact_tokens_for_tokens() -> Weight {
    Weight::from_parts(10_000, 0)
  }
}
