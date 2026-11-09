"""Validate and save a non-destructive Mark Linux installation plan."""

from __future__ import annotations

import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


MINIMUM_DISK_BYTES = 20 * 1024**3
DEVICE_PATTERN = re.compile(
    r"^/dev/(?:sd[a-z]+|vd[a-z]+|xvd[a-z]+|nvme[0-9]+n[0-9]+|mmcblk[0-9]+)$"
)
USERNAME_PATTERN = re.compile(r"^[a-z_][a-z0-9_-]{0,31}$")
TIMEZONE_PATTERN = re.compile(r"^[A-Za-z0-9_+-]+(?:/[A-Za-z0-9_+.-]+)*$")
KEYBOARD_LAYOUTS = {"us", "gb", "de", "fr"}
PERFORMANCE_PROFILES = {"Ultra Lite", "Balanced", "Cinematic", "Custom"}


def build_plan(payload: dict[str, Any], zoneinfo_root: Path = Path("/usr/share/zoneinfo")) -> dict[str, Any]:
    """Validate UI data and return a plan; never probes or modifies a device."""
    expected = {
        "disk_path", "disk_size_bytes", "disk_model", "username", "timezone",
        "keyboard_layout", "performance_profile",
    }
    if set(payload) != expected:
        raise ValueError("The installer form is incomplete or contains unsupported fields.")

    disk_path = payload["disk_path"]
    if not isinstance(disk_path, str) or not DEVICE_PATTERN.fullmatch(disk_path):
        raise ValueError("Select a whole disk device, not a partition or volume.")
    disk_size = payload["disk_size_bytes"]
    if isinstance(disk_size, bool) or not isinstance(disk_size, int) or disk_size < MINIMUM_DISK_BYTES:
        raise ValueError("The selected disk is smaller than the 20 GiB planning minimum.")
    model = payload["disk_model"]
    if not isinstance(model, str) or len(model) > 256 or any(ord(char) < 32 for char in model):
        raise ValueError("The disk model field is invalid.")

    username = payload["username"]
    if not isinstance(username, str) or not USERNAME_PATTERN.fullmatch(username) or username == "root":
        raise ValueError("Choose a lowercase account name starting with a letter or underscore.")

    timezone_name = payload["timezone"]
    if not isinstance(timezone_name, str) or not TIMEZONE_PATTERN.fullmatch(timezone_name):
        raise ValueError("The time zone name is invalid.")
    timezone_path = (zoneinfo_root / timezone_name).resolve()
    if not timezone_path.is_relative_to(zoneinfo_root.resolve()) or not timezone_path.is_file():
        raise ValueError("The selected time zone is not available in this live environment.")

    keyboard = payload["keyboard_layout"]
    if keyboard not in KEYBOARD_LAYOUTS:
        raise ValueError("Choose a supported keyboard layout.")
    profile = payload["performance_profile"]
    if profile not in PERFORMANCE_PROFILES:
        raise ValueError("Choose a supported performance profile.")

    return {
        "plan_format": 1,
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "system": {"name": "Mark Linux", "version": "0.0.1_dev", "build": "1101126_1"},
        "mode": "plan-only",
        "target": {
            "device_path": disk_path,
            "model": model.strip() or "Unknown model",
            "size_bytes": disk_size,
            "size_gib": round(disk_size / 1024**3, 2),
            "planned_action": "erase-selected-disk-and-install",
        },
        "account": {"username": username},
        "regional_settings": {"timezone": timezone_name, "keyboard_layout": keyboard},
        "desktop": {"environment": "Hyprland", "performance_profile": profile},
        "warnings": [
            "This is a plan only. This build does not partition, format, install, or write to any disk.",
            "A future installer must verify the exact physical device and request explicit erase confirmation.",
            "No account password or credential is collected or stored by this planner.",
        ],
    }


def save_plan(destination: Path, plan: dict[str, Any]) -> None:
    """Create a new private plan file; refuse to overwrite an existing path."""
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    descriptor = os.open(destination, flags, 0o600)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(plan, stream, ensure_ascii=False, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException:
        try:
            destination.unlink()
        except OSError:
            pass
        raise
