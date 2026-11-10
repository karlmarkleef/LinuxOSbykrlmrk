-- Mark Linux desktop configuration for the Arch repository Hyprland release.
-- macOS-inspired visual language: frosted translucency, rounded corners, soft shadows.
local terminal = "foot"
local file_manager = "thunar"
local launcher = "fuzzel"

-- Monitor: "auto" scale mặc định — integer scale giữ wallpaper/UI sắc nét.
hl.monitor({ output = "", mode = "preferred", position = "auto", scale = 1 })

hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_TYPE", "wayland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")
hl.env("GTK_THEME", "Adwaita")
hl.env("COLOR_SCHEME", "prefer-light")
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
hl.env("QT_WAYLAND_DISABLE_WINDOWDECORATION", "1")
hl.env("MOZ_ENABLE_WAYLAND", "1")
hl.env("GDK_SCALE", "1")
hl.env("GDK_DPI_SCALE", "1")
hl.env("QT_SCALE_FACTOR", "1")
hl.env("WAYDROID_DISPLAY", "wayland-1")

hl.config({
  general = {
    gaps_in = 12, gaps_out = 16, border_size = 1,
    col = {
      active_border = { colors = { "rgba(96,165,250,0.55)", "rgba(192,132,252,0.55)" }, angle = 45 },
      inactive_border = "rgba(90,90,110,0.22)",
    },
    resize_on_border = true, allow_tearing = false, layout = "dwindle",
  },
  decoration = {
    rounding = 12, rounding_power = 2,
    active_opacity = 0.98, inactive_opacity = 0.92,
    shadow = { enabled = true, range = 20, render_power = 3, color = "rgba(15,23,42,0.30)" },
    blur = { enabled = true, size = 8, passes = 4, vibrancy = 0.35, xray = false },
  },
  animations = { enabled = true },
  input = { kb_layout = "us", follow_mouse = 1, sensitivity = 0, touchpad = { natural_scroll = false } },
  dwindle = { preserve_split = true },
  misc = { disable_hyprland_logo = true, force_default_wallpaper = 0 },
  xwayland = { force_zero_scaling = true },
})

hl.curve("markEase", { type = "bezier", points = { { 0.16, 1.0 }, { 0.30, 1.0 } } })
hl.animation({ leaf = "global", enabled = true, speed = 7, bezier = "markEase" })
hl.animation({ leaf = "windowsIn", enabled = true, speed = 5, bezier = "markEase", style = "popin 85%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 4, bezier = "markEase", style = "popin 85%" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 5, bezier = "markEase", style = "fade" })
hl.animation({ leaf = "fade", enabled = true, speed = 4, bezier = "markEase" })

-- Frosted-glass layer surfaces (menu bar, dock, notifications, launcher).
hl.layer_rule({ name = "mark-waybar", match = { namespace = "waybar" }, blur = true })
hl.layer_rule({ name = "mark-mako", match = { namespace = "mako" }, blur = true })
hl.layer_rule({ name = "mark-dock", match = { namespace = "nwg-dock-hyprland" }, blur = true })

hl.bind("SUPER + RETURN", hl.dsp.exec_cmd(terminal))
hl.bind("SUPER + R", hl.dsp.exec_cmd(launcher))
hl.bind("SUPER + E", hl.dsp.exec_cmd(file_manager))
hl.bind("SUPER + G", hl.dsp.exec_cmd("mark-quick-settings"))
hl.bind("SUPER + W", hl.dsp.exec_cmd("mark-welcome"))
hl.bind("SUPER + I", hl.dsp.exec_cmd("mark-installer"))
hl.bind("SUPER + H", hl.dsp.exec_cmd("mark-hardware-report"))
hl.bind("SUPER + P", hl.dsp.exec_cmd("mark-profile-menu"))
hl.bind("SUPER + M", hl.dsp.exec_cmd("missioncenter"))
hl.bind("SUPER + SHIFT + W", hl.dsp.exec_cmd("mark-windows-apps"))
hl.bind("SUPER + SHIFT + A", hl.dsp.exec_cmd("mark-android-launch"))
hl.bind("SUPER + CTRL + W", hl.dsp.exec_cmd("mark-windows-vm"))
hl.bind("SUPER + Q", hl.dsp.window.close())
hl.bind("SUPER + SHIFT + Q", hl.dsp.exec_cmd("hyprctl dispatch exit"))
hl.bind("SUPER + V", hl.dsp.window.float({ action = "toggle" }))
hl.bind("SUPER + SHIFT + V", hl.dsp.exec_cmd("mark-effects-toggle --toggle"))
hl.bind("SUPER + LEFT", hl.dsp.focus({ direction = "left" }))
hl.bind("SUPER + RIGHT", hl.dsp.focus({ direction = "right" }))
hl.bind("SUPER + UP", hl.dsp.focus({ direction = "up" }))
hl.bind("SUPER + DOWN", hl.dsp.focus({ direction = "down" }))
for i = 1, 5 do
  hl.bind("SUPER + " .. i, hl.dsp.focus({ workspace = i }))
  hl.bind("SUPER + SHIFT + " .. i, hl.dsp.window.move({ workspace = i }))
end
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), { locked = true, repeating = true })
hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), { locked = true, repeating = true })
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true, repeating = true })
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("brightnessctl -e4 -n2 set 5%+"), { locked = true, repeating = true })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("brightnessctl -e4 -n2 set 5%-"), { locked = true, repeating = true })

hl.on("hyprland.start", function()
  hl.exec_cmd("mark-profile auto")
  hl.exec_cmd("hyprpaper")
  hl.exec_cmd("waybar")
  hl.exec_cmd("mako")
  hl.exec_cmd("nm-applet --indicator")
  hl.exec_cmd("blueman-applet")
  hl.exec_cmd("/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1")
  hl.exec_cmd("nwg-dock-hyprland -r -p bottom -i 44 -w 6 -c fuzzel -lp start -mb 14 -mt 8 -mr 8")
  hl.exec_cmd("mark-welcome --once")
end)
