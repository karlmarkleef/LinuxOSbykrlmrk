use mark_power_manager::{parse_action, PowerAction, PowerBackend, PowerCapabilities, PowerSettings};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut only_capabilities = false;
    let mut only_settings = false;
    let mut confirm = false;
    let mut action: Option<PowerAction> = None;

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].clone();
        match arg.as_str() {
            "--capabilities" | "-c" => only_capabilities = true,
            "--settings" | "-s" => only_settings = true,
            "--yes" | "--confirm" | "-y" => confirm = true,
            "--action" | "-a" => {
                if let Some(value) = args.get(index + 1) {
                    action = parse_action(&value.clone());
                }
                index += 1;
            }
            other => {
                if !other.starts_with('-') && action.is_none() {
                    action = parse_action(&other);
                }
            }
        }
        index += 1;
    }

    if only_capabilities {
        print_capabilities();
    }
    if only_settings {
        print_settings();
    }
    if only_capabilities || only_settings {
        return;
    }

    let Some(action) = action else {
        print_capabilities();
        return;
    };

    match action {
        PowerAction::Shutdown => {
            if !confirm {
                println!("mark-power: shutdown requires explicit confirmation (--yes or --confirm)");
                return;
            }
            if let Ok(backend) = PowerBackend::new() {
                if let Err(error) = backend.shutdown() {
                    eprintln!("mark-power: shutdown failed: {}", error);
                } else {
                    println!("mark-power: shutdown requested");
                }
            }
        }
        PowerAction::Restart => {
            if !confirm {
                println!("mark-power: restart requires explicit confirmation (--yes or --confirm)");
                return;
            }
            if let Ok(backend) = PowerBackend::new() {
                if let Err(error) = backend.restart() {
                    eprintln!("mark-power: restart failed: {}", error);
                } else {
                    println!("mark-power: restart requested");
                }
            }
        }
        PowerAction::Suspend => {
            if let Ok(backend) = PowerBackend::new() {
                if let Err(error) = backend.suspend() {
                    eprintln!("mark-power: suspend failed: {}", error);
                } else {
                    println!("mark-power: suspend requested");
                }
            }
        }
        PowerAction::Hibernate => {
            if let Ok(backend) = PowerBackend::new() {
                if let Err(error) = backend.hibernate() {
                    eprintln!("mark-power: hibernate failed: {}", error);
                } else {
                    println!("mark-power: hibernate requested");
                }
            }
        }
        PowerAction::Lock => {
            if let Ok(backend) = PowerBackend::new() {
                if let Err(error) = backend.lock() {
                    eprintln!("mark-power: lock failed: {}", error);
                } else {
                    println!("mark-power: lock requested");
                }
            }
        }
        PowerAction::Logout => {
            if let Ok(backend) = PowerBackend::new() {
                if let Err(error) = backend.logout() {
                    eprintln!("mark-power: logout failed: {}", error);
                } else {
                    println!("mark-power: logout requested");
                }
            }
        }
    }
}

fn print_capabilities() {
    let caps = PowerCapabilities::detect();
    println!("Mark Linux power capabilities:");
    println!("  shutdown:     {}", caps.shutdown);
    println!("  restart:      {}", caps.restart);
    println!("  suspend:      {}", caps.suspend);
    println!("  hibernate:    {}", caps.hibernate);
    println!("  hybrid_sleep: {}", caps.hybrid_sleep);
    println!("  lock:         {}", caps.lock);
    println!("  logout:       {}", caps.logout);
    println!("  logind:       {}", caps.logind_available);
}

fn print_settings() {
    let settings = PowerSettings::load();
    println!("Mark Linux power settings:");
    println!("  auto-sleep-minutes: {}", settings.auto_sleep_minutes);
    println!("  hibernate-enabled: {}", settings.hibernate_enabled);
    println!("  confirm-shutdown: {}", settings.confirm_shutdown);
    println!("  confirm-restart: {}", settings.confirm_restart);
    println!("  lock-screen-enabled: {}", settings.lock_screen_enabled);
    println!("  lid-close-action: {:?}", settings.lid_close_action);
    println!("  power-button-action: {:?}", settings.power_button_action);
    println!("  idle-action: {:?}", settings.idle_action);
}
