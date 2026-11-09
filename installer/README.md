# Mark Linux installation planner

The current installer component is a graphical, plan-only preflight. It uses Zenity dialogs, the read-only `mark-hardware` scanner and `lsblk` metadata to select a whole-disk target, validate a username/time zone/keyboard/performance profile, show a summary, and save a private JSON plan to a user-selected path.

The plan is explicitly marked `plan-only`. The planner does not partition, format, install packages, create accounts, change boot configuration, elevate privileges, or write to the selected disk. It requires a 20 GiB minimum for planning and rejects partition paths, unsupported keyboard/profile values, invalid time zones and attempts to overwrite a plan file. The disk listing uses `lsblk --nodeps` and does not open disk contents.

The selected target and the `erase-selected-disk-and-install` action are future intent only. The plan warns that a future installer must rediscover and verify the physical device and ask for explicit erase confirmation. No plan password or credential is collected.

This is an early-stage installer module. Full-disk erase, alongside-Windows, manual partitioning, bootloader setup, recovery and an authorized privileged backend remain unimplemented for live execution. The GUI warns the user before it creates a plan.

## Components and Tests

- `installer/core/partition.py`: Sector-aligned GPT partition layout calculator (512 MiB ESP, RAM-adaptive Swap, ext4/btrfs Root) and disk-safety validation engine. Prohibits operations on partitions, active mounts, read-only devices, and disks under 20 GiB.
- `installer/tests/test_partition.py`: 8 unit tests validating sector alignment, partition sizing, filesystem flags, and mount/read-only guardrails.
- `installer/tests/test_plan.py`: 8 unit tests covering plan schema, adaptive profile detection via `mark-hardware`, and disk enumeration safety.
- All 16 installer tests pass under Python 3.11 (`py -3.11 -B -m unittest discover -s installer/tests -v`).

The planner files and executable wrapper are packaged in the ISO. In QEMU direct-kernel testing, headless automation successfully navigated the Welcome dialog and opened the Installation Planner wizard (`builds/logs/marklinux-0.0.1_dev_1101126_1-x86_64-direct-kernel-20261109T145606Z.png`).
