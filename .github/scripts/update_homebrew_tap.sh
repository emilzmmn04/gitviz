#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <version>" >&2
  exit 1
fi

version="$1"
: "${HOMEBREW_TAP_GITHUB_TOKEN:?HOMEBREW_TAP_GITHUB_TOKEN is required}"

tap_repo="${HOMEBREW_TAP_REPO:-emilzmmn04/homebrew-tap}"
source_repo="${GITHUB_REPOSITORY:-emilzmmn04/gitviz}"
release_url="https://github.com/${source_repo}/releases/download/v${version}"
clone_url="https://x-access-token:${HOMEBREW_TAP_GITHUB_TOKEN}@github.com/${tap_repo}.git"
template_path="packaging/homebrew/gitviz.rb"

if [[ ! -f "$template_path" ]]; then
  echo "formula template not found: $template_path" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

git clone --depth 1 "$clone_url" "$tmp_dir/tap"

read_sha() {
  local file_name="$1"
  curl --fail --silent --show-error --location "${release_url}/${file_name}" | awk '{print $1}'
}

linux_x64_file="gitviz-v${version}-x86_64-unknown-linux-gnu.tar.gz"
linux_arm64_file="gitviz-v${version}-aarch64-unknown-linux-gnu.tar.gz"
macos_x64_file="gitviz-v${version}-x86_64-apple-darwin.tar.gz"
macos_arm64_file="gitviz-v${version}-aarch64-apple-darwin.tar.gz"

linux_x64_sha="$(read_sha "gitviz-v${version}-x86_64-unknown-linux-gnu.sha256")"
linux_arm64_sha="$(read_sha "gitviz-v${version}-aarch64-unknown-linux-gnu.sha256")"
macos_x64_sha="$(read_sha "gitviz-v${version}-x86_64-apple-darwin.sha256")"
macos_arm64_sha="$(read_sha "gitviz-v${version}-aarch64-apple-darwin.sha256")"

mkdir -p "$tmp_dir/tap/Formula"
formula_path="$tmp_dir/tap/Formula/gitviz.rb"
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
printf '%s\n' "$template_contents" > "$formula_path"

pushd "$tmp_dir/tap" >/dev/null
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add Formula/gitviz.rb

if git diff --cached --quiet; then
  echo "homebrew formula already up to date"
  exit 0
fi

git commit -m "gitviz ${version}"
git push origin HEAD
popd >/dev/null
