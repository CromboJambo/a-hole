#!/bin/bash

# Agent Updater Script
# This script synchronizes agent configurations and updates from .dotfiles

set -e

# Configuration
DOTFILES_DIR="${HOME}/.dotfiles"
AGENTS_DIR="${HOME}/agents"
LOG_FILE="${HOME}/.agent_updater.log"

# Function to log messages
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE"
}

# Function to sync agent comments
sync_agent_comments() {
    local agent_name=$1
    local dotfiles_path="${DOTFILES_DIR}/${agent_name}_comments.toml"
    local agent_path="${AGENTS_DIR}/${agent_name}/comments.toml"

    if [ -f "$dotfiles_path" ]; then
        log_message "Syncing comments for agent: $agent_name"
        rsync -av "$dotfiles_path" "$agent_path"
        log_message "Comments synced successfully for agent: $agent_name"
    else
        log_message "No comments file found for agent: $agent_name"
    fi
}

# Function to check version and update
check_and_update() {
    local agent_name=$1

    # Check if agent has git directory
    local git_path="${AGENTS_DIR}/${agent_name}/git"

    if [ -d "$git_path" ]; then
        log_message "Checking updates for agent: $agent_name"

        # Pull latest changes from git
        cd "$git_path"
        git pull origin main

        # Check if there are local changes to commit
        if git diff-index --quiet HEAD --; then
            log_message "No local changes to commit for agent: $agent_name"
        else
            log_message "Local changes detected for agent: $agent_name"
            git add .
            git commit -m "Auto-commit from updater script"
            git push origin main
        fi

        # Rebuild the agent if needed
        cd "${AGENTS_DIR}/${agent_name}"
        cargo build --release

        log_message "Agent updated successfully: $agent_name"
    else
        log_message "No git directory found for agent: $agent_name"
    fi
}

# Main function
main() {
    log_message "Starting agent updater script"

    # Sync all agents
    for agent_dir in "${AGENTS_DIR}"/*; do
        if [ -d "$agent_dir" ]; then
            agent_name=$(basename "$agent_dir")
            sync_agent_comments "$agent_name"
            check_and_update "$agent_name"
        fi
    done

    log_message "Agent updater script completed"
}

# Run main function
main
