#!/bin/bash

# Local Zombienet Test Script for Parachain Template
# This script sets up and runs zombienet tests locally to verify everything works
# before running in GitHub Actions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
POLKADOT_VERSION="stable2503"
BINARIES_DIR="./target/release"
ZOMBIENET_DIR="/tmp/zn-test-local"

echo -e "${GREEN}Starting local zombienet test setup...${NC}"

# Step 1: Check prerequisites
echo -e "${YELLOW}Step 1: Checking prerequisites...${NC}"

# Check if we're in the right directory, auto-navigate if needed
if [ ! -f "Cargo.toml" ]; then
    # Try to find project root by looking for Cargo.toml
    if [ -f "../Cargo.toml" ]; then
        echo -e "${YELLOW}Auto-navigating to project root...${NC}"
        cd ..
    elif [ -f "../../Cargo.toml" ]; then
        echo -e "${YELLOW}Auto-navigating to project root...${NC}"
        cd ../..
    else
        echo -e "${RED}Error: Cannot find project root (Cargo.toml not found)${NC}"
        echo -e "${YELLOW}Please run this script from the project root or scripts directory${NC}"
        exit 1
    fi
fi
echo "✓ Working from project root: $(pwd)"

# Step 2: Build the runtime
echo -e "${YELLOW}Step 2: Building the runtime...${NC}"
cargo build --package parachain-template-runtime --release

# Step 3: Create binaries directory if it doesn't exist
mkdir -p "$BINARIES_DIR"

# Step 4: Download binaries if they don't exist
echo -e "${YELLOW}Step 3: Downloading required binaries...${NC}"
cd "$BINARIES_DIR"

# Check and download polkadot
if [ ! -f "polkadot" ]; then
    echo "Downloading polkadot..."
    wget --no-verbose "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$POLKADOT_VERSION/polkadot"
    chmod +x polkadot
else
    echo "polkadot binary already exists"
fi

# Check and download polkadot-prepare-worker
if [ ! -f "polkadot-prepare-worker" ]; then
    echo "Downloading polkadot-prepare-worker..."
    wget --no-verbose "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$POLKADOT_VERSION/polkadot-prepare-worker"
    chmod +x polkadot-prepare-worker
else
    echo "polkadot-prepare-worker binary already exists"
fi

# Check and download polkadot-execute-worker
if [ ! -f "polkadot-execute-worker" ]; then
    echo "Downloading polkadot-execute-worker..."
    wget --no-verbose "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$POLKADOT_VERSION/polkadot-execute-worker"
    chmod +x polkadot-execute-worker
else
    echo "polkadot-execute-worker binary already exists"
fi

# Check and download polkadot-omni-node
if [ ! -f "polkadot-omni-node" ]; then
    echo "Downloading polkadot-omni-node..."
    wget --no-verbose "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$POLKADOT_VERSION/polkadot-omni-node"
    chmod +x polkadot-omni-node
else
    echo "polkadot-omni-node binary already exists"
fi

# Go back to project root
cd ../../

# Step 5: Verify chain spec exists
echo -e "${YELLOW}Step 4: Verifying chain spec...${NC}"
if [ ! -f "dev_chain_spec.json" ]; then
    echo -e "${RED}Error: dev_chain_spec.json not found!${NC}"
    exit 1
fi
echo "Chain spec found: dev_chain_spec.json"

# Step 6: Install zombienet CLI if not available
echo -e "${YELLOW}Step 5: Checking zombienet CLI...${NC}"
if ! command -v zombienet &> /dev/null; then
    echo "zombienet CLI not found. Installing via npx..."
    # We'll use npx to run it directly
else
    echo "zombienet CLI found"
fi

# Step 7: Clean up any previous test directory
echo -e "${YELLOW}Step 6: Cleaning up previous test runs...${NC}"
rm -rf "$ZOMBIENET_DIR" || echo "No previous test directory to clean"

# Step 8: Add binaries to PATH and run zombienet test
echo -e "${YELLOW}Step 7: Running zombienet test...${NC}"
export PATH="$BINARIES_DIR:$PATH"

# Verify binaries are accessible
echo "Verifying binaries are accessible:"
which polkadot
which polkadot-omni-node

# Run the test
echo -e "${GREEN}Starting zombienet test...${NC}"
npx --yes @zombienet/cli --dir "$ZOMBIENET_DIR" --provider native test .github/tests/zombienet-smoke-test.zndsl

# Step 9: Check results
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Zombienet test completed successfully!${NC}"
    echo -e "${GREEN}The GitHub Actions workflow should now work correctly.${NC}"
else
    echo -e "${RED}❌ Zombienet test failed!${NC}"
    echo -e "${YELLOW}Check the logs in: $ZOMBIENET_DIR/logs/${NC}"
    echo -e "${YELLOW}Available log files:${NC}"
    ls -la "$ZOMBIENET_DIR/logs/" 2>/dev/null || echo "No logs directory found"
    exit 1
fi

# Optional: Show log locations
echo -e "${YELLOW}Test logs are available at:${NC}"
echo "  - Alice: $ZOMBIENET_DIR/logs/alice.log"
echo "  - Bob: $ZOMBIENET_DIR/logs/bob.log"
echo "  - Charlie: $ZOMBIENET_DIR/logs/charlie.log"

echo -e "${GREEN}Local zombienet test completed successfully! 🎉${NC}"
