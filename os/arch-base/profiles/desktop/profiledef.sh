#!/usr/bin/env bash
# shellcheck disable=SC2034
# Mark Linux Phase 1 live image, adapted from the ArchISO baseline profile.
iso_name="marklinux"
iso_label="MARKLINUX_001"
iso_publisher="Mark Linux Project"
iso_application="Mark Linux Live ISO"
iso_version="0.0.1_dev_1101126_1"
install_dir="marklinux"
bootmodes=('bios.syslinux'
           'uefi.grub')
pacman_conf="pacman.conf"
airootfs_image_type="erofs"
airootfs_image_tool_options=('-zlzma,109' -E 'ztailpacking')
bootstrap_tarball_compression=(xz -9e)
# mkarchiso copies airootfs with --no-preserve=mode, so every executable in
# the overlay must be listed here. Staged MCP binaries are present during the
# desktop ISO build and are removed from the source overlay afterwards.
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/usr/local/bin/mark-adobe-compat"]="0:0:755"
  ["/usr/local/bin/mark-android-launch"]="0:0:755"
  ["/usr/local/bin/mark-android-manager"]="0:0:755"
  ["/usr/local/bin/mark-apk"]="0:0:755"
  ["/usr/local/bin/mark-apk-inspect"]="0:0:755"
  ["/usr/local/bin/mark-benchmark"]="0:0:755"
  ["/usr/local/bin/mark-compat-center"]="0:0:755"
  ["/usr/local/bin/mark-effects-toggle"]="0:0:755"
  ["/usr/local/bin/mark-hardware"]="0:0:755"
  ["/usr/local/bin/mark-hardware-report"]="0:0:755"
  ["/usr/local/bin/mark-installer"]="0:0:755"
  ["/usr/local/bin/mark-install"]="0:0:755"
  ["/usr/local/bin/mark-pe-inspect"]="0:0:755"
  ["/usr/local/bin/mark-profile"]="0:0:755"
  ["/usr/local/bin/mark-profile-menu"]="0:0:755"
  ["/usr/local/bin/mark-quick-settings"]="0:0:755"
  ["/usr/local/bin/mark-runtime"]="0:0:755"
  ["/usr/local/bin/mark-session"]="0:0:755"
  ["/usr/local/bin/mark-welcome"]="0:0:755"
  ["/usr/local/bin/mark-win-launch"]="0:0:755"
  ["/usr/local/bin/mark-windows-apps"]="0:0:755"
  ["/usr/local/bin/mark-windows-vm"]="0:0:755"
  ["/usr/local/bin/marklinux-live-user"]="0:0:755"
)
