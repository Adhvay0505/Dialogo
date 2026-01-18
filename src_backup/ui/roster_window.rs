use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, ListBox, ListBoxRow, ScrolledWindow, Entry,
    Button, Label, SearchEntry, MenuButton, PopoverMenu,
    Image, Separator, Frame, FlowBox,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, Bin, PreferencesGroup, PreferencesRow};
use std::collections::HashMap;
use xmpp_parsers::Jid;

pub struct RosterWindow {
    widget: GtkBox,
    
    // Roster list
    roster_list: ListBox,
    search_entry: SearchEntry,
    add_contact_button: Button,
    menu_button: MenuButton,
    
    // Grouping
    flow_box: FlowBox,
    online_group: PreferencesGroup,
    offline_group: PreferencesGroup,
    
    // State
    roster_items: HashMap<String, RosterItem>,
    current_filter: String,
    
    // Command sender
    command_tx: Option<tokio::sync::mpsc::Sender<crate::xmpp::XmppCommand>>,
}

#[derive(Debug, Clone)]
pub struct RosterItem {
    pub jid: Jid,
    pub name: Option<String>,
    pub show: String,
    pub status: Option<String>,
    pub subscription: String,
    pub groups: Vec<String>,
}

impl RosterWindow {
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

        // Create header with search
        let header_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_bottom(12)
            .build();

        let title_label = Label::builder()
            .label("Contacts")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading".to_string()])
            .build();

        let search_controls = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();

        let search_entry = SearchEntry::builder()
            .placeholder_text("Search contacts...")
            .hexpand(true)
            .build();

        let add_contact_button = Button::builder()
            .icon_name("contact-new-symbolic")
            .tooltip_text("Add contact")
            .build();

        let menu_button = MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("More options")
            .build();

        search_controls.append(&search_entry);
        search_controls.append(&add_contact_button);
        search_controls.append(&menu_button);

        header_box.append(&title_label);
        header_box.append(&search_controls);

        // Create scrollable area
        let scrolled_window = ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(600)
            .policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic)
            .build();

        // Create main content area
        let content_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .build();

        // Create groups
        let online_group = PreferencesGroup::builder()
            .title("Online")
            .build();

        let offline_group = PreferencesGroup::builder()
            .title("Offline")
            .build();

        content_box.append(&online_group);
        content_box.append(&offline_group);

        scrolled_window.set_child(Some(&content_box));

        // Assemble main widget
        widget.append(&header_box);
        widget.append(&scrolled_window);

        let mut roster_window = Self {
            widget,
            roster_list: ListBox::new(), // Not used in favor of preference groups
            search_entry,
            add_contact_button,
            menu_button,
            flow_box: FlowBox::new(),
            online_group,
            offline_group,
            roster_items: HashMap::new(),
            current_filter: String::new(),
            command_tx: None,
        };

        // Setup connections
        roster_window.setup_connections();

        roster_window
    }

    fn setup_connections(&self) {
        // Search entry
        self.search_entry.connect_search_changed(clone!(@strong self as this => move |entry| {
            let filter_text = entry.text().to_string().to_lowercase();
            this.filter_contacts(&filter_text);
        }));

        // Add contact button
        self.add_contact_button.connect_clicked(clone!(@strong self as this => move |_| {
            this.show_add_contact_dialog();
        }));

        // Double-click on roster item to start chat
        self.online_group.connect_row_activated(clone!(@strong self as this => move |_, row| {
            if let Some(action_row) = row.downcast_ref::<ActionRow>() {
                let jid_str = action_row.title();
                if let Some(jid) = jid_str.parse().ok() {
                    this.open_chat_with_contact(jid);
                }
            }
        }));

        self.offline_group.connect_row_activated(clone!(@strong self as this => move |_, row| {
            if let Some(action_row) = row.downcast_ref::<ActionRow>() {
                let jid_str = action_row.title();
                if let Some(jid) = jid_str.parse().ok() {
                    this.open_chat_with_contact(jid);
                }
            }
        }));
    }

    pub fn get_widget(&self) -> &GtkBox {
        &self.widget
    }

    pub fn get_command_tx(&self) -> &Option<tokio::sync::mpsc::Sender<crate::xmpp::XmppCommand>> {
        &self.command_tx
    }

    pub fn set_command_tx(&mut self, tx: tokio::sync::mpsc::Sender<crate::xmpp::XmppCommand>) {
        self.command_tx = Some(tx);
    }

    pub fn set_roster(&mut self, items: Vec<crate::xmpp::events::RosterItem>) {
        // Clear existing roster
        self.clear_roster();

        // Add new items
        for item in items {
            let roster_item = RosterItem {
                jid: item.jid.clone(),
                name: item.name.clone(),
                show: "online".to_string(), // Default to online
                status: None,
                subscription: item.subscription.clone(),
                groups: item.groups.clone(),
            };

            let jid_str = item.jid.to_string();
            self.roster_items.insert(jid_str.clone(), roster_item.clone());
            
            self.add_roster_item_widget(roster_item);
        }
    }

    pub fn update_presence(&mut self, jid: &Jid, show: &str, status: Option<&str>) {
        let jid_str = jid.to_string();
        
        if let Some(roster_item) = self.roster_items.get_mut(&jid_str) {
            roster_item.show = show.to_string();
            roster_item.status = status.map(|s| s.to_string());
            
            // Update the UI
            self.update_roster_item_widget(jid, show, status);
        }
    }

    pub fn add_roster_item(&mut self, jid: Jid, name: Option<String>, groups: Vec<String>) {
        let roster_item = RosterItem {
            jid: jid.clone(),
            name: name.clone(),
            show: "offline".to_string(),
            status: None,
            subscription: "none".to_string(),
            groups: groups.clone(),
        };

        let jid_str = jid.to_string();
        self.roster_items.insert(jid_str.clone(), roster_item.clone());
        
        self.add_roster_item_widget(roster_item);
    }

    pub fn remove_roster_item(&mut self, jid: &Jid) {
        let jid_str = jid.to_string();
        
        // Remove from state
        self.roster_items.remove(&jid_str);
        
        // Remove from UI
        self.remove_roster_item_widget(jid);
    }

    fn clear_roster(&mut self) {
        // Clear online group
        while let Some(row) = self.online_group.first_child() {
            self.online_group.remove(&row);
        }

        // Clear offline group
        while let Some(row) = self.offline_group.first_child() {
            self.offline_group.remove(&row);
        }

        self.roster_items.clear();
    }

    fn add_roster_item_widget(&self, item: RosterItem) {
        let display_name = item.name.as_deref().unwrap_or_else(|| item.jid.node().unwrap_or("Unknown"));
        
        let row = ActionRow::builder()
            .title(display_name)
            .subtitle(item.jid.to_string())
            .activatable(true)
            .build();

        // Add presence indicator
        let presence_icon = Image::builder()
            .icon_name(self.get_presence_icon(&item.show))
            .icon_size(gtk4::IconSize::Small)
            .build();

        row.add_prefix(&presence_icon);

        // Add status text if available
        if let Some(status) = &item.status {
            row.set_subtitle(&format!("{} - {}", item.jid.to_string(), status));
        }

        // Add to appropriate group based on presence
        if item.show == "online" || item.show == "chat" || item.show == "away" || item.show == "dnd" {
            self.online_group.add(&row);
        } else {
            self.offline_group.add(&row);
        }
    }

    fn update_roster_item_widget(&self, jid: &Jid, show: &str, status: Option<&str>) {
        // Find and update the existing row
        // This is a simplified approach - in a real implementation,
        // you would need to track row references
        self.set_roster(&self.roster_items.values().map(|item| crate::xmpp::events::RosterItem {
            jid: item.jid.clone(),
            name: item.name.clone(),
            subscription: item.subscription.clone(),
            groups: item.groups.clone(),
            approved: false,
            ask: None,
        }).collect());
    }

    fn remove_roster_item_widget(&self, jid: &Jid) {
        // Remove from UI - simplified approach
        self.set_roster(self.roster_items.values().map(|item| crate::xmpp::events::RosterItem {
            jid: item.jid.clone(),
            name: item.name.clone(),
            subscription: item.subscription.clone(),
            groups: item.groups.clone(),
            approved: false,
            ask: None,
        }).collect());
    }

    fn filter_contacts(&self, filter: &str) {
        self.current_filter = filter.to_string();
        
        // Show/hide items based on filter
        for (_, item) in &self.roster_items {
            let display_name = item.name.as_deref().unwrap_or_else(|| item.jid.node().unwrap_or("Unknown"));
            let jid_str = item.jid.to_string();
            
            let matches_filter = filter.is_empty() || 
                display_name.to_lowercase().contains(filter) ||
                jid_str.to_lowercase().contains(filter);
            
            // Update visibility in the appropriate group
            // This would require tracking row references in a real implementation
        }
    }

    fn get_presence_icon(&self, show: &str) -> &str {
        match show {
            "online" | "chat" => "user-available-symbolic",
            "away" => "user-away-symbolic",
            "xa" | "extended_away" => "user-idle-symbolic",
            "dnd" => "user-busy-symbolic",
            "offline" => "user-offline-symbolic",
            _ => "user-offline-symbolic",
        }
    }

    fn show_add_contact_dialog(&self) {
        let dialog = gtk4::MessageDialog::builder()
            .title("Add Contact")
            .message_type(gtk4::MessageType::Question)
            .buttons(gtk4::ButtonsType::OkCancel)
            .text("Enter JID of the contact to add")
            .modal(true)
            .build();

        // Create entry for JID input
        let entry = Entry::builder()
            .placeholder_text("user@domain.com")
            .build();

        // Add entry to dialog
        dialog.content_area().append(&entry);

        dialog.connect_response(None, clone!(@strong self as this, @strong entry => move |dialog, response| {
            if response == gtk4::ResponseType::Ok {
                let jid_text = entry.text().to_string();
                if let Ok(jid) = jid_text.parse() {
                    if let Some(tx) = &this.command_tx {
                        let _ = tx.try_send(crate::xmpp::XmppCommand::AddRosterItem {
                            jid,
                            name: None,
                            groups: vec!["General".to_string()],
                        });
                    }
                }
            }
            dialog.close();
        }));

        dialog.show();
    }

    fn open_chat_with_contact(&self, jid: Jid) {
        // This would communicate with the main window to open a chat
        // In a real implementation, you would emit a signal or use a channel
        tracing::info!("Opening chat with: {}", jid);
    }
}