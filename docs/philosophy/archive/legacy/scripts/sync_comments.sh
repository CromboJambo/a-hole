#!/bin/bash

# Sync Comments Script
# This script synchronizes user comments from ~/.dotfiles to agent comments.toml files

set -e

# Configuration
DOTFILES_DIR="${HOME}/.dotfiles"
AGENTS_DIR="${HOME}/agents"
LOG_FILE="${HOME}/.sync_comments.log"

# Function to log messages
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE"
}

# Function to sync comments for a specific agent
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

# Function to sync all agents
sync_all_agents() {
    log_message "Starting sync of all agents comments"

    # Iterate through all agents
    for agent_dir in "${AGENTS_DIR}"/*; do
        if [ -d "$agent_dir" ]; then
            agent_name=$(basename "$agent_dir")
            sync_agent_comments "$agent_name"
        fi
    done

    log_message "All agents comments synced completed"
}

# Function to enable/disable comments layer
toggle_comments_layer() {
    local agent_name=$1
    local enable_flag=$2

    if [ "$enable_flag" = "true" ] || [ "$enable_flag" = "false" ]; then
        log_message "Setting comments layer enabled for agent: $agent_name to $enable_flag"
        # Set environment variable or config file
        export AGENT_COMMENTS_ENABLED="$enable_flag"
        echo "AGENT_COMMENTS_ENABLED=$enable_flag" >> "${AGENTS_DIR}/${agent_name}/.env"
    else
        log_message "Invalid flag value: $enable_flag"
        echo "Usage: toggle_comments_layer <agent_name> <true|false>"
        exit 1
    fi
}

# Main function
main() {
    log_message "Starting sync_comments script"

    case "$1" in
        "sync-all")
            sync_all_agents
            ;;
        "sync-agent")
            if [ -z "$2" ]; then
                log_message "Agent name required for sync-agent command"
                echo "Usage: sync_comments.sh sync-agent <agent_name>"
                exit 1
            fi
            sync_agent_comments "$2"
            ;;
        "toggle")
            if [ -z "$2" ] || [ -z "$3" ]; then
                log_message "Agent name and flag required for toggle command"
                echo "Usage: sync_comments.sh toggle <agent_name> <true|false>"
                exit 1
            fi
            toggle_comments_layer "$2" "$3"
            ;;
        *)
            echo "Usage: sync_comments.sh <sync-all|sync-agent|toggle>"
            echo "sync-all: Sync all agents comments"
            echo "sync-agent <agent_name>: Sync specific agent comments"
            echo "toggle <agent_name> <true|false>: Enable/disable comments layer"
            exit 1
            ;;
    esac

    log_message "Sync_comments script completed"
}

# Run main function
main "$@"
