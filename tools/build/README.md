# Build tooling

The Phase 1 build is native to Arch userspace inside a disposable Arch bootstrap root hosted by the existing Debian WSL2 distribution. The bootstrap archive is hash- and signature-verified before use. ArchISO and QEMU are installed only in that root. See `docs/development/BUILD_INPUTS.md` for versions and the supported boundary.

The build commands are implemented in `scripts/build/`. Product configuration and logs are under this repository; temporary ISO staging is on WSL ext4. No host disk is exposed to the boot-test VM.
