# android/gpu-bridge

## Purpose

Bridges the **Android GPU stack** (inside Waydroid container) to the Mark Linux host GPU drivers. Provides hardware-accelerated graphics for Android apps via Virtio-GPU or direct GPU passthrough. This enables Android games and GPU-intensive apps to use the host's GPU at near-native performance.

## Status

**Phase 5 — Planned**

Waydroid supports `--render-engine` options (`drm`/`virgl`/`mesa`). Mark Linux GPU bridge configuration and auto-detection of the optimal renderer is planned.

## Upstream

- **Waydroid** — `waydroid session start --render-engine`
- **Mesa (virglrenderer)** — Vulkan/OpenGL translation for Virtio-GPU
- **gbinder** — Waydroid's Binder implementation for GPU commands

## Components

| File | Description |
|------|-------------|
| *(planned)* `gpu-detect.sh` | Detect host GPU and select optimal render engine |
| *(planned)* `gpu-config.toml` | GPU bridge configuration |

## Render Engine Options

| Engine | Description | Use Case |
|--------|-------------|----------|
| `drm` | Direct DRM/KMS rendering | Best for NVIDIA/AMD |
| `virgl` | Virtio-GPU via virglrenderer | Good compatibility |
| `mesa` | Mesa software rasterizer | Fallback |

## Notes

NVIDIA proprietary drivers may require additional configuration. AMD and Intel GPUs generally work out of the box with the Mesa `drm` backend.
