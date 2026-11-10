//! Minimal APK (Android package) inspection primitives.
//!
//! Reads the ZIP central directory of an APK to detect embedded native
//! library entries per ABI. Pure Rust with no external crates so the inspector
//! has no transitive runtime dependency of its own.

use std::io;

/// End-of-central-directory signature (bytes `50 4B 05 06`).
const EOCD_SIG: u32 = 0x06054B50;
/// Central directory file header signature (bytes `50 4B 01 02`).
const CEN_SIG: u32 = 0x02014B50;

/// Native library ABI directories understood by the inspector.
pub const ABI_ARM64_V8A: &str = "arm64-v8a";
pub const ABI_ARABIE_V7A: &str = "armeabi-v7a";
pub const ABI_X86_64: &str = "x86_64";
pub const ABI_X86: &str = "x86";

/// Inspection result for one APK file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkInfo {
    pub valid_apk: bool,
    pub supported_abis: Vec<String>,
    pub has_native_x86_64: bool,
    pub has_native_arm64: bool,
    pub needs_arm_translation: bool,
    pub translation_method: String,
    pub files_count: usize,
}

/// Return the native ABI targeted by a file entry inside the APK.
///
/// Only entries below `lib/<abi>/` are considered native library entries.
fn abi_of_entry(name: &str) -> Option<String> {
    let rest = name.strip_prefix("lib/")?;
    let abi = rest.split('/').next()?;
    if is_known_abi(abi) {
        Some(abi.to_string())
    } else {
        None
    }
}

fn is_known_abi(abi: &str) -> bool {
    abi == ABI_ARM64_V8A || abi == ABI_ARABIE_V7A || abi == ABI_X86_64 || abi == ABI_X86
}

fn read_u16_le(buf: &[u8], offset: usize) -> io::Result<u16> {
    if offset + 2 > buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "u16 read at offset {offset} exceeds buffer length {}",
                buf.len()
            ),
        ));
    }
    Ok(u16::from_le_bytes([buf[offset], buf[offset + 1]]))
}

fn read_u32_le(buf: &[u8], offset: usize) -> io::Result<u32> {
    if offset + 4 > buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "u32 read at offset {offset} exceeds buffer length {}",
                buf.len()
            ),
        ));
    }
    Ok(u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]))
}

/// Locate the end-of-central-directory record and return
/// `(central_directory_offset, central_directory_size, entry_count)`.
fn find_eocd(data: &[u8]) -> io::Result<(usize, usize, usize)> {
    if data.len() < 22 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to be a valid APK (< 22 bytes)",
        ));
    }
    let max_scan = 22 + 0xFFFF;
    let start = data.len().saturating_sub(max_scan);
    let mut pos = data.len() - 22;
    loop {
        if pos < start {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ZIP end-of-central-directory record not found",
            ));
        }
        if read_u32_le(data, pos)? == EOCD_SIG {
            let comment_len = read_u16_le(data, pos + 20)? as usize;
            if pos + 22 + comment_len == data.len() {
                let entries = read_u16_le(data, pos + 10)? as usize;
                let cd_size = read_u16_le(data, pos + 12)? as usize;
                let cd_offset = read_u32_le(data, pos + 16)? as usize;
                return Ok((cd_offset, cd_size, entries));
            }
        }
        if pos == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ZIP end-of-central-directory record not found",
            ));
        }
        pos = pos - 1;
    }
}

/// Inspect an APK byte buffer and report its native ABI coverage.
pub fn inspect_apk(data: &[u8]) -> io::Result<ApkInfo> {
    let (cd_offset, cd_size, entries) = find_eocd(data)?;
    let end = cd_offset.saturating_add(cd_size);
    let mut supported: Vec<String> = Vec::new();
    let mut files_count = 0usize;
    let mut offset = cd_offset;
    let mut processed = 0usize;
    while processed < entries && offset + 46 <= end {
        let sig = read_u32_le(data, offset)?;
        if sig != CEN_SIG {
            break;
        }
        let name_len = read_u16_le(data, offset + 28)? as usize;
        let extra_len = read_u16_le(data, offset + 30)? as usize;
        let comment_len = read_u16_le(data, offset + 32)? as usize;
        let name_start = offset + 46 + extra_len + comment_len;
        let next = offset + 46 + name_len + extra_len + comment_len;
        if next > end || name_start + name_len > data.len() {
            break;
        }
        let name = String::from_utf8_lossy(&data[name_start..name_start + name_len]).to_string();
        if let Some(abi) = abi_of_entry(&name) {
            if !supported.iter().any(|s| *s == abi) {
                supported.push(abi);
            }
        }
        files_count += 1;
        offset = next;
        processed += 1;
    }

    supported.sort();
    supported.dedup();
    let has_native_x86_64 = supported.iter().any(|abi| *abi == ABI_X86_64);
    let has_native_arm64 = supported.iter().any(|abi| *abi == ABI_ARM64_V8A);
    let has_native_x86 = supported.iter().any(|abi| *abi == ABI_X86);
    let needs_arm_translation = !supported.is_empty() && !has_native_x86_64 && !has_native_x86;

    Ok(ApkInfo {
        valid_apk: true,
        supported_abis: supported.clone(),
        has_native_x86_64,
        has_native_arm64,
        needs_arm_translation,
        translation_method: if needs_arm_translation {
            "libndk-translation".to_string()
        } else {
            "none".to_string()
        },
        files_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    /// Build a minimal synthetic APK image: central directory entries plus a
    /// well-formed end-of-central-directory record. No local file headers are
    /// emitted because the inspector only reads the central directory.
    fn build_apk_bytes(entries: &[&str]) -> Vec<u8> {
        let mut cd = Vec::new();
        for name in entries {
            let name_bytes = name.as_bytes();
            cd.push(0x50);
            cd.push(0x4B);
            cd.push(0x01);
            cd.push(0x02);
            push_u16(&mut cd, 0x0014);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u32(&mut cd, 0x0000_0000);
            push_u32(&mut cd, 0x0000_0000);
            push_u32(&mut cd, 0x0000_0000);
            push_u16(&mut cd, name_bytes.len() as u16);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u16(&mut cd, 0x0000);
            push_u32(&mut cd, 0x0000_0000);
            push_u32(&mut cd, 0x0000_0000);
            cd.extend_from_slice(name_bytes);
        }
        let cd_offset = 0usize;
        let cd_size = cd.len();
        let entry_count = entries.len() as u16;
        let mut out = cd;
        // End-of-central-directory record: sig(4) + disk(2) + cdDisk(2) +
        // entriesDisk(2) + entriesTotal(2) + cdSize(4) + cdOffset(4) + commentLen(2)
        push_u32(&mut out, 0x06_05_4B50);
        push_u16(&mut out, 0x0000);
        push_u16(&mut out, 0x0000);
        push_u16(&mut out, entry_count);
        push_u16(&mut out, entry_count);
        push_u32(&mut out, cd_size as u32);
        push_u32(&mut out, cd_offset as u32);
        push_u16(&mut out, 0x0000);
        out
    }

    #[test]
    fn test_arm64_only_apk_needs_translation() {
        let data = build_apk_bytes(&["lib/arm64-v8a/libtest.so", "AndroidManifest.xml"]);
        let info = inspect_apk(&data).expect("valid synthetic APK");
        assert!(info.valid_apk);
        assert_eq!(info.files_count, 2);
        assert_eq!(info.supported_abis, vec![ABI_ARM64_V8A.to_string()]);
        assert!(info.has_native_arm64);
        assert!(!info.has_native_x86_64);
        assert!(info.needs_arm_translation);
        assert_eq!(info.translation_method, "libndk-translation");
    }

    #[test]
    fn test_x86_64_and_arm64_apk_no_translation() {
        let data = build_apk_bytes(&["lib/x86_64/libtest.so", "lib/arm64-v8a/libtest2.so"]);
        let info = inspect_apk(&data).expect("valid synthetic APK");
        assert!(info.valid_apk);
        assert_eq!(info.supported_abis.len(), 2);
        assert!(info.has_native_x86_64);
        assert!(info.has_native_arm64);
        assert!(!info.needs_arm_translation);
        assert_eq!(info.translation_method, "none");
    }

    #[test]
    fn test_pure_java_apk_has_no_native_abi() {
        let data = build_apk_bytes(&["classes.dex", "AndroidManifest.xml"]);
        let info = inspect_apk(&data).expect("valid synthetic APK");
        assert!(info.valid_apk);
        assert!(info.supported_abis.is_empty());
        assert!(!info.has_native_x86_64);
        assert!(!info.has_native_arm64);
        assert!(!info.needs_arm_translation);
        assert_eq!(info.translation_method, "none");
    }

    #[test]
    fn test_invalid_apk_is_rejected() {
        let result = inspect_apk(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_abi_of_entry_detection() {
        assert_eq!(
            abi_of_entry("lib/arm64-v8a/libfoo.so").as_deref(),
            Some(ABI_ARM64_V8A)
        );
        assert_eq!(
            abi_of_entry("lib/x86_64/libfoo.so").as_deref(),
            Some(ABI_X86_64)
        );
        assert_eq!(
            abi_of_entry("lib/armeabi-v7a/libfoo.so").as_deref(),
            Some(ABI_ARABIE_V7A)
        );
        assert_eq!(abi_of_entry("lib/x86/libfoo.so").as_deref(), Some(ABI_X86));
        // Non-library and unknown ABI entries are ignored.
        assert_eq!(abi_of_entry("AndroidManifest.xml"), None);
        assert_eq!(abi_of_entry("assets/www/index.html"), None);
        assert_eq!(abi_of_entry("lib/unknownabi/libfoo.so"), None);
    }
}
