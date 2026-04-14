#!/usr/bin/env nu

# crabjar/wasm_test.nu
# Nu script for legacy wasm test compatibility

print "🦀 CrabJar Smoke Test"
print "===================="
print ""

# Check if Rust is installed
if not (command -v rustc | complete).exit_code == 0 {
    print "❌ Error: Rust not found"
    print "   Please install Rust from https://rustup.rs/"
    exit 1
}

print "✅ Required tools are available"
print ""

# Check if CrabJar executable exists
let crabjar_path = "/home/crombo/crabjar/target/release/crabjar"
if not ($crabjar_path | path exists) {
    print "❌ Error: CrabJar executable not found at ($crabjar_path)"
    print "   Please build the project first:"
    print "   1. Make sure you're in the crabjar directory"
    print "   2. Run 'cargo build --release' to compile the executable"
    exit 1
}

print $"✅ CrabJar executable found at ($crabjar_path)"
print ""
print "Testing integration with CrabJar agent..."

# Run a simple CLI smoke test
let input = "state list\nexit"

# Try to execute the command and capture output
let result = ($input | split row "\n" | each { echo $it } | run $crabjar_path)

if ($result | str contains '"success": true') {
    print ""
    print "✅ Smoke test completed successfully!"
} else {
    print ""
    print "❌ Smoke test failed."
    print $"Output was: ($result)"
    exit 1
}
