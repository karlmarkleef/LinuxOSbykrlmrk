# Architecture overview

## Product direction

Mark Linux is intended to be a real Arch-based distribution, not a Windows theme or a Linux-looking web application. Upstream Linux, systemd and pacman remain responsible for their established system roles. Original management utilities are planned in Rust.

## Planned layers

1. **Image and boot:** ArchISO inputs, upstream kernel, initramfs, systemd and a bootloader.
2. **Base OS:** pacman repositories and package sets, filesystem configuration, identity and service defaults.
3. **Session:** Wayland with Hyprland as the compositor/window manager; no single-window shell pretending to be a desktop.
4. **Desktop utilities:** native panel, launcher, settings, file manager, system monitor, welcome experience and installer.
5. **Platform tools:** hardware detection, performance profiles, package integration and installer orchestration.
6. **Windows experience:** Windows applications run through upstream Wine/Proton with DXVK/VKD3D-Proton on the Linux GPU; the Mark Compatibility Platform layers runtime, prefix, PE and APK integration on top. No Windows VM or desktop is required per application.

## Build boundary

Windows is the development host. The existing Debian-Recovered WSL2 distribution is used only as a host for a verified, disposable Arch bootstrap root. Build tooling stays inside that root; the Debian package set and Windows host are not modified. Project sources and deliverables stay on H:, while ArchISO's Unix-sensitive scratch tree is stored on WSL ext4. The first image must still be built and boot-tested in an isolated VM before Phase 1 can be marked complete.

## Phase gates

A subsystem is not complete because its directory or design exists. Completion requires source, a reproducible build command, test evidence and recorded limitations in docs/DEVELOPMENT_STATUS.md.