# Mark Linux

Mark Linux is an early-stage Arch-based Linux distribution project.

- Version: 0.0.1_dev
- Build ID: 1101126_1
- Current milestone: hardware inventory and installation planner integration
- Planned desktop: Wayland with Hyprland
- Planned platform tools: Rust

The project contains a verified Arch bootstrap input, a BIOS-boot-tested minimal live image archived under `builds/archive/phase1-complete/`, and a Hyprland desktop ISO whose live session has been visually verified through direct-kernel and BIOS-menu QEMU boot. The read-only Rust hardware inventory passes Linux fixture tests and is packaged in the desktop image with a Hardware Summary menu action. A no-write GUI installation planner is included in the latest ISO and has seven passing source tests. It is not yet an OS installer. See `docs/DEVELOPMENT_STATUS.md` for verified progress and current limits.

Desktop controls and current limitations are documented in [`desktop/README.md`](desktop/README.md).

## Project principles

- Build a real Arch-based system using upstream components rather than a simulated Linux interface.
- Keep project source, inputs, logs and release artifacts under H:. Build-tool scratch files use a disposable WSL ext4 workspace.
- Preserve source assets and user data. Never format or modify host disks as part of an ordinary build.
- Do not claim a feature or artifact is complete until it has been built and tested.
- Prefer small, reviewable phases with reproducible commands and recorded results.
