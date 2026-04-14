# crabjar/build.sh
# Build script for the stripped-down CrabJar CLI

set -e

echo "🦀 CrabJar Build Script"
echo "====================================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found"
    echo "   Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build the main agent
echo "📦 Building main agent..."
cd /home/crombo/crabjar

if cargo build --release; then
    echo ""
    echo "✅ Main agent built successfully!"
    echo ""
    echo "Agent binary location:"
    echo "  /home/crombo/crabjar/target/release/crabjar"
    echo ""
    echo "To run the agent:"
    echo "  ./target/release/crabjar"
else
    echo ""
    echo "❌ Main agent build failed"
    exit 1
fi

echo "🎉 Build complete! All components built successfully."
