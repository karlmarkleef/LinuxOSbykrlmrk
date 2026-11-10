# Mark Linux installer

The live ISO now includes a real full-disk installer. The Zenity front end selects a writable whole disk, collects account and regional settings, requires the user to type `ERASE`, and invokes the privileged backend through `pkexec`.

The backend creates a GPT layout with an ESP, optional swap, a BIOS GRUB partition, and an ext4/btrfs root partition. It formats and mounts the target, copies the live root filesystem offline, creates the selected account, writes `fstab`, rebuilds initramfs, installs GRUB for UEFI and legacy BIOS, and verifies the installed kernel and boot configuration. The disk listing uses `lsblk --nodeps` and the backend refuses mounted disks and the live medium.

The installer asks for the new account password only after the target is selected. It is sent to the backend over the local process pipe and is never written to a plan or log.

This release supports full-disk erase installation. Alongside-Windows and manual partitioning remain intentionally unavailable; the GUI clearly labels the destructive full-disk operation.

## Components and Tests

- `installer/core/partition.py`: Sector-aligned GPT partition layout calculator (512 MiB ESP, optional 1 MiB BIOS boot partition, RAM-adaptive Swap, ext4/btrfs Root) and disk-safety validation engine.
- `installer/core/install.py`: Destructive, safety-gated offline rootfs installer and dual-mode GRUB setup.
- `installer/tests/test_partition.py`: 8 unit tests validating sector alignment, partition sizing, filesystem flags, and mount/read-only guardrails.
- `installer/tests/test_plan.py`: 8 unit tests covering plan schema, adaptive profile detection via `mark-hardware`, and disk enumeration safety.
- All 16 installer tests pass under Python 3.11 (`py -3.11 -B -m unittest discover -s installer/tests -v`).

The planner files and executable wrapper are packaged in the ISO. In QEMU direct-kernel testing, headless automation successfully navigated the Welcome dialog and opened the Installation Planner wizard (`builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T145606Z.png`).
