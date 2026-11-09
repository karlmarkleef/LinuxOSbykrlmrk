# android/security

## Purpose

Defines the **security policy** for the Mark Android Subsystem. Controls filesystem isolation, network access modes, and permission defaults for Android apps running in the Waydroid container. Ensures Android apps cannot access sensitive host system resources.

## Status

**Phase 5 — Planned**

The sandbox policy configuration (`sandbox-policy.toml`) defines the intended security model. Enforcement via LXC profiles and AppArmor policies is planned.

## Upstream

- **LXC** — container isolation (Linux Containers)
- **AppArmor** — Mandatory Access Control for Waydroid on Ubuntu/Arch
- **Waydroid** — container security baseline

## Components

| File | Description |
|------|-------------|
| `sandbox-policy.toml` | Sandbox security policy definition |
| *(planned)* `apparmor-waydroid.conf` | AppArmor profile for Waydroid |
| *(planned)* `lxc-security.conf` | LXC container security configuration |

## Security Model

| Area | Policy |
|------|--------|
| Filesystem | Android data isolated; Linux root not accessible |
| Shared folder | `~/Android/shared/` only |
| Network | Bridged mode with system DNS |
| Camera/Mic | Prompt on first use |
| Location | Denied by default |
| Contacts | Denied by default |

## Notes

The security model intentionally mirrors Android's permission model: sensitive resources require explicit user consent or are denied by default. The goal is that Mark Android apps are no less safe than apps on a physical Android device.
