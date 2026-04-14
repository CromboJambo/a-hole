#!/usr/bin/env nu

# crabjar/podman-test.nu
# Nu script for running tests in Podman container, converted from podman-test.sh

def get_root_dir [] {
    let current_dir = (pwd)
    # This is a simplified approach - in practice you'd want to use
    # the actual directory resolution logic from the original bash script
    $current_dir | path join ".."
}

let root_dir = (get_root_dir)
let image_name = "crabjar-dev"

let result = (podman run --rm -it \
    -v ($root_dir | path join ":/workspace:rw,z") \
    -w /workspace \
    $image_name \
    /bin/bash -lc "cargo test")

if $result.exit_code != 0 {
    print $"Error running tests: ($result.stderr)"
    exit 1
}
