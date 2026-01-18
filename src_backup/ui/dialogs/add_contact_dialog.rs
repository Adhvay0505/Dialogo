use gtk4::prelude::*;
use gtk4::{Window, Box as GtkBox, Label, Entry, Button, ListBox};
use libadwaita::prelude::*;
use libadwaita::{PreferencesGroup, EntryRow};
use xmpp_parsers::Jid;

pub struct AddContactDialog {
    window: gtk4::Window,
    jid_entry: EntryRow,
    name_entry: EntryRow,
    groups_entry: EntryRow,
    callback: Option<Box<dyn FnOnce(Jid, Option<String>, Vec<String>)>>,
}

impl AddContactDialog {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = gtk4::Window::builder()
            .title("Add Contact")
            .modal(true)
            .default_width(400)
            .default_height(300)
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

        let group = PreferencesGroup::builder()
            .title("Contact Information")
            .build();

        let jid_entry = EntryRow::builder()
            .title("JID")
            .subtitle("user@domain.com")
            .build();

        let name_entry = EntryRow::builder()
            .title("Display Name")
            .subtitle("Optional")
            .build();

        let groups_entry = EntryRow::builder()
            .title("Groups")
            .subtitle("Comma-separated group names")
            .text("General")
            .build();

        group.add(&jid_entry);
        group.add(&name_entry);
        group.add(&groups_entry);

        let button_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .margin_top(12)
            .build();

        let cancel_button = Button::builder()
            .label("Cancel")
            .build();

        let add_button = Button::builder()
            .label("Add Contact")
            .css_classes(vec!["suggested-action".to_string()])
            .sensitive(false)
            .build();

        button_box.append(&cancel_button);
        button_box.append(&add_button);

        content.append(&group);
        content.append(&button_box);

        window.set_content(Some(&content));

        // Enable/disable add button based on JID validity
        jid_entry.connect_changed(clone!(@strong add_button => move |entry| {
            let text = entry.text().to_string();
            let valid = !text.is_empty() && text.contains('@');
            add_button.set_sensitive(valid);
        }));

        let mut dialog = Self {
            window,
            jid_entry,
            name_entry,
            groups_entry,
            callback: None,
        };

        // Connect button handlers
        cancel_button.connect_clicked(clone!(@strong dialog.window as window => move |_| {
            window.close();
        }));

        add_button.connect_clicked(clone!(@strong dialog.window as window,
                                             @strong dialog.jid_entry as jid_entry,
                                             @strong dialog.name_entry as name_entry,
                                             @strong dialog.groups_entry as groups_entry => move |_| {
            let jid_text = jid_entry.text().to_string();
            let name_text = name_entry.text().to_string();
            let groups_text = groups_entry.text().to_string();

            if let Ok(jid) = jid_text.parse() {
                let name = if name_text.is_empty() { None } else { Some(name_text) };
                let groups = groups_text.split(',')
                    .map(|g| g.trim().to_string())
                    .filter(|g| !g.is_empty())
                    .collect();

                window.close();
                
                // TODO: Call callback with the contact information
            }
        }));

        dialog
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnOnce(Jid, Option<String>, Vec<String>) + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    pub fn show(&self) {
        self.window.show();
    }
}