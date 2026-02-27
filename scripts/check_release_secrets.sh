#!/usr/bin/env bash
set -euo pipefail

required=(
  NPM_TOKEN
  HOMEBREW_TAP_GITHUB_TOKEN
  APT_GPG_PRIVATE_KEY
)

missing=()
for name in "${required[@]}"; do
  if [[ -z "${!name:-}" ]]; then
    missing+=("$name")
  fi
done

if (( ${#missing[@]} > 0 )); then
  echo "[FAIL] Missing required secret(s): ${missing[*]}" >&2
  exit 1
fi

if [[ "${APT_GPG_PRIVATE_KEY}" != *"BEGIN PGP PRIVATE KEY BLOCK"* ]]; then
  echo "[FAIL] APT_GPG_PRIVATE_KEY does not look like an armored private key block." >&2
  exit 1
fi

if command -v gpg >/dev/null 2>&1; then
  if ! printf '%s\n' "${APT_GPG_PRIVATE_KEY}" | gpg --batch --import-options show-only --dry-run --import >/dev/null 2>&1; then
    echo "[FAIL] APT_GPG_PRIVATE_KEY failed gpg dry-run import validation." >&2
    exit 1
  fi
  echo "[PASS] APT_GPG_PRIVATE_KEY validated by gpg dry-run import."
else
  echo "[WARN] gpg not installed; skipped dry-run key import validation."
fi

if [[ -n "${APT_GPG_PASSPHRASE:-}" ]]; then
  echo "[PASS] Optional APT_GPG_PASSPHRASE is set."
else
  echo "[INFO] Optional APT_GPG_PASSPHRASE is not set (valid for unencrypted key)."
fi

echo "[PASS] Required release secrets are present."
