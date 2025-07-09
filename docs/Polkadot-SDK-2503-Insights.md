# Polkadot SDK 2503: Migration Insights and Best Practices

## Overview

This document captures critical insights and patterns discovered during the migration of a parachain runtime and custom pallets from legacy Substrate patterns to the modern Polkadot SDK 2503 architecture. These insights go beyond basic documentation to provide deeper understanding of the architectural shifts and subtle patterns that enable successful migrations.

## Key Architectural Shifts

### 1. From `#[frame_support::runtime]` to `construct_runtime!`

**Legacy Pattern:**
```rust
#[frame_support::runtime]
pub mod runtime {
    // Runtime definition
}
```

**Modern SDK 2503 Pattern:**
```rust
construct_runtime!(
    pub enum Runtime {
        System: frame_system = 0,
        Balances: pallet_balances = 10,
        CustomPallet: pallet_custom = 20,
    }
);
```

**Critical Insight:** The `construct_runtime!` macro generates types dynamically, requiring careful handling of type exports and imports. The macro creates `RuntimeCall`, `RuntimeEvent`, `RuntimeOrigin`, and other types that must be properly re-exported for use in other modules.

### 2. Pallet Structure Evolution

**Legacy Pattern:**
```rust
#[frame_support::pallet]
pub mod pallet {
    // Pallet definition
}
```

**Modern SDK 2503 Pattern:**
```rust
#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Configuration
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // Events, errors, calls, etc.
}
```

**Critical Insight:** The `dev_mode` flag enables simplified development patterns but requires specific macro structure. The pallet must properly export its `Config` trait and implement required macro patterns for `construct_runtime!` compatibility.

## Import Strategy Evolution

### Legacy Imports
```rust
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::Get,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::*;
```

### Modern SDK 2503 Imports
```rust
use frame::prelude::*;
use polkadot_sdk::{pallet_asset_conversion, pallet_balances};
```

**Critical Insight:** The `frame::prelude::*` provides a unified import surface that includes all necessary types and traits. This reduces import complexity but requires understanding what's included in the prelude.

## Runtime Type Export Patterns

### The Type Export Challenge

After `construct_runtime!`, the generated types need to be properly exported:

```rust
construct_runtime!(
    pub enum Runtime {
        // Pallet declarations
    }
);

// CORRECT: Export pallet instances
pub use {
    System, Balances, Assets, CustomPallet,
};

// CORRECT: Export runtime types
pub type RuntimeCall = <Runtime as frame_system::Config>::RuntimeCall;
pub type RuntimeEvent = <Runtime as frame_system::Config>::RuntimeEvent;
pub type RuntimeOrigin = <Runtime as frame_system::Config>::RuntimeOrigin;

// INCORRECT: Recursive definitions
// pub type RuntimeCall = RuntimeCall; // This creates infinite recursion!
```

**Critical Insight:** The `construct_runtime!` macro creates a complex type hierarchy. You must export both the pallet instances (for configuration) and the runtime types (for APIs and external use).

## Pallet Integration Patterns

### Asset Conversion Integration

**Pattern Discovery:**
```rust
// In pallet config
type AssetConversion: pallet_asset_conversion::Config<
    AssetKind = Self::AssetKind,
    Balance = Self::Balance,
    AccountId = Self::AccountId,
>;

// In runtime config
impl pallet_custom::Config for Runtime {
    type AssetConversion = AssetConversion; // Reference to pallet instance
}

// In pallet implementation
pallet_asset_conversion::Pallet::<T::AssetConversion>::swap_exact_tokens_for_tokens(
    <T::AssetConversion as frame_system::Config>::RuntimeOrigin::from(
        frame_system::RawOrigin::Signed(who.clone())
    ),
    path,
    amount_in,
    amount_out_min,
    send_to,
    keep_alive,
)
```

**Critical Insight:** Inter-pallet communication requires careful origin type management. The calling pallet must convert its origin to the target pallet's expected origin type.

## Dependency Management Evolution

### Cargo.toml Patterns

**Legacy Pattern:**
```toml
[dependencies]
frame-support = { version = "4.0.0", default-features = false }
frame-system = { version = "4.0.0", default-features = false }
pallet-balances = { version = "4.0.0", default-features = false }
```

**Modern SDK 2503 Pattern:**
```toml
[dependencies]
frame = { workspace = true, features = ["runtime"] }
polkadot-sdk = { workspace = true, default-features = false, features = [
    "pallet-asset-conversion",
    "pallet-balances",
]}
```

**Critical Insight:** The workspace-based dependency management requires careful feature flag management. The `polkadot-sdk` crate provides access to all pallets through feature flags, reducing dependency complexity.

## Common Migration Pitfalls

### 1. The `tt_default_parts` Issue

**Problem:** Custom pallets may not be compatible with `construct_runtime!` if they don't implement required macro patterns.

**Solution:**
```rust
#[frame::pallet(dev_mode)]
pub mod pallet {
    // Ensure proper pallet structure
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // All required sections must be present
    #[pallet::config]
    pub trait Config: frame_system::Config { /* ... */ }

    // Events, errors, calls as needed
}
```

### 2. Circular Import Issues

**Problem:** Runtime modules importing from each other can create circular dependencies.

**Solution:** Use proper module hierarchy and re-export patterns:
```rust
// In runtime/src/lib.rs
mod configs;
pub use configs::*;

// In runtime/src/configs/mod.rs
mod pallet_config;
pub use pallet_config::*;
```

### 3. Origin Type Mismatches

**Problem:** Different pallets expect different origin types.

**Solution:** Proper origin conversion patterns:
```rust
// Convert from pallet origin to target pallet origin
let target_origin = <TargetPallet as frame_system::Config>::RuntimeOrigin::from(
    frame_system::RawOrigin::Signed(who.clone())
);
```

## Testing Strategy Evolution

### Legacy Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::assert_ok;
    use sp_runtime::testing::Header;

    // Complex mock runtime setup
}
```

### Modern SDK 2503 Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame::testing_prelude::*;

    construct_runtime!(
        pub enum TestRuntime {
            System: frame_system,
            Pallet: crate::pallet,
        }
    );

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for TestRuntime {
        type Block = MockBlock<TestRuntime>;
        type AccountId = u64;
    }
}
```

**Critical Insight:** The testing framework provides simplified mock runtime patterns that reduce boilerplate significantly.

## Performance and Optimization Insights

### Weight Management
```rust
// Modern weight patterns
#[pallet::weight(T::WeightInfo::operation_name())]
pub fn operation_name(origin: OriginFor<T>) -> DispatchResult {
    // Implementation
}

// Weight info trait
pub trait WeightInfo {
    fn operation_name() -> Weight;
}
```

### Storage Optimization
```rust
// Efficient storage patterns
#[pallet::storage]
pub type OptimizedStorage<T: Config> = StorageMap<
    _,
    Blake2_128Concat,  // Efficient hasher
    T::AccountId,
    Balance,
    OptionQuery,       // Explicit query type
>;
```

## Best Practices Summary

1. **Start with Examples:** Always reference official SDK 2503 examples before migration
2. **Incremental Migration:** Migrate one pallet at a time, testing at each step
3. **Type Safety First:** Ensure all type constraints are properly defined
4. **Test Early:** Set up testing infrastructure before implementing complex logic
5. **Documentation:** Document all configuration choices and architectural decisions

## Migration Checklist

- [ ] Update Cargo.toml to use workspace dependencies
- [ ] Convert pallet to `#[frame::pallet(dev_mode)]`
- [ ] Update imports to use `frame::prelude::*`
- [ ] Convert runtime to `construct_runtime!`
- [ ] Fix type exports and re-exports
- [ ] Update configuration patterns
- [ ] Test pallet integration
- [ ] Update API implementations
- [ ] Verify benchmarking compatibility
- [ ] Document changes and patterns

## Future Evolution Considerations

The SDK 2503 represents a significant architectural shift towards:
- Simplified development patterns
- Better type safety
- Improved ergonomics
- Unified dependency management

Future migrations should consider:
- Continued evolution of the `frame` prelude
- Potential changes in macro patterns
- Performance optimization opportunities
- Integration with upcoming Substrate features

---

**Note:** This document reflects insights gained during practical migration work. Patterns may evolve as the SDK matures. Always consult the latest official documentation for authoritative guidance.
