#!/usr/bin/env nu

# crabjar/wasm_runner.nu
# Nu script for testing and running WASM tools in CrabJar, converted from wasm_runner.sh

print "🦀 CrabJar WASM Tool Runner"
print "=========================="
print ""

# Check if wasmer is available
if not (command -v wasmer | complete).exit_code == 0 {
    print "❌ Error: wasmer not found"
    print "   Please install wasmer from https://wasmer.io/"
    exit 1
}

print "✅ wasmer is available"

# Check if WASM file exists
let wasm_file = "/home/crombo/crabjar/agent-wasm/tools/target/wasm32-unknown-unknown/release/file_read_wasm.wasm"
if not ($wasm_file | path exists) {
    print $"❌ Error: WASM file not found at ($wasm_file)"
    print "   Please build the tools first with: ./tools/build.sh"
    exit 1
}

print "✅ WASM tool file found"

# Test input JSON (this would be dynamically generated in real usage)
let input_json = '{"path": "/tmp/test.txt"}'

print ""
print "Testing WASM tool execution..."
print ""

# Run the WASM tool with wasmer
let result = (wasmer run $wasm_file --exec --input $input_json)

print "✅ WASM tool executed successfully"
print ""
print "Output:"
print $result.stdout

if ($result.stdout | str contains '"success": true') {
    print ""
    print "✅ Test completed successfully!"
} else {
    print ""
    print "❌ Test failed. Check the output above for details."
}
