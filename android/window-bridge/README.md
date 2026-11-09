# android/window-bridge

## Purpose

Integrates **Android application windows** from Waydroid into the Mark Linux Hyprland desktop. Android apps can appear as regular Wayland windows with proper rounding, animations, and tiling support — indistinguishable from native Linux apps.

## Status

**Phase 5 — In Progress**

Hyprland window rules are defined for Waydroid windows. Full multi-window Android app support depends on Waydroid's `waydroid-extensions` or multi-window patches.

## Upstream

- **Waydroid** — exposes Android windows as Wayland surfaces
- **Hyprland** — Mark Linux compositor handles window rules
- **XDG Shell** — Wayland protocol for window management

## Components

| File | Description |
|------|-------------|
| `waydroid-window-rules.conf` | Hyprland window rules for Android apps |

## Window Integration

Android apps appear as Wayland windows with class names matching `waydroid.*`. The window rules apply:
- 12px corner rounding (matches Mark Linux theme)
- Slide animations
- Fullscreen support
- Opacity consistency

## Activation

Add to `~/.config/hypr/hyprland.conf`:
```ini
source = ~/.config/hypr/waydroid-window-rules.conf
```
