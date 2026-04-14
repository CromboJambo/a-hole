#!/usr/bin/env nu

# crabjar/podman.nu
# Nu script for Podman container management

def get_root_dir [] {
    let current_dir = (pwd)
    # Navigate up one level from current directory
    $current_dir | path join ".."
}

let root_dir = (get_root_dir)
let image_name = "crabjar-dev"

def usage [] {
    print "Usage: scripts/podman.nu <command>"
    print ""
    print "Commands:"
    print "  build   Build the dev container image"
    print "  shell   Start a shell with the repo mounted at /workspace (rw)"
    print "  run     Run a one-off command in the container (pass after --)"
    print ""
    print "Examples:"
    print "  scripts/podman.nu build"
    print "  scripts/podman.nu shell"
    print "  scripts/podman.nu run -- cargo build"
}

let cmd = ($argv | get 0)

match $cmd {
    "build" => {
        let result = (podman build -f ($root_dir | path join "Containerfile") -t $image_name $root_dir)
        if $result.exit_code != 0 {
            print $"Error building image: ($result.stderr)"
            exit 1
        }
    },
    "shell" => {
        let result = (podman run --rm -it \
            -v ($root_dir | path join ":/workspace:rw,z") \
            -w /workspace \
            $image_name \
            /bin/bash)
        if $result.exit_code != 0 {
            print $"Error running shell: ($result.stderr)"
            exit 1
        }
    },
    "run" => {
        let args = ($argv | skip 1)
        if ($args | length) == 0 {
            print "No command provided."
            usage
            exit 1
        }

        # Skip the "--" argument if present
        let cmd_args = if ($args | first) == "--" {
            $args | skip 1
        } else {
            $args
        }

        let result = (podman run --rm -it \
            -v ($root_dir | path join ":/workspace:rw,z") \
            -w /workspace \
            $image_name \
            ...$cmd_args)
        if $result.exit_code != 0 {
            print $"Error running command: ($result.stderr)"
            exit 1
        }
    },
    _ => {
        usage
        exit 1
    }
}
