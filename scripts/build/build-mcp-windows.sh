#!/usr/bin/env bash
# build-mcp-windows.sh - Build the Mark Compatibility Platform Windows runtime tools
#
# Phase 1-3 of the Mark Compatibility Platform (Mark Linux MCP):
#   - mark-runtime       (windows/runtime-manager)  - Wine/Proton runtime detection
#   - mark-pe-inspect     (windows/pe-inspector)      - PE32/PE32+ inspection
#   - mark-apk-inspect    (android/apk-installer)     - APK ABI inspection
#
# The Rust tools are compiled inside the isolated Arch build root, staged into
# the desktop ISO source overlay, and the desktop ISO build is then invoked.
# The staged binaries are removed on exit so source validation only sees the
# checked-in overlay scripts.

set -Eeuo pipefail

project=/mnt/h/LinuxOSbykrlmrk
root=/var/tmp/marklinux-bootstrap/root.x86_64
artifact_dir="$project/builds/artifacts"
desktop_profile="$project/os/arch-base/profiles/desktop"
staged_dir="$desktop_profile/airootfs/usr/local/bin"

build_mcp_crate() {
  local crate_dir="$1"
  local bin_name="$2"
  local source="$crate_dir/Cargo.toml"
  local manifest_in_chroot="/mnt/project${crate_dir#"$project"}/Cargo.toml"
  local target_in_chroot="/mnt/project/builds/artifacts/mark-mcp-${bin_name}-target"

  if [[ ! -x "$root/usr/bin/cargo" || ! -x "$root/usr/bin/rustfmt" ]]; then
    echo "Rust is missing from the isolated Arch build root. Run prepare-arch-tools.sh first." >&2
    exit 2
  fi
  if [[ ! -f "$source" ]]; then
    echo "MCP crate manifest is missing: $source" >&2
    exit 4
  fi
  echo "Checking Rust formatting for $crate_dir..."
  "$root/usr/bin/arch-chroot" "$root" /usr/bin/env CARGO_TARGET_DIR="$target_in_chroot" /usr/bin/cargo fmt --manifest-path "$manifest_in_chroot" -- --check
  echo "Running Linux unit tests for $crate_dir..."
  "$root/usr/bin/arch-chroot" "$root" /usr/bin/env CARGO_TARGET_DIR="$target_in_chroot" /usr/bin/cargo test --manifest-path "$manifest_in_chroot" --all-targets
  echo "Building optimized Linux CLI for $crate_dir..."
  "$root/usr/bin/arch-chroot" "$root" /usr/bin/env CARGO_TARGET_DIR="$target_in_chroot" /usr/bin/cargo build --manifest-path "$manifest_in_chroot" --release --bin "$bin_name"
  local built="$target_in_chroot/release/$bin_name"
  if [[ ! -s "$built" ]]; then
    echo "Cargo did not produce a non-empty Linux executable: $built" >&2
    exit 5
  fi
  install -m 0755 -- "$built" "$artifact_dir/$bin_name"
  sha256sum "$artifact_dir/$bin_name" | tee "$artifact_dir/$bin_name.sha256"
}

stage_binary() {
  local source="$1"
  local target="$2"
  if [[ -e "$target" || -L "$target" ]]; then
    echo "Refusing to overwrite an existing desktop overlay file: $target" >&2
    exit 6
  fi
  install -m 0755 -- "$source" "$target"
}

cleanup_staged() {
  rm -f -- \
    "$staged_dir/mark-runtime" \
    "$staged_dir/mark-pe-inspect" \
    "$staged_dir/mark-apk-inspect" \
    "$staged_dir/mark-android-manager" \
    "$staged_dir/mark-apk"
}

mkdir -p "$root/mnt/project" "$artifact_dir"
mount --bind "$root" "$root"
mount --bind "$project" "$root/mnt/project"
cleanup_mounts() {
  mountpoint -q "$root/mnt/project" && umount "$root/mnt/project"
  mountpoint -q "$root" && umount "$root"
}
trap cleanup_mounts EXIT INT TERM

build_mcp_crate "$project/windows/runtime-manager" mark-runtime
build_mcp_crate "$project/windows/pe-inspector" mark-pe-inspect
build_mcp_crate "$project/android/apk-installer" mark-apk-inspect

stage_binary "$artifact_dir/mark-runtime" "$staged_dir/mark-runtime"
stage_binary "$artifact_dir/mark-pe-inspect" "$staged_dir/mark-pe-inspect"
stage_binary "$artifact_dir/mark-apk-inspect" "$staged_dir/mark-apk-inspect"
stage_binary "$project/android/container-manager/mark-android-manager" "$staged_dir/mark-android-manager"
stage_binary "$project/android/package-manager/mark-apk" "$staged_dir/mark-apk"

cleanup_staged

MARKLINUX_PROFILE="$project/os/arch-base/profiles/desktop" \
  bash "$project/scripts/build/build-base-iso.sh"
