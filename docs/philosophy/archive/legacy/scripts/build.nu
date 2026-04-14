#!/usr/bin/env nu

# crabjar/build.nu
# Build script for the stripped-down CrabJar CLI, converted to Nu

print "🦀 CrabJar Build Script"
print "====================================="
print ""

# Check if Rust is installed
if not (command -v cargo | complete).exit_code == 0 {
    print "❌ Error: cargo not found"
    print "   Please install Rust from https://rustup.rs/"
    exit 1
}

# Build the main agent
print "📦 Building main agent..."
cd /home/crombo/crabjar

let result = (cargo build --release)

if $result.exit_code == 0 {
    print ""
    print "✅ Main agent built successfully!"
    print ""
    print "Agent binary location:"
    print "  /home/crombo/crabjar/target/release/crabjar"
    print ""
    print "To run the agent:"
    print "  ./target/release/crabjar"
} else {
    print ""
    print "❌ Main agent build failed"
    exit 1
}

print "🎉 Build complete! All components built successfully."
