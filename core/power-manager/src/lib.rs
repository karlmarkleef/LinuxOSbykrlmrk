//! Mark Linux power management capabilities, persisted settings and backend actions.

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use zbus::blocking::Connection;

/// Kernel power state file consulted by suspend and hibernate capability checks.
pub const POWER_STATE_PATH: &str = "/sys/power/state";

/// User power preferences persisted by Mark Settings.
pub const POWER_SETTINGS_PATH: &str = "/var/lib/marklinux/power-settings";

/// How the laptop lid should behave when it closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidCloseAction {
    Ignore,
    Sleep,
    Lock,
    PowerOff,
}

/// Physical power button behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerButtonAction {
    PowerOff,
    Sleep,
    Lock,
    Ignore,
}

/// Behaviour when the desktop is idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleAction {
    Nothing,
    Sleep,
    Lock,
}

/// User controllable power preferences managed by Mark Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSettings {
    pub auto_sleep_minutes: u32,
    pub hibernate_enabled: bool,
    pub confirm_shutdown: bool,
    pub confirm_restart: bool,
    pub lock_screen_enabled: bool,
    pub lid_close_action: LidCloseAction,
    pub power_button_action: PowerButtonAction,
    pub idle_action: IdleAction,
}

impl Default for PowerSettings {
    fn default() -> Self {
        Self {
            auto_sleep_minutes: 30,
            hibernate_enabled: true,
            confirm_shutdown: false,
            confirm_restart: false,
            lock_screen_enabled: true,
            lid_close_action: LidCloseAction::Sleep,
            power_button_action: PowerButtonAction::PowerOff,
            idle_action: IdleAction::Sleep,
        }
    }
}

impl PowerSettings {
    /// Read persisted Mark Linux power settings, falling back to defaults.
    pub fn load() -> PowerSettings {
        Self::load_at(Path::new(POWER_SETTINGS_PATH))
    }

    /// Read persisted Mark Linux power settings from an explicit file path.
    pub fn load_at(path: &Path) -> PowerSettings {
        let text = fs::read_to_string(path).unwrap_or_default();
        Self::from_conf_text(&text)
    }

    /// Persist Mark Linux power settings to the default settings path.
    pub fn save(&self) -> io::Result<()> {
        self.save_at(Path::new(POWER_SETTINGS_PATH), self)
    }

    /// Persist Mark Linux power settings to an explicit file path.
    pub fn save_at(path: &Path, settings: &PowerSettings) -> io::Result<()> {
        let mut text = format!("auto-sleep-minutes={}\n", settings.auto_sleep_minutes);
        text.push_str(&format!(
            "hibernate-enabled={}\n",
            settings.hibernate_enabled
        ));
        text.push_str(&format!(
            "confirm-shutdown={}\n",
            settings.confirm_shutdown
        ));
        text.push_str(&format!(
            "confirm-restart={}\n",
            settings.confirm_restart
        ));
        text.push_str(&format!(
            "lock-screen-enabled={}\n",
            settings.lock_screen_enabled
        ));
        text.push_str(&format!(
            "lid-close-action={}\n",
            lid_close_action_name(settings.lid_close_action)
        ));
        text.push_str(&format!(
            "power-button-action={}\n",
            power_button_action_name(settings.power_button_action)
        ));
        text.push_str(&format!(
            "idle-action={}\n",
            idle_action_name(settings.idle_action)
        ));
        fs::write(path, text)
    }

    /// Parse Mark Linux power settings from a key=value document.
    pub fn from_conf_text(text: &str) -> PowerSettings {
        let mut settings = PowerSettings::default();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "auto-sleep-minutes" => {
                    if let Ok(minutes) = value.parse::<u32>() {
                        settings.auto_sleep_minutes = minutes;
                    }
                }
                "hibernate-enabled" => settings.hibernate_enabled = is_true(value),
                "confirm-shutdown" => settings.confirm_shutdown = is_true(value),
                "confirm-restart" => settings.confirm_restart = is_true(value),
                "lock-screen-enabled" => settings.lock_screen_enabled = is_true(value),
                "lid-close-action" => {
                    settings.lid_close_action = parse_lid_close_action(value);
                }
                "power-button-action" => {
                    settings.power_button_action = parse_power_button_action(value);
                }
                "idle-action" => {
                    settings.idle_action = parse_idle_action(value);
                }
                _ => {}
            }
        }
        settings
    }
}

/// Real system power capabilities detected from kernel state and available helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PowerCapabilities {
    pub shutdown: bool,
    pub restart: bool,
    pub suspend: bool,
    pub hibernate: bool,
    pub hybrid_sleep: bool,
    pub lock: bool,
    pub logout: bool,
    pub logind_available: bool,
}

impl PowerCapabilities {
    /// Detect capabilities of the running Mark Linux system.
    pub fn detect() -> Self {
        let state = fs::read_to_string(POWER_STATE_PATH).unwrap_or_default();
        Self::detect_from_state(&state)
    }

    /// Detect capabilities from an explicit kernel power state document.
    pub fn detect_from_state(state: &str) -> Self {
        let suspend = Self::suspend_supported(state);
        let hibernate = Self::hibernate_supported(state, swap_is_available());
        Self {
            shutdown: true,
            restart: true,
            suspend,
            hibernate,
            hybrid_sleep: hibernate,
            lock: command_exists("hyprctl"),
            logout: command_exists("hyprctl"),
            logind_available: Self::logind_available(),
        }
    }

    /// Suspend is supported only when the kernel reports the memory state.
    pub fn suspend_supported(state: &str) -> bool {
        state.trim() == "mem"
    }

    /// Hibernate is supported only with a swap device and the disk state.
    pub fn hibernate_supported(state: &str, swap_available: bool) -> bool {
        state.trim() == "disk" && swap_available
    }

    /// Query systemd-logind availability over the system D-Bus.
    pub fn logind_available() -> bool {
        Connection::system().is_ok()
    }
}

/// Backend that performs real power actions through systemd-logind D-Bus and Hyprland IPC.
pub struct PowerBackend {
    conn: Connection,
}

impl PowerBackend {
    pub fn new() -> Result<Self, zbus::Error> {
        Ok(Self {
            conn: Connection::system()?,
        })
    }

    /// Request a real system shutdown through systemd-logind.
    pub fn shutdown(&self) -> Result<(), zbus::Error> {
        self.call_login1("PowerOff")
    }

    /// Request a real system reboot through systemd-logind.
    pub fn restart(&self) -> Result<(), zbus::Error> {
        self.call_login1("Reboot")
    }

    /// Request suspend through systemd-logind.
    pub fn suspend(&self) -> Result<(), zbus::Error> {
        self.call_login1("Suspend")
    }

    /// Request hibernate through systemd-logind.
    pub fn hibernate(&self) -> Result<(), zbus::Error> {
        self.call_login1("Hibernate")
    }

    /// Lock the desktop session through Hyprland IPC.
    pub fn lock(&self) -> Result<(), String> {
        run_hyprctl(&["lock"])
    }

    /// Unlock the desktop session through Hyprland IPC.
    pub fn unlock(&self) -> Result<(), String> {
        run_hyprctl(&["unlock"])
    }

    /// Log out the desktop session through Hyprland IPC.
    pub fn logout(&self) -> Result<(), String> {
        run_hyprctl(&["exit"])
    }

    fn call_login1(&self, method: &str) -> Result<(), zbus::Error> {
        let proxy = zbus::blocking::Proxy::new(
            &self.conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )?;
        proxy.call(method, &(true,))?;
        Ok(())
    }
}

/// Resolve a user supplied action name such as `shutdown` or `lock`.
pub fn parse_action(name: &str) -> Option<PowerAction> {
    match name.trim().to_ascii_lowercase().as_str() {
        "shutdown" => Some(PowerAction::Shutdown),
        "restart" => Some(PowerAction::Restart),
        "suspend" => Some(PowerAction::Suspend),
        "hibernate" => Some(PowerAction::Hibernate),
        "lock" => Some(PowerAction::Lock),
        "logout" => Some(PowerAction::Logout),
        _ => None,
    }
}

fn is_true(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}

fn parse_lid_close_action(value: &str) -> LidCloseAction {
    match value.to_ascii_lowercase().as_str() {
        "ignore" => LidCloseAction::Ignore,
        "sleep" => LidCloseAction::Sleep,
        "lock" => LidCloseAction::Lock,
        _ => LidCloseAction::Sleep,
    }
}

fn parse_power_button_action(value: &str) -> PowerButtonAction {
    match value.to_ascii_lowercase().as_str() {
        "shutdown" | "power-off" | "poweroff" => PowerButtonAction::PowerOff,
        "sleep" => PowerButtonAction::Sleep,
        "lock" => PowerButtonAction::Lock,
        _ => PowerButtonAction::Ignore,
    }
}

fn parse_idle_action(value: &str) -> IdleAction {
    match value.to_ascii_lowercase().as_str() {
        "nothing" | "none" | "off" => IdleAction::Nothing,
        "sleep" => IdleAction::Sleep,
        "lock" => IdleAction::Lock,
        _ => IdleAction::Sleep,
    }
}

fn lid_close_action_name(action: LidCloseAction) -> &'static str {
    match action {
        LidCloseAction::Ignore => "ignore",
        LidCloseAction::Sleep => "sleep",
        LidCloseAction::Lock => "lock",
        LidCloseAction::PowerOff => "power-off",
    }
}

fn power_button_action_name(action: PowerButtonAction) -> &'static str {
    match action {
        PowerButtonAction::PowerOff => "shutdown",
        PowerButtonAction::Sleep => "sleep",
        PowerButtonAction::Lock => "lock",
        PowerButtonAction::Ignore => "ignore",
    }
}

fn idle_action_name(action: IdleAction) -> &'static str {
    match action {
        IdleAction::Nothing => "nothing",
        IdleAction::Sleep => "sleep",
        IdleAction::Lock => "lock",
    }
}

fn swap_is_available() -> bool {
    fs::read_to_string("/proc/swaps")
        .map(|text| text.lines().skip(1).any(|line| !line.trim().is_empty()))
        .unwrap_or(false)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} >/dev/null 2>&1", cmd))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_hyprctl(args: &[&str]) -> Result<(), String> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(|error| format!("failed to execute hyprctl: {}", error))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let detail = if stderr.is_empty() {
            format!("hyprctl {} failed", args.join(" "))
        } else {
            stderr
        };
        return Err(detail);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        IdleAction, LidCloseAction, PowerAction, PowerButtonAction, PowerCapabilities, PowerSettings,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mark-power-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn suspend_requires_memory_state() {
        assert!(PowerCapabilities::suspend_supported("mem"));
        assert!(!PowerCapabilities::suspend_supported("disk"));
        assert!(!PowerCapabilities::suspend_supported(""));
    }

    #[test]
    fn hibernate_requires_disk_state_and_swap() {
        assert!(PowerCapabilities::hibernate_supported("disk", true));
        assert!(!PowerCapabilities::hibernate_supported("mem", true));
        assert!(!PowerCapabilities::hibernate_supported("disk", false));
        assert!(!PowerCapabilities::hibernate_supported("", true));
    }

    #[test]
    fn default_power_settings_are_valid() {
        let settings = PowerSettings::default();
        assert_eq!(settings.auto_sleep_minutes, 30);
        assert!(settings.hibernate_enabled);
        assert!(!settings.confirm_shutdown);
        assert!(!settings.confirm_restart);
        assert!(settings.lock_screen_enabled);
        assert_eq!(settings.lid_close_action, LidCloseAction::Sleep);
        assert_eq!(settings.power_button_action, PowerButtonAction::PowerOff);
        assert_eq!(settings.idle_action, IdleAction::Sleep);
    }

    #[test]
    fn parses_mark_linux_power_settings() {
        let settings = PowerSettings::from_conf_text(
            "# Mark Linux power settings\n\nauto-sleep-minutes=45\nhibernate-enabled=false\nlid-close-action=lock\n",
        );
        assert_eq!(settings.auto_sleep_minutes, 45);
        assert!(!settings.hibernate_enabled);
        assert_eq!(settings.lid_close_action, LidCloseAction::Lock);
        assert_eq!(settings.power_button_action, PowerButtonAction::PowerOff);
    }

    #[test]
    fn saves_and_loads_power_settings_roundtrip() {
        let settings = PowerSettings::from_conf_text("auto-sleep-minutes=60\nidle-action=nothing\n");
        let temp = TempDir::new();
        let path = temp.path().join("power-settings");
        settings.save_at(&path, &settings).unwrap();
        let loaded = PowerSettings::load_at(&path);
        assert_eq!(loaded, settings);
        assert_eq!(loaded.auto_sleep_minutes, 60);
        assert_eq!(loaded.idle_action, IdleAction::Nothing);
    }

    #[test]
    fn unknown_setting_keys_are_ignored() {
        let settings = PowerSettings::from_conf_text("unknown-key=whatever\nauto-sleep-minutes=not-a-number\n");
        assert_eq!(settings.auto_sleep_minutes, 30);
    }

    #[test]
    fn power_action_variants_are_distinct() {
        assert_ne!(PowerAction::Shutdown, PowerAction::Restart);
        assert_ne!(PowerAction::Suspend, PowerAction::Hibernate);
        assert_ne!(PowerAction::Lock, PowerAction::Logout);
    }

    #[test]
    fn parses_known_action_names() {
        assert_eq!(super::parse_action("Shutdown"), Some(PowerAction::Shutdown));
        assert_eq!(super::parse_action("  restart "), Some(PowerAction::Restart));
        assert_eq!(super::parse_action("suspend"), Some(PowerAction::Suspend));
        assert_eq!(super::parse_action("hibernate"), Some(PowerAction::Hibernate));
        assert_eq!(super::parse_action("lock"), Some(PowerAction::Lock));
        assert_eq!(super::parse_action("logout"), Some(PowerAction::Logout));
        assert_eq!(super::parse_action("bogus"), None);
    }
}
