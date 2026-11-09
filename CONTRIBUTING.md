# Contributing

Mark Linux is in Phase 0. Keep changes small and update docs/DEVELOPMENT_STATUS.md at each phase boundary.

## Before changing files

- Inspect existing content and preserve user assets.
- Keep all source, build inputs, logs and outputs under H:\LinuxOSbykrlmrk.
- Never write to or repair host partitions as part of a normal build.
- Use disposable VM disks for destructive installer tests.
- Do not claim a build or test passed without its command and result.

## Code direction

- Prefer Rust for new system components.
- Use upstream Arch, Linux, systemd, pacman, Wayland and Hyprland rather than reimplementing them.
- Pin or record upstream versions before relying on configuration syntax.
- Document dependencies that are unavailable on the host; avoid installing software system-wide without a concrete phase need.