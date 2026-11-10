//! Runtime detection primitives for the Mark Compatibility Platform.
//!
//! Mark Linux reuses upstream Wine/Proton based runtimes instead of shipping a
//! proprietary Windows API implementation. The runtime manager answers one
//! question: which Wine-compatible runtimes are installed on this host, and
//! with which versions.

use std::path::PathBuf;

/// A Windows compatibility runtime managed by Mark Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeKind {
    Wine,
    WineStaging,
    Proton,
    Bottles,
    Winetricks,
}

impl RuntimeKind {
    /// Stable lowercase identifier used in reports and JSON keys.
    pub fn id(&self) -> &'static str {
        match self {
            RuntimeKind::Wine => "wine",
            RuntimeKind::WineStaging => "wine_staging",
            RuntimeKind::Proton => "proton",
            RuntimeKind::Bottles => "bottles",
            RuntimeKind::Winetricks => "winetricks",
        }
    }

    /// Binary name resolved on `PATH` for this runtime.
    pub fn binary_name(&self) -> &'static str {
        match self {
            RuntimeKind::Wine => "wine",
            RuntimeKind::WineStaging => "wine-staging",
            RuntimeKind::Proton => "proton",
            RuntimeKind::Bottles => "bottles",
            RuntimeKind::Winetricks => "winetricks",
        }
    }

    /// Human-readable runtime name used in reports.
    pub fn label(&self) -> &'static str {
        match self {
            RuntimeKind::Wine => "Wine",
            RuntimeKind::WineStaging => "Wine Staging",
            RuntimeKind::Proton => "Proton",
            RuntimeKind::Bottles => "Bottles",
            RuntimeKind::Winetricks => "Winetricks",
        }
    }
}

/// Detection result for a single runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub kind: RuntimeKind,
    pub installed: bool,
    pub version: Option<String>,
}

/// Search the current `PATH` and return the first matching executable.
fn find_binary(kind: RuntimeKind) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };
    for dir in path_var.split(separator) {
        if dir.is_empty() {
            continue;
        }
        let candidate = std::path::PathBuf::from(dir).join(kind.binary_name());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Query `<path> --version` and return the first trimmed output line.
fn version_of(path: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let first = stdout
        .lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())?;
    Some(first.to_string())
}

/// Probe a single runtime and report installation state plus version.
pub fn detect(kind: RuntimeKind) -> RuntimeStatus {
    let installed = find_binary(kind).is_some();
    let version = if installed {
        find_binary(kind).and_then(|path| version_of(&path))
    } else {
        None
    };
    RuntimeStatus {
        kind,
        installed,
        version,
    }
}

/// Probe every runtime managed by the Mark Compatibility Platform.
pub fn all() -> Vec<RuntimeStatus> {
    vec![
        detect(RuntimeKind::Wine),
        detect(RuntimeKind::WineStaging),
        detect(RuntimeKind::Proton),
        detect(RuntimeKind::Bottles),
        detect(RuntimeKind::Winetricks),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_kind_ids() {
        assert_eq!(RuntimeKind::Wine.id(), "wine");
        assert_eq!(RuntimeKind::WineStaging.id(), "wine_staging");
        assert_eq!(RuntimeKind::Proton.id(), "proton");
        assert_eq!(RuntimeKind::Bottles.id(), "bottles");
        assert_eq!(RuntimeKind::Winetricks.id(), "winetricks");
    }

    #[test]
    fn test_runtime_kind_binary_names() {
        assert_eq!(RuntimeKind::Wine.binary_name(), "wine");
        assert_eq!(RuntimeKind::WineStaging.binary_name(), "wine-staging");
        assert_eq!(RuntimeKind::Proton.binary_name(), "proton");
        assert_eq!(RuntimeKind::Bottles.binary_name(), "bottles");
        assert_eq!(RuntimeKind::Winetricks.binary_name(), "winetricks");
    }

    #[test]
    fn test_runtime_kind_labels() {
        assert_eq!(RuntimeKind::Wine.label(), "Wine");
        assert_eq!(RuntimeKind::WineStaging.label(), "Wine Staging");
        assert_eq!(RuntimeKind::Proton.label(), "Proton");
        assert_eq!(RuntimeKind::Bottles.label(), "Bottles");
        assert_eq!(RuntimeKind::Winetricks.label(), "Winetricks");
    }

    #[test]
    fn test_runtime_status_shape() {
        let status = RuntimeStatus {
            kind: RuntimeKind::Wine,
            installed: true,
            version: Some("wine-8.0".to_string()),
        };
        assert_eq!(status.kind, RuntimeKind::Wine);
        assert!(status.installed);
        assert_eq!(status.version.as_deref(), Some("wine-8.0"));

        let absent = RuntimeStatus {
            kind: RuntimeKind::Proton,
            installed: false,
            version: None,
        };
        assert!(!absent.installed);
        assert_eq!(absent.version, None);
    }

    #[test]
    fn test_detect_reports_valid_status_shape() {
        // Probing an arbitrary runtime kind must return a well-formed status
        // without panicking, regardless of whether the binary is installed.
        let status = detect(RuntimeKind::Winetricks);
        assert_eq!(status.kind, RuntimeKind::Winetricks);
        // `detect` sets `installed` from binary presence and only then queries
        // the version, so the two fields must stay consistent.
        assert_eq!(status.version.is_some(), status.installed);
    }
}
