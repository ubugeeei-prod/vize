#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/select-historical-capture.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git() {
  [[ "$1" == diff && "$2" == --name-only && "$3" == "$BASE_SHA" && "$4" == "$GITHUB_SHA" ]] || return 91
  if [[ "$MOCK_GIT_STATUS" != 0 ]]; then return "$MOCK_GIT_STATUS"; fi
  printf '%s' "$MOCK_CHANGED_PATHS"
}

source "$1"
