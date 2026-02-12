#!/bin/sh
# kdex installer script
# Usage: curl -sSf https://urbanisierung.github.io/kdex/install.sh | sh
#
# This script downloads the latest kdex binary and installs it to ~/.local/bin
# Re-run this script to update to the latest version.

set -e

# Configuration
REPO="urbanisierung/kdex"
BINARY_NAME="kdex"
INSTALL_DIR="${KDEX_INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    printf "${BLUE}info${NC}: %s\n" "$1"
}

success() {
    printf "${GREEN}success${NC}: %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${NC}: %s\n" "$1"
}

error() {
    printf "${RED}error${NC}: %s\n" "$1" >&2
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "Unsupported operating system: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get the download URL for the latest release
get_download_url() {
    local os="$1"
    local arch="$2"
    
    # Map to release artifact names
    case "${os}-${arch}" in
        linux-x86_64)
            TARGET="x86_64-unknown-linux-gnu"
            EXT="tar.gz"
            ;;
        darwin-aarch64)
            TARGET="aarch64-apple-darwin"
            EXT="tar.gz"
            ;;
        darwin-x86_64)
            # Fall back to aarch64 for now (Rosetta 2 compatible)
            TARGET="aarch64-apple-darwin"
            EXT="tar.gz"
            warn "x86_64 macOS binary not available, using aarch64 (Rosetta 2 compatible)"
            ;;
        windows-x86_64)
            TARGET="x86_64-pc-windows-msvc"
            EXT="zip"
            ;;
        *)
            error "No pre-built binary available for ${os}-${arch}"
            ;;
    esac
    
    # Get latest release download URL from GitHub API
    RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
    
    if command -v curl > /dev/null 2>&1; then
        DOWNLOAD_URL=$(curl -sSf "$RELEASE_URL" | grep "browser_download_url.*${TARGET}.*${EXT}" | head -1 | cut -d '"' -f 4)
    elif command -v wget > /dev/null 2>&1; then
        DOWNLOAD_URL=$(wget -qO- "$RELEASE_URL" | grep "browser_download_url.*${TARGET}.*${EXT}" | head -1 | cut -d '"' -f 4)
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
    
    if [ -z "$DOWNLOAD_URL" ]; then
        error "Could not find download URL for ${TARGET}. Check https://github.com/${REPO}/releases"
    fi
    
    echo "$DOWNLOAD_URL"
}

# Download and extract
download_and_install() {
    local url="$1"
    local install_dir="$2"
    
    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT
    
    info "Downloading from: $url"
    
    # Download
    if command -v curl > /dev/null 2>&1; then
        curl -sSfL "$url" -o "$TMP_DIR/archive"
    elif command -v wget > /dev/null 2>&1; then
        wget -q "$url" -O "$TMP_DIR/archive"
    fi
    
    # Extract based on extension
    if echo "$url" | grep -q "\.tar\.gz$"; then
        tar -xzf "$TMP_DIR/archive" -C "$TMP_DIR"
    elif echo "$url" | grep -q "\.zip$"; then
        unzip -q "$TMP_DIR/archive" -d "$TMP_DIR"
    fi
    
    # Find and install binary
    BINARY_PATH=$(find "$TMP_DIR" -name "$BINARY_NAME" -type f | head -1)
    
    if [ -z "$BINARY_PATH" ]; then
        error "Binary not found in archive"
    fi
    
    # Create install directory if needed
    mkdir -p "$install_dir"
    
    # Install binary
    cp "$BINARY_PATH" "$install_dir/$BINARY_NAME"
    chmod +x "$install_dir/$BINARY_NAME"
    
    # Create install method marker for self-update support
    CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/kdex"
    mkdir -p "$CONFIG_DIR"
    echo "script" > "$CONFIG_DIR/.install-method"
    
    success "Installed $BINARY_NAME to $install_dir/$BINARY_NAME"
}

# Check if directory is in PATH
check_path() {
    local dir="$1"
    
    case ":$PATH:" in
        *":$dir:"*) return 0 ;;
        *) return 1 ;;
    esac
}

# Suggest PATH update
suggest_path_update() {
    local dir="$1"
    
    if check_path "$dir"; then
        return
    fi
    
    warn "$dir is not in your PATH"
    echo ""
    echo "Add it to your shell configuration:"
    echo ""
    
    # Detect shell
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        bash)
            echo "  echo 'export PATH=\"$dir:\$PATH\"' >> ~/.bashrc"
            echo "  source ~/.bashrc"
            ;;
        zsh)
            echo "  echo 'export PATH=\"$dir:\$PATH\"' >> ~/.zshrc"
            echo "  source ~/.zshrc"
            ;;
        fish)
            echo "  fish_add_path $dir"
            ;;
        *)
            echo "  export PATH=\"$dir:\$PATH\""
            ;;
    esac
    echo ""
}

# Print post-install message
print_success() {
    local version
    version=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null | head -1 || echo "unknown")
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    success "kdex installed successfully!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  Version:  $version"
    echo "  Location: $INSTALL_DIR/$BINARY_NAME"
    echo ""
    echo "  Get started:"
    echo "    kdex add ~/notes          # Add a repository"
    echo "    kdex index                # Build search index"
    echo "    kdex \"search query\"       # Search your knowledge"
    echo "    kdex                      # Open interactive TUI"
    echo ""
    echo "  To update kdex, re-run this script."
    echo ""
}

# Main
main() {
    echo ""
    echo "  📚 kdex installer"
    echo "  AI-powered knowledge index for your code and notes"
    echo ""
    
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    info "Detected platform: ${OS}-${ARCH}"
    
    # Windows not fully supported via this script
    if [ "$OS" = "windows" ]; then
        warn "Windows support is experimental. Consider using:"
        echo "  cargo install kdex"
        echo "  or download from: https://github.com/${REPO}/releases"
        echo ""
    fi
    
    DOWNLOAD_URL=$(get_download_url "$OS" "$ARCH")
    
    download_and_install "$DOWNLOAD_URL" "$INSTALL_DIR"
    
    suggest_path_update "$INSTALL_DIR"
    
    print_success
}

main "$@"
