# DEX Parachain Runtime

This runtime implements a comprehensive decentralized exchange (DEX) for Polkadot parachains, optimized for Omni Node architecture.

## Overview

The runtime provides automated market making capabilities with:

- **Multi-asset support** through pallet-assets
- **Liquidity pools** for any asset pair
- **Automated price discovery** via constant product formula
- **LP token rewards** for liquidity providers
- **Comprehensive trading APIs** for swaps and quotes

## Core Pallets

### Assets (Index 12)

- Create and manage fungible tokens
- Mint, burn, transfer operations
- Asset metadata and approval system
- Economic security through deposits

### Asset Conversion (Index 13)

- Uniswap V2-style automated market maker
- Create pools between any two assets
- Add/remove liquidity with LP tokens
- Execute swaps with slippage protection
- Price quotation for trading estimates

## Architecture

```
┌──────────────────┐    ┌──────────────────┐
│ pallet-assets    │    │ asset-conversion │
│                  │    │                  │
│ • Asset creation │    │ • Pool creation  │
│ • Token minting  │◄──►│ • Liquidity mgmt │
│ • Transfers      │    │ • Token swaps    │
│ • Metadata       │    │ • Price quotes   │
└──────────────────┘    └──────────────────┘
         │                       │
         └───────────┬───────────┘
                     ▼
            ┌─────────────────┐
            │ DEX Runtime     │
            │                 │
            │ • Native asset  │
            │ • LP tokens     │
            │ • Pool reserves │
            │ • Trading fees  │
            └─────────────────┘
```

## Asset Types

The runtime currently uses a simple `u32` type for `AssetKind` but is architected for future expansion to a comprehensive enum:

**Current Implementation:**

```rust
pub enum AssetKind {
    Native,     // Parachain's native token
    Local(u32), // Local assets created via pallet-assets
}
```

**Future Architecture (XCM v5 Ready):**

```rust
pub enum AssetKind {
    Native,            // Parachain's native token
    Local(u32),        // Local assets created via pallet-assets
    Foreign(Location), // Cross-chain assets via XCM v5
}
```

**Key Implementation Insights:**

- ✅ **Type Safety**: Enum prevents accidental mixing of asset types
- ✅ **Derive Traits**: `DecodeWithMemTracking` critical for runtime integration
- ✅ **Pool Ordering**: Native asset must be first in `WithFirstAsset` configuration
- ✅ **Balance Management**: Native tokens require careful `keep_alive` handling
- ✅ **Conversion Logic**: `NativeOrAssetIdConverter` bridges enum to pallet requirements

### Native Asset (AssetKind::Native)

- The parachain's native token (via Balances pallet)
- Used for transaction fees and governance
- Base asset for most trading pairs
- Internally represented as `AssetKind::Native` enum variant

### Local Assets (AssetKind::Local(u32))

- Created through pallet-assets with unique asset IDs
- Configurable metadata, supply, and permissions
- Can be paired with native or other assets
- Internally represented as `AssetKind::Local(asset_id)` enum variant

### Foreign Assets (Future: AssetKind::Foreign(Location))

- Cross-chain assets received via XCM v5 (planned feature)
- Will be identified by their XCM `Location` (multilocation)
- Will enable full interoperability with other parachains
- Architecture ready - add Foreign variant to existing AssetKind enum

## DEX Operations

### Pool Creation

```rust
AssetConversion::create_pool(
    origin,
    Box::new(AssetKind::Native),     // Native asset
    Box::new(AssetKind::Local(1))    // Local asset ID 1
)
```

### Adding Liquidity

```rust
AssetConversion::add_liquidity(
    origin,
    Box::new(AssetKind::Native),     // Native asset
    Box::new(AssetKind::Local(1)),   // Local asset ID 1
    amount1_desired,
    amount2_desired,
    amount1_min,
    amount2_min,
    mint_to
)
```

### Token Swaps

```rust
AssetConversion::swap_exact_tokens_for_tokens(
    origin,
    vec![
        Box::new(AssetKind::Native),     // Native asset
        Box::new(AssetKind::Local(1)),   // Local asset ID 1
        Box::new(AssetKind::Local(2))    // Another local asset
    ],
    amount_in,
    amount_out_min,
    send_to,
    keep_alive  // Set false for native token swaps
)
```

### Price Quotes

```rust
AssetConversion::quote_price_exact_tokens_for_tokens(
    AssetKind::Native,      // Native asset
    AssetKind::Local(1),    // Local asset ID 1
    amount,
    include_fee
)
```

## Configuration

### Key Parameters

- **LP Fee**: 0.3% (30 basis points)
- **Pool Setup Fee**: 0 (free pool creation)
- **Max Swap Path**: 4 hops maximum
- **Min Liquidity**: 100 units minimum

### Economic Security

- **Asset Deposit**: Minimal deposit for asset creation
- **Account Deposit**: Small deposit to maintain asset accounts
- **Withdrawal Fee**: 0% (no fee for removing liquidity)

## Testing

The runtime includes comprehensive tests:

```bash
# Run all DEX tests
cargo test -p parachain-template-runtime asset_conversion_tests

# Run specific test
cargo test create_asset_and_pool_works
```

### Test Coverage

- ✅ Asset creation and minting
- ✅ Pool creation and management
- ✅ Liquidity provision/removal
- ✅ Token swapping mechanics
- ✅ Price quotation accuracy
- ✅ Error handling and edge cases
- ✅ AssetKind enum type safety
- ✅ Native-Local and Local-Local pool combinations
- ✅ Balance management for native tokens

## Integration with Omni Node

This runtime is optimized for Omni Node deployment:

1. **Build runtime WASM**:

   ```bash
   cargo build --profile production
   ```

2. **Generate chain spec**:

   ```bash
   chain-spec-builder create --relay-chain "rococo-local" --para-id 1000 --runtime \
       target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm \
       named-preset development
   ```

3. **Run with Omni Node**:
   ```bash
   polkadot-omni-node --chain <chain_spec.json> --dev --dev-block-time 1000
   ```

## Development Workflow

### Quick Testing with Chopsticks

```bash
# Build and generate raw chain spec
cargo build --profile production
chain-spec-builder create --raw-storage --relay-chain "rococo-local" --para-id 1000 \
    --runtime target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm \
    named-preset development

# Start chopsticks for rapid iteration
npx @acala-network/chopsticks@latest --chain-spec <chain_spec.json>
```

### Multi-Node Testing

```bash
# Use zombienet for realistic parachain testing
zombienet --provider native spawn zombienet-omni-node.toml
```

## API Integration

Connect via Polkadot-JS Apps or custom frontends:

```javascript
// Example: Create asset and pool
const api = await ApiPromise.create({
  provider: new WsProvider("ws://localhost:9988"),
});

// Create new asset
await api.tx.assets.create(1, alice.address, 1000).signAndSend(alice);

// Create trading pool between native and local asset (1)
// Note: API accepts u32 values that convert to AssetKind internally
await api.tx.assetConversion.createPool(0, 1).signAndSend(alice);

// Add liquidity
await api.tx.assetConversion
  .addLiquidity(
    0, // Native asset (converts to AssetKind::Native)
    1, // Local asset ID 1 (converts to AssetKind::Local(1))
    1000000,
    2000000,
    1000000,
    2000000,
    alice.address,
  )
  .signAndSend(alice);

// Swap native for local asset
await api.tx.assetConversion
  .swapExactTokensForTokens(
    [0, 1], // Path: native -> local asset 1
    100000,
    95000,
    bob.address,
    false, // Critical: set false for native token swaps
  )
  .signAndSend(bob);
```

**AssetKind Conversion Notes:**

- API accepts `u32` values for backward compatibility
- `0` → `AssetKind::Native` internally
- `1+` → `AssetKind::Local(asset_id)` internally
- `NativeOrAssetIdConverter` handles the conversion

## Future Enhancements

- **XCM Integration**: 🔄 Ready - Add Foreign(Location) variant to existing AssetKind enum
- **Cross-Chain Trading**: Enhanced routing through foreign asset pools
- **Advanced Features**: Limit orders, concentrated liquidity, flash loans
- **Governance**: Parameter adjustment through on-chain governance
- **Analytics**: Enhanced pool metrics and trading statistics
- **Test Improvements**: Complete coverage for existential deposit edge cases

## Security Considerations

- **Economic Security**: All operations require appropriate deposits
- **Slippage Protection**: Minimum output amounts prevent MEV attacks
- **Path Validation**: Multi-hop swaps are validated for circular references
- **Overflow Protection**: Safe math operations throughout
- **Type Safety**: AssetKind enum prevents asset type confusion at compile time
- **Balance Protection**: Native token handling preserves existential deposits

For detailed implementation, see the source code in `src/configs/assets_config.rs`.
