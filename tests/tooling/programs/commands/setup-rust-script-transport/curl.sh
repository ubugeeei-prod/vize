#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-transport/curl.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
count=0; [ ! -f "$CURL_COUNT" ] || count="$(cat "$CURL_COUNT")"
count=$((count + 1)); printf "%s" "$count" > "$CURL_COUNT"
printf "%s\n" "$@" > "$CURL_LOG.$count"
url=""; output=""; writeout=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output) output="$2"; shift 2 ;;
    --write-out) [ "$2" = "%{http_code}" ]; writeout=true; shift 2 ;;
    --header|--proto) shift 2 ;;
    --*) shift ;;
    *) url="$1"; shift ;;
  esac
done
if [ "$url" = "$PRIMARY_URL" ]; then
  status="$PRIMARY_STATUS"; http="$PRIMARY_HTTP"; body="$FIXTURE_ARCHIVE"
elif [ "$url" = "$API_URL" ]; then
  status="$API_STATUS"; http="$API_HTTP"; body="$API_BODY"
else exit 65; fi
cp "$body" "$output"
$writeout && printf "%s" "$http"
if [ "$status" -ne 0 ]; then
  printf "curl fixture: original failure %s HTTP %s\n" "$status" "$http" >&2
fi
exit "$status"
