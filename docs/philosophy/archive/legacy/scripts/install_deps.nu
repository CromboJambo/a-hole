#!/usr/bin/env nu

# crabjar/install_deps.nu
# Install script for all required dependencies, converted to Nu

print "🦀 CrabJar Dependency Installer"
print "=============================="
print ""

# Check if we're running as root (needed for some installations)
let user_id = (id -u | complete).stdout | str trim
if $user_id != "0" {
    print "⚠️  Warning: Not running as root. Some operations may require sudo."
    print ""
}

# Update package lists
print "Updating package lists..."
if (command -v apt-get | complete).exit_code == 0 {
    sudo apt-get update
} else if (command -v pacman | complete).exit_code == 0 {
    sudo pacman -Sy
} else if (command -v dnf | complete).exit_code == 0 {
    sudo dnf check-update
}

# Install Rust (if not already installed)
print "Installing Rust..."
if not (command -v rustup | complete).exit_code == 0 {
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    print "Exporting PATH for Rust"
    export PATH="$HOME/.cargo/bin:$PATH"
} else {
    print "✅ Rust already installed"
}

# Install wasm-pack (for WASM tool development)
print "Installing wasm-pack..."
if not (command -v wasm-pack | complete).exit_code == 0 {
    cargo install wasm-pack-cli --locked
} else {
    print "✅ wasm-pack already installed"
}

# Install wasmer (for running WASM tools)
print "Installing wasmer..."
if not (command -v wasmer | complete).exit_code == 0 {
    curl https://get.wasmer.io -sSfL | sh
} else {
    print "✅ wasmer already installed"
}

# Install other required packages based on OS
if (command -v apt-get | complete).exit_code == 0 {
    # Debian/Ubuntu
    sudo apt-get install -y build-essential curl git sqlite3 libsqlite3-dev
    print "⚠️  Some packages may require manual installation"
} else if (command -v pacman | complete).exit_code == 0 {
    # Arch Linux
    sudo pacman -S --noconfirm base-devel curl git sqlite
    print "⚠️  Some packages may require manual installation"
} else if (command -v dnf | complete).exit_code == 0 {
    # Fedora
    sudo dnf install -y make automake gcc-c++ kernel-devel curl git sqlite
    print "⚠️  Some packages may require manual installation"
}

# Install Nushell (if not already installed)
print "Installing Nushell..."
if not (command -v nu | complete).exit_code == 0 {
    curl https://getnu.sh | sh
} else {
    print "✅ Nushell already installed"
}

# Check if Redox OS is available
if (command -v redox | complete).exit_code == 0 or ($env.HOME | path join "redox" | path exists) {
    print "✅ Redox OS detected"
} else {
    print "⚠️  Redox OS not detected. You may need to set up a VM or chroot."
}

print ""
print "🎉 Dependency installation complete!"
print ""
print "To build and run CrabJar:"
print "  ./build.sh"
print "  ./target/release/crabjar"
print ""
