# Changelog

## Mark Compatibility Platform - gaming benchmark frame statistics

- Added `tools/benchmark/frame_stats.py` with FPS, frame-time, stutter-ratio and percentile statistics for the Mark Linux gaming benchmark suite.
- Added `tools/benchmark/tests/test_frame_stats.py` with six unit tests over synthetic frame-time samples; all pass via `python3 -B -m unittest discover -s tools/benchmark/tests -v`.

## Mark Compatibility Platform - runtime, APK and lifecycle tooling

- Added the Windows runtime manager crate (`mark-runtime-manager`) with the `mark-runtime` binary under `windows/runtime-manager/`. It detects installed Wine, Wine Staging, Proton, Bottles and Winetricks runtimes through the real host `PATH` and emits text or JSON status reports. Unit tests cover runtime identifiers, binary names, labels and status shape.
- Added `scripts/build/build-mcp-windows.sh`, the Mark Compatibility Platform build script. It builds the MCP Rust tools (`mark-runtime`, `mark-pe-inspect`, `mark-apk-inspect`) in the isolated Arch build root, stages release binaries plus the Android manager/APK scripts into the desktop ISO source overlay, and invokes the desktop ISO build. Staged binaries are removed on exit.
- Added the `mark-apk-inspect` Rust crate under `android/apk-installer/` with a pure-Rust ZIP central directory reader that detects native ABI coverage (`arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`) and ARM translation requirements. Unit tests use synthetic central-directory fixtures.
- Added the `mark-android-manager` lifecycle script at `android/container-manager/mark-android-manager` (start/stop/restart/status/init/init --gapps) wrapping upstream Waydroid commands.
- Added the `mark-apk` package manager script at `android/package-manager/mark-apk` (install/list/uninstall/launch/inspect) that uses `mark-apk-inspect` for pre-installation ABI checks.
- Wired the Mark Compatibility Center runtime management tab to the real `mark-runtime` binary instead of the missing `detect-runtimes.sh`.
- Added the game compatibility database at `windows/compatibility-database/games.json` with schema version 1, classification statuses (`supported`, `partially_supported`, `unsupported`, `requires_official_developer_support`) and per-entry graphics/runtime metadata. The source validator now checks the database file, schema and entry fields.
- Added the Windows dependency manager at `windows/dependency-manager/` (`dependencies.json` manifest, `resolver.py` winetricks component resolution for Wine/Wine Staging/Proton runtimes and DirectX sets); six unit tests pass.

## Phase 3 - Adaptive hardware detection & partition engine

- Extended `scripts/build/verify-desktop-kernel.sh` with post-boot action orchestration. QEMU direct-kernel boot reached Hyprland and Welcome, successfully simulated keyboard navigation to select `Installation Planner`, launched `mark-installer` (`gui.py`), and captured the opening dialog on screen (`builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T145606Z.png`).
- Implemented an adaptive performance profile recommendation engine in Rust (`core/hardware-detect/src/lib.rs` and `main.rs`). It evaluates CPU thread concurrency, memory capacity, disk rotational type, and GPU/VRAM to recommend `Ultra Lite`, `Balanced`, or `Cinematic` profiles with human-readable explanations.
- Added `--profile` and `--profile-name` flags to `mark-hardware` CLI. 8 Rust Linux unit tests pass (`cargo test --locked --all-targets`) with clean `cargo fmt --check`. Optimized Linux binary built to `builds/artifacts/mark-hardware-x86_64-unknown-linux-gnu`.
- Integrated adaptive profile detection into `gui.py` so the installer pre-selects the recommended profile for target hardware while giving user full customization freedom.
- Implemented `installer/core/partition.py` for sector-aligned GPT partitioning (512 MiB ESP FAT32, RAM-adaptive Swap, ext4/btrfs Root) and disk-safety checks.
- Added `installer/tests/test_partition.py` (8 tests) and updated `installer/tests/test_plan.py` (8 tests). All 16 installer tests pass with Python `-B`.
- Verified `tests/validate_source.py` with 40/40 passing checks.

## 0.0.1_dev — build 1101126_1

- Built a minimal ArchISO live image with Mark Linux identity, upstream Linux kernel/firmware, initramfs and BIOS/UEFI boot files.
- BIOS QEMU boot reached the `marklinux login:` prompt; the full build log, boot log, screenshot and checksum are saved under `builds/logs/`.
- Added verified bootstrap, isolated tool preparation, guarded build and boot-verification scripts.
- Preserved earlier validation ISO attempts in `builds/archive/phase1-validation/`.
- The requested character logo was not attached. The pre-existing worksheet image was preserved and is not used as a logo.

## Phase 2 work in progress

- Added a separate ArchISO desktop profile with Hyprland, Waybar, launch/dock, notifications, terminal, file manager and settings applications; selected AMD microcode and NVIDIA open modules for the inspected host.
- Added live-user startup, Wayland/portal settings, welcome and quick-settings menus, desktop performance profiles and an original abstract wallpaper. No character logo is claimed.
- Desktop scripts passed `bash -n`; JSON and TOML configuration files parsed. The original Debian-Recovered WSL distro remains preserved on C:. With user authorization, its VHDX was copied and registered as the separate H:-hosted `MarkLinuxBuildTools` build distro.
- Corrected the AMD microcode path in Syslinux, GRUB, loopback GRUB and EFI boot entries. Direct-kernel QEMU boot reached the live `marklinux login:` prompt on `ttyS0` and showed the graphical session, Waybar and welcome dialog. That screenshot exposed an unsupported `fade` style for `windowsOut`; changed it to documented `popin 88%`, archived the prior image and rebuilt.
- Current desktop ISO at the time of this entry: 1,866,039,296 bytes; SHA-256 `93763979e64078863c97d8e3918f442fdca62595c4882f9ca7e09709bfd183c0`. It includes the Linux hardware scanner and Hardware Summary Welcome action. Standard BIOS/Syslinux boot was subsequently verified; the automated QEMU key did not activate the report row. Physical NVIDIA driver testing is not covered by QEMU TCG.

## Phase 2 verification

- Standard BIOS/Syslinux QEMU boot of the hardware-scanner ISO completed successfully. The live Hyprland session and Welcome GUI are visible in `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-boot-20261109T133207Z.png`. VM used QEMU TCG, 4 vCPU, 4 GiB RAM and CD-ROM only; no host disks were attached. UEFI firmware boot and physical NVIDIA driver operation remain untested.
- Expanded the Rust hardware scanner and installed Rust only in the isolated Arch build root. Five fixture-based Linux tests pass and the optimized Linux binary is packaged in the ISO.

## Phase 3 groundwork

- Added a plan-only Zenity installation GUI with read-only whole-disk enumeration, strict plan validation and no-overwrite private JSON output. Seven Python tests pass. It collects no password and performs no disk or boot changes. The planner source is now included in the desktop ISO.
- A privileged install backend, alongside/manual partitioning, account creation, recovery and bootloader setup remain unimplemented.

## Latest ISO verification — 2026-11-09

- Rebuilt `builds/iso/marklinux-0.0.1_dev_1101126_1-x86_64.iso` with the planner source: 1,866,039,296 bytes, SHA-256 `896ac53fb985162ffa110837e11cf438ca5a796207202320daf375a4baffc3c6`. The `.sha256` sidecar matches the Windows-computed hash.
- Source validation passed; all seven installer tests passed. BIOS/Syslinux QEMU boot exited 0 and a 1280×800 screenshot shows the live Hyprland desktop. VM used QEMU TCG, 4 vCPU, 4 GiB RAM, CD-ROM only, with no host disks passed through. The planner GUI itself was not manually exercised in this test. UEFI firmware boot and physical NVIDIA driver operation remain untested.
- `dump.erofs` confirmed the installer GUI files and launcher plus the hardware scanner in the image. `fsck.erofs --path` extracted the planner GUI source successfully for a packaging check; temporary inspection files were removed.
- Extended the QEMU verifier with a no-boot-key mode and a planner-open attempt. The VM again reached Hyprland but keyboard automation did not reveal Welcome or the planner (`...boot-20261109T140933Z.png`); this remains a failed GUI interaction check. A shorter 115-second capture was black and is not treated as a successful boot.
- Direct-kernel boot of the same ISO ran for 180 seconds and visibly showed the Welcome window with the Installation Planner option (`builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T142103Z.png`); serial output reached the live login prompt. The planner option was not selected. This confirms desktop welcome packaging and startup, not the wizard flow.
- Removed test-generated Python bytecode caches from the desktop source overlay, ignored `__pycache__`/`.pyc` in Git, and added a source-validator check to prevent such generated caches entering future ISO builds. Installer tests run with Python `-B` to keep the overlay clean.
- Rebuilt the ISO after cache cleanup. See the newer build verification entry below for the current hash and boot results.

## Clean-overlay ISO rebuild — 2026-11-09

- Archived the prior image and checksum under `builds/archive/desktop-before-cache-cleanup/` before rebuilding.
- Build completed with ArchISO; 1,866,039,296 bytes, SHA-256 `f8d20ad392f7dd423cae2f49c5e70ed7ccc360d3fbc0a35ce9f034064f9b294f`. Windows-computed SHA-256 matched the sidecar.
- Package resolution selected Hyprland 0.57.2-1 and Python 3.14.9-1. This rolling-repository build is not bit-for-bit reproducible.
- `fsck.erofs` reported no errors. `dump.erofs` confirmed planner files and launcher plus hardware scanner; the planner directory contains only `gui.py` and `plan.py` with no Python cache.
- BIOS/Syslinux QEMU boot completed with exit status 0 and reached the live Hyprland desktop. VM used QEMU TCG, 4 vCPU, 4 GiB RAM, CD-ROM only, no host disks passed through. Screenshot: `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-boot-20261109T143629Z.png`. The Welcome dialog was not shown in this BIOS-mode capture. UEFI and physical NVIDIA operation remain untested.

## Mark Compatibility Platform - ISO boot verification and dependency manifest staging

- Verified the current desktop ISO `builds/iso/marklinux-0.0.1_dev_1101126_1-x86_64.iso` end to end: SHA-256 matched the `.sha256` sidecar (`f8d20ad392f7dd423cae2f49c5e70ed7ccc360d3fbc0a35ce9f034064f9b294f`), and no boot error was found in the QEMU phase 1/2 BIOS, desktop or direct-kernel captures. The latest direct-kernel serial capture reached the `marklinux login:` prompt and the live desktop, and the final phase 1 build log reported successful package install, Syslinux BIOS setup and GRUB UEFI setup.
- Added the desktop ISO overlay dependency manifest at `os/arch-base/profiles/desktop/airootfs/usr/share/marklinux/mcp/dependencies.json` (schema version 1, runtime winetricks components and DirectX sets such as `dxvk-d3d9`, `dxvk-d3d11`, `vkd3d-d3d12`) so `mark-win-launch --deps-set` resolves the manifest shipped inside the image via `/usr/share/marklinux/mcp/dependencies.json`.
- Extended `tests/validate_source.py` with overlay checks that the MCP dependency manifest exists and declares a valid schema version and dependency sets.


## Clean-overlay ISO rebuild — 2026-11-09

- Archived the prior image and checksum under `builds/archive/desktop-before-cache-cleanup/` before rebuilding.
- Build completed with ArchISO; 1,866,039,296 bytes, SHA-256 `f8d20ad392f7dd423cae2f49c5e70ed7ccc360d3fbc0a35ce9f034064f9b294f`. Windows-computed SHA-256 matched the sidecar.
- Package resolution selected Hyprland 0.57.2-1 and Python 3.14.9-1. This rolling-repository build is not bit-for-bit reproducible.
- `fsck.erofs` reported no errors. `dump.erofs` confirmed planner files and launcher plus hardware scanner; the planner directory contains only `gui.py` and `plan.py` with no Python cache.
- BIOS/Syslinux QEMU boot completed with exit status 0 and reached the live Hyprland desktop. VM used QEMU TCG, 4 vCPU, 4 GiB RAM, CD-ROM only, no host disks passed through. Screenshot: `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-boot-20261109T143629Z.png`. The Welcome dialog was not shown in this BIOS-mode capture. UEFI and physical NVIDIA operation remain untested.

## Phase 3 groundwork

- Added a read-only Rust hardware inventory library and CLI source for Linux procfs/sysfs. No device writes or external commands are used.
- `cargo check --offline --all-targets` and `cargo fmt --check` passed. `cargo test` could not link because MSVC `link.exe` is unavailable; no unit tests ran. Linux runtime verification and installer integration remain pending.

## Phase 0 foundation

- Recorded Windows hardware/toolchain and initialized Git.
- Added architecture, security, branding, installation and development documentation.

## Mark Linux installer - real installation reliability

- Fixed `Installer.chroot_run()` to accept and forward the `check` keyword argument used by `_configure_installed_system` and `_verify`.
- Reordered `_install_bootloader()` so EFI and BIOS GRUB are installed before `grub-mkconfig`, and create `/boot/grub` first.
- Recreate `/tmp` and `/var/tmp` with mode `1777` after the rootfs copy; the copy excludes them, and their absence made `mkinitcpio -P` fail.
- Use the available `lat9w-16` console font instead of the missing `eVGA8x16`.
- Force target `userdel`/`groupdel` removal of the live `mark` account with `--force` so a busy UID cannot block account recreation.
- Default the installer full-name prompt to `Mark` instead of the performance profile name.
- Verified the backend now reaches `Installation finished` logic and packaged the fixes in the rebuilt desktop ISO (`marklinux-0.0.1_dev_1101126_1-x86_64.iso`, SHA-256 `05c60d54e4726d84edc65b9e132982f1e554d82a91b414d45ed3ee6c4ed71eba`).
- Source validation and all 22 installer unit tests pass.
