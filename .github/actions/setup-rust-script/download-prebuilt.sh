#!/usr/bin/env bash
set -euo pipefail

# Contain the staging-file trap: this helper is sourced by the install action,
# whose existing trap still owns the complete runner temporary directory.
download_rust_script() (
  download_archive="${1:?expected archive}"
  if [ "$#" -ne 1 ] || [ -d "$download_archive" ]; then
    echo "::error::expected one rust-script archive file path" >&2
    exit 1
  fi
  download_primary="https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz"
  download_partial="$(mktemp "${download_archive}.partial.XXXXXX")"
  trap 'rm -f -- "$download_partial"' EXIT
  download_attempt=1
  while :; do
    : > "$download_partial"
    download_status=0
    download_result="$(
      curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
        --connect-timeout 10 --max-time 30 --speed-limit 1 --speed-time 5 \
        --output "$download_partial" --write-out '%{http_code}' "$download_primary"
      download_curl_status=$?
      printf '|'
      exit "$download_curl_status"
    )" || download_status=$?
    # Keep the HTTP field byte-exact; command substitution removes trailing LF.
    download_http="${download_result%|}"
    if [ "$download_status" -eq 0 ]; then
      break
    fi
    printf 'rust-script primary download failed: attempt=%s curl=%s HTTP=%s URL=%s\n' \
      "$download_attempt" "$download_status" "$download_http" "$download_primary" >&2
    case "$download_status:$download_http" in
      7:000|18:200|28:000|28:200|52:000|55:000|55:200|56:000|56:200)
        if [ "$download_attempt" -eq 1 ]; then
          download_attempt=2
          sleep 1
          continue
        fi
        exit "$download_status"
        ;;
      22:500|22:502|22:503|22:504)
        : > "$download_partial"
        curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
          --connect-timeout 10 --max-time 30 --speed-limit 1 --speed-time 5 \
          --header 'Accept: application/octet-stream' --header 'X-GitHub-Api-Version: 2022-11-28' \
          --output "$download_partial" \
          "https://api.github.com/repos/fornwall/rust-script/releases/assets/127872817" || exit "$?"
        break
        ;;
      *)
        exit "$download_status"
        ;;
    esac
  done
  # No partial body becomes the caller's archive; extraction still authenticates
  # the complete downloaded bytes against the unchanged production digest.
  mv -f -- "$download_partial" "$download_archive"
)
download_rust_script "$@"
