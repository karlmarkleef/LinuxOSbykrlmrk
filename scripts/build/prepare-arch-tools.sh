#!/usr/bin/env bash
set -Eeuo pipefail
project=/mnt/h/LinuxOSbykrlmrk
root=/var/tmp/marklinux-bootstrap/root.x86_64
packages="$project/tools/build/bootstrap-packages.txt"
mirrorlist="$root/etc/pacman.d/mirrorlist"
log="$project/builds/logs/arch-toolchain-packages.txt"
if [[ ! -x "$root/usr/bin/pacman" ]]; then
  echo "Arch bootstrap is not extracted: $root" >&2
  exit 2
fi
if ! grep -Ev '^[[:space:]]*($|#)' "$packages" | grep -q .; then
  echo "Build tool package list is empty." >&2
  exit 3
fi
# Use the official Arch mirror; this changes only the disposable bootstrap root.
printf '%s\n' 'Server = https://fastly.mirror.pkgbuild.com/$repo/os/$arch' > "$mirrorlist"
mkdir -p "$root/mnt/project"
mount --bind "$root" "$root"
mount --bind "$project" "$root/mnt/project"
cleanup() {
  mountpoint -q "$root/mnt/project" && umount "$root/mnt/project"
  mountpoint -q "$root" && umount "$root"
}
trap cleanup EXIT
xargs -r -a "$packages" "$root/usr/bin/arch-chroot" "$root" /usr/bin/pacman -Syu --needed --noconfirm
"$root/usr/bin/arch-chroot" "$root" /usr/bin/pacman -Q archiso grub qemu-system-x86 rust | tee "$log"
