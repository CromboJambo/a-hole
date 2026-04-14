#!/usr/bin/env nu

# Build script for format-to-psv tool
# Converted from build.sh

# Change to the directory containing this script
cd (dirname $env.PWD)/../..

print "Building format-to-psv..."

# Run cargo build command
let result = (cargo build --release --manifest-path tools/format-to-psv/Cargo.toml)

if $result.exit_code == 0 {
    print "Build complete!"
    print "Binary location: target/release/format-to-psv"
} else {
    print $"Error during build: ($result.stderr)"
    exit 1
}
