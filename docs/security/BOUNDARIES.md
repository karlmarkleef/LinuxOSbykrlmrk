# Security and data boundaries

- Project source, inputs, build logs and ISO artifacts belong under `H:\LinuxOSbykrlmrk`. The Arch build toolchain and scratch area are inside the separately registered `MarkLinuxBuildTools` clone on H:, under WSL `/var/tmp`; the original C:-hosted Debian package set and Windows installation were not modified.
- The current installation planner is read-only with respect to all disks. It only creates a user-selected JSON plan file and does not elevate or partition, format, install, create users, or modify boot configuration.
- No host partitioning, formatting, boot changes, registry changes or driver installation are part of this work.
- QEMU boot checks use the ISO as a CD-ROM and do not pass through physical disks.
- The Phase 1 live profile has no SSH service enabled and does not automatically mount host or guest data volumes.
- The Rust scanner reads Linux procfs/sysfs metadata only and never opens block-device contents or writes to storage.
- Verify package signatures and checksums and retain build logs before release.
- Do not run downloaded scripts without reviewing source and provenance.
- Secure Boot, persistent recovery, a privileged installer backend, disk-operation recovery and host installer isolation are not implemented or claimed yet.
