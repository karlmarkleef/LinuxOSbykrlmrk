# android/package-manager

## Purpose

Provides the **`mark-apk` command** — a user-friendly APK package manager for the Mark Android Subsystem. It wraps Waydroid's `app install/list/remove/launch` commands with better UX: pre-installation inspection, ABI compatibility checks, confirmation prompts, and formatted output.

## Status

**Phase 5 - In Progress**

The `mark-apk` script is implemented at `android/package-manager/mark-apk`. It wraps Waydroid's `app install/list/remove/launch` commands with pre-installation inspection, ABI compatibility checks, and confirmation prompts. Integration with a GUI app store frontend is planned.

The `mark-apk-inspect` binary (from `android/apk-installer/`) provides the pre-installation ABI inspection used by `mark-apk install`.

## Upstream

- **Waydroid** — `waydroid app install/list/remove/launch`
- **mark-apk-inspect** — from `android/apk-installer/` Rust crate

## Components

| File | Description |
|------|-------------|
| `mark-apk` | Main APK package manager CLI script |

## Usage

```bash
mark-apk install /path/to/app.apk     # Inspect then install
mark-apk list                          # List installed Android apps
mark-apk uninstall com.example.app     # Remove an app
mark-apk launch com.example.app        # Launch an app
mark-apk inspect /path/to/app.apk      # Show APK info without installing
```

## Dependencies

- `waydroid` — Android container runtime
- `mark-apk-inspect` — APK file inspector (from `android/apk-installer/`)
- Waydroid container must be running (`mark-android-manager start`)
