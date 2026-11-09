# Package groups

`base.x86_64` is the Phase 1 Arch live base. `desktop.x86_64` is the Phase 2 add-on. The complete package lists used by ArchISO are checked into `os/arch-base/profiles/base/packages.x86_64` and `profiles/desktop/packages.x86_64`.

The desktop set includes Hyprland, Waybar, a dock and launcher, notifications, file/audio/network/display tools, Mission Center, Fastfetch, AMD microcode, and NVIDIA open modules for the detected RTX 2060 SUPER. Package availability was checked against the official Arch sync database before adding the packages. NVIDIA's open kernel modules are for Turing and newer GPUs; driver installation is limited to the Linux image, never the Windows host.
