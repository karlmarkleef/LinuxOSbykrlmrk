from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "windows" / "dependency-manager"))

import resolver  # noqa: E402


class DependencyResolverTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = resolver.load_manifest(
            ROOT / "windows" / "dependency-manager" / "dependencies.json"
        )

    def test_load_manifest_reads_valid_file(self) -> None:
        self.assertEqual(self.manifest.get("schema_version"), 1)
        self.assertIn("wine", self.manifest.get("runtimes", {}))
        self.assertIn("wine_staging", self.manifest.get("runtimes", {}))
        self.assertIn("proton", self.manifest.get("runtimes", {}))

    def test_load_manifest_rejects_wrong_schema(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            bad_path = Path(tmp) / "dependencies.json"
            bad_path.write_text(json.dumps({"schema_version": 2}), encoding="utf-8")
            with self.assertRaises(ValueError):
                resolver.load_manifest(bad_path)

    def test_dependency_set_lookup(self) -> None:
        entry = resolver.dependency_set(self.manifest, "dxvk-d3d9")
        self.assertIsNotNone(entry)
        self.assertEqual(entry.get("runtime"), "wine_staging")
        self.assertIsNone(resolver.dependency_set(self.manifest, "does-not-exist"))

    def test_components_for_dxvk_d3d9(self) -> None:
        components = resolver.components_for(self.manifest, "wine_staging", ["9"])
        self.assertIn("dxvk", components)
        self.assertIn("corefonts", components)

    def test_components_for_vkd3d_d3d12(self) -> None:
        components = resolver.components_for(self.manifest, "proton", ["12"])
        self.assertIn("vkd3d-proton", components)
        self.assertIn("proton-ge", components)

    def test_components_for_unknown_runtime_fall_back_to_defaults(self) -> None:
        components = resolver.components_for(self.manifest, "not-a-runtime", ["9"])
        self.assertEqual(components, [])


if __name__ == "__main__":
    unittest.main()
