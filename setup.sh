#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

BUILD_ONLY=false
if [[ "$1" == "--build-only" ]]; then
    BUILD_ONLY=true
fi

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" != "aarch64" ]]; then
    warn "Expected aarch64 architecture, got: $ARCH"
    warn "This script is intended for Raspberry Pi Zero 2 W (64-bit)"
fi

if [[ "$BUILD_ONLY" == false ]]; then
    info "Installing system dependencies..."
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        libudev-dev \
        libx11-dev \
        libxi-dev \
        libxcursor-dev \
        libxrandr-dev \
        libxinerama-dev \
        libasound2-dev

    # Install Rust if not present
    if ! command -v rustc &> /dev/null; then
        info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        info "Rust already installed: $(rustc --version)"
    fi
else
    info "Skipping dependency installation (--build-only)"
    if ! command -v cargo &> /dev/null; then
        error "cargo not found. Run without --build-only first."
    fi
fi

info "Building release binary..."
cargo build --release

info "Build complete!"
info "Binary located at: target/release/ic"
