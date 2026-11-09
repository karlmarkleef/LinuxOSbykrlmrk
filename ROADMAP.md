# Roadmap

## Phase 0 — Environment and project foundation (complete)

- Inspect H:, the existing project tree, and available development tools.
- Preserve existing project data.
- Initialize Git and record architecture, security boundaries and tool availability.
- Done when the environment record and development status are recorded in the repository. No build artifact is expected in this phase.

## Phase 1 — Arch base and reproducible image build

- Select and pin an ArchISO-based build input.
- Add version/build identity, package manifests, os-release and boot configuration as source files.
- Build in a Linux-capable environment and retain logs/artifacts under builds/.
- Test that a generated ISO boots before calling the base phase complete.

## Phase 2 — Hyprland desktop

- Add a real Wayland/Hyprland session with a minimal usable panel, launcher and configuration.
- Add profile-driven motion settings using directives supported by the pinned Hyprland version.
- Build and boot-test the desktop in a VM.

Current progress: a bootable desktop ISO with Hyprland, Waybar, dock, launcher and Welcome GUI has been built and tested through both direct-kernel and standard Syslinux BIOS boot in QEMU. The captured desktop shows no Hyprland config warnings. Physical NVIDIA driver and UEFI firmware boot are not tested.

## Phase 3 — Installer and hardware detection

- Implement a GUI installer and Rust hardware-detection component.
- Keep destructive disk operations behind explicit confirmation and test on disposable VM disks.

Current progress: the read-only Rust inventory covers CPU topology, memory, storage metadata, PCI GPU/driver/optional VRAM, DRM monitor modes/EDID, network and Bluetooth. Five fixture-based Linux tests pass; an optimized Linux binary is packaged in the desktop image. A Zenity GUI now creates validated plan-only installation JSON and has seven passing Python tests. EROFS inspection confirmed the planner files, launcher, and hardware scanner in the latest image. The image boots to Hyprland and the direct-kernel screenshot shows the Welcome screen with the Installation Planner action. QEMU keyboard automation has not yet opened that planner dialog. It performs no disk writes. A complete privileged installer and safe disk-operation integration are still pending.

## Mark Compatibility Platform

- Phase 1 source of the Mark Compatibility Platform (section XVIII): Windows runtime manager (`mark-runtime`), PE inspector (`mark-pe-inspect`), prefix manager (`mark-prefix`) and application launcher (`mark-win-launch`); Android APK inspector (`mark-apk-inspect`), package manager (`mark-apk`) and container manager (`mark-android-manager`); Mark Compatibility Center runtime tab wired to the real runtime tool.
- Current progress: runtime detection, APK ABI inspection and Android lifecycle tooling are implemented in source and pass `tests/validate_source.py`. The reproducible Linux build for the MCP tools is wired through `scripts/build/build-mcp-windows.sh` and runs in the isolated Arch build root.

## Later phases

See the master prompt for the intended performance manager, software center, extension manager, Windows VM experience, security review and release gates. Each remains unimplemented until code and tests exist.
