#!/usr/bin/env nu

# crabjar/setup_redox_ml_agent.nu
# Nu script for setting up Redox OS with ML + Autonomous Rust Coding Agent, converted from setup_redox_ml_agent.sh

print "=== Setting up Redox OS with ML + Autonomous Rust Coding Agent ==="
print ""

# Color codes for output (using nu's print capabilities)
let green = "✅"
let blue = "ℹ️ "
let yellow = "⚠️ "

# Step 1: Install prerequisites
print $"${blue}Step 1: Installing prerequisites...${NC}"

# Check if podman is installed
if not (command -v podman | complete).exit_code == 0 {
    print "Installing Podman..."
    sudo apt-get update
    sudo apt-get install -y podman
}

# Check if qemu is installed
if not (command -v qemu-system-x86_64 | complete).exit_code == 0 {
    print "Installing QEMU..."
    sudo apt-get install -y qemu-system-x86 qemu-utils
}

# Check if fuse is installed
if not (command -v fuse | complete).exit_code == 0 {
    print "Installing FUSE..."
    sudo apt-get install -y fuse libfuse-dev
}

# Step 2: Set up Redox OS
print "\n${blue}Step 2: Setting up Redox OS build environment...${NC}"

let redox_dir = ($env.HOME | path join "redox")
if not ($redox_dir | path exists) {
    print "Cloning Redox OS repository..."
    git clone https://gitlab.redox-os.org/redox-os/redox.git
    cd $redox_dir
} else {
    print "Redox directory already exists"
    cd $redox_dir
    git pull --rebase --recurse-submodules
    git submodule sync
    git submodule update --recursive --init
}

# Configure for Podman build
let config_content = "# Enable Podman build\nPODMAN_BUILD=1\n"
$config_content | save -f ($redox_dir | path join ".config")

print $"${green}Redox OS setup complete!${NC}"

# Step 3: Build Redox (optional - can be done later)
print "\n${yellow}To build Redox OS, run:${NC}"
print "  cd redox && make all"
print ""
print "${yellow}To run Redox in QEMU:${NC}"
print "  cd redox && make qemu"
print ""

# Step 4: Set up Rust coding agent framework
print $"${blue}Step 3: Setting up AutoAgents (Rust autonomous coding agent framework)...${NC}"

let autoagents_dir = ($env.HOME | path join "AutoAgents")
if not ($autoagents_dir | path exists) {
    print "Cloning AutoAgents..."
    git clone https://github.com/liquidos-ai/AutoAgents.git
    cd $autoagents_dir
} else {
    print "AutoAgents directory already exists"
    cd $autoagents_dir
    git pull
}

# Install Rust if not present
if not (command -v cargo | complete).exit_code == 0 {
    print "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Note: In nu, we can't easily source the environment like in bash,
    # so this would need to be handled differently in practice
}

print "Building AutoAgents..."
let build_result = (cargo build --release --all-features)
if $build_result.exit_code != 0 {
    print $"Error building AutoAgents: ($build_result.stderr)"
    exit 1
}

print $"${green}AutoAgents setup complete!${NC}"

# Step 5: Alternative - ADK-Rust setup
print "\n${blue}Step 4: Setting up ADK-Rust as alternative agent framework...${NC}"
let adk_dir = ($env.HOME | path join "adk-rust-project")
mkdir $adk_dir

let cargo_toml_content = "[package]\nname = \"redox-ml-agent\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nadk-rust = { version = \"0.1\", features = [\"minimal\"] }\ntokio = { version = \"1.40\", features = [\"full\"] }\ndotenv = \"0.15\"\n"
$cargo_toml_content | save -f ($adk_dir | path join "Cargo.toml")

print $"${green}ADK-Rust project initialized!${NC}"

# Final summary
print "\n${green}======================================${NC}"
print "${green}Setup Complete!${NC}"
print "${green}======================================${NC}"
print ""
print "${blue}Directory structure:${NC}"
print "  ./redox/           - Redox OS build environment"
print "  ./AutoAgents/      - AutoAgents Rust framework"
print "  ./adk-rust-project/ - ADK-Rust starter project"
print ""
print "${blue}Next steps:${NC}"
print "1. Build Redox OS:"
print "   cd redox && make all"
print ""
print "2. Run Redox in VM:"
print "   cd redox && make qemu"
print ""
print "3. Test AutoAgents:"
print "   cd AutoAgents/examples && cargo run --example simple_agent"
print ""
print "4. Configure API keys for agents:"
print "   export OPENAI_API_KEY=your_key_here"
print "   export ANTHROPIC_API_KEY=your_key_here"
print ""
print "${yellow}Note: OpenClaw is TypeScript-based, not Rust. We're using AutoAgents/ADK-Rust instead.${NC}"
print ""
