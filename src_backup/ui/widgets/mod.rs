use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Label, Button, Image, Entry,
    Frame, Scale, SpinButton, Switch, DrawingArea,
};
use libadwaita::prelude::*;
use std::collections::HashMap;

// Custom widgets for the XMPP client

pub mod message_bubble;
pub mod presence_indicator;
pub mod status_icon;
pub mod file_upload_widget;
pub mod emoji_picker;

pub use message_bubble::MessageBubble;
pub use presence_indicator::PresenceIndicator;
pub use status_icon::StatusIcon;
pub use file_upload_widget::FileUploadWidget;
pub use emoji_picker::EmojiPicker;

pub struct RosterItemWidget {
    widget: libadwaita::ActionRow,
    avatar: Image,
    presence_indicator: PresenceIndicator,
    status_text: Label,
}

impl RosterItemWidget {
    pub fn new(jid: &str, display_name: &str, show: &str, status: Option<&str>) -> Self {
        let widget = libadwaita::ActionRow::builder()
            .title(display_name)
            .subtitle(jid)
            .activatable(true)
            .build();

        let avatar = Image::builder()
            .icon_name("avatar-default-symbolic")
            .icon_size(gtk4::IconSize::Large)
            .build();

        let presence_indicator = PresenceIndicator::new(show);
        let status_text = Label::builder()
            .label(status.unwrap_or(""))
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        widget.add_prefix(&avatar);
        widget.add_suffix(&presence_indicator.get_widget());
        
        if let Some(status_msg) = status {
            widget.set_subtitle(&format!("{} - {}", jid, status_msg));
        }

        Self {
            widget,
            avatar,
            presence_indicator,
            status_text,
        }
    }

    pub fn update_presence(&mut self, show: &str, status: Option<&str>) {
        self.presence_indicator.set_show(show);
        
        if let Some(status_msg) = status {
            self.status_text.set_label(status_msg);
            let jid = self.widget.subtitle().unwrap_or_default();
            self.widget.set_subtitle(&format!("{} - {}", jid, status_msg));
        } else {
            self.status_text.set_label("");
            let jid = self.widget.subtitle().unwrap_or_default();
            self.widget.set_subtitle(jid);
        }
    }

    pub fn get_widget(&self) -> &libadwaita::ActionRow {
        &self.widget
    }
}

pub struct ChatInputWidget {
    widget: GtkBox,
    text_view: gtk4::TextView,
    text_buffer: gtk4::TextBuffer,
    emoji_button: Button,
    attach_button: Button,
    send_button: Button,
}

impl ChatInputWidget {
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .css_classes(vec!["chat-input-container".to_string()])
            .build();

        // Text input area
        let scrolled_window = gtk4::ScrolledWindow::builder()
            .height_request(80)
            .min_content_height(60)
            .max_content_height(200)
            .policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic)
            .build();

        let text_view = gtk4::TextView::builder()
            .wrap_mode(gtk4::WrapMode::WordChar)
            .css_classes(vec!["chat-input".to_string()])
            .build();

        let text_buffer = text_view.buffer();
        scrolled_window.set_child(Some(&text_view));

        // Button bar
        let button_bar = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_top(6)
            .build();

        let emoji_button = Button::builder()
            .icon_name("emoticon-symbolic")
            .tooltip_text("Insert emoji")
            .css_classes(vec!["flat".to_string()])
            .build();

        let attach_button = Button::builder()
            .icon_name("paperclip-symbolic")
            .tooltip_text("Attach file")
            .css_classes(vec!["flat".to_string()])
            .build();

        let typing_label = Label::builder()
            .label("")
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .hexpand(true)
            .build();

        let send_button = Button::builder()
            .label("Send")
            .icon_name("send-symbolic")
            .sensitive(false)
            .css_classes(vec!["suggested-action".to_string()])
            .build();

        button_bar.append(&emoji_button);
        button_bar.append(&attach_button);
        button_bar.append(&typing_label);
        button_bar.append(&send_button);

        // Assemble widget
        widget.append(&scrolled_window);
        widget.append(&button_bar);

        Self {
            widget,
            text_view,
            text_buffer,
            emoji_button,
            attach_button,
            send_button,
        }
    }

    pub fn get_text(&self) -> String {
        let start = self.text_buffer.start_iter();
        let end = self.text_buffer.end_iter();
        self.text_buffer.text(&start, &end, false)
    }

    pub fn clear(&self) {
        let start = self.text_buffer.start_iter();
        let end = self.text_buffer.end_iter();
        self.text_buffer.delete(&mut start.clone(), &end);
    }

    pub fn set_send_sensitive(&self, sensitive: bool) {
        self.send_button.set_sensitive(sensitive);
    }

    pub fn get_widget(&self) -> &GtkBox {
        &self.widget
    }

    pub fn connect_send<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.send_button.connect_clicked(move |_| callback());
        
        self.text_buffer.connect_changed(clone!(@strong self.send_button => move |buffer| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            send_button.set_sensitive(!text.is_empty());
        }));
    }

    pub fn connect_emoji<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.emoji_button.connect_clicked(move |_| callback());
    }

    pub fn connect_attach<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.attach_button.connect_clicked(move |_| callback());
    }
}

pub struct StatusIndicator {
    widget: GtkBox,
    status_label: Label,
    status_icon: StatusIcon,
}

impl StatusIndicator {
    pub fn new(status: &str) -> Self {
        let widget = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .css_classes(vec!["status-indicator".to_string()])
            .build();

        let status_icon = StatusIcon::new(status);
        let status_label = Label::builder()
            .label(status)
            .css_classes(vec!["caption".to_string()])
            .build();

        widget.append(&status_icon.get_widget());
        widget.append(&status_label);

        Self {
            widget,
            status_label,
            status_icon,
        }
    }

    pub fn set_status(&mut self, status: &str) {
        self.status_label.set_label(status);
        self.status_icon.set_status(status);
    }

    pub fn get_widget(&self) -> &GtkBox {
        &self.widget
    }
}

pub struct TypingIndicator {
    widget: Label,
    dots: Vec<Label>,
    animation_timer: Option<glib::SourceId>,
}

impl TypingIndicator {
    pub fn new() -> Self {
        let widget = Label::builder()
            .label("")
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        Self {
            widget,
            dots: Vec::new(),
            animation_timer: None,
        }
    }

    pub fn start_typing(&mut self, user_name: &str) {
        self.widget.set_label(&format!("{} is typing", user_name));
        // TODO: Start animation
    }

    pub fn stop_typing(&mut self) {
        self.widget.set_label("");
        // TODO: Stop animation
    }

    pub fn get_widget(&self) -> &Label {
        &self.widget
    }
}

pub struct ConnectionStatusWidget {
    widget: GtkBox,
    status_icon: Image,
    status_label: Label,
    details_label: Label,
}

impl ConnectionStatusWidget {
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .css_classes(vec!["connection-status".to_string()])
            .build();

        let status_icon = Image::builder()
            .icon_name("network-offline-symbolic")
            .icon_size(gtk4::IconSize::Small)
            .build();

        let status_label = Label::builder()
            .label("Disconnected")
            .css_classes(vec!["caption".to_string()])
            .build();

        let details_label = Label::builder()
            .label("")
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        widget.append(&status_icon);
        widget.append(&status_label);
        widget.append(&details_label);

        Self {
            widget,
            status_icon,
            status_label,
            details_label,
        }
    }

    pub fn set_connected(&mut self, jid: &str) {
        self.status_icon.set_from_icon_name(Some("network-wired-symbolic"));
        self.status_label.set_label("Connected");
        self.details_label.set_label(jid);
    }

    pub fn set_connecting(&mut self) {
        self.status_icon.set_from_icon_name(Some("network-idle-symbolic"));
        self.status_label.set_label("Connecting...");
        self.details_label.set_label("");
    }

    pub fn set_disconnected(&mut self, reason: &str) {
        self.status_icon.set_from_icon_name(Some("network-offline-symbolic"));
        self.status_label.set_label("Disconnected");
        self.details_label.set_label(reason);
    }

    pub fn set_error(&mut self, error: &str) {
        self.status_icon.set_from_icon_name(Some("network-error-symbolic"));
        self.status_label.set_label("Error");
        self.details_label.set_label(error);
    }

    pub fn get_widget(&self) -> &GtkBox {
        &self.widget
    }
}