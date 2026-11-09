# android/arm-translation

## Purpose

Manages **ARM-to-x86 binary translation** for the Mark Android Subsystem. Most Android apps (and games) are compiled for ARM CPUs. When running Waydroid on an x86 PC, ARM native libraries need a translation layer. This module configures and sets up translation methods.

## Status

**Phase 5 — Planned**

Configuration files are present. The `setup-arm-translation.sh` script automates `libndk-translation` installation via `waydroid_script`. Full integration with Mark Linux installer is planned.

## Upstream

- **libndk-translation** — libhoudini-compatible ARM translator (community)
- **waydroid_script** — github.com/casualsnek/waydroid_script — community setup tool
- **Waydroid** — ARM translation hooks in Waydroid image

## Components

| File | Description |
|------|-------------|
| `translation-config.toml` | Translation method configuration |
| `setup-arm-translation.sh` | Automated ARM translation setup script |

## Translation Methods

| Method | ABIs Supported | Status |
|--------|---------------|--------|
| `libndk-translation` | armeabi-v7a, arm64-v8a | Community-supported |
| `none` | x86, x86_64 only | Built-in |

## Notes

ARM translation significantly increases app compatibility. Without it, only apps with x86/x86_64 native code or pure Java/Kotlin apps will run. With `libndk-translation`, the vast majority of ARM64 apps and games become compatible.
