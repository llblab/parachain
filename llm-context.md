# Project Context

### Meta-Protocol Principles

This document is a **living protocol** designed for continuous, intelligent self-improvement. Its core principles are:

1.  **Decreasing Abstraction**: Always structure information from the general to the specific.
2.  **Mandatory Self-Improvement**: Every task must end with an update to this document.
3.  **Protocol Evolution**: The rules themselves, especially the Task Completion Protocol, should be improved if a more efficient workflow is discovered.
4.  **Knowledge Consolidation**: Important insights from rotated history entries must be preserved in permanent sections to prevent knowledge loss.
5.  **Non-Duplication**: Information should exist in only one authoritative section to maintain consistency and avoid confusion.

---

### 1. Overall Concept

- A Polkadot SDK parachain template project that demonstrates modern pallet development patterns and serves as a foundation for building custom parachains, with Omni Node as the primary deployment architecture and sophisticated development workflows optimized for runtime-focused development.

---

### 2. Core Entities

- **Parachain Runtime**: The core runtime that aggregates pallets and defines the blockchain logic with configurable parachain ID
- **Assets Pallet**: Fungible asset management system with full create/transfer/metadata capabilities (index 12)
- **Asset Conversion Pallet**: Uniswap V2-like DEX functionality for automated market making and asset swapping (index 13, configured with AssetKind enum)
- **DEX Implementation**: Production-ready DEX with AssetKind enum supporting Native and Local(u32) variants, with fully operational Native-Local liquidity provision demonstrating mastery of Substrate's protective architecture
- **DEX Router Implementation**: Production-ready trait-based DEX Router pallet with comprehensive integration testing and dual fee structure implementation, featuring multi-AMM aggregation architecture with Asset Conversion integration, tokenomics-compliant fee structure (0.2% router fee for buyback + 0.3% XYK pool fee = 0.5% total user cost), comprehensive error handling with proper balance constraints (AtLeast32BitUnsigned + Saturating + CheckedSub + PartialOrd), and 11 passing integration tests covering swap execution, fee mechanisms, error handling, and access control
- **XCM v5 Architecture**: AssetKind implemented as enum with Native and Local(u32) variants, documented with Foreign(Location) extension path for future cross-chain asset interoperability
- **Node**: The blockchain client implementation for running the parachain
- **Pallets**: Modular runtime components that implement specific blockchain functionality
- **Omni Node**: Primary deployment architecture using polkadot-omni-node for streamlined runtime execution and development
- **Chain Spec**: JSON configuration files that define network parameters and genesis state
- **Zombienet**: Multi-node test network orchestration for realistic parachain testing environments
- **Integration Complexity Spectrum**: From simple (Assets) → intermediate (Asset Conversion with enum) → complex (Asset Conversion with XCM Location) - understanding this spectrum enables strategic simplification
- **Substrate's Layered Protection Model**: Deep understanding of the three-tier account protection system (economic/dependency/logical layers) that creates defense-in-depth against account loss in complex DeFi scenarios
- **Conservative Transaction Philosophy**: Operational wisdom that 75% balance utilization respects Substrate's protection mechanisms while enabling full DEX functionality—a pattern derived from production Asset Hub insights

---

### 3. Architectural Decisions

- **Polkadot SDK Framework**: Latest SDK with automatic upstream synchronization, modern FRAME macros (`#[frame::pallet]`), and workspace structure for modularity.

- **Omni Node Exclusive Architecture**:
  - **Rationale**: Fully optimized for Omni Node deployment, eliminating Template Node complexity to focus purely on runtime development and modern polkadot-sdk patterns.
  - **Trade-offs**: Simplified deployment and development workflow, but sacrifices custom node flexibility for streamlined runtime-focused development.

- **Asset Management Integration**: Integrated pallet-assets for comprehensive fungible asset support following modern FRAME patterns.

- **DEX Implementation Strategy**:
  - **Rationale**: Implemented AssetKind as enum with Native and Local(u32) variants for type safety and clear semantics, with documented Foreign(Location) extension path for future XCM v5 Location integration.
  - **Trade-offs**: Enum implementation provides type safety and clear DEX functionality while maintaining clear upgrade path to full XCM support.
  - **Current Status**: AssetKind enum operational with Native and Local(u32) variants, basic pool creation tests passing, documented Foreign(Location) extension for XCM v5 migration.
  - **Architecture**: AssetKind implemented with Native, Local(u32) variants and Foreign(Location) enum extension ready for XCM v5 implementation.
  - **SDK Integration**: Leverages polkadot-sdk-2503 auto-weight generation and modern configuration patterns with proper derive traits (Encode, Decode, MaxEncodedLen, TypeInfo, DecodeWithMemTracking).

---

### 4. Project Structure

- `/runtime/README.md`: Comprehensive DEX runtime documentation with architecture overview and integration examples
- `/runtime/`: Parachain runtime that aggregates pallets and defines chain behavior
- `/runtime/src/configs/`: Runtime configuration modules for different pallet groups
- `/runtime/src/configs/assets_config.rs`: Complete Assets and Asset Conversion pallet configurations using polkadot-sdk-2503 modern patterns
- `/pallets/`: Custom pallets directory containing modular blockchain functionality
- `/.github/`: GitHub workflows and CI/CD configuration
- `/scripts/`: Local development and testing scripts with smart path resolution
- `/scripts/test-ci-local.sh`: Local CI workflow testing with auto-navigation to project root
- `/scripts/test-release-local.sh`: Local release workflow testing and WASM artifact generation
- `/scripts/test-zombienet-local.sh`: Local zombienet multi-node testing with binary management
- `/scripts/download-omni-node.sh`: Automated polkadot-omni-node binary download utility
- `/target/`: Rust build artifacts directory
- `/target/release/wbuild/`: WebAssembly runtime build artifacts for chain spec generation
- `/zombienet-omni-node.toml`: Multi-node test network configuration for Omni Node
- `/dev_chain_spec.json`: Pre-configured development chain specification

---

### 5. Development Conventions

- **Modern FRAME Patterns**: Use `#[frame::pallet]` macro and latest pallet development patterns
- **Asset Management**: Full fungible asset support with economic security through deposits
- **DEX Architecture**: AssetKind enum with Native and Local(u32) variants, trait-based DEX Router with 0.3% configurable fees, integrated with Asset Conversion pallet using SDK 2503 patterns and proper balance constraints
- **SDK 2503 Migration Patterns**: Use `#[frame::pallet(dev_mode)]` for simplified development, `frame::prelude::*` for unified imports, proper `construct_runtime!` type export patterns to avoid recursive definitions, and workspace-based dependency management for consistent feature flags
- **Complexity Resolution Strategy**: When facing integration challenges, simplify abstractions progressively until a working foundation emerges, then build complexity incrementally. Always establish basic functionality with simple types (u32) before migrating to complex abstractions (XCM Location)
- **Substrate Development Philosophy**: Apparent obstacles in Substrate are often architectural wisdom in disguise—defensive programming that prevents catastrophic failures in production. Complex systems implement multiple protection layers that must be understood holistically, not circumvented
- **Blockchain Performance Optimization**: Eliminate heap allocations (Box, Vec) that create unpredictable execution times and non-deterministic behavior. Use BoundedVec with compile-time limits and stack-based allocation patterns for consensus-critical applications
- **Modular Configuration**: Separate configuration modules for different pallet groups
- **Omni Node Exclusive Development**: Fully optimized for Omni Node architecture with streamlined runtime-focused workflow
- **Chain Spec Generation**: Use staging-chain-spec-builder for consistent network configuration
- **Zombienet Integration**: Multi-node test networks for realistic parachain development and testing
- **Streamlined Development Workflow**: Omni Node for deployment, chopsticks for rapid iteration, zombienet for multi-node testing
- **Comprehensive Testing**: Include unit tests, mock runtime, and benchmarking for all pallets
- **Auto-Weight Management**: Leverage polkadot-sdk-2503 automatic weight generation (`WeightInfo = ()`) instead of manual weight calculation
- **Documentation**: Extensive inline documentation following Rust doc conventions, with architectural insights captured in `/docs/Polkadot-SDK-2503-Insights.md`
- **Error Handling**: Use descriptive error types and proper error propagation
- **Upstream Synchronization**: Automatic template updates from main Polkadot SDK monorepo
- **Type Safety Strategy**: AssetKind enum implementation provides compile-time safety with Native and Local(u32) variants, documented upgrade path to Foreign(Location) for XCM v5
- **SDK Import Modernization**: Use unified `polkadot_sdk::*` imports over fragmented crate-specific imports for cleaner dependency management
- **License Header Prohibition**: Do not include license headers or copyright notices at the beginning of source files to maintain clean, focused code structure

---

### 6. Task Completion Protocol

Every task must be concluded by strictly following this protocol. This ensures consistency and knowledge accumulation.

**Step 1: Verify Changes.** Ensure all changes align with the principles outlined in sections 3, 4, and 5 of this document.

**Step 2: Code Check.** If applicable, run the primary code quality check command (e.g., a linter or test runner) and ensure it passes without errors.

`cargo check --workspace`

**Step 3: Update Context.** This is the **final action** before completing the task. You must update **this file (`llm-context.md`)** to reflect the changes made.

---

### 7. Change History

Entries are numbered in chronological order. The list should not exceed 20 entries.

1.  **Context Initialization**:
    - **Task**: Initialize the project's knowledge base.
    - **Implementation**: This document was created to centralize project knowledge and establish a self-improving protocol.
    - **Rationale**: To provide a single source of truth, ensuring consistent and efficient development from the outset.
    - **Impact on Context**: The foundation for systematic knowledge accumulation is now in place.

2.  **DEX Router Integration Tests Completion**:
    - **Task**: Complete DEX Router integration testing infrastructure with runtime context.
    - **Implementation**: Developed comprehensive integration test suite with 11 passing tests covering swap execution, fee mechanisms, error handling, access control, path validation, minimum amount protection, event emission, and buyback mechanisms. Fixed compilation errors, resolved balance/liquidity issues, implemented actual swap execution in XYKAdapter, and updated fee collection mechanisms.
    - **Rationale**: Ensure DEX Router pallet functions correctly in runtime context with proper integration to AssetConversion pallet, validating fee splitting, access control, and error handling.
    - **Impact on Context**: DEX Router is now fully validated with production-ready integration tests, establishing confidence in the dual fee mechanism (0.2% router fee) and single entry point architecture.

3.  **Dual Fee Structure Implementation According to Tokenomics**:
    - **Task**: Implement proper dual fee structure: 0.2% router fee for buyback mechanism + 0.3% XYK pool fee for liquidity providers = 0.5% total user cost.
    - **Implementation**: Updated DEX Router pallet to collect 0.2% router fee for buyback and burning of base network asset before passing remaining amount to AssetConversion, which applies its own 0.3% fee for liquidity providers. Updated configuration, logic, tests, and documentation to reflect correct tokenomics structure.
    - **Rationale**: Align DEX Router fee structure with project tokenomics to support token price through buyback mechanism while rewarding liquidity providers through XYK pool fees.
    - **Impact on Context**: DEX Router now implements correct tokenomics-compliant fee structure with router fee supporting buyback mechanism and XYK pool fee supporting liquidity providers, all validated through comprehensive integration tests.
