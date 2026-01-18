# Development Guide

This guide covers the development process, coding standards, and best practices for contributing to the XMPP client.

## Getting Started

### Prerequisites

- Rust 1.70+ with stable toolchain
- GTK4 development libraries
- SQLite development libraries
- Git

### Setup

1. Clone the repository
```bash
git clone https://github.com/example/xmpp-client.git
cd xmpp-client
```

2. Install dependencies
```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev libsqlite3-dev pkg-config

# Fedora
sudo dnf install gtk4-devel sqlite-devel pkg-config

# macOS
brew install gtk4 sqlite pkg-config
```

3. Build the project
```bash
cargo build
```

4. Run tests
```bash
cargo test
```

## Development Workflow

### Branch Strategy

- `main`: Stable, production-ready code
- `develop`: Integration branch for features
- `feature/*`: Feature development branches
- `bugfix/*`: Bug fix branches
- `release/*`: Release preparation branches

### Commit Message Format

Follow conventional commits format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting (no logic changes)
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Maintenance tasks

Examples:
```
feat(ui): add typing indicators in chat windows

Add real-time typing indicators that show when a contact is
composing a message. The indicators appear below the chat
input area and disappear after 10 seconds of inactivity.

Closes #123
```

```
fix(xmpp): handle connection timeouts gracefully

Add proper timeout handling for XMPP connections to prevent
infinite hanging when server is unresponsive.
```

### Code Review Process

1. Create a feature branch from `develop`
2. Implement your changes with tests
3. Ensure all tests pass and code is formatted
4. Submit a pull request with:
   - Clear description of changes
   - Related issues linked
   - Testing performed
   - Screenshots for UI changes

## Coding Standards

### Rust Style Guidelines

1. Follow `rustfmt` formatting
```bash
cargo fmt
```

2. Use `clippy` for linting
```bash
cargo clippy -- -D warnings
```

3. Use meaningful variable and function names
4. Document public APIs with `///` doc comments
5. Use `Result<T, Error>` for error handling
6. Prefer `Arc<Mutex<>>` over `Rc<RefCell<>>` in async contexts

### Code Organization

#### Module Structure
```rust
// src/module/mod.rs
pub use self::submodule::SubModule;
pub use self::types::{Type1, Type2};

mod submodule;
mod types;

pub mod public_api;
```

#### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, ModuleError>;
```

#### Async Patterns
```rust
use tokio::sync::mpsc;

pub async fn process_messages(mut rx: mpsc::Receiver<Message>) -> Result<()> {
    while let Some(message) = rx.recv().await {
        match message {
            Message::Text(text) => handle_text_message(text).await?,
            Message::Binary(data) => handle_binary_message(data).await?,
            Message::Close => break,
        }
    }
    Ok(())
}
```

### GTK4 Integration Patterns

#### Widget Creation
```rust
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, Label};

pub struct MyWidget {
    widget: gtk4::Box,
    button: Button,
    label: Label,
}

impl MyWidget {
    pub fn new() -> Self {
        let widget = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .build();

        let button = Button::builder()
            .label("Click me")
            .build();

        let label = Label::builder()
            .label("Ready")
            .build();

        // Connect signals
        button.connect_clicked(clone!(@strong label => move |_| {
            label.set_text("Clicked!");
        }));

        widget.append(&button);
        widget.append(&label);

        Self { widget, button, label }
    }

    pub fn get_widget(&self) -> &gtk4::Box {
        &self.widget
    }
}
```

#### Async Integration
```rust
use glib::MainContext;

pub fn setup_async_handler() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    // Spawn async task in GTK main context
    MainContext::default().spawn_local(async move {
        while let Some(data) = rx.recv().await {
            // Process data asynchronously
            handle_async_data(data).await;
        }
    });
}
```

## Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_function_name() -> Result<()> {
        // Arrange
        let input = "test_input";
        
        // Act
        let result = function_under_test(input).await?;
        
        // Assert
        assert_eq!(result, "expected_output");
        Ok(())
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use xmpp_client::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = format!("sqlite:{}", temp_dir.path().join("test.db").display());
    
    // Test the complete workflow
    let database = Database::new(&db_path).await?;
    let config = XmppClientConfig::default();
    
    // ... test implementation
    
    Ok(())
}
```

### UI Testing

```rust
#[test]
fn test_widget_creation() {
    let widget = MyWidget::new();
    
    // Verify widget structure
    assert!(widget.get_widget().first_child().is_some());
}
```

## Debugging

### Logging

Configure logging with environment variables:

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable specific module logging
RUST_LOG= xmpp_client::xmpp=debug cargo run

# Enable trace logging with file output
RUST_LOG=trace cargo run 2>&1 | tee debug.log
```

### Common Issues

1. **GTK4 initialization errors**
   - Ensure GTK4 development libraries are installed
   - Check `pkg-config --modversion gtk4`

2. **Database connection errors**
   - Verify SQLite development libraries
   - Check database file permissions
   - Ensure correct URI format

3. **Async context errors**
   - Use `glib::MainContext::spawn_local()` for GTK integration
   - Avoid blocking operations in async functions

### Debugging Tools

```rust
// Add debug logging
tracing::debug!("Processing XMPP stanza: {:?}", stanza);
tracing::error!("Connection failed: {}", error);

// Debug async tasks
tokio::spawn(async move {
    tracing::info!("Background task started");
    // ... task implementation
});
```

## Performance Profiling

### CPU Profiling

```bash
# Install profiler
cargo install cargo-profdata

# Profile with debug symbols
cargo build --release
perf record --call-graph=dwarf ./target/release/xmpp-client
perf report
```

### Memory Profiling

```bash
# Install memory profiler
cargo install cargo-flamegraph

# Generate flamegraph
cargo flamegraph --bin xmpp-client
```

### Database Performance

```sql
-- Enable query logging
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 10000;

-- Analyze query performance
EXPLAIN QUERY PLAN SELECT * FROM messages WHERE from_jid = ? ORDER BY created_at;
```

## Documentation

### Code Documentation

```rust
/// Represents an XMPP client connection
/// 
/// The XmppClient handles all protocol-level communication with an XMPP server.
/// It manages connection state, processes incoming stanzas, and sends outgoing messages.
/// 
/// # Examples
/// 
/// ```rust
/// let config = XmppClientConfig::default();
/// let client = XmppClient::new(config)?;
/// client.connect().await?;
/// ```
/// 
/// # Errors
/// 
/// - [`XmppError::AuthenticationError`] if credentials are invalid
/// - [`XmppError::ConnectionError`] if server is unreachable
/// 
/// # See Also
/// 
/// - [`XmppClientConfig`] for configuration options
/// - [`XmppEvent`] for handling client events
pub struct XmppClient {
    // ... implementation
}
```

### User Documentation

- Update README.md with new features
- Add screenshots for UI changes
- Document configuration options
- Provide troubleshooting guides

## Release Process

### Version Management

Use semantic versioning (MAJOR.MINOR.PATCH):
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes (backward compatible)

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Perform security review
5. Create release tag
6. Build release packages
7. Update documentation
8. Deploy to distribution channels

### Automated Releases

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build release
        run: cargo build --release
      - name: Create release
        uses: actions/create-release@v1
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
```

## Contributing Guidelines

### Before Contributing

1. Read the codebase to understand patterns
2. Check for existing issues or pull requests
3. Discuss major changes in an issue first
4. Follow the coding standards above

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

### Code Review Guidelines

1. **Functionality**: Does the code work as intended?
2. **Security**: Are there any security concerns?
3. **Performance**: Will this impact performance?
4. **Maintainability**: Is the code clear and maintainable?
5. **Testing**: Are tests comprehensive?
6. **Documentation**: Is the code well documented?

## Community and Support

- **Issues**: Use GitHub Issues for bug reports and feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Discord/Matrix**: Community chat for real-time discussion
- **Email**: security@example.com for security issues

Thank you for contributing to the XMPP client project!