"""Mark Linux partition planning and disk safety validation engine.

Computes sector-aligned GPT partition tables and validates device safety.
Never writes to disks; produces structured layout plans for inspection and execution.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


MINIMUM_DISK_BYTES = 20 * 1024**3  # 20 GiB planning minimum
ESP_SIZE_BYTES = 512 * 1024 * 1024  # 512 MiB EFI System Partition
BIOS_BOOT_SIZE_BYTES = 1024 * 1024  # 1 MiB BIOS boot partition for GRUB i386-pc
SECTOR_SIZE = 512
ALIGNMENT_SECTORS = 2048  # 1 MiB alignment boundary
GPT_HEADER_SECTORS = 34  # Primary GPT header + partition table sectors
GPT_BACKUP_SECTORS = 33  # Backup GPT sectors at end of disk

GPT_TYPE_ESP = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b"
GPT_TYPE_BIOS_BOOT = "21686148-6449-6e6f-744e-656564454649"
GPT_TYPE_LINUX_ROOT_X86_64 = "4f68bce3-e8cd-4db1-96e7-f311fe641777"
GPT_TYPE_LINUX_SWAP = "0657fd6d-a4ab-43c4-84e5-0933c84b4f4f"

DEVICE_PATTERN = re.compile(
    r"^/dev/(?:sd[a-z]+|vd[a-z]+|xvd[a-z]+|nvme[0-9]+n[0-9]+|mmcblk[0-9]+)$"
)


class FilesystemType(str, Enum):
    EXT4 = "ext4"
    BTRFS = "btrfs"


class BootloaderType(str, Enum):
    SYSTEMD_BOOT = "systemd-boot"
    GRUB = "grub"


class InstallMode(str, Enum):
    ERASE_DISK = "erase_disk"
    ALONGSIDE = "alongside"
    MANUAL = "manual"


@dataclass
class PartitionPlan:
    number: int
    name: str
    type_guid: str
    start_sector: int
    end_sector: int
    size_bytes: int
    filesystem: str
    mount_point: str
    flags: list[str] = field(default_factory=list)

    @property
    def size_gib(self) -> float:
        return round(self.size_bytes / (1024**3), 2)


@dataclass
class DiskLayoutPlan:
    device_path: str
    disk_size_bytes: int
    partition_table: str
    partitions: list[PartitionPlan]
    filesystem: FilesystemType
    bootloader: BootloaderType
    swap_enabled: bool
    swap_size_bytes: int
    unallocated_bytes: int

    def to_dict(self) -> dict[str, Any]:
        return {
            "device_path": self.device_path,
            "disk_size_bytes": self.disk_size_bytes,
            "disk_size_gib": round(self.disk_size_bytes / (1024**3), 2),
            "partition_table": self.partition_table,
            "filesystem": self.filesystem.value,
            "bootloader": self.bootloader.value,
            "swap_enabled": self.swap_enabled,
            "swap_size_bytes": self.swap_size_bytes,
            "unallocated_bytes": self.unallocated_bytes,
            "partitions": [
                {
                    "number": p.number,
                    "name": p.name,
                    "type_guid": p.type_guid,
                    "start_sector": p.start_sector,
                    "end_sector": p.end_sector,
                    "size_bytes": p.size_bytes,
                    "size_gib": p.size_gib,
                    "filesystem": p.filesystem,
                    "mount_point": p.mount_point,
                    "flags": p.flags,
                }
                for p in self.partitions
            ],
        }


def validate_target_disk(
    disk_path: str,
    disk_size_bytes: int,
    is_read_only: bool = False,
    active_mount_devices: set[str] | None = None,
) -> None:
    """Validate that target device is a writable whole disk not in active system use."""
    if not isinstance(disk_path, str) or not DEVICE_PATTERN.fullmatch(disk_path):
        raise ValueError(f"Invalid target disk path: {disk_path}. Must be a whole block device.")

    if isinstance(disk_size_bytes, bool) or not isinstance(disk_size_bytes, int):
        raise TypeError("Disk size must be an integer number of bytes.")

    if disk_size_bytes < MINIMUM_DISK_BYTES:
        raise ValueError(
            f"Disk size ({disk_size_bytes / 1024**3:.1f} GiB) is smaller than the required 20 GiB minimum."
        )

    if is_read_only:
        raise ValueError(f"Target disk {disk_path} is flagged read-only and cannot be modified.")

    if active_mount_devices:
        for mounted in active_mount_devices:
            if mounted == disk_path or mounted.startswith(disk_path):
                raise ValueError(
                    f"Target disk {disk_path} or one of its partitions ({mounted}) is currently mounted."
                )


def align_sector(sector: int, alignment: int = ALIGNMENT_SECTORS) -> int:
    """Round up sector to alignment boundary."""
    remainder = sector % alignment
    return sector if remainder == 0 else sector + (alignment - remainder)


def calculate_swap_size(memory_bytes: int | None) -> int:
    """Calculate recommended swap partition size based on system RAM."""
    if memory_bytes is None or memory_bytes <= 0:
        return 2 * 1024**3  # Default 2 GiB

    four_gib = 4 * 1024**3
    eight_gib = 8 * 1024**3

    if memory_bytes <= four_gib:
        return 4 * 1024**3  # 4 GiB swap for <= 4 GiB RAM
    elif memory_bytes <= eight_gib:
        return 2 * 1024**3  # 2 GiB swap for 4-8 GiB RAM
    else:
        return 0  # Systems with > 8 GiB RAM can use zram or optional swapfile


def calculate_erase_disk_layout(
    disk_path: str,
    disk_size_bytes: int,
    memory_bytes: int | None = None,
    filesystem: FilesystemType = FilesystemType.EXT4,
    bootloader: BootloaderType = BootloaderType.SYSTEMD_BOOT,
    include_swap: bool = True,
    include_bios_boot: bool = True,
    active_mount_devices: set[str] | None = None,
) -> DiskLayoutPlan:
    """Calculate a clean GPT partition layout for full-disk erase installation.

    Calculates:
    - Partition 1: ESP (512 MiB FAT32), type c12a7328-f81f-11d2-ba4b-00a0c93ec93b, mount /boot
    - Partition 2 (optional): BIOS boot (1 MiB), type 21686148-6449-6e6f-744e-656564454649
    - Partition 2/3 (optional): Swap, type 0657fd6d-a4ab-43c4-84e5-0933c84b4f4f
    - Partition 3/4: Root (ext4/btrfs), type 4f68bce3-e8cd-4db1-96e7-f311fe641777, mount /

    The BIOS boot partition lets the same disk boot on legacy-BIOS firmware
    through GRUB i386-pc while the ESP keeps UEFI boot working.
    """
    validate_target_disk(
        disk_path,
        disk_size_bytes,
        is_read_only=False,
        active_mount_devices=active_mount_devices,
    )

    total_sectors = disk_size_bytes // SECTOR_SIZE
    usable_end_sector = total_sectors - GPT_BACKUP_SECTORS - 1

    partitions: list[PartitionPlan] = []
    part_num = 1

    # Partition 1: ESP (512 MiB)
    esp_start = align_sector(GPT_HEADER_SECTORS, ALIGNMENT_SECTORS)
    esp_sectors = ESP_SIZE_BYTES // SECTOR_SIZE
    esp_end = esp_start + esp_sectors - 1
    partitions.append(
        PartitionPlan(
            number=part_num,
            name="MarkLinux-ESP",
            type_guid=GPT_TYPE_ESP,
            start_sector=esp_start,
            end_sector=esp_end,
            size_bytes=esp_sectors * SECTOR_SIZE,
            filesystem="vfat",
            mount_point="/boot",
            flags=["boot", "esp"],
        )
    )
    part_num += 1

    current_sector = align_sector(esp_end + 1, ALIGNMENT_SECTORS)

    # Optional BIOS boot partition (1 MiB) for GRUB's i386-pc target.
    if include_bios_boot:
        bios_sectors = BIOS_BOOT_SIZE_BYTES // SECTOR_SIZE
        bios_end = current_sector + bios_sectors - 1
        partitions.append(
            PartitionPlan(
                number=part_num,
                name="MarkLinux-BIOS",
                type_guid=GPT_TYPE_BIOS_BOOT,
                start_sector=current_sector,
                end_sector=bios_end,
                size_bytes=bios_sectors * SECTOR_SIZE,
                filesystem="bios-boot",
                mount_point="none",
                flags=["bios_grub"],
            )
        )
        part_num += 1
        current_sector = align_sector(bios_end + 1, ALIGNMENT_SECTORS)

    # Swap partition (optional)
    swap_size_bytes = calculate_swap_size(memory_bytes) if include_swap else 0
    if swap_size_bytes > 0:
        swap_sectors = swap_size_bytes // SECTOR_SIZE
        swap_end = current_sector + swap_sectors - 1
        partitions.append(
            PartitionPlan(
                number=part_num,
                name="MarkLinux-Swap",
                type_guid=GPT_TYPE_LINUX_SWAP,
                start_sector=current_sector,
                end_sector=swap_end,
                size_bytes=swap_sectors * SECTOR_SIZE,
                filesystem="swap",
                mount_point="none",
                flags=["swap"],
            )
        )
        part_num += 1
        current_sector = align_sector(swap_end + 1, ALIGNMENT_SECTORS)

    # Root Partition: Remaining usable disk space
    if current_sector >= usable_end_sector:
        raise ValueError("Insufficient disk space to allocate root partition.")

    root_end = usable_end_sector
    root_sectors = root_end - current_sector + 1
    root_bytes = root_sectors * SECTOR_SIZE

    if root_bytes < 10 * 1024**3:  # Root must have at least 10 GiB
        raise ValueError(
            f"Calculated root partition ({root_bytes / 1024**3:.1f} GiB) is smaller than the 10 GiB minimum."
        )

    partitions.append(
        PartitionPlan(
            number=part_num,
            name="MarkLinux-Root",
            type_guid=GPT_TYPE_LINUX_ROOT_X86_64,
            start_sector=current_sector,
            end_sector=root_end,
            size_bytes=root_bytes,
            filesystem=filesystem.value,
            mount_point="/",
            flags=[],
        )
    )

    allocated_bytes = sum(p.size_bytes for p in partitions)
    unallocated_bytes = disk_size_bytes - allocated_bytes

    return DiskLayoutPlan(
        device_path=disk_path,
        disk_size_bytes=disk_size_bytes,
        partition_table="gpt",
        partitions=partitions,
        filesystem=filesystem,
        bootloader=bootloader,
        swap_enabled=swap_size_bytes > 0,
        swap_size_bytes=swap_size_bytes,
        unallocated_bytes=unallocated_bytes,
    )
