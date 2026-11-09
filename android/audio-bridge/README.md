# android/audio-bridge

## Purpose

Routes **Android audio** (from inside the Waydroid container) to the Mark Linux host audio system (PipeWire/PulseAudio). Handles audio output from Android apps and games, as well as microphone input passthrough for Android apps that need recording access.

## Status

**Phase 5 — Planned**

Waydroid handles audio via PulseAudio/PipeWire socket sharing. Mark Linux audio bridge configuration and troubleshooting tools are planned.

## Upstream

- **Waydroid** — exposes PulseAudio socket to Android container
- **PipeWire** — Mark Linux primary audio server
- **PulseAudio** — PipeWire's PA compatibility layer used by Waydroid

## Components

| File | Description |
|------|-------------|
| *(planned)* `audio-config.toml` | Audio bridge configuration |
| *(planned)* `audio-setup.sh` | Configure PipeWire/PA socket for Waydroid |

## Notes

Waydroid communicates with the host audio server via the `PULSE_SERVER` environment variable pointing to the PulseAudio/PipeWire socket. Mark Linux PipeWire is configured to expose a PulseAudio-compatible socket by default (`/run/user/<UID>/pulse/native`).
