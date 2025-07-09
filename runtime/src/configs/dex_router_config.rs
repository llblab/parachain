//! DEX Router pallet configuration for the parachain runtime.
//!
//! Configures the trait-based DEX router with built-in fees.

use polkadot_sdk::*;
use sp_runtime::Permill;

use crate::configs::assets_config::AssetKind;
use crate::{AccountId, Balance, Runtime, RuntimeEvent};

frame_support::parameter_types! {
  /// Router fee percentage (0.2% = 20 basis points) for buyback mechanism
  pub const RouterFee: Permill = Permill::from_parts(2000);

  /// Account that receives router fees for buyback and burning
  pub const RouterFeeCollector: AccountId = AccountId::new([0u8; 32]);
}

impl pallet_dex_router::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Balance = Balance;
  type AssetKind = AssetKind;
  type RouterFee = RouterFee;
  type RouterFeeCollector = RouterFeeCollector;
  type WeightInfo = ();
  type AssetConversion = Runtime;
  type Balances = Runtime;
}
