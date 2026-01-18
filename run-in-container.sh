#!/bin/bash

# Podman Development Container for XMPP Client

set -e

CONTAINER_NAME="xmpp-client-dev"
IMAGE_NAME="xmpp-client-builder"

echo "🐳 Building XMPP Client with Podman..."

# Clean up any existing container
if podman container exists $CONTAINER_NAME 2>/dev/null; then
    echo "🗑️  Removing existing container..."
    podman rm -f $CONTAINER_NAME
fi

# Build the development image
echo "🔨 Building development image..."
podman build -f Containerfile -t $IMAGE_NAME .

# Run the development container with X11 forwarding
echo "🚀 Starting development container..."
podman run -it --rm \
    --name $CONTAINER_NAME \
    --env DISPLAY=$DISPLAY \
    --env WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
    --volume /tmp/.X11-unix:/tmp/.X11-unix:rw \
    --volume $PWD:/app:rw \
    --device /dev/dri \
    --net host \
    $IMAGE_NAME

echo "✅ Container started. The XMPP client should now be running in the container."