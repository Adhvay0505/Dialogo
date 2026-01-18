use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox,
    Label, Button, Entry, Dialog, ResponseType,
};
use libadwaita::prelude::*;

pub mod about_dialog;
pub mod connection_dialog;
pub mod add_contact_dialog;
pub mod subscription_dialog;
pub mod file_transfer_dialog;

pub use about_dialog::AboutDialog;
pub use connection_dialog::ConnectionDialog;
pub use add_contact_dialog::AddContactDialog;
pub use subscription_dialog::SubscriptionDialog;
pub use file_transfer_dialog::FileTransferDialog;

// Re-export dialog utilities
pub fn show_info_dialog(
    parent: &impl IsA<gtk4::Window>,
    title: &str,
    message: &str,
) {
    let dialog = gtk4::MessageDialog::builder()
        .title(title)
        .message_type(gtk4::MessageType::Info)
        .buttons(gtk4::ButtonsType::Ok)
        .text(message)
        .modal(true)
        .transient_for(parent)
        .build();

    dialog.connect_response(None, move |dialog, _| {
        dialog.close();
    });

    dialog.show();
}

pub fn show_error_dialog(
    parent: &impl IsA<gtk4::Window>,
    title: &str,
    message: &str,
) {
    let dialog = gtk4::MessageDialog::builder()
        .title(title)
        .message_type(gtk4::MessageType::Error)
        .buttons(gtk4::ButtonsType::Ok)
        .text(message)
        .modal(true)
        .transient_for(parent)
        .build();

    dialog.connect_response(None, move |dialog, _| {
        dialog.close();
    });

    dialog.show();
}

pub fn show_question_dialog(
    parent: &impl IsA<gtk4::Window>,
    title: &str,
    message: &str,
) -> gtk4::MessageDialog {
    gtk4::MessageDialog::builder()
        .title(title)
        .message_type(gtk4::MessageType::Question)
        .buttons(gtk4::ButtonsType::YesNo)
        .text(message)
        .modal(true)
        .transient_for(parent)
        .build()
}

pub fn show_input_dialog(
    parent: &impl IsA<gtk4::Window>,
    title: &str,
    message: &str,
    default_value: Option<&str>,
) -> gtk4::Window {
    let dialog = gtk4::Window::builder()
        .title(title)
        .modal(true)
        .default_width(400)
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

    let label = Label::builder()
        .label(message)
        .halign(gtk4::Align::Start)
        .build();

    let entry = Entry::builder()
        .text(default_value.unwrap_or(""))
        .build();

    let button_box = GtkBox::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk4::Align::End)
        .margin_top(12)
        .build();

    let cancel_button = Button::builder()
        .label("Cancel")
        .build();

    let ok_button = Button::builder()
        .label("OK")
        .css_classes(vec!["suggested-action".to_string()])
        .build();

    button_box.append(&cancel_button);
    button_box.append(&ok_button);

    content.append(&label);
    content.append(&entry);
    content.append(&button_box);

    dialog.set_content(Some(&content));

    cancel_button.connect_clicked(clone!(@strong dialog => move |_| {
        dialog.close();
    }));

    dialog
}