# DEX Router Pallet

A trait-based DEX router pallet that provides a unified interface for multiple Automated Market Makers (AMMs) with built-in router fees.

## Overview

This pallet implements a sophisticated routing system that can aggregate multiple AMM protocols (XYK, TBC, etc.) and automatically select the best price for token swaps while collecting router fees. The architecture is designed to be extensible and composable.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    DEX Router Pallet                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Public Interface                        │   │
│  │  • swap_exact_tokens_for_tokens()                   │   │
│  │  • Router fee collection                            │   │
│  │  • Best price selection                             │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                Trait Layer                          │   │
│  │  • AMM trait abstraction                           │   │
│  │  • FeeCollector trait                              │   │
│  │  • RoutingStrategy trait                           │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              AMM Adapters                           │   │
│  │  • XYKAdapter (pallet-asset-conversion)            │   │
│  │  • TBCAdapter (future Token Bonding Curves)        │   │
│  │  • Future: Curve, Balancer, etc.                   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Trait-Based Design

The pallet uses a trait-based architecture that allows for easy extension:

#### 1. AMM Trait
```rust
pub trait AMM<AssetKind, Balance, AccountId> {
    type Error: Into<DispatchError>;

    /// Check if this AMM can handle the given asset pair
    fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool;

    /// Get price quote
    fn quote_price(&self, asset_in: &AssetKind, asset_out: &AssetKind, amount_in: Balance) -> Option<Balance>;

    /// Execute swap (receives amount after router fee deduction)
    fn execute_swap(&self, who: &AccountId, asset_in: AssetKind, asset_out: AssetKind, amount_in: Balance, min_amount_out: Balance) -> Result<Balance, Self::Error>;

    /// AMM identifier
    fn name(&self) -> &'static str;
}
```

#### 2. Fee Collector Trait
```rust
pub trait FeeCollector<AssetKind, Balance, AccountId> {
    fn collect_fee(&self, from: &AccountId, asset: &AssetKind, amount: Balance) -> DispatchResult;
}
```

#### 3. Routing Strategy Trait
```rust
pub trait RoutingStrategy<AssetKind, Balance> {
    fn select_best_amm(&self, quotes: Vec<(AMMType, Balance)>, asset_in: &AssetKind, asset_out: &AssetKind) -> Option<AMMType>;
}
```

## Key Features

### 1. **Multi-AMM Support**
- **XYK (Constant Product)**: Integrates with `pallet-asset-conversion`
- **TBC (Token Bonding Curves)**: Future integration for bonding curve AMMs
- **Extensible**: Easy to add new AMM types through trait implementation

### 2. **Automatic Best Price Selection**
- Queries all compatible AMMs for price quotes
- Automatically selects the AMM offering the best output amount
- Configurable routing strategies (best price, prefer specific AMM, etc.)

### 3. **Built-in Router Fees**
- Configurable fee percentage (e.g., 0.3%)
- Automatic fee collection before swap execution
- Designated fee collector account
- Supports both native and local asset fees

### 4. **Unified Interface**
- Single `swap_exact_tokens_for_tokens` extrinsic for all AMMs
- Transparent routing - users don't need to know which AMM is used
- Consistent API regardless of underlying AMM complexity

### 5. **Safety and Validation**
- Path validation (currently supports direct swaps)
- Slippage protection through minimum output amounts
- Error handling with descriptive error types

## Technical Implementation

### Current Status

✅ **Completed:**
- Core trait definitions (AMM, FeeCollector, RoutingStrategy)
- Basic pallet structure with configuration
- XYK adapter framework (placeholder implementation)
- Event system for swap tracking
- Router fee collection mechanism

🚧 **In Progress:**
- Full XYK adapter implementation with `pallet-asset-conversion`
- TBC adapter implementation
- Comprehensive test suite

📋 **Future Work:**
- Runtime integration
- Multi-hop routing (A -> B -> C)
- Split order execution (partial fills across multiple AMMs)
- Advanced routing strategies
- Benchmarking and weight optimization

### Configuration

```rust
impl pallet_dex_router::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetKind = AssetKind;
    type RouterFee = RouterFee;                    // e.g., 0.3%
    type RouterFeeCollector = RouterFeeCollector;  // Treasury account
    type WeightInfo = ();
}
```

### Usage Example

```rust
// User calls the unified swap interface
DexRouter::swap_exact_tokens_for_tokens(
    origin,
    vec![Box::new(AssetKind::Native), Box::new(AssetKind::Local(1))], // path
    1000_000_000_000, // amount_in (1 token)
    900_000_000_000,  // amount_out_min (0.9 tokens minimum)
    user_account,     // send_to
    false,           // keep_alive
)?;

// Internally, the router:
// 1. Finds compatible AMMs (XYK, TBC)
// 2. Gets quotes: XYK = 950, TBC = 980
// 3. Selects TBC (best price)
// 4. Collects router fee: 3_000_000_000 (0.3%)
// 5. Executes swap through TBC with remaining amount
// 6. User receives ~980 tokens, router gets fee
```

## AMM Integration Guide

### Adding a New AMM

1. **Implement the AMM Trait:**
```rust
pub struct NewAMMAdapter<T> {
    _phantom: PhantomData<T>,
}

impl<T, AssetKind, Balance, AccountId> AMM<AssetKind, Balance, AccountId> for NewAMMAdapter<T>
where
    T: pallet_new_amm::Config<AssetKind = AssetKind, Balance = Balance>,
{
    type Error = pallet_new_amm::Error<T>;

    fn can_handle_pair(&self, asset_in: &AssetKind, asset_out: &AssetKind) -> bool {
        // Implementation-specific logic
    }

    fn quote_price(&self, asset_in: &AssetKind, asset_out: &AssetKind, amount_in: Balance) -> Option<Balance> {
        pallet_new_amm::Pallet::<T>::get_quote(asset_in, asset_out, amount_in)
    }

    fn execute_swap(&self, who: &AccountId, asset_in: AssetKind, asset_out: AssetKind, amount_in: Balance, min_amount_out: Balance) -> Result<Balance, Self::Error> {
        pallet_new_amm::Pallet::<T>::swap(who, asset_in, asset_out, amount_in, min_amount_out)
    }

    fn name(&self) -> &'static str {
        "NewAMM"
    }
}
```

2. **Register in Router:**
```rust
fn get_amm_registry() -> Vec<Box<dyn AMM<AssetKind, Balance, AccountId>>> {
    vec![
        Box::new(XYKAdapter::new()),
        Box::new(TBCAdapter::new()),
        Box::new(NewAMMAdapter::new()), // Add new AMM
    ]
}
```

## Business Logic

### Router Fee Economics

- **Fee Collection**: Router collects a small percentage (e.g., 0.3%) from input amount
- **Value Proposition**: Users get best price across all AMMs automatically
- **Fee Distribution**: Collected fees go to designated treasury/DAO account
- **Transparency**: All fees and routing decisions are logged in events

### Routing Strategies

1. **Best Price (Default)**: Always routes to AMM with highest output
2. **Prefer XYK**: Try XYK first, fallback to others
3. **Prefer TBC**: Try TBC first, fallback to others
4. **AMM-Only**: Force routing through specific AMM type

### Asset Pair Support

- **Native-Local**: Both XYK and TBC can handle (TBC works specifically with Native tokens)
- **Local-Local**: Only XYK supports arbitrary local asset pairs
- **Future**: Cross-chain assets through XCM integration

## Security Considerations

### Account Protection
- Respects Substrate's account reference counter system
- Conservative transaction sizing to prevent `NotExpendable` errors
- Follows Asset Hub production patterns

### Fee Collection Safety
- Router fees collected before swap execution
- Prevents fee bypass through failed swaps
- Supports both native and local asset fee collection

### Input Validation
- Path validation (length, asset existence)
- Amount validation (non-zero, sufficient balance)
- Slippage protection through minimum output enforcement

## Integration with Existing DEX Infrastructure

### Asset Hub Compatibility
- Uses same `AssetKind` enum structure
- Compatible with existing `pallet-asset-conversion` pools
- Follows Asset Hub parameter patterns for production safety

### XCM Integration Path
The current `AssetKind` enum provides clean upgrade path for cross-chain assets:

```rust
pub enum AssetKind {
    Native,
    Local(u32),
    // Future: Foreign(Location) for XCM v5 support
}
```

### Migration Strategy
1. **Phase 1**: Deploy router alongside existing `pallet-asset-conversion`
2. **Phase 2**: Add TBC support through trait implementation
3. **Phase 3**: Gradually migrate UI/frontend to use unified router
4. **Phase 4**: Add advanced features (multi-hop, split orders)

## Performance and Scalability

### Quote Aggregation
- Parallel quote collection from multiple AMMs
- Cached results for frequently traded pairs
- Configurable timeout for quote collection

### Gas Optimization
- Single transaction for multi-AMM routing
- Minimal overhead for single-AMM scenarios
- Efficient event emission for tracking

### Future Optimizations
- **Batch Operations**: Support for multiple swaps in single transaction
- **Liquidity Aggregation**: Virtual unified liquidity pool view
- **MEV Protection**: Transaction ordering and front-running protection

## Development Status

**Current Version**: 0.1.0 (MVP Implementation)

**Next Milestones**:
1. Complete XYK adapter integration
2. Add comprehensive test suite
3. Runtime integration and configuration
4. TBC adapter implementation
5. Multi-hop routing support

**Long-term Vision**:
- Universal DEX aggregator for Substrate ecosystem
- Cross-chain routing through XCM
- Advanced trading features (limit orders, DCA)
- Integration with other DeFi primitives (lending, staking)

## Contributing

The trait-based architecture makes it easy to contribute new AMM integrations or routing strategies. Key areas for contribution:

1. **New AMM Adapters**: Implement the `AMM` trait for other DEX protocols
2. **Routing Strategies**: Implement `RoutingStrategy` for different selection algorithms
3. **Fee Collectors**: Implement `FeeCollector` for different fee distribution mechanisms
4. **Testing**: Add comprehensive test coverage for new features
5. **Benchmarking**: Optimize performance and provide accurate weight calculations

## License

MIT-0
