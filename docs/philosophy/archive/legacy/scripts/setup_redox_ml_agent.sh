#!/bin/bash
set -e

echo "=== Setting up Redox OS with ML + Autonomous Rust Coding Agent ==="
echo ""

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Install prerequisites
echo -e "${BLUE}Step 1: Installing prerequisites...${NC}"
if command -v podman &> /dev/null; then
    echo "Podman already installed"
else
    echo "Installing Podman..."
    sudo apt-get update
    sudo apt-get install -y podman
fi

if command -v qemu-system-x86_64 &> /dev/null; then
    echo "QEMU already installed"
else
    echo "Installing QEMU..."
    sudo apt-get install -y qemu-system-x86 qemu-utils
fi

if command -v fuse &> /dev/null; then
    echo "FUSE already installed"
else
    echo "Installing FUSE..."
    sudo apt-get install -y fuse libfuse-dev
fi

# Step 2: Set up Redox OS
echo -e "\n${BLUE}Step 2: Setting up Redox OS build environment...${NC}"
if [ ! -d "redox" ]; then
    echo "Cloning Redox OS repository..."
    git clone https://gitlab.redox-os.org/redox-os/redox.git
    cd redox
else
    echo "Redox directory already exists"
    cd redox
    git pull --rebase --recurse-submodules
    git submodule sync
    git submodule update --recursive --init
fi

# Configure for Podman build
echo "Configuring Podman build..."
cat > .config << 'REDOX_CONFIG'
# Enable Podman build
PODMAN_BUILD=1
REDOX_CONFIG

echo -e "${GREEN}Redox OS setup complete!${NC}"

# Step 3: Build Redox (optional - can be done later)
echo -e "\n${YELLOW}To build Redox OS, run:${NC}"
echo "  cd redox"
echo "  make all"
echo ""
echo -e "${YELLOW}To run Redox in QEMU:${NC}"
echo "  make qemu"
echo ""

cd ..

# Step 4: Set up Rust coding agent framework
echo -e "${BLUE}Step 3: Setting up AutoAgents (Rust autonomous coding agent framework)...${NC}"

if [ ! -d "AutoAgents" ]; then
    echo "Cloning AutoAgents..."
    git clone https://github.com/liquidos-ai/AutoAgents.git
    cd AutoAgents
else
    echo "AutoAgents directory already exists"
    cd AutoAgents
    git pull
fi

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "Building AutoAgents..."
cargo build --release --all-features

echo -e "${GREEN}AutoAgents setup complete!${NC}"

cd ..

# Step 5: Alternative - ADK-Rust setup
echo -e "\n${BLUE}Step 4: Setting up ADK-Rust as alternative agent framework...${NC}"
mkdir -p adk-rust-project
cd adk-rust-project

cat > Cargo.toml << 'ADK_TOML'
[package]
name = "redox-ml-agent"
version = "0.1.0"
edition = "2021"

[dependencies]
adk-rust = { version = "0.1", features = ["minimal"] }
tokio = { version = "1.40", features = ["full"] }
dotenv = "0.15"
ADK_TOML

echo -e "${GREEN}ADK-Rust project initialized!${NC}"

cd ..

# Final summary
echo -e "\n${GREEN}======================================${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo -e "${BLUE}Directory structure:${NC}"
echo "  ./redox/           - Redox OS build environment"
echo "  ./AutoAgents/      - AutoAgents Rust framework"
echo "  ./adk-rust-project/ - ADK-Rust starter project"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Build Redox OS:"
echo "   cd redox && make all"
echo ""
echo "2. Run Redox in VM:"
echo "   cd redox && make qemu"
echo ""
echo "3. Test AutoAgents:"
echo "   cd AutoAgents/examples && cargo run --example simple_agent"
echo ""
echo "4. Configure API keys for agents:"
echo "   export OPENAI_API_KEY=your_key_here"
echo "   export ANTHROPIC_API_KEY=your_key_here"
echo ""
echo -e "${YELLOW}Note: OpenClaw is TypeScript-based, not Rust. We're using AutoAgents/ADK-Rust instead.${NC}"
echo ""

