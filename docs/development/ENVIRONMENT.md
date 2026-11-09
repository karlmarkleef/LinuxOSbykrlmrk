# Development environment record

Inspection date: 2026-11-09

## Host and project storage

- OS: Windows 10 IoT Enterprise LTSC 2024 Evaluation, version 24H2, build 26100.9725.
- Project root: H:\LinuxOSbykrlmrk.
- H: accessible; PowerShell reported 363.17 GB free and 98.59 GB used.
- Existing project content before Phase 0: assets/branding/original/mark-linux-original.jpg only.
- Git repository: none before Phase 0; initialized on branch main.
- No files existed at the planned Phase 0 metadata/document paths before creation.

## Available commands

| Tool | Detected version/state |
|---|---|
| Git | 2.56.0.windows.4 |
| Rust compiler | 1.99.1, x86_64-pc-windows-msvc |
| Cargo | 1.99.1 |
| Installed Rust targets | x86_64-pc-windows-msvc |
| Python | 3.11.9 |
| CMake | 4.5.0-rc2 |
| MSYS2 GCC | 16.2.0 |
| WSL distributions | Debian-Recovered (original, preserved on C:) and MarkLinuxBuildTools (separate build clone on H:) |
| QEMU | Not found on PATH |
| Ninja | Not found on PATH |
| Docker / Podman | Not found on PATH |
| make / clang / isoinfo | Not found on PATH |

Command lookup is not a complete inventory of every installed application. No system-wide software was installed in Phase 0.

## Build implications

- Rust code can be compiled for Windows, but there is no installed Linux Rust target or validated Linux cross-toolchain.
- The ArchISO and QEMU tools are installed only inside a temporary Arch root hosted by MarkLinuxBuildTools. Its VHDX is a verified copy under H:\\LinuxOSbykrlmrk\\tools\\wsl-build; Debian-Recovered remains separately registered and unchanged on C:.
- Windows QEMU was not on PATH; QEMU was used inside the isolated Arch root for BIOS-only VM tests with no physical disks attached.

## Phase 1 recheck — 2026-11-09

- Hardware read from Windows CIM: AMD Ryzen 5 5600X (6 cores/12 logical processors), NVIDIA GeForce RTX 2060 SUPER, 23.9 GiB RAM.
- H: remained accessible with 359.9 GiB free in a later check.
- Debian-Recovered was launched read-only for inspection, then used as the WSL2 host for a temporary Arch bootstrap root. Debian's own packages were not installed or changed.
- ArchISO, GRUB and QEMU are available inside the temporary Arch root only; they are not installed on the Windows host PATH.
- BIOS boot-test VM ran under QEMU TCG with the ISO as CD-ROM and no physical disk passthrough.
- The first desktop ISO build failed while extracting packages. C: fell below 50 MiB free and Debian-Recovered stopped starting. With user approval, its VHDX was copied to H:, SHA-256 verified and registered as MarkLinuxBuildTools; the original remains on C: untouched. WSL warns that low C: free space prevents creating its default swap VHDX, but the build clone starts and QEMU works under TCG.
- The latest desktop ISO has been built on H:. Direct-kernel QEMU reached the `marklinux login:` prompt and the Hyprland session, Waybar and Welcome window appeared in `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T142103Z.png`; the Installation Planner entry is visible but was not selected. A separate BIOS-menu QEMU boot reached the live desktop at `builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-boot-20261109T135601Z.png`. QEMU TCG does not test the host's physical NVIDIA driver. Interactive planner verification remains outstanding.
