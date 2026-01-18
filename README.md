# XMPP Client

A modern, full-featured XMPP client built with GTK4 and Rust.

## Features

- 🗣️ **User Authentication**: Connect to any XMPP server with configurable settings
- 👥 **Contact Management**: Full roster support with presence handling and group management
- 💬 **1-on-1 Chat**: Real-time messaging with chat state notifications
- 🏠 **Group Chat**: Multi-User Chat (MUC) support with room management
- 📁 **File Transfer**: Send and receive files with progress tracking
- ⚙️ **Settings**: Comprehensive configuration interface
- 🔔 **Notifications**: Desktop notifications for new messages
- 🎨 **Modern UI**: Clean, intuitive interface built with GTK4 and libadwaita
- 💾 **Message History**: Persistent chat history with search capabilities
- 🔐 **Secure**: TLS encryption support with certificate validation

## Architecture

This project demonstrates modern Rust best practices for building desktop applications:

### Core Components

- **XMPP Protocol Layer** (`src/xmpp/`): Full XMPP client implementation using tokio-xmpp
- **UI Layer** (`src/ui/`): GTK4-based user interface with custom widgets
- **Storage Layer** (`src/storage/`): SQLite database for persistent data
- **Configuration** (`src/config/`): TOML-based configuration management
- **Application State** (`src/app.rs`): Main application coordination

### State Management Pattern

The application uses a robust state management pattern with:
- `Arc<Mutex<>>` for shared state between UI and XMPP threads
- Tokio channels for async communication between components
- GTK4 main loop integration for responsive UI

### Async Integration

Seamless integration between tokio async runtime and GTK4 main loop:
- `glib::MainContext::spawn_local()` for GTK-safe async tasks
- Broadcast channels for event distribution
- Channel-based command pattern for UI-to-XMPP communication

## Building and Running

### Prerequisites

- Rust 1.70+ with `stable` toolchain
- GTK4 development libraries
- SQLite development libraries

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install libgtk-4-dev libsqlite3-dev pkg-config
```

#### Fedora
```bash
sudo dnf install gtk4-devel sqlite-devel pkg-config
```

#### macOS
```bash
brew install gtk4 sqlite pkg-config
```

#### Arch Linux
```bash
sudo pacman -S gtk4 sqlite pkg-config
```

### Building

```bash
git clone <repository-url>
cd xmpp-client
cargo build --release
```

### Running

```bash
cargo run
```

Or run the release binary:
```bash
./target/release/xmpp-client
```

## Configuration

Configuration is stored in:
- **Linux**: `~/.config/xmpp-client/config.toml`
- **macOS**: `~/Library/Application Support/xmpp-client/config.toml`
- **Windows**: `%APPDATA%\xmpp-client\config.toml`

### Example Configuration

```toml
[[accounts]]
jid = "user@domain.com"
password = "your-password"
resource = "xmpp-client"
auto_connect = true
save_password = false

[accounts.server]
host = "domain.com"
port = 5222
use_tls = true
accept_invalid_certs = false

[general]
log_level = "info"
theme = "system"
notification_enabled = true

[file_transfer]
download_dir = "~/Downloads"
max_file_size = 104857600  # 100MB
```

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main application coordination
├── xmpp/                # XMPP protocol implementation
│   ├── mod.rs
│   ├── client.rs         # Core XMPP client
│   ├── events.rs         # Event definitions
│   └── stanza_handler.rs # XMPP stanza processing
├── ui/                  # User interface
│   ├── mod.rs
│   ├── main_window.rs    # Main application window
│   ├── chat_window.rs    # Chat interface
│   ├── roster_window.rs  # Contact list
│   ├── settings_window.rs # Configuration interface
│   ├── dialogs/          # Various dialogs
│   └── widgets/         # Custom UI components
├── storage/             # Database layer
├── config.rs            # Configuration management
└── error.rs             # Error handling
```

### Testing

Run the test suite:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=debug cargo run
```

### Code Style

This project uses standard Rust formatting:
```bash
cargo fmt
cargo clippy
```

## XMPP Protocol Support

### Supported XEPs

- **XEP-0030**: Service Discovery
- **XEP-0045**: Multi-User Chat (MUC)
- **XEP-0082**: XMPP Date and Time
- **XEP-0084**: User Avatar
- **XEP-0198**: Stream Management
- **XEP-0199**: XMPP Ping
- **XEP-0203**: Delayed Delivery
- **XEP-0224**: Message Processing Hints
- **XEP-0280**: Message Carbons
- **XEP-0313**: Message Archive Management
- **XEP-0363**: HTTP File Upload
- **XEP-0384**: OMEMO Encryption (planned)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.

## Acknowledgments

- [tokio-xmpp](https://github.com/xmpp-rs/tokio-xmpp) - XMPP library
- [GTK4-rs](https://github.com/gtk-rs/gtk4-rs) - GTK4 bindings
- [libadwaita](https://github.com/bilelmoussaoui/libadwaita-rs) - Modern GNOME widgets
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- The Rust community for excellent tooling and libraries