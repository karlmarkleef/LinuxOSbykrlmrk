from __future__ import annotations

import pathlib
import sys
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "installer/core"))

from install import (  # noqa: E402
    InstallConfig,
    partition_path,
    render_fstab,
    sgdisk_commands,
    validate_install_config,
)
from partition import FilesystemType, calculate_erase_disk_layout  # noqa: E402


class InstallHelpersTests(unittest.TestCase):
    def test_partition_path_handles_common_disk_names(self) -> None:
        self.assertEqual(partition_path("/dev/sda", 1), "/dev/sda1")
        self.assertEqual(partition_path("/dev/vda", 2), "/dev/vda2")
        self.assertEqual(partition_path("/dev/nvme0n1", 3), "/dev/nvme0n1p3")
        self.assertEqual(partition_path("/dev/mmcblk0", 4), "/dev/mmcblk0p4")

    def test_fstab_uses_uuid_entries(self) -> None:
        text = render_fstab("root-id", "esp-id", "ext4", "swap-id")
        self.assertIn("UUID=root-id\t/\text4\tdefaults,noatime", text)
        self.assertIn("UUID=esp-id\t/boot\tvfat", text)
        self.assertIn("UUID=swap-id\tnone\tswap", text)

    def test_sgdisk_plan_contains_bios_boot_and_verification(self) -> None:
        layout = calculate_erase_disk_layout(
            "/dev/vda", 40 * 1024**3, memory_bytes=16 * 1024**3,
            filesystem=FilesystemType.EXT4, include_swap=False,
            include_bios_boot=True,
        )
        commands = sgdisk_commands(layout)
        self.assertEqual(commands[0], ["wipefs", "-a", "/dev/vda"])
        self.assertTrue(any(
            any("MarkLinux-BIOS" in argument for argument in command)
            for command in commands
        ))
        self.assertEqual(commands[-2], ["sgdisk", "-v", "/dev/vda"])

    def test_validation_rejects_bad_installer_values(self) -> None:
        valid = InstallConfig(
            disk_path="/dev/vda", disk_size_bytes=40 * 1024**3,
            username="mark", full_name="Mark User", timezone="UTC",
            keyboard_layout="us", performance_profile="Balanced",
            filesystem="ext4", include_swap=True, hostname="marklinux",
        )
        with self.assertRaises(ValueError):
            validate_install_config(valid, pathlib.Path("/definitely/missing"))
        self.assertEqual(valid.username, "mark")


if __name__ == "__main__":
    unittest.main()
