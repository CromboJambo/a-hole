#!/usr/bin/env nu

# crabjar/test_agent.nu
# Nu script for running agent tests, converted from test_agent.sh

let root_dir = (dirname $env.PWD)
print $"Running Crabjar checks from ($root_dir)"
cd $root_dir

print "1) Workspace check"
let check_result = (cargo check --workspace)
if $check_result.exit_code != 0 {
    print $"Error in workspace check: ($check_result.stderr)"
    exit 1
}

print "2) Root crate tests"
let test_result = (cargo test -p crabjar)
if $test_result.exit_code != 0 {
    print $"Error in root crate tests: ($test_result.stderr)"
    exit 1
}

print "All checks passed."
