# crabjar/install_deps.sh
# Install script for all required dependencies

set -e

echo "🦀 CrabJar Dependency Installer"
echo "=============================="
echo ""

# Check if we're running as root (needed for some installations)
if [ "$(id -u)" != "0" ]; then
    echo "⚠️  Warning: Not running as root. Some operations may require sudo."
    echo ""
fi

# Update package lists
echo "Updating package lists..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update
elif command -v pacman &> /dev/null; then
    sudo pacman -Sy
elif command -v dnf &> /dev/null; then
    sudo dnf check-update
fi

# Install Rust (if not already installed)
echo "Installing Rust..."
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    echo "Exporting PATH for Rust"
    export PATH="$HOME/.cargo/bin:$PATH"
else
    echo "✅ Rust already installed"
fi

# Install wasm-pack (for WASM tool development)
echo "Installing wasm-pack..."
if ! command -v wasm-pack &> /dev/null; then
    cargo install wasm-pack-cli --locked
else
    echo "✅ wasm-pack already installed"
fi

# Install wasmer (for running WASM tools)
echo "Installing wasmer..."
if ! command -v wasmer &> /dev/null; then
    curl https://get.wasmer.io -sSfL | sh
else
    echo "✅ wasmer already installed"
fi

# Install other required packages based on OS
if command -v apt-get &> /dev/null; then
    # Debian/Ubuntu
    sudo apt-get install -y \
        build-essential \
        curl \
        git \
        sqlite3 \
        libsqlite3-dev \
        || echo "⚠️  Some packages may require manual installation"
elif command -v pacman &> /dev/null; then
    # Arch Linux
    sudo pacman -S --noconfirm \
        base-devel \
        curl \
        git \
        sqlite \
        || echo "⚠️  Some packages may require manual installation"
elif command -v dnf &> /dev/null; then
    # Fedora
    sudo dnf install -y \
        make automake gcc-c++ kernel-devel \
        curl git sqlite \
        || echo "⚠️  Some packages may require manual installation"
fi

# Install Nushell (if not already installed)
echo "Installing Nushell..."
if ! command -v nu &> /dev/null; then
    curl https://getnu.sh | sh
else
    echo "✅ Nushell already installed"
fi

# Check if Redox OS is available
if command -v redox &> /dev/null || [ -d "/redox" ]; then
    echo "✅ Redox OS detected"
else
    echo "⚠️  Redox OS not detected. You may need to set up a VM or chroot."
fi

echo ""
echo "🎉 Dependency installation complete!"
echo ""
echo "To build and run CrabJar:"
echo "  ./build.sh"
echo "  ./target/release/crabjar"
echo ""
