pub mod main_window;
pub mod chat_window;
pub mod roster_window;
pub mod settings_window;
pub mod dialogs;
pub mod widgets;

pub use main_window::MainWindow;
pub use chat_window::ChatWindow;
pub use roster_window::RosterWindow;
pub use settings_window::SettingsWindow;

use gtk4::prelude::*;
use gtk4::{gio, glib};
use std::sync::Arc;

// UI Constants
pub const WINDOW_WIDTH: i32 = 1200;
pub const WINDOW_HEIGHT: i32 = 800;
pub const CHAT_WIDTH: i32 = 400;
pub const ROSTER_WIDTH: i32 = 300;

// Theme and styling
pub const APPLICATION_ID: &str = "com.example.xmpp-client";
pub const APPLICATION_NAME: &str = "XMPP Client";
pub const APPLICATION_VERSION: &str = "0.1.0";

// Color scheme
pub mod colors {
    pub const PRIMARY: &str = "#3584e4";
    pub const ACCENT: &str = "#99c1f1";
    pub const SUCCESS: &str = "#26a269";
    pub const WARNING: &str = "#e5a50a";
    pub const ERROR: &str = "#c01c28";
    pub const BACKGROUND: &str = "#ffffff";
    pub const SURFACE: &str = "#f6f5f4";
    pub const TEXT_PRIMARY: &str = "#241f31";
    pub const TEXT_SECONDARY: &str = "#666666";
}

// Utility functions for UI
pub fn setup_application_actions(app: &gtk4::Application) {
    // Connect action
    let connect_action = gio::SimpleAction::new("connect", None);
    connect_action.connect_activate(|_, _| {
        // Handle connect action
        tracing::info!("Connect action triggered");
    });
    app.add_action(&connect_action);

    // Disconnect action
    let disconnect_action = gio::SimpleAction::new("disconnect", None);
    disconnect_action.connect_activate(|_, _| {
        // Handle disconnect action
        tracing::info!("Disconnect action triggered");
    });
    app.add_action(&disconnect_action);

    // Settings action
    let settings_action = gio::SimpleAction::new("settings", None);
    settings_action.connect_activate(|_, _| {
        // Handle settings action
        tracing::info!("Settings action triggered");
    });
    app.add_action(&settings_action);

    // About action
    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate(|_, _| {
        // Handle about action
        tracing::info!("About action triggered");
    });
    app.add_action(&about_action);

    // Quit action
    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate(|_, app| {
        // Handle quit action
        tracing::info!("Quit action triggered");
        app.application_id();
    });
    app.add_action(&quit_action);
}

pub fn create_css_provider() -> gtk4::CssProvider {
    let provider = gtk4::CssProvider::new();
    let css = format!(r#"
        .chat-message {
            border-radius: 8px;
            padding: 8px;
            margin: 2px;
        }

        .message-sent {{
            background-color: {};
            color: white;
            margin-left: 40px;
        }}

        .message-received {{
            background-color: {};
            color: {};
            margin-right: 40px;
        }}

        .roster-item {{
            padding: 8px;
            border-bottom: 1px solid #e0e0e0;
        }}

        .roster-item:hover {{
            background-color: #f0f0f0;
        }}

        .online-indicator {{
            color: {};
            font-weight: bold;
        }}

        .offline-indicator {{
            color: #666666;
        }}

        .away-indicator {{
            color: {};
        }}

        .status-text {{
            font-size: 0.9em;
            color: #666666;
        }}

        .chat-input {{
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            padding: 8px;
        }}

        .header-bar {{
            background-color: {};
            color: white;
            border: none;
        }}

        .room-header {{
            background-color: {};
            color: white;
            padding: 16px;
            border-radius: 8px 8px 0 0;
        }}
    "#,
        colors::PRIMARY,
        colors::SURFACE,
        colors::TEXT_PRIMARY,
        colors::SUCCESS,
        colors::WARNING,
        colors::PRIMARY,
        colors::ACCENT
    );

    provider.load_from_data(&css);
    provider
}

pub fn setup_dark_mode_support() {
    let style_manager = libadwaita::StyleManager::default();
    style_manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);
}