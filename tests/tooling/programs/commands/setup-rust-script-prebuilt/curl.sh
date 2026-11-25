#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-prebuilt/curl.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
printf '%s\n' "$@" > "$CURL_LOG"
if [ "$CURL_STATUS" != 0 ]; then exit "$CURL_STATUS"; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = --output ]; then cp "$FIXTURE_ARCHIVE" "$2"; exit 0; fi
  shift
done
exit 65
