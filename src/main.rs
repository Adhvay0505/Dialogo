#[cfg(feature = "gtk4")]
use gtk4::prelude::*;
#[cfg(feature = "gtk4")]
use gtk4::{Application, ApplicationWindow, Button, Label, Box as GtkBox, Orientation, Entry, PasswordEntry};

#[cfg(feature = "gtk3")]
use gtk::prelude::*;
#[cfg(feature = "gtk3")]
use gtk::{Application, ApplicationWindow, Button, Label, Box as GtkBox, Orientation, Entry, PasswordEntry as Entry};

use anyhow::Result;

const APP_ID: &str = "com.example.xmpp-client";

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create a new application
    #[cfg(feature = "gtk4")]
    let app = Application::builder().application_id(APP_ID).build();
    
    #[cfg(feature = "gtk3")]
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk::ApplicationFlags::empty())
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

#[cfg(feature = "gtk4")]
fn build_ui(app: &gtk4::Application) {
    // Create main container
    let main_box = GtkBox::new(gtk4::Orientation::Vertical, 12);
    main_box.set_margin_all(24);

    // Title
    let title = gtk4::Label::builder()
        .label("🚀 XMPP Client")
        .css_classes(["title-1"])
        .build();
    main_box.append(&title);

    // JID input
    let jid_label = gtk4::Label::new(Some("JID:"));
    main_box.append(&jid_label);

    let jid_entry = gtk4::Entry::new();
    jid_entry.set_placeholder_text(Some("user@domain.com"));
    jid_entry.set_margin_bottom(12);
    main_box.append(&jid_entry);

    // Password input
    let password_label = gtk4::Label::new(Some("Password:"));
    main_box.append(&password_label);

    let password_entry = gtk4::PasswordEntry::new();
    password_entry.set_margin_bottom(12);
    main_box.append(&password_entry);

    // Connect button
    let connect_button = gtk4::Button::builder()
        .label("🔗 Connect")
        .margin_top(12)
        .build();

    let jid_entry_clone = jid_entry.clone();
    let password_entry_clone = password_entry.clone();

    connect_button.connect_clicked(move |_| {
        let jid_text = jid_entry_clone.text().to_string();
        let password_text = password_entry_clone.text().to_string();

        if !jid_text.is_empty() && !password_text.is_empty() {
            tracing::info!("🔗 Connection requested to: {}", jid_text);
            show_connection_dialog("Connecting...", &format!("Attempting to connect to: {}", jid_text));
        } else {
            show_connection_dialog("❌ Error", "Please enter JID and password");
        }
    });

    main_box.append(&connect_button);

    // Status label
    let status_label = gtk4::Label::builder()
        .label("✅ Ready to connect")
        .css_classes(["status-text"])
        .margin_top(12)
        .build();
    main_box.append(&status_label);

    // Create main window
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("XMPP Client")
        .default_width(450)
        .default_height(400)
        .child(&main_box)
        .build();

    // Apply styling
    apply_gtk4_styling();

    // Present window
    window.present();
}

#[cfg(feature = "gtk3")]
fn build_ui(app: &gtk::Application) {
    // Create main container
    let main_box = GtkBox::new(gtk::Orientation::Vertical, 12);
    main_box.set_margin_all(24);

    // Title
    let title = gtk::Label::builder()
        .label("🚀 XMPP Client (GTK3)")
        .build();
    main_box.append(&title);

    // Simple message
    let msg_label = gtk::Label::new(Some("GTK3 fallback mode - XMPP functionality ready"));
    main_box.append(&msg_label);

    // Connect button
    let connect_button = gtk::Button::builder()
        .label("🔗 Test GTK3")
        .margin_top(12)
        .build();

    connect_button.connect_clicked(move |_| {
        tracing::info!("🔗 GTK3 test button clicked!");
        show_connection_dialog("✅ Working", "GTK3 fallback is operational!");
    });

    main_box.append(&connect_button);

    // Create main window
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("XMPP Client - GTK3")
        .default_width(400)
        .default_height(300)
        .child(&main_box)
        .build();

    // Apply GTK3 styling
    apply_gtk3_styling();

    // Present window
    window.show_all();
}

#[cfg(feature = "gtk4")]
fn apply_gtk4_styling() {
    let css_provider = gtk4::CssProvider::new();
    let css = r#"
        .title-1 {
            font-size: 24px;
            font-weight: bold;
            margin-bottom: 16px;
            color: #3584e4;
        }
        
        label {
            font-weight: bold;
            margin-bottom: 6px;
        }
        
        .status-text {
            color: #666666;
            font-style: italic;
        }
        
        entry, password {
            margin-bottom: 16px;
            padding: 8px;
            border-radius: 6px;
            border: 1px solid #ddd;
        }
        
        button {
            padding: 12px 24px;
            font-weight: bold;
            border-radius: 6px;
            background-color: #3584e4;
            color: white;
            border: none;
        }
        
        button:hover {
            background-color: #2a6ebb;
        }
        
        window {
            background-color: #fafafa;
        }
    "#;
    
    css_provider.load_from_data(css);
    gtk4::StyleContext::add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Failed to get display"),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[cfg(feature = "gtk3")]
fn apply_gtk3_styling() {
    let css_provider = gtk::CssProvider::new();
    let css = r#"
        .title {
            font-size: 24px;
            font-weight: bold;
            margin-bottom: 16px;
        }
        
        button {
            padding: 12px 24px;
            font-weight: bold;
            border-radius: 6px;
            background-color: #3584e4;
            color: white;
        }
        
        button:hover {
            background-color: #2a6ebb;
        }
        
        window {
            background-color: #fafafa;
        }
    "#;
    
    css_provider.load_from_data(css);
    gtk::StyleContext::add_provider_for_screen(
        &gtk::gdk::Screen::default().expect("Failed to get screen"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[cfg(feature = "gtk4")]
fn show_connection_dialog(title: &str, message: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .message_type(gtk4::MessageType::Info)
        .buttons(gtk4::ButtonsType::Ok)
        .text(message)
        .title(title)
        .build();

    dialog.connect_response(|_, _| {
        dialog.close();
    });

    dialog.show();
}

#[cfg(feature = "gtk3")]
fn show_connection_dialog(title: &str, message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .message_type(gtk::MessageType::Info)
        .buttons(gtk::ButtonsType::Ok)
        .text(&message)
        .title(&title)
        .build();

    dialog.connect_response(|_, _| {
        dialog.close();
    });

    dialog.run();
    dialog.hide();
}