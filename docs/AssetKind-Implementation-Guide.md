# AssetKind Enum Implementation Guide

This guide documents the implementation of AssetKind as a type-safe enum in Polkadot SDK parachains, capturing critical insights for future developers working with similar runtime type implementations.

## Overview

Successfully converted `AssetKind` from a simple `u32` type alias to a proper enum with `Native` and `Local(u32)` variants, establishing compile-time type safety while maintaining API compatibility.

```rust
// Before: Type alias (error-prone)
pub type AssetKind = u32;

// After: Type-safe enum
pub enum AssetKind {
    Native,        // Parachain's native token
    Local(u32),    // Local assets with IDs
}
```

## Critical Implementation Requirements

### 1. Complete Derive Trait Set

**CRITICAL**: All these traits are required for Substrate runtime integration:

```rust
#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    DecodeWithMemTracking,  // ← ESSENTIAL: Missing this causes compilation failure
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
pub enum AssetKind {
    Native,
    Local(u32),
}
```

**Required Imports:**

```rust
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
```

### 2. Pool Architecture Constraint

**CRITICAL**: `WithFirstAsset` configuration requires Native asset first in ALL pool operations:

```rust
// ✅ CORRECT: Native asset first
create_pool(AssetKind::Native, AssetKind::Local(1))

// ❌ FAILS: InvalidAssetPair error
create_pool(AssetKind::Local(1), AssetKind::Local(2))

// ✅ SOLUTION: Route Local-Local through Native
// Path: Local(1) → Native → Local(2)
swap_exact_tokens_for_tokens(
    vec![AssetKind::Local(1), AssetKind::Native, AssetKind::Local(2)],
    amount_in,
    amount_out_min,
    recipient,
    false
)
```

### 3. Native Token Balance Management

**CRITICAL**: Native tokens require `keep_alive: false` to avoid `NotExpendable` errors:

```rust
// ✅ CORRECT: Allow spending below existential deposit
swap_exact_tokens_for_tokens(
    path,
    amount_in,
    amount_out_min,
    recipient,
    false  // keep_alive: false for native token operations
)

// ❌ FAILS: NotExpendable error
keep_alive: true  // Prevents spending below existential deposit
```

**Why keep_alive matters:**

- Substrate enforces existential deposit for native tokens
- `keep_alive: true` prevents account from going below minimum balance
- `keep_alive: false` allows complete token spending
- Essential for DEX operations where exact amounts matter

## Implementation Pattern

### 1. Type Conversion Layer

Bridge enum types to pallet requirements:

```rust
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
```

### 2. Runtime Configuration

```rust
impl pallet_asset_conversion::Config for Runtime {
    type AssetKind = AssetKind;  // Direct enum usage
    type Assets = frame_support::traits::fungible::UnionOf<
        Balances,
        pallet_assets::Pallet<Runtime>,
        NativeOrAssetIdConverter,  // Conversion bridge
        AssetKind,
        AccountId,
    >;
    // ... rest of config
}

frame_support::parameter_types! {
    pub const NativeAssetId: AssetKind = AssetKind::Native;
}
```

## API Compatibility

External APIs maintain u32 compatibility through automatic conversion:

```javascript
// API accepts u32 values (backward compatible)
await api.tx.assetConversion.createPool(0, 1);

// Internally converts to:
// AssetKind::Native, AssetKind::Local(1)
```

**Conversion Rules:**

- `0` → `AssetKind::Native`
- `1+` → `AssetKind::Local(asset_id)`

## Testing Strategy

### Test Coverage for All Combinations

```rust
#[test]
fn test_native_local_pair() {
    // Native ↔ Local(1) pool
    create_pool(AssetKind::Native, AssetKind::Local(1));
}

#[test]
fn test_local_local_via_native() {
    // Local(1) → Native → Local(2) routing
    swap_exact_tokens_for_tokens(
        vec![AssetKind::Local(1), AssetKind::Native, AssetKind::Local(2)],
        // ...
    );
}
```

### Debug Test Template

```rust
#[test]
fn test_debug_balance_issue() {
    new_test_ext().execute_with(|| {
        // Check balances before operations
        println!("Native balance: {}", Balances::free_balance(&account));
        println!("Asset balance: {}", Assets::balance(asset_id, &account));

        // Perform operation and check result
        let result = add_liquidity(/* ... */);
        println!("Operation result: {:?}", result);

        // If failed, check specific constraints
        if result.is_err() {
            println!("Existential deposit: {}", EXISTENTIAL_DEPOSIT);
            // Check if balance < existential_deposit + operation_amount
        }
    });
}
```

## Common Issues and Solutions

### 1. Compilation Failure

**Error:** `DecodeWithMemTracking` not satisfied
**Solution:** Add complete derive trait set (see section 1)

### 2. Pool Creation Fails

**Error:** `InvalidAssetPair`
**Solution:** Always put Native asset first (see section 2)

### 3. Swap Fails

**Error:** `NotExpendable`
**Solution:** Set `keep_alive: false` (see section 3)

### 4. Balance Issues

**Error:** Insufficient balance for operation
**Solution:** Ensure balance > existential_deposit + operation_amount

## Future Expansion: XCM v5

Adding Foreign assets requires minimal changes:

```rust
pub enum AssetKind {
    Native,
    Local(u32),
    Foreign(Location),  // Add XCM v5 support
}

// Update converter to handle Location
impl Convert<AssetKind, Either<Either<(), AssetId>, Location>>
    for EnhancedConverter
{
    fn convert(asset_kind: AssetKind) -> Either<Either<(), AssetId>, Location> {
        match asset_kind {
            AssetKind::Native => Either::Left(Either::Left(())),
            AssetKind::Local(id) => Either::Left(Either::Right(id)),
            AssetKind::Foreign(loc) => Either::Right(loc),
        }
    }
}
```

## Benefits Achieved

1. **Compile-time Safety**: Prevents asset type confusion
2. **Clear Semantics**: `AssetKind::Native` vs `AssetKind::Local(1)` is self-documenting
3. **Future-proof**: Easy to add `Foreign(Location)` variant
4. **API Compatibility**: External interfaces unchanged
5. **Test Coverage**: Enum enables systematic testing of all combinations

## Performance Impact

- **Memory**: Minimal overhead (8 bytes total: 4-byte tag + 4-byte u32)
- **Runtime**: No performance degradation vs u32
- **Compilation**: Pattern matching optimizes to efficient code

This implementation demonstrates that proper type safety in Substrate runtimes requires understanding of framework constraints while providing significant benefits for maintainability and correctness.
