#!/usr/bin/python
"""Zenity-backed graphical installation planner. It never writes to disks."""

from __future__ import annotations

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from plan import build_plan, save_plan


MINIMUM_DISK_BYTES = 20 * 1024**3
INSTALLER_TITLE = "Mark Linux Installation Planner"


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
        "--text", "Choose a target for the future install plan. This planner will not modify it.",
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


def collect_settings(default_profile: str | None = None) -> list[str] | None:
    if default_profile is None:
        default_profile = detect_recommended_profile()

    all_profiles = ["Balanced", "Cinematic", "Ultra Lite", "Custom"]
    if default_profile in all_profiles:
        all_profiles.remove(default_profile)
        combo_values = f"{default_profile}|" + "|".join(all_profiles)
    else:
        combo_values = "Balanced|Cinematic|Ultra Lite|Custom"

    form = zenity(
        "--forms", "--title", INSTALLER_TITLE,
        "--text", f"These settings are recorded in the plan only. Adaptive recommendation: {default_profile}.",
        "--separator", "\t",
        "--add-entry", "User name (lowercase)",
        "--add-entry", "Time zone (example: UTC)",
        "--add-combo", "Keyboard layout", "--combo-values", "us|gb|de|fr",
        "--add-combo", f"Performance profile (recommended: {default_profile})", "--combo-values", combo_values,
        "--width", "620",
        capture=True,
    )
    if form is None:
        return None
    fields = form.split("\t")
    return fields if len(fields) == 4 else None


def main() -> int:
    intro = (
        "This early release builds and saves an installation plan. It does not partition, "
        "format, install software, or write to any disk. Full-disk erase, alongside-Windows, "
        "and manual partitioning are not available yet."
    )
    if zenity("--question", "--title", INSTALLER_TITLE, "--text", intro,
              "--ok-label", "Create Plan", "--cancel-label", "Cancel", "--width", "620") is None:
        return 0

    if zenity("--question", "--title", INSTALLER_TITLE,
              "--text", "Open the read-only hardware summary first?",
              "--ok-label", "Open Summary", "--cancel-label", "Continue", "--width", "520") is not None:
        subprocess.Popen(["mark-hardware-report"], close_fds=True)

    disks = eligible_disks()
    disk = choose_disk(disks)
    if disk is None:
        return 0
    settings = collect_settings()
    if settings is None:
        return 0
    username, timezone_name, keyboard, profile = settings

    payload = {
        "disk_path": disk["path"],
        "disk_size_bytes": disk["size"],
        "disk_model": disk["model"],
        "username": username,
        "timezone": timezone_name,
        "keyboard_layout": keyboard,
        "performance_profile": profile,
    }
    try:
        plan = build_plan(payload)
    except (ValueError, TypeError) as error:
        zenity("--error", "--title", INSTALLER_TITLE, "--text", str(error), "--width", "560")
        return 1

    summary = (
        f"Target: {disk['path']}\nModel: {disk['model']}\nSize: {human_size(int(disk['size']))}\n\n"
        f"User: {username}\nTime zone: {timezone_name}\nKeyboard: {keyboard}\n"
        f"Desktop: Hyprland ({profile})\n\n"
        "Plan mode only: no partitions or files on the selected disk will be changed."
    )
    if zenity("--question", "--title", INSTALLER_TITLE, "--text", summary,
              "--ok-label", "Save Plan", "--cancel-label", "Cancel", "--width", "620") is None:
        return 0

    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    default_name = f"marklinux-install-plan-{timestamp}.json"
    destination = zenity(
        "--file-selection", "--save", "--title", "Save Mark Linux Installation Plan",
        "--filename", str(Path.home() / default_name),
        capture=True,
    )
    if destination is None:
        return 0
    try:
        save_plan(Path(destination), plan)
    except (OSError, ValueError) as error:
        zenity("--error", "--title", INSTALLER_TITLE,
               "--text", f"Could not save a new plan file: {error}", "--width", "620")
        return 1

    zenity("--info", "--title", INSTALLER_TITLE,
           "--text", f"Plan saved to:\n{destination}\n\nNo disk was modified.", "--width", "560")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, json.JSONDecodeError, subprocess.SubprocessError) as error:
        zenity("--error", "--title", INSTALLER_TITLE,
               "--text", f"The read-only preflight failed: {error}", "--width", "620")
        sys.exit(1)
