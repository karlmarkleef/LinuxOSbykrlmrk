#!/usr/bin/env bash
set -Eeuo pipefail

project=/mnt/h/LinuxOSbykrlmrk
root=/var/tmp/marklinux-bootstrap/root.x86_64
profile="${MARKLINUX_PROFILE:-$project/os/arch-base/profiles/base}"
output="$project/builds/iso"
logs="$project/builds/logs"
log="$logs/$(basename "$profile")-build-$(date -u +%Y%m%dT%H%M%SZ).log"

if [[ ! -x "$root/usr/bin/arch-chroot" || ! -x "$root/usr/bin/mkarchiso" ]]; then
  echo "Verified ArchISO build tools are not installed in $root" >&2
  exit 2
fi
if [[ "$profile" != "$project"/os/arch-base/profiles/* ]]; then
  echo "Profile must stay under os/arch-base/profiles/: $profile" >&2
  exit 3
fi
if [[ ! -f "$profile/profiledef.sh" || ! -f "$profile/packages.x86_64" ]]; then
  echo "ArchISO profile is incomplete: $profile" >&2
  exit 3
fi
if [[ "$profile" == "$project/os/arch-base/profiles/base" ]] && ! cmp -s "$project/os/packages/base.x86_64" "$profile/packages.x86_64"; then
  echo "Base profile packages differ from os/packages/base.x86_64" >&2
  exit 3
fi
# Source only the project's checked-in build metadata to determine the output name.
# shellcheck disable=SC1091
# ArchISO uses an associative array for file_permissions; predeclare it for Bash too.
declare -A file_permissions=()
source "$profile/profiledef.sh"
expected="$output/${iso_name}-${iso_version}-x86_64.iso"
hash_file="$expected.sha256"
if [[ -e "$expected" || -e "$hash_file" ]]; then
  echo "Refusing to overwrite existing image or checksum: $expected" >&2
  exit 4
fi
mkdir -p "$output" "$logs"
exec > >(tee -a "$log") 2>&1

# Fail before downloading or extracting packages if the Linux build filesystem
# cannot hold the temporary rootfs and ISO. A desktop profile needs more room.
required_kib=4194304
if [[ "$profile" == "$project/os/arch-base/profiles/desktop" ]]; then
  required_kib=12582912
fi
available_kib=$(df -Pk "$root" | awk 'NR == 2 { print $4 }')
if [[ ! "$available_kib" =~ ^[0-9]+$ ]]; then
  echo "Could not determine free space for build filesystem: $root" >&2
  exit 7
fi
if (( available_kib < required_kib )); then
  echo "Insufficient free space on build filesystem: need $((required_kib / 1048576)) GiB, have $((available_kib / 1048576)) GiB. No packages were installed." >&2
  exit 7
fi

echo "Build log: $log"
echo "Project: $project"
echo "Profile: $profile"
echo "Output: $expected"
profile_in_chroot="/mnt/project${profile#"$project"}"
echo "Temporary build space: Arch bootstrap root on the isolated MarkLinuxBuildTools WSL ext4 filesystem under /var/tmp"
echo "Build filesystem free: $((available_kib / 1048576)) GiB; required: $((required_kib / 1048576)) GiB"

# Stage the real installer backend beside the GUI for the live ISO. These
# modules stay in the repository's installer/core tree and are removed from
# the overlay again after mkarchiso completes.
installer_stage="$profile/airootfs/usr/share/marklinux/installer"
installer_stage_owned=0
stage_installer() {
  mkdir -p "$installer_stage"
  install -m 0644 "$project/installer/core/partition.py" "$installer_stage/partition.py"
  install -m 0644 "$project/installer/core/install.py" "$installer_stage/install.py"
  installer_stage_owned=1
}
cleanup_installer_stage() {
  if (( installer_stage_owned )); then
    rm -f "$installer_stage/partition.py" "$installer_stage/install.py"
  fi
}
trap cleanup_installer_stage EXIT
stage_installer

if mountpoint -q "$root"; then
  echo "The disposable Arch root already has a mount at its root; inspect before retrying." >&2
  exit 5
fi
work="$root/var/tmp/marklinux-work-$(date -u +%Y%m%dT%H%M%SZ)-$$"
mkdir -m 700 -p "$work"
: > "$work/.marklinux-owned"
mkdir -p "$root/mnt/project" "$root/mnt/marklinux-work"
mount --bind "$root" "$root"
project_mounted=0
work_mounted=0
cleanup() {
  local mounts_clean=1
  if (( work_mounted )); then umount "$root/mnt/marklinux-work" || mounts_clean=0; fi
  if (( project_mounted )); then umount "$root/mnt/project" || mounts_clean=0; fi
  if mountpoint -q "$root"; then umount "$root" || mounts_clean=0; fi
  if (( mounts_clean )) && [[ -n "$work" && "$work" == "$root"/var/tmp/marklinux-work-* && ! -L "$work" && -f "$work/.marklinux-owned" && "${MARKLINUX_KEEP_WORK:-0}" != 1 ]]; then
    rm -rf -- "$work"
    echo "Removed task-owned temporary build files: $work"
  elif [[ -n "$work" && -d "$work" ]]; then
    echo "Retained build workspace for recovery: $work"
  fi
  cleanup_installer_stage
}
trap cleanup EXIT
mount --bind "$project" "$root/mnt/project"
project_mounted=1
mount --bind "$work" "$root/mnt/marklinux-work"
work_mounted=1

"$root/usr/bin/arch-chroot" "$root" /usr/bin/mkarchiso -v \
  -w /mnt/marklinux-work \
  -o /mnt/project/builds/iso \
  "$profile_in_chroot"

if [[ ! -s "$expected" ]]; then
  echo "mkarchiso returned without creating the expected image: $expected" >&2
  exit 6
fi
sha256sum "$expected" | tee "$hash_file"
stat --printf='ISO bytes: %s\n' "$expected"
echo "ISO build completed. Task-owned temporary files are removed after unmount. Set MARKLINUX_KEEP_WORK=1 only for debugging."
