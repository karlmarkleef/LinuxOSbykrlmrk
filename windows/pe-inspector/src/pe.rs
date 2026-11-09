//! PE32/PE32+ parser — pure Rust, zero external crates.
//!
//! Parses the DOS header, PE signature, COFF header, and Optional Header
//! of a Windows Portable Executable to extract architecture, subsystem,
//! and minimum OS version.

use std::io;

// ── machine constants (COFF header) ─────────────────────────────────────────
pub const MACHINE_AMD64: u16 = 0x8664;
pub const MACHINE_I386:  u16 = 0x014C;
pub const MACHINE_ARM64: u16 = 0xAA64;
pub const MACHINE_THUMB: u16 = 0x01C4;

// ── optional-header magic ────────────────────────────────────────────────────
pub const MAGIC_PE32:  u16 = 0x010B;
pub const MAGIC_PE32P: u16 = 0x020B; // PE32+

// ── subsystem constants ──────────────────────────────────────────────────────
pub const SUBSYSTEM_GUI:     u16 = 2;
pub const SUBSYSTEM_CONSOLE: u16 = 3;
pub const SUBSYSTEM_EFI:     u16 = 9;

// ── DOS header magic ─────────────────────────────────────────────────────────
const DOS_MAGIC: [u8; 2] = [0x4D, 0x5A]; // "MZ"
const PE_SIG:    [u8; 4] = [0x50, 0x45, 0x00, 0x00]; // "PE\0\0"

// ── optional-header field offsets (from start of optional header) ────────────
// These are the same for both PE32 and PE32+:
const OPT_MAGIC_OFF:          usize = 0;
const OPT_MAJOR_OS_VER_OFF:   usize = 40;
const OPT_MINOR_OS_VER_OFF:   usize = 42;
const OPT_SUBSYSTEM_OFF:      usize = 68;

/// Parsed PE header information.
#[derive(Debug, Clone, PartialEq)]
pub struct PeInfo {
    pub machine:            u16,
    pub number_of_sections: u16,
    pub opt_magic:          u16,
    pub major_os_version:   u16,
    pub minor_os_version:   u16,
    pub subsystem:          u16,
}

impl PeInfo {
    /// Human-readable architecture string.
    pub fn architecture(&self) -> &'static str {
        match self.machine {
            MACHINE_AMD64 => "x86_64",
            MACHINE_I386  => "x86_32",
            MACHINE_ARM64 => "arm64",
            MACHINE_THUMB => "arm32",
            _             => "unknown",
        }
    }

    /// Whether the binary can run natively on an x86-64 host.
    pub fn can_run_native(&self) -> bool {
        self.machine == MACHINE_AMD64
    }

    /// Whether architecture translation (e.g. QEMU) is needed.
    pub fn needs_translation(&self) -> bool {
        matches!(self.machine, MACHINE_ARM64 | MACHINE_THUMB)
    }

    /// PE format string.
    pub fn pe_type(&self) -> &'static str {
        match self.opt_magic {
            MAGIC_PE32  => "PE32",
            MAGIC_PE32P => "PE32+",
            _           => "unknown",
        }
    }

    /// Subsystem as a human-readable string.
    pub fn subsystem_name(&self) -> &'static str {
        match self.subsystem {
            SUBSYSTEM_GUI     => "windows_gui",
            SUBSYSTEM_CONSOLE => "windows_console",
            SUBSYSTEM_EFI     => "efi_application",
            _                 => "unknown",
        }
    }

    /// Minimum Windows version as "major.minor" string.
    pub fn min_windows_version(&self) -> String {
        format!("{}.{}", self.major_os_version, self.minor_os_version)
    }
}

// ── helper: read a little-endian u16 from a byte slice ──────────────────────
fn read_u16_le(buf: &[u8], offset: usize) -> io::Result<u16> {
    if offset + 2 > buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("u16 read at offset {offset} exceeds buffer length {}", buf.len()),
        ));
    }
    Ok(u16::from_le_bytes([buf[offset], buf[offset + 1]]))
}

// ── helper: read a little-endian u32 from a byte slice ──────────────────────
fn read_u32_le(buf: &[u8], offset: usize) -> io::Result<u32> {
    if offset + 4 > buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("u32 read at offset {offset} exceeds buffer length {}", buf.len()),
        ));
    }
    Ok(u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]))
}

/// Parse a PE binary from a raw byte buffer.
///
/// Returns `Err` for any of:
/// - Invalid DOS magic ("MZ" signature missing)
/// - Invalid PE signature ("PE\0\0" missing)
/// - Buffer too short to contain the full set of required headers
pub fn parse_pe(data: &[u8]) -> io::Result<PeInfo> {
    // 1. DOS header: check MZ magic (bytes 0–1)
    if data.len() < 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to be a valid PE (< 64 bytes)",
        ));
    }
    if data[0..2] != DOS_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Invalid DOS magic: expected 0x4D 0x5A ('MZ'), found 0x{:02X} 0x{:02X}",
                data[0], data[1]
            ),
        ));
    }

    // 2. PE offset is a u32 LE at byte 0x3C (60)
    let pe_offset = read_u32_le(data, 0x3C)? as usize;

    // 3. PE signature: 4 bytes at pe_offset
    if pe_offset + 4 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("PE offset {pe_offset} exceeds file length"),
        ));
    }
    if data[pe_offset..pe_offset + 4] != PE_SIG {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Invalid PE signature at offset {pe_offset}: {:?}",
                &data[pe_offset..pe_offset + 4]
            ),
        ));
    }

    // 4. COFF header: 20 bytes immediately after PE signature
    let coff_offset = pe_offset + 4;
    if coff_offset + 20 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "File truncated inside COFF header",
        ));
    }

    let machine            = read_u16_le(data, coff_offset)?;         // offset 0
    let number_of_sections = read_u16_le(data, coff_offset + 2)?;    // offset 2
    // timestamp at +4 (4 bytes), skipped
    // PointerToSymbolTable +8, NumberOfSymbols +12 — skipped
    let size_of_opt_header = read_u16_le(data, coff_offset + 16)?;   // offset 16
    // characteristics at +18 — skipped

    if size_of_opt_header == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "SizeOfOptionalHeader is 0 — no optional header present",
        ));
    }

    // 5. Optional header starts immediately after COFF header
    let opt_offset = coff_offset + 20;

    // We need at least OPT_SUBSYSTEM_OFF + 2 = 70 bytes of optional header
    let required_opt_bytes = OPT_SUBSYSTEM_OFF + 2;
    if (size_of_opt_header as usize) < required_opt_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Optional header too small ({} bytes); need at least {required_opt_bytes}",
                size_of_opt_header
            ),
        ));
    }
    if opt_offset + required_opt_bytes > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "File truncated inside Optional header",
        ));
    }

    let opt_magic        = read_u16_le(data, opt_offset + OPT_MAGIC_OFF)?;
    let major_os_version = read_u16_le(data, opt_offset + OPT_MAJOR_OS_VER_OFF)?;
    let minor_os_version = read_u16_le(data, opt_offset + OPT_MINOR_OS_VER_OFF)?;
    let subsystem        = read_u16_le(data, opt_offset + OPT_SUBSYSTEM_OFF)?;

    // Validate the optional-header magic
    if opt_magic != MAGIC_PE32 && opt_magic != MAGIC_PE32P {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown optional header magic: 0x{opt_magic:04X}"),
        ));
    }

    Ok(PeInfo {
        machine,
        number_of_sections,
        opt_magic,
        major_os_version,
        minor_os_version,
        subsystem,
    })
}

// ── unit tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal but structurally valid PE32+ byte buffer.
    ///
    /// Layout:
    ///   [0..2]   DOS magic "MZ"
    ///   [0x3C..0x40] PE offset (u32 LE) = 0x40 (64)
    ///   [0x40..0x44] PE signature "PE\0\0"
    ///   [0x44..0x58] COFF header (20 bytes)
    ///   [0x58..]     Optional header (at least 70 bytes)
    fn build_minimal_pe(
        machine: u16,
        opt_magic: u16,
        subsystem: u16,
        major_os: u16,
        minor_os: u16,
    ) -> Vec<u8> {
        let pe_offset: u32 = 0x40;
        let opt_header_size: u16 = 240; // standard PE32+ optional header size

        // Start with a zeroed buffer large enough
        // pe_offset(64) + 4(sig) + 20(COFF) + 240(OPT) = 328 bytes
        let total = pe_offset as usize + 4 + 20 + opt_header_size as usize;
        let mut buf = vec![0u8; total];

        // DOS magic
        buf[0] = 0x4D;
        buf[1] = 0x5A;

        // PE offset at 0x3C
        let off_bytes = pe_offset.to_le_bytes();
        buf[0x3C] = off_bytes[0];
        buf[0x3D] = off_bytes[1];
        buf[0x3E] = off_bytes[2];
        buf[0x3F] = off_bytes[3];

        // PE signature
        let pe_start = pe_offset as usize;
        buf[pe_start]     = 0x50; // 'P'
        buf[pe_start + 1] = 0x45; // 'E'
        buf[pe_start + 2] = 0x00;
        buf[pe_start + 3] = 0x00;

        // COFF header
        let coff = pe_start + 4;
        let m = machine.to_le_bytes();
        buf[coff]     = m[0];
        buf[coff + 1] = m[1];
        // number_of_sections = 3
        buf[coff + 2] = 3;
        buf[coff + 3] = 0;
        // SizeOfOptionalHeader at coff+16
        let s = opt_header_size.to_le_bytes();
        buf[coff + 16] = s[0];
        buf[coff + 17] = s[1];

        // Optional header
        let opt = coff + 20;
        let om = opt_magic.to_le_bytes();
        buf[opt + OPT_MAGIC_OFF]        = om[0];
        buf[opt + OPT_MAGIC_OFF + 1]    = om[1];
        let ma = major_os.to_le_bytes();
        buf[opt + OPT_MAJOR_OS_VER_OFF]     = ma[0];
        buf[opt + OPT_MAJOR_OS_VER_OFF + 1] = ma[1];
        let mi = minor_os.to_le_bytes();
        buf[opt + OPT_MINOR_OS_VER_OFF]     = mi[0];
        buf[opt + OPT_MINOR_OS_VER_OFF + 1] = mi[1];
        let ss = subsystem.to_le_bytes();
        buf[opt + OPT_SUBSYSTEM_OFF]     = ss[0];
        buf[opt + OPT_SUBSYSTEM_OFF + 1] = ss[1];

        buf
    }

    #[test]
    fn test_parse_valid_pe32plus_amd64() {
        let buf = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 6, 1);
        let info = parse_pe(&buf).expect("Should parse valid PE32+");
        assert_eq!(info.machine,            MACHINE_AMD64);
        assert_eq!(info.opt_magic,          MAGIC_PE32P);
        assert_eq!(info.subsystem,          SUBSYSTEM_GUI);
        assert_eq!(info.major_os_version,   6);
        assert_eq!(info.minor_os_version,   1);
        assert_eq!(info.number_of_sections, 3);
    }

    #[test]
    fn test_architecture_strings() {
        let machines = [
            (MACHINE_AMD64, "x86_64"),
            (MACHINE_I386,  "x86_32"),
            (MACHINE_ARM64, "arm64"),
            (MACHINE_THUMB, "arm32"),
            (0xFFFF,        "unknown"),
        ];
        for (machine, expected) in &machines {
            let buf = build_minimal_pe(*machine, MAGIC_PE32P, SUBSYSTEM_CONSOLE, 10, 0);
            // unknown machine still passes parsing (we only validate opt_magic)
            match parse_pe(&buf) {
                Ok(info) => assert_eq!(info.architecture(), *expected),
                Err(_)   => {
                    // 0xFFFF machine is fine, but if opt_magic were also wrong it would fail.
                    // Here opt_magic is MAGIC_PE32P so parsing must succeed.
                    panic!("Parsing failed for machine 0x{machine:04X}");
                }
            }
        }
    }

    #[test]
    fn test_can_run_native_and_needs_translation() {
        let amd64 = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 10, 0);
        let info  = parse_pe(&amd64).unwrap();
        assert!(info.can_run_native());
        assert!(!info.needs_translation());

        let arm64 = build_minimal_pe(MACHINE_ARM64, MAGIC_PE32P, SUBSYSTEM_GUI, 10, 0);
        let info  = parse_pe(&arm64).unwrap();
        assert!(!info.can_run_native());
        assert!(info.needs_translation());

        let i386 = build_minimal_pe(MACHINE_I386, MAGIC_PE32, SUBSYSTEM_CONSOLE, 5, 1);
        let info  = parse_pe(&i386).unwrap();
        assert!(!info.can_run_native());
        assert!(!info.needs_translation()); // x86_32 uses WoW64 equiv, not QEMU
    }

    #[test]
    fn test_subsystem_names() {
        let cases = [
            (SUBSYSTEM_GUI,     "windows_gui"),
            (SUBSYSTEM_CONSOLE, "windows_console"),
            (SUBSYSTEM_EFI,     "efi_application"),
            (99u16,             "unknown"),
        ];
        for (ss, name) in &cases {
            let buf  = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, *ss, 10, 0);
            let info = parse_pe(&buf).unwrap();
            assert_eq!(info.subsystem_name(), *name);
        }
    }

    #[test]
    fn test_pe_type_strings() {
        let pe32  = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32,  SUBSYSTEM_GUI, 6, 1);
        let pe32p = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 6, 1);
        assert_eq!(parse_pe(&pe32).unwrap().pe_type(),  "PE32");
        assert_eq!(parse_pe(&pe32p).unwrap().pe_type(), "PE32+");
    }

    #[test]
    fn test_min_windows_version_string() {
        let buf  = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 6, 1);
        let info = parse_pe(&buf).unwrap();
        assert_eq!(info.min_windows_version(), "6.1");
    }

    #[test]
    fn test_invalid_dos_magic_returns_error() {
        let mut buf = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 10, 0);
        buf[0] = 0x00; // corrupt DOS magic
        let result = parse_pe(&buf);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Invalid DOS magic") || msg.contains("MZ"));
    }

    #[test]
    fn test_invalid_pe_signature_returns_error() {
        let mut buf = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 10, 0);
        // Corrupt PE signature
        buf[0x40] = 0xDE;
        buf[0x41] = 0xAD;
        let result = parse_pe(&buf);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Invalid PE signature") || msg.contains("PE"));
    }

    #[test]
    fn test_too_small_buffer_returns_error() {
        let buf    = vec![0x4D, 0x5A, 0x00, 0x00]; // only 4 bytes
        let result = parse_pe(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_opt_magic_returns_error() {
        let mut buf = build_minimal_pe(MACHINE_AMD64, MAGIC_PE32P, SUBSYSTEM_GUI, 10, 0);
        // Corrupt optional header magic
        let opt_offset = 0x40 + 4 + 20; // pe_offset + sig + COFF
        buf[opt_offset]     = 0xAB;
        buf[opt_offset + 1] = 0xCD;
        let result = parse_pe(&buf);
        assert!(result.is_err());
    }
}
