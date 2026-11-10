"""Dependency resolution for the Mark Linux Windows dependency manager.

The dependency manager wraps upstream Winetricks instead of implementing
Windows runtime dependency installation itself. This module resolves which
upstream winetricks component sets apply to a given runtime and DirectX
requirement.

Pure helpers keep the resolution logic unit testable without touching the
host system.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Sequence

MANIFEST_SCHEMA_VERSION = 1
DEFAULT_MANIFEST_PATH = (
    Path(__file__).resolve().parents[2] / "windows" / "dependency-manager" / "dependencies.json"
)


def load_manifest(path) -> dict:
    """Load a dependency manager manifest from a JSON file."""
    with open(path, "r", encoding="utf-8") as handle:
        manifest = json.load(handle)
    if manifest.get("schema_version") != MANIFEST_SCHEMA_VERSION:
        raise ValueError(
            f"Unsupported dependency manager schema version: {manifest.get('schema_version')}"
        )
    return manifest


def dependency_set(manifest: dict, set_id: str) -> dict | None:
    """Return the dependency set with the given id, if present."""
    for entry in manifest.get("sets", []):
        if entry.get("set_id") == set_id:
            return entry
    return None


def runtime_components(manifest: dict, runtime: str) -> list[str]:
    """Return the default winetricks components for a runtime."""
    runtimes = manifest.get("runtimes", {})
    entry = runtimes.get(runtime)
    if entry is None:
        return []
    return [str(component) for component in entry.get("components", [])]


def components_for(manifest: dict, runtime: str, directx: Sequence[str]) -> list[str]:
    """Resolve the winetricks components required by a runtime and DirectX level.

    Matching dependency sets contribute their components; when no set matches,
    the runtime default components are returned.
    """
    wanted = [str(level) for level in directx]
    components: list[str] = []
    matched = False
    for entry in manifest.get("sets", []):
        if entry.get("runtime") != runtime:
            continue
        levels = [str(level) for level in entry.get("directx", [])]
        if wanted and not any(level in wanted for level in levels):
            continue
        matched = True
        for component in entry.get("components", []):
            if str(component) not in components:
                components.append(str(component))
    if matched:
        return components
    return runtime_components(manifest, runtime)
