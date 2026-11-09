#!/usr/bin/env bash
set -Eeuo pipefail
project=/mnt/h/LinuxOSbykrlmrk
export MARKLINUX_PROFILE="$project/os/arch-base/profiles/desktop"
artifact="$project/builds/artifacts/mark-hardware-x86_64-unknown-linux-gnu"
staged_binary="$MARKLINUX_PROFILE/airootfs/usr/local/bin/mark-hardware"
staged=0
cleanup() {
  if (( staged )); then
    rm -f -- "$staged_binary"
    echo "Removed task-staged hardware scanner from the ArchISO source overlay."
  fi
}
trap cleanup EXIT INT TERM

if [[ -e "$staged_binary" || -L "$staged_binary" ]]; then
  echo "Refusing to overwrite an existing desktop overlay file: $staged_binary" >&2
  exit 4
fi
bash "$project/scripts/build/build-hardware-detect.sh"
if [[ ! -s "$artifact" ]]; then
  echo "Linux hardware scanner artifact is missing: $artifact" >&2
  exit 5
fi
staged=1
install -m 0755 -- "$artifact" "$staged_binary"

if ! diff -u <(cat "$project/os/packages/base.x86_64" "$project/os/packages/desktop.x86_64") "$MARKLINUX_PROFILE/packages.x86_64"; then
  echo "Desktop profile packages differ from the checked-in package manifests." >&2
  exit 3
fi
bash "$project/scripts/build/build-base-iso.sh"
