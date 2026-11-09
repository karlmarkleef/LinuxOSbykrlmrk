//! mark-runtime - Windows runtime manager for the Mark Compatibility Platform
//!
//! Usage:
//!   mark-runtime            Print a human-readable runtime status report
//!   mark-runtime --json     Print a JSON runtime status report
//!
//! Detection only. This tool never installs or launches runtimes; it reports
//! which upstream Wine/Proton compatible runtimes are present on the host.

mod runtime;

use std::env;

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

fn render_json(kinds: &[runtime::RuntimeKind]) -> String {
    let mut out = String::from("{\n  \"runtimes\": [\n");
    for (i, kind) in kinds.iter().enumerate() {
        let status = runtime::detect(*kind);
        let version = status
            .version
            .as_deref()
            .map(|v| format!("\"{}\"", escape_json_string(v)))
            .unwrap_or_else(|| "null".to_string());
        out.push_str(&format!(
            "    {{\n      \"name\": \"{}\",\n      \"label\": \"{}\",\n      \"installed\": {},\n      \"version\": {}\n    }}",
            kind.id(),
            kind.label(),
            status.installed,
            version
        ));
        if i + 1 < kinds.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn print_json() {
    let kinds = [
        runtime::RuntimeKind::Wine,
        runtime::RuntimeKind::WineStaging,
        runtime::RuntimeKind::Proton,
        runtime::RuntimeKind::Bottles,
        runtime::RuntimeKind::Winetricks,
    ];
    print!("{}", render_json(&kinds));
}

fn print_text() {
    let kinds = [
        runtime::RuntimeKind::Wine,
        runtime::RuntimeKind::WineStaging,
        runtime::RuntimeKind::Proton,
        runtime::RuntimeKind::Bottles,
        runtime::RuntimeKind::Winetricks,
    ];
    println!("Mark Compatibility Platform - Runtime status");
    for kind in kinds {
        let status = runtime::detect(kind);
        let state = if status.installed { "installed" } else { "not installed" };
        let version = status.version.as_deref().unwrap_or("-");
        println!("{}: {} ({})", kind.id(), state, version);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg.as_str() == "--json") {
        print_json();
    } else {
        print_text();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{self, RuntimeKind};

    #[test]
    fn test_escape_json_string_quotes() {
        assert_eq!(escape_json_string("say \"hi\""), "say \\\"hi\\\"");
    }

    #[test]
    fn test_render_json_contains_all_runtime_ids() {
        let kinds = [
            runtime::RuntimeKind::Wine,
            runtime::RuntimeKind::WineStaging,
            runtime::RuntimeKind::Proton,
            runtime::RuntimeKind::Bottles,
            runtime::RuntimeKind::Winetricks,
        ];
        let rendered = render_json(&kinds);
        assert!(rendered.contains("\"name\": \"wine\""));
        assert!(rendered.contains("\"name\": \"wine_staging\""));
        assert!(rendered.contains("\"name\": \"proton\""));
        assert!(rendered.contains("\"name\": \"bottles\""));
        assert!(rendered.contains("\"name\": \"winetricks\""));
        assert!(rendered.contains("\"installed\": false"));
        assert!(rendered.trim_end().ends_with('}'));
    }
}
