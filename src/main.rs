mod task_object;
mod task_row;
mod utils;
mod window;

use gio::{ActionEntry, File};
use gtk::gdk::Display;
use gtk::prelude::*;
use gtk::{gio, glib, Application, CssProvider};
// use std::sync::atomic::{AtomicUsize, Ordering};
use window::Window;

const APP_ID: &str = "org.gtk_rs.ToDoApp";

// fn get_id() -> usize {
//     static COUNTER: AtomicUsize = AtomicUsize::new(1);
//     COUNTER.fetch_add(1, Ordering::Relaxed)
// }

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("src_resources.gresource")
        .expect("Failed to register resources.");

    // Create a new applicationw
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to signals
    app.connect_startup(|app| {
        setup_shortcuts(app);
        load_css();
    });
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_resource("/org/gtk_rs/todo/style.css");

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_shortcuts(app: &Application) {
    app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
    app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
    app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
}

fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    // Add action "close" to `window` taking no parameter
    let action_close = ActionEntry::builder("otro")
        .activate(|window: &Window, _, _| {
            window.close();
        })
        .build();

    window.add_action_entries([action_close]);

    window.present();
}
