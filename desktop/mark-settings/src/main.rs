use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ScrolledWindow, Stack,
    StackSidebar,
};
use mark_hardware_detect::HardwareInventory;

const APP_ID: &str = "com.marklinux.settings";

fn make_page(title: &str, body: &str) -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 12);
    page.set_margin_top(24);
    page.set_margin_start(24);
    page.set_margin_end(24);
    page.set_margin_bottom(24);
    let t = Label::new(Some(title));
    t.set_halign(gtk4::Align::Start);
    let b = Label::new(Some(body));
    b.set_halign(gtk4::Align::Start);
    b.set_wrap(true);
    page.append(&t);
    page.append(&b);
    page
}

fn system_page_body(inventory: &HardwareInventory) -> String {
    let mut body = String::from("Mark Linux 0.0.1_dev (build 1101126_1)\n");
    body.push_str(&format!(
        "CPU: {}\n",
        inventory.cpu_model.as_deref().unwrap_or("unavailable")
    ));
    body.push_str(&format!(
        "Memory: {:.2} GiB\n",
        inventory
            .memory_bytes
            .map(|bytes| bytes as f64 / 1_073_741_824.0)
            .unwrap_or(0.0)
    ));
    body.push_str(&format!(
        "Logical threads: {}\n",
        inventory
            .logical_threads
            .map(|threads| threads.to_string())
            .unwrap_or_else(|| "unavailable".to_owned())
    ));
    body.push_str(&format!("Storage devices: {}\n", inventory.storage_devices.len()));
    for disk in &inventory.storage_devices {
        let size = disk
            .size_bytes
            .map(|bytes| format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0))
            .unwrap_or_else(|| "size unavailable".to_owned());
        body.push_str(&format!(
            "  {} - {} - {} - {}\n",
            disk.name,
            disk.model.as_deref().unwrap_or("model unavailable"),
            size,
            if disk.rotational == Some(true) {
                "rotational"
            } else if disk.rotational == Some(false) {
                "non-rotational"
            } else {
                "media type unknown"
            },
        ));
    }
    let caps = mark_power_manager::PowerCapabilities::detect();
    body.push_str(&format!(
        "Power: suspend={} hibernate={} hybrid_sleep={} lock={} logout={} logind={}\n",
        caps.suspend,
        caps.hibernate,
        caps.hybrid_sleep,
        caps.lock,
        caps.logout,
        caps.logind_available
    ));
    body
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Mark Settings")
            .default_width(1100)
            .default_height(720)
            .build();

        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);

        let sidebar = StackSidebar::new();
        sidebar.set_stack(&stack);

        let content = ScrolledWindow::new();
        content.set_child(Some(&stack));
        content.set_hexpand(true);

        let main_box = GtkBox::new(Orientation::Horizontal, 0);
        main_box.append(&sidebar);
        main_box.append(&content);

        let inventory = mark_hardware_detect::scan_linux().ok().unwrap_or_default();
        stack.add_titled(
            &make_page("System", &system_page_body(&inventory)),
            Some("system"),
            "System",
        );
        stack.add_titled(
            &make_page(
                "Personalization",
                "Wallpaper, themes, accent colors, dark/light mode, icons, cursor, fonts, panel appearance.",
            ),
            Some("personalization"),
            "Personalization",
        );
        stack.add_titled(
            &make_page(
                "Display",
                "Resolution, refresh rate, monitor arrangement, scaling, orientation, night light.",
            ),
            Some("display"),
            "Display",
        );
        stack.add_titled(
            &make_page(
                "Network & Internet",
                "Wi-Fi, Ethernet, VPN, connection properties.",
            ),
            Some("network"),
            "Network",
        );
        stack.add_titled(
            &make_page(
                "Bluetooth & Devices",
                "Bluetooth, mouse, keyboard, touchpad, audio devices.",
            ),
            Some("bluetooth"),
            "Bluetooth",
        );
        stack.add_titled(
            &make_page(
                "Sound",
                "Output/input devices, volume, microphone level, per-app volume.",
            ),
            Some("sound"),
            "Sound",
        );
        stack.add_titled(
            &make_page(
                "Applications",
                "Installed apps, defaults, permissions, uninstall, startup apps.",
            ),
            Some("applications"),
            "Applications",
        );
        stack.add_titled(
            &make_page(
                "Updates",
                "Check, download, install updates via Pacman. Update history.",
            ),
            Some("updates"),
            "Updates",
        );
        stack.add_titled(
            &make_page(
                "Accounts",
                "User info, profile picture, password, login preferences.",
            ),
            Some("accounts"),
            "Accounts",
        );
        stack.add_titled(
            &make_page(
                "Privacy & Security",
                "Firewall, permissions, screen locking, security status.",
            ),
            Some("privacy"),
            "Privacy",
        );
        stack.add_titled(
            &make_page(
                "Accessibility",
                "Text scaling, cursor size, high contrast, keyboard accessibility, reduced motion.",
            ),
            Some("accessibility"),
            "Accessibility",
        );

        window.set_child(Some(&main_box));
        window.present();
    });
    app.run();
}
