#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <version>" >&2
  exit 1
fi

version="$1"
deb_file="dist/gitviz_${version}_amd64.deb"

if [[ ! -f "$deb_file" ]]; then
  echo "deb package not found: $deb_file" >&2
  exit 1
fi

: "${APT_GPG_PRIVATE_KEY:?APT_GPG_PRIVATE_KEY is required}"
: "${GITHUB_TOKEN:?GITHUB_TOKEN is required}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

repo_url="https://x-access-token:${GITHUB_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
repo_dir="$work_dir/gh-pages"

if ! git clone --depth 1 --branch gh-pages "$repo_url" "$repo_dir"; then
  mkdir -p "$repo_dir"
  pushd "$repo_dir" >/dev/null
  git init
  git checkout --orphan gh-pages
  popd >/dev/null
fi

apt_root="$repo_dir/apt"
pool_dir="$apt_root/pool/main/g/gitviz"
packages_dir="$apt_root/dists/stable/main/binary-amd64"

mkdir -p "$pool_dir" "$packages_dir" "$apt_root/keyrings"
cp "$deb_file" "$pool_dir/"

pushd "$apt_root" >/dev/null

dpkg-scanpackages --arch amd64 --multiversion pool > dists/stable/main/binary-amd64/Packages
gzip -kf dists/stable/main/binary-amd64/Packages

apt-ftparchive \
  -o APT::FTPArchive::Release::Origin="gitviz" \
  -o APT::FTPArchive::Release::Label="gitviz" \
  -o APT::FTPArchive::Release::Suite="stable" \
  -o APT::FTPArchive::Release::Codename="stable" \
  -o APT::FTPArchive::Release::Architectures="amd64" \
  -o APT::FTPArchive::Release::Components="main" \
  release dists/stable > dists/stable/Release

gpg --batch --import <(printf '%s\n' "$APT_GPG_PRIVATE_KEY")
key_fingerprint="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr:/ {print $10; exit}')"

if [[ -z "$key_fingerprint" ]]; then
  echo "unable to determine GPG key fingerprint" >&2
  exit 1
fi

passphrase_args=()
if [[ -n "${APT_GPG_PASSPHRASE:-}" ]]; then
  passphrase_args=(--pinentry-mode loopback --passphrase "$APT_GPG_PASSPHRASE")
fi

gpg --batch --yes "${passphrase_args[@]}" -u "$key_fingerprint" \
  --armor --detach-sign --output dists/stable/Release.gpg dists/stable/Release

gpg --batch --yes "${passphrase_args[@]}" -u "$key_fingerprint" \
  --clearsign --output dists/stable/InRelease dists/stable/Release

gpg --batch --yes --output keyrings/gitviz-archive-keyring.gpg --export "$key_fingerprint"

popd >/dev/null

pushd "$repo_dir" >/dev/null
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add apt

if git diff --cached --quiet; then
  echo "apt repo already up to date"
  exit 0
fi

git commit -m "Publish APT repo for v${version}"
git push "$repo_url" gh-pages:gh-pages
popd >/dev/null
