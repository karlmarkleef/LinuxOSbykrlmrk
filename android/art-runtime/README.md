# android/art-runtime

## Purpose

Interfaces with the **Android Runtime (ART)** inside the Waydroid container. ART is Android's application runtime that executes DEX bytecode (compiled from Java/Kotlin). This module handles ART tuning, profile-guided optimization hints, and runtime diagnostics.

## Status

**Phase 5 — Planned**

ART is embedded in the AOSP image and runs inside the Waydroid container. Direct tuning from the host (Mark Linux) is planned via Waydroid's shell interface.

## Upstream

- **ART (Android Runtime)** — Part of AOSP — android.googlesource.com/platform/art
- **Waydroid shell** — `waydroid shell` for access to Android internals

## Components

| File | Description |
|------|-------------|
| *(planned)* `art-tuning.sh` | Apply ART performance tuning via waydroid shell |
| *(planned)* `art-diagnostics.sh` | Dump ART runtime info and GC statistics |

## Notes

ART compiles DEX to native code using AOT (Ahead-Of-Time) compilation via `dex2oat`. Profile-guided optimization can significantly improve app startup time. This module will expose controls for `dex2oat` compilation filters (`speed`, `speed-profile`, `quicken`) via the Waydroid shell.
