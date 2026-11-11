//! Real Pacman-backed package manager backend for the Mark Store frontend.

use std::process::Command;

/// Interface for executing package-manager commands. Kept as a trait so unit
/// tests can run against fake pacman outputs without touching a real database.
pub trait PacmanRunner {
    /// Run a pacman command and return its standard output.
    fn run(&self, args: &[&str]) -> Result<String, String>;
}

/// Default runner that executes the real pacman binary.
pub struct RealRunner;

impl PacmanRunner for RealRunner {
    fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("pacman")
            .args(args)
            .output()
            .map_err(|error| format!("failed to execute pacman: {}", error))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let detail = if stderr.is_empty() {
                format!("pacman {} failed", args.join(" "))
            } else {
                stderr
            };
            return Err(detail);
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Result of a package metadata lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPackage {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub installed_size: Option<u64>,
}

/// Pacman backend used by Mark Store for browsing, installing and removing packages.
pub struct PacmanBackend {
    runner: Box<dyn PacmanRunner>,
}

impl PacmanBackend {
    pub fn new() -> Self {
        Self {
            runner: Box::new(RealRunner),
        }
    }

    pub fn with_runner(runner: Box<dyn PacmanRunner>) -> Self {
        Self { runner }
    }

    /// Search the configured repositories for packages matching a pattern.
    pub fn search(&self, pattern: &str) -> Result<Vec<String>, String> {
        let output = self.runner.run(&["-Ss", pattern])?;
        Ok(parse_search_output(&output))
    }

    /// List installed package names.
    pub fn installed(&self) -> Result<Vec<String>, String> {
        let output = self.runner.run(&["-Qe"])?;
        Ok(parse_search_output(&output))
    }

    /// Fetch detailed metadata for one package.
    pub fn info(&self, name: &str) -> Result<ParsedPackage, String> {
        let output = self.runner.run(&["-Qi", name])?;
        Ok(parse_package_info(&output, name))
    }

    /// Install packages through pacman. The caller is responsible for user
    /// confirmation and privilege handling before invoking this backend.
    pub fn install(&self, names: &[String]) -> Result<String, String> {
        let mut args: Vec<&str> = vec!["-S"];
        args.extend(names.iter().map(|name| name.as_str()));
        self.runner.run(&args)
    }

    /// Remove packages through pacman. The caller is responsible for user
    /// confirmation and privilege handling before invoking this backend.
    pub fn remove(&self, names: &[String]) -> Result<String, String> {
        let mut args: Vec<&str> = vec!["-R"];
        args.extend(names.iter().map(|name| name.as_str()));
        self.runner.run(&args)
    }
}

/// Extract package names from pacman search/list output.
pub fn parse_search_output(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Parse package metadata from `pacman -Qi` output.
pub fn parse_package_info(output: &str, name: &str) -> ParsedPackage {
    let mut version = None;
    let mut description = None;
    let mut installed_size = None;
    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if key == "version" {
            version = Some(value.to_owned());
        } else if key == "description" {
            description = Some(value.to_owned());
        } else if key == "installed size" {
            installed_size = value.parse::<u64>().ok();
        }
    }
    ParsedPackage {
        name: name.to_owned(),
        version,
        description,
        installed_size,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_package_info, parse_search_output, PacmanBackend, PacmanRunner};

    struct FakeRunner {
        output: String,
        fail: bool,
    }

    impl PacmanRunner for FakeRunner {
        fn run(&self, _args: &[&str]) -> Result<String, String> {
            if self.fail {
                return Err("pacman database unavailable".to_owned());
            }
            Ok(self.output.clone())
        }
    }

    #[test]
    fn parses_search_output_names() {
        let names = parse_search_output("linux\nfirefox\n");
        assert_eq!(names, vec!["linux".to_owned(), "firefox".to_owned()]);
        let empty = parse_search_output("# comment\n\n");
        assert!(empty.is_empty());
    }

    #[test]
    fn parses_package_info_fields() {
        let info = parse_package_info(
            "Name        : firefox\nVersion     : 130.0-1\nDescription : Web browser\nInstalled Size: 120 MB\n",
            "firefox",
        );
        assert_eq!(info.name, "firefox");
        assert_eq!(info.version.as_deref(), Some("130.0-1"));
        assert_eq!(info.description.as_deref(), Some("Web browser"));
        assert_eq!(info.installed_size, Some(120));
    }

    #[test]
    fn install_propagates_runner_failures() {
        let backend = PacmanBackend::with_runner(Box::new(FakeRunner {
            output: String::new(),
            fail: true,
        }));
        assert!(backend.install(&["firefox".to_owned()]).is_err());
    }

    #[test]
    fn search_uses_runner_output() {
        let backend = PacmanBackend::with_runner(Box::new(FakeRunner {
            output: "firefox\n".to_owned(),
            fail: false,
        }));
        assert_eq!(
            backend.search("fire").unwrap(),
            vec!["firefox".to_owned()]
        );
    }
}
