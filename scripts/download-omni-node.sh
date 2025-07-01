#!/bin/bash

# Download script for polkadot-omni-node
# This script downloads the latest polkadot-omni-node binary for local development

set -e

REPO="paritytech/polkadot-sdk"
BINARY_NAME="polkadot-omni-node"
INSTALL_DIR="."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        print_warning "jq not found, will use basic parsing (may be less reliable)"
    fi
}

get_latest_release_url() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"

    if command -v jq &> /dev/null; then
        # Use jq for reliable JSON parsing
        local download_url=$(curl -s "$api_url" | jq -r ".assets[] | select(.name == \"$BINARY_NAME\") | .browser_download_url")
    else
        # Fallback to basic parsing
        local download_url=$(curl -s "$api_url" | grep "browser_download_url.*$BINARY_NAME" | cut -d '"' -f 4)
    fi

    if [ -z "$download_url" ] || [ "$download_url" = "null" ]; then
        print_error "Could not find download URL for $BINARY_NAME"
        exit 1
    fi

    echo "$download_url"
}

download_binary() {
    local url="$1"
    local output_path="$INSTALL_DIR/$BINARY_NAME"

    print_info "Downloading $BINARY_NAME from $url"

    if curl -L -f -o "$output_path" "$url"; then
        chmod +x "$output_path"
        print_info "Downloaded and made executable: $output_path"
    else
        print_error "Failed to download $BINARY_NAME"
        exit 1
    fi
}

verify_binary() {
    local binary_path="$INSTALL_DIR/$BINARY_NAME"

    if [ -x "$binary_path" ]; then
        print_info "Verifying binary..."
        if "$binary_path" --version > /dev/null 2>&1; then
            local version=$("$binary_path" --version 2>/dev/null | head -n1)
            print_info "Successfully verified: $version"
        else
            print_warning "Binary downloaded but version check failed"
        fi
    else
        print_error "Binary is not executable"
        exit 1
    fi
}

main() {
    print_info "Polkadot Omni Node Download Script"
    print_info "Repository: $REPO"
    print_info "Install directory: $INSTALL_DIR"

    # Check if binary already exists
    if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        print_warning "Binary already exists at $INSTALL_DIR/$BINARY_NAME"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Aborted by user"
            exit 0
        fi
    fi

    check_dependencies

    local download_url=$(get_latest_release_url)
    print_info "Found latest release URL: $download_url"

    download_binary "$download_url"
    verify_binary

    print_info "✅ $BINARY_NAME successfully downloaded!"
    print_info "You can now use: ./$BINARY_NAME --help"

    # Suggest adding to PATH for global access
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]] && [ "$INSTALL_DIR" != "." ]; then
        print_info "💡 To use globally, add to your PATH or move to /usr/local/bin:"
        print_info "   sudo mv $INSTALL_DIR/$BINARY_NAME /usr/local/bin/"
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--system]"
        echo ""
        echo "Downloads the latest polkadot-omni-node binary"
        echo ""
        echo "Options:"
        echo "  --system    Install to /usr/local/bin (requires sudo)"
        echo "  --help,-h   Show this help message"
        exit 0
        ;;
    --system)
        INSTALL_DIR="/usr/local/bin"
        if [ "$EUID" -ne 0 ]; then
            print_error "System installation requires sudo privileges"
            print_info "Run: sudo $0 --system"
            exit 1
        fi
        ;;
esac

main "$@"
