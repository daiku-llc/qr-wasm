#!/bin/bash
# Build script for Cloudflare Workers deployment
# Installs Rust if not present, then builds the worker

set -e

echo "🔧 Setting up Rust toolchain..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
    source "$HOME/.cargo/env" || true
else
    echo "✅ Rust already installed"
fi

# Verify Rust installation
rustc --version
cargo --version

echo "🔨 Building worker..."
cargo install -q worker-build || true
worker-build --release

echo "✅ Build complete!"

