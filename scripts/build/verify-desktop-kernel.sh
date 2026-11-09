#!/usr/bin/env bash
set -Eeuo pipefail

project=/mnt/h/LinuxOSbykrlmrk
root=/var/tmp/marklinux-bootstrap/root.x86_64
logs="$project/builds/logs"
iso="${1:-}"
if [[ -z "$iso" ]]; then
  iso=$(find "$project/builds/iso" -maxdepth 1 -type f -name 'marklinux-*.iso' -printf '%T@ %p\n' | sort -nr | head -n1 | cut -d' ' -f2-)
fi
if [[ -z "$iso" || ! -s "$iso" ]]; then
  echo "No Mark Linux ISO found; pass an existing image path." >&2
  exit 2
fi
iso=$(realpath -e -- "$iso")
if [[ "$iso" != "$project"/* ]]; then
  echo "Refusing to test an ISO outside the Mark Linux project: $iso" >&2
  exit 2
fi
if [[ ! -x "$root/usr/bin/qemu-system-x86_64" || ! -x "$root/usr/bin/xorriso" || ! -x "$root/usr/bin/arch-chroot" ]]; then
  echo "QEMU, xorriso or arch-chroot is missing from the build root." >&2
  exit 3
fi

qemu_cpus="${MARKLINUX_QEMU_CPUS:-4}"
qemu_memory_mb="${MARKLINUX_QEMU_MEMORY_MB:-4096}"
boot_wait_seconds="${MARKLINUX_QEMU_WAIT_SECONDS:-240}"
post_boot_action="${MARKLINUX_QEMU_POST_BOOT_ACTION:-}"
post_boot_wait="${MARKLINUX_QEMU_POST_BOOT_WAIT_SECONDS:-12}"
for value in "$qemu_cpus" "$qemu_memory_mb" "$boot_wait_seconds" "$post_boot_wait"; do
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "QEMU CPU, memory and wait settings must be positive integers." >&2
    exit 2
  fi
done
if (( qemu_cpus < 1 || qemu_cpus > 8 || qemu_memory_mb < 1024 || qemu_memory_mb > 8192 || boot_wait_seconds < 60 || post_boot_wait < 1 || post_boot_wait > 60 )); then
  echo "QEMU settings outside safe limits (1-8 CPUs, 1024-8192 MiB, >=60s wait, 1-60s post-boot wait)." >&2
  exit 2
fi
if [[ -n "$post_boot_action" && "$post_boot_action" != "hardware-summary" && "$post_boot_action" != "installation-planner" ]]; then
  echo "Unsupported post-boot action: $post_boot_action" >&2
  exit 2
fi

mkdir -p "$logs" "$root/mnt/project" "$root/var/tmp"
stamp=$(date -u +%Y%m%dT%H%M%SZ)
iso_tag=$(basename "$iso" .iso)
log="$logs/$iso_tag-direct-kernel-$stamp.log"
screen="/mnt/project/builds/logs/$iso_tag-direct-kernel-$stamp.ppm"
screen_host="$logs/$iso_tag-direct-kernel-$stamp.ppm"
serial="/mnt/project/builds/logs/$iso_tag-direct-kernel-$stamp-serial.log"
serial_host="$logs/$iso_tag-direct-kernel-$stamp-serial.log"
iso_in_chroot="/mnt/project${iso#"$project"}"
work_name="marklinux-kernel-test-$stamp-$$"
work="$root/var/tmp/$work_name"
work_in_chroot="/var/tmp/$work_name"

exec > >(tee -a "$log") 2>&1
echo "Image: $iso"
echo "Direct-kernel test log: $log"
echo "QEMU TCG: $qemu_cpus vCPU, $qemu_memory_mb MiB; ISO is CD-ROM only; no host disks passed through."
echo "This test bypasses the ISO boot menu and directly boots the kernel/initramfs extracted from this image."

if mountpoint -q "$root"; then
  echo "The disposable Arch root is already mounted; inspect before retrying." >&2
  exit 4
fi
mkdir -m 700 -p "$work"
: > "$work/.marklinux-owned"
mount --bind "$root" "$root"
project_mounted=0
work_mounted=0
cleanup() {
  local mounts_clean=1
  if (( work_mounted )); then umount "$root/mnt/marklinux-kernel-work" || mounts_clean=0; fi
  if (( project_mounted )); then umount "$root/mnt/project" || mounts_clean=0; fi
  if mountpoint -q "$root"; then umount "$root" || mounts_clean=0; fi
  if (( mounts_clean )) && [[ -d "$work" && ! -L "$work" && -f "$work/.marklinux-owned" ]]; then
    rm -rf -- "$work"
    echo "Removed task-owned kernel test files: $work"
  elif [[ -d "$work" ]]; then
    echo "Retained kernel test workspace for recovery: $work"
  fi
}
trap cleanup EXIT
mkdir -p "$root/mnt/marklinux-kernel-work"
mount --bind "$project" "$root/mnt/project"
project_mounted=1
mount --bind "$work" "$root/mnt/marklinux-kernel-work"
work_mounted=1

"$root/usr/bin/arch-chroot" "$root" /usr/bin/xorriso \
  -osirrox on -indev "$iso_in_chroot" \
  -extract /marklinux/boot/x86_64/vmlinuz-linux /mnt/marklinux-kernel-work/vmlinuz-linux \
  -extract /marklinux/boot/amd-ucode.img /mnt/marklinux-kernel-work/amd-ucode.img \
  -extract /marklinux/boot/x86_64/initramfs-linux.img /mnt/marklinux-kernel-work/initramfs-linux.img
for file in vmlinuz-linux amd-ucode.img initramfs-linux.img; do
  if [[ ! -s "$work/$file" ]]; then
    echo "Could not extract a non-empty boot file from the ISO: $file" >&2
    exit 5
  fi
done
cat "$work/amd-ucode.img" "$work/initramfs-linux.img" > "$work/initramfs-with-ucode.img"

if [[ "$post_boot_action" == "hardware-summary" ]]; then
  echo "After boot, selecting the first Welcome action (Hardware Summary) and waiting ${post_boot_wait}s."
elif [[ "$post_boot_action" == "installation-planner" ]]; then
  echo "After boot, selecting Installation Planner in Welcome and waiting ${post_boot_wait}s for its first dialog."
fi

set +e
{
  sleep "$boot_wait_seconds"
  if [[ "$post_boot_action" == "hardware-summary" ]]; then
    printf 'sendkey ret\n'
    sleep "$post_boot_wait"
  elif [[ "$post_boot_action" == "installation-planner" ]]; then
    printf 'sendkey down\n'
    sleep 1
    printf 'sendkey ret\n'
    sleep "$post_boot_wait"
  fi
  printf 'info status\nscreendump %s\nquit\n' "$screen"
} |
  timeout "$((boot_wait_seconds + post_boot_wait + 90))s" "$root/usr/bin/arch-chroot" "$root" /usr/bin/qemu-system-x86_64 \
    -machine q35,accel=tcg -cpu max -smp "$qemu_cpus" -m "$qemu_memory_mb" \
    -kernel "$work_in_chroot/vmlinuz-linux" -initrd "$work_in_chroot/initramfs-with-ucode.img" \
    -append 'archisobasedir=marklinux archisolabel=MARKLINUX_001 console=ttyS0,115200n8 console=tty0' \
    -cdrom "$iso_in_chroot" -boot order=d \
    -display none -monitor stdio -serial "file:$serial" -no-reboot -no-shutdown
qemu_status=${PIPESTATUS[1]}
set -e
if (( qemu_status != 0 )); then
  echo "QEMU exited with status $qemu_status" >&2
  exit "$qemu_status"
fi
if [[ ! -s "$screen_host" ]]; then
  echo "QEMU did not produce a VGA screenshot." >&2
  exit 6
fi
sha256sum "$screen_host" | tee "$logs/$iso_tag-direct-kernel-screen-$stamp.sha256"
if [[ -s "$serial_host" ]]; then
  sha256sum "$serial_host" | tee "$logs/$iso_tag-direct-kernel-serial-$stamp.sha256"
fi
echo "Captured guest VGA: $screen_host"
echo "Captured guest serial log: $serial_host"
