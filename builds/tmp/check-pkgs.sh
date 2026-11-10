#!/usr/bin/env bash
for p in sudo rsync gptfdisk dosfstools grub efibootmgr arch-install-scripts e2fsprogs btrfs-progs ttf-font-awesome; do
  if /usr/bin/pacman -Si "$p" >/dev/null 2>&1; then echo "OK $p"; else echo "MISSING $p"; fi
done
