//! mark-pe-inspect — CLI entry point
//!
//! Usage: mark-pe-inspect <path-to-exe>
//!
//! Reads the PE32/PE32+ headers of a Windows executable and prints a JSON
//! object describing the binary's architecture, subsystem, minimum Windows
//! version, and whether it can run natively on this host.

mod pe;

use std::{env, fs, process};

fn usage() -> ! {
    eprintln!("Usage: mark-pe-inspect <path-to-exe>");
    eprintln!();
    eprintln!("Reads PE32/PE32+ headers and prints a JSON compatibility report.");
    eprintln!();
    eprintln!("Exit codes:");
    eprintln!("  0  success (even if binary cannot run natively)");
    eprintln!("  1  file not found or not a valid PE binary");
    process::exit(1);
}

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
    }

    let path = &args[1];

    // Read file
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            // Output JSON even on error so callers can always parse stdout
            let escaped = escape_json_string(path);
            let err_msg = escape_json_string(&e.to_string());
            println!(
                "{{\n  \"path\": \"{escaped}\",\n  \"valid_pe\": false,\n  \"error\": \"{err_msg}\"\n}}"
            );
            process::exit(1);
        }
    };

    // Parse PE headers
    match pe::parse_pe(&data) {
        Ok(info) => {
            let path_escaped = escape_json_string(path);
            let arch         = info.architecture();
            let subsystem    = info.subsystem_name();
            let pe_type      = info.pe_type();
            let min_ver      = info.min_windows_version();
            let native       = info.can_run_native();
            let translate    = info.needs_translation();

            println!(
                "{{\n  \"path\": \"{path_escaped}\",\n  \"valid_pe\": true,\n  \"architecture\": \"{arch}\",\n  \"subsystem\": \"{subsystem}\",\n  \"pe_type\": \"{pe_type}\",\n  \"min_windows_version\": \"{min_ver}\",\n  \"can_run_native\": {native},\n  \"needs_translation\": {translate}\n}}"
            );
        }
        Err(e) => {
            let escaped = escape_json_string(path);
            let err_msg = escape_json_string(&e.to_string());
            println!(
                "{{\n  \"path\": \"{escaped}\",\n  \"valid_pe\": false,\n  \"error\": \"{err_msg}\"\n}}"
            );
            process::exit(1);
        }
    }
}
