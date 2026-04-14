#!/usr/bin/env nu

# crabjar/sync_comments.nu
# Nu script for syncing agent comments

# Configuration
let dotfiles_dir = ($env.HOME | path join ".dotfiles")
let agents_dir = ($env.HOME | path join "agents")
let log_file = ($env.HOME | path join ".sync_comments.log")

# Function to log messages
def log_message [message: string] {
    let timestamp = (date now | date format "%Y-%m-%d %H:%M:%S")
    let log_entry = $"($timestamp) - ($message)"

    # Append to log file
    $log_entry | save -a $log_file
}

# Function to sync comments for a specific agent
def sync_agent_comments [agent_name: string] {
    let dotfiles_path = ($dotfiles_dir | path join $"($agent_name)_comments.toml")
    let agent_path = ($agents_dir | path join $agent_name "comments.toml")

    if ($dotfiles_path | path exists) {
        log_message $"Syncing comments for agent: ($agent_name)"
        # Using cp command to copy files
        cp $dotfiles_path $agent_path
        log_message $"Comments synced successfully for agent: ($agent_name)"
    } else {
        log_message $"No comments file found for agent: ($agent_name)"
    }
}

# Function to sync all agents
def sync_all_agents [] {
    log_message "Starting sync of all agents comments"

    # Iterate through all agents
    let agents = (ls $agents_dir | where { |x| $x.type == "dir" } | get name)

    for agent in $agents {
        sync_agent_comments $agent
    }

    log_message "All agents comments synced completed"
}

# Function to enable/disable comments layer
def toggle_comments_layer [agent_name: string, enable_flag: string] {
    if ($enable_flag == "true" or $enable_flag == "false") {
        log_message $"Setting comments layer enabled for agent: ($agent_name) to ($enable_flag)"
        # Set environment variable directly via $env assignment
        $env.AGENT_COMMENTS_ENABLED = $enable_flag
        let env_file = ($agents_dir | path join $agent_name ".env")
        $"AGENT_COMMENTS_ENABLED=($enable_flag)" | save -a $env_file
    } else {
        log_message $"Invalid flag value: ($enable_flag)"
        print "Usage: sync_comments.nu toggle <agent_name> <true|false>"
        exit 1
    }
}

# Main function
def main [args: list<string>] {
    log_message "Starting sync_comments script"

    match ($args | first) {
        "sync-all" => {
            sync_all_agents
        },
        "sync-agent" => {
            if ($args | length) < 2 {
                log_message "Agent name required for sync-agent command"
                print "Usage: sync_comments.nu sync-agent <agent_name>"
                exit 1
            }
            let agent_name = ($args | get 1)
            sync_agent_comments $agent_name
        },
        "toggle" => {
            if ($args | length) < 3 {
                log_message "Agent name and flag required for toggle command"
                print "Usage: sync_comments.nu toggle <agent_name> <true|false>"
                exit 1
            }
            let agent_name = ($args | get 1)
            let enable_flag = ($args | get 2)
            toggle_comments_layer $agent_name $enable_flag
        },
        _ => {
            print "Usage: sync_comments.nu <sync-all|sync-agent|toggle>"
            print "sync-all: Sync all agents comments"
            print "sync-agent <agent_name>: Sync specific agent comments"
            print "toggle <agent_name> <true|false>: Enable/disable comments layer"
            exit 1
        }
    }

    log_message "sync_comments script completed"
}

# Run main function with command line arguments
main $env.ARGS
