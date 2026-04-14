#!/usr/bin/env nu

# Agent Updater Script
# Converted from updater.sh
# This script synchronizes agent configurations and updates from .dotfiles

# Configuration
let dotfiles_dir = ($env.HOME | path join ".dotfiles")
let agents_dir = ($env.HOME | path join "agents")
let log_file = ($env.HOME | path join ".agent_updater.log")

# Function to log messages
def log_message [message: string] {
    let timestamp = (date now | format "%Y-%m-%d %H:%M:%S")
    let log_entry = $"($timestamp) - ($message)"

    # Append to log file
    $log_entry | save -a $log_file
}

# Function to sync agent comments
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

# Function to check version and update
def check_and_update [agent_name: string] {
    let git_path = ($agents_dir | path join $agent_name "git")

    if ($git_path | path exists) {
        log_message $"Checking updates for agent: ($agent_name)"

        # Pull latest changes from git
        cd $git_path
        let pull_result = (git pull origin main)

        if $pull_result.exit_code != 0 {
            log_message $"Failed to pull from git for agent: ($agent_name)"
        }

        # Check if there are local changes to commit
        let diff_result = (git diff-index --quiet HEAD --)
        if $diff_result.exit_code != 0 {
            log_message $"Local changes detected for agent: ($agent_name)"
            let add_result = (git add .)
            let commit_result = (git commit -m "Auto-commit from updater script")

            # Check if commit was successful
            if $commit_result.exit_code == 0 {
                let push_result = (git push origin main)
                if $push_result.exit_code != 0 {
                    log_message $"Failed to push changes for agent: ($agent_name)"
                }
            } else {
                log_message $"Failed to commit changes for agent: ($agent_name)"
            }
        } else {
            log_message $"No local changes to commit for agent: ($agent_name)"
        }

        # Rebuild the agent if needed
        cd $agents_dir
        let build_result = (cargo build --release)

        if $build_result.exit_code == 0 {
            log_message $"Agent updated successfully: ($agent_name)"
        } else {
            log_message $"Failed to rebuild agent: ($agent_name)"
        }
    } else {
        log_message $"No git directory found for agent: ($agent_name)"
    }
}

# Main function
def main [] {
    log_message "Starting agent updater script"

    # Get all agent directories
    let agents = (ls $agents_dir | where { |x| $x.type == "dir" } | get name)

    for agent in $agents {
        sync_agent_comments $agent
        check_and_update $agent
    }

    log_message "Agent updater script completed"
}

# Run main function
main
