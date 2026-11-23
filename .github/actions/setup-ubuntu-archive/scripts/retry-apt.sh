#!/usr/bin/env bash
set -o pipefail

max_attempts="${VIZE_CI_APT_RETRIES:-4}"
delay_seconds="${VIZE_CI_APT_RETRY_DELAY_SECONDS:-8}"
if [[ "$max_attempts" =~ ^[0-9]+$ ]]; then
  max_attempts=$((10#$max_attempts))
else
  max_attempts=1
fi
if (( max_attempts < 1 )); then
  max_attempts=1
fi

if [[ "$delay_seconds" =~ ^[0-9]+$ ]]; then
  delay_seconds=$((10#$delay_seconds))
else
  delay_seconds=8
fi

for attempt in $(seq 1 "$max_attempts"); do
  if "$@"; then
    exit 0
  else
    status=$?
  fi

  if [[ "$attempt" = "$max_attempts" ]]; then
    exit "$status"
  fi

  echo "::warning title=Apt-backed setup retry::attempt ${attempt}/${max_attempts} failed with exit ${status}; retrying" >&2
  sudo rm -rf /var/lib/apt/lists/partial /var/cache/apt/archives/partial || true
  sleep "$((delay_seconds * attempt))"
done
