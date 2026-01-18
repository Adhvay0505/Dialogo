use gtk4::prelude::*;
use gtk4::{Window, Box as GtkBox, Label, Button, ProgressBar, Image, Scale};
use libadwaita::prelude::*;
use libadwaita::{PreferencesGroup, ActionRow};
use std::path::PathBuf;

pub struct FileTransferDialog {
    window: gtk4::Window,
    progress_bar: ProgressBar,
    status_label: Label,
    file_name_label: Label,
    size_label: Label,
    speed_label: Label,
    from_jid_label: Label,
    callback: Option<Box<dyn FnOnce(PathBuf, bool)>>,
}

impl FileTransferDialog {
    pub fn new(
        parent: &impl IsA<Window>,
        file_name: String,
        file_size: u64,
        from_jid: xmpp_parsers::Jid,
        is_incoming: bool,
    ) -> Self {
        let title = if is_incoming {
            "Incoming File Transfer"
        } else {
            "Outgoing File Transfer"
        };

        let window = gtk4::Window::builder()
            .title(title)
            .modal(true)
            .default_width(500)
            .default_height(400)
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

        // File information group
        let file_group = PreferencesGroup::builder()
            .title("File Information")
            .build();

        let file_name_label = Label::builder()
            .label(&file_name)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading".to_string()])
            .build();

        let size_label = Label::builder()
            .label(&format_size(file_size))
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();

        let from_jid_label = Label::builder()
            .label(&from_jid.to_string())
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();

        file_group.add(&file_name_label);
        file_group.add(&size_label);
        file_group.add(&from_jid_label);

        // Progress information
        let progress_group = PreferencesGroup::builder()
            .title("Transfer Progress")
            .build();

        let progress_bar = ProgressBar::builder()
            .hexpand(true)
            .text("Waiting...")
            .show_text(true)
            .build();

        let status_label = Label::builder()
            .label("Waiting for response...")
            .halign(gtk4::Align::Start)
            .build();

        let speed_label = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();

        progress_group.add(&progress_bar);
        progress_group.add(&status_label);
        progress_group.add(&speed_label);

        // Buttons
        let button_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .margin_top(12)
            .build();

        let accept_button = Button::builder()
            .label("Accept")
            .css_classes(vec!["suggested-action".to_string()])
            .visible(is_incoming)
            .build();

        let reject_button = Button::builder()
            .label("Reject")
            .css_classes(vec!["destructive-action".to_string()])
            .visible(is_incoming)
            .build();

        let cancel_button = Button::builder()
            .label("Cancel")
            .visible(!is_incoming)
            .build();

        let close_button = Button::builder()
            .label("Close")
            .visible(false)
            .build();

        button_box.append(&accept_button);
        button_box.append(&reject_button);
        button_box.append(&cancel_button);
        button_box.append(&close_button);

        // Assemble dialog
        content.append(&file_group);
        content.append(&progress_group);
        content.append(&button_box);

        window.set_content(Some(&content));

        let mut dialog = Self {
            window,
            progress_bar,
            status_label,
            file_name_label,
            size_label,
            speed_label,
            from_jid_label,
            callback: None,
        };

        // Connect button handlers
        accept_button.connect_clicked(clone!(@strong dialog.window as window,
                                                 @strong mut dialog => move |_| {
            dialog.set_status("Transfer starting...");
            dialog.set_progress(0.0);
            // TODO: Start file transfer
        }));

        reject_button.connect_clicked(clone!(@strong dialog.window as window,
                                                 @strong dialog.from_jid_label as from_jid_label => move |_| {
            window.close();
            // TODO: Reject file transfer
        }));

        cancel_button.connect_clicked(clone!(@strong dialog.window as window,
                                                 @strong mut dialog => move |_| {
            dialog.set_status("Transfer cancelled");
            dialog.set_progress(0.0);
            // TODO: Cancel file transfer
        }));

        close_button.connect_clicked(clone!(@strong dialog.window as window => move |_| {
            window.close();
        }));

        dialog
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnOnce(PathBuf, bool) + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    pub fn update_progress(&mut self, progress: f64, bytes_transferred: u64) {
        self.progress_bar.set_fraction(progress);
        self.progress_bar.set_text(&format!("{}% / {}", 
            (progress * 100.0) as i32, 
            format_size(bytes_transferred)
        ));
    }

    pub fn update_speed(&mut self, bytes_per_second: u64) {
        self.speed_label.set_label(&format!("{} /s", format_size(bytes_per_second)));
    }

    pub fn set_status(&mut self, status: &str) {
        self.status_label.set_label(status);
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress_bar.set_fraction(progress);
        self.progress_bar.set_text(&format!("{}%", (progress * 100.0) as i32));
    }

    pub fn set_completed(&mut self, file_path: PathBuf) {
        self.progress_bar.set_fraction(1.0);
        self.progress_bar.set_text("100% - Complete");
        self.status_label.set_label("Transfer completed successfully");
        self.speed_label.set_label("");
        
        // Show close button, hide others
        // TODO: Update button visibility
    }

    pub fn set_error(&mut self, error: &str) {
        self.status_label.set_label(&format!("Error: {}", error));
        self.progress_bar.set_text("Failed");
    }

    pub fn show(&self) {
        self.window.show();
    }

    fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}