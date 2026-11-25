#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/replay-shas.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [[ -z "$BASE" ]]; then
  BASE=$(git log -1 --format='%(trailers:key=Replay-Base,valueonly)' "$HEAD" | head -1 | tr -d '[:space:]')
fi
[[ -n "$BASE" ]] || BASE=$(git merge-base origin/main "$HEAD")
[[ "$BASE" =~ ^[0-9a-f]{40}$ && "$HEAD" =~ ^[0-9a-f]{40}$ ]]
echo "base=$BASE" >> "$GITHUB_OUTPUT"
echo "head=$HEAD" >> "$GITHUB_OUTPUT"
echo "base $BASE, head $HEAD"
