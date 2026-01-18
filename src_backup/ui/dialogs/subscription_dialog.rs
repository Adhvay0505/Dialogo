use gtk4::prelude::*;
use gtk4::{Window, Box as GtkBox, Label, Button, Image, TextView};
use libadwaita::prelude::*;
use libadwaita::{PreferencesGroup, ActionRow};
use xmpp_parsers::Jid;

pub struct SubscriptionDialog {
    window: gtk4::Window,
    from_jid: Jid,
    callback: Option<Box<dyn FnOnce(Jid, bool)>>,
}

impl SubscriptionDialog {
    pub fn new(parent: &impl IsA<Window>, from_jid: Jid) -> Self {
        let window = gtk4::Window::builder()
            .title("Subscription Request")
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

        // Header with avatar
        let header_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .margin_bottom(20)
            .build();

        let avatar = Image::builder()
            .icon_name("avatar-default-symbolic")
            .icon_size(gtk4::IconSize::Large)
            .build();

        let info_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .build();

        let title_label = Label::builder()
            .label("Subscription Request")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading".to_string()])
            .build();

        let jid_label = Label::builder()
            .label(&from_jid.to_string())
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();

        info_box.append(&title_label);
        info_box.append(&jid_label);

        header_box.append(&avatar);
        header_box.append(&info_box);

        // Message
        let message_label = Label::builder()
            .label(&format!("{} wants to add you to their contact list and see your presence status.", from_jid.to_string()))
            .halign(gtk4::Align::Start)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::Word)
            .margin_bottom(20)
            .build();

        // Options group
        let options_group = PreferencesGroup::builder()
            .title("Options")
            .build();

        let approve_row = ActionRow::builder()
            .title("Approve and Add to Contact List")
            .subtitle("Allow them to see your presence and add them to your contacts")
            .activatable(true)
            .build();

        let approve_only_row = ActionRow::builder()
            .title("Approve Only")
            .subtitle("Allow them to see your presence but don't add them to your contacts")
            .activatable(true)
            .build();

        options_group.add(&approve_row);
        options_group.add(&approve_only_row);

        // Buttons
        let button_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .margin_top(12)
            .build();

        let ignore_button = Button::builder()
            .label("Ignore")
            .css_classes(vec!["destructive-action".to_string()])
            .build();

        let block_button = Button::builder()
            .label("Block")
            .css_classes(vec!["destructive-action".to_string()])
            .build();

        button_box.append(&ignore_button);
        button_box.append(&block_button);

        // Assemble dialog
        content.append(&header_box);
        content.append(&message_label);
        content.append(&options_group);
        content.append(&button_box);

        window.set_content(Some(&content));

        let mut dialog = Self {
            window,
            from_jid,
            callback: None,
        };

        // Connect handlers
        approve_row.connect_activated(clone!(@strong dialog.window as window,
                                                 @strong dialog.from_jid as from_jid => move |_| {
            window.close();
            // TODO: Call callback with approve=true, add_to_roster=true
        }));

        approve_only_row.connect_activated(clone!(@strong dialog.window as window,
                                                     @strong dialog.from_jid as from_jid => move |_| {
            window.close();
            // TODO: Call callback with approve=true, add_to_roster=false
        }));

        ignore_button.connect_clicked(clone!(@strong dialog.window as window,
                                                 @strong dialog.from_jid as from_jid => move |_| {
            window.close();
            // TODO: Call callback with approve=false
        }));

        block_button.connect_clicked(clone!(@strong dialog.window as window,
                                                @strong dialog.from_jid as from_jid => move |_| {
            window.close();
            // TODO: Block the user and call callback with approve=false
        }));

        dialog
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnOnce(Jid, bool) + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    pub fn show(&self) {
        self.window.show();
    }
}