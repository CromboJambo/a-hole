# crabjar/wasm_test.sh
# Legacy filename retained for compatibility; now runs a basic CrabJar smoke test.

set -e

echo "🦀 CrabJar Smoke Test"
echo "===================="
echo ""

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Error: Rust not found"
    echo "   Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✅ Required tools are available"
echo ""

cd /home/crombo/crabjar || exit 1
echo ""
echo "Testing integration with CrabJar agent..."
if cargo build --release; then
    echo "✅ Agent built successfully"
else
    echo "❌ Agent build failed"
    exit 1
fi

# Run a simple CLI smoke test
echo ""
echo "Running CLI smoke test..."
OUTPUT=$(./target/release/crabjar << EOF
state list
exit
EOF
)

echo "$OUTPUT"

if echo "$OUTPUT" | grep -q '"success": true'; then
    echo ""
    echo "✅ Smoke test completed successfully!"
else
    echo ""
    echo "❌ Smoke test failed."
    exit 1
fi
