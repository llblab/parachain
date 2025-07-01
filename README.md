# DEX Parachain - Omni Node Optimized

> A DEX-focused [parachain](https://wiki.polkadot.network/docs/learn-parachains) optimized for Omni Node architecture.
>
> Features comprehensive asset management with XCM v5 ready architecture and automated market making capabilities.

## Table of Contents

- [Features](#features)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Asset Architecture](#asset-architecture)
- [Running with Omni Node](#running-with-omni-node)
- [DEX Functionality](#dex-functionality)
- [Runtime Development](#runtime-development)
- [Testing](#testing)
- [Contributing](#contributing)

## Features

- 🏪 **Complete DEX Infrastructure**: Asset creation, liquidity pools, and automated market making
- 🔄 **Multi-Asset Support**: Full fungible asset support with XCM v5 ready architecture
- 🌐 **Omni Node Optimized**: Streamlined architecture focused on runtime development
- ⚡ **Modern FRAME Patterns**: Latest Polkadot SDK features and conventions
- 🧪 **Comprehensive Testing**: Unit tests, integration tests, and benchmarking ready
- 🌍 **Cross-Chain Ready**: AssetKind enum architecture supporting native, local, and foreign assets

## Project Structure

This Omni Node optimized parachain consists of:

- 🧮 **[Runtime](./runtime/)** - Core DEX logic with Assets and Asset Conversion pallets
- 🎨 **[Pallets](./pallets/)** - Custom pallet demonstrating modern FRAME patterns
- 📋 **[Tests](./runtime/src/tests/)** - Comprehensive test suite for DEX functionality
- 🔧 **[Configs](./runtime/src/configs/)** - Modular runtime configuration

## Getting Started

- 🦀 The template is using the Rust language.

- 👉 Check the
  [Rust installation instructions](https://www.rust-lang.org/tools/install) for your system.

- 🛠️ Depending on your operating system and Rust version, there might be additional
  packages required to compile this template - please take note of the Rust compiler output.

### Prerequisites

1. **Install Rust and dependencies:**

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   rustup target add wasm32-unknown-unknown
   ```

2. **Download polkadot-omni-node:**

   ```sh
   # Option 1: Use our convenience script
   ./scripts/download-omni-node.sh

   # Option 2: Manual download
   curl -L -o polkadot-omni-node https://github.com/paritytech/polkadot-sdk/releases/latest/download/polkadot-omni-node
   chmod +x polkadot-omni-node
   ```

3. **Build the runtime:**
   ```sh
   cargo build --release --manifest-path runtime/Cargo.toml
   ```

## Asset Architecture

The runtime uses a sophisticated asset type system designed for current functionality and future XCM v5 expansion:

### Current Implementation

```rust
pub enum AssetKind {
    Native,     // Parachain's native token
    Local(u32), // Local assets with asset IDs
}
```

- **Native Asset**: Parachain's native token for fees and governance
- **Local Assets**: Custom tokens created via pallet-assets with unique IDs

### Future XCM v5 Architecture

```rust
pub enum AssetKind {
    Native,            // Parachain's native token
    Local(u32),        // Local assets with asset IDs
    Foreign(Location), // Cross-chain assets via XCM v5
}
```

**Key Implementation Insights:**

- ✅ **Type Safety Achieved**: Enum prevents mixing asset types accidentally
- ✅ **Derive Traits Critical**: `DecodeWithMemTracking` required for runtime integration
- ✅ **Pool Ordering Matters**: Native asset must be first in `WithFirstAsset` configuration
- ✅ **Balance Management**: Native tokens require careful `keep_alive` handling
- ✅ **Test Architecture**: Enum variants enable Native-Local and Local-Local pool combinations

### Asset Capabilities

- **Asset Creation**: Issue fungible tokens with configurable parameters
- **Economic Security**: Deposits required for asset creation and account maintenance
- **Metadata Support**: Names, symbols, decimals, and custom metadata
- **Cross-Chain Ready**: Architecture supports XCM v5 Location for foreign assets

## Starting a Development Chain

The parachain template relies on a hardcoded parachain id which is defined in the runtime code
and referenced throughout the contents of this file as `{{PARACHAIN_ID}}`. Please replace
any command or file referencing this placeholder with the value of the `PARACHAIN_ID` constant:

```rust,ignore
pub const PARACHAIN_ID: u32 = 1000;
```

## Running with Omni Node

This parachain is optimized for [Omni Node](https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/omni_node/index.html) architecture, providing a streamlined development experience focused on runtime logic.

### Install `polkadot-omni-node`

Please see the installation section at [`crates.io/omni-node`](https://crates.io/crates/polkadot-omni-node).

### Build `parachain-template-runtime`

```sh
cargo build --profile production
```

### Install `staging-chain-spec-builder`

Please see the installation section at [`crates.io/staging-chain-spec-builder`](https://crates.io/crates/staging-chain-spec-builder).

### Use `chain-spec-builder` to generate the `chain_spec.json` file

```sh
chain-spec-builder create --relay-chain "rococo-local" --para-id {{PARACHAIN_ID}} --runtime \
    target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm named-preset development
```

**Note**: the `relay-chain` and `para-id` flags are mandatory information required by
Omni Node, and for parachain template case the value for `para-id` must be set to `{{PARACHAIN_ID}}`, since this
is also the value injected through [ParachainInfo](https://docs.rs/staging-parachain-info/0.17.0/staging_parachain_info/)
pallet into the `parachain-template-runtime`'s storage. The `relay-chain` value is set in accordance
with the relay chain ID where this instantiation of parachain-template will connect to.

### Run Omni Node

Start Omni Node with the generated chain spec. We'll start it in development mode (without a relay chain config), producing
and finalizing blocks based on manual seal, configured below to seal a block with each second.

```bash
polkadot-omni-node --chain <path/to/chain_spec.json> --dev --dev-block-time 1000
```

However, such a setup is not close to what would run in production, and for that we need to setup a local
relay chain network that will help with the block finalization. In this guide we'll setup a local relay chain
as well. We'll not do it manually, by starting one node at a time, but we'll use [zombienet](https://paritytech.github.io/zombienet/intro.html).

Follow through the next section for more details on how to do it.

### Multi-Node Testing with Zombienet

For realistic parachain testing, use the included Zombienet configuration:

#### Relay chain prerequisites

Download the `polkadot` (and the accompanying `polkadot-prepare-worker` and `polkadot-execute-worker`) binaries from
[Polkadot SDK releases](https://github.com/paritytech/polkadot-sdk/releases). Then expose them on `PATH` like so:

```sh
export PATH="$PATH:<path/to/binaries>"
```

### Update `zombienet-omni-node.toml` with a valid chain spec path

To simplify the process of using the parachain-template with zombienet and Omni Node, we've added a pre-configured
development chain spec (dev_chain_spec.json) to the parachain template. The zombienet-omni-node.toml file of this
template points to it, but you can update it to an updated chain spec generated on your machine. To generate a
chain spec refer to [staging-chain-spec-builder](https://crates.io/crates/staging-chain-spec-builder)

Then make the changes in the network specification like so:

```toml
# ...
[[parachains]]
id = "<PARACHAIN_ID>"
chain_spec_path = "<TO BE UPDATED WITH A VALID PATH>"
# ...
```

### Run Omni Node Test

```sh
zombienet --provider native spawn zombienet-omni-node.toml
```

## DEX Functionality

This parachain provides comprehensive DEX capabilities built on Uniswap V2 logic:

### Core DEX Features

- **Liquidity Pools**: Create trading pairs between any asset types (native + local assets)
- **Automated Market Making**: Constant product formula for price discovery
- **Add/Remove Liquidity**: Provide liquidity and receive LP tokens
- **Token Swaps**: Execute trades with slippage protection
- **Price Quotes**: Get accurate swap estimates via runtime APIs
- **Multi-Hop Swaps**: Route trades through multiple pools (up to 4 hops)

### Asset Operations

```javascript
// Connect to parachain via Omni Node
const api = await ApiPromise.create({
  provider: new WsProvider("ws://localhost:9988"),
});

// Create a new local asset (AssetKind: Local(1))
await api.tx.assets.create(1, alice.address, 1000).signAndSend(alice);

// Mint tokens to account
await api.tx.assets.mint(1, alice.address, 1000000000).signAndSend(alice);
```

**AssetKind Enum Usage:**

- **Native**: Represented internally as `AssetKind::Native`
- **Local Assets**: Represented as `AssetKind::Local(asset_id)`
- **Pool Creation**: Always use Native as first asset for `WithFirstAsset` compliance

### Pool Operations

```javascript
// Create liquidity pool (Native <-> Local(1))
// Note: Native asset automatically converted to AssetKind::Native internally
await api.tx.assetConversion.createPool(0, 1).signAndSend(alice);

// Add liquidity to pool
await api.tx.assetConversion
  .addLiquidity(
    0, // Native asset (AssetKind::Native)
    1, // Local asset ID 1 (AssetKind::Local(1))
    1000000000, // Native amount desired
    2000000000, // Local asset amount desired
    1000000000, // Native amount min
    2000000000, // Local asset amount min
    alice.address,
  )
  .signAndSend(alice);

// Execute swap: Native -> Local Asset
await api.tx.assetConversion
  .swapExactTokensForTokens(
    [0, 1], // Path: Native -> Local(1)
    100000000, // Amount in
    95000000, // Min amount out
    bob.address, // Recipient
    false, // Keep alive (critical for native token swaps)
  )
  .signAndSend(bob);
```

**Critical Implementation Notes:**

- **Pool Ordering**: Native asset must always be first due to `WithFirstAsset` constraint
- **Keep Alive**: Set to `false` for native token swaps to avoid `NotExpendable` errors
- **Balance Requirements**: Ensure sufficient native balance for existential deposit

### Runtime API Queries

```javascript
// Get pool reserves (Native <-> Local(1))
const reserves = await api.rpc.assetConversionApi.getReserves(0, 1);

// Quote swap price
const quote = await api.rpc.assetConversionApi.quotePriceExactTokensForTokens(
  0, // Asset in (Native -> AssetKind::Native)
  1, // Asset out (Local 1 -> AssetKind::Local(1))
  100000000, // Amount in
  false, // Include fee
);
```

**AssetKind Conversion Logic:**

- API accepts `u32` values that get converted internally
- `0` → `AssetKind::Native`
- `1+` → `AssetKind::Local(asset_id)`
- Runtime handles conversion via `NativeOrAssetIdConverter`

### Omni Node Integration

Connect via [Polkadot-JS Apps](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9988) to interact with the DEX:

- 📊 Monitor pool reserves and liquidity
- 💱 Execute swaps through the extrinsics tab
- 📈 View events and transaction history
- 🔍 Query runtime APIs for price quotes

### Economic Parameters

- **LP Fee**: 0.3% (30 basis points) on all swaps
- **Pool Setup Fee**: 0 (free pool creation)
- **Max Swap Path**: 4 hops maximum
- **Min Liquidity**: 100 units minimum for new pools

## Runtime Development

### Omni Node Focused Development

This parachain is optimized for [Omni Node](https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/omni_node/index.html) architecture, eliminating node complexity to focus purely on runtime logic:

#### Quick Development with Chopsticks

For rapid runtime iteration:

```sh
# Build runtime WASM
cargo build --profile production

# Generate raw chain spec
chain-spec-builder create --raw-storage --relay-chain "rococo-local" --para-id 1000 \
    --runtime target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm \
    named-preset development

# Start chopsticks for instant testing
npx @acala-network/chopsticks@latest --chain-spec <path/to/chain_spec.json>
```

#### Development Mode with Omni Node

For standalone development without relay chain:

```bash
polkadot-omni-node --chain <path/to/chain_spec.json> --dev --dev-block-time 1000
```

#### Production-Ready Testing

For realistic parachain environment with Zombienet:

```bash
# Ensure polkadot binaries are in PATH
export PATH="$PATH:<path/to/polkadot/binaries>"

# Launch multi-node test network
zombienet --provider native spawn zombienet-omni-node.toml
```

### Runtime Configuration

The runtime is modularly configured in `runtime/src/configs/`:

- **assets_config.rs**: Assets and Asset Conversion pallet configurations
- **xcm_config.rs**: XCM configuration for cross-chain operations
- **mod.rs**: Main runtime assembly and pallet integration

### AssetKind Integration

The AssetKind enum is now implemented with type safety:

```rust
// Current: Enum implementation
pub enum AssetKind {
    Native,           // Parachain's native token
    Local(u32),      // Local assets with IDs
}

// Future: XCM v5 ready
pub enum AssetKind {
    Native,
    Local(u32),
    Foreign(Location), // Cross-chain assets
}
```

**Implementation Insights:**

- **Derive Traits**: `Encode, Decode, MaxEncodedLen, TypeInfo, DecodeWithMemTracking` all required
- **Converter Pattern**: `NativeOrAssetIdConverter` handles enum → Either<(), u32> conversion
- **Pool Constraints**: `WithFirstAsset` requires Native as first asset in pools
- **Balance Management**: Native tokens need careful handling for existential deposits

The architecture enables type-safe asset operations while maintaining API compatibility.

## Testing

### Comprehensive Test Suite

The runtime includes extensive testing for all DEX functionality:

```sh
# Run all tests
cargo test --workspace

# Run DEX-specific tests
cargo test -p parachain-template-runtime asset_conversion_tests

# Run with detailed output
cargo test -p parachain-template-runtime asset_conversion_tests -- --nocapture

# Run specific test
cargo test create_asset_and_pool_works
```

### Test Coverage

- ✅ **Asset Management**: Creation, minting, and metadata operations
- ✅ **Pool Operations**: Pool creation, liquidity provision/removal
- ✅ **Trading Mechanics**: Exact token swaps, price quotations
- ✅ **Error Handling**: Duplicate pools, insufficient liquidity, slippage
- ✅ **Edge Cases**: Minimum liquidity, maximum path length
- ✅ **Economic Security**: Deposit requirements and account management

### Integration Testing

Test with real Omni Node environment:

```sh
# Build and start Omni Node
cargo build --profile production
polkadot-omni-node --chain <chain_spec.json> --dev

# Connect via Polkadot-JS Apps
# Execute test transactions through UI
```

### Benchmarking

Runtime includes benchmarking setup for weight optimization:

```sh
# Run benchmarks (when enabled)
cargo test --features runtime-benchmarks
```

## Contributing

This DEX parachain demonstrates modern Polkadot SDK patterns optimized for Omni Node architecture:

### Key Design Principles

- 🔧 **Runtime Focus**: Streamlined for runtime development without node complexity
- 🏗️ **Modern Architecture**: Latest FRAME patterns and polkadot-sdk unified imports
- 🧪 **Test-Driven**: Comprehensive testing for DeFi functionality
- 📚 **Well-Documented**: Clear examples and patterns for DEX development
- 🌍 **XCM Ready**: AssetKind enum implemented with Foreign(Location) extension path
- ⚡ **Omni Node Optimized**: Leverages polkadot-omni-node for simplified deployment
- 🔒 **Type Safety**: AssetKind enum prevents asset type confusion at compile time

### Development Workflow

1. **Focus on Runtime**: No node development required - use Omni Node binary
2. **Iterative Development**: Use chopsticks for rapid testing
3. **Realistic Testing**: Use zombienet for multi-node validation
4. **Production Ready**: Deploy with standard Omni Node infrastructure

### Architecture Highlights

- **Asset Types**: AssetKind enum with Native and Local(u32) variants, Foreign(Location) ready
- **Modular Configuration**: Separated pallet configs in `runtime/src/configs/`
- **Auto-Weight Generation**: Leverages `WeightInfo = ()` for automatic weight calculation
- **Economic Security**: Proper deposit requirements and error handling
- **Type Safety**: Compile-time asset type validation prevents runtime errors
- **Conversion Layer**: `NativeOrAssetIdConverter` bridges enum types to pallet requirements

### Critical Implementation Learnings

- **Derive Completeness**: All substrate traits required for runtime integration
- **Pool Architecture**: `WithFirstAsset` constraint shapes pool creation patterns
- **Balance Handling**: Native token `keep_alive` parameter critical for swap success
- **Test Strategy**: Enum enables comprehensive Native-Local and Local-Local combinations

### Future Enhancements

- 🔄 **XCM v5 Integration**: Add Foreign(Location) variant to existing AssetKind enum
- 🌉 **Cross-Chain Pools**: Trading between assets from different parachains
- 📊 **Advanced Features**: DCA, governance, concentrated liquidity
- 🔍 **Enhanced Testing**: Complete test coverage for existential deposit edge cases
- ⚖️ **Balance Optimization**: Improved native token handling for complex swap scenarios

For questions and discussions:

- 🐛 [GitHub Issues](https://github.com/paritytech/polkadot-sdk/issues)
- 💬 [Substrate StackExchange](https://substrate.stackexchange.com/)
- 👥 [Polkadot Discord](https://polkadot-discord.w3f.tools/)
- 📱 [Substrate Telegram](https://t.me/substratedevs)
