#!/usr/bin/env bash
set -Eeuo pipefail
project=/mnt/h/LinuxOSbykrlmrk
archive="$project/builds/cache/archlinux-bootstrap-2026.11.01-x86_64.tar"
root=/var/tmp/marklinux-bootstrap/root.x86_64
parent=/var/tmp/marklinux-bootstrap
if [[ ! -f "$archive" ]]; then
  echo "Verified decompressed bootstrap tar is missing: $archive" >&2
  exit 2
fi
if [[ -e "$root" ]]; then
  echo "Bootstrap root already exists; refusing to overwrite: $root" >&2
  exit 3
fi
mkdir -p "$parent"
tar --numeric-owner -xf "$archive" -C "$parent"
test -x "$root/usr/bin/pacman"
echo "Extracted verified Arch bootstrap to $root"