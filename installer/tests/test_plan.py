from __future__ import annotations

import json
import os
import pathlib
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = pathlib.Path(__file__).resolve().parents[2]
PLANNER = (
    ROOT / "os/arch-base/profiles/desktop/airootfs/usr/share/marklinux/installer"
)
sys.path.insert(0, str(PLANNER))

from plan import build_plan, save_plan  # noqa: E402
import gui  # noqa: E402


class InstallPlanTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.temp_root = pathlib.Path(self.temporary.name)
        self.zoneinfo = self.temp_root / "zoneinfo"
        (self.zoneinfo / "America").mkdir(parents=True)
        (self.zoneinfo / "America/Los_Angeles").write_text("fixture", encoding="utf-8")
        self.payload = {
            "disk_path": "/dev/nvme0n1",
            "disk_size_bytes": 512_110_190_592,
            "disk_model": "Fixture SSD",
            "username": "mark_user",
            "timezone": "America/Los_Angeles",
            "keyboard_layout": "us",
            "performance_profile": "Balanced",
        }

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_builds_explicit_plan_only_without_credentials(self) -> None:
        plan = build_plan(self.payload, self.zoneinfo)
        self.assertEqual(plan["mode"], "plan-only")
        self.assertEqual(plan["target"]["planned_action"], "erase-selected-disk-and-install")
        self.assertEqual(plan["target"]["device_path"], "/dev/nvme0n1")
        self.assertEqual(plan["desktop"]["performance_profile"], "Balanced")
        self.assertNotIn("password", plan["account"])
        self.assertIn("does not partition", plan["warnings"][0])

    def test_rejects_partition_paths_and_small_disks(self) -> None:
        for bad_path in ("/dev/sda1", "/dev/mapper/root", "/dev/../sda"):
            with self.subTest(path=bad_path):
                payload = dict(self.payload, disk_path=bad_path)
                with self.assertRaises(ValueError):
                    build_plan(payload, self.zoneinfo)
        payload = dict(self.payload, disk_size_bytes=1)
        with self.assertRaises(ValueError):
            build_plan(payload, self.zoneinfo)

    def test_rejects_invalid_identity_locale_and_unknown_fields(self) -> None:
        invalid_values = (
            ("username", "Root"),
            ("username", "root"),
            ("timezone", "../../etc/passwd"),
            ("timezone", "Pacific/NotAZone"),
            ("keyboard_layout", "../../bin/sh"),
            ("performance_profile", "Erase Disk"),
        )
        for field, value in invalid_values:
            with self.subTest(field=field, value=value):
                payload = dict(self.payload, **{field: value})
                with self.assertRaises(ValueError):
                    build_plan(payload, self.zoneinfo)
        with self.assertRaises(ValueError):
            build_plan(dict(self.payload, password="secret"), self.zoneinfo)

    def test_saves_private_new_file_and_refuses_overwrite(self) -> None:
        plan = build_plan(self.payload, self.zoneinfo)
        destination = self.temp_root / "plan.json"
        save_plan(destination, plan)
        self.assertEqual(json.loads(destination.read_text(encoding="utf-8")), plan)
        if os.name == "posix":
            self.assertEqual(destination.stat().st_mode & 0o777, 0o600)
        with self.assertRaises(FileExistsError):
            save_plan(destination, plan)

    @patch("gui.subprocess.run")
    def test_gui_disk_query_is_read_only_and_filters_partitions(self, run) -> None:
        run.return_value.stdout = json.dumps({
            "blockdevices": [
                {"path": "/dev/sda", "size": 100_000_000_000, "model": "Disk A", "type": "disk", "ro": False},
                {"path": "/dev/sda1", "size": 50_000_000_000, "model": "Disk A", "type": "part", "ro": False},
                {"path": "/dev/sdb", "size": 100_000_000_000, "model": "Read only", "type": "disk", "ro": True},
                {"path": "/dev/sdc", "size": 1_000_000_000, "model": "Small", "type": "disk", "ro": False},
                {"path": "/dev/sr0", "size": 100_000_000_000, "model": "ISO", "type": "rom", "ro": True},
            ]
        })
        disks = gui.eligible_disks()
        self.assertEqual([disk["path"] for disk in disks], ["/dev/sda"])
        arguments = run.call_args.args[0]
        self.assertEqual(arguments[0], "lsblk")
        self.assertIn("--nodeps", arguments)
        self.assertNotIn("--output-all", arguments)
        self.assertIs(run.call_args.kwargs["check"], True)

    def test_planner_contains_no_disk_writer_or_elevation_command(self) -> None:
        source = (PLANNER / "gui.py").read_text(encoding="utf-8").casefold()
        for command in ("mkfs", "parted", "sgdisk", "pacstrap", "pkexec", "sudo"):
            with self.subTest(command=command):
                self.assertNotIn(command, source)

    @patch("gui.eligible_disks")
    @patch("gui.build_plan", return_value={"mode": "plan-only", "target": {"device_path": "/dev/sda"}})
    @patch("gui.zenity")
    def test_gui_flow_saves_only_a_plan_file(self, zenity, _build_plan, eligible_disks) -> None:
        eligible_disks.return_value = [{
            "path": "/dev/sda", "size": 100_000_000_000, "model": "Fixture Disk"
        }]
        destination = self.temp_root / "install-plan.json"
        zenity.side_effect = [
            "", None, "/dev/sda", "mark\tUTC\tus\tBalanced", "", str(destination), ""
        ]
        self.assertEqual(gui.main(), 0)
        saved = json.loads(destination.read_text(encoding="utf-8"))
        self.assertEqual(saved["mode"], "plan-only")
        self.assertEqual(saved["target"]["device_path"], "/dev/sda")

    @patch("gui.subprocess.run")
    def test_detects_recommended_profile_from_hardware_tool(self, run) -> None:
        run.return_value.returncode = 0
        run.return_value.stdout = "Cinematic\n"
        self.assertEqual(gui.detect_recommended_profile(), "Cinematic")

        run.side_effect = FileNotFoundError
        self.assertEqual(gui.detect_recommended_profile(), "Balanced")


if __name__ == "__main__":
    unittest.main()
