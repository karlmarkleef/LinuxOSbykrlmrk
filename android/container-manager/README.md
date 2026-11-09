# android/container-manager

## Purpose

Manages the **Waydroid container lifecycle** — the core LXC container that runs Android (AOSP) on Mark Linux. This module provides high-level control over starting, stopping, initializing, and querying the Waydroid Android container.

## Status

**Phase 5 - Implemented**

The `mark-android-manager` shell script is implemented at
`android/container-manager/mark-android-manager`. It controls an
already-installed Waydroid instance through the upstream `waydroid` commands.

Full integration with Mark Linux boot services and the Mark System Manager is
planned for a later phase.

## Upstream

- **Waydroid** — [waydroid.org](https://waydroid.org) — LXC-based Android container for Linux
- **binder_linux** kernel module — required for Android IPC
- **LXC** — Linux Containers runtime

## Components

| File | Description |
|------|-------------|
| `mark-android-manager` | Main lifecycle management script |

## Usage

```bash
mark-android-manager start       # Start the Waydroid container
mark-android-manager stop        # Stop gracefully
mark-android-manager status      # Show status as JSON
mark-android-manager restart     # Restart
mark-android-manager init        # Initialize Waydroid (first time)
mark-android-manager init --gapps # Initialize with GApps support
```

## Dependencies

- `waydroid` — must be installed on the host
- `binder_linux` kernel module (or `binder_linux` DKMS)
- Proper GPU drivers for hardware acceleration

## Notes

Waydroid requires a Linux kernel with `binder` and `ashmem` support (or `CONFIG_ANDROID_BINDER_IPC=y`). Mark Linux kernels are built with these enabled.
