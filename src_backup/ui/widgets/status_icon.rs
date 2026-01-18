use gtk4::prelude::*;
use gtk4::{Image, Button, MenuButton, PopoverMenu, ListBox, ListBoxRow};

pub struct StatusIcon {
    widget: MenuButton,
    inner_widget: Image,
    status: String,
}

impl StatusIcon {
    pub fn new(status: &str) -> Self {
        let inner_widget = Image::builder()
            .icon_name(Self::get_icon_name(status))
            .icon_size(gtk4::IconSize::Large)
            .css_classes(vec!["status-icon".to_string()])
            .build();

        let widget = MenuButton::builder()
            .icon_name("dialog-information-symbolic")
            .tooltip_text("Change status")
            .build();

        Self {
            widget,
            inner_widget,
            status: status.to_string(),
        }
    }

    pub fn create_status_menu(&self) -> PopoverMenu {
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .build();

        let statuses = vec![
            ("online", "Available", "user-available-symbolic"),
            ("chat", "Free to Chat", "user-available-symbolic"),
            ("away", "Away", "user-away-symbolic"),
            ("xa", "Extended Away", "user-idle-symbolic"),
            ("dnd", "Do Not Disturb", "user-busy-symbolic"),
            ("offline", "Offline", "user-offline-symbolic"),
        ];

        for (status, label, icon) in statuses {
            let row = ListBoxRow::builder()
                .activatable(true)
                .build();

            let row_content = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(12)
                .margin_start(12)
                .margin_end(12)
                .margin_top(6)
                .margin_bottom(6)
                .build();

            let status_icon = Image::builder()
                .icon_name(icon)
                .icon_size(gtk4::IconSize::Small)
                .build();

            let status_label = gtk4::Label::builder()
                .label(label)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();

            row_content.append(&status_icon);
            row_content.append(&status_label);
            row.set_child(Some(&row_content));

            // Connect click handler
            let status_str = status.to_string();
            row.connect_activated(clone!(@strong self as this => move |_| {
                this.set_status(&status_str);
                // TODO: Close popover and send status change
            }));

            list_box.append(&row);
        }

        let popover = PopoverMenu::builder()
            .child(&list_box)
            .build();

        popover
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
        self.inner_widget.set_from_icon_name(Some(Self::get_icon_name(status)));
        
        // Update CSS class
        self.inner_widget.remove_css_class(&[
            "status-online",
            "status-chat",
            "status-away", 
            "status-xa",
            "status-dnd",
            "status-offline",
        ]);
        
        self.inner_widget.add_css_class(&format!("status-{}", status));
    }

    pub fn get_status(&self) -> &str {
        &self.status
    }

    pub fn get_widget(&self) -> &MenuButton {
        &self.widget
    }

    fn get_icon_name(status: &str) -> &str {
        match status {
            "online" | "chat" => "user-available-symbolic",
            "away" => "user-away-symbolic",
            "xa" | "extended_away" => "user-idle-symbolic",
            "dnd" => "user-busy-symbolic",
            "offline" => "user-offline-symbolic",
            _ => "user-offline-symbolic",
        }
    }
}

impl Default for StatusIcon {
    fn default() -> Self {
        Self::new("offline")
    }
}