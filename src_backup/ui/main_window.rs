use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Builder, 
    Box as GtkBox, Paned, Button, Label, Entry,
    Statusbar, MenuBar, Menu, MenuItem, SeparatorMenuItem,
    HeaderBar, ToggleButton, Stack,
};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow as AdwApplicationWindow, HeaderBar as AdwHeaderBar};
use glib::clone;

use crate::xmpp::{XmppClient, XmppClientConfig, XmppEvent, create_message_jid};
use crate::ui::{setup_application_actions, create_css_provider, WINDOW_WIDTH, WINDOW_HEIGHT};
use crate::ui::chat_window::ChatWindow;
use crate::ui::roster_window::RosterWindow;
use crate::ui::settings_window::SettingsWindow;
use crate::storage::Database;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

pub struct MainWindow {
    app: Application,
    window: AdwApplicationWindow,
    
    // UI Components
    main_stack: Stack,
    header_bar: AdwHeaderBar,
    paned: Paned,
    chat_window: Arc<ChatWindow>,
    roster_window: Arc<RosterWindow>,
    status_bar: Statusbar,
    
    // XMPP Integration
    xmpp_client: Option<Arc<XmppClient>>,
    xmpp_command_tx: Option<mpsc::Sender<crate::xmpp::XmppCommand>>,
    event_rx: Option<broadcast::Receiver<XmppEvent>>,
    
    // Database
    database: Arc<Database>,
    
    // Actions
    connect_btn: ToggleButton,
    disconnect_btn: Button,
    settings_btn: Button,
}

impl MainWindow {
    pub fn new(
        app: Application,
        command_tx: mpsc::Sender<crate::xmpp::XmppCommand>,
        mut event_rx: broadcast::Receiver<XmppEvent>,
        database: Arc<Database>,
    ) -> Self {
        // Create main window
        let window = AdwApplicationWindow::builder()
            .application(&app)
            .title("XMPP Client")
            .default_width(WINDOW_WIDTH)
            .default_height(WINDOW_HEIGHT)
            .build();

        // Create main layout
        let main_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();

        // Create header bar
        let header_bar = AdwHeaderBar::new();
        
        // Create connection buttons
        let connect_btn = ToggleButton::builder()
            .label("Connect")
            .icon_name("network-wired-symbolic")
            .build();

        let disconnect_btn = Button::builder()
            .label("Disconnect")
            .icon_name("network-offline-symbolic")
            .sensitive(false)
            .build();

        let settings_btn = Button::builder()
            .label("Settings")
            .icon_name("preferences-system-symbolic")
            .build();

        // Add buttons to header bar
        header_bar.pack_start(&connect_btn);
        header_bar.pack_start(&disconnect_btn);
        header_bar.pack_end(&settings_btn);

        // Create main stack for different views
        let main_stack = Stack::new();
        
        // Create paned for roster and chat
        let paned = Paned::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .position(300)
            .build();

        // Create chat and roster windows
        let chat_window = Arc::new(ChatWindow::new());
        let roster_window = Arc::new(RosterWindow::new());

        paned.set_start_child(Some(&roster_window.get_widget()));
        paned.set_end_child(Some(&chat_window.get_widget()));

        // Create status bar
        let status_bar = Statusbar::builder()
            .margin_start(10)
            .margin_end(10)
            .build();

        // Assemble main layout
        main_box.append(&header_bar);
        main_stack.add_named(&paned, "main");
        main_stack.set_visible_child_name("main");
        main_box.append(&main_stack);
        main_box.append(&status_bar);

        window.set_content(Some(&main_box));

        // Setup window
        let mut main_window = Self {
            app,
            window,
            main_stack,
            header_bar,
            paned,
            chat_window,
            roster_window,
            status_bar,
            xmpp_client: None,
            xmpp_command_tx: Some(command_tx),
            event_rx: Some(event_rx),
            database,
            connect_btn,
            disconnect_btn,
            settings_btn,
        };

        // Setup connections and event handlers
        main_window.setup_connections();
        main_window.setup_event_handling();

        main_window
    }

    fn setup_connections(&self) {
        // Connect button handler
        self.connect_btn.connect_toggled(clone!(@strong self as this => move |btn| {
            if btn.is_active() {
                // Start connection process
                this.show_connection_dialog();
            } else {
                // Disconnect
                if let Some(tx) = &this.xmpp_command_tx {
                    let _ = tx.try_send(crate::xmpp::XmppCommand::Disconnect);
                }
            }
        }));

        // Disconnect button handler
        self.disconnect_btn.connect_clicked(clone!(@strong self as this => move |_| {
            if let Some(tx) = &this.xmpp_command_tx {
                let _ = tx.try_send(crate::xmpp::XmppCommand::Disconnect);
            }
        }));

        // Settings button handler
        self.settings_btn.connect_clicked(clone!(@strong self as this => move |_| {
            this.show_settings_window();
        }));
    }

    fn setup_event_handling(&mut self) {
        if let Some(mut event_rx) = self.event_rx.take() {
            let chat_window = self.chat_window.clone();
            let roster_window = self.roster_window.clone();
            let connect_btn = self.connect_btn.clone();
            let disconnect_btn = self.disconnect_btn.clone();
            let status_bar = self.status_bar.clone();

            glib::MainContext::default().spawn_local(async move {
                while let Ok(event) = event_rx.recv().await {
                    match event {
                        XmppEvent::Connected { jid } => {
                            connect_btn.set_active(true);
                            connect_btn.set_sensitive(false);
                            disconnect_btn.set_sensitive(true);
                            
                            let context_id = status_bar.get_context_id("connection");
                            status_bar.push(context_id, &format!("Connected as {}", jid));
                            
                            // Request roster
                            if let Some(tx) = &roster_window.get_command_tx() {
                                let _ = tx.try_send(crate::xmpp::XmppCommand::GetRoster);
                            }
                        }
                        XmppEvent::Disconnected { reason } => {
                            connect_btn.set_active(false);
                            connect_btn.set_sensitive(true);
                            disconnect_btn.set_sensitive(false);
                            
                            let context_id = status_bar.get_context_id("connection");
                            status_bar.push(context_id, &format!("Disconnected: {}", reason));
                        }
                        XmppEvent::ConnectionError { error } => {
                            connect_btn.set_active(false);
                            connect_btn.set_sensitive(true);
                            disconnect_btn.set_sensitive(false);
                            
                            let context_id = status_bar.get_context_id("connection");
                            status_bar.push(context_id, &format!("Connection error: {}", error));
                        }
                        XmppEvent::MessageReceived { from, to, body, .. } => {
                            chat_window.add_message(&from, &to, &body, false);
                        }
                        XmppEvent::MessageSent { to, body, .. } => {
                            // Note: We would need the current user's JID here
                            chat_window.add_message(&to, &to, &body, true);
                        }
                        XmppEvent::PresenceReceived { from, show, status, .. } => {
                            roster_window.update_presence(&from, &show, status.as_deref());
                        }
                        XmppEvent::RosterReceived { items } => {
                            roster_window.set_roster(items);
                        }
                        XmppEvent::ChatStateReceived { from, state } => {
                            chat_window.update_chat_state(&from, &format!("{:?}", state));
                        }
                        XmppEvent::MucMessageReceived { room_jid, from, nickname, body, .. } => {
                            chat_window.add_groupchat_message(&room_jid, &nickname, &body);
                        }
                        XmppEvent::SubscriptionRequest { from } => {
                            // Show subscription request dialog
                            Self::show_subscription_request_dialog(&from);
                        }
                        _ => {
                            tracing::debug!("Unhandled XMPP event: {:?}", event);
                        }
                    }
                }
            });
        }
    }

    pub fn show(&self) {
        self.window.show();
    }

    fn show_connection_dialog(&self) {
        let dialog = gtk4::MessageDialog::builder()
            .title("Connect to XMPP Server")
            .message_type(gtk4::MessageType::Question)
            .buttons(gtk4::ButtonsType::OkCancel)
            .text("Enter your XMPP credentials")
            .secondary_text("This will be replaced with a proper connection dialog")
            .modal(true)
            .transient_for(&self.window)
            .build();

        dialog.connect_response(None, clone!(@strong self as this => move |dialog, response| {
            if response == gtk4::ResponseType::Ok {
                // TODO: Show proper connection dialog with form fields
                // For now, just connect with default config
                this.connect_with_default_config();
            }
            dialog.close();
        }));

        dialog.show();
    }

    fn connect_with_default_config(&self) {
        let config = XmppClientConfig {
            jid: "user@localhost".to_string(),
            password: "password".to_string(),
            resource: "xmpp-client".to_string(),
            server_host: "localhost".to_string(),
            server_port: 5222,
            use_tls: false,
            accept_invalid_certs: true,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay: std::time::Duration::from_secs(10),
        };

        if let Some(tx) = &self.xmpp_command_tx {
            let _ = tx.try_send(crate::xmpp::XmppCommand::Connect);
        }
    }

    fn show_settings_window(&self) {
        let settings_window = SettingsWindow::new(&self.window, self.database.clone());
        settings_window.show();
    }

    fn show_subscription_request_dialog(from: &xmpp_parsers::Jid) {
        // TODO: Implement subscription request dialog
        tracing::info!("Subscription request from: {}", from);
    }
}