//! Asset-related pallet configurations for the parachain runtime.
//!
//! Configures:
//! - `pallet-assets`: Fungible asset management
//! - `pallet-asset-conversion`: Uniswap V2-like DEX functionality

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::traits::*;
use polkadot_sdk::*;
use scale_info::TypeInfo;

use crate::{AccountId, Balance, Balances, Runtime, RuntimeEvent, EXISTENTIAL_DEPOSIT};

/// Asset ID type used throughout the runtime
pub type AssetId = u32;

/// Asset kind for Asset Conversion
///
/// - Native: Parachain's native token (pallet-balances)
/// - Local(u32): Local assets (pallet-assets)
#[derive(
  Clone,
  Copy,
  Debug,
  Decode,
  DecodeWithMemTracking,
  Encode,
  Eq,
  MaxEncodedLen,
  Ord,
  PartialEq,
  PartialOrd,
  TypeInfo,
)]
pub enum AssetKind {
  /// Native token managed by pallet-balances
  Native,
  /// Local asset managed by pallet-assets
  Local(u32),
}

impl From<u32> for AssetKind {
  fn from(asset_id: u32) -> Self {
    AssetKind::Local(asset_id)
  }
}

frame_support::parameter_types! {
  /// Minimum balance required to create an asset
  pub const AssetDeposit: Balance = EXISTENTIAL_DEPOSIT;
  /// Minimum balance required to create metadata for an asset
  pub const MetadataDepositBase: Balance = EXISTENTIAL_DEPOSIT;
  /// Additional deposit required per byte of metadata
  pub const MetadataDepositPerByte: Balance = EXISTENTIAL_DEPOSIT;
  /// Minimum balance required to approve an asset transfer
  pub const ApprovalDeposit: Balance = EXISTENTIAL_DEPOSIT;
  /// Maximum length of asset name
  pub const StringLimit: u32 = 50;
  /// Minimum balance required to keep an asset account alive
  pub const AssetAccountDeposit: Balance = EXISTENTIAL_DEPOSIT;

  // Asset Conversion parameters
  pub const AssetConversionPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/ascon");

  /// Liquidity withdrawal fee (0%)
  pub const LiquidityWithdrawalFee: sp_runtime::Permill = sp_runtime::Permill::from_percent(0);

  /// Minimum liquidity that must be minted when creating a pool
  /// Matches Asset Hub configuration to ensure proper account reference counting
  pub const MintMinLiquidity: Balance = 100;

  /// Pool setup fee to prevent spam pool creation (temporarily disabled for testing)
  pub const PoolSetupFee: Balance = 0;
}

/// Ensure that the asset operations can only be performed by root or the asset owner
pub type AssetsForceOrigin = frame_system::EnsureRoot<AccountId>;

impl pallet_assets::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Balance = Balance;
  type AssetId = AssetId;
  type AssetIdParameter = AssetId;
  type Currency = Balances;
  type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
  type ForceOrigin = AssetsForceOrigin;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type Extra = ();
  type WeightInfo = ();
  type RemoveItemsLimit = ConstU32<1000>;
  type AssetAccountDeposit = AssetAccountDeposit;
  type CallbackHandle = ();
  type Holder = ();
  #[cfg(feature = "runtime-benchmarks")]
  type BenchmarkHelper = ();
}

impl pallet_asset_conversion::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Balance = Balance;
  type HigherPrecisionBalance = sp_core::U256;
  type AssetKind = AssetKind;
  type Assets = frame_support::traits::fungible::UnionOf<
    Balances,
    pallet_assets::Pallet<Runtime>,
    NativeOrAssetIdConverter,
    AssetKind,
    AccountId,
  >;
  type PoolId = (AssetKind, AssetKind);
  type PoolLocator = pallet_asset_conversion::WithFirstAsset<
    NativeAssetId,
    AccountId,
    AssetKind,
    pallet_asset_conversion::AccountIdConverter<AssetConversionPalletId, (AssetKind, AssetKind)>,
  >;
  type PoolAssetId = u32;
  type PoolAssets = pallet_assets::Pallet<Runtime>;
  type LPFee = ConstU32<3>;
  type PoolSetupFee = PoolSetupFee;
  type PoolSetupFeeAsset = NativeAssetId;
  type PoolSetupFeeTarget = ();
  type LiquidityWithdrawalFee = LiquidityWithdrawalFee;
  type MintMinLiquidity = MintMinLiquidity;
  type MaxSwapPathLength = ConstU32<4>;
  type PalletId = AssetConversionPalletId;
  type WeightInfo = ();
  #[cfg(feature = "runtime-benchmarks")]
  type BenchmarkHelper = AssetKindBenchmarkHelper;
}

/// Benchmark helper for AssetKind
#[cfg(feature = "runtime-benchmarks")]
pub struct AssetKindBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_asset_conversion::BenchmarkHelper<AssetKind> for AssetKindBenchmarkHelper {
  fn create_pair(asset1: u32, asset2: u32) -> (AssetKind, AssetKind) {
    (AssetKind::Local(asset1), AssetKind::Local(asset2))
  }
}

frame_support::parameter_types! {
  /// Native asset ID
  pub const NativeAssetId: AssetKind = AssetKind::Native;
}

/// Converter to distinguish between native and asset tokens
pub struct NativeOrAssetIdConverter;

impl sp_runtime::traits::Convert<AssetKind, sp_runtime::Either<(), AssetId>>
  for NativeOrAssetIdConverter
{
  fn convert(asset_kind: AssetKind) -> sp_runtime::Either<(), AssetId> {
    match asset_kind {
      AssetKind::Native => sp_runtime::Either::Left(()),
      AssetKind::Local(asset_id) => sp_runtime::Either::Right(asset_id),
    }
  }
}
