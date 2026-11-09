//! Prefix configuration types — shared between manager, runner, and CLI.

use serde::{Deserialize, Serialize};

/// Architecture for the Wine prefix / WINEARCH.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrefixArch {
    X86_64,
    X86_32,
}

impl PrefixArch {
    /// Return the WINEARCH env-var value Wine expects.
    pub fn winearch_str(&self) -> &'static str {
        match self {
            PrefixArch::X86_64 => "win64",
            PrefixArch::X86_32 => "win32",
        }
    }
}

impl std::fmt::Display for PrefixArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefixArch::X86_64 => write!(f, "x86_64"),
            PrefixArch::X86_32 => write!(f, "x86_32"),
        }
    }
}

impl std::str::FromStr for PrefixArch {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64" | "win64" | "64" => Ok(PrefixArch::X86_64),
            "x86_32" | "win32" | "32" => Ok(PrefixArch::X86_32),
            other => Err(format!("Unknown arch: {other}; expected x86_64 or x86_32")),
        }
    }
}

/// Which Wine-compatible runtime to use when launching executables.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Runtime {
    Wine,
    Proton,
    WineStaging,
}

impl std::fmt::Display for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Runtime::Wine        => write!(f, "wine"),
            Runtime::Proton      => write!(f, "proton"),
            Runtime::WineStaging => write!(f, "wine-staging"),
        }
    }
}

impl std::str::FromStr for Runtime {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wine"         => Ok(Runtime::Wine),
            "proton"       => Ok(Runtime::Proton),
            "wine-staging" | "wine_staging" => Ok(Runtime::WineStaging),
            other => Err(format!("Unknown runtime: {other}; expected wine, proton, or wine-staging")),
        }
    }
}

/// Persistent metadata for a single Wine prefix managed by MCP.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefixConfig {
    /// Short alphanumeric name used as the directory name.
    pub name: String,
    /// Target architecture for this prefix.
    pub arch: PrefixArch,
    /// Runtime used to launch executables in this prefix.
    pub runtime: Runtime,
    /// ISO 8601 creation timestamp (UTC).
    pub created_at: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Default executable to launch with `mark-prefix run <name>` (no `-- exe`).
    pub app_exe: Option<String>,
}

// ── unit tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> PrefixConfig {
        PrefixConfig {
            name:        "test-app".into(),
            arch:        PrefixArch::X86_64,
            runtime:     Runtime::WineStaging,
            created_at:  "2026-11-09T00:00:00Z".into(),
            description: Some("Test prefix".into()),
            app_exe:     Some("/path/to/app.exe".into()),
        }
    }

    #[test]
    fn test_toml_roundtrip() {
        let original = sample_config();
        let serialized   = toml::to_string(&original).expect("serialize");
        let deserialized: PrefixConfig = toml::from_str(&serialized).expect("deserialize");

        assert_eq!(deserialized.name,       original.name);
        assert_eq!(deserialized.arch,       original.arch);
        assert_eq!(deserialized.runtime,    original.runtime);
        assert_eq!(deserialized.created_at, original.created_at);
        assert_eq!(deserialized.description, original.description);
        assert_eq!(deserialized.app_exe,    original.app_exe);
    }

    #[test]
    fn test_prefix_arch_display_and_parse() {
        assert_eq!(PrefixArch::X86_64.to_string(), "x86_64");
        assert_eq!(PrefixArch::X86_32.to_string(), "x86_32");

        let a: PrefixArch = "x86_64".parse().unwrap();
        assert_eq!(a, PrefixArch::X86_64);
        let b: PrefixArch = "win32".parse().unwrap();
        assert_eq!(b, PrefixArch::X86_32);
        assert!("bad".parse::<PrefixArch>().is_err());
    }

    #[test]
    fn test_runtime_display_and_parse() {
        assert_eq!(Runtime::Wine.to_string(),        "wine");
        assert_eq!(Runtime::Proton.to_string(),      "proton");
        assert_eq!(Runtime::WineStaging.to_string(), "wine-staging");

        let r: Runtime = "wine-staging".parse().unwrap();
        assert_eq!(r, Runtime::WineStaging);
        assert!("bad".parse::<Runtime>().is_err());
    }

    #[test]
    fn test_winearch_str() {
        assert_eq!(PrefixArch::X86_64.winearch_str(), "win64");
        assert_eq!(PrefixArch::X86_32.winearch_str(), "win32");
    }

    #[test]
    fn test_toml_deserialize_minimal() {
        let toml_str = r#"
            name       = "minimal"
            arch       = "x86_64"
            runtime    = "wine"
            created_at = "2026-01-01T00:00:00Z"
        "#;
        let cfg: PrefixConfig = toml::from_str(toml_str).expect("deserialize minimal");
        assert_eq!(cfg.name, "minimal");
        assert_eq!(cfg.arch, PrefixArch::X86_64);
        assert!(cfg.description.is_none());
        assert!(cfg.app_exe.is_none());
    }
}
