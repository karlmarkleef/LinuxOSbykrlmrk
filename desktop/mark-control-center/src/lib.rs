//! Real control center backend for quick status and quick actions.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Interface for executing control backend commands.
pub trait ControlRunner {
    /// Run a command and return its standard output.
    fn run(&self, args: &[&str]) -> Result<String, String>;
}

/// Default runner that executes real system commands.
pub struct RealRunner;

impl ControlRunner for RealRunner {
    fn run(&self, args: &[&str]) -> Result<String, String> {
        if args.is_empty() {
            return Err("empty command".to_owned());
        }
        let output = Command::new(args[0])
            .args(&args[1..])
            .output()
            .map_err(|error| format!("failed to execute {}: {}", args[0], error))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let detail = if stderr.is_empty() {
                format!("{} {} failed", args[0], args[1..].join(" "))
            } else {
                stderr
            };
            return Err(detail);
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Quick network status reported by the control center.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkStatus {
    pub wifi_enabled: bool,
}

/// Quick audio status reported by the control center.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AudioStatus {
    pub volume_percent: Option<u8>,
    pub muted: bool,
}

/// Quick display brightness status reported by the control center.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrightnessStatus {
    pub percent: Option<u8>,
}

/// Battery information reported by the control center.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BatteryStatus {
    pub present: bool,
    pub percent: Option<u8>,
    pub charging: bool,
}

/// Control center backend backed by real system commands.
pub struct ControlBackend {
    runner: Box<dyn ControlRunner>,
}

impl ControlBackend {
    pub fn new() -> Self {
        Self {
            runner: Box::new(RealRunner),
        }
    }

    pub fn with_runner(runner: Box<dyn ControlRunner>) -> Self {
        Self { runner }
    }

    /// Query whether Wi-Fi is enabled through NetworkManager.
    pub fn wifi_enabled(&self) -> Result<bool, String> {
        let output = self.runner.run(&["nmcli", "radio", "wifi"])?;
        Ok(parse_bool(&output.trim()))
    }

    /// Enable or disable Wi-Fi through NetworkManager.
    pub fn set_wifi(&self, enabled: bool) -> Result<String, String> {
        let state = if enabled { "on" } else { "off" };
        self.runner.run(&["nmcli", "radio", "wifi", state])
    }

    /// Query whether Bluetooth is powered through bluetoothctl.
    pub fn bluetooth_enabled(&self) -> Result<bool, String> {
        let output = self.runner.run(&["bluetoothctl", "show"])?;
        Ok(bluetoothctl_powered(&output))
    }

    /// Enable or disable Bluetooth through bluetoothctl.
    pub fn set_bluetooth(&self, enabled: bool) -> Result<String, String> {
        let state = if enabled { "on" } else { "off" };
        self.runner.run(&["bluetoothctl", "power", state])
    }

    /// Read current audio volume and mute state through PipeWire/PulseAudio.
    pub fn audio_status(&self) -> Result<AudioStatus, String> {
        let volume = self.runner.run(&["pactl", "get-sink-volume", "@DEFAULT_SINK"]);
        let muted = self.runner.run(&["pactl", "get-mute", "@DEFAULT_SINK"]);
        Ok(AudioStatus {
            volume_percent: volume.ok().and_then(|output| parse_percentage(&output)),
            muted: muted.map(|output| parse_bool(&output)).unwrap_or(false),
        })
    }

    /// Set the audio sink volume through PipeWire/PulseAudio.
    pub fn set_volume(&self, percent: u8) -> Result<String, String> {
        self.runner.run(&[
            "pactl",
            "set-sink-volume",
            "@DEFAULT_SINK",
            &percent.to_string(),
        ])
    }

    /// Read current brightness through brightnessctl.
    pub fn brightness_status(&self) -> Result<BrightnessStatus, String> {
        let output = self.runner.run(&["brightnessctl", "info"])?;
        Ok(BrightnessStatus {
            percent: parse_current_brightness(&output),
        })
    }

    /// Set brightness through brightnessctl.
    pub fn set_brightness(&self, percent: u8) -> Result<String, String> {
        self.runner.run(&["brightnessctl", "set", &percent.to_string()])
    }

    /// Read battery status from the kernel power supply hierarchy.
    pub fn battery_status(&self) -> BatteryStatus {
        battery_status()
    }
}

/// Read battery percentage and charging state from the kernel power supply paths.
pub fn battery_status() -> BatteryStatus {
    let capacity = fs::read_to_string(Path::new("/sys/class/power_supply/BAT0/capacity"))
        .ok()
        .and_then(|text| parse_percentage(&text));
    let charging = fs::read_to_string(Path::new("/sys/class/power_supply/BAT0/status"))
        .map(|text| parse_bool(&text))
        .unwrap_or(false);
    BatteryStatus {
        present: capacity.is_some(),
        percent: capacity,
        charging,
    }
}

fn parse_bool(text: &str) -> bool {
    text.trim().to_ascii_lowercase() == "yes"
}

/// Parse a percentage value such as `45%` or `45.00%` into a whole number.
pub fn parse_percentage(text: &str) -> Option<u8> {
    let numeric = text.trim().trim_end_matches('%');
    let value = numeric.parse::<f64>().ok()?;
    u8::try_from(value.round()).ok()
}

/// Extract the current brightness percentage from `brightnessctl info` output.
pub fn parse_current_brightness(output: &str) -> Option<u8> {
    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().to_ascii_lowercase() == "current brightness" {
                return value.trim().parse::<f64>().ok().and_then(|v| u8::try_from(v.round()).ok());
            }
        }
    }
    None
}

/// Parse the `Powered:` line of `bluetoothctl show` output.
pub fn bluetoothctl_powered(output: &str) -> bool {
    output.lines().any(|line| {
        let trimmed = line.trim().to_ascii_lowercase();
        trimmed.starts_with("powered:") && trimmed.contains("yes")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        battery_status, bluetoothctl_powered, parse_bool, parse_current_brightness, parse_percentage,
        BatteryStatus, ControlBackend, ControlRunner,
    };
    use std::cell::RefCell;

    struct FakeRunner {
        output: String,
        fail: bool,
        last_args: RefCell<Vec<String>>,
    }

    impl ControlRunner for FakeRunner {
        fn run(&self, args: &[&str]) -> Result<String, String> {
            *self.last_args.borrow_mut() = args.iter().map(|arg| arg.to_string()).collect();
            if self.fail {
                return Err("control runner failed".to_owned());
            }
            Ok(self.output.clone())
        }
    }

    fn fake_runner(output: &str) -> FakeRunner {
        FakeRunner {
            output: output.to_owned(),
            fail: false,
            last_args: RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn parses_percentage_values() {
        assert_eq!(parse_percentage("45%"), Some(45));
        assert_eq!(parse_percentage("45.00%"), Some(45));
        assert_eq!(parse_percentage("n/a"), None);
    }

    #[test]
    fn parses_bool_values() {
        assert!(parse_bool("yes"));
        assert!(!parse_bool("no"));
    }

    #[test]
    fn parses_bluetoothctl_power_state() {
        assert!(bluetoothctl_powered("  Powered: yes\n"));
        assert!(!bluetoothctl_powered("  Powered: no\n"));
    }

    #[test]
    fn parses_current_brightness() {
        assert_eq!(parse_current_brightness("  Current brightness: 77\n"), Some(77));
        assert_eq!(parse_current_brightness("  Current brightness: 0\n"), Some(0));
        assert_eq!(parse_current_brightness("  Error: no device"), None);
    }

    #[test]
    fn reads_battery_status_from_kernel_paths() {
        let status: BatteryStatus = battery_status();
        assert!(!status.present);
    }

    #[test]
    fn control_backend_queries_wifi_enabled() {
        let backend = ControlBackend::with_runner(Box::new(fake_runner("enabled\n")));
        assert_eq!(backend.wifi_enabled().unwrap(), true);
    }

    #[test]
    fn control_backend_sets_wifi_with_nmcli_args() {
        let fake = FakeRunner {
            output: String::new(),
            fail: false,
            last_args: RefCell::new(Vec::new()),
        };
        let backend = ControlBackend::with_runner(Box::new(fake));
        backend.set_wifi(false).unwrap();
        assert_eq!(
            *fake.last_args.borrow(),
            vec![
                "nmcli".to_string(),
                "radio".to_string(),
                "wifi".to_string(),
                "off".to_string(),
            ]
        );
    }

    #[test]
    fn control_backend_propagates_runner_failures() {
        let backend = ControlBackend::with_runner(Box::new(FakeRunner {
            output: String::new(),
            fail: true,
            last_args: RefCell::new(Vec::new()),
        }));
        assert!(backend.wifi_enabled().is_err());
    }
}
