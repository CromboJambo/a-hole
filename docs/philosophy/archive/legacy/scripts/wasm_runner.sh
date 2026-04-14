# crabjar/wasm_runner.sh
# Script to test and run WASM tools in CrabJar

set -e

echo "🦀 CrabJar WASM Tool Runner"
echo "=========================="
echo ""

# Check if wasmer is available
if ! command -v wasmer &> /dev/null; then
    echo "❌ Error: wasmer not found"
    echo "   Please install wasmer from https://wasmer.io/"
    exit 1
fi

echo "✅ wasmer is available"

# Check if WASM file exists
WASM_FILE="/home/crombo/crabjar/agent-wasm/tools/target/wasm32-unknown-unknown/release/file_read_wasm.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo "❌ Error: WASM file not found at $WASM_FILE"
    echo "   Please build the tools first with: ./tools/build.sh"
    exit 1
fi

echo "✅ WASM tool file found"

# Test input JSON (this would be dynamically generated in real usage)
INPUT_JSON='{"path": "/tmp/test.txt"}'

echo ""
echo "Testing WASM tool execution..."
echo ""

# Run the WASM tool with wasmer
OUTPUT=$(wasmer run $WASM_FILE --exec --input $INPUT_JSON)

echo "✅ WASM tool executed successfully"
echo ""
echo "Output:"
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q '"success": true'; then
    echo ""
    echo "✅ Test completed successfully!"
else
    echo ""
    echo "❌ Test failed. Check the output above for details."
fi
