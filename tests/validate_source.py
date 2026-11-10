"""Read-only validation for checked-in Mark Linux source and profile data."""

from __future__ import annotations

import json
import ast
import pathlib
import re
import shutil
import subprocess
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[1]
PROFILE_BASE = ROOT / "os/arch-base/profiles/base"
PROFILE_DESKTOP = ROOT / "os/arch-base/profiles/desktop"


def check(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)
    print(f"PASS {message}")


def manifest(path: pathlib.Path) -> list[str]:
    return [line.strip() for line in path.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.lstrip().startswith("#")]


def shell_validator() -> pathlib.Path | None:
    git_bash = (pathlib.Path(r"C:\Program Files\Git\bin\bash.exe"),
                pathlib.Path(r"C:\Program Files (x86)\Git\bin\bash.exe"))
    for path in git_bash:
        if path.is_file():
            return path
    found = shutil.which("bash") or shutil.which("bash.exe")
    if not found:
        return None
    path = pathlib.Path(found)
    # Windows' system32 bash.exe is the WSL launcher, not a Bash interpreter.
    if "system32" in {part.casefold() for part in path.parts}:
        return None
    return path


def main() -> int:
    version = (ROOT / "VERSION").read_text(encoding="utf-8").strip()
    build_id = (ROOT / "BUILD_ID").read_text(encoding="utf-8").strip()
    expected_identity = f"{version}_{build_id}"

    for profile in (PROFILE_BASE, PROFILE_DESKTOP):
        definition = (profile / "profiledef.sh").read_text(encoding="utf-8")
        match = re.search(r'^iso_version="([^"]+)"$', definition, re.MULTILINE)
        check(match is not None and match.group(1) == expected_identity,
              f"{profile.name} ISO identity matches VERSION and BUILD_ID")

    base_packages = manifest(ROOT / "os/packages/base.x86_64")
    desktop_packages = manifest(ROOT / "os/packages/desktop.x86_64")
    check(len(base_packages) == len(set(base_packages)), "base package manifest has no duplicates")
    check(len(desktop_packages) == len(set(desktop_packages)), "desktop add-on manifest has no duplicates")
    check(manifest(PROFILE_BASE / "packages.x86_64") == base_packages,
          "base ArchISO profile matches its package manifest")
    check(manifest(PROFILE_DESKTOP / "packages.x86_64") == base_packages + desktop_packages,
          "desktop ArchISO profile matches base plus desktop manifests")
    check({"hyprland", "waybar", "hyprpaper", "nvidia-open", "amd-ucode", "unzip", "zip"}
          <= set(desktop_packages), "desktop set includes the selected session, host firmware, and archive tools")
    boot_configs = [
        PROFILE_DESKTOP / "syslinux/syslinux-linux.cfg",
        PROFILE_DESKTOP / "grub/grub.cfg",
        PROFILE_DESKTOP / "grub/loopback.cfg",
        PROFILE_DESKTOP / "efiboot/loader/entries/01-archiso-linux.conf",
    ]
    check(all("/boot/amd-ucode.img" in path.read_text(encoding="utf-8")
              and "/boot/%ARCH%/amd-ucode.img" not in path.read_text(encoding="utf-8")
              for path in boot_configs), "BIOS and UEFI configs use the ISO's actual AMD microcode path")

    rootfs = PROFILE_DESKTOP / "airootfs"
    generated_python_cache = list(rootfs.rglob("__pycache__")) + list(rootfs.rglob("*.pyc"))
    check(not generated_python_cache,
          "desktop ISO overlay contains no generated Python bytecode caches")
    python_files = [
        rootfs / "usr/share/marklinux/installer/plan.py",
        rootfs / "usr/share/marklinux/installer/gui.py",
        ROOT / "installer/core/install.py",
        ROOT / "installer/core/partition.py",
        ROOT / "installer/tests/test_plan.py",
        ROOT / "installer/tests/test_partition.py",
        ROOT / "installer/tests/test_install.py",
    ]
    for path in python_files:
        ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    check(all(path.is_file() for path in python_files),
          "installer Python sources parse")

    json_files = sorted(rootfs.rglob("*.jsonc"))
    for path in json_files:
        json.loads(path.read_text(encoding="utf-8"))
    check(bool(json_files), f"{len(json_files)} JSON configuration file(s) parse")

    toml_files = sorted(rootfs.rglob("*.toml"))
    for path in toml_files:
        tomllib.loads(path.read_text(encoding="utf-8"))
    check(bool(toml_files), f"{len(toml_files)} TOML configuration file(s) parse")

    profiles_path = ROOT / "desktop/motion/performance-profiles.toml"
    profiles = tomllib.loads(profiles_path.read_text(encoding="utf-8"))
    required = {"animations_enabled", "blur_enabled", "shadow_enabled", "rounding",
                "gaps_in", "gaps_out", "active_opacity", "inactive_opacity", "animation_speed"}
    check(set(profiles["profiles"]) == {"ultra-lite", "balanced", "cinematic", "custom"},
          "four named motion profiles are defined")
    check(all(set(values) == required for values in profiles["profiles"].values()),
          "each motion profile defines all values consumed by mark-profile")

    compat_db_path = ROOT / "windows" / "compatibility-database" / "games.json"
    check(compat_db_path.is_file(), "game compatibility database file exists")
    compat_db = json.loads(compat_db_path.read_text(encoding="utf-8"))
    check(compat_db.get("schema_version") == 1, "game compatibility database declares schema version 1")
    check(isinstance(compat_db.get("entries"), list) and len(compat_db["entries"]) > 0,
          "game compatibility database defines entries")
    valid_statuses = {
        "supported",
        "partially_supported",
        "unsupported",
        "requires_official_developer_support",
    }
    for entry in compat_db["entries"]:
        check(entry.get("status") in valid_statuses,
              f"game compatibility database entry has a valid status: {entry.get('game_id')}")
        check(isinstance(entry.get("graphics"), dict),
              f"game compatibility database entry declares graphics: {entry.get('game_id')}")
        check(isinstance(entry["graphics"].get("vulkan"), bool),
              f"game compatibility database entry declares a Vulkan graphics layer: {entry.get('game_id')}")

    mcp_manifest = rootfs / "usr/share/marklinux/mcp/dependencies.json"
    check(mcp_manifest.is_file(), "desktop ISO overlay carries the MCP dependency manifest")
    mcp_manifest_data = json.loads(mcp_manifest.read_text(encoding="utf-8"))
    check(mcp_manifest_data.get("schema_version") == 1,
          "MCP dependency manifest declares schema version 1")
    check(isinstance(mcp_manifest_data.get("sets"), list) and len(mcp_manifest_data["sets"]) > 0,
          "MCP dependency manifest defines dependency sets")

    shadow = (rootfs / "etc/shadow").read_text(encoding="utf-8").splitlines()
    check(bool(shadow) and shadow[0].startswith("root:!:"), "desktop live root password is locked")
    for name in ("mark-welcome", "mark-quick-settings", "mark-profile-menu",
                 "mark-profile", "mark-effects-toggle", "marklinux-live-user",
                 "mark-hardware-report", "mark-installer",
                 "mark-windows-vm", "mark-windows-apps", "mark-android-launch",
                 "mark-adobe-compat"):
        check((rootfs / "usr/local/bin" / name).is_file(), f"desktop command exists: {name}")

    # mkarchiso copies the overlay with --no-preserve=mode, so commands not
    # listed in file_permissions end up non-executable in the ISO.
    profiledef_text = (PROFILE_DESKTOP / "profiledef.sh").read_text(encoding="utf-8")
    listed_commands = set(re.findall(
        r'\["(/usr/local/bin/[^"]+)"\]="0:0:755"', profiledef_text))
    overlay_commands = {
        f"/usr/local/bin/{path.name}" for path in (rootfs / "usr/local/bin").iterdir()
        if path.is_file()
    }
    staged_commands = {
        "/usr/local/bin/mark-hardware",
        "/usr/local/bin/mark-runtime",
        "/usr/local/bin/mark-pe-inspect",
        "/usr/local/bin/mark-apk-inspect",
        "/usr/local/bin/mark-android-manager",
        "/usr/local/bin/mark-apk",
    }
    missing_permissions = (overlay_commands | staged_commands) - listed_commands
    check(not missing_permissions,
          "desktop overlay and staged commands declare executable permissions"
          + (f": {sorted(missing_permissions)}" if missing_permissions else ""))

    bash = shell_validator()
    if bash:
        shell_files = [PROFILE_BASE / "profiledef.sh", PROFILE_DESKTOP / "profiledef.sh"]
        shell_files += sorted((ROOT / "scripts/build").glob("*.sh"))
        shell_files += sorted((rootfs / "usr/local/bin").glob("mark-*"))
        shell_files.append(rootfs / "usr/local/bin/marklinux-live-user")
        shell_files.append(rootfs / "etc/skel/.bash_profile")
        for path in shell_files:
            result = subprocess.run([str(bash), "-n", str(path)],
                                    capture_output=True, text=True, timeout=20)
            check(result.returncode == 0,
                  f"shell syntax passes: {path.relative_to(ROOT)}" if result.returncode == 0
                  else f"shell syntax failed: {path.relative_to(ROOT)}: {result.stderr.strip()}")
        mcp_scripts = [
            ROOT / "android/container-manager/mark-android-manager",
            ROOT / "android/package-manager/mark-apk",
        ]
        for path in mcp_scripts:
            check(path.is_file(), f"MCP script exists: {path.relative_to(ROOT)}")
            result = subprocess.run([str(bash), "-n", str(path)],
                                    capture_output=True, text=True, timeout=20)
            check(result.returncode == 0,
                  f"shell syntax passes: {path.relative_to(ROOT)}" if result.returncode == 0
                  else f"shell syntax failed: {path.relative_to(ROOT)}: {result.stderr.strip()}")
    else:
        print("SKIP bash syntax checks (Git Bash or bash is not installed)")

    if any(path.exists() for path in (ROOT / "builds/iso").glob("*.iso")):
        print("NOTE ISO artifacts are generated outputs and are validated separately.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AssertionError, KeyError, json.JSONDecodeError, tomllib.TOMLDecodeError) as exc:
        print(f"FAIL {exc}", file=sys.stderr)
        sys.exit(1)
