#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
  echo "usage: download-artifact.sh URL DESTINATION SHA256 [FALLBACK_URL]" >&2
  exit 2
fi
url="$1"
destination="$2"
checksum="$3"
fallback_url="${4:-}"
temporary="$(mktemp "${destination}.download.XXXXXX")"
trap 'rm -f "$temporary"' EXIT

# Bound connection, stalled transfer, each attempt, and the retry window.
# Tests lower the total timeouts against a local fault-injection server.
download() {
  curl --fail --location --silent --show-error \
    --connect-timeout 15 \
    --max-time "${VIZE_EDITOR_DOWNLOAD_TIMEOUT_SECONDS:-90}" \
    --speed-limit 1024 --speed-time 30 \
    --retry 2 --retry-delay 1 --retry-all-errors \
    --retry-max-time "${VIZE_EDITOR_DOWNLOAD_RETRY_SECONDS:-180}" \
    --output "$temporary" "$@"
}
primary_status=0
download "$url" || primary_status=$?
if [ "$primary_status" -ne 0 ]; then
  if [ -z "$fallback_url" ]; then
    exit "$primary_status"
  fi
  : > "$temporary"
  download \
    --header 'Accept: application/octet-stream' \
    --header 'X-GitHub-Api-Version: 2022-11-28' \
    "$fallback_url"
fi
actual="$(sha256sum "$temporary")"
if [ "${actual%% *}" != "$checksum" ]; then
  echo "Editor artifact SHA-256 checksum did NOT match" >&2
  exit 1
fi
# An interrupted/corrupt download never replaces an already verified artifact.
mv -f "$temporary" "$destination"
