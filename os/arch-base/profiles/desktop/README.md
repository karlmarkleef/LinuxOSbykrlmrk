# ArchISO profile

This is the Phase 1 live-boot profile for Mark Linux. It is adapted from the ArchISO baseline profile installed from the verified Arch bootstrap input recorded in `docs/development/BUILD_INPUTS.md`.

The first image intentionally contains only the Arch live base, upstream kernel and firmware, initramfs tools, and BIOS boot support. It is a bootable engineering base, not yet the planned Hyprland desktop or installer. The overlay sets the Mark Linux release identity, and the boot menus use the project name. Upstream ArchISO files are retained where needed for boot behavior.


The profile sets the live hostname to `marklinux`. Service links for cloud-init, SSH and optional VM guest packages were removed because those packages are not in this minimal image. NetworkManager and the graphical session will be designed in later phases.
