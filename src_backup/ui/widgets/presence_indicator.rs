use gtk4::prelude::*;
use gtk4::{Image, DrawingArea, Frame};

pub struct PresenceIndicator {
    widget: Frame,
    inner_widget: Image,
    show: String,
}

impl PresenceIndicator {
    pub fn new(show: &str) -> Self {
        let inner_widget = Image::builder()
            .icon_name(Self::get_icon_name(show))
            .icon_size(gtk4::IconSize::Small)
            .css_classes(vec!["presence-indicator".to_string()])
            .build();

        let widget = Frame::builder()
            .css_classes(vec!["presence-indicator-frame".to_string()])
            .child(&inner_widget)
            .build();

        Self {
            widget,
            inner_widget,
            show: show.to_string(),
        }
    }

    pub fn set_show(&mut self, show: &str) {
        self.show = show.to_string();
        self.inner_widget.set_from_icon_name(Some(Self::get_icon_name(show)));
        
        // Update CSS class based on presence
        self.widget.remove_css_class(&[
            "presence-online",
            "presence-away", 
            "presence-xa",
            "presence-dnd",
            "presence-offline",
        ]);
        
        self.widget.add_css_class(&format!("presence-{}", show));
    }

    pub fn get_show(&self) -> &str {
        &self.show
    }

    pub fn get_widget(&self) -> &Frame {
        &self.widget
    }

    fn get_icon_name(show: &str) -> &str {
        match show {
            "online" | "chat" => "user-available-symbolic",
            "away" => "user-away-symbolic",
            "xa" | "extended_away" => "user-idle-symbolic",
            "dnd" => "user-busy-symbolic",
            "offline" => "user-offline-symbolic",
            _ => "user-offline-symbolic",
        }
    }
}

impl Default for PresenceIndicator {
    fn default() -> Self {
        Self::new("offline")
    }
}