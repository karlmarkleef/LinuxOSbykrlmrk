//! mark-apk-inspect - CLI entry point
//!
//! Usage: mark-apk-inspect <path-to-apk>
//!
//! Reads the APK ZIP central directory and prints a JSON object describing
//! the native ABI coverage and ARM translation requirements.

mod apk;

use std::env;

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
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

fn render_supported_abis(abis: &[String]) -> String {
    if abis.is_empty() {
        return "[]".to_string();
    }
    let items: Vec<String> = abis
        .iter()
        .map(|abi| format!("    \"{}\"", escape_json_string(abi)))
        .collect();
    format!("[\n{}\n  ]", items.join(",\n"))
}

fn render_apk_json(path: &str, info: &apk::ApkInfo) -> String {
    let supported = render_supported_abis(&info.supported_abis);
    format!(
        "{{\n  \"path\": \"{}\",\n  \"valid_apk\": {},\n  \"supported_abis\": {},\n  \"has_native_x86_64\": {},\n  \"has_native_arm64\": {},\n  \"needs_arm_translation\": {},\n  \"translation_method\": \"{}\",\n  \"files_count\": {}\n}}",
        escape_json_string(path),
        info.valid_apk,
        supported,
        info.has_native_x86_64,
        info.has_native_arm64,
        info.needs_arm_translation,
        escape_json_string(&info.translation_method),
        info.files_count
    )
}

fn render_invalid(path: &str, error: &str) -> String {
    format!(
        "{{\n  \"path\": \"{}\",\n  \"valid_apk\": false,\n  \"error\": \"{}\"\n}}",
        escape_json_string(path),
        escape_json_string(error)
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mark-apk-inspect <path-to-apk>");
        std::process::exit(1);
    }
    let path = &args[1];
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            println!("{}", render_invalid(path, &e.to_string()));
            std::process::exit(1);
        }
    };
    match apk::inspect_apk(&data) {
        Ok(info) => {
            println!("{}", render_apk_json(path, &info));
        }
        Err(e) => {
            println!("{}", render_invalid(path, &e.to_string()));
            std::process::exit(1);
        }
    }
}
