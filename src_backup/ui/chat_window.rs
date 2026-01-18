use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, TextView, TextBuffer, ScrolledWindow, 
    Entry, Button, Label, Frame, Stack, ListBox, ListBoxRow,
    Adjustment, Image, Badge,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, Bin};
use std::collections::HashMap;
use xmpp_parsers::Jid;

pub struct ChatWindow {
    widget: GtkBox,
    
    // Chat interface
    message_list: ListBox,
    message_text: TextView,
    message_buffer: TextBuffer,
    message_entry: Entry,
    send_button: Button,
    chat_stack: Stack,
    
    // UI elements
    chat_title: Label,
    chat_status: Label,
    typing_indicator: Label,
    
    // State
    current_chat: Option<Jid>,
    chat_widgets: HashMap<String, ChatWidget>,
    
    // Command sender
    command_tx: Option<tokio::sync::mpsc::Sender<crate::xmpp::XmppCommand>>,
}

#[derive(Debug)]
struct ChatWidget {
    widget: GtkBox,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    from: Jid,
    to: Jid,
    body: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    is_sent: bool,
}

impl ChatWindow {
    pub fn new() -> Self {
        // Create main container
        let widget = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // Create header for current chat
        let chat_header = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .margin_bottom(12)
            .build();

        let chat_avatar = Image::builder()
            .icon_name("avatar-default-symbolic")
            .icon_size(gtk4::IconSize::Large)
            .build();

        let chat_info = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .build();

        let chat_title = Label::builder()
            .label("Select a chat")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading".to_string()])
            .build();

        let chat_status = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();

        let typing_indicator = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        chat_info.append(&chat_title);
        chat_info.append(&chat_status);
        chat_info.append(&typing_indicator);

        chat_header.append(&chat_avatar);
        chat_header.append(&chat_info);

        // Create message display area
        let message_list = ListBox::builder()
            .vexpand(true)
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(vec!["message-list".to_string()])
            .build();

        let scrolled_window = ScrolledWindow::builder()
            .child(&message_list)
            .vexpand(true)
            .min_content_height(400)
            .policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic)
            .build();

        // Create message input area
        let input_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_top(12)
            .build();

        let message_text = TextView::builder()
            .wrap_mode(gtk4::WrapMode::WordChar)
            .height_request(80)
            .css_classes(vec!["chat-input".to_string()])
            .build();

        let message_buffer = message_text.buffer();

        let message_entry = Entry::builder()
            .placeholder_text("Type a message...")
            .secondary_icon_name("emoticon-symbolic")
            .build();

        let button_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .build();

        let attach_button = Button::builder()
            .icon_name("paperclip-symbolic")
            .tooltip_text("Attach file")
            .build();

        let send_button = Button::builder()
            .label("Send")
            .icon_name("send-symbolic")
            .sensitive(false)
            .css_classes(vec!["suggested-action".to_string()])
            .build();

        button_box.append(&attach_button);
        button_box.append(&send_button);

        input_box.append(&Box::new(gtk4::Orientation::Horizontal, 0)); // Separator
        input_box.append(&message_text);
        input_box.append(&Box::new(gtk4::Orientation::Horizontal, 0)); // Separator
        input_box.append(&message_entry);
        input_box.append(&Box::new(gtk4::Orientation::Horizontal, 0)); // Separator
        input_box.append(&button_box);

        // Create stack for different views
        let chat_stack = Stack::new();
        
        let welcome_label = Label::builder()
            .label("Select a contact to start chatting")
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .css_classes(vec!["dim-label".to_string(), "heading-2".to_string()])
            .build();

        let welcome_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        
        welcome_box.append(&welcome_label);
        
        let chat_content = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        
        chat_content.append(&chat_header);
        chat_content.append(&scrolled_window);
        chat_content.append(&input_box);

        chat_stack.add_named(&welcome_box, "welcome");
        chat_stack.add_named(&chat_content, "chat");
        chat_stack.set_visible_child_name("welcome");

        // Assemble main widget
        widget.append(&chat_stack);

        let mut chat_window = Self {
            widget,
            message_list,
            message_buffer,
            message_entry,
            send_button,
            chat_stack,
            chat_title,
            chat_status,
            typing_indicator,
            current_chat: None,
            chat_widgets: HashMap::new(),
            command_tx: None,
        };

        // Setup connections
        chat_window.setup_connections();

        chat_window
    }

    fn setup_connections(&self) {
        // Send button
        self.send_button.connect_clicked(clone!(@strong self.message_buffer as buffer, @strong self.message_entry as entry => move |_| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            
            if !text.is_empty() {
                buffer.delete(&mut start.clone(), &end);
                entry.set_text("");
                
                // Send message through command channel
                // This will be set up from the main window
            }
        }));

        // Message text view
        self.message_buffer.connect_changed(clone!(@strong self.send_button as send_btn => move |buffer| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            
            send_btn.set_sensitive(!text.is_empty());
            
            // Send typing indicator
            if !text.is_empty() {
                // Send composing state
            }
        }));

        // Message entry (for quick one-liners)
        self.message_entry.connect_activate(clone!(@strong self.message_entry as entry, @strong self.message_buffer as buffer => move |_| {
            let text = entry.text().to_string();
            if !text.is_empty() {
                buffer.insert_at_cursor(&text);
                entry.set_text("");
                
                // Trigger send
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let full_text = buffer.text(&start, &end, false);
                
                if !full_text.is_empty() {
                    // Send message through command channel
                    buffer.delete(&mut start.clone(), &end);
                }
            }
        }));

        // Attachment button
        // TODO: Implement file attachment
    }

    pub fn get_widget(&self) -> &GtkBox {
        &self.widget
    }

    pub fn set_command_tx(&mut self, tx: tokio::sync::mpsc::Sender<crate::xmpp::XmppCommand>) {
        self.command_tx = Some(tx);
    }

    pub fn open_chat(&mut self, jid: &Jid, display_name: &str) {
        self.current_chat = Some(jid.clone());
        
        // Update UI
        self.chat_title.set_label(display_name);
        self.chat_status.set_label("Online");
        self.typing_indicator.set_label("");
        
        // Show chat view
        self.chat_stack.set_visible_child_name("chat");
        
        // Load or create chat widget
        let jid_str = jid.to_string();
        if !self.chat_widgets.contains_key(&jid_str) {
            self.chat_widgets.insert(jid_str.clone(), ChatWidget {
                widget: GtkBox::new(gtk4::Orientation::Vertical, 6),
                messages: Vec::new(),
            });
        }
        
        // Load chat history
        self.load_chat_history(jid);
    }

    pub fn add_message(&mut self, from: &Jid, to: &Jid, body: &str, is_sent: bool) {
        let chat_jid = if is_sent { to } else { from };
        let jid_str = chat_jid.to_string();
        
        // Create message widget
        let message_row = ActionRow::builder()
            .title(body)
            .css_classes(if is_sent {
                vec!["message-sent".to_string()]
            } else {
                vec!["message-received".to_string()]
            })
            .build();

        // Add timestamp
        let timestamp = chrono::Utc::now();
        message_row.set_subtitle(&timestamp.format("%H:%M").to_string());

        // Add to message list
        self.message_list.append(&message_row);

        // Store message in chat widget
        if let Some(chat_widget) = self.chat_widgets.get_mut(&jid_str) {
            chat_widget.messages.push(ChatMessage {
                from: from.clone(),
                to: to.clone(),
                body: body.to_string(),
                timestamp,
                is_sent,
            });
        }

        // Scroll to bottom
        self.message_list.emit_row_activated(&message_row.index());
    }

    pub fn add_groupchat_message(&mut self, room_jid: &Jid, nickname: &str, body: &str) {
        let message_row = ActionRow::builder()
            .title(format!("{}: {}", nickname, body))
            .css_classes(vec!["message-groupchat".to_string()])
            .build();

        let timestamp = chrono::Utc::now();
        message_row.set_subtitle(&timestamp.format("%H:%M").to_string());

        self.message_list.append(&message_row);
    }

    pub fn update_chat_state(&mut self, from: &Jid, state: &str) {
        if let Some(current_chat) = &self.current_chat {
            if from == current_chat {
                match state {
                    "Composing" => {
                        self.typing_indicator.set_label("typing...");
                    }
                    "Paused" => {
                        self.typing_indicator.set_label("paused typing");
                    }
                    "Active" => {
                        self.typing_indicator.set_label("");
                    }
                    "Inactive" | "Gone" => {
                        self.typing_indicator.set_label("");
                    }
                    _ => {}
                }
            }
        }
    }

    fn load_chat_history(&self, jid: &Jid) {
        // Clear current messages
        while let Some(row) = self.message_list.first_child() {
            self.message_list.remove(&row);
        }

        // TODO: Load chat history from database
        // For now, just show empty chat
    }

    pub fn clear_chat(&mut self) {
        while let Some(row) = self.message_list.first_child() {
            self.message_list.remove(&row);
        }
        
        self.current_chat = None;
        self.chat_title.set_label("Select a chat");
        self.chat_status.set_label("");
        self.typing_indicator.set_label("");
        self.chat_stack.set_visible_child_name("welcome");
    }
}