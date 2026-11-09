# Motion and performance profiles

The active Hyprland session defaults to `cinematic`. `mark-profile` reads the profile TOML in `/usr/share/marklinux/` and applies those values through Hyprland's runtime Lua API. Waybar exposes a control that disables compositor animation, blur and shadows. The tool changes live compositor state and does not rewrite user configuration.

`ultra-lite`, `balanced`, `cinematic` and `custom` are implemented for compositor properties. GTK application and dock internals may retain their own motion; this toggle is not described as a system-wide application animation controller.
