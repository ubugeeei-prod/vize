#!/usr/bin/env bash
set -euo pipefail

download_archive="${1:?expected archive}"
if [ "$#" -ne 1 ]; then
  echo "::error::expected one rust-script archive path" >&2
  exit 1
fi
download_primary="https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz"
download_status=0
download_result="$(
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
    --output "$download_archive" --write-out '%{http_code}' "$download_primary"
  download_curl_status=$?
  printf '|'
  exit "$download_curl_status"
)" || download_status=$?
# Keep the HTTP field byte-exact; command substitution removes trailing LF.
download_http="${download_result%|}"
if [ "$download_status" -ne 0 ]; then
  printf 'rust-script primary download failed: curl=%s HTTP=%s URL=%s\n' \
    "$download_status" "$download_http" "$download_primary" >&2
  case "$download_status:$download_http" in
    22:500|22:502|22:503|22:504)
      curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
        --header 'Accept: application/octet-stream' --header 'X-GitHub-Api-Version: 2022-11-28' \
        --output "$download_archive" \
        "https://api.github.com/repos/fornwall/rust-script/releases/assets/127872817"
      ;;
    *)
      exit "$download_status"
      ;;
  esac
fi
