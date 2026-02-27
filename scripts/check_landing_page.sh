#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <landing_url> [custom_domain]" >&2
  echo "example: $0 https://gitviz-production.up.railway.app gitviz.dev" >&2
  exit 1
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
fi

landing_url="${1%/}"
custom_domain="${2:-}"

if [[ ! "$landing_url" =~ ^https?:// ]]; then
  echo "landing_url must start with http:// or https:// (got: $landing_url)" >&2
  exit 1
fi

tmp_headers="$(mktemp)"
tmp_body="$(mktemp)"
trap 'rm -f "$tmp_headers" "$tmp_body"' EXIT

request_and_assert_html() {
  local url="$1"
  local label="$2"

  # no-cache headers emulate a hard refresh
  local status
  status="$(curl -sS --location \
    --header 'Cache-Control: no-cache' \
    --header 'Pragma: no-cache' \
    --dump-header "$tmp_headers" \
    --output "$tmp_body" \
    --write-out '%{http_code}' \
    "$url")"

  if [[ "$status" != "200" ]]; then
    echo "[FAIL] $label did not return HTTP 200 (got: $status) for $url" >&2
    exit 1
  fi

  local content_type
  content_type="$(awk 'BEGIN{IGNORECASE=1} /^content-type:/ {print tolower($0); exit}' "$tmp_headers")"
  if [[ "$content_type" != *"text/html"* ]]; then
    echo "[FAIL] $label content-type is not text/html (got: ${content_type:-<missing>})" >&2
    exit 1
  fi

  echo "[PASS] $label returned 200 + text/html"
}

echo "Running landing page smoke checks against $landing_url"
request_and_assert_html "${landing_url}/" "GET /"
request_and_assert_html "${landing_url}/index.html" "GET /index.html"

# Repeat root request to catch obvious restart/crash loops.
request_and_assert_html "${landing_url}/" "GET / (repeat)"

if [[ -n "$custom_domain" ]]; then
  domain="${custom_domain#http://}"
  domain="${domain#https://}"
  domain="${domain%%/*}"
  if [[ -z "$domain" ]]; then
    echo "[FAIL] custom_domain could not be parsed: $custom_domain" >&2
    exit 1
  fi
  request_and_assert_html "https://${domain}/" "Custom domain TLS endpoint"
fi

echo
echo "Landing page smoke checks passed."
echo "Manual follow-up: inspect Railway logs for nginx startup and no restart loop."
