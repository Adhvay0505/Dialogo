use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Image, Frame};
use libadwaita::prelude::*;
use chrono::{DateTime, Utc};
use xmpp_parsers::Jid;

pub struct MessageBubble {
    widget: libadwaita::ActionRow,
    timestamp: DateTime<Utc>,
    is_sent: bool,
}

impl MessageBubble {
    pub fn new(
        from: &Jid,
        body: &str,
        timestamp: DateTime<Utc>,
        is_sent: bool,
    ) -> Self {
        let widget = libadwaita::ActionRow::builder()
            .title(body)
            .css_classes(if is_sent {
                vec!["message-bubble".to_string(), "message-sent".to_string()]
            } else {
                vec!["message-bubble".to_string(), "message-received".to_string()]
            })
            .build();

        // Add timestamp as subtitle
        widget.set_subtitle(&timestamp.format("%H:%M").to_string());

        // Add sender info for received messages
        if !is_sent {
            let display_name = from.node().unwrap_or("Unknown");
            widget.set_subtitle(&format!("{} - {}", display_name, timestamp.format("%H:%M")));
        }

        Self {
            widget,
            timestamp,
            is_sent,
        }
    }

    pub fn new_system_message(body: &str) -> Self {
        let widget = libadwaita::ActionRow::builder()
            .title(body)
            .css_classes(vec!["message-bubble".to_string(), "message-system".to_string()])
            .halign(gtk4::Align::Center)
            .sensitive(false)
            .build();

        Self {
            widget,
            timestamp: Utc::now(),
            is_sent: false,
        }
    }

    pub fn get_widget(&self) -> &libadwaita::ActionRow {
        &self.widget
    }

    pub fn get_timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn is_sent(&self) -> bool {
        self.is_sent
    }
}