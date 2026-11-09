//! PrefixManager — create, list, load, and delete Wine prefixes on disk.

use std::{
    fs,
    io,
    path::PathBuf,
};

use crate::prefix::PrefixConfig;

/// Returns the base directory where all MCP prefixes are stored.
/// Default: `~/.local/share/marklinux/mcp/prefixes`
pub fn base_dir() -> PathBuf {
    // Prefer XDG_DATA_HOME if set
    let data_home = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            PathBuf::from(home).join(".local").join("share")
        });
    data_home.join("marklinux").join("mcp").join("prefixes")
}

/// Returns the path to a specific prefix directory.
pub fn prefix_dir(name: &str) -> PathBuf {
    base_dir().join(name)
}

/// Returns the path to the `pfx/` subdirectory (the actual WINEPREFIX).
pub fn prefix_path(name: &str) -> PathBuf {
    prefix_dir(name).join("pfx")
}

/// Returns the path to the `prefix.toml` metadata file.
fn config_path(name: &str) -> PathBuf {
    prefix_dir(name).join("prefix.toml")
}

/// Create a new prefix on disk: writes directory tree and `prefix.toml`.
///
/// Does NOT initialise the Wine WINEPREFIX — that happens on first run.
///
/// # Errors
/// Returns `Err` if the prefix already exists or if filesystem I/O fails.
pub fn create(config: &PrefixConfig) -> io::Result<PathBuf> {
    let dir = prefix_dir(&config.name);
    if dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Prefix '{}' already exists at {}", config.name, dir.display()),
        ));
    }

    // Create prefix directory and pfx/ subdirectory
    fs::create_dir_all(prefix_path(&config.name))?;

    // Serialise config to TOML
    let toml_str = toml::to_string_pretty(config).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("TOML serialization error: {e}"))
    })?;

    fs::write(config_path(&config.name), toml_str)?;

    Ok(dir)
}

/// Load a single prefix configuration from disk.
pub fn load(name: &str) -> io::Result<PrefixConfig> {
    let path = config_path(name);
    let raw = fs::read_to_string(&path).map_err(|e| {
        io::Error::new(e.kind(), format!("Cannot read {}: {e}", path.display()))
    })?;
    toml::from_str(&raw).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid prefix.toml for '{}': {e}", name),
        )
    })
}

/// List all prefixes found in the base directory.
/// Non-parsable directories are silently skipped (not all subdirs need to be prefixes).
pub fn list() -> io::Result<Vec<PrefixConfig>> {
    let base = base_dir();
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut prefixes = Vec::new();
    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        match load(&name) {
            Ok(cfg)  => prefixes.push(cfg),
            Err(_)   => { /* skip directories that aren't valid prefixes */ }
        }
    }

    prefixes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(prefixes)
}

/// Delete a prefix and all its contents.
pub fn delete(name: &str) -> io::Result<()> {
    let dir = prefix_dir(name);
    if !dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Prefix '{}' not found at {}", name, dir.display()),
        ));
    }
    fs::remove_dir_all(dir)
}

/// Update the runtime field of an existing prefix.
pub fn set_runtime(name: &str, runtime: crate::prefix::Runtime) -> io::Result<()> {
    let mut cfg = load(name)?;
    cfg.runtime = runtime;

    let toml_str = toml::to_string_pretty(&cfg).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("TOML serialization error: {e}"))
    })?;
    fs::write(config_path(name), toml_str)
}

// ── unit tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefix::{PrefixArch, PrefixConfig, Runtime};
    use std::env;

    /// Override XDG_DATA_HOME so tests write to a temp directory.
    fn with_tempdir<F: FnOnce()>(f: F) {
        let tmp = std::env::temp_dir().join(format!(
            "mcp_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        // Set env var so base_dir() returns inside temp dir
        env::set_var("XDG_DATA_HOME", tmp.to_str().unwrap());
        f();
        // Cleanup
        let _ = fs::remove_dir_all(&tmp);
        env::remove_var("XDG_DATA_HOME");
    }

    fn make_config(name: &str) -> PrefixConfig {
        PrefixConfig {
            name:        name.into(),
            arch:        PrefixArch::X86_64,
            runtime:     Runtime::Wine,
            created_at:  "2026-11-09T00:00:00Z".into(),
            description: None,
            app_exe:     None,
        }
    }

    #[test]
    fn test_create_and_load() {
        with_tempdir(|| {
            let cfg = make_config("myapp");
            let dir = create(&cfg).expect("create");
            assert!(dir.exists());
            assert!(config_path("myapp").exists());
            assert!(prefix_path("myapp").exists());

            let loaded = load("myapp").expect("load");
            assert_eq!(loaded.name, "myapp");
            assert_eq!(loaded.arch, PrefixArch::X86_64);
        });
    }

    #[test]
    fn test_create_duplicate_returns_error() {
        with_tempdir(|| {
            let cfg = make_config("dup");
            create(&cfg).expect("first create");
            let result = create(&cfg);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        });
    }

    #[test]
    fn test_list_multiple_prefixes() {
        with_tempdir(|| {
            for name in &["alpha", "beta", "gamma"] {
                create(&make_config(name)).expect("create");
            }
            let prefixes = list().expect("list");
            assert_eq!(prefixes.len(), 3);
            // Should be sorted alphabetically
            let names: Vec<&str> = prefixes.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["alpha", "beta", "gamma"]);
        });
    }

    #[test]
    fn test_delete() {
        with_tempdir(|| {
            create(&make_config("todelete")).expect("create");
            delete("todelete").expect("delete");
            assert!(!prefix_dir("todelete").exists());
        });
    }

    #[test]
    fn test_delete_nonexistent_returns_error() {
        with_tempdir(|| {
            let result = delete("doesnotexist");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
        });
    }

    #[test]
    fn test_set_runtime() {
        with_tempdir(|| {
            create(&make_config("changeruntime")).expect("create");
            set_runtime("changeruntime", Runtime::WineStaging).expect("set_runtime");
            let cfg = load("changeruntime").expect("load");
            assert_eq!(cfg.runtime, Runtime::WineStaging);
        });
    }

    #[test]
    fn test_list_empty_base_dir() {
        with_tempdir(|| {
            let result = list().expect("list on empty dir");
            assert!(result.is_empty());
        });
    }
}
