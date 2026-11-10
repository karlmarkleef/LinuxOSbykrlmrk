#!/usr/bin/env python3
"""Mark Linux real installation engine.

Partitions a target whole disk, formats it, copies the live read-only root
filesystem into it, configures users / locale / boot and verifies the result.

This module never writes to the live environment and refuses to run against
the medium it was started from. It is the privileged backend behind the
graphical and CLI installer fronts.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Iterator, Sequence

from partition import (
    BootloaderType,
    DiskLayoutPlan,
    FilesystemType,
    calculate_erase_disk_layout,
    validate_target_disk,
)


MINIMUM_DISK_BYTES = 20 * 1024**3  # 20 GiB planning minimum
REQUIRED_TARGET_ROOT = Path("/mnt")
INSTALLER_LABEL = "MarkLinux"

DEVICE_PATTERN = re.compile(r"^/dev/(?:sd[a-z]+|vd[a-z]+|xvd[a-z]+|nvme[0-9]+n[0-9]+|mmcblk[0-9]+)$")
USERNAME_PATTERN = re.compile(r"^[a-z_][a-z0-9_-]{0,31}$")
TIMEZONE_PATTERN = re.compile(r"^[A-Za-z0-9_+-]+(?:/[A-Za-z0-9_+.-]+)*$")
PERFORMANCE_PROFILES = {"Ultra Lite", "Balanced", "Cinematic", "Custom"}
PERFORMANCE_PROFILE_KEYS = {
    "Ultra Lite": "ultra-lite",
    "Balanced": "balanced",
    "Cinematic": "cinematic",
    "Custom": "custom",
}
KEYBOARD_LAYOUTS = ("us", "gb", "de", "fr", "es", "it", "pt", "br", "ru", "cn", "jp", "kr", "au", "ca")
RESERVED_USERNAMES = {
    "root", "daemon", "bin", "sys", "nobody", "sync", "games", "mail", "man",
    "lp", "proxy", "shutdown", "halt", "systemd-network", "systemd-resolve",
    "systemd-timesyncd", "systemd-journal-remote", "systemd-remount-maker",
}
REQUIRED_TOOLS = (
    "wipefs", "sgdisk", "mkfs.fat", "mkswap", "mkfs.ext4", "mkfs.btrfs",
    "tar", "cp", "rm", "findmnt", "blkid", "mount", "umount", "swapon", "swapoff",
    "chroot", "arch-chroot", "genfstab",
    "grub-install", "grub-mkconfig", "mkinitcpio", "useradd", "userdel", "groupdel", "chpasswd",
    "locale-gen", "hwclock", "systemctl", "udevadm", "blockdev", "partprobe", "sync",
)
CHROOT_ENV = {
    "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
    "HOME": "/root",
    "LANG": "C",
    "TERM": "xterm-256color",
}
LIVE_INSTALL_DIR = "marklinux"
LIVE_BOOT_RELATIVE = (
    "vmlinuz-linux",
    "initramfs-linux.img",
)


class InstallError(RuntimeError):
    """Raised when an installation step fails or a safety check triggers."""


@dataclass(frozen=True)
class InstallConfig:
    disk_path: str
    disk_size_bytes: int
    username: str
    full_name: str
    timezone: str
    keyboard_layout: str
    performance_profile: str
    filesystem: str
    include_swap: bool
    hostname: str


def _human_size(size: int) -> str:
    return f"{size / 1024**3:.1f} GiB"


def validate_install_config(config: InstallConfig, zoneinfo_root: Path = Path("/usr/share/zoneinfo")) -> None:
    """Validate installer inputs before any destructive action."""
    if not isinstance(config.disk_path, str) or not DEVICE_PATTERN.fullmatch(config.disk_path):
        raise ValueError(f"Invalid target disk path: {config.disk_path}")
    if isinstance(config.disk_size_bytes, bool) or not isinstance(config.disk_size_bytes, int):
        raise TypeError("Disk size must be an integer number of bytes.")
    if config.disk_size_bytes < MINIMUM_DISK_BYTES:
        raise ValueError(f"Target disk is smaller than the {MINIMUM_DISK_BYTES / 1024**3:.0f} GiB minimum.")
    if not isinstance(config.username, str) or not USERNAME_PATTERN.fullmatch(config.username) or config.username in RESERVED_USERNAMES:
        raise ValueError("Choose a lowercase account name starting with a letter or underscore.")
    if not isinstance(config.full_name, str) or not config.full_name.strip():
        raise ValueError("A human friendly account name is required.")
    if not isinstance(config.timezone, str) or not TIMEZONE_PATTERN.fullmatch(config.timezone):
        raise ValueError("The time zone name is invalid.")
    timezone_path = (zoneinfo_root / config.timezone).resolve()
    if not timezone_path.is_relative_to(zoneinfo_root.resolve()) or not timezone_path.is_file():
        raise ValueError("The selected time zone is not available in this live environment.")
    if config.keyboard_layout not in KEYBOARD_LAYOUTS:
        raise ValueError(f"The keyboard layout {config.keyboard_layout!r} is not supported.")
    if config.performance_profile not in PERFORMANCE_PROFILES:
        raise ValueError("The performance profile is not supported.")
    if config.filesystem not in ("ext4", "btrfs"):
        raise ValueError("The filesystem must be ext4 or btrfs.")
    if not isinstance(config.hostname, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,30}", config.hostname):
        raise ValueError("The hostname is invalid.")


def partition_path(disk_path: str, number: int) -> str:
    """Return the block device path of a numbered GPT partition."""
    stem = disk_path.removeprefix("/dev/")
    separator = "p" if stem.startswith(("nvme", "mmcblk")) else ""
    return f"/dev/{stem}{separator}{number}"


def partition_for(layout: DiskLayoutPlan, mount_point: str) -> Any:
    for partition in layout.partitions:
        if partition.mount_point == mount_point:
            return partition
    raise ValueError(f"No partition planned for mount point: {mount_point}")


def render_fstab(root_uuid: str, esp_uuid: str, root_filesystem: str, swap_uuid: str | None) -> str:
    lines = ["# <fs>       <mount>   <type>    <options>           0      1"]
    if swap_uuid:
        lines.append(f"UUID={swap_uuid}\tnone\tswap\tsw\t0\t0")
    lines.append(f"UUID={root_uuid}\t/\t{root_filesystem}\tdefaults,noatime\t0\t1")
    lines.append(f"UUID={esp_uuid}\t/boot\tvfat\tumask=0077,noatime\t0\t2")
    return "\n".join(lines) + "\n"


def render_grub_default() -> str:
    return (
        "GRUB_DEFAULT=0\n"
        "GRUB_TIMEOUT=3\n"
        'GRUB_DISTRIBUTOR="Mark Linux"\n'
        "GRUB_CMDLINE_LINUX=\"loglevel=3 quiet\"\n"
        'GRUB_CMDLINE_LINUX_DEFAULT="loglevel=3 quiet"\n'
        "GRUB_ENABLE_BLUETOOTH=false\n"
        "GRUB_DISABLE_OS_PROBER=false\n"
        "GRUB_DISABLE_RECOVERY=false\n"
    )


def sgdisk_commands(layout: DiskLayoutPlan) -> list[list[str]]:
    commands: list[list[str]] = [
        ["wipefs", "-a", layout.device_path],
        ["sgdisk", "--zap-all", layout.device_path],
    ]
    for partition in layout.partitions:
        commands.append(
            [
                "sgdisk",
                "-n",
                f"{partition.number}:{partition.start_sector}:{partition.end_sector}",
                "-t",
                f"{partition.number}:{partition.type_guid}",
                "-c",
                f"{partition.number}:{partition.name}",
                layout.device_path,
            ]
        )
    commands.append(["sgdisk", "-v", layout.device_path])
    commands.append(["udevadm", "settle"])
    return commands


def grub_efi_commands(no_nvram: bool = False) -> list[list[str]]:
    no_nvram_flag = ["--no-nvram"] if no_nvram else []
    return [
        [
            "chroot", str(REQUIRED_TARGET_ROOT), "/usr/bin/grub-install", "--target=x86_64-efi",
            "--efi-directory=/boot", "--bootloader-id=MarkLinux", "--recheck", *no_nvram_flag,
        ],
        [
            "chroot", str(REQUIRED_TARGET_ROOT), "/usr/bin/grub-install", "--target=x86_64-efi",
            "--efi-directory=/boot", "--removable", "--recheck", "--no-nvram",
        ],
    ]


def grub_bios_commands(disk_path: str) -> list[list[str]]:
    device = disk_path.removeprefix("/dev/")
    return [
        [
            "chroot", str(REQUIRED_TARGET_ROOT), "/usr/bin/grub-install", "--target=i386-pc",
            "--recheck", "--boot-directory=/boot", f"/dev/{device}",
        ]
    ]


class CommandRunner:
    def __init__(self) -> None:
        self.log_path = Path("/tmp/marklinux-install.log")

    def log(self, line: str) -> None:
        try:
            self.log_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.log_path, "a", encoding="utf-8") as stream:
                stream.write(line + "\n")
        except OSError:
            pass

    def run(
        self,
        argv: Sequence[str],
        *,
        check: bool = True,
        input_text: str | None = None,
        capture: bool = False,
    ) -> subprocess.CompletedProcess:
        argv = [str(part) for part in argv]
        command = " ".join(argv)
        self.log(f"$ {command}")
        result = subprocess.run(argv, check=False, input=input_text, capture_output=capture, text=True)
        if check and result.returncode != 0:
            detail = (result.stderr or result.stdout or "").strip()
            raise InstallError(f"Command failed with status {result.returncode}: {command}. {detail}")
        return result


class Installer:
    """Privileged, safety-gated real installation backend."""

    def __init__(
        self,
        config: InstallConfig,
        password: str,
        *,
        progress: Callable[[str, int | None], None] | None = None,
        progress_mode: bool = False,
        runner: CommandRunner | None = None,
    ) -> None:
        self.config = config
        self.password = password
        self._progress = progress or (lambda message, percent=None: None)
        self.progress_mode = progress_mode
        self._runner = runner or CommandRunner()
        self._percent = 0
        self.layout: DiskLayoutPlan | None = None
        self.target_root = REQUIRED_TARGET_ROOT
        self._root_mounted = False
        self._boot_mounted = False
        self._swap_device: str | None = None

    # -- progress / logging -------------------------------------------------
    def _emit(self, message: str, percent: int | None = None) -> None:
        value = self._percent if percent is None else percent
        self._progress(f"{value}% {message}", value)
        self._percent = min(100, max(self._percent, value))

    def _run(self, argv: Sequence[str], *, check: bool = True, input_text: str | None = None, capture: bool = False) -> subprocess.CompletedProcess:
        result = self._runner.run(list(argv), check=check, input_text=input_text, capture=capture)
        command_text = " ".join(str(item) for item in argv)
        self._emit(f"ran: {command_text[:100]}", self._percent)
        return result

    @contextmanager
    def chroot_mounts(self) -> Iterator[Path]:
        target = self.target_root
        mounts = [
            ("/dev", "dev", "bind"),
            ("/proc", "proc", "proc"),
            ("/sys", "sys", "bind"),
            ("/run", "run", "bind"),
        ]
        bound: list[Path] = []
        try:
            for source, relative, mount_type in mounts:
                destination = target / relative
                destination.mkdir(parents=True, exist_ok=True)
                self._run(["mount", "--bind", source, str(destination)])
                bound.append(destination)
            yield target
        finally:
            for destination in reversed(bound):
                self._run(["umount", str(destination)], check=False)

    def chroot_run(self, argv: Sequence[str], *, input_text: str | None = None) -> subprocess.CompletedProcess:
        return self._run(["chroot", str(self.target_root), *[str(part) for part in argv]], input_text=input_text)

    def write_text(self, relative: str, content: str, mode: int = 0o644) -> None:
        path = self.target_root / relative.lstrip("/")
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.exists() or path.is_symlink():
            path.unlink()
        path.write_text(content, encoding="utf-8")
        path.chmod(mode)

    # -- safety checks ------------------------------------------------------
    def mounted_devices(self) -> set[str]:
        devices: set[str] = set()
        try:
            for line in Path("/proc/mounts").read_text(encoding="utf-8").splitlines():
                fields = line.split()
                if len(fields) >= 2 and fields[0].startswith("/"):
                    devices.add(fields[0])
            for line in Path("/proc/swaps").read_text(encoding="utf-8").splitlines():
                fields = line.split()
                if len(fields) >= 1 and fields[0].startswith("/"):
                    devices.add(fields[0])
        except OSError:
            pass
        return devices

    def _live_medium_source(self) -> str | None:
        result = self._runner.run(
            ["/usr/bin/findmnt", "-rn", "-no", "SOURCE", "/run/archiso/bootmnt"],
            check=False,
            capture=True,
        )
        source = (result.stdout or "").strip().split("\n")[0]
        return source or None

    def _validate_target(self) -> None:
        validate_install_config(self.config)
        missing = [tool for tool in REQUIRED_TOOLS if shutil.which(tool) is None]
        if missing:
            raise InstallError(f"The live environment is missing installer tools: {', '.join(missing)}")
        mounted = self.mounted_devices()
        if self.config.disk_path in mounted or any(
            device.startswith(self.config.disk_path) for device in mounted
        ):
            raise InstallError(f"Target disk {self.config.disk_path} or one of its partitions is mounted.")
        validate_target_disk(
            self.config.disk_path,
            self.config.disk_size_bytes,
            is_read_only=False,
            active_mount_devices=mounted or None,
        )
        live_source = self._live_medium_source()
        if live_source and self.config.disk_path.startswith(live_source):
            raise RuntimeError(f"Refusing to overwrite the live installation medium ({live_source}).")

    # -- installation steps -------------------------------------------------
    def preflight(self) -> DiskLayoutPlan:
        self._validate_target()
        size_result = self._runner.run(["/usr/bin/blockdev", "--getsize64", self.config.disk_path], capture=True)
        try:
            actual_size = int((size_result.stdout or "").strip())
        except (TypeError, ValueError) as error:
            raise InstallError(f"Could not read the size of {self.config.disk_path}.") from error
        read_only = self._runner.run(["/usr/bin/blockdev", "--getro", self.config.disk_path], capture=True)
        if (read_only.stdout or "").strip() != "0":
            raise InstallError(f"Target disk {self.config.disk_path} is read-only.")
        if actual_size < self.config.disk_size_bytes:
            raise InstallError(
                f"The target disk changed size ({_human_size(actual_size)}); refusing to install."
            )
        self.config = InstallConfig(
            disk_path=self.config.disk_path,
            disk_size_bytes=actual_size,
            username=self.config.username,
            full_name=self.config.full_name,
            timezone=self.config.timezone,
            keyboard_layout=self.config.keyboard_layout,
            performance_profile=self.config.performance_profile,
            filesystem=self.config.filesystem,
            include_swap=self.config.include_swap,
            hostname=self.config.hostname,
        )
        self.layout = calculate_erase_disk_layout(
            disk_path=self.config.disk_path,
            disk_size_bytes=self.config.disk_size_bytes,
            memory_bytes=None,
            filesystem=FilesystemType(self.config.filesystem),
            bootloader=BootloaderType.GRUB,
            include_swap=self.config.include_swap,
            include_bios_boot=True,
        )
        self._emit("Target disk validated", 5)
        return self.layout

    def run(self) -> dict[str, Any]:
        if os.geteuid() != 0:
            raise RuntimeError("The installer backend must run as root (via sudo or pkexec).")
        self._percent = 0
        layout = self.preflight()
        try:
            self._percent = 20
            self._emit("Erasing target disk", 25)
            for argv in sgdisk_commands(layout):
                self._run(argv)
            self._run(["/usr/bin/partprobe", layout.device_path])
            self._run(["/usr/bin/udevadm", "settle"])
            self._emit("GPT layout created", 35)

            esp = partition_for(layout, "/boot")
            swap = next((partition for partition in layout.partitions if partition.filesystem == "swap"), None)
            root = partition_for(layout, "/")
            self._run(["/usr/sbin/mkfs.fat", "-F32", "-n", INSTALLER_LABEL, partition_path(layout.device_path, esp.number)])
            if swap is not None:
                self._run(["/usr/sbin/mkswap", "-L", f"{INSTALLER_LABEL}-swap", partition_path(layout.device_path, swap.number)])
            mkfs = "/usr/sbin/mkfs.btrfs" if layout.filesystem.value == "btrfs" else "/usr/sbin/mkfs.ext4"
            self._run([mkfs, "-F", "-L", "MarkLinux-Root", partition_path(layout.device_path, root.number)])
            self._emit("Partitions formatted", 45)

            root_type = layout.filesystem.value
            self._run(["mount", "-t", root_type, partition_path(layout.device_path, root.number), str(self.target_root)])
            self._root_mounted = True
            self._run(["mkdir", "-p", str(self.target_root / "boot")])
            self._run(["mount", "-t", "vfat", "-o", "umask=0077", partition_path(layout.device_path, esp.number), str(self.target_root / "boot")])
            self._boot_mounted = True
            if swap is not None:
                self._swap_device = partition_path(layout.device_path, swap.number)
                self._run(["swapon", self._swap_device])

            self._copy_rootfs()
            self._copy_boot_artifacts()
            self._emit("System files and boot files copied from the live ISO", 60)

            self._configure_installed_system(layout)
            self._install_bootloader()
            self._verify()
            self._emit("Installation finished", 100)
            return {
                "device_path": layout.device_path,
                "filesystem": layout.filesystem.value,
                "partitions": [partition.mount_point for partition in layout.partitions],
            }
        finally:
            self._cleanup_mounts()

    def _copy_rootfs(self) -> None:
        excludes = (
            "proc", "sys", "dev", "run", "tmp", "mnt", "media", "boot", "lost+found",
            "home", "var/tmp", "var/cache/pacman", "var/log",
            "var/lib/systemd", "var/lib/dbus", "var/lib/private", "etc/resolv.conf",
        )
        tar = ["/usr/bin/tar", "-C", "/", "--one-file-system", "--xattrs", "--acls", "--numeric-owner", "-cpf", "-"]
        for entry in excludes:
            tar += ["--exclude", f"./{entry}", f"./{entry}/*"]
        tar.append(".")
        producer = subprocess.Popen(tar, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        consumer = subprocess.Popen(
            ["/usr/bin/tar", "-C", str(self.target_root), "--xattrs", "--acls", "--numeric-owner", "-xpf", "-"],
            stdin=producer.stdout,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        producer.stdout.close()
        _consumer_out, consumer_err = consumer.communicate()
        producer_err = producer.stderr.read()
        producer_err_text = producer_err.decode("utf-8", "replace") if isinstance(producer_err, bytes) else (producer_err or "")
        producer.wait()
        consumer.wait()
        if consumer.returncode != 0:
            raise RuntimeError(f"Live rootfs copy failed: {consumer_err.decode('utf-8', 'replace').strip()}")
        if producer.returncode != 0:
            raise RuntimeError(f"Live rootfs tar reader failed: {producer_err_text.strip()}")

    def _copy_boot_artifacts(self) -> None:
        """Copy the kernel files that ArchISO stores outside the live rootfs."""
        candidates = (
            Path("/run/archiso/bootmnt") / LIVE_INSTALL_DIR / "boot" / "x86_64",
            Path("/run/archiso/bootmnt") / "boot" / "x86_64",
            Path("/boot"),
        )
        source_root = next(
            (candidate for candidate in candidates if all((candidate / name).is_file() for name in LIVE_BOOT_RELATIVE)),
            None,
        )
        if source_root is None:
            searched = ", ".join(str(candidate) for candidate in candidates)
            raise InstallError(f"Could not find the live kernel and initramfs. Searched: {searched}")

        for name in LIVE_BOOT_RELATIVE:
            self._run([
                "/usr/bin/cp", "-f",
                str(source_root / name), str(self.target_root / "boot" / name),
            ])

    def _remove(self, path: Path) -> None:
        self._run(["/usr/bin/rm", "-rf", str(path)], check=False)

    def _blkid_uuid(self, device: str) -> str:
        result = self._run(["/usr/bin/blkid", "-s", "UUID", "-o", "value", device], capture=True)
        value = (result.stdout or "").strip()
        if not value:
            raise InstallError(f"Could not read filesystem UUID from {device}.")
        return value

    def _configure_installed_system(self, layout: DiskLayoutPlan) -> None:
        self._remove(self.target_root / "etc" / "mkinitcpio.conf.d" / "archiso.conf")
        self._remove(self.target_root / "etc" / "systemd" / "system" / "marklinux-live-user.service")
        self._remove(self.target_root / "etc" / "systemd" / "system" / "multi-user.target.wants" / "marklinux-live-user.service")
        self._remove(self.target_root / "etc" / "systemd" / "system" / "getty@tty1.service.d")
        self._remove(self.target_root / "etc" / "sudoers.d" / "10-live")
        self._remove(self.target_root / "etc" / "polkit-1" / "rules.d" / "49-marklinux-installer.rules")
        self._remove(self.target_root / "etc" / "machine-id")
        self._remove(self.target_root / "var" / "lib" / "systemd" / "random-seed")

        self.write_text(
            "etc/systemd/system/getty@tty1.service.d/autologin.conf",
            "[Unit]\nDescription=Log in the Mark Linux user on the console\n"
            "[Service]\nExecStart=\nExecStart=-/usr/bin/agetty --autologin "
            f"{self.config.username} --noclear %I $TERM\n"
            "[Install]\nWantedBy=getty.target\n",
        )
        root_device = partition_path(layout.device_path, partition_for(layout, "/").number)
        esp_device = partition_path(layout.device_path, partition_for(layout, "/boot").number)
        swap_partition = next((p for p in layout.partitions if p.filesystem == "swap"), None)
        swap_uuid = self._blkid_uuid(partition_path(layout.device_path, swap_partition.number)) if swap_partition else None
        self.write_text("etc/fstab", render_fstab(
            self._blkid_uuid(root_device), self._blkid_uuid(esp_device),
            self.config.filesystem, swap_uuid,
        ))
        self.write_text("etc/default/grub", render_grub_default(), 0o644)
        self.write_text("etc/sudoers.d/10-wheel", f"%wheel\tALL=(ALL:ALL) ALL\n", 0o440)
        self.write_text("etc/hostname", f"{self.config.hostname}\n", 0o644)
        self.write_text(
            "etc/hosts",
            f"127.0.0.1\tlocalhost\n::1\tlocalhost\n127.0.1.1\t{self.config.hostname}.localdomain {self.config.hostname}\n",
        )
        self.write_text("etc/locale.conf", "LANG=en_US.UTF-8\n")
        self.write_text("etc/locale.gen", "en_US.UTF-8 UTF-8\n", 0o644)
        self.write_text("etc/vconsole.conf", f"KEYMAP={self.config.keyboard_layout}\nFONT=eVGA8x16\n")
        localtime = self.target_root / "etc" / "localtime"
        if localtime.exists() or localtime.is_symlink():
            localtime.unlink()
        localtime.symlink_to(f"/usr/share/zoneinfo/{self.config.timezone}")
        self.write_text(
            "etc/skel/.config/marklinux/performance-profile",
            f"{PERFORMANCE_PROFILE_KEYS[self.config.performance_profile]}\n",
            0o644,
        )

        with self.chroot_mounts() as chroot_target:
            self.chroot_run(["/usr/bin/systemctl", "enable", "NetworkManager.service", "bluetooth.service", "fstrim.timer"])
            self.chroot_run(["/usr/bin/systemctl", "enable", "getty@tty1.service"])
            self.chroot_run(["/usr/bin/systemctl", "set-default", "multi-user.target"])
            self.chroot_run(["/usr/bin/systemd-machine-id-setup"])
            # The live session account is created in the running ISO before
            # this backend starts. It must not leak into the installed system
            # or collide with the account selected in the installer.
            self.chroot_run(["/usr/bin/userdel", "--remove", "mark"], check=False)
            self.chroot_run(["/usr/bin/groupdel", "mark"], check=False)
            self.chroot_run([
                "/usr/bin/useradd", "-m", "-U", "-c", self.config.full_name,
                "-G", "wheel,video,render,input,audio,storage,optical",
                "-s", "/usr/bin/bash", self.config.username,
            ])
            self.chroot_run(["/usr/bin/chpasswd"], input_text=f"{self.config.username}:{self.password}\n")
            self.chroot_run(["/usr/bin/locale-gen"])
            self.chroot_run(["/usr/bin/systemctl", "daemon-reload"])
            self.chroot_run(["/usr/bin/mkinitcpio", "-P"])

    def _install_bootloader(self) -> None:
        has_efi = Path("/sys/firmware/efi").is_dir()
        with self.chroot_mounts() as chroot_target:
            self.chroot_run(["/usr/bin/grub-mkconfig", "-o", "/boot/grub/grub.cfg"])
            if has_efi:
                # The removable fallback path makes the installed disk boot
                # even when VMware/firmware exposes no writable EFI vars.
                for argv in grub_efi_commands(no_nvram=True):
                    self._run(argv)
            self.chroot_run(["/usr/bin/grub-install", "--target=i386-pc", "--recheck", "--boot-directory=/boot", f"/dev/{self.config.disk_path.removeprefix('/dev/')}"])

    def _verify(self) -> None:
        with self.chroot_mounts() as chroot_target:
            ok = self.chroot_run(["/usr/bin/test", "-s", "/boot/grub/grub.cfg"], check=False)
            if ok.returncode != 0:
                raise RuntimeError("GRUB configuration was not generated at /boot/grub/grub.cfg.")
            ok = self.chroot_run(["/usr/bin/test", "-s", "/boot/vmlinuz-linux"], check=False)
            if ok.returncode != 0:
                raise RuntimeError("Kernel image was not found at /boot/vmlinuz-linux.")
            ok = self.chroot_run(["/usr/bin/test", "-s", "/boot/initramfs-linux.img"], check=False)
            if ok.returncode != 0:
                raise RuntimeError("Initramfs was not generated at /boot/initramfs-linux.img.")
            ok = self.chroot_run(["/usr/bin/id", self.config.username], check=False)
            if ok.returncode != 0:
                raise RuntimeError(f"Installed account {self.config.username} was not created.")

    def _cleanup_mounts(self) -> None:
        if self._swap_device:
            self._run(["swapoff", self._swap_device], check=False)
            self._swap_device = None
        if self._boot_mounted:
            self._run(["/usr/sbin/umount", str(self.target_root / "boot")], check=False)
            self._boot_mounted = False
        if self._root_mounted:
            self._run(["/usr/sbin/umount", str(self.target_root)], check=False)
            self._root_mounted = False
        self._run(["/usr/bin/sync"], check=False)


def build_config_from_arguments(
    *,
    disk: str,
    disk_size_bytes: int,
    user: str,
    full_name: str,
    timezone: str,
    keyboard_layout: str,
    performance_profile: str,
    filesystem: str,
    no_swap: bool,
    hostname: str,
) -> InstallConfig:
    return InstallConfig(
        disk_path=disk,
        disk_size_bytes=disk_size_bytes,
        username=user,
        full_name=full_name or user,
        timezone=timezone,
        keyboard_layout=keyboard_layout,
        performance_profile=performance_profile,
        filesystem=filesystem,
        include_swap=not no_swap,
        hostname=hostname,
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Mark Linux real installer backend")
    parser.add_argument("--disk", default="/dev/sdb")
    parser.add_argument("--disk-size-bytes", type=int, default=0)
    parser.add_argument("--user", default="mark")
    parser.add_argument("--full-name", default="")
    parser.add_argument("--timezone", default="UTC")
    parser.add_argument("--keyboard-layout", default="us")
    parser.add_argument("--performance-profile", default="Balanced")
    parser.add_argument("--filesystem", default="ext4")
    parser.add_argument("--no-swap", action="store_true")
    parser.add_argument("--hostname", default="marklinux")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--yes", action="store_true")
    parser.add_argument("--password-stdin", action="store_true")
    args = parser.parse_args(argv)

    if not args.dry_run and not args.yes:
        answer = input("Type ERASE to start the real installation on the selected disk: ")
        if answer.strip() != "ERASE":
            print("Aborted: erase confirmation was not typed.")
            return 1

    if args.password_stdin:
        password = sys.stdin.readline().rstrip("\n")
    else:
        password = __import__("getpass").getpass("New password: ")

    if not password:
        print("Mark Linux installer failed: password must not be empty.", file=sys.stderr)
        return 1

    config = build_config_from_arguments(
        disk=args.disk,
        disk_size_bytes=args.disk_size_bytes,
        user=args.user,
        full_name=args.full_name,
        timezone=args.timezone,
        keyboard_layout=args.keyboard_layout,
        performance_profile=args.performance_profile,
        filesystem=args.filesystem,
        no_swap=args.no_swap,
        hostname=args.hostname,
    )

    installer = Installer(
        config=config,
        password=password,
        progress=lambda message, percent=None: print(message, flush=True),
        progress_mode=False,
    )
    if args.dry_run:
        print("dry-run requested; validate and plan only")
        return 0
    try:
        installer.run()
    except (RuntimeError, ValueError, OSError) as error:
        print(f"Mark Linux installer failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
