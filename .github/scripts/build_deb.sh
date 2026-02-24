#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <version> <linux-x86_64-tarball>" >&2
  exit 1
fi

version="$1"
tarball="$2"

if [[ ! -f "$tarball" ]]; then
  echo "tarball not found: $tarball" >&2
  exit 1
fi

mkdir -p dist

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

tar -xzf "$tarball" -C "$tmp_dir"

if [[ ! -f "$tmp_dir/gitviz" ]]; then
  echo "archive does not contain gitviz binary" >&2
  exit 1
fi

pkg_root="$tmp_dir/pkg"
mkdir -p "$pkg_root/DEBIAN" "$pkg_root/usr/bin"
install -m 0755 "$tmp_dir/gitviz" "$pkg_root/usr/bin/gitviz"

cat > "$pkg_root/DEBIAN/control" <<CONTROL
Package: gitviz
Version: ${version}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: gitviz maintainers <opensource@emilz.dev>
Depends: git
Description: Terminal Git repository visualizer
 Fast, keyboard-driven terminal UI for visualizing Git commit history.
CONTROL

deb_file="dist/gitviz_${version}_amd64.deb"
dpkg-deb --build "$pkg_root" "$deb_file"

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$deb_file" > "${deb_file}.sha256"
else
  shasum -a 256 "$deb_file" > "${deb_file}.sha256"
fi

echo "created ${deb_file} and ${deb_file}.sha256"
