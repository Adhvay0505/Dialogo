use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, gio::SimpleActionGroup};
use libadwaita::prelude::*;
use libadwaita::{Application as AdwApplication, ApplicationWindow as AdwApplicationWindow};

mod app;
mod xmpp;
mod ui;
mod storage;
mod config;
mod error;

use app::XmppApp;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Create communication channels
    let (command_tx, command_rx) = mpsc::channel(1000);
    let (event_tx, event_rx) = broadcast::channel(1000);

    // Initialize database
    let database = Arc::new(
        storage::Database::new("sqlite:xmpp-client.db")
            .await
            .expect("Failed to initialize database")
    );

    // Create GTK application
    let app = AdwApplication::new(
        Some("com.example.xmpp-client"),
        gio::ApplicationFlags::HANDLES_OPEN,
    );

    // Connect to activate signal
    app.connect_activate(move |app| {
        // Create the main application instance
        let xmpp_app = XmppApp::new(
            app.clone(),
            command_tx.clone(),
            event_rx.subscribe(),
            database.clone(),
        );

        // Run the application in the GTK main context
        glib::MainContext::default().spawn_local(async move {
            xmpp_app.run().await;
        });
    });

    // Set up application metadata
    app.set_resource_base_path(Some("/com/example/xmpp-client"));
    
    // Run the application
    app.run();
}