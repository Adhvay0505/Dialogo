use gtk4::prelude::*;
use gtk4::{AboutDialog, Window};
use crate::ui::APPLICATION_NAME;
use crate::ui::APPLICATION_VERSION;

pub struct AboutDialog {
    dialog: AboutDialog,
}

impl AboutDialog {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let dialog = AboutDialog::builder()
            .program_name(APPLICATION_NAME)
            .version(APPLICATION_VERSION)
            .copyright("© 2024 XMPP Client Developers")
            .license_type(gtk4::License::Gpl30)
            .website("https://github.com/example/xmpp-client")
            .website_label("Project Website")
            .authors(vec![
                "Developer Name <developer@example.com>".to_string(),
            ])
            .artists(vec![
                "Artist Name <artist@example.com>".to_string(),
            ])
            .logo_icon_name(APPLICATION_NAME)
            .comments("A modern XMPP client built with GTK4 and Rust")
            .transient_for(parent)
            .modal(true)
            .build();

        Self { dialog }
    }

    pub fn show(&self) {
        self.dialog.show();
    }
}