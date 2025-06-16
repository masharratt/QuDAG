#!/bin/bash

echo "Building QuDAG CLI..."

# Kill any existing cargo processes
echo "Cleaning up existing processes..."
pkill -9 cargo 2>/dev/null || true
pkill -9 rustc 2>/dev/null || true
sleep 1

# Clean the target directory
echo "Cleaning build artifacts..."
rm -rf target

# Set some environment variables to help with the build
export CARGO_HOME=/home/codespace/.cargo
export RUSTUP_HOME=/home/codespace/.rustup
export PATH="$CARGO_HOME/bin:$PATH"

# Try to build just the CLI binary with minimal features
echo "Building CLI binary..."
cd /workspaces/QuDAG/tools/cli
cargo build --bin qudag 2>&1 | tee build.log

if [ -f ../../target/debug/qudag ]; then
    echo "Build successful! Binary located at: /workspaces/QuDAG/target/debug/qudag"
    echo "Testing binary with --help:"
    ../../target/debug/qudag --help
else
    echo "Build failed. Checking log for errors..."
    tail -20 build.log
fi