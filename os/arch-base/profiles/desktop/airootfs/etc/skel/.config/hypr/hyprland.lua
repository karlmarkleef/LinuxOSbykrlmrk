-- Mark Linux desktop configuration for the Arch repository Hyprland release.
local terminal = "foot"
local file_manager = "thunar"
local launcher = "fuzzel"

-- Monitor: "auto" scale mặc định — integer scale giữ wallpaper/UI sắc nét.
-- Nếu HiDPI 4K: thay scale = 2 (integer, không dùng 1.5 để tránh blur).
hl.monitor({ output = "", mode = "preferred", position = "auto", scale = 1 })
hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_TYPE", "wayland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")
hl.env("GTK_THEME", "Adwaita:dark")
hl.env("COLOR_SCHEME", "prefer-dark")
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
hl.env("QT_WAYLAND_DISABLE_WINDOWDECORATION", "1")
hl.env("MOZ_ENABLE_WAYLAND", "1")
-- Sharp rendering: prevent GTK/apps from double-scaling on HiDPI
hl.env("GDK_SCALE", "1")
hl.env("GDK_DPI_SCALE", "1")
hl.env("QT_SCALE_FACTOR", "1")
-- Waydroid / Android integration
hl.env("WAYDROID_DISPLAY", "wayland-1")

hl.config({
  general = {
    gaps_in = 5, gaps_out = 14, border_size = 2,
    col = {
      active_border = { colors = { "rgba(55d6c2ee)", "rgba(62a8f5ee)" }, angle = 35 },
      inactive_border = "rgba(547078aa)",
    },
    resize_on_border = true, allow_tearing = false, layout = "dwindle",
  },
  decoration = {
    rounding = 12, active_opacity = 1.0, inactive_opacity = 0.96,
    shadow = { enabled = true, range = 12, render_power = 3, color = "rgba(07151dcc)" },
    blur = { enabled = true, size = 5, passes = 2, vibrancy = 0.12 },
  },
  animations = { enabled = true },
  input = { kb_layout = "us", follow_mouse = 1, sensitivity = 0 },
  dwindle = { preserve_split = true },
  misc = {
    disable_hyprland_logo = true,
    force_default_wallpaper = 0,
  },
  xwayland = {
    -- XWayland apps không bị blur khi dùng fractional scale
    force_zero_scaling = true,
  },
})

hl.curve("markEase", { type = "bezier", points = { { 0.16, 1.0 }, { 0.30, 1.0 } } })
hl.animation({ leaf = "global", enabled = true, speed = 7, bezier = "markEase" })
hl.animation({ leaf = "windowsIn", enabled = true, speed = 5, bezier = "markEase", style = "popin 88%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 4, bezier = "markEase", style = "popin 88%" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 5, bezier = "markEase", style = "fade" })
hl.animation({ leaf = "fade", enabled = true, speed = 4, bezier = "markEase" })

hl.bind("SUPER + RETURN", hl.dsp.exec_cmd(terminal))
hl.bind("SUPER + R", hl.dsp.exec_cmd(launcher))
hl.bind("SUPER + E", hl.dsp.exec_cmd(file_manager))
hl.bind("SUPER + G", hl.dsp.exec_cmd("mark-quick-settings"))
hl.bind("SUPER + W", hl.dsp.exec_cmd("mark-welcome"))
hl.bind("SUPER + I", hl.dsp.exec_cmd("mark-installer"))
hl.bind("SUPER + H", hl.dsp.exec_cmd("mark-hardware-report"))
hl.bind("SUPER + P", hl.dsp.exec_cmd("mark-profile-menu"))
hl.bind("SUPER + M", hl.dsp.exec_cmd("missioncenter"))
-- Windows apps via Proton/Wine (no VM, no GPU conflict)
hl.bind("SUPER + SHIFT + W", hl.dsp.exec_cmd("mark-windows-apps"))
-- Android apps via Waydroid LXC (like ChromeOS ARC++)
hl.bind("SUPER + SHIFT + A", hl.dsp.exec_cmd("mark-android-launch"))
-- Full Windows OS VM (only when full OS needed)
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
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true })
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("brightnessctl -e4 -n2 set 5%+"), { locked = true, repeating = true })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("brightnessctl -e4 -n2 set 5%-"), { locked = true, repeating = true })

hl.on("hyprland.start", function()
  hl.exec_cmd("mark-profile auto")
  -- hyprpaper thay swaybg: sharp rendering, per-monitor, reload runtime
  hl.exec_cmd("hyprpaper")
  hl.exec_cmd("waybar")
  hl.exec_cmd("mako")
  hl.exec_cmd("nm-applet --indicator")
  hl.exec_cmd("blueman-applet")
  hl.exec_cmd("/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1")
  hl.exec_cmd("nwg-dock-hyprland -r -p bottom -i 42 -w 5 -c fuzzel -lp start")
  hl.exec_cmd("mark-welcome --once")
end)
