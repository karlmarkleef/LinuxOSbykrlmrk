//! Read-only Linux hardware inventory from procfs and sysfs.
//!
//! The scanner never writes to a device or runs an external command. Storage
//! fields are informational only; no block-device contents are opened.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct HardwareInventory {
    pub cpu_model: Option<String>,
    pub physical_cores: Option<usize>,
    pub logical_threads: Option<usize>,
    pub memory_bytes: Option<u64>,
    pub storage_devices: Vec<StorageDevice>,
    pub display_adapters: Vec<DisplayAdapter>,
    pub monitors: Vec<DisplayMonitor>,
    pub network_adapters: Vec<NetworkAdapter>,
    pub bluetooth_adapters: Vec<BluetoothAdapter>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StorageDevice {
    pub name: String,
    pub model: Option<String>,
    pub size_bytes: Option<u64>,
    /// None means the kernel did not expose a readable flag.
    pub read_only: Option<bool>,
    /// True is normally an HDD; false is normally an SSD or flash device.
    pub rotational: Option<bool>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DisplayAdapter {
    pub pci_address: String,
    pub vendor_id: Option<u16>,
    pub device_id: Option<u16>,
    pub class_code: u32,
    pub driver: Option<String>,
    /// Reported only when the active kernel driver exposes this sysfs value.
    pub vram_bytes: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    /// Millihertz, derived from the first EDID detailed timing descriptor.
    pub refresh_millihz: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DisplayMonitor {
    pub connector: String,
    pub connected: Option<bool>,
    /// Modes advertised by DRM. They are supported modes, not necessarily the
    /// mode currently selected by a Wayland compositor.
    pub supported_modes: Vec<String>,
    /// Preferred EDID detailed timing, when the monitor provides readable EDID.
    pub preferred_mode: Option<DisplayMode>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NetworkAdapter {
    pub name: String,
    pub kind: String,
    pub operstate: Option<String>,
    pub driver: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BluetoothAdapter {
    pub name: String,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedProfile {
    UltraLite,
    Balanced,
    Cinematic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRecommendation {
    pub profile: RecommendedProfile,
    pub name: &'static str,
    pub slug: &'static str,
    pub reason: String,
}

impl HardwareInventory {
    /// Recommend an adaptive desktop performance profile based on hardware specs.
    pub fn recommend_profile(&self) -> ProfileRecommendation {
        let ram_bytes = self.memory_bytes.unwrap_or(0);
        let threads = self.logical_threads.unwrap_or(1);
        let four_gib = 4 * 1024 * 1024 * 1024;
        let eight_gib = 8 * 1024 * 1024 * 1024;

        if (ram_bytes > 0 && ram_bytes < four_gib) || threads <= 2 {
            let reason = if ram_bytes > 0 && ram_bytes < four_gib && threads <= 2 {
                format!(
                    "System has limited memory ({:.1} GiB) and {} logical thread(s); Ultra Lite disables blur, shadows, and animations to preserve resources.",
                    ram_bytes as f64 / 1_073_741_824.0,
                    threads
                )
            } else if ram_bytes > 0 && ram_bytes < four_gib {
                format!(
                    "System memory ({:.1} GiB) is below 4 GiB; Ultra Lite disables compositor blur and shadows to maximize available RAM.",
                    ram_bytes as f64 / 1_073_741_824.0
                )
            } else {
                format!(
                    "Processor has {} logical thread(s); Ultra Lite removes compositor transitions to ensure responsive desktop interaction.",
                    threads
                )
            };
            return ProfileRecommendation {
                profile: RecommendedProfile::UltraLite,
                name: "Ultra Lite",
                slug: "ultra-lite",
                reason,
            };
        }

        let has_discrete_gpu = self.display_adapters.iter().any(|gpu| {
            gpu.vram_bytes.is_some_and(|v| v >= 2 * 1024 * 1024 * 1024)
                || gpu.vendor_id == Some(0x10de)
                || (gpu.vendor_id == Some(0x1002) && gpu.class_code == 0x030000)
        });
        let all_rotational = !self.storage_devices.is_empty()
            && self
                .storage_devices
                .iter()
                .all(|d| d.rotational == Some(true));

        if ram_bytes >= eight_gib
            && threads >= 6
            && !all_rotational
            && (has_discrete_gpu || !self.display_adapters.is_empty())
        {
            let gpu_desc = if has_discrete_gpu {
                "dedicated GPU"
            } else {
                "capable graphics adapter"
            };
            return ProfileRecommendation {
                profile: RecommendedProfile::Cinematic,
                name: "Cinematic",
                slug: "cinematic",
                reason: format!(
                    "High-performance hardware detected ({:.1} GiB RAM, {} threads, {}); Cinematic profile enables full blur, drop shadows, and fluid animations.",
                    ram_bytes as f64 / 1_073_741_824.0,
                    threads,
                    gpu_desc
                ),
            };
        }

        ProfileRecommendation {
            profile: RecommendedProfile::Balanced,
            name: "Balanced",
            slug: "balanced",
            reason: format!(
                "Mainstream hardware detected ({:.1} GiB RAM, {} threads); Balanced profile provides smooth animations with restrained blur and shadow effects.",
                ram_bytes as f64 / 1_073_741_824.0,
                threads
            ),
        }
    }
}

/// Scan Linux procfs and sysfs roots. The caller supplies roots so parsing and
/// device enumeration can be tested against synthetic, read-only fixtures.
pub fn scan_linux_at(proc_root: &Path, sys_root: &Path) -> HardwareInventory {
    let cpu_text = fs::read_to_string(proc_root.join("cpuinfo")).unwrap_or_default();
    let (cpu_model, physical_cores, logical_threads) = parse_cpu_info(&cpu_text);
    let memory_bytes = fs::read_to_string(proc_root.join("meminfo"))
        .ok()
        .and_then(|text| parse_memory_bytes(&text));

    HardwareInventory {
        cpu_model,
        physical_cores,
        logical_threads,
        memory_bytes,
        storage_devices: scan_storage(&sys_root.join("block")),
        display_adapters: scan_display_adapters(&sys_root.join("bus/pci/devices")),
        monitors: scan_monitors(&sys_root.join("class/drm")),
        network_adapters: scan_network(&sys_root.join("class/net")),
        bluetooth_adapters: scan_bluetooth(&sys_root.join("class/bluetooth")),
    }
}

/// Scan the running Linux system using read-only kernel information files.
pub fn scan_linux() -> io::Result<HardwareInventory> {
    if !Path::new("/proc").is_dir() || !Path::new("/sys").is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Linux procfs and sysfs are required",
        ));
    }
    Ok(scan_linux_at(Path::new("/proc"), Path::new("/sys")))
}

fn parse_cpu_info(cpuinfo: &str) -> (Option<String>, Option<usize>, Option<usize>) {
    let mut model = None;
    let mut threads = 0usize;
    let mut cores = BTreeSet::new();

    for processor in cpuinfo.split("\n\n") {
        let mut processor_seen = false;
        let mut physical_id = None;
        let mut core_id = None;
        for line in processor.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            if model.is_none()
                && matches!(key, "model name" | "Hardware" | "Processor")
                && !value.is_empty()
            {
                model = Some(value.to_owned());
            }
            match key {
                "processor" | "Processor" if value.parse::<usize>().is_ok() => {
                    processor_seen = true;
                }
                "physical id" => physical_id = Some(value.to_owned()),
                "core id" => core_id = Some(value.to_owned()),
                _ => {}
            }
        }
        if processor_seen {
            threads += 1;
        }
        if let Some(core_id) = core_id {
            cores.insert((physical_id.unwrap_or_else(|| "0".to_owned()), core_id));
        }
    }

    (
        model,
        (!cores.is_empty()).then_some(cores.len()),
        (threads > 0).then_some(threads),
    )
}

fn parse_memory_bytes(meminfo: &str) -> Option<u64> {
    let kib = meminfo.lines().find_map(|line| {
        let (key, rest) = line.split_once(':')?;
        if key.trim() != "MemTotal" {
            return None;
        }
        let mut fields = rest.split_whitespace();
        let amount = fields.next()?.parse::<u64>().ok()?;
        (fields.next()? == "kB").then_some(amount)
    })?;
    kib.checked_mul(1024)
}

fn scan_storage(block_root: &Path) -> Vec<StorageDevice> {
    let Ok(entries) = fs::read_dir(block_root) else {
        return Vec::new();
    };
    let mut devices = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_virtual_block_name(&name) {
                return None;
            }
            let path = entry.path();
            if path.join("partition").exists() {
                return None;
            }
            let sectors = read_trimmed(path.join("size"))?.parse::<u64>().ok()?;
            let model = read_trimmed(path.join("device/model")).filter(|value| !value.is_empty());
            let read_only = read_trimmed(path.join("ro")).and_then(|value| match value.as_str() {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            });
            let rotational = read_trimmed(path.join("queue/rotational")).and_then(|value| {
                match value.as_str() {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => None,
                }
            });
            Some(StorageDevice {
                name,
                model,
                size_bytes: sectors.checked_mul(512),
                read_only,
                rotational,
            })
        })
        .collect::<Vec<_>>();
    devices.sort_by(|left, right| left.name.cmp(&right.name));
    devices
}

fn is_virtual_block_name(name: &str) -> bool {
    ["loop", "ram", "zram", "fd"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
        || name.starts_with("sr")
}

fn scan_display_adapters(device_root: &Path) -> Vec<DisplayAdapter> {
    let Ok(entries) = fs::read_dir(device_root) else {
        return Vec::new();
    };
    let mut adapters = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let class_code = parse_hex_u32(&read_trimmed(path.join("class"))?)?;
            if class_code >> 16 != 0x03 {
                return None;
            }
            Some(DisplayAdapter {
                pci_address: entry.file_name().to_string_lossy().into_owned(),
                vendor_id: parse_hex_u32(&read_trimmed(path.join("vendor"))?)
                    .and_then(|value| u16::try_from(value).ok()),
                device_id: parse_hex_u32(&read_trimmed(path.join("device"))?)
                    .and_then(|value| u16::try_from(value).ok()),
                class_code,
                driver: driver_name(&path.join("driver")),
                vram_bytes: read_trimmed(path.join("mem_info_vram_total"))
                    .and_then(|value| value.parse().ok()),
            })
        })
        .collect::<Vec<_>>();
    adapters.sort_by(|left, right| left.pci_address.cmp(&right.pci_address));
    adapters
}

fn scan_monitors(drm_root: &Path) -> Vec<DisplayMonitor> {
    let Ok(entries) = fs::read_dir(drm_root) else {
        return Vec::new();
    };
    let mut monitors = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("card") || !name.contains('-') {
                return None;
            }
            let path = entry.path();
            let connected =
                read_trimmed(path.join("status")).and_then(|status| match status.as_str() {
                    "connected" => Some(true),
                    "disconnected" => Some(false),
                    _ => None,
                });
            let supported_modes = read_trimmed(path.join("modes"))
                .map(|text| text.lines().map(str::to_owned).collect())
                .unwrap_or_default();
            let preferred_mode = fs::read(path.join("edid"))
                .ok()
                .and_then(|bytes| parse_edid_preferred_mode(&bytes));
            Some(DisplayMonitor {
                connector: name,
                connected,
                supported_modes,
                preferred_mode,
            })
        })
        .collect::<Vec<_>>();
    monitors.sort_by(|left, right| left.connector.cmp(&right.connector));
    monitors
}

fn parse_edid_preferred_mode(edid: &[u8]) -> Option<DisplayMode> {
    if edid.len() < 128 {
        return None;
    }
    for descriptor in (54..126).step_by(18) {
        let timing = edid.get(descriptor..descriptor + 18)?;
        let pixel_clock_units = u16::from_le_bytes([timing[0], timing[1]]) as u64;
        if pixel_clock_units == 0 {
            continue;
        }
        let h_active = timing[2] as u32 | (((timing[4] & 0xf0) as u32) << 4);
        let h_blank = timing[3] as u32 | (((timing[4] & 0x0f) as u32) << 8);
        let v_active = timing[5] as u32 | (((timing[7] & 0xf0) as u32) << 4);
        let v_blank = timing[6] as u32 | (((timing[7] & 0x0f) as u32) << 8);
        let total_pixels =
            (h_active as u64 + h_blank as u64).checked_mul(v_active as u64 + v_blank as u64)?;
        if h_active == 0 || v_active == 0 || total_pixels == 0 {
            continue;
        }
        let mut refresh_millihz = pixel_clock_units
            .checked_mul(10_000_000)?
            .checked_div(total_pixels)?;
        if timing[17] & 0x80 != 0 {
            refresh_millihz = refresh_millihz.checked_mul(2)?;
        }
        return Some(DisplayMode {
            width: h_active,
            height: v_active,
            refresh_millihz: u32::try_from(refresh_millihz).ok(),
        });
    }
    None
}

fn scan_network(net_root: &Path) -> Vec<NetworkAdapter> {
    let Ok(entries) = fs::read_dir(net_root) else {
        return Vec::new();
    };
    let mut adapters = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == "lo" {
                return None;
            }
            let path = entry.path();
            let link_type = read_trimmed(path.join("type"))?.parse::<u16>().ok()?;
            let wireless = path.join("wireless").exists()
                || path.join("phy80211").exists()
                || path.join("device/ieee80211").exists();
            let kind = if wireless {
                "wireless"
            } else if link_type == 1 {
                "ethernet"
            } else {
                "other"
            };
            Some(NetworkAdapter {
                name,
                kind: kind.to_owned(),
                operstate: read_trimmed(path.join("operstate")),
                driver: driver_name(&path.join("device/driver")),
            })
        })
        .collect::<Vec<_>>();
    adapters.sort_by(|left, right| left.name.cmp(&right.name));
    adapters
}

fn scan_bluetooth(bluetooth_root: &Path) -> Vec<BluetoothAdapter> {
    let Ok(entries) = fs::read_dir(bluetooth_root) else {
        return Vec::new();
    };
    let mut adapters = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with("hci").then(|| BluetoothAdapter {
                name,
                driver: driver_name(&entry.path().join("device/driver")),
            })
        })
        .collect::<Vec<_>>();
    adapters.sort_by(|left, right| left.name.cmp(&right.name));
    adapters
}

fn driver_name(path: &Path) -> Option<String> {
    fs::read_link(path)
        .ok()?
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
}

fn parse_hex_u32(input: &str) -> Option<u32> {
    u32::from_str_radix(input.trim().strip_prefix("0x")?, 16).ok()
}

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayAdapter, HardwareInventory, RecommendedProfile, StorageDevice,
        is_virtual_block_name, parse_cpu_info, parse_edid_preferred_mode, parse_hex_u32,
        parse_memory_bytes, scan_linux_at,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mark-hardware-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn root(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write(root: &Path, relative: &str, value: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }

    #[test]
    fn reads_cpu_model_core_topology_and_threads() {
        let cpuinfo = "processor : 0\nmodel name : AMD Ryzen 5 5600X\nphysical id : 0\ncore id : 0\n\nprocessor : 1\nmodel name : AMD Ryzen 5 5600X\nphysical id : 0\ncore id : 0\n\nprocessor : 2\nmodel name : AMD Ryzen 5 5600X\nphysical id : 0\ncore id : 1\n";
        let (model, cores, threads) = parse_cpu_info(cpuinfo);
        assert_eq!(model.as_deref(), Some("AMD Ryzen 5 5600X"));
        assert_eq!(cores, Some(2));
        assert_eq!(threads, Some(3));
    }

    #[test]
    fn converts_meminfo_kib_to_bytes_and_checks_overflow() {
        assert_eq!(
            parse_memory_bytes("MemFree: 10 kB\nMemTotal: 24576 kB\n"),
            Some(25_165_824)
        );
        assert_eq!(
            parse_memory_bytes("MemTotal: 18446744073709551615 kB\n"),
            None
        );
    }

    #[test]
    fn parses_edid_preferred_mode_and_refresh_rate() {
        let mut edid = vec![0u8; 128];
        let descriptor = 54;
        let timing = &mut edid[descriptor..descriptor + 18];
        let pixel_clock_10khz = 14_850u16;
        timing[0..2].copy_from_slice(&pixel_clock_10khz.to_le_bytes());
        timing[2] = 128; // 1920 active pixels.
        timing[3] = 24;
        timing[4] = 0x71; // 7 high bits for active pixels, 1 for blank pixels.
        timing[5] = 56; // 1080 active lines.
        timing[6] = 45;
        timing[7] = 0x40; // 4 high bits for active lines, none for blank lines.
        let mode = parse_edid_preferred_mode(&edid).unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_millihz, Some(60_000));
    }

    #[test]
    fn parses_hex_pci_ids_and_filters_virtual_devices() {
        assert_eq!(parse_hex_u32("0x10de\n"), Some(0x10de));
        assert_eq!(parse_hex_u32("10de"), None);
        assert!(is_virtual_block_name("loop0"));
        assert!(is_virtual_block_name("zram0"));
        assert!(!is_virtual_block_name("nvme0n1"));
        assert!(!is_virtual_block_name("sda"));
    }

    #[test]
    fn scans_read_only_fixture_without_opening_device_contents() {
        let tree = TempTree::new();
        let proc_root = tree.root().join("proc");
        let sys_root = tree.root().join("sys");
        write(
            &proc_root,
            "cpuinfo",
            "processor : 0\nmodel name : Fixture CPU\nphysical id : 0\ncore id : 0\n",
        );
        write(&proc_root, "meminfo", "MemTotal: 4096 kB\n");
        write(&sys_root, "block/sda/size", "1000\n");
        write(&sys_root, "block/sda/device/model", "Fixture Disk\n");
        write(&sys_root, "block/sda/ro", "0\n");
        write(&sys_root, "block/sda/queue/rotational", "1\n");
        write(&sys_root, "block/sda/sda1/partition", "1\n");
        write(&sys_root, "block/sda/sda1/size", "500\n");
        write(&sys_root, "block/loop0/size", "100\n");
        write(
            &sys_root,
            "bus/pci/devices/0000:01:00.0/class",
            "0x030000\n",
        );
        write(&sys_root, "bus/pci/devices/0000:01:00.0/vendor", "0x10de\n");
        write(&sys_root, "bus/pci/devices/0000:01:00.0/device", "0x1f06\n");
        write(
            &sys_root,
            "bus/pci/devices/0000:01:00.0/mem_info_vram_total",
            "8589934592\n",
        );
        write(&sys_root, "class/drm/card0-DP-1/status", "connected\n");
        write(
            &sys_root,
            "class/drm/card0-DP-1/modes",
            "1920x1080\n1280x720\n",
        );
        write(&sys_root, "class/net/lo/type", "772\n");
        write(&sys_root, "class/net/eth0/type", "1\n");
        write(&sys_root, "class/net/eth0/operstate", "up\n");
        write(&sys_root, "class/net/wlan0/type", "1\n");
        write(&sys_root, "class/net/wlan0/wireless/name", "phy0\n");
        write(&sys_root, "class/net/wlan0/operstate", "down\n");
        write(&sys_root, "class/bluetooth/hci0/type", "primary\n");

        let inventory = scan_linux_at(&proc_root, &sys_root);
        assert_eq!(inventory.cpu_model.as_deref(), Some("Fixture CPU"));
        assert_eq!(inventory.physical_cores, Some(1));
        assert_eq!(inventory.logical_threads, Some(1));
        assert_eq!(inventory.memory_bytes, Some(4_194_304));
        assert_eq!(inventory.storage_devices.len(), 1);
        assert_eq!(inventory.storage_devices[0].size_bytes, Some(512_000));
        assert_eq!(inventory.storage_devices[0].rotational, Some(true));
        assert_eq!(inventory.display_adapters.len(), 1);
        assert_eq!(inventory.display_adapters[0].vendor_id, Some(0x10de));
        assert_eq!(
            inventory.display_adapters[0].vram_bytes,
            Some(8_589_934_592)
        );
        assert_eq!(inventory.monitors.len(), 1);
        assert_eq!(inventory.monitors[0].connected, Some(true));
        assert_eq!(
            inventory.monitors[0].supported_modes,
            ["1920x1080", "1280x720"]
        );
        assert_eq!(inventory.network_adapters.len(), 2);
        assert_eq!(inventory.network_adapters[0].kind, "ethernet");
        assert_eq!(inventory.network_adapters[1].kind, "wireless");
        assert_eq!(inventory.bluetooth_adapters[0].name, "hci0");
    }

    #[test]
    fn recommends_ultra_lite_for_low_memory_or_dual_core() {
        let low_ram = HardwareInventory {
            memory_bytes: Some(2 * 1024 * 1024 * 1024),
            logical_threads: Some(8),
            ..Default::default()
        };
        assert_eq!(
            low_ram.recommend_profile().profile,
            RecommendedProfile::UltraLite
        );

        let dual_core = HardwareInventory {
            memory_bytes: Some(16 * 1024 * 1024 * 1024),
            logical_threads: Some(2),
            ..Default::default()
        };
        assert_eq!(
            dual_core.recommend_profile().profile,
            RecommendedProfile::UltraLite
        );
    }

    #[test]
    fn recommends_cinematic_for_multi_core_dedicated_gpu() {
        let gaming_rig = HardwareInventory {
            memory_bytes: Some(16 * 1024 * 1024 * 1024),
            logical_threads: Some(12),
            storage_devices: vec![StorageDevice {
                name: "nvme0n1".to_owned(),
                model: Some("Fast NVMe".to_owned()),
                size_bytes: Some(1_000_000_000_000),
                read_only: Some(false),
                rotational: Some(false),
            }],
            display_adapters: vec![DisplayAdapter {
                pci_address: "0000:01:00.0".to_owned(),
                vendor_id: Some(0x10de),
                device_id: Some(0x1f06),
                class_code: 0x030000,
                driver: Some("nvidia".to_owned()),
                vram_bytes: Some(8 * 1024 * 1024 * 1024),
            }],
            ..Default::default()
        };
        let recommendation = gaming_rig.recommend_profile();
        assert_eq!(recommendation.profile, RecommendedProfile::Cinematic);
        assert_eq!(recommendation.slug, "cinematic");
        assert_eq!(recommendation.name, "Cinematic");
    }

    #[test]
    fn recommends_balanced_for_mid_range_hardware() {
        let office_pc = HardwareInventory {
            memory_bytes: Some(6 * 1024 * 1024 * 1024),
            logical_threads: Some(4),
            storage_devices: vec![StorageDevice {
                name: "sda".to_owned(),
                model: Some("SATA SSD".to_owned()),
                size_bytes: Some(250_000_000_000),
                read_only: Some(false),
                rotational: Some(false),
            }],
            ..Default::default()
        };
        let recommendation = office_pc.recommend_profile();
        assert_eq!(recommendation.profile, RecommendedProfile::Balanced);
        assert_eq!(recommendation.slug, "balanced");
    }
}
