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
iso_in_chroot="/mnt/project${iso#"$project"}"
if [[ ! -x "$root/usr/bin/qemu-system-x86_64" || ! -x "$root/usr/bin/xorriso" ]]; then
  echo "QEMU/xorriso are missing from the disposable Arch build root." >&2
  exit 3
fi
mkdir -p "$logs" "$root/mnt/project"
stamp=$(date -u +%Y%m%dT%H%M%SZ)
iso_tag=$(basename "$iso" .iso)
log="$logs/$iso_tag-boot-$stamp.log"
screen="/mnt/project/builds/logs/$iso_tag-boot-$stamp.ppm"
screen_host="$logs/$iso_tag-boot-$stamp.ppm"
exec > >(tee -a "$log") 2>&1

echo "Image: $iso"
echo "Boot-test log: $log"
qemu_cpus="${MARKLINUX_QEMU_CPUS:-4}"
qemu_memory_mb="${MARKLINUX_QEMU_MEMORY_MB:-4096}"
boot_wait_seconds="${MARKLINUX_QEMU_WAIT_SECONDS:-140}"
boot_key_delay="${MARKLINUX_QEMU_BOOT_KEY_DELAY_SECONDS:-5}"
send_boot_key="${MARKLINUX_QEMU_SEND_BOOT_KEY:-1}"
post_boot_action="${MARKLINUX_QEMU_POST_BOOT_ACTION:-}"
post_boot_wait="${MARKLINUX_QEMU_POST_BOOT_WAIT_SECONDS:-10}"
for value in "$qemu_cpus" "$qemu_memory_mb" "$boot_wait_seconds" "$boot_key_delay" "$post_boot_wait"; do
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "QEMU CPU, memory and wait settings must be positive integers." >&2
    exit 2
  fi
done
if (( qemu_cpus < 1 || qemu_cpus > 8 || qemu_memory_mb < 1024 || qemu_memory_mb > 8192 || boot_wait_seconds < 30 || boot_key_delay < 1 || boot_key_delay > 300 )); then
  echo "QEMU settings outside safe test limits (1-8 CPUs, 1024-8192 MiB, >=30s wait, 1-300s key delay)." >&2
  exit 2
fi
if [[ "$send_boot_key" != "0" && "$send_boot_key" != "1" ]]; then
  echo "MARKLINUX_QEMU_SEND_BOOT_KEY must be 0 or 1." >&2
  exit 2
fi
if [[ -n "$post_boot_action" && "$post_boot_action" != "hardware-summary" && "$post_boot_action" != "installation-planner" ]]; then
  echo "Unsupported post-boot action: $post_boot_action" >&2
  exit 2
fi
if (( post_boot_wait < 1 || post_boot_wait > 60 )); then
  echo "Post-boot action wait must be between 1 and 60 seconds." >&2
  exit 2
fi
echo "Virtual machine: QEMU TCG, ${qemu_cpus} vCPU, ${qemu_memory_mb} MiB RAM, CD-ROM only; no host disks passed through."
if mountpoint -q "$root"; then
  echo "The disposable Arch root is already mounted; inspect before retrying." >&2
  exit 4
fi
mount --bind "$root" "$root"
project_mounted=0
cleanup() {
  if (( project_mounted )); then umount "$root/mnt/project"; fi
  umount "$root"
}
trap cleanup EXIT
mount --bind "$project" "$root/mnt/project"
project_mounted=1
"$root/usr/bin/arch-chroot" "$root" /usr/bin/xorriso \
  -indev "$iso_in_chroot" -report_el_torito as_mkisofs

# Let the default boot entry reach its live console, then capture the VGA output.
if [[ "$send_boot_key" == "1" ]]; then
  echo "Sending Enter after ${boot_key_delay}s, then waiting ${boot_wait_seconds}s before console capture."
else
  echo "No boot key will be sent; waiting ${boot_wait_seconds}s before console capture."
fi
echo "Set MARKLINUX_QEMU_SEND_BOOT_KEY=0 to avoid sending a boot key."
if [[ "$post_boot_action" == "hardware-summary" ]]; then
  echo "After boot, selecting the first Welcome action (Hardware Summary) and waiting ${post_boot_wait}s."
elif [[ "$post_boot_action" == "installation-planner" ]]; then
  echo "After boot, selecting Installation Planner in Welcome and waiting ${post_boot_wait}s for its first dialog."
fi
set +e
{ if [[ "$send_boot_key" == "1" ]]; then sleep "$boot_key_delay"; printf 'sendkey ret\n'; fi; sleep "$boot_wait_seconds"; if [[ "$post_boot_action" == "hardware-summary" ]]; then printf 'sendkey ret\n'; sleep "$post_boot_wait"; elif [[ "$post_boot_action" == "installation-planner" ]]; then printf 'sendkey down\nsendkey ret\n'; sleep "$post_boot_wait"; fi; printf 'info status\nscreendump %s\nquit\n' "$screen"; } |
  timeout "$((boot_wait_seconds + (send_boot_key == 1 ? boot_key_delay : 0) + post_boot_wait + 70))s" "$root/usr/bin/arch-chroot" "$root" /usr/bin/qemu-system-x86_64 \
    -machine q35,accel=tcg -cpu max -smp "$qemu_cpus" -m "$qemu_memory_mb" \
    -cdrom "$iso_in_chroot" -boot order=d \
    -display none -monitor stdio -serial none -no-reboot -no-shutdown
qemu_status=${PIPESTATUS[1]}
set -e
if (( qemu_status != 0 )); then
  echo "QEMU exited with status $qemu_status" >&2
  exit "$qemu_status"
fi
if [[ ! -s "$screen_host" ]]; then
  echo "QEMU did not produce a VGA screenshot." >&2
  exit 5
fi
sha256sum "$screen_host" | tee "$logs/$iso_tag-screen-$stamp.sha256"
echo "Captured guest console: $screen_host"
