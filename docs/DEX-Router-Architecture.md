# DEX Router Architecture: Trait-Based Multi-AMM Aggregation

## Executive Summary

This document describes the architecture and implementation of a trait-based DEX router that provides unified access to multiple Automated Market Makers (AMMs) with built-in router fees. The solution addresses the core challenge of integrating XYK (constant product) and TBC (Token Bonding Curve) AMMs under a single interface while collecting router fees and automatically selecting optimal execution paths.

**Key Innovation**: Rather than creating "routers over routers," this architecture implements a **composable aggregation layer** that coordinates between independent AMM protocols through trait abstractions, enabling automatic best-price routing with transparent fee collection.

## Problem Statement

### Business Requirements

1. **Unified Interface**: Single entry point for all DEX operations regardless of underlying AMM
2. **Router Fees**: Built-in fee collection (e.g., 0.3%) on all transactions
3. **Multi-AMM Support**:
   - XYK (via `pallet-asset-conversion`) for any asset pairs
   - TBC (future) specifically for Native-Local token pairs
4. **Automatic Optimization**: Best price selection across all available AMMs
5. **Extensibility**: Easy addition of new AMM protocols

### Technical Challenges

1. **Architectural Complexity**: Avoiding layered routing overhead while maintaining modularity
2. **Fee Integration**: Seamless fee collection without bypassing existing AMM protections
3. **Asset Type Constraints**: TBC only works with Native tokens, XYK works with any pairs
4. **Substrate Compatibility**: Respecting account protection mechanisms and reference counters
5. **Performance**: Single-transaction execution with minimal overhead

## Architectural Solution

### Core Design Principles

1. **Trait-Based Polymorphism**: Abstract AMM protocols through common interfaces
2. **Composition Over Inheritance**: Coordinate independent AMMs rather than extending them
3. **Single Transaction Execution**: All routing and fee collection in one atomic operation
4. **Conservative Resource Management**: Respect Substrate's protective architecture
5. **Progressive Enhancement**: Start simple, add complexity incrementally

### System Architecture

```
┌──────────────────────────────────────────────────────────┐
│                       User Interface                     │
│  swap_exact_tokens_for_tokens(path, amount, min_out)     │
└──────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────┐
│                     DEX Router Pallet                    │
│  ┌────────────────────────────────────────────────────┐  │
│  │                  Core Routing Logic                │  │
│  │  • Find compatible AMMs                            │  │
│  │  • Collect price quotes                            │  │
│  │  • Select best price                               │  │
│  │  • Collect router fee                              │  │
│  │  • Execute via optimal AMM                         │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────┐
│                     Trait Abstractions                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │  AMM Trait                 FeeCollector Trait      │  │
│  │  • can_handle_pair()       • collect_fee()         │  │
│  │  • quote_price()                                   │  │
│  │  • execute_swap()          RoutingStrategy Trait   │  │
│  │  • name()                  • select_best_amm()     │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────┐
│                     AMM Implementations                  │
│  ┌────────────────────────────────────────────────────┐  │
│  │  XYKAdapter                    TBCAdapter          │  │
│  │  • Wraps pallet-asset-       • Wraps future TBC    │  │
│  │     conversion                  pallet             │  │
│  │  • Handles any asset pairs   • Native-Local only   │  │
│  │  • Constant product formula  • Bonding curve math  │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Implementation Details

### 1. Core Trait Definitions

#### AMM Trait

```rust
pub trait AMM<AssetKind, Balance, AccountId> {
    type Error: Into<DispatchError>;

    /// Check if this AMM can handle the given asset pair
    fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool;

    /// Get price quote for the swap
    fn quote_price(
        &self,
        asset_in: &AssetKind,
        asset_out: &AssetKind,
        amount_in: Balance,
    ) -> Option<Balance>;

    /// Execute the swap (receives amount_in after router fee deduction)
    fn execute_swap(
        &self,
        who: &AccountId,
        asset_in: AssetKind,
        asset_out: AssetKind,
        amount_in: Balance,
        min_amount_out: Balance,
    ) -> Result<Balance, Self::Error>;

    /// AMM identifier for logging and tracking
    fn name(&self) -> &'static str;
}
```

**Design Rationale**:

- Generic over asset types for maximum flexibility
- `can_handle_pair()` enables intelligent AMM filtering
- `quote_price()` for price discovery without state changes
- `execute_swap()` receives post-fee amount for clean separation of concerns
- `name()` for transparency and debugging

#### Fee Collector Trait

```rust
pub trait FeeCollector<AssetKind, Balance, AccountId> {
    fn collect_fee(
        &self,
        from: &AccountId,
        asset: &AssetKind,
        amount: Balance,
    ) -> DispatchResult;
}
```

**Design Rationale**:

- Abstracted fee collection supports different asset types
- Enables different fee distribution strategies (treasury, DAO, burning)
- Separates fee logic from routing logic

#### Routing Strategy Trait

```rust
pub trait RoutingStrategy<AssetKind, Balance> {
    fn select_best_amm(
        &self,
        quotes: Vec<(AMMType, Balance)>,
        asset_in: &AssetKind,
        asset_out: &AssetKind,
    ) -> Option<AMMType>;
}
```

**Design Rationale**:

- Pluggable routing strategies (best price, prefer specific AMM, etc.)
- Context-aware selection considering asset types
- Extensible for advanced strategies (MEV protection, gas optimization)

### 2. AMM Adapter Implementations

#### XYK Adapter

```rust
pub struct XYKAdapter<T> {
    _phantom: PhantomData<T>,
}

impl<T, AssetKind, Balance, AccountId> AMM<AssetKind, Balance, AccountId> for XYKAdapter<T>
where
    T: polkadot_sdk::pallet_asset_conversion::Config<
        AssetKind = AssetKind,
        Balance = Balance,
        AccountId = AccountId,
    >,
{
    fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
        // XYK can handle any pair if pool exists
        polkadot_sdk::pallet_asset_conversion::Pallet::<T>::get_pool_id(asset_in, asset_out).is_ok()
    }

    fn quote_price(&self, asset_in: &AssetKind, asset_out: &AssetKind, amount_in: Balance) -> Option<Balance> {
        polkadot_sdk::pallet_asset_conversion::Pallet::<T>::quote_price_exact_tokens_for_tokens(
            *asset_in, *asset_out, amount_in, true
        )
    }

    fn execute_swap(&self, who: &AccountId, asset_in: AssetKind, asset_out: AssetKind, amount_in: Balance, min_amount_out: Balance) -> Result<Balance, Self::Error> {
        let path = vec![Box::new(asset_in), Box::new(asset_out)];
        polkadot_sdk::pallet_asset_conversion::Pallet::<T>::swap_exact_tokens_for_tokens(
            polkadot_sdk::frame_system::RawOrigin::Signed(who.clone()).into(),
            path,
            amount_in,
            min_amount_out,
            who.clone(),
            false,
        )?;

        // Return actual amount received (would parse from events in production)
        Ok(amount_in) // Placeholder
    }

    fn name(&self) -> &'static str {
        "XYK"
    }
}
```

#### TBC Adapter (Future Implementation)

```rust
pub struct TBCAdapter<T> {
    _phantom: PhantomData<T>,
}

impl<T, AssetKind, Balance, AccountId> AMM<AssetKind, Balance, AccountId> for TBCAdapter<T>
where
    T: pallet_tbc::Config<Balance = Balance>,
{
    fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
        // TBC only works with Native-Local pairs
        matches!(
            (asset_in, asset_out),
            (AssetKind::Native, AssetKind::Local(_)) |
            (AssetKind::Local(_), AssetKind::Native)
        )
    }

    fn quote_price(&self, asset_in: &AssetKind, asset_out: &AssetKind, amount_in: Balance) -> Option<Balance> {
        match (asset_in, asset_out) {
            (AssetKind::Native, AssetKind::Local(token_id)) => {
                pallet_tbc::Pallet::<T>::calculate_buy_price(*token_id, amount_in)
            },
            (AssetKind::Local(token_id), AssetKind::Native) => {
                pallet_tbc::Pallet::<T>::calculate_sell_price(*token_id, amount_in)
            },
            _ => None,
        }
    }

    fn execute_swap(&self, who: &AccountId, asset_in: AssetKind, asset_out: AssetKind, amount_in: Balance, min_amount_out: Balance) -> Result<Balance, Self::Error> {
        match (asset_in, asset_out) {
            (AssetKind::Native, AssetKind::Local(token_id)) => {
                pallet_tbc::Pallet::<T>::buy_tokens(
                    RuntimeOrigin::signed(who.clone()),
                    token_id,
                    amount_in,
                    min_amount_out,
                )?;
            },
            (AssetKind::Local(token_id), AssetKind::Native) => {
                pallet_tbc::Pallet::<T>::sell_tokens(
                    RuntimeOrigin::signed(who.clone()),
                    token_id,
                    amount_in,
                    min_amount_out,
                )?;
            },
            _ => return Err(pallet_tbc::Error::<T>::InvalidAssetPair.into()),
        }

        let amount_out = Self::extract_amount_from_events()?;
        Ok(amount_out)
    }

    fn name(&self) -> &'static str {
        "TBC"
    }
}
```

### 3. Core Router Logic

```rust
impl<T: Config> Pallet<T> {
    pub fn swap_exact_tokens_for_tokens(
        origin: OriginFor<T>,
        path: Vec<Box<T::AssetKind>>,
        amount_in: T::Balance,
        amount_out_min: T::Balance,
        send_to: T::AccountId,
        keep_alive: bool,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        // Currently support only direct swaps (path length = 2)
        ensure!(path.len() == 2, Error::<T>::InvalidPath);

        let asset_in = *path[0].clone();
        let asset_out = *path[1].clone();

        // 1. Find compatible AMMs
        let compatible_amms = Self::find_compatible_amms(&asset_in, &asset_out);
        ensure!(!compatible_amms.is_empty(), Error::<T>::NoCompatibleAMM);

        // 2. Get quotes from all AMMs
        let quotes = Self::get_quotes(&compatible_amms, &asset_in, &asset_out, amount_in);
        ensure!(!quotes.is_empty(), Error::<T>::NoLiquidityAvailable);

        // 3. Select best AMM using strategy
        let best_amm_type = T::RoutingStrategy::select_best_amm(quotes, &asset_in, &asset_out)
            .ok_or(Error::<T>::NoOptimalRoute)?;

        // 4. Collect router fee
        let router_fee = T::RouterFee::get() * amount_in;
        let effective_amount_in = amount_in.saturating_sub(router_fee);

        T::FeeCollector::collect_fee(&who, &asset_in, router_fee)?;

        // 5. Execute swap through selected AMM
        let selected_amm = compatible_amms.into_iter()
            .find(|amm| Self::amm_to_type(amm) == best_amm_type)
            .ok_or(Error::<T>::AMMNotFound)?;

        let amount_out = selected_amm.execute_swap(
            &who,
            asset_in,
            asset_out,
            effective_amount_in,
            amount_out_min,
        ).map_err(|e| e.into())?;

        // 6. Emit event for transparency
        Self::deposit_event(Event::SwapExecuted {
            who,
            asset_in,
            asset_out,
            amount_in,
            amount_out,
            router_fee,
            amm_used: selected_amm.name(),
        });

        Ok(())
    }
}
```

## Key Benefits of This Approach

### 1. **Architectural Elegance**

**Single Responsibility**: Each component has a clear, focused purpose:

- Router: Orchestration and fee collection
- AMM Traits: Protocol abstraction
- Adapters: Protocol-specific implementation

**Composition Over Complexity**: Rather than building complex inheritance hierarchies, the system composes simple, well-understood components.

### 2. **Extensibility**

**Adding New AMMs**: Implement the `AMM` trait and register in the router:

```rust
// Add Curve Finance style AMM
pub struct CurveAdapter<T> {
    _phantom: PhantomData<T>,
}

impl<T> AMM<AssetKind, Balance, AccountId> for CurveAdapter<T> {
    // Implement trait methods...
}

// Register in router
fn get_amm_registry() -> Vec<Box<dyn AMM<AssetKind, Balance, AccountId>>> {
    vec![
        Box::new(XYKAdapter::new()),
        Box::new(TBCAdapter::new()),
        Box::new(CurveAdapter::new()), // Just add here!
    ]
}
```

**New Routing Strategies**: Implement `RoutingStrategy` for different selection algorithms:

```rust
// Prefer low-slippage over best price
pub struct MinSlippageStrategy;

impl RoutingStrategy<AssetKind, Balance> for MinSlippageStrategy {
    fn select_best_amm(&self, quotes: Vec<(AMMType, Balance)>, asset_in: &AssetKind, asset_out: &AssetKind) -> Option<AMMType> {
        // Custom logic for slippage optimization
    }
}
```

### 3. **Performance Optimization**

**Single Transaction**: All routing, fee collection, and execution happen atomically.

**Parallel Quote Collection**: Can query multiple AMMs simultaneously.

**Efficient Fallbacks**: Failed AMMs are automatically excluded from routing.

### 4. **User Experience**

**Transparent Optimization**: Users get best prices automatically without manual AMM selection.

**Unified Interface**: Same API regardless of underlying complexity.

**Clear Fee Structure**: Router fees are explicit and predictable.

## Advanced Features (Future Development)

### 1. **Multi-Hop Routing**

```rust
// Support complex paths: TokenA -> Native -> TokenB -> TokenC
pub fn execute_multi_hop_swap(
    path: Vec<AssetKind>,
    amount_in: Balance,
) -> DispatchResult {
    for i in 0..path.len()-1 {
        let best_amm = Self::find_best_amm(&path[i], &path[i+1], current_amount)?;
        current_amount = best_amm.execute_swap(/* ... */)?;
    }
}
```

### 2. **Split Order Execution**

```rust
// Split large orders across multiple AMMs for better pricing
pub fn execute_split_order(
    asset_in: AssetKind,
    asset_out: AssetKind,
    amount_in: Balance,
) -> DispatchResult {
    let allocation = Self::calculate_optimal_split(asset_in, asset_out, amount_in)?;

    for (amm_type, amount) in allocation {
        let amm = Self::get_amm(amm_type)?;
        amm.execute_swap(/* ... */)?;
    }
}
```

### 3. **MEV Protection**

```rust
pub struct MEVProtectedStrategy;

impl RoutingStrategy<AssetKind, Balance> for MEVProtectedStrategy {
    fn select_best_amm(&self, quotes: Vec<(AMMType, Balance)>, asset_in: &AssetKind, asset_out: &AssetKind) -> Option<AMMType> {
        // Consider transaction ordering, front-running protection
        // Use commit-reveal schemes or other MEV mitigation
    }
}
```

## Integration Examples

### Example 1: Adding Balancer-Style AMM

```rust
pub struct BalancerAdapter<T> {
    _phantom: PhantomData<T>,
}

impl<T> AMM<AssetKind, Balance, AccountId> for BalancerAdapter<T>
where
    T: pallet_balancer::Config,
{
    fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
        // Check if assets are in the same Balancer pool
        pallet_balancer::Pallet::<T>::find_pool_with_assets(asset_in, asset_out).is_some()
    }

    fn quote_price(&self, asset_in: &AssetKind, asset_out: &AssetKind, amount_in: Balance) -> Option<Balance> {
        pallet_balancer::Pallet::<T>::calculate_swap_amount(asset_in, asset_out, amount_in)
    }

    fn execute_swap(&self, who: &AccountId, asset_in: AssetKind, asset_out: AssetKind, amount_in: Balance, min_amount_out: Balance) -> Result<Balance, Self::Error> {
        pallet_balancer::Pallet::<T>::swap_exact_tokens_for_tokens(
            RuntimeOrigin::signed(who.clone()),
            asset_in,
            asset_out,
            amount_in,
            min_amount_out,
        )
    }

    fn name(&self) -> &'static str {
        "Balancer"
    }
}
```

### Example 2: Custom Fee Distribution

```rust
pub struct DAOFeeCollector<T> {
    _phantom: PhantomData<T>,
}

impl<T> FeeCollector<AssetKind, Balance, AccountId> for DAOFeeCollector<T>
where
    T: pallet_dao::Config + pallet_dex_router::Config,
{
    fn collect_fee(&self, from: &AccountId, asset: &AssetKind, amount: Balance) -> DispatchResult {
        // 70% to DAO treasury
        let dao_amount = amount * 70 / 100;
        Self::transfer_to_dao(from, asset, dao_amount)?;

        // 20% to liquidity providers
        let lp_amount = amount * 20 / 100;
        Self::distribute_to_lps(from, asset, lp_amount)?;

        // 10% burn
        let burn_amount = amount - dao_amount - lp_amount;
        Self::burn_tokens(from, asset, burn_amount)?;

        Ok(())
    }
}
```

## Production Considerations

### 1. **Security**

- **Router Fee Validation**: Ensure fees are collected before swap execution
- **Slippage Protection**: Validate minimum output amounts across all AMMs
- **Access Control**: Consider who can add new AMMs to the registry
- **Reentrancy Protection**: Ensure atomic execution of routing operations

### 2. **Performance**

- **Weight Optimization**: Benchmark different routing strategies
- **Quote Caching**: Cache frequently requested quotes
- **Batch Operations**: Support multiple swaps in single transaction

### 3. **Monitoring**

- **Route Analytics**: Track which AMMs are used most frequently
- **Fee Analytics**: Monitor total fees collected and distribution
- **Performance Metrics**: Measure execution times and gas costs
- **Arbitrage Detection**: Identify and possibly prevent sandwich attacks

### 4. **Upgrades**

- **Versioned Traits**: Plan for trait evolution without breaking changes
- **Migration Paths**: Clear upgrade process for AMM integrations
- **Backward Compatibility**: Maintain API stability for existing integrations

## Conclusion

This trait-based DEX router architecture provides a clean, extensible solution for multi-AMM aggregation with built-in fee collection. The key innovations are:

1. **Trait Abstraction**: Clean separation between routing logic and AMM implementation
2. **Composition Pattern**: Coordinate independent AMMs without complex inheritance
3. **Automatic Optimization**: Best price selection with configurable strategies
4. **Built-in Economics**: Transparent router fee collection and distribution
5. **Extensibility**: Easy addition of new AMMs and routing strategies

The architecture scales from simple two-AMM scenarios (XYK + TBC) to complex multi-protocol ecosystems while maintaining clean abstractions and optimal user experience. By working with Substrate's protective architecture rather than against it, the router provides production-ready functionality that respects the platform's defensive programming philosophy.

This approach transforms the challenge of "integrating multiple AMMs" into an opportunity to build a comprehensive DeFi infrastructure that benefits from the best features of each protocol while providing users with automatic optimization and transparent fee structures.
