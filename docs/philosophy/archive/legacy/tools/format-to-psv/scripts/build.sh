#!/usr/bin/env bash
# Build script for format-to-psv tool

set -e

cd "$(dirname "$0")/../.."

echo "Building format-to-psv..."

cargo build --release --manifest-path tools/format-to-psv/Cargo.toml

echo "Build complete!"
echo "Binary location: target/release/format-to-psv"
