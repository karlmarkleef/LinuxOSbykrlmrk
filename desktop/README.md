# Mark Linux desktop profile

This source tree defines the separate ArchISO desktop profile at `os/arch-base/profiles/desktop`. The live session uses Hyprland's Lua configuration, Waybar, `nwg-dock-hyprland`, Fuzzel, Mako, Foot, Thunar, and the standard NetworkManager, PipeWire, Bluetooth, display and audio tools.

## Live session

- The `mark` live account is created without a password and is not in the `wheel` group. The root password is locked. Autologin is limited to `tty1` for the temporary live session.
- `Super+Enter` opens Foot; `Super+R` opens Fuzzel; `Super+E` opens Thunar; `Super+G` opens quick settings; `Super+W` opens the welcome menu; `Super+Q` closes the focused window; `Super+Shift+Q` exits Hyprland.
- The quick-settings menu starts NetworkManager, audio, display, Bluetooth, performance-profile and compositor-effects tools.
- The welcome menu offers Hardware Summary, Installation Planner, the file manager, settings tools, Mission Center, terminal and quick settings. Hardware Summary runs the read-only Rust scanner. The planner saves a no-write JSON plan only; it is not an OS installer.
- `Super+1` through `Super+5` select workspaces; `Super+Shift+1` through `Super+Shift+5` move the focused window.

## Performance controls

Four compositor profiles are defined in `motion/performance-profiles.toml`: `ultra-lite`, `balanced`, `cinematic`, and `custom`. Cinematic is applied at session start. The Waybar control and `Super+Shift+V` toggle Hyprland animations, blur, and shadows. They do not claim to disable animation inside every individual application.

## Build status and limits

The package list includes AMD microcode for the inspected Ryzen host and NVIDIA open kernel modules for the Turing-class RTX 2060 SUPER. The exact driver/hardware path has not been tested on physical hardware. The desktop boots in QEMU BIOS mode and its Hyprland GUI was visually verified; the QEMU display uses virtual hardware. The Rust hardware scanner is part of the current ISO build and its Linux fixture tests pass. The newer installation planner source is being integrated into the next ISO build. See `docs/DEVELOPMENT_STATUS.md` for current limits and build records.

The wallpaper is original abstract artwork. The requested character logo was not included in the provided files and is not fabricated from the unrelated worksheet image.
