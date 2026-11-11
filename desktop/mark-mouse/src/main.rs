use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};

const APP_ID: &str = "com.marklinux.mouse";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Mark Mouse Interaction")
            .default_width(640)
            .default_height(480)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let label = Label::new(Some(
            "Mouse interaction module scaffold.\n\
             Implement context menus, drag-and-drop, selection rectangles here.",
        ));
        vbox.append(&label);
        window.set_child(Some(&vbox));
        window.present();
    });
    app.run();
}
