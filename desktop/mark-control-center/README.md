# Mark Control Center

Mark Linux quick settings panel.

Built with GTK4. The panel exposes quick controls for network, Bluetooth, volume,
brightness, battery and power actions backed by real system commands and the
Mark Linux power manager.

## Backend

- `ControlBackend` executes quick actions through a `ControlRunner` trait.
- Default runner executes real system commands (`nmcli`, `bluetoothctl`, `pactl`,
  `brightnessctl`).
- Battery status is read from the kernel power supply hierarchy.

## Build

```
cargo build --release
```

## Run

```
./target/release/mark-control-center
```
