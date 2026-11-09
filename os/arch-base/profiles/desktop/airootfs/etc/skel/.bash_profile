# Start the real Hyprland session on the live VT. Other shells stay usable for recovery.
if [[ -z "${WAYLAND_DISPLAY:-}" && "${XDG_VTNR:-0}" == "1" ]] && command -v start-hyprland >/dev/null 2>&1; then
  exec start-hyprland
fi
