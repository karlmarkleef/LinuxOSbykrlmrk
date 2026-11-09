# android/input-bridge

## Purpose

Bridges **keyboard, mouse, touchpad, and gamepad input** from Mark Linux to Android apps running in Waydroid. Handles input event translation, touch simulation from mouse input, and game controller mapping for Android games.

## Status

**Phase 5 — Planned**

Waydroid handles basic input via Wayland input protocols. Enhanced input mapping (e.g., keyboard-to-gamepad for games) is planned.

## Upstream

- **Waydroid** — receives Wayland input events
- **udev** — Linux input device management
- **libinput** — Wayland compositor input handling (Hyprland)

## Components

| File | Description |
|------|-------------|
| *(planned)* `input-mapping.toml` | Custom input mapping configuration |
| *(planned)* `gamepad-remap.sh` | Remap Linux gamepad for Android games |

## Input Types

| Type | Status | Notes |
|------|--------|-------|
| Keyboard | Working | Via Wayland |
| Mouse (cursor) | Working | Via Wayland |
| Touch simulation | Working | Mouse click → touch |
| Gamepad | Partial | Needs mapping config |
| Stylus | Planned | — |
