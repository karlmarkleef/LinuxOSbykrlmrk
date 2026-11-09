# android/aosp-integration

## Purpose

Handles integration with **AOSP (Android Open Source Project)** images used by Waydroid. This module manages image downloads, verification, patching, and updates for the Android system image that runs inside the Waydroid container.

## Status

**Phase 5 — Planned**

AOSP image management automation and Mark-specific patches are planned. Currently relies on upstream Waydroid images directly.

## Upstream

- **AOSP** — Android Open Source Project (android.googlesource.com)
- **Waydroid Images** — LineageOS-based AOSP images built for Waydroid
- **LineageOS** — lineageos.org — provides the Android base for Waydroid

## Components

| File | Description |
|------|-------------|
| *(planned)* `fetch-aosp-image.sh` | Download and verify Waydroid AOSP images |
| *(planned)* `patch-aosp.sh` | Apply Mark Linux specific AOSP patches |

## Image Variants

| Variant | Description |
|---------|-------------|
| `VANILLA` | Base AOSP without Google apps |
| `GAPPS` | AOSP with Google Play Services |

## Notes

Waydroid uses a compressed squashfs system image (`system.img`) and a data image (`vendor.img`) from the LineageOS project. Mark Linux will host a verified mirror and optionally apply additional patches for better Waydroid-host integration.
