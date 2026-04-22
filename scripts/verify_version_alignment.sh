#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repo_root/Cargo.toml" | head -n1)"
npm_version="$(node -p "require('$repo_root/packaging/npm/package.json').version")"

if [[ -z "$cargo_version" || -z "$npm_version" ]]; then
  echo "[FAIL] Unable to determine Cargo.toml or npm package version." >&2
  exit 1
fi

if [[ "$cargo_version" != "$npm_version" ]]; then
  echo "[FAIL] Version mismatch: Cargo.toml=$cargo_version packaging/npm/package.json=$npm_version" >&2
  exit 1
fi

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [expected-version]" >&2
  exit 1
fi

if [[ $# -eq 1 ]]; then
  expected="${1#v}"
  if [[ "$cargo_version" != "$expected" ]]; then
    echo "[FAIL] Version mismatch: expected=$expected Cargo.toml=$cargo_version packaging/npm/package.json=$npm_version" >&2
    exit 1
  fi
fi

echo "[PASS] Version alignment verified: $cargo_version"
