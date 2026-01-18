#!/bin/bash

# XMPP Client Container-based Build Script

set -e

echo "🐳 XMPP Client Container Build System"
echo "===================================="

# Check for Podman
if ! command -v podman &> /dev/null; then
    echo "❌ Podman not found. Please install Podman:"
    echo "  Fedora/CentOS: sudo dnf install podman"
    echo "  Ubuntu/Debian: sudo apt install podman"
    echo "  Or use Docker: docker build -f Containerfile -t xmpp-builder ."
    exit 1
fi

# Show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  build              Build in container"
    echo "  run                Build and run in container"
    echo "  shell               Open shell in container"
    echo "  clean              Remove container and image"
    echo ""
    echo "Examples:"
    echo "  $0 build           # Build the application"
    echo "  $0 run             # Build and run"
    echo "  $0 shell            # Open development shell"
}

# Clean function
clean_container() {
    echo "🗑️  Cleaning up..."
    podman rmi -f xmpp-client-builder 2>/dev/null || true
    podman rm -f xmpp-client-dev 2>/dev/null || true
    echo "✅ Cleanup completed"
}

# Build function
build_in_container() {
    echo "🔨 Building in container..."
    
    podman build -f Containerfile -t xmpp-client-builder .
    
    echo "✅ Build completed!"
    echo "📂 Binary location: In container at /app/target/release/xmpp-client"
}

# Run function
run_in_container() {
    echo "🚀 Building and running..."
    
    # Clean up existing container
    podman rm -f xmpp-client-dev 2>/dev/null || true
    
    # Build and run
    podman run -it --rm \
        --name xmpp-client-dev \
        --env DISPLAY=$DISPLAY \
        --env WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
        --env RUST_LOG=info \
        --volume /tmp/.X11-unix:/tmp/.X11-unix:rw \
        --device /dev/dri \
        --net host \
        --security-opt label=disable \
        --volume $PWD:/app:rw \
        xmpp-client-builder \
        cargo run --release
}

# Shell function
shell_in_container() {
    echo "🐚 Opening development shell..."
    
    podman run -it --rm \
        --name xmpp-client-dev-shell \
        --env DISPLAY=$DISPLAY \
        --env WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
        --volume /tmp/.X11-unix:/tmp/.X11-unix:rw \
        --device /dev/dri \
        --net host \
        --security-opt label=disable \
        --volume $PWD:/app:rw \
        xmpp-client-builder \
        bash
}

# Main execution
case "${1:-build}" in
    build)
        build_in_container
        ;;
    run)
        run_in_container
        ;;
    shell)
        shell_in_container
        ;;
    clean)
        clean_container
        ;;
    --help|-h)
        show_usage
        ;;
    *)
        echo "❌ Unknown command: $1"
        show_usage
        exit 1
        ;;
esac