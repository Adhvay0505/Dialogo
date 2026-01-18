use gtk4::prelude::*;
use gtk4::{
    Window, Box as GtkBox, Label, Entry, Button,
    Switch, SpinButton, Grid, Separator,
};
use libadwaita::prelude::*;
use libadwaita::{EntryRow, PasswordEntryRow, SwitchRow, SpinRow, PreferencesGroup};
use crate::config::{AccountConfig, ServerConfig};
use crate::xmpp::XmppClientConfig;

pub struct ConnectionDialog {
    window: gtk4::Window,
    config: XmppClientConfig,
    callback: Option<Box<dyn FnOnce(XmppClientConfig)>>,
}

impl ConnectionDialog {
    pub fn new(parent: &impl IsA<Window>, config: Option<XmppClientConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        let window = gtk4::Window::builder()
            .title("Connect to XMPP Server")
            .modal(true)
            .default_width(500)
            .default_height(700)
            .transient_for(parent)
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
            .text(&config.jid)
            .build();

        let password_row = PasswordEntryRow::builder()
            .title("Password")
            .text(&config.password)
            .build();

        let resource_row = EntryRow::builder()
            .title("Resource")
            .text(&config.resource)
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
            .text(&config.server_host)
            .build();

        let port_row = SpinRow::builder()
            .title("Port")
            .range(1.0, 65535.0)
            .value(config.server_port as f64)
            .build();

        let tls_row = SwitchRow::builder()
            .title("Use TLS")
            .active(config.use_tls)
            .build();

        let invalid_certs_row = SwitchRow::builder()
            .title("Accept Invalid Certificates")
            .subtitle("Only for testing purposes")
            .active(config.accept_invalid_certs)
            .build();

        server_group.add(&host_row);
        server_group.add(&port_row);
        server_group.add(&tls_row);
        server_group.add(&invalid_certs_row);

        // Connection options group
        let options_group = PreferencesGroup::builder()
            .title("Connection Options")
            .build();

        let auto_reconnect_row = SwitchRow::builder()
            .title("Auto Reconnect")
            .active(config.auto_reconnect)
            .build();

        let max_attempts_row = SpinRow::builder()
            .title("Max Reconnect Attempts")
            .range(1.0, 20.0)
            .value(config.max_reconnect_attempts as f64)
            .build();

        let reconnect_delay_row = SpinRow::builder()
            .title("Reconnect Delay (seconds)")
            .range(5.0, 300.0)
            .value(config.reconnect_delay.as_secs() as f64)
            .build();

        options_group.add(&auto_reconnect_row);
        options_group.add(&max_attempts_row);
        options_group.add(&reconnect_delay_row);

        // Buttons
        let button_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .margin_top(12)
            .build();

        let cancel_button = Button::builder()
            .label("Cancel")
            .build();

        let connect_button = Button::builder()
            .label("Connect")
            .css_classes(vec!["suggested-action".to_string()])
            .build();

        button_box.append(&cancel_button);
        button_box.append(&connect_button);

        // Assemble dialog
        content.append(&account_group);
        content.append(&server_group);
        content.append(&options_group);
        content.append(&button_box);

        window.set_content(Some(&content));

        let mut dialog = Self {
            window,
            config,
            callback: None,
        };

        // Connect button handlers
        cancel_button.connect_clicked(clone!(@strong dialog.window as window => move |_| {
            window.close();
        }));

        connect_button.connect_clicked(clone!(@strong dialog.window as window,
                                               @strong jid_row, @strong password_row,
                                               @strong resource_row, @strong host_row,
                                               @strong port_row, @strong tls_row,
                                               @strong invalid_certs_row,
                                               @strong auto_reconnect_row,
                                               @strong max_attempts_row,
                                               @strong reconnect_delay_row => move |_| {
            let new_config = XmppClientConfig {
                jid: jid_row.text().to_string(),
                password: password_row.text().to_string(),
                resource: resource_row.text().to_string(),
                server_host: host_row.text().to_string(),
                server_port: port_row.value() as u16,
                use_tls: tls_row.is_active(),
                accept_invalid_certs: invalid_certs_row.is_active(),
                auto_reconnect: auto_reconnect_row.is_active(),
                max_reconnect_attempts: max_attempts_row.value() as u32,
                reconnect_delay: std::time::Duration::from_secs(reconnect_delay_row.value() as u64),
            };

            window.close();
            
            // TODO: Call callback or emit signal with the new config
        }));

        dialog
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnOnce(XmppClientConfig) + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    pub fn show(&self) {
        self.window.show();
    }
}

impl From<AccountConfig> for XmppClientConfig {
    fn from(account: AccountConfig) -> Self {
        Self {
            jid: account.jid,
            password: account.password,
            resource: account.resource,
            server_host: account.server.host,
            server_port: account.server.port,
            use_tls: account.server.use_tls,
            accept_invalid_certs: account.server.accept_invalid_certs,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay: std::time::Duration::from_secs(10),
        }
    }
}