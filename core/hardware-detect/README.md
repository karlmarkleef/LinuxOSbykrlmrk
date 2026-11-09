# Read-only hardware inventory

`mark-hardware` reads CPU topology/model and memory from `/proc`; whole-disk model, size, read-only/rotational flags, PCI display adapter IDs/driver/optional VRAM, DRM connector status/modes/EDID, network interfaces and Bluetooth controllers from `/sys`. It does not run commands, read disk contents, change device flags, or write to storage. Disk flags are informational; an installer must independently enforce disk safety.

The Rust library separates fixture-testable parsing from live Linux scanning and has no third-party dependencies. Tests use synthetic procfs/sysfs fixtures. It includes an adaptive performance profile recommendation engine (`recommend_profile()`) that evaluates CPU thread count, RAM, storage rotational type, and GPU/VRAM to recommend `Ultra Lite`, `Balanced`, or `Cinematic`. The executable supports `--profile` and `--profile-name` flags for shell/desktop integration. The executable reports an EDID preferred monitor mode (not the compositor's current mode), and reports VRAM only when the kernel driver exports a sysfs value. It must run on Linux; the Windows build prints a clear unsupported-platform message.

Run `scripts/build/build-hardware-detect.sh` from the H:-hosted `MarkLinuxBuildTools` WSL distro. It checks formatting, runs 8 Linux unit tests, builds an optimized x86_64 Linux executable inside the isolated Arch root, and writes the binary and SHA-256 under `builds/artifacts/`.

Build artifacts belong under `builds/artifacts/`. This tool is not an installer and has no partitioning or formatting functions.
