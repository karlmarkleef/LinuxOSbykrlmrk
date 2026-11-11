# mark-power-manager

Real power management capabilities, persisted settings and backend actions for Mark Linux.

## Capabilities

- Detects suspend/hibernate support from the kernel power state (`/sys/power/state`) and swap availability.
- Detects systemd-logind availability over system D-Bus.
- Detects Hyprland IPC availability for lock and logout actions.

## Backend actions

- Shutdown and restart are performed through the `org.freedesktop.login1.Manager` D-Bus interface.
- Suspend and hibernate are performed through the `org.freedesktop.login1.Manager` D-Bus interface.
- Lock, unlock and logout are performed through the Hyprland IPC socket (`hyprctl`).

## Settings

Mark Settings persists user power preferences as a `key=value` file at
`/var/lib/marklinux/power-settings`. The defaults and parsing rules are implemented in the library.

## Build

```
cargo build --release
```

## Test

```
cargo test --locked --all-targets
```

## Run

```
./target/release/mark-power --capabilities
./target/release/mark-power --settings
./target/release/mark-power suspend
./target/release/mark-power shutdown --yes
```
