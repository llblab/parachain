# AMM Adapter Usage Guide: Extending Multi-AMM Support

## Overview

This guide explains how to use and extend the DEX Router's adapter system to support multiple AMMs. The adapter pattern allows seamless integration of different AMM types (XYK, Curve, Balancer, etc.) through a unified interface.

## Current Architecture

### 1. **Adapter Interface (Trait)**

All AMM adapters implement the `AMM` trait:

```rust
pub trait AMM<AssetKind, Balance, AccountId> {
  type Error: Into<DispatchError>;

  fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool;
  fn quote_price(&self, asset_in: &AssetKind, asset_out: &AssetKind, amount_in: Balance) -> Option<Balance>;
  fn execute_swap(&self, who: &AccountId, asset_in: AssetKind, asset_out: AssetKind, amount_in: Balance, min_amount_out: Balance) -> Result<Balance, Self::Error>;
  fn name(&self) -> &'static str;
}
```

### 2. **Current Implementation**

Currently, the system has:
- **XYKAdapter**: Wraps `pallet-asset-conversion` (Uniswap V2 style)
- **DefaultFeeCollector**: Handles fee collection through `pallet-balances`

## How to Add New AMM Adapters

### Example 1: Adding a Curve AMM Adapter

```rust
use crate::traits::AMM;
use core::marker::PhantomData;
use frame::prelude::*;

/// Curve AMM adapter for stable coin swaps
pub struct CurveAdapter<T> {
  _phantom: PhantomData<T>,
}

impl<T> CurveAdapter<T> {
  pub fn new() -> Self {
    Self {
      _phantom: PhantomData,
    }
  }
}

impl<T, AssetKind, Balance, AccountId> AMM<AssetKind, Balance, AccountId> for CurveAdapter<T>
where
  T: pallet_curve::Config<AssetKind = AssetKind, Balance = Balance, AccountId = AccountId>
    + frame_system::Config<AccountId = AccountId>,
  AssetKind: Clone + Copy,
  Balance: Zero + From<u32> + Copy + PartialOrd,
  AccountId: Clone,
{
  type Error = DispatchError;

  fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
    // Check if assets are in the same Curve pool
    pallet_curve::Pallet::<T>::pool_exists_for_pair(asset_in, asset_out)
  }

  fn quote_price(
    &self,
    asset_in: &AssetKind,
    asset_out: &AssetKind,
    amount_in: Balance,
  ) -> Option<Balance> {
    pallet_curve::Pallet::<T>::get_dy(*asset_in, *asset_out, amount_in)
  }

  fn execute_swap(
    &self,
    who: &AccountId,
    asset_in: AssetKind,
    asset_out: AssetKind,
    amount_in: Balance,
    min_amount_out: Balance,
  ) -> Result<Balance, Self::Error> {
    let actual_out = pallet_curve::Pallet::<T>::exchange(
      frame_system::RawOrigin::Signed(who.clone()).into(),
      asset_in,
      asset_out,
      amount_in,
      min_amount_out,
    )
    .map_err(|_| DispatchError::Other("Curve swap failed"))?;

    Ok(actual_out)
  }

  fn name(&self) -> &'static str {
    "Curve"
  }
}
```

### Example 2: Adding a Balancer-style AMM

```rust
/// Balancer AMM adapter for weighted pools
pub struct BalancerAdapter<T> {
  _phantom: PhantomData<T>,
}

impl<T, AssetKind, Balance, AccountId> AMM<AssetKind, Balance, AccountId> for BalancerAdapter<T>
where
  T: pallet_balancer::Config<AssetKind = AssetKind, Balance = Balance, AccountId = AccountId>
    + frame_system::Config<AccountId = AccountId>,
  AssetKind: Clone + Copy,
  Balance: Zero + From<u32> + Copy + PartialOrd,
  AccountId: Clone,
{
  type Error = DispatchError;

  fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
    // Check if there's a weighted pool containing both assets
    pallet_balancer::Pallet::<T>::find_pool_with_assets(asset_in, asset_out).is_some()
  }

  fn quote_price(
    &self,
    asset_in: &AssetKind,
    asset_out: &AssetKind,
    amount_in: Balance,
  ) -> Option<Balance> {
    pallet_balancer::Pallet::<T>::quote_swap_exact_amount_in(
      *asset_in, *asset_out, amount_in
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
    let pool_id = pallet_balancer::Pallet::<T>::find_pool_with_assets(&asset_in, &asset_out)
      .ok_or(DispatchError::Other("No suitable pool found"))?;

    let actual_out = pallet_balancer::Pallet::<T>::swap_exact_amount_in(
      frame_system::RawOrigin::Signed(who.clone()).into(),
      pool_id,
      asset_in,
      amount_in,
      asset_out,
      min_amount_out,
    )
    .map_err(|_| DispatchError::Other("Balancer swap failed"))?;

    Ok(actual_out)
  }

  fn name(&self) -> &'static str {
    "Balancer"
  }
}
```

## Updating the Main Pallet for Multiple AMMs

### Enhanced AMM Selection Logic

```rust
impl<T: Config> Pallet<T> {
  /// Get all available adapters
  fn get_adapters() -> Vec<Box<dyn AMM<T::AssetKind, T::Balance, T::AccountId, Error = DispatchError>>> {
    let mut adapters = Vec::new();

    // Add XYK adapter
    adapters.push(Box::new(XYKAdapter::<T::AssetConversion>::new()));

    // Add Curve adapter if available
    #[cfg(feature = "curve-support")]
    adapters.push(Box::new(CurveAdapter::<T::CurveConfig>::new()));

    // Add Balancer adapter if available
    #[cfg(feature = "balancer-support")]
    adapters.push(Box::new(BalancerAdapter::<T::BalancerConfig>::new()));

    adapters
  }

  /// Get the best quote from all available AMMs
  fn get_best_quote(
    asset_in: &T::AssetKind,
    asset_out: &T::AssetKind,
    amount_in: T::Balance,
  ) -> Option<(T::Balance, AMMType)> {
    let mut best_quote = None;
    let mut best_amm = AMMType::XYK;

    // XYK Adapter
    let xyk_adapter = XYKAdapter::<T::AssetConversion>::new();
    if xyk_adapter.can_handle_pair(asset_in, asset_out) {
      if let Some(quote) = xyk_adapter.quote_price(asset_in, asset_out, amount_in) {
        best_quote = Some(quote);
        best_amm = AMMType::XYK;
      }
    }

    // Curve Adapter
    #[cfg(feature = "curve-support")]
    {
      let curve_adapter = CurveAdapter::<T::CurveConfig>::new();
      if curve_adapter.can_handle_pair(asset_in, asset_out) {
        if let Some(quote) = curve_adapter.quote_price(asset_in, asset_out, amount_in) {
          if best_quote.map_or(true, |best| quote > best) {
            best_quote = Some(quote);
            best_amm = AMMType::Curve;
          }
        }
      }
    }

    // Balancer Adapter
    #[cfg(feature = "balancer-support")]
    {
      let balancer_adapter = BalancerAdapter::<T::BalancerConfig>::new();
      if balancer_adapter.can_handle_pair(asset_in, asset_out) {
        if let Some(quote) = balancer_adapter.quote_price(asset_in, asset_out, amount_in) {
          if best_quote.map_or(true, |best| quote > best) {
            best_quote = Some(quote);
            best_amm = AMMType::Balancer;
          }
        }
      }
    }

    best_quote.map(|quote| (quote, best_amm))
  }

  /// Execute swap using the best available AMM
  fn execute_best_swap(
    who: &T::AccountId,
    asset_in: T::AssetKind,
    asset_out: T::AssetKind,
    amount_in: T::Balance,
    min_amount_out: T::Balance,
  ) -> Result<(T::Balance, AMMType), DispatchError> {
    // Get the best quote and AMM type
    let (best_quote, best_amm) = Self::get_best_quote(&asset_in, &asset_out, amount_in)
      .ok_or(Error::<T>::NoCompatibleAMM)?;

    // Execute swap based on the best AMM
    let actual_out = match best_amm {
      AMMType::XYK => {
        let adapter = XYKAdapter::<T::AssetConversion>::new();
        adapter.execute_swap(who, asset_in, asset_out, amount_in, min_amount_out)?
      }

      #[cfg(feature = "curve-support")]
      AMMType::Curve => {
        let adapter = CurveAdapter::<T::CurveConfig>::new();
        adapter.execute_swap(who, asset_in, asset_out, amount_in, min_amount_out)?
      }

      #[cfg(feature = "balancer-support")]
      AMMType::Balancer => {
        let adapter = BalancerAdapter::<T::BalancerConfig>::new();
        adapter.execute_swap(who, asset_in, asset_out, amount_in, min_amount_out)?
      }

      _ => return Err(Error::<T>::NoCompatibleAMM.into()),
    };

    Ok((actual_out, best_amm))
  }
}
```

## Extended AMMType Enum

```rust
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum AMMType {
  /// Uniswap V2-style constant product AMM
  XYK,
  /// Curve-style stable coin AMM
  Curve,
  /// Balancer-style weighted pool AMM
  Balancer,
  /// Token Bonding Curve AMM
  TBC,
}
```

## Configuration Extension

### Runtime Configuration

```rust
// In runtime/src/configs/dex_router_config.rs
impl pallet_dex_router::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Balance = Balance;
  type AssetKind = AssetKind;
  type RouterFee = RouterFee;
  type RouterFeeCollector = RouterFeeCollector;
  type WeightInfo = ();
  type AssetConversion = Runtime;  // For XYK
  type Balances = Runtime;

  // Additional AMM configurations
  #[cfg(feature = "curve-support")]
  type CurveConfig = Runtime;

  #[cfg(feature = "balancer-support")]
  type BalancerConfig = Runtime;
}
```

### Cargo.toml Features

```toml
[features]
default = ["std"]
std = [
  "frame/std",
  "polkadot-sdk/std",
]
curve-support = ["pallet-curve"]
balancer-support = ["pallet-balancer"]
runtime-benchmarks = [
  "frame/runtime-benchmarks",
]

[dependencies]
# Optional AMM dependencies
pallet-curve = { workspace = true, optional = true }
pallet-balancer = { workspace = true, optional = true }
```

## Usage Examples

### 1. **Simple Swap (Current)**

```rust
// User calls the DEX Router
DexRouter::swap_exact_tokens_for_tokens(
  origin,
  bounded_vec![AssetKind::Native, AssetKind::Local(1)],
  1000 * UNIT,
  900 * UNIT,
  alice.clone(),
  false,
)?;

// Router automatically:
// 1. Checks XYK adapter
// 2. Gets quote
// 3. Executes swap
// 4. Collects fees
```

### 2. **Multi-AMM Comparison (Future)**

```rust
// User calls the same function, but now:
DexRouter::swap_exact_tokens_for_tokens(
  origin,
  bounded_vec![AssetKind::Native, AssetKind::Local(1)],
  1000 * UNIT,
  900 * UNIT,
  alice.clone(),
  false,
)?;

// Router automatically:
// 1. Checks XYK adapter -> Quote: 950 tokens
// 2. Checks Curve adapter -> Quote: 980 tokens (better for stablecoins)
// 3. Checks Balancer adapter -> Quote: 960 tokens
// 4. Selects Curve (best quote)
// 5. Executes swap through Curve
// 6. Collects fees
```

### 3. **Custom Routing Strategy**

```rust
pub struct CustomRoutingStrategy;

impl<AssetKind, Balance> RoutingStrategy<AssetKind, Balance> for CustomRoutingStrategy
where
  Balance: Ord + Copy,
{
  fn select_best_amm(
    &self,
    quotes: Vec<(AMMType, Balance)>,
    asset_in: &AssetKind,
    asset_out: &AssetKind,
  ) -> Option<AMMType> {
    // Custom logic: prefer Curve for stablecoins, XYK for others
    let is_stablecoin_pair = self.is_stablecoin_pair(asset_in, asset_out);

    if is_stablecoin_pair {
      // Prefer Curve for stablecoins even if not the best quote
      quotes.iter()
        .find(|(amm_type, _)| matches!(amm_type, AMMType::Curve))
        .map(|(amm_type, _)| *amm_type)
        .or_else(|| {
          // Fallback to best quote
          quotes.into_iter()
            .max_by_key(|(_, quote)| *quote)
            .map(|(amm_type, _)| amm_type)
        })
    } else {
      // For non-stablecoins, just pick the best quote
      quotes.into_iter()
        .max_by_key(|(_, quote)| *quote)
        .map(|(amm_type, _)| amm_type)
    }
  }
}
```

## Testing Multiple AMMs

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_multi_amm_selection() {
    ExtBuilder::default().build_and_execute(|| {
      // Setup: Create pools in different AMMs
      setup_xyk_pool();
      setup_curve_pool();
      setup_balancer_pool();

      // Test: Router selects best AMM
      let result = DexRouter::swap_exact_tokens_for_tokens(
        RuntimeOrigin::signed(ALICE),
        bounded_vec![AssetKind::Native, AssetKind::Local(1)],
        1000 * UNIT,
        900 * UNIT,
        ALICE,
        false,
      );

      assert_ok!(result);

      // Verify: Best AMM was selected
      let events = System::events();
      assert!(events.iter().any(|e| matches!(
        e.event,
        RuntimeEvent::DexRouter(Event::SwapExecuted { amm_used: AMMType::Curve, .. })
      )));
    });
  }
}
```

## Best Practices

### 1. **Adapter Design**
- Keep adapters stateless
- Use PhantomData for type parameters
- Implement proper error handling
- Make adapters lightweight

### 2. **Performance**
- Cache adapter instances when possible
- Use bounded collections (no `Vec<Box<T>>`)
- Avoid unnecessary cloning
- Implement efficient quote comparison

### 3. **Security**
- Validate all inputs in adapters
- Handle edge cases (zero amounts, etc.)
- Implement proper access controls
- Use safe math operations

### 4. **Extensibility**
- Use feature flags for optional AMMs
- Keep the core trait minimal
- Allow custom routing strategies
- Support runtime configuration

## Conclusion

The adapter pattern provides a clean, extensible way to support multiple AMMs in the DEX Router. Each AMM is wrapped in its own adapter that implements the common `AMM` trait, allowing the router to:

1. **Compare quotes** from multiple sources
2. **Select the best AMM** for each trade
3. **Execute swaps** through the chosen AMM
4. **Maintain consistent behavior** across different AMM types

This architecture makes it easy to add new AMMs, customize routing logic, and provide users with the best possible trading experience.
