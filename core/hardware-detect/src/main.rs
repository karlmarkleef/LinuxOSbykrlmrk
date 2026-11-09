#[cfg(target_os = "linux")]
fn main() {
    match mark_hardware_detect::scan_linux() {
        Ok(report) => {
            if std::env::args().any(|a| a == "--profile" || a == "-p") {
                println!("{}", report.recommend_profile().slug);
                return;
            }
            if std::env::args().any(|a| a == "--profile-name") {
                println!("{}", report.recommend_profile().name);
                return;
            }

            let recommendation = report.recommend_profile();
            println!("Mark Linux hardware inventory (read-only)");
            println!(
                "CPU: {}",
                report.cpu_model.as_deref().unwrap_or("unavailable")
            );
            println!("Physical cores: {}", display_count(report.physical_cores));
            println!("Logical threads: {}", display_count(report.logical_threads));
            match report.memory_bytes {
                Some(bytes) => println!("Memory: {:.2} GiB", bytes as f64 / 1_073_741_824.0),
                None => println!("Memory: unavailable"),
            }

            println!("Storage devices (metadata only; no device contents read):");
            if report.storage_devices.is_empty() {
                println!("  none reported by sysfs");
            }
            for disk in report.storage_devices {
                let size = disk
                    .size_bytes
                    .map(|value| format!("{:.1} GiB", value as f64 / 1_073_741_824.0))
                    .unwrap_or_else(|| "size unavailable".to_owned());
                let model = disk.model.as_deref().unwrap_or("model unavailable");
                let read_only = match disk.read_only {
                    Some(true) => "read-only",
                    Some(false) => "writable flag clear",
                    None => "read-only flag unavailable",
                };
                let media = match disk.rotational {
                    Some(true) => "rotational",
                    Some(false) => "non-rotational",
                    None => "media type unknown",
                };
                println!(
                    "  {} — {} — {} — {} — {}",
                    disk.name, model, size, media, read_only
                );
            }

            println!("Display adapters:");
            if report.display_adapters.is_empty() {
                println!("  none reported by PCI sysfs");
            }
            for gpu in report.display_adapters {
                let vendor = gpu
                    .vendor_id
                    .map(|value| format!("{value:04x}"))
                    .unwrap_or_else(|| "unknown".to_owned());
                let device = gpu
                    .device_id
                    .map(|value| format!("{value:04x}"))
                    .unwrap_or_else(|| "unknown".to_owned());
                let driver = gpu.driver.as_deref().unwrap_or("driver unavailable");
                let vram = gpu
                    .vram_bytes
                    .map(|bytes| format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0))
                    .unwrap_or_else(|| "unavailable from sysfs".to_owned());
                println!(
                    "  {} — vendor {}, device {}, class {:06x} — driver {} — VRAM {}",
                    gpu.pci_address, vendor, device, gpu.class_code, driver, vram,
                );
            }

            println!("Display connectors (EDID mode is preferred, not necessarily active):");
            if report.monitors.is_empty() {
                println!("  none reported by DRM sysfs");
            }
            for monitor in report.monitors {
                let connection = match monitor.connected {
                    Some(true) => "connected",
                    Some(false) => "disconnected",
                    None => "connection state unavailable",
                };
                let mode = monitor.preferred_mode.map_or_else(
                    || "preferred mode unavailable".to_owned(),
                    |mode| {
                        let refresh = mode.refresh_millihz.map_or_else(
                            || "refresh unavailable".to_owned(),
                            |value| format!("{:.3} Hz", value as f64 / 1000.0),
                        );
                        format!("preferred {}x{} @ {}", mode.width, mode.height, refresh)
                    },
                );
                println!("  {} — {} — {}", monitor.connector, connection, mode);
                if !monitor.supported_modes.is_empty() {
                    println!(
                        "    supported modes: {}",
                        monitor.supported_modes.join(", ")
                    );
                }
            }

            println!("Network adapters:");
            if report.network_adapters.is_empty() {
                println!("  none reported by sysfs");
            }
            for adapter in report.network_adapters {
                println!(
                    "  {} — {} — {} — driver {}",
                    adapter.name,
                    adapter.kind,
                    adapter.operstate.as_deref().unwrap_or("state unavailable"),
                    adapter.driver.as_deref().unwrap_or("unavailable"),
                );
            }

            println!("Bluetooth controllers:");
            if report.bluetooth_adapters.is_empty() {
                println!("  none reported by sysfs");
            }
            for adapter in report.bluetooth_adapters {
                println!(
                    "  {} — driver {}",
                    adapter.name,
                    adapter.driver.as_deref().unwrap_or("unavailable"),
                );
            }

            println!("Adaptive performance profile recommendation:");
            println!(
                "  Recommended profile: {} ({})",
                recommendation.name, recommendation.slug
            );
            println!("  Reason: {}", recommendation.reason);
        }
        Err(error) => {
            eprintln!("Could not read Linux hardware inventory: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("mark-hardware reads Linux procfs/sysfs and must run on Linux.");
    std::process::exit(2);
}

fn display_count(value: Option<usize>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |count| count.to_string())
}
