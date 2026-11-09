from __future__ import annotations

import pathlib
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
CORE = ROOT / "installer/core"
sys.path.insert(0, str(CORE))

from partition import (  # noqa: E402
    align_sector,
    calculate_erase_disk_layout,
    calculate_swap_size,
    validate_target_disk,
    BootloaderType,
    FilesystemType,
    GPT_TYPE_ESP,
    GPT_TYPE_LINUX_ROOT_X86_64,
    GPT_TYPE_LINUX_SWAP,
)


class PartitionPlanningTests(unittest.TestCase):
    def test_align_sector(self) -> None:
        self.assertEqual(align_sector(0), 0)
        self.assertEqual(align_sector(1), 2048)
        self.assertEqual(align_sector(2047), 2048)
        self.assertEqual(align_sector(2048), 2048)
        self.assertEqual(align_sector(2049), 4096)

    def test_validate_target_disk_accepts_valid_devices(self) -> None:
        for path in ("/dev/sda", "/dev/sdb", "/dev/vda", "/dev/nvme0n1", "/dev/mmcblk0"):
            with self.subTest(path=path):
                validate_target_disk(path, 50 * 1024**3)

    def test_validate_target_disk_rejects_partitions_and_invalid_names(self) -> None:
        for bad in ("/dev/sda1", "/dev/nvme0n1p1", "/dev/mapper/root", "sda", "/dev/../sdb"):
            with self.subTest(path=bad):
                with self.assertRaises(ValueError):
                    validate_target_disk(bad, 50 * 1024**3)

    def test_validate_target_disk_rejects_read_only_and_small_disks(self) -> None:
        with self.assertRaises(ValueError):
            validate_target_disk("/dev/sda", 50 * 1024**3, is_read_only=True)

        with self.assertRaises(ValueError):
            validate_target_disk("/dev/sda", 15 * 1024**3)  # < 20 GiB

    def test_validate_target_disk_rejects_mounted_devices(self) -> None:
        active = {"/dev/sda1", "/dev/sr0"}
        with self.assertRaises(ValueError):
            validate_target_disk("/dev/sda", 50 * 1024**3, active_mount_devices=active)

        # Unrelated disk passes
        validate_target_disk("/dev/sdb", 50 * 1024**3, active_mount_devices=active)

    def test_calculate_swap_size(self) -> None:
        # Low memory: <= 4 GiB -> 4 GiB swap
        self.assertEqual(calculate_swap_size(2 * 1024**3), 4 * 1024**3)
        self.assertEqual(calculate_swap_size(4 * 1024**3), 4 * 1024**3)
        # Moderate memory: 4-8 GiB -> 2 GiB swap
        self.assertEqual(calculate_swap_size(6 * 1024**3), 2 * 1024**3)
        self.assertEqual(calculate_swap_size(8 * 1024**3), 2 * 1024**3)
        # High memory: > 8 GiB -> 0 bytes
        self.assertEqual(calculate_swap_size(16 * 1024**3), 0)

    def test_calculate_erase_disk_layout_ext4_with_swap(self) -> None:
        disk_size = 100 * 1024**3  # 100 GiB
        memory = 4 * 1024**3  # 4 GiB RAM -> 4 GiB swap
        plan = calculate_erase_disk_layout(
            disk_path="/dev/sda",
            disk_size_bytes=disk_size,
            memory_bytes=memory,
            filesystem=FilesystemType.EXT4,
            bootloader=BootloaderType.SYSTEMD_BOOT,
        )
        self.assertEqual(plan.device_path, "/dev/sda")
        self.assertEqual(plan.partition_table, "gpt")
        self.assertEqual(plan.filesystem, FilesystemType.EXT4)
        self.assertEqual(len(plan.partitions), 3)

        # Partition 1: ESP
        esp = plan.partitions[0]
        self.assertEqual(esp.number, 1)
        self.assertEqual(esp.type_guid, GPT_TYPE_ESP)
        self.assertEqual(esp.filesystem, "vfat")
        self.assertEqual(esp.mount_point, "/boot")
        self.assertEqual(esp.size_bytes, 512 * 1024 * 1024)
        self.assertIn("esp", esp.flags)

        # Partition 2: Swap
        swap = plan.partitions[1]
        self.assertEqual(swap.number, 2)
        self.assertEqual(swap.type_guid, GPT_TYPE_LINUX_SWAP)
        self.assertEqual(swap.filesystem, "swap")
        self.assertEqual(swap.size_bytes, 4 * 1024**3)

        # Partition 3: Root
        root = plan.partitions[2]
        self.assertEqual(root.number, 3)
        self.assertEqual(root.type_guid, GPT_TYPE_LINUX_ROOT_X86_64)
        self.assertEqual(root.filesystem, "ext4")
        self.assertEqual(root.mount_point, "/")
        self.assertGreater(root.size_bytes, 90 * 1024**3)

        # Verify alignment
        for part in plan.partitions:
            self.assertEqual(part.start_sector % 2048, 0)

    def test_calculate_erase_disk_layout_btrfs_no_swap(self) -> None:
        disk_size = 64 * 1024**3
        plan = calculate_erase_disk_layout(
            disk_path="/dev/nvme0n1",
            disk_size_bytes=disk_size,
            memory_bytes=16 * 1024**3,  # > 8 GiB -> 0 swap
            filesystem=FilesystemType.BTRFS,
            bootloader=BootloaderType.GRUB,
        )
        self.assertEqual(plan.device_path, "/dev/nvme0n1")
        self.assertEqual(plan.filesystem, FilesystemType.BTRFS)
        self.assertEqual(plan.bootloader, BootloaderType.GRUB)
        self.assertEqual(len(plan.partitions), 2)  # ESP + Root

        esp = plan.partitions[0]
        self.assertEqual(esp.filesystem, "vfat")
        root = plan.partitions[1]
        self.assertEqual(root.filesystem, "btrfs")

        data = plan.to_dict()
        self.assertEqual(data["filesystem"], "btrfs")
        self.assertEqual(data["bootloader"], "grub")
        self.assertEqual(len(data["partitions"]), 2)


if __name__ == "__main__":
    unittest.main()
