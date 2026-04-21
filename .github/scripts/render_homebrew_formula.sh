#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <version> <output-path>" >&2
  exit 1
fi

version="$1"
output_path="$2"
template_path="packaging/homebrew/gitviz.rb"
release_url="https://github.com/emilzmmn04/gitviz/releases/download/v${version}"

read_sha() {
  local checksum_file="$1"
  awk '{print $1}' "$checksum_file"
}

linux_x64_file="gitviz-v${version}-x86_64-unknown-linux-gnu.tar.gz"
linux_arm64_file="gitviz-v${version}-aarch64-unknown-linux-gnu.tar.gz"
macos_x64_file="gitviz-v${version}-x86_64-apple-darwin.tar.gz"
macos_arm64_file="gitviz-v${version}-aarch64-apple-darwin.tar.gz"

linux_x64_sha="$(read_sha "dist/gitviz-v${version}-x86_64-unknown-linux-gnu.sha256")"
linux_arm64_sha="$(read_sha "dist/gitviz-v${version}-aarch64-unknown-linux-gnu.sha256")"
macos_x64_sha="$(read_sha "dist/gitviz-v${version}-x86_64-apple-darwin.sha256")"
macos_arm64_sha="$(read_sha "dist/gitviz-v${version}-aarch64-apple-darwin.sha256")"

mkdir -p "$(dirname "$output_path")"
template_contents="$(cat "$template_path")"
template_contents="${template_contents//__VERSION__/$version}"
template_contents="${template_contents//__MACOS_ARM64_URL__/${release_url}/${macos_arm64_file}}"
template_contents="${template_contents//__MACOS_ARM64_SHA__/$macos_arm64_sha}"
template_contents="${template_contents//__MACOS_X64_URL__/${release_url}/${macos_x64_file}}"
template_contents="${template_contents//__MACOS_X64_SHA__/$macos_x64_sha}"
template_contents="${template_contents//__LINUX_ARM64_URL__/${release_url}/${linux_arm64_file}}"
template_contents="${template_contents//__LINUX_ARM64_SHA__/$linux_arm64_sha}"
template_contents="${template_contents//__LINUX_X64_URL__/${release_url}/${linux_x64_file}}"
template_contents="${template_contents//__LINUX_X64_SHA__/$linux_x64_sha}"
printf '%s\n' "$template_contents" > "$output_path"
