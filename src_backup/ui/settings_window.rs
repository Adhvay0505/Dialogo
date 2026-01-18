use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Window, Dialog, Box as GtkBox, 
    Entry, Label, Button, Switch, SpinButton, FileChooserButton,
    Grid, Frame, HeaderBar, Stack, StackSwitcher,
};
use libadwaita::prelude::*;
use libadwaita::{
    ApplicationWindow as AdwApplicationWindow,
    HeaderBar as AdwHeaderBar,
    PreferencesGroup, PreferencesRow, PreferencesWindow,
    ActionRow, EntryRow, SpinRow, SwitchRow,
};
use std::sync::Arc;
use crate::config::{ConfigManager, AppConfig, AccountConfig, ServerConfig};
use crate::storage::Database;

pub struct SettingsWindow {
    window: PreferencesWindow,
    config_manager: ConfigManager,
    config: AppConfig,
    database: Arc<Database>,
}

impl SettingsWindow {
    pub fn new(parent: &impl IsA<gtk4::Window>, database: Arc<Database>) -> Self {
        let config_manager = ConfigManager::new().expect("Failed to create config manager");
        let mut config = config_manager.load_config().unwrap_or_default();

        // Create preferences window
        let window = PreferencesWindow::builder()
            .title("Settings")
            .modal(true)
            .transient_for(parent)
            .default_width(800)
            .default_height(600)
            .build();

        let mut settings_window = Self {
            window,
            config_manager,
            config,
            database,
        };

        settings_window.setup_pages();
        settings_window
    }

    fn setup_pages(&mut self) {
        // Accounts page
        self.setup_accounts_page();

        // General settings page
        self.setup_general_page();

        // Notifications page
        self.setup_notifications_page();

        // File transfer page
        self.setup_file_transfer_page();

        // Advanced page
        self.setup_advanced_page();
    }

    fn setup_accounts_page(&mut self) {
        let page = libadwaita::PreferencesPage::builder()
            .title("Accounts")
            .icon_name("avatar-default-symbolic")
            .build();

        let accounts_group = PreferencesGroup::builder()
            .title("XMPP Accounts")
            .description("Manage your XMPP account configurations")
            .build();

        // Add existing accounts
        for (index, account) in self.config.accounts.iter().enumerate() {
            let account_row = ActionRow::builder()
                .title(&account.jid)
                .subtitle(account.server.host.clone())
                .activatable(true)
                .build();

            let edit_button = Button::builder()
                .label("Edit")
                .valign(gtk4::Align::Center)
                .build();

            let remove_button = Button::builder()
                .label("Remove")
                .valign(gtk4::Align::Center)
                .css_classes(vec!["destructive-action".to_string()])
                .build();

            let button_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .build();

            button_box.append(&edit_button);
            button_box.append(&remove_button);

            account_row.add_suffix(&button_box);

            // Connect edit button
            let account_clone = account.clone();
            edit_button.connect_clicked(clone!(@strong self as this => move |_| {
                this.edit_account(&account_clone);
            }));

            // Connect remove button
            let account_jid = account.jid.clone();
            remove_button.connect_clicked(clone!(@strong self as this => move |_| {
                this.remove_account(&account_jid);
            }));

            accounts_group.add(&account_row);
        }

        // Add new account button
        let add_account_row = ActionRow::builder()
            .title("Add Account")
            .subtitle("Configure a new XMPP account")
            .activatable(true)
            .icon_name("list-add-symbolic")
            .build();

        add_account_row.connect_activated(clone!(@strong self as this => move |_| {
            this.add_new_account();
        }));

        accounts_group.add(&add_account_row);

        page.add(&accounts_group);
        self.window.add(&page);
    }

    fn setup_general_page(&mut self) {
        let page = libadwaita::PreferencesPage::builder()
            .title("General")
            .icon_name("preferences-system-symbolic")
            .build();

        // Interface group
        let interface_group = PreferencesGroup::builder()
            .title("Interface")
            .build();

        let theme_row = libadwaita::ComboRow::builder()
            .title("Theme")
            .subtitle("Choose application theme")
            .model(&libadwaita::StringList::new(&["System", "Light", "Dark"]))
            .build();

        interface_group.add(&theme_row);

        // Logging group
        let logging_group = PreferencesGroup::builder()
            .title("Logging")
            .description("Configure application logging level")
            .build();

        let log_level_row = libadwaita::ComboRow::builder()
            .title("Log Level")
            .model(&libadwaita::StringList::new(&["Error", "Warn", "Info", "Debug", "Trace"]))
            .build();

        logging_group.add(&log_level_row);

        // Set current values
        let theme_index = match self.config.theme.as_str() {
            "system" => 0,
            "light" => 1,
            "dark" => 2,
            _ => 0,
        };
        theme_row.set_selected(theme_index);

        let log_index = match self.config.log_level.as_str() {
            "error" => 0,
            "warn" => 1,
            "info" => 2,
            "debug" => 3,
            "trace" => 4,
            _ => 2,
        };
        log_level_row.set_selected(log_index);

        page.add(&interface_group);
        page.add(&logging_group);
        self.window.add(&page);
    }

    fn setup_notifications_page(&mut self) {
        let page = libadwaita::PreferencesPage::builder()
            .title("Notifications")
            .icon_name("preferences-system-notifications-symbolic")
            .build();

        let notifications_group = PreferencesGroup::builder()
            .title("Desktop Notifications")
            .build();

        let enable_notifications_row = SwitchRow::builder()
            .title("Enable Notifications")
            .subtitle("Show desktop notifications for new messages")
            .build();

        enable_notifications_row.set_active(self.config.notification_enabled);

        let message_notifications_row = SwitchRow::builder()
            .title("Message Notifications")
            .subtitle("Notify when receiving new messages")
            .sensitive(self.config.notification_enabled)
            .build();

        let presence_notifications_row = SwitchRow::builder()
            .title("Presence Notifications")
            .subtitle("Notify when contacts come online or go offline")
            .sensitive(self.config.notification_enabled)
            .build();

        notifications_group.add(&enable_notifications_row);
        notifications_group.add(&message_notifications_row);
        notifications_group.add(&presence_notifications_row);

        page.add(&notifications_group);
        self.window.add(&page);
    }

    fn setup_file_transfer_page(&mut self) {
        let page = libadwaita::PreferencesPage::builder()
            .title("File Transfer")
            .icon_name("folder-documents-symbolic")
            .build();

        // Download location group
        let download_group = PreferencesGroup::builder()
            .title("Download Location")
            .build();

        let download_row = libadwaita::ActionRow::builder()
            .title("Download Folder")
            .subtitle(&self.config.file_transfer_dir.to_string_lossy())
            .activatable(true)
            .build();

        let choose_button = Button::builder()
            .label("Choose...")
            .valign(gtk4::Align::Center)
            .build();

        download_row.add_suffix(&choose_button);

        // File size limits group
        let limits_group = PreferencesGroup::builder()
            .title("File Size Limits")
            .build();

        let max_file_size_row = SpinRow::builder()
            .title("Maximum File Size")
            .subtitle("Maximum size for file transfers (MB)")
            .build();

        max_file_size_row.set_range(1.0, 1024.0);
        max_file_size_row.set_value((self.config.max_file_size / (1024 * 1024)) as f64);

        limits_group.add(&max_file_size_row);

        download_group.add(&download_row);
        page.add(&download_group);
        page.add(&limits_group);
        self.window.add(&page);
    }

    fn setup_advanced_page(&mut self) {
        let page = libadwaita::PreferencesPage::builder()
            .title("Advanced")
            .icon_name("preferences-system-network-symbolic")
            .build();

        // Connection group
        let connection_group = PreferencesGroup::builder()
            .title("Connection Settings")
            .description("Advanced connection configuration")
            .build();

        let auto_reconnect_row = SwitchRow::builder()
            .title("Auto Reconnect")
            .subtitle("Automatically reconnect when connection is lost")
            .active(true)
            .build();

        let keepalive_row = SpinRow::builder()
            .title("Keepalive Interval")
            .subtitle("XMPP keepalive interval (seconds)")
            .range(30.0, 300.0)
            .value(60.0)
            .build();

        let timeout_row = SpinRow::builder()
            .title("Connection Timeout")
            .subtitle("Connection timeout (seconds)")
            .range(10.0, 120.0)
            .value(30.0)
            .build();

        connection_group.add(&auto_reconnect_row);
        connection_group.add(&keepalive_row);
        connection_group.add(&timeout_row);

        // Message history group
        let history_group = PreferencesGroup::builder()
            .title("Message History")
            .build();

        let history_limit_row = SpinRow::builder()
            .title("History Limit")
            .subtitle("Maximum number of messages to keep in chat history")
            .range(100.0, 10000.0)
            .value(self.config.message_history_limit as f64)
            .build();

        history_group.add(&history_limit_row);

        page.add(&connection_group);
        page.add(&history_group);
        self.window.add(&page);
    }

    fn add_new_account(&mut self) {
        let dialog = gtk4::Window::builder()
            .title("Add XMPP Account")
            .modal(true)
            .default_width(500)
            .default_height(600)
            .transient_for(&self.window)
            .build();

        let header_bar = AdwHeaderBar::builder()
            .title_widget(&gtk4::Label::new(Some("Add XMPP Account")))
            .build();

        let content = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .spacing(12)
            .build();

        // Account information group
        let account_group = PreferencesGroup::builder()
            .title("Account Information")
            .build();

        let jid_row = EntryRow::builder()
            .title("JID")
            .subtitle("your-jid@domain.com")
            .build();

        let password_row = libadwaita::PasswordEntryRow::builder()
            .title("Password")
            .build();

        let resource_row = EntryRow::builder()
            .title("Resource")
            .text("xmpp-client")
            .build();

        account_group.add(&jid_row);
        account_group.add(&password_row);
        account_group.add(&resource_row);

        // Server configuration group
        let server_group = PreferencesGroup::builder()
            .title("Server Configuration")
            .build();

        let host_row = EntryRow::builder()
            .title("Server")
            .subtitle("domain.com")
            .build();

        let port_row = SpinRow::builder()
            .title("Port")
            .range(1.0, 65535.0)
            .value(5222.0)
            .build();

        let tls_row = SwitchRow::builder()
            .title("Use TLS")
            .active(true)
            .build();

        let invalid_certs_row = SwitchRow::builder()
            .title("Accept Invalid Certificates")
            .subtitle("Only for testing purposes")
            .active(false)
            .build();

        server_group.add(&host_row);
        server_group.add(&port_row);
        server_group.add(&tls_row);
        server_group.add(&invalid_certs_row);

        // Options group
        let options_group = PreferencesGroup::builder()
            .title("Options")
            .build();

        let auto_connect_row = SwitchRow::builder()
            .title("Auto Connect")
            .active(false)
            .build();

        let save_password_row = SwitchRow::builder()
            .title("Save Password")
            .active(false)
            .build();

        options_group.add(&auto_connect_row);
        options_group.add(&save_password_row);

        // Buttons
        let button_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .margin_top(12)
            .build();

        let cancel_button = Button::builder()
            .label("Cancel")
            .build();

        let save_button = Button::builder()
            .label("Save")
            .css_classes(vec!["suggested-action".to_string()])
            .build();

        button_box.append(&cancel_button);
        button_box.append(&save_button);

        // Assemble dialog
        content.append(&account_group);
        content.append(&server_group);
        content.append(&options_group);
        content.append(&button_box);

        dialog.set_titlebar(Some(&header_bar));
        dialog.set_content(Some(&content));

        // Connect buttons
        cancel_button.connect_clicked(clone!(@strong dialog => move |_| {
            dialog.close();
        }));

        save_button.connect_clicked(clone!(@strong self as this, @strong dialog, @strong jid_row, @strong password_row,
                                        @strong resource_row, @strong host_row, @strong port_row,
                                        @strong tls_row, @strong invalid_certs_row,
                                        @strong auto_connect_row, @strong save_password_row => move |_| {
            let jid = jid_row.text().to_string();
            let password = password_row.text().to_string();
            let resource = resource_row.text().to_string();
            let host = host_row.text().to_string();
            let port = port_row.value() as u16;
            let use_tls = tls_row.is_active();
            let accept_invalid_certs = invalid_certs_row.is_active();
            let auto_connect = auto_connect_row.is_active();
            let save_password = save_password_row.is_active();

            let account = AccountConfig {
                jid: jid.clone(),
                password,
                resource,
                server: ServerConfig {
                    host,
                    port,
                    use_tls,
                    accept_invalid_certs,
                },
                auto_connect,
                save_password,
            };

            this.config.accounts.push(account);
            let _ = this.config_manager.save_config(&this.config);

            // Refresh the accounts page
            this.window.close();
            // TODO: Refresh the UI to show the new account
        }));

        dialog.show();
    }

    fn edit_account(&mut self, account: &AccountConfig) {
        // Similar to add_new_account but with pre-filled values
        // Implementation would be similar but loading existing values
    }

    fn remove_account(&mut self, jid: &str) {
        self.config.accounts.retain(|acc| acc.jid != jid);
        let _ = self.config_manager.save_config(&self.config);
        
        // Refresh the UI
        self.window.close();
        // TODO: Refresh the UI to show the updated accounts list
    }

    pub fn show(&self) {
        self.window.show();
    }
}