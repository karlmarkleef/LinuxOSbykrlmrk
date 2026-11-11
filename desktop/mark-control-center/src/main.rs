use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation};
use mark_control_center::ControlBackend;
use mark_power_manager::PowerBackend;

const APP_ID: &str = "com.marklinux.controlcenter";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Mark Control Center")
            .default_width(440)
            .default_height(640)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let status = Label::new(Some("Mark Control Center\nStatus: loading"));
        vbox.append(&status);

        let wifi_on = Button::builder().label("Wi-Fi On").build();
        let wifi_on_status = status.clone();
        wifi_on.connect_clicked(move |_| {
            let backend = ControlBackend::new();
            let result = backend.set_wifi(true);
            wifi_on_status.set_text(&format!("Wi-Fi on: {}", result));
        });
        let wifi_off = Button::builder().label("Wi-Fi Off").build();
        let wifi_off_status = status.clone();
        wifi_off.connect_clicked(move |_| {
            let backend = ControlBackend::new();
            let result = backend.set_wifi(false);
            wifi_off_status.set_text(&format!("Wi-Fi off: {}", result));
        });
        let bluetooth_on = Button::builder().label("Bluetooth On").build();
        let bluetooth_on_status = status.clone();
        bluetooth_on.connect_clicked(move |_| {
            let backend = ControlBackend::new();
            let result = backend.set_bluetooth(true);
            bluetooth_on_status.set_text(&format!("Bluetooth on: {}", result));
        });
        let bluetooth_off = Button::builder().label("Bluetooth Off").build();
        let bluetooth_off_status = status.clone();
        bluetooth_off.connect_clicked(move |_| {
            let backend = ControlBackend::new();
            let result = backend.set_bluetooth(false);
            bluetooth_off_status.set_text(&format!("Bluetooth off: {}", result));
        });
        let volume_down = Button::builder().label("Volume Down").build();
        let volume_down_status = status.clone();
        volume_down.connect_clicked(move |_| {
            let backend = ControlBackend::new();
            let status = backend.audio_status().unwrap_or_default();
            let next = status.volume_percent.map_or(50, |value| value.saturating_sub(5));
            let result = backend.set_volume(next);
            volume_down_status.set_text(&format!("Volume down: {} -> {}", status.volume_percent.unwrap_or(0), next));
        });
        let volume_up = Button::builder().label("Volume Up").build();
        let volume_up_status = status.clone();
        volume_up.connect_clicked(move |_| {
            let backend = ControlBackend::new();
            let status = backend.audio_status().unwrap_or_default();
            let next = status.volume_percent.map_or(50, |value| value.saturating_add(5));
            let result = backend.set_volume(next);
            volume_up_status.set_text(&format!("Volume up: {} -> {}", status.volume_percent.unwrap_or(0), next));
        });
        let hibernate = Button::builder().label("Hibernate").build();
        let power_status = status.clone();
        hibernate.connect_clicked(move |_| {
            let backend = PowerBackend::new();
            if let Ok(backend) = backend {
                if let Err(error) = backend.hibernate() {
                    power_status.set_text(&format!("Hibernate failed: {}", error));
                } else {
                    power_status.set_text("Hibernate requested");
                }
            }
        });
        let lock = Button::builder().label("Lock Screen").build();
        let lock_status = status.clone();
        lock.connect_clicked(move |_| {
            let backend = mark_power_manager::PowerBackend::new();
            if let Ok(backend) = backend {
                if let Err(error) = backend.lock() {
                    lock_status.set_text(&format!("Lock failed: {}", error));
                } else {
                    lock_status.set_text("Lock requested");
                }
            }
        });
        vbox.append(&wifi_on);
        vbox.append(&wifi_off);
        vbox.append(&bluetooth_on);
        vbox.append(&bluetooth_off);
        vbox.append(&volume_down);
        vbox.append(&volume_up);
        vbox.append(&hibernate);
        vbox.append(&lock);
        vbox.append(&status);

        window.set_child(Some(&vbox));
        window.present();
    });
    app.run();
}
