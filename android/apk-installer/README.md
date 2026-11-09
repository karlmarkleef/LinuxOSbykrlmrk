# android/apk-installer

## Purpose

Houses the **`mark-apk-inspect`** Rust crate - a pure-Rust APK file inspector. It reads the APK ZIP structure to detect ABI targets, counts files, and validates APK integrity before installation.

## Status

**Phase 5 - Implemented**

`mark-apk-inspect` is a working Rust binary. It inspects APK files using a minimal pure-Rust ZIP central directory reader (no external crates).

## Upstream

- **APK format** - Android Package (ZIP with `AndroidManifest.xml`)
- **ZIP specification** - PKWARE ZIP format (EOCD + central directory)

## Components

| File | Description |
|------|-------------|
| `Cargo.toml` | Rust crate manifest |
| `src/main.rs` | CLI entrypoint - outputs JSON |
| `src/apk.rs` | APK inspection logic, pure-Rust ZIP reader |

## Usage

```bash
mark-apk-inspect /path/to/app.apk
```

### Example Output

```json
{
  "path": "/path/to/app.apk",
  "valid_apk": true,
  "supported_abis": ["arm64-v8a", "armeabi-v7a"],
  "has_native_x86_64": false,
  "has_native_arm64": true,
  "needs_arm_translation": true,
  "translation_method": "libndk-translation",
  "files_count": 142
}
```

## ABI Detection

APKs embed native libraries under `lib/<ABI>/`. The inspector reads the ZIP central directory and looks for:

| Path prefix | ABI |
|-------------|-----|
| `lib/arm64-v8a/` | 64-bit ARM |
| `lib/armeabi-v7a/` | 32-bit ARM |
| `lib/x86_64/` | 64-bit x86 |
| `lib/x86/` | 32-bit x86 |

APKs with no `lib/` entries are pure Java/Kotlin and run on any ABI.

## Build

```bash
cd android/apk-installer
cargo build --release
# Binary: target/release/mark-apk-inspect
```
