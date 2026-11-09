# android/storage-bridge

## Purpose

Manages **shared storage** between the Android container and Mark Linux host filesystem. Provides a secure, sandboxed shared folder that Android apps can access, while keeping the Linux root filesystem isolated from Android apps.

## Status

**Phase 5 — Planned**

Waydroid's built-in shared folder (`~/Android/shared`) is used. Mark Linux storage bridge will add automatic mounting, permission management, and optional expanded share paths.

## Upstream

- **Waydroid** — `waydroid show-full-ui` and shared folder feature
- **LXC** — bind-mount for shared folder
- **FUSE** — optional for expanded storage access

## Components

| File | Description |
|------|-------------|
| *(planned)* `storage-config.toml` | Storage bridge configuration |
| *(planned)* `setup-shared-folder.sh` | Initialize and configure shared folder |

## Shared Folder

The default shared folder is `~/Android/shared/`. Files placed here are accessible from Android at `/sdcard/`.

```
Host:    ~/Android/shared/  ←→  Android: /sdcard/
```

## Security Notes

Only the shared folder is accessible from Android by default. The Linux root filesystem is not accessible. This is enforced by the LXC container policy in `security/sandbox-policy.toml`.
