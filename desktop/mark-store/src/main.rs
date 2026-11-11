use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};

const APP_ID: &str = "com.marklinux.store";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Mark Store")
            .default_width(900)
            .default_height(600)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let backend = mark_store::PacmanBackend::new();
        let installed = backend.installed().unwrap_or_default();
        let body = format!(
            "Mark Store\nInstalled packages: {}\nUse search, install, update and remove actions from this interface.",
            installed.len()
        );
        let label = Label::new(Some(body.as_str()));
        vbox.append(&label);
        window.set_child(Some(&vbox));
        window.present();
    });
    app.run();
}
