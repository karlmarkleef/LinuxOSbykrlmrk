# android/binder-bridge

## Purpose

Manages the **Binder IPC bridge** between the Android container (Waydroid) and the Mark Linux host. Binder is Android's core inter-process communication mechanism. This module handles the `binder_linux` kernel module loading, Binder device node setup (`/dev/binder`, `/dev/hwbinder`, `/dev/vndbinder`), and monitors Binder health.

## Status

**Phase 5 — Planned**

The `binder_linux` kernel module loading is handled by `mark-android-manager`. Detailed Binder monitoring and diagnostics tools are planned.

## Upstream

- **binder_linux** kernel module — `linux/drivers/android/binder.c`
- **Waydroid** — uses Binder for Android IPC across the container boundary
- **Linux kernel** — Mark Linux kernels ship with `CONFIG_ANDROID_BINDER_IPC=y`

## Components

| File | Description |
|------|-------------|
| *(planned)* `binder-monitor.sh` | Monitor Binder transactions and detect stalls |
| *(planned)* `binder-setup.sh` | Set up Binder device nodes with correct permissions |

## Binder Devices

| Device | Purpose |
|--------|---------|
| `/dev/binder` | App-to-system-server IPC |
| `/dev/hwbinder` | Hardware HAL IPC |
| `/dev/vndbinder` | Vendor IPC |

## Notes

If the host kernel does not have `binder_linux` built-in, the DKMS module from `linux-android-binder-dkms` (AUR) can be used. Mark Linux kernels ship with native Binder support.
