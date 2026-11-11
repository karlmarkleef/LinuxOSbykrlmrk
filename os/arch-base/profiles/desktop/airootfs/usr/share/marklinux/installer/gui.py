#!/usr/bin/python
"""Zenity-backed graphical Mark Linux installer front end.

Collects target disk, account, regional and performance settings, asks for an
explicit erase confirmation, then drives the privileged backend through
`pkexec mark-install`. No disk write happens before the user types ERASE.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

INSTALLER_TITLE = "Mark Linux Installation"
ERASE_CONFIRMATION = "ERASE"
MINIMUM_DISK_BYTES = 20 * 1024**3

import install as install_backend  # noqa: E402  (staged next to this file)


def zenity(*arguments: str, capture: bool = False) -> str | None:
    result = subprocess.run(
        ["zenity", *arguments],
        check=False,
        text=True,
        capture_output=capture,
    )
    if result.returncode != 0:
        return None
    return result.stdout.strip() if capture else ""


def eligible_disks() -> list[dict[str, object]]:
    """Read whole-disk sysfs metadata through lsblk; never read disk contents."""
    result = subprocess.run(
        ["lsblk", "--json", "--bytes", "--nodeps", "--output", "PATH,SIZE,MODEL,TYPE,RO"],
        check=True,
        text=True,
        capture_output=True,
    )
    import json

    document = json.loads(result.stdout)
    disks = []
    for item in document.get("blockdevices", []):
        path = item.get("path")
        size = item.get("size")
        if (
            item.get("type") == "disk"
            and item.get("ro") in (False, 0, "0")
            and isinstance(path, str)
            and isinstance(size, int)
            and size >= MINIMUM_DISK_BYTES
        ):
            disks.append({"path": path, "size": size, "model": item.get("model") or "Unknown model"})
    return sorted(disks, key=lambda disk: str(disk["path"]))


def human_size(size: int) -> str:
    return f"{size / 1024**3:.1f} GiB"


def choose_disk(disks: list[dict[str, object]]) -> dict[str, object] | None:
    if not disks:
        zenity(
            "--error", "--title", INSTALLER_TITLE,
            "--text", "No writable whole disk of at least 20 GiB was reported. No changes were made.",
            "--width", "560",
        )
        return None

    rows: list[str] = []
    for disk in disks:
        rows.extend((str(disk["path"]), human_size(int(disk["size"])), str(disk["model"])))
    selected = zenity(
        "--list", "--title", INSTALLER_TITLE,
        "--text", "Choose the whole disk Mark Linux will install to.",
        "--column", "Device", "--column", "Size", "--column", "Model",
        "--print-column", "1", "--separator", "\t", "--width", "760", "--height", "420",
        *rows,
        capture=True,
    )
    return next((disk for disk in disks if disk["path"] == selected), None)


def detect_recommended_profile() -> str:
    """Query read-only mark-hardware for adaptive profile recommendation."""
    try:
        result = subprocess.run(
            ["mark-hardware", "--profile-name"],
            check=False,
            text=True,
            capture_output=True,
            timeout=5,
        )
        if result.returncode == 0:
            name = result.stdout.strip()
            if name in ("Ultra Lite", "Balanced", "Cinematic"):
                return name
    except (OSError, subprocess.SubprocessError):
        pass
    return "Balanced"


def collect_settings(default_profile: str) -> tuple[str, str, str, str, str] | None:
    full_name = zenity(
        "--entry", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", "Full name (for the account):", "--entry-text", default_profile,
        capture=True,
    )
    username = zenity(
        "--entry", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", "Account name (lowercase):", "--entry-text", "mark",
        capture=True,
    )
    timezone = zenity(
        "--entry", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", "Time zone (example: UTC):", "--entry-text", "UTC",
        capture=True,
    )
    keyboard = zenity(
        "--entry", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", "Keyboard layout (us, gb, de, fr, ...):", "--entry-text", "us",
        capture=True,
    )
    profile = zenity(
        "--entry", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", f"Performance profile (recommended: {default_profile}):", "--entry-text", default_profile,
        capture=True,
    )
    if not (full_name and username and timezone and keyboard and profile):
        return None

    config = install_backend.InstallConfig(
        disk_path="/dev/sda",
        disk_size_bytes=20 * 1024**3,
        username=username,
        full_name=full_name,
        timezone=timezone,
        keyboard_layout=keyboard,
        performance_profile=profile,
        filesystem="ext4",
        include_swap=True,
        hostname="marklinux",
    )
    try:
        install_backend.validate_install_config(config)
    except (ValueError, TypeError) as error:
        zenity("--error", "--title", INSTALLER_TITLE, "--text", str(error), "--width", "560")
        return None
    return username, full_name, timezone, keyboard, profile


def ask_password() -> str | None:
    password = zenity(
        "--password", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", "Choose the password for the new account. It is never shown or logged.",
        capture=True,
    )
    if not password:
        return None
    confirmation = zenity(
        "--password", "--title", INSTALLER_TITLE, "--width", "520",
        "--text", "Confirm the password:",
        capture=True,
    )
    if confirmation is None or confirmation != password:
        return None
    return password


def confirm_erase(disk: dict[str, object], username: str, performance_profile: str) -> bool:
    summary = (
        "Mark Linux will ERASE and rewrite the following disk:\n"
        f"  {disk['path']} ({human_size(int(disk['size']))}, {disk['model']})\n\n"
        f"Account: {username}\nPerformance profile: {performance_profile}\n\n"
        "UEFI and legacy-BIOS boot are supported. No changes are made before you type ERASE."
    )
    if zenity(
        "--question", "--title", INSTALLER_TITLE, "--width", "680",
        "--text", summary,
        "--ok-label", "Continue to erase", "--cancel-label", "Cancel",
    ) is None:
        return False
    answer = zenity(
        "--entry", "--title", INSTALLER_TITLE, "--width", "640",
        "--text", f"Type {ERASE_CONFIRMATION} to start the installation:",
        "--entry-text", "",
        capture=True,
    )
    if answer is None:
        return False
    return answer.strip() == ERASE_CONFIRMATION


def launch_backend(
    disk: dict[str, object],
    settings: tuple[str, str, str, str, str],
) -> int:
    username, full_name, timezone, keyboard, profile = settings
    password = ask_password()
    if password is None or not confirm_erase(disk, username, profile):
        return 0

    command = [
        "pkexec", "/usr/bin/mark-install",
        "--yes",
        "--disk", str(disk["path"]),
        "--disk-size-bytes", str(int(disk["size"])),
        "--user", username,
        "--full-name", full_name,
        "--timezone", timezone,
        "--keyboard-layout", keyboard,
        "--performance-profile", profile,
        "--filesystem", "ext4",
        "--hostname", "marklinux",
        "--password-stdin",
    ]
    try:
        process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        process.stdin.write(password + "\n")
        process.stdin.close()
        progress = subprocess.Popen(
            [
                "zenity", "--progress", "--pulsate", "--auto-close",
                "--title", f"{INSTALLER_TITLE} - {disk['path']}",
            ],
            stdin=subprocess.PIPE,
            text=True,
        )
        for line in process.stdout:
            line = line.strip()
            if line.startswith("#"):
                progress.stdin.write(line + "\n")
            elif line.isdigit():
                progress.stdin.write(f"{int(line)}\n")
            progress.stdin.flush()
        progress.stdin.write("100\n")
        progress.stdin.close()
        progress.wait()
    except OSError:
        return 1
    if process.returncode != 0:
        return 1
    return 0


def main() -> int:
    intro = (
        "Mark Linux will erase the selected disk and write its own system, then boot "
        "from it. UEFI and legacy-BIOS boot are both supported. No disk is touched "
        "before you confirm with ERASE."
    )
    if zenity("--question", "--title", INSTALLER_TITLE, "--text", intro, "--width", "620") is None:
        return 0

    disks = eligible_disks()
    disk = choose_disk(disks)
    if disk is None:
        return 0
    default_profile = detect_recommended_profile()
    settings = collect_settings(default_profile)
    if settings is None:
        return 0
    status = launch_backend(disk, settings)
    if status != 0:
        zenity(
            "--error", "--title", INSTALLER_TITLE,
            "--text", "The Mark Linux installation backend did not complete. No disk writes were lost on purpose before confirmation.",
            "--width", "560",
        )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError) as error:
        zenity("--error", "--title", INSTALLER_TITLE, "--text", f"Installer front end failed: {error}", "--width", "620")
        sys.exit(1)
