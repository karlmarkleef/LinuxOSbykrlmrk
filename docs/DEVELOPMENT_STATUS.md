# Development status

Updated: 2026-11-09  
Project: Mark Linux 0.0.1_dev - build 1101126_1

## Mark Compatibility Platform (section XVIII) - current status

**Status: Phase 1 source implemented; runtime, APK and lifecycle tooling present, real build pending.**

The Mark Compatibility Platform (MCP) reuses upstream Wine, Proton, Waydroid and Android userspace instead of reimplementing Windows or Android compatibility layers.

### Implemented

- Windows runtime manager (`mark-runtime`) detects installed Wine, Wine Staging, Proton, Bottles and Winetricks runtimes via the host `PATH` and reports text or JSON status.
- Mark Compatibility Platform build script (`scripts/build/build-mcp-windows.sh`) builds and stages the MCP Windows tools into the desktop ISO overlay and invokes the desktop ISO build.
- `mark-apk-inspect` (Rust) inspects APK ZIP central directories for native ABI coverage and ARM translation requirements.
- `mark-android-manager` (bash) wraps upstream Waydroid lifecycle commands.
- `mark-apk` (bash) wraps Waydroid app install/list/uninstall/launch plus `mark-apk-inspect` pre-installation checks.
- Mark Compatibility Center runtime tab is wired to the real `mark-runtime` binary.

### Limitations and next steps

- Runtime detection and APK inspection are source-complete but the full Linux build and unit test cycle still runs in the isolated Arch build root.
- Gaming graphics layers (DXVK, VKD3D-Proton, Vulkan) configuration, game-specific compatibility database and benchmark suite remain in later phases.
- Creative (Adobe) profiles and Android window/GPU/audio bridges remain in later phases.

## Phase 0 - Environment and project foundation

**Status: complete.** Project root is `H:\LinuxOSbykrlmrk`. It existed before this work and contained one JPEG asset; it was inspected and preserved unchanged.

- Confirmed Windows 10 IoT Enterprise LTSC 2024 Evaluation, 24H2, build 26100.9725.
- Rechecked the machine hardware: AMD Ryzen 5 5600X (6 cores/12 threads), NVIDIA GeForce RTX 2060 SUPER, 23.9 GiB RAM. H: had 359.9 GiB free at the Phase 2 preparation check.
- Recorded development tools, initialized Git on `main`, and added project metadata and technical documentation.
- Existing `assets/branding/original/mark-linux-original.jpg` SHA-256 is `40FDB4254724758E5C222E78CAD0B0CE677C8AF2CDEC7EBCE33E89D18207C571`. Visual inspection shows a photographed Vietnamese math worksheet, not the described character logo. It remains unmodified and is not used as Mark Linux branding. The prompt attachment contains text only, with no separate image.

## Phase 1 — Base OS

**Status: complete for a minimal live boot image.**

### Implemented

- Verified Arch Linux bootstrap 2026.11.01 x86_64 by the pinned SHA-256 and detached OpenPGP signature using the release signer identity from the official Arch download page.
- Prepared an isolated Arch build root under a separate `MarkLinuxBuildTools` WSL2 distribution on H:. Its ext4 VHDX is at `H:\\LinuxOSbykrlmrk\\tools\\wsl-build\\ext4.vhdx`; the existing `Debian-Recovered` distribution remains registered and unchanged on C:. Installed ArchISO 92-1, GRUB 2:2.16-1, and QEMU 11.1.3-1 inside the isolated build root. Windows and Debian's system package set were not changed.
- Added the ArchISO profile, Mark Linux `/etc/os-release`, live hostname `marklinux`, minimal package list, and BIOS Syslinux plus UEFI GRUB boot support. Excluded SSH/cloud-init/VM guest service links when those packages were removed.
- Built and BIOS-boot-tested `marklinux-0.0.1_dev_1101126_1-x86_64.iso`: 911,552,512 bytes (870 MiB), SHA-256 `e58477083846883ed818721946c45d44b2c7484353dd75c3f4ca48c5739cd98d`. The preserved image and adjacent checksum are under `builds/archive/phase1-complete/`; the original build log is under `builds/logs/`.
- Inspected the ISO boot catalog with xorriso: hybrid MBR/GPT, BIOS El Torito entry, and EFI El Torito image are present. BIOS boot was tested with QEMU TCG (2 vCPU, 2 GiB RAM, CD-ROM only, no host disk passthrough); the live system reached `marklinux login:` on `tty1`. Screenshot: `builds/logs/phase1-boot-20261109T101415Z.png`; raw QEMU capture and full log are alongside it.
- Archived the earlier validation iterations under `builds/archive/phase1-validation/` rather than overwriting them.

### Build and test evidence

- Build: `wsl -d Debian-Recovered -u root -- bash /mnt/h/LinuxOSbykrlmrk/scripts/build/build-base-iso.sh`
- ISO verification and QEMU BIOS boot: `wsl -d Debian-Recovered -u root -- bash /mnt/h/LinuxOSbykrlmrk/scripts/build/verify-base-iso.sh /mnt/h/LinuxOSbykrlmrk/builds/archive/phase1-complete/marklinux-0.0.1_dev_1101126_1-x86_64.iso`
- Shell syntax checks passed for the build scripts and ArchISO profile.
- UEFI GRUB setup completed as part of the ISO build and its EFI image is present in the ISO catalog. An actual UEFI firmware boot has not yet been tested.

### Limitations and warnings

- This is a console-only live base. Hyprland, Waybar, launcher, desktop apps, GUI installer, user-session flow, performance manager, Windows launcher and update system are not implemented yet.
- The exact requested logo was not attached. The unrelated existing worksheet image is preserved but not used. Fastfetch and graphical branding are pending the correct source art.
- Arch packages resolve from the current official rolling repository; the bootstrap archive is pinned, but repository snapshots/package versions are not. The image is not claimed bit-for-bit reproducible.
- mkinitcpio reported optional firmware omissions for several uncommon devices and uses `cp` instead of optional `pv` when copying the live system to RAM. GRUB emitted an SBAT-level mismatch warning; UEFI catalog generation completed. Secure Boot signing is not configured or claimed.
- This BIOS test verifies the image boot path in QEMU only; it does not prove installation or the host's physical firmware configuration.

## Phase 2 — Desktop

**Status: complete for a boot-tested QEMU live desktop.** The detected host is a Ryzen 5 5600X and RTX 2060 SUPER system. The desktop ArchISO profile includes Hyprland, Waybar, launcher, dock, notification service, terminal, file manager, network/audio/display/Bluetooth tools, AMD microcode, and NVIDIA open modules. Live-user startup, desktop settings, welcome and quick-settings menus, motion profiles, and an abstract wallpaper are present in source. The original JPG is a worksheet photo and is not used as a logo.

- The desktop package list resolves from Arch's current repository; the resolver selected Hyprland 0.57.1-1, Waybar 0.16.0-1, NVIDIA open modules 615.78.08-4, and AMD microcode 20261019-1 for the current build.
- Shell syntax passed for the desktop scripts and profile, and JSON/TOML configuration files parsed successfully.
- Added and ran `py -3.11 -B tests/validate_source.py`: all checks passed for both image identities, package manifests, parsed desktop JSON/TOML, motion-profile schemas, locked root password, and required live commands. Added build-storage preflight checks and guarded cleanup for the task-owned ArchISO work directory.
- The first full desktop ISO attempt failed during package extraction when C: fell to 17 MiB free and the original Debian-Recovered WSL2 could no longer start. With explicit authorization, the original VHDX was copied to H:, its SHA-256 verified, and the copy was registered separately as `MarkLinuxBuildTools`; the original was left in place. The clone has about 946 GiB free inside its virtual filesystem. WSL reports that it cannot create its default swap VHDX on C: because of the low free space, so the clone is used without modifying C: or `.wslconfig`.
- Rebuilt the desktop ISO from the H:-hosted clone after correcting the AMD microcode path in Syslinux, GRUB, loopback GRUB, and the EFI entry. The `windowsOut` animation error was fixed using the documented Hyprland `popin 88%` style. Latest ISO: `builds/iso/marklinux-0.0.1_dev_1101126_1-x86_64.iso`, 1,866,039,296 bytes, SHA-256 `896ac53fb985162ffa110837e11cf438ca5a796207202320daf375a4baffc3c6`. It includes the Linux hardware scanner, Hardware Summary launcher, and plan-only installer source.
- Standard BIOS/Syslinux QEMU boot completed with exit status 0, QEMU TCG, 4 vCPU, 4 GiB RAM, and CD-ROM only; no host disks were attached. The 1280×800 capture at `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-boot-20261109T135601Z.png` shows the live Hyprland desktop. The Welcome dialog and planner interaction were not observed in this run. QEMU TCG uses virtual display hardware; the physical RTX driver is untested.
- `dump.erofs` confirmed `/usr/share/marklinux/installer/gui.py`, `plan.py`, `/usr/local/bin/mark-installer`, and `/usr/local/bin/mark-hardware` in the current image. `fsck.erofs --path` also extracted the planner GUI source successfully into temporary build-root storage, which was removed afterward.
- A separate direct-kernel QEMU test of the same ISO ran for 180 seconds and captured the live desktop with the Welcome menu open, visibly listing Hardware Summary and Installation Planner: `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T142103Z.png`. Serial capture reached the `marklinux login:` prompt on ttyS0. The planner action itself was not selected. The VM used QEMU TCG and CD-ROM only.
- Extended `verify-base-iso.sh` with a no-boot-key option and an Installation Planner interaction attempt. The BIOS-boot QEMU run captured the desktop without Welcome or planner dialogs (`builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-boot-20261109T140933Z.png`), so that input attempt did not activate the action. The VM remained CD-ROM-only. A separate 115-second capture was black and is not counted as a successful desktop boot.
- NVIDIA's installation guidance supports open kernel modules on Turing and newer GPUs; RTX 2060 SUPER is Turing. Driver operation has not yet been tested on physical hardware.

## Phase 3 — Installer and hardware detection (in progress)

**Status: hardware inventory with adaptive profile recommendation is integrated; the GUI planner wizard has been exercised and captured in QEMU; GPT partition planning and disk-safety validation engine implemented with 16 passing unit tests.**

### Implemented

- Added adaptive performance profile recommendation engine in Rust (`core/hardware-detect/src/lib.rs` and `src/main.rs`). `recommend_profile()` evaluates CPU threads, RAM, rotational vs. SSD storage, and GPU/VRAM to recommend `Ultra Lite`, `Balanced`, or `Cinematic` with human-readable rationale. Added `--profile` and `--profile-name` CLI flags.
- Built and tested the Linux hardware scanner inside `MarkLinuxBuildTools`: 8 unit tests pass (`cargo test --manifest-path ... --locked --all-targets`) with clean formatting (`cargo fmt --check`). Native release binary produced at `builds/artifacts/mark-hardware-x86_64-unknown-linux-gnu` with SHA-256 sidecar.
- Verified QEMU GUI interaction for the Installation Planner: updated `scripts/build/verify-desktop-kernel.sh` with `MARKLINUX_QEMU_POST_BOOT_ACTION=installation-planner`. QEMU direct-kernel test booted into Hyprland, autostarted `mark-welcome`, executed `sendkey down` and `sendkey ret` to select Installation Planner, launched `mark-installer` (`gui.py`), and captured the wizard opening dialog on screen (`builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T145606Z.png`).
- Integrated adaptive profile detection into `gui.py`: queries `mark-hardware --profile-name` and automatically orders combo values to prioritize the hardware-recommended performance profile while preserving user choice.
- Implemented `installer/core/partition.py`: calculates sector-aligned (2048-sector boundary) GPT partition tables (512 MiB ESP FAT32, RAM-scaled Swap, Root ext4/btrfs) and validates target device safety. Strictly rejects partition paths, read-only disks, disks smaller than 20 GiB, and any device currently in active system mount tables.
- Added comprehensive unit tests in `installer/tests/test_partition.py` (8 tests) and updated `installer/tests/test_plan.py` (8 tests). All 16 installer tests pass under Windows Python 3.11 (`py -3.11 -B -m unittest discover -s installer/tests -v`).
- All 40 source integrity and shell syntax checks pass in `tests/validate_source.py`.

### Limitations and next steps

- A privileged backend that executes the partition layout and installs packages remains to be connected behind multi-step confirmation barriers.
- All destructive disk testing must run strictly against disposable virtual disks in QEMU, never against host physical disks.
- Proceed with Phase 4 (Performance Manager & Visual Effects toggle synchronization) and Phase 5 (Windows Virtual Experience launcher architecture).
