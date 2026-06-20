#!/bin/sh
# Wasmee installer script
# Detects OS and CPU architecture, downloads the correct wasmee binary, and installs it.

set -eu

# Configuration
WASMEE_BASE_URL=${WASMEE_BASE_URL:-"https://test.wasmee.com"}
INSTALL_DIR=${INSTALL_DIR:-"/usr/local/bin"}
LOCAL_INSTALL_DIR="$HOME/.wasmee/bin"

# Colors for output
info() {
    printf "\033[1;34m[info]\033[0m %s\n" "$1"
}
success() {
    printf "\033[1;32m[success]\033[0m %s\n" "$1"
}
error() {
    printf "\033[1;31m[error]\033[0m %s\n" "$1" >&2
}
warning() {
    printf "\033[1;33m[warn]\033[0m %s\n" "$1"
}

# 1. Detect OS
OS=$(uname -s)
case "$OS" in
    Darwin)
        OS_NAME="darwin"
        ;;
    Linux)
        OS_NAME="linux"
        ;;
    *)
        error "Unsupported operating system: $OS. Wasmee currently supports macOS and Linux."
        exit 1
        ;;
esac

# 2. Detect Architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64)
        ARCH_NAME="amd64"
        ;;
    arm64|aarch64)
        ARCH_NAME="arm64"
        ;;
    *)
        error "Unsupported CPU architecture: $ARCH. Wasmee supports x86_64 (amd64) and arm64 (aarch64)."
        exit 1
        ;;
esac

# Construct binary name
BINARY_NAME="wasmee-${OS_NAME}-${ARCH_NAME}"

# Handle missing combos
if [ "$OS_NAME" = "darwin" ] && [ "$ARCH_NAME" = "amd64" ]; then
    error "Wasmee currently only provides Apple Silicon (ARM64) builds for macOS. Intel macOS is not supported."
    exit 1
fi

DOWNLOAD_URL="${WASMEE_BASE_URL}/bin/${BINARY_NAME}"

info "Detected System: OS=$OS ($OS_NAME), ARCH=$ARCH ($ARCH_NAME)"
info "Downloading Wasmee binary from: $DOWNLOAD_URL"

# Create a secure temp directory
TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'wasmee')
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Download binary using curl or wget
if command -v curl >/dev/null 2>&1; then
    curl -sSfL "$DOWNLOAD_URL" -o "$TMP_DIR/wasmee"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$TMP_DIR/wasmee" "$DOWNLOAD_URL"
else
    error "Could not find curl or wget. Please install one of them to proceed."
    exit 1
fi

chmod +x "$TMP_DIR/wasmee"

# 3. Determine install destination
if [ -w "$INSTALL_DIR" ]; then
    info "Installing wasmee to $INSTALL_DIR..."
    cp "$TMP_DIR/wasmee" "$INSTALL_DIR/wasmee"
    success "Wasmee installed successfully at $INSTALL_DIR/wasmee!"
else
    warning "No write permission for $INSTALL_DIR. Installing locally to $LOCAL_INSTALL_DIR..."
    mkdir -p "$LOCAL_INSTALL_DIR"
    cp "$TMP_DIR/wasmee" "$LOCAL_INSTALL_DIR/wasmee"
    success "Wasmee installed successfully at $LOCAL_INSTALL_DIR/wasmee!"
    
    # Check if local bin is in PATH
    case ":$PATH:" in
        *:"$LOCAL_INSTALL_DIR":*)
            ;;
        *)
            warning "Please add the installation directory to your PATH by adding the following line to your ~/.bashrc or ~/.zshrc:"
            printf "\n    export PATH=\"\$PATH:%s\"\n\n" "$LOCAL_INSTALL_DIR"
            ;;
    esac
fi

# Print version info
info "Checking installed binary version:"
if command -v wasmee >/dev/null 2>&1; then
    wasmee --help | head -n 1
elif [ -f "$LOCAL_INSTALL_DIR/wasmee" ]; then
    "$LOCAL_INSTALL_DIR/wasmee" --help | head -n 1
fi
