#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/github-apt-retry-wrapper/probe.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
count="$(cat "$1" 2>/dev/null || true)"
count="$((10#${count:-0} + 1))"
printf "%s" "$count" > "$1"
if (( count >= 10#$2 )); then exit 0; fi
exit "$3"
