#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-profile/read-the-pushed-commit-s-profile-trailers.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
trailer() { git show -s --format="%(trailers:key=$1,valueonly,separator=%x2C)" HEAD | tr -d '[:space:]'; }
for pair in BASE_SHA:Profile-Base CORPORA:Profile-Corpora LANES:Profile-Lanes FREQUENCY:Profile-Frequency; do
  value="$(trailer "${pair#*:}")"
  if [[ -n "$value" ]]; then echo "${pair%%:*}=$value" >> "$GITHUB_ENV"; fi
done
