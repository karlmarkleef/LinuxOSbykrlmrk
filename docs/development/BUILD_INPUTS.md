# Phase 1 build inputs

## Verified Arch bootstrap

- Release: Arch Linux 2026.11.01, x86_64.
- Archive: `archlinux-bootstrap-2026.11.01-x86_64.tar.zst`.
- Official release metadata and signer details: https://archlinux.org/download/
- Official mirror: `https://fastly.mirror.pkgbuild.com/iso/2026.11.01/`.
- Expected SHA-256: `655bc99849b80859173fa39338a6c910185e952173f6a1a61526cfa34ab8339c`.
- Signature: matching `.tar.zst.sig` from the same official release path.
- Signer identity: `pierre@archlinux.org`, discovered through GnuPG WKD; expected key ID suffix `54449A5C` per Arch release metadata. The script checks both the hash and signature before extraction.
- Verification script: `scripts/build/verify-arch-bootstrap.ps1`. Download and keyring cache are under `builds/cache/` and ignored by Git.

## Build tool environment

The Windows host did not have a Linux-native ISO toolchain. The original `Debian-Recovered` WSL2 distribution was inspected, then copied to H: with explicit user approval and registered separately as `MarkLinuxBuildTools`. The copy's SHA-256 was verified; the original remains registered on C: and was not modified or removed. The isolated Arch bootstrap root lives at `/var/tmp/marklinux-bootstrap/root.x86_64` inside the clone. ArchISO 92-1 and QEMU 11.1.3-1, plus their dependencies, were installed only inside that temporary Arch root; Debian's own packages and Windows were not modified. The package list is `tools/build/bootstrap-packages.txt`, and the package record is `builds/logs/arch-toolchain-packages.txt`.

Project profiles, the verified archive, build scripts, logs and ISO artifacts are kept on H:. ArchISO's working tree uses the WSL ext4 filesystem under the disposable Arch root's `/var/tmp`, because ISO staging needs Linux filesystem semantics. That scratch area is not source or a deliverable. The build script now checks free space before extraction and removes only its marker-protected temporary workspace after a clean unmount.

`scripts/build/clone-wsl-build-distro.ps1` copied and SHA-256-checked the existing Debian WSL VHDX to `tools/wsl-build/` on H: and registered the separate `MarkLinuxBuildTools` distribution. WSL warns that C: has insufficient free space for its default swap VHDX; do not relocate or clean C: without a separate request. The local copy is gitignored.

## Image profile source

The live images use ArchISO's supported profile format. The official documentation describes `profiledef.sh`, `packages.x86_64`, `pacman.conf`, airootfs overlays and BIOS/UEFI boot modes. `os/arch-base/profiles/base/` is adapted from the installed ArchISO baseline profile and contains the minimal live system. `os/arch-base/profiles/desktop/` adds Hyprland, its desktop tools and live-session configuration. The latest ISO reached the graphical desktop in the standard BIOS-menu QEMU test. Its plan-only installer source is integrated, while the fully functional installer, native management utilities and Windows VM launcher remain later work.

- ArchISO manual and profile format: https://github.com/archlinux/archiso
- Profile documentation: https://github.com/archlinux/archiso/blob/master/docs/README.profile.rst
- `mkarchiso` reference: https://github.com/archlinux/archiso/blob/master/man/mkarchiso.1.rst

Arch is rolling. This build pins the bootstrap archive and its signature, but package versions are resolved from the live official repository. It is not claimed to be bit-for-bit reproducible until repository snapshots and every package version are pinned.
