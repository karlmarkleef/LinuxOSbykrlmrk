# Start the real Hyprland session on the live VT. Other shells stay usable for recovery.
if [[ -z "${WAYLAND_DISPLAY:-}" && "${XDG_VTNR:-0}" == "1" && -z "${MARK_SESSION_ACTIVE:-}" ]]; then
  if command -v mark-session >/dev/null 2>&1; then
    export MARK_SESSION_ACTIVE=1
    exec mark-session
  elif command -v start-hyprland >/dev/null 2>&1; then
    exec start-hyprland
  fi
fi
