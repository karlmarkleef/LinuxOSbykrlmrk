#!/usr/bin/env bash
set -Eeuo pipefail

project=/mnt/h/LinuxOSbykrlmrk
root=/var/tmp/marklinux-bootstrap/root.x86_64
source_dir="$project/core/hardware-detect"
target_dir="$project/builds/artifacts/mark-hardware-target"
binary="$project/builds/artifacts/mark-hardware-x86_64-unknown-linux-gnu"
manifest_in_chroot=/mnt/project/core/hardware-detect/Cargo.toml
target_in_chroot=/mnt/project/builds/artifacts/mark-hardware-target

if [[ ! -x "$root/usr/bin/cargo" || ! -x "$root/usr/bin/rustfmt" ]]; then
  echo "Rust is missing from the isolated Arch build root. Run prepare-arch-tools.sh first." >&2
  exit 2
fi
if [[ ! -f "$source_dir/Cargo.toml" ]]; then
  echo "Hardware detector Cargo manifest is missing: $source_dir/Cargo.toml" >&2
  exit 2
fi
if mountpoint -q "$root"; then
  echo "The isolated Arch root is already mounted; inspect before retrying." >&2
  exit 3
fi

mkdir -p "$root/mnt/project" "$project/builds/artifacts"
mount --bind "$root" "$root"
mount --bind "$project" "$root/mnt/project"
cleanup() {
  mountpoint -q "$root/mnt/project" && umount "$root/mnt/project"
  mountpoint -q "$root" && umount "$root"
}
trap cleanup EXIT

echo "Checking Rust formatting..."
"$root/usr/bin/arch-chroot" "$root" /usr/bin/env CARGO_TARGET_DIR="$target_in_chroot" \
  /usr/bin/cargo fmt --manifest-path "$manifest_in_chroot" -- --check
echo "Running Linux unit tests..."
"$root/usr/bin/arch-chroot" "$root" /usr/bin/env CARGO_TARGET_DIR="$target_in_chroot" \
  /usr/bin/cargo test --manifest-path "$manifest_in_chroot" --locked --all-targets
echo "Building optimized Linux CLI..."
"$root/usr/bin/arch-chroot" "$root" /usr/bin/env CARGO_TARGET_DIR="$target_in_chroot" \
  /usr/bin/cargo build --manifest-path "$manifest_in_chroot" --locked --release --bin mark-hardware

built="$target_dir/release/mark-hardware"
if [[ ! -s "$built" ]]; then
  echo "Cargo did not produce a non-empty Linux executable: $built" >&2
  exit 4
fi
install -m 0755 -- "$built" "$binary"
sha256sum "$binary" | tee "$binary.sha256"
echo "Linux hardware detector built: $binary"
