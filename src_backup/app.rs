use crate::storage::Database;
use crate::xmpp::XmppCommand;
use crate::error::{XmppResult, XmppError};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, gio::SimpleAction};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use crate::ui::{MainWindow, setup_application_actions};
use crate::xmpp::{XmppClient, XmppClientConfig, XmppEvent};
use crate::config::ConfigManager;

pub struct XmppApp {
    app: Application,
    main_window: MainWindow,
    xmpp_client: Option<Arc<XmppClient>>,
    database: Arc<Database>,
    
    // Communication channels
    command_tx: mpsc::Sender<XmppCommand>,
    event_rx: broadcast::Receiver<XmppEvent>,
    config_manager: ConfigManager,
}

impl XmppApp {
    pub fn new(
        app: Application,
        command_tx: mpsc::Sender<XmppCommand>,
        event_rx: broadcast::Receiver<XmppEvent>,
        database: Arc<Database>,
    ) -> Self {
        let config_manager = ConfigManager::new()
            .expect("Failed to create config manager");

        // Create main window
        let main_window = MainWindow::new(
            app.clone(),
            command_tx.clone(),
            event_rx,
            database.clone(),
        );

        Self {
            app,
            main_window,
            xmpp_client: None,
            database,
            command_tx,
            event_rx,
            config_manager,
        }
    }

    pub async fn run(mut self) {
        // Show main window
        self.main_window.show();

        // Setup application actions
        self.setup_actions();

        // Load configuration and auto-connect if enabled
        self.handle_auto_connect().await;

        // Start the main event processing loop
        self.process_events().await;
    }

    fn setup_actions(&self) {
        // Connect action
        let connect_action = gio::SimpleAction::new("connect", None);
        connect_action.connect_activate(clone!(@strong self.command_tx as tx => move |_, _| {
            let _ = tx.try_send(XmppCommand::Connect);
        }));
        self.app.add_action(&connect_action);

        // Disconnect action
        let disconnect_action = gio::SimpleAction::new("disconnect", None);
        disconnect_action.connect_activate(clone!(@strong self.command_tx as tx => move |_, _| {
            let _ = tx.try_send(XmppCommand::Disconnect);
        }));
        self.app.add_action(&disconnect_action);

        // Settings action
        let settings_action = gio::SimpleAction::new("settings", None);
        settings_action.connect_activate(clone!(@strong self.main_window as window => move |_, _| {
            window.show_settings_window();
        }));
        self.app.add_action(&settings_action);

        // About action
        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(@strong self.main_window as window => move |_, _| {
            crate::ui::dialogs::AboutDialog::new(&window.get_widget()).show();
        }));
        self.app.add_action(&about_action);

        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(@strong self.app => move |_, _| {
            app.quit();
        }));
        self.app.add_action(&quit_action);

        // Enable keyboard shortcuts
        self.setup_keyboard_shortcuts();
    }

    fn setup_keyboard_shortcuts(&self) {
        let app = &self.app;
        
        // Ctrl+C - Connect
        app.set_accels_for_action("app.connect", &["<Primary>c"]);
        
        // Ctrl+D - Disconnect
        app.set_accels_for_action("app.disconnect", &["<Primary>d"]);
        
        // Ctrl+Shift+S - Settings
        app.set_accels_for_action("app.settings", &["<Primary><Shift>s"]);
        
        // Ctrl+Q - Quit
        app.set_accels_for_action("app.quit", &["<Primary>q"]);
        
        // F1 - About
        app.set_accels_for_action("app.about", &["F1"]);
    }

    async fn handle_auto_connect(&mut self) {
        if let Ok(config) = self.config_manager.load_config() {
            // Find the default account or first account
            let account = if let Some(default_jid) = config.default_account {
                config.accounts.iter()
                    .find(|acc| acc.jid == default_jid)
            } else {
                config.accounts.first()
            };

            if let Some(acc) = account {
                if acc.auto_connect {
                    tracing::info!("Auto-connecting to account: {}", acc.jid);
                    
                    let client_config = XmppClientConfig {
                        jid: acc.jid.clone(),
                        password: acc.password.clone(),
                        resource: acc.resource.clone(),
                        server_host: acc.server.host.clone(),
                        server_port: acc.server.port,
                        use_tls: acc.server.use_tls,
                        accept_invalid_certs: acc.server.accept_invalid_certs,
                        auto_reconnect: true,
                        max_reconnect_attempts: 5,
                        reconnect_delay: std::time::Duration::from_secs(10),
                    };

                    // Create XMPP client
                    let (event_tx, _) = broadcast::channel(1000);
                    if let Ok((client, _)) = XmppClient::new(
                        client_config,
                        self.database.clone(),
                        event_tx,
                    ) {
                        self.xmpp_client = Some(Arc::new(client));
                        
                        // Send connect command
                        let _ = self.command_tx.try_send(XmppCommand::Connect);
                    }
                }
            }
        }
    }

    async fn process_events(&mut self) {
        let mut command_rx = self.command_tx.clone();
        
        // Listen for XMPP events
        let mut event_rx = self.event_rx.resubscribe();
        
        loop {
            tokio::select! {
                // Handle XMPP events
                Some(event) = event_rx.recv() => {
                    self.handle_xmpp_event(event).await;
                }
                
                // Handle shutdown signal (GTK application closed)
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received shutdown signal");
                    if let Some(_client) = &self.xmpp_client {
                        let _ = self.command_tx.try_send(XmppCommand::Disconnect);
                    }
                    break;
                }
            }
        }
    }

    async fn handle_xmpp_event(&mut self, event: XmppEvent) {
        match event {
            XmppEvent::Connected { jid } => {
                tracing::info!("Connected as {}", jid);
                // UI will be updated through the main window's event handling
            }
            XmppEvent::Disconnected { reason } => {
                tracing::info!("Disconnected: {}", reason);
                // Handle reconnection logic if needed
            }
            XmppEvent::ConnectionError { error } => {
                tracing::error!("Connection error: {}", error);
                // Show error notification
            }
            XmppEvent::AuthenticationError { error } => {
                tracing::error!("Authentication error: {}", error);
                // Show authentication error dialog
            }
            XmppEvent::MessageReceived { from, to, body, .. } => {
                tracing::debug!("Message from {} to {}: {}", from, to, body);
                // UI handles this automatically through the main window's event subscription
            }
            XmppEvent::PresenceReceived { from, show, status, .. } => {
                tracing::debug!("Presence from {}: {} - {}", from, show, status.unwrap_or_default());
                // UI handles this automatically
            }
            XmppEvent::RosterReceived { items } => {
                tracing::info!("Received roster with {} items", items.len());
                // UI handles this automatically
            }
            XmppEvent::SubscriptionRequest { from } => {
                tracing::info!("Subscription request from {}", from);
                // Show subscription request dialog
            }
            XmppEvent::FileTransferRequest { from, filename, size, .. } => {
                tracing::info!("File transfer request from {}: {} ({} bytes)", from, filename, size);
                // Show file transfer dialog
            }
            XmppEvent::MucMessageReceived { room_jid, from, nickname, body, .. } => {
                tracing::debug!("MUC message from {} in {}: {}", nickname, room_jid, body);
                // UI handles this automatically
            }
            XmppEvent::MucUserJoined { room_jid, nickname, .. } => {
                tracing::info!("User {} joined room {}", nickname, room_jid);
                // Update MUC participant list
            }
            XmppEvent::MucUserLeft { room_jid, nickname } => {
                tracing::info!("User {} left room {}", nickname, room_jid);
                // Update MUC participant list
            }
            XmppEvent::Error { error, stanza } => {
                tracing::error!("XMPP Error: {} (stanza: {:?})", error, stanza);
                // Show error dialog
            }
            XmppEvent::StanzaError { from, error_type, condition, text } => {
                tracing::error!("Stanza error from {}: {} - {} ({})", 
                    from, error_type, condition, text.unwrap_or_default());
                // Show stanza error dialog
            }
            _ => {
                tracing::debug!("Unhandled XMPP event: {:?}", event);
            }
        }
    }

    pub fn get_main_window(&self) -> &MainWindow {
        &self.main_window
    }
}