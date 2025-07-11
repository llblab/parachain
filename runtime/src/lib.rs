#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;
mod genesis_config_presets;
#[cfg(test)]
mod tests;
mod weights;

extern crate alloc;
use alloc::vec::Vec;
use smallvec::smallvec;

use polkadot_sdk::{frame_support::construct_runtime, staging_parachain_info as parachain_info, *};

use sp_runtime::{
  generic, impl_opaque_keys,
  traits::{BlakeTwo256, IdentifyAccount, Verify},
  MultiSignature,
};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::weights::{
  constants::WEIGHT_REF_TIME_PER_SECOND, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
  WeightToFeePolynomial,
};
pub use genesis_config_presets::PARACHAIN_ID;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

use weights::ExtrinsicBaseWeight;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The extension to the basic transaction logic.
#[docify::export(template_signed_extra)]
pub type TxExtension = (
  frame_system::CheckNonZeroSender<Runtime>,
  frame_system::CheckSpecVersion<Runtime>,
  frame_system::CheckTxVersion<Runtime>,
  frame_system::CheckGenesis<Runtime>,
  frame_system::CheckEra<Runtime>,
  frame_system::CheckNonce<Runtime>,
  frame_system::CheckWeight<Runtime>,
  pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
  frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
  generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
  type Balance = Balance;
  fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
    // in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLI_UNIT:
    // in our template, we map to 1/10 of that, or 1/10 MILLI_UNIT
    let p = MILLI_UNIT / 10;
    let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
    smallvec![WeightToFeeCoefficient {
      degree: 1,
      negative: false,
      coeff_frac: Perbill::from_rational(p % q, q),
      coeff_integer: p / q,
    }]
  }
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
  use super::*;
  pub use polkadot_sdk::sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
  use polkadot_sdk::sp_runtime::{
    generic,
    traits::{BlakeTwo256, Hash as HashT},
  };

  /// Opaque block header type.
  pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
  /// Opaque block type.
  pub type Block = generic::Block<Header, UncheckedExtrinsic>;
  /// Opaque block identifier type.
  pub type BlockId = generic::BlockId<Block>;
  /// Opaque block hash type.
  pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
  spec_name: alloc::borrow::Cow::Borrowed("parachain-template-runtime"),
  impl_name: alloc::borrow::Cow::Borrowed("parachain-template-runtime"),
  authoring_version: 1,
  spec_version: 1,
  impl_version: 0,
  apis: apis::RUNTIME_API_VERSIONS,
  transaction_version: 1,
  system_version: 1,
};

#[docify::export]
mod block_times {
  /// This determines the average expected block time that we are targeting. Blocks will be
  /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
  /// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
  /// slot_duration()`.
  ///
  /// Change this to adjust the block time.
  pub const MILLI_SECS_PER_BLOCK: u64 = 6000;

  // NOTE: Currently it is not possible to change the slot duration after the chain has started.
  // Attempting to do so will brick block production.
  pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;
}
pub use block_times::*;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLI_SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLI_UNIT: Balance = 1_000_000_000;
pub const MICRO_UNIT: Balance = 1_000_000;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLI_UNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

#[docify::export(max_block_weight)]
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
  WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
  cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

#[docify::export]
mod async_backing_params {
  /// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
  /// into the relay chain.
  pub(crate) const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;
  /// How many parachain blocks are processed by the relay chain per parent. Limits the
  /// number of blocks authored per slot.
  pub(crate) const BLOCK_PROCESSING_VELOCITY: u32 = 1;
  /// Relay chain slot duration, in milliseconds.
  pub(crate) const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;
}
pub(crate) use async_backing_params::*;

#[docify::export]
/// Aura consensus hook
type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
  Runtime,
  RELAY_CHAIN_SLOT_DURATION_MILLIS,
  BLOCK_PROCESSING_VELOCITY,
  UNINCLUDED_SEGMENT_CAPACITY,
>;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
  NativeVersion {
    runtime_version: VERSION,
    can_author_with: Default::default(),
  }
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
  pub enum Runtime {
    // System support stuff.
    System: frame_system = 0,
    ParachainSystem: cumulus_pallet_parachain_system = 1,
    Timestamp: pallet_timestamp = 2,
    ParachainInfo: parachain_info = 3,
    WeightReclaim: cumulus_pallet_weight_reclaim = 4,

    // Monetary stuff.
    Balances: pallet_balances = 10,
    TransactionPayment: pallet_transaction_payment = 11,

    // Assets for fungible tokens
    Assets: pallet_assets = 12,
    AssetConversion: pallet_asset_conversion = 13,
    DexRouter: pallet_dex_router = 14,

    // Governance
    Sudo: pallet_sudo = 15,

    // Collator support. The order of these 4 are important and shall not change.
    Authorship: pallet_authorship = 20,
    CollatorSelection: pallet_collator_selection = 21,
    Session: pallet_session = 22,
    Aura: pallet_aura = 23,
    AuraExt: cumulus_pallet_aura_ext = 24,

    // XCM helpers.
    XcmpQueue: cumulus_pallet_xcmp_queue = 30,
    PolkadotXcm: pallet_xcm = 31,
    CumulusXcm: cumulus_pallet_xcm = 32,
    MessageQueue: pallet_message_queue = 33,
  }
);

// Runtime is already exported by construct_runtime! macro - no need to re-export
// The types RuntimeCall, RuntimeEvent, RuntimeOrigin, etc. are also generated by the macro

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
  Runtime,
  Block,
  frame_system::ChainContext<Runtime>,
  Runtime,
  AllPalletsWithSystem,
  Migrations,
>;

#[docify::export(register_validate_block)]
cumulus_pallet_parachain_system::register_validate_block! {
  Runtime = Runtime,
  BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
