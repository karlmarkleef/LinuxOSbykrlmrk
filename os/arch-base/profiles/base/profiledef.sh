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
file_permissions=(
  ["/etc/shadow"]="0:0:400"
)
