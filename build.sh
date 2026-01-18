#!/bin/bash

# XMPP Client Build Script - Distro Agnostic

set -e

echo "🚀 Building XMPP Client..."

# Detect OS and package manager
detect_package_manager() {
    if command -v apt-get &> /dev/null; then
        echo "apt"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v zypper &> /dev/null; then
        echo "zypper"
    elif command -v brew &> /dev/null; then
        echo "brew"
    else
        echo "unknown"
    fi
}

PKG_MANAGER=$(detect_package_manager)
echo "📦 Detected package manager: $PKG_MANAGER"

# Install dependencies based on package manager
install_dependencies() {
    case $PKG_MANAGER in
        apt)
            echo "📥 Installing dependencies with apt..."
            sudo apt update
            sudo apt install -y \
                libgtk-4-dev \
                libglib2.0-dev \
                libcairo2-dev \
                libpango1.0-dev \
                libgdk-pixbuf-2.0-dev \
                sqlite3 \
                libsqlite3-dev \
                pkg-config \
                build-essential \
                libssl-dev
            ;;
        dnf)
            echo "📥 Installing dependencies with dnf..."
            sudo dnf install -y \
                gtk4-devel \
                glib2-devel \
                cairo-devel \
                pango-devel \
                gdk-pixbuf2-devel \
                sqlite \
                sqlite-devel \
                pkgconfig \
                gcc \
                openssl-devel
            ;;
        pacman)
            echo "📥 Installing dependencies with pacman..."
            sudo pacman -S --needed \
                gtk4 \
                glib2 \
                cairo \
                pango \
                gdk-pixbuf2 \
                sqlite \
                pkgconf \
                gcc \
                openssl
            ;;
        zypper)
            echo "📥 Installing dependencies with zypper..."
            sudo zypper install -y \
                gtk4-devel \
                glib2-devel \
                cairo-devel \
                pango-devel \
                gdk-pixbuf-devel \
                sqlite3-devel \
                pkg-config \
                gcc \
                libopenssl-devel
            ;;
        brew)
            echo "📥 Installing dependencies with brew..."
            brew install \
                gtk4 \
                pkg-config \
                sqlite3 \
                openssl
            ;;
        *)
            echo "❌ Unknown package manager: $PKG_MANAGER"
            echo "Please install the following dependencies manually:"
            echo "- GTK4 development libraries"
            echo "- SQLite development libraries"
            echo "- pkg-config"
            echo "- C compiler (gcc/clang)"
            echo "- OpenSSL development libraries"
            exit 1
            ;;
    esac
}

# Check for Rust and install if needed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "🦀 Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source "$HOME/.cargo/env"
    else
        echo "✅ Rust is already installed"
    fi
}

# Verify dependencies
verify_dependencies() {
    echo "🔍 Verifying dependencies..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo not found"
        exit 1
    fi
    echo "✅ Rust: $(cargo --version)"
    
    # Check for pkg-config
    if ! command -v pkg-config &> /dev/null; then
        echo "❌ pkg-config not found"
        exit 1
    fi
    echo "✅ pkg-config: $(pkg-config --version)"
    
    # Check for GTK
    if pkg-config --exists gtk4; then
        echo "✅ GTK4: $(pkg-config --modversion gtk4)"
    elif pkg-config --exists gtk+-3.0; then
        echo "✅ GTK3: $(pkg-config --modversion gtk+-3.0) (will use fallback)"
    else
        echo "❌ GTK not found"
        exit 1
    fi
    
    # Check for SQLite
    if pkg-config --exists sqlite3; then
        echo "✅ SQLite: $(pkg-config --modversion sqlite3)"
    else
        echo "⚠️  SQLite not found via pkg-config (may still work)"
    fi
}

# Build the application
build_app() {
    echo "🔨 Building XMPP Client..."
    
    # Try with GTK4 first
    echo "Attempting build with GTK4..."
    if cargo build --release --features gtk4 2>/dev/null; then
        echo "✅ GTK4 build successful!"
        return 0
    fi
    
    # Fallback to GTK3
    echo "GTK4 not available, trying GTK3 fallback..."
    if cargo build --release --features gtk3 2>/dev/null; then
        echo "✅ GTK3 fallback build successful!"
        return 0
    fi
    
    # Try without specific features
    echo "Attempting minimal build..."
    if cargo build --release 2>/dev/null; then
        echo "✅ Minimal build successful!"
        return 0
    fi
    
    echo "❌ All build attempts failed"
    exit 1
}

# Main execution
main() {
    echo "🏗️  XMPP Client Build System"
    echo "==============================="
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        echo "❌ Cargo.toml not found. Please run from project root."
        exit 1
    fi
    
    # Install dependencies if requested
    if [ "$1" = "--install-deps" ]; then
        install_dependencies
    fi
    
    # Install Rust if needed
    check_rust
    
    # Verify dependencies
    verify_dependencies
    
    # Build application
    build_app
    
    echo ""
    echo "🎉 Build completed successfully!"
    echo "📂 Binary location: target/release/xmpp-client"
    echo ""
    echo "🚀 To run the application:"
    echo "   ./target/release/xmpp-client"
    echo ""
    echo "⚙️  To run with debug logging:"
    echo "   RUST_LOG=debug ./target/release/xmpp-client"
}

# Show usage if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --install-deps    Install system dependencies"
    echo "  --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --install-deps    # Install dependencies and build"
    echo "  $0                 # Build only (assumes dependencies installed)"
    exit 0
fi

# Run main function
main "$@"