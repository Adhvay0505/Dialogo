// Minimal test for GTK4 setup
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button};
use gtk4 as gtk;

const APP_ID: &str = "com.example.xmpp-client";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label and margins
    let button = Button::builder()
        .label("Hello XMPP Client!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|_| {
        println!("Button clicked!");
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("XMPP Client Test")
        .default_width(350)
        .default_height(200)
        .child(&button)
        .build();

    // Present window
    window.present();
}