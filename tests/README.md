# Test strategy

Phase gates require focused tests. The standard-library source validator is read-only and runs on Windows with Python 3.11:

```powershell
py -3.11 tests\validate_source.py
```

The read-only Rust hardware scanner is tested and built for Linux inside the isolated Arch build root:

```powershell
wsl -d MarkLinuxBuildTools -u root -- bash /mnt/h/LinuxOSbykrlmrk/scripts/build/build-hardware-detect.sh
```

The script runs rustfmt check and fixture-based Linux unit tests, then builds the release CLI under `builds/artifacts/`. Windows has no Visual C++ `link.exe`, so the Linux toolchain is used for executable tests and builds.

The Python validator checks version identity, package manifest consistency, JSON/TOML/Python syntax, the four motion-profile schemas, required live commands and root lockdown. When Git Bash is installed, it also runs `bash -n` on the build and live-session scripts. Installer plan tests run with `py -3.11 -m unittest discover -s installer\tests -p test_plan.py -v`; they cover target/path validation, privacy and read-only disk enumeration. The base image has BIOS boot coverage; the desktop image has direct-kernel and Syslinux BIOS boot coverage with VGA captures. Any future disk-writing installer tests must use disposable virtual disks; never attach physical disks to the test VM.
