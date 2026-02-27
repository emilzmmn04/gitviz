#!/usr/bin/env bash
set -euo pipefail

workflow_file="${1:-.github/workflows/release.yml}"

if [[ ! -f "$workflow_file" ]]; then
  echo "[FAIL] Workflow file not found: $workflow_file" >&2
  exit 1
fi

assert_pattern() {
  local pattern="$1"
  local label="$2"
  if rg --fixed-strings --quiet -- "$pattern" "$workflow_file"; then
    echo "[PASS] $label"
  else
    echo "[FAIL] $label (missing pattern: $pattern)" >&2
    exit 1
  fi
}

assert_pattern "if: \${{ secrets.NPM_TOKEN != '' }}" "npm publish gate exists"
assert_pattern "NODE_AUTH_TOKEN: \${{ secrets.NPM_TOKEN }}" "npm token wiring exists"
assert_pattern "if: \${{ secrets.HOMEBREW_TAP_GITHUB_TOKEN != '' }}" "homebrew gate exists"
assert_pattern "HOMEBREW_TAP_GITHUB_TOKEN: \${{ secrets.HOMEBREW_TAP_GITHUB_TOKEN }}" "homebrew token wiring exists"
assert_pattern "if: \${{ secrets.APT_GPG_PRIVATE_KEY != '' }}" "apt gate exists"
assert_pattern "APT_GPG_PRIVATE_KEY: \${{ secrets.APT_GPG_PRIVATE_KEY }}" "apt private key wiring exists"
assert_pattern "APT_GPG_PASSPHRASE: \${{ secrets.APT_GPG_PASSPHRASE }}" "apt passphrase wiring exists"

echo "[PASS] Release workflow secret gates match expected configuration."
