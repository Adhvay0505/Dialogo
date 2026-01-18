# Containerfile for XMPP Client Development
FROM fedora:39

# Install development tools
RUN dnf update -y && dnf groupinstall -y "Development Tools" "Development Libraries"

# Install GTK4 and dependencies
RUN dnf install -y \
    gcc \
    gcc-c++ \
    pkg-config \
    gtk4-devel \
    glib2-devel \
    cairo-devel \
    pango-devel \
    gdk-pixbuf2-devel \
    sqlite-devel \
    openssl-devel \
    rust \
    cargo

# Install Rust if not present
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    source "$HOME/.cargo/env"

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN cargo build --release

# Create user for running the app
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# Set environment
ENV RUST_LOG=info
ENV DISPLAY=:0

# Run the application
CMD ["./target/release/xmpp-client"]