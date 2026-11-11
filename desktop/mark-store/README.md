# Mark Store

Graphical package manager frontend for Mark Linux using Pacman.

Built with GTK4. The frontend browses, installs and removes packages through the
real pacman backend implemented in `src/lib.rs`.

## Backend

- `PacmanBackend` executes pacman through a `PacmanRunner` trait.
- Default runner (`RealRunner`) runs the real pacman binary.
- Search uses `pacman -Ss`, installed listing uses `pacman -Qe`, metadata uses
  `pacman -Qi`, install uses `pacman -S` and remove uses `pacman -R`.

## Build

```
cargo build --release
```

## Run

```
./target/release/mark-store
```
