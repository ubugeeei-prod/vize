#!/usr/bin/env bash
count="$(cat "$1" 2>/dev/null || true)"
count="$((10#${count:-0} + 1))"
printf "%s" "$count" > "$1"
if (( count >= 10#$2 )); then exit 0; fi
exit "$3"
