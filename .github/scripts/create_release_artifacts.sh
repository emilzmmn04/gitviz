#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <version> <target-triple> <binary-path>" >&2
  exit 1
fi

version="$1"
target="$2"
binary_path="$3"

if [[ ! -f "$binary_path" ]]; then
  echo "binary not found: $binary_path" >&2
  exit 1
fi

archive_name="gitviz-v${version}-${target}.tar.gz"
checksum_name="gitviz-v${version}-${target}.sha256"

mkdir -p dist

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

cp "$binary_path" "$tmp_dir/gitviz"
chmod 0755 "$tmp_dir/gitviz"

tar -C "$tmp_dir" -czf "dist/${archive_name}" gitviz

if command -v shasum >/dev/null 2>&1; then
  hash="$(shasum -a 256 "dist/${archive_name}" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  hash="$(sha256sum "dist/${archive_name}" | awk '{print $1}')"
else
  echo "missing checksum tool (shasum or sha256sum)" >&2
  exit 1
fi

printf '%s  %s\n' "$hash" "$archive_name" > "dist/${checksum_name}"

echo "created dist/${archive_name} and dist/${checksum_name}"
