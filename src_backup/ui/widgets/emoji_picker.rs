use gtk4::prelude::*;
use gtk4::{
    Button, Grid, ScrolledWindow, SearchEntry, FlowBox,
    FlowBoxChild, Image, Label, Popover, PopoverMenu,
};
use libadwaita::prelude::*;
use std::collections::HashMap;

pub struct EmojiPicker {
    widget: Button,
    popover: Popover,
    search_entry: SearchEntry,
    emoji_grid: FlowBox,
    emoji_map: HashMap<&'static str, &'static str>,
}

impl EmojiPicker {
    pub fn new() -> Self {
        let widget = Button::builder()
            .icon_name("emoticon-symbolic")
            .tooltip_text("Insert emoji")
            .css_classes(vec!["flat".to_string()])
            .build();

        let popover = Popover::builder()
            .position(gtk4::PositionType::Bottom)
            .build();

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .width_request(350)
            .height_request(300)
            .build();

        let search_entry = SearchEntry::builder()
            .placeholder_text("Search emojis...")
            .build();

        let scrolled_window = ScrolledWindow::builder()
            .vexpand(true)
            .policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic)
            .build();

        let emoji_grid = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(10)
            .min_children_per_line(8)
            .row_spacing(2)
            .column_spacing(2)
            .build();

        // Initialize emoji map
        let emoji_map = Self::initialize_emoji_map();

        // Populate emoji grid
        Self::populate_emoji_grid(&emoji_grid, &emoji_map);

        scrolled_window.set_child(Some(&emoji_grid));

        content.append(&search_entry);
        content.append(&scrolled_window);

        popover.set_child(Some(&content));

        let mut picker = Self {
            widget,
            popover,
            search_entry,
            emoji_grid,
            emoji_map,
        };

        picker.setup_connections();

        picker
    }

    fn setup_connections(&self) {
        // Show popover when button is clicked
        self.widget.connect_clicked(clone!(@strong self.popover as popover => move |_| {
            popover.popup();
        }));

        // Filter emojis on search
        self.search_entry.connect_search_changed(clone!(@strong self as this => move |entry| {
            let search_text = entry.text().to_string().to_lowercase();
            this.filter_emojis(&search_text);
        }));

        // Handle emoji selection
        self.emoji_grid.connect_child_activated(clone!(@strong self.popover as popover) => move |_, child| {
            if let Some(button) = child.first_child().and_downcast_ref::<Button>() {
                if let Some(label) = button.first_child().and_downcast_ref::<Label>() {
                    let emoji = label.text().to_string();
                    
                    // TODO: Insert emoji into text buffer
                    popover.popdown();
                }
            }
        }));
    }

    fn initialize_emoji_map() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        
        // Smilies
        map.insert("😀", "grinning face");
        map.insert("😃", "grinning face with big eyes");
        map.insert("😄", "grinning face with smiling eyes");
        map.insert("😁", "beaming face with smiling eyes");
        map.insert("😅", "grinning face with sweat");
        map.insert("🤣", "rolling on the floor laughing");
        map.insert("😂", "face with tears of joy");
        map.insert("🙂", "slightly smiling face");
        map.insert("🙃", "upside-down face");
        map.insert("😉", "winking face");
        map.insert("😊", "smiling face with smiling eyes");
        map.insert("😇", "smiling face with halo");
        map.insert("🥰", "smiling face with hearts");
        map.insert("😍", "heart-eyes face");
        map.insert("🤩", "star-struck face");
        map.insert("😘", "face blowing a kiss");
        map.insert("😗", "kissing face");
        map.insert("😚", "kissing face with smiling eyes");
        map.insert("😙", "kissing face with closed eyes");
        map.insert("😋", "yum face");

        // Gestures
        map.insert("👋", "waving hand");
        map.insert("🤚", "raised back of hand");
        map.insert("🖐", "raised hand with fingers splayed");
        map.insert("✋", "raised hand");
        map.insert("🖖", "vulcan salute");
        map.insert("👌", "OK hand");
        map.insert("🤌", "pinched fingers");
        map.insert("🤏", "pinching hand");
        map.insert("✌️", "victory hand");
        map.insert("🤞", "crossed fingers");
        map.insert("🤟", "love-you gesture");
        map.insert("🤘", "sign of the horns");
        map.insert("🤙", "call me hand");
        map.insert("👈", "backhand index pointing left");
        map.insert("👉", "backhand index pointing right");
        map.insert("👆", "backhand index pointing up");
        map.insert("🖕", "middle finger");
        map.insert("👇", "backhand index pointing down");
        map.insert("☝️", "index pointing up");

        // Animals
        map.insert("🐶", "dog face");
        map.insert("🐱", "cat face");
        map.insert("🐭", "mouse face");
        map.insert("🐹", "hamster face");
        map.insert("🐰", "rabbit face");
        map.insert("🦊", "fox face");
        map.insert("🐻", "bear face");
        map.insert("🐼", "panda face");
        map.insert("🐨", "koala");
        map.insert("🐯", "tiger face");
        map.insert("🦁", "lion face");
        map.insert("🐮", "cow face");
        map.insert("🐷", "pig face");
        map.insert("🐽", "pig nose");
        map.insert("🐸", "frog face");
        map.insert("🐵", "monkey face");
        map.insert("🙈", "see-no-evil monkey");
        map.insert("🙉", "hear-no-evil monkey");
        map.insert("🙊", "speak-no-evil monkey");
        map.insert("🐒", "monkey");

        // Objects
        map.insert("⌚", "watch");
        map.insert("📱", "mobile phone");
        map.insert("📲", "mobile phone with arrow");
        map.insert("💻", "laptop");
        map.insert("⌨️", "keyboard");
        map.insert("🖥️", "desktop computer");
        map.insert("🖨️", "printer");
        map.insert("🖱️", "computer mouse");
        map.insert("🖲️", "trackball");
        map.insert("🕹️", "joystick");
        map.insert("💽", "optical disk");
        map.insert("💾", "floppy disk");
        map.insert("💿", "optical disc");
        map.insert("📀", "dvd");

        // Symbols
        map.insert("❤️", "red heart");
        map.insert("🧡", "orange heart");
        map.insert("💛", "yellow heart");
        map.insert("💚", "green heart");
        map.insert("💙", "blue heart");
        map.insert("💜", "purple heart");
        map.insert("🖤", "black heart");
        map.insert("🤍", "white heart");
        map.insert("🤎", "brown heart");
        map.insert("💔", "broken heart");
        map.insert("❣️", "exclamation heart");
        map.insert("💕", "two hearts");
        map.insert("💞", "revolving hearts");
        map.insert("💓", "beating heart");
        map.insert("💗", "growing heart");
        map.insert("💖", "sparkling heart");
        map.insert("💘", "heart with arrow");
        map.insert("💝", "heart with ribbon");

        map
    }

    fn populate_emoji_grid(grid: &FlowBox, emoji_map: &HashMap<&str, &str>) {
        for (emoji, description) in emoji_map {
            let button = Button::builder()
                .css_classes(vec!["emoji-button".to_string()])
                .tooltip_text(*description)
                .build();

            let label = Label::builder()
                .label(*emoji)
                .css_classes(vec!["emoji-label".to_string()])
                .build();

            button.set_child(Some(&label));

            let flow_box_child = FlowBoxChild::builder()
                .child(&button)
                .build();

            grid.append(&flow_box_child);
        }
    }

    fn filter_emojis(&self, search_text: &str) {
        // Clear current grid
        while let Some(child) = self.emoji_grid.first_child() {
            self.emoji_grid.remove(&child);
        }

        // Repopulate with filtered results
        for (emoji, description) in &self.emoji_map {
            if search_text.is_empty() || 
               description.to_lowercase().contains(search_text) ||
               emoji.to_lowercase().contains(search_text) {
                
                let button = Button::builder()
                    .css_classes(vec!["emoji-button".to_string()])
                    .tooltip_text(*description)
                    .build();

                let label = Label::builder()
                    .label(*emoji)
                    .css_classes(vec!["emoji-label".to_string()])
                    .build();

                button.set_child(Some(&label));

                let flow_box_child = FlowBoxChild::builder()
                    .child(&button)
                    .build();

                self.emoji_grid.append(&flow_box_child);
            }
        }
    }

    pub fn get_widget(&self) -> &Button {
        &self.widget
    }

    pub fn set_relative_to(&self, widget: &impl IsA<gtk4::Widget>) {
        self.popover.set_parent(Some(widget));
    }
}