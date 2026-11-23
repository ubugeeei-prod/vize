#!/usr/bin/env bash
git() {
  [[ "$1" == diff && "$2" == --name-only && "$3" == "$BASE_SHA" && "$4" == "$GITHUB_SHA" ]] || return 91
  if [[ "$MOCK_GIT_STATUS" != 0 ]]; then return "$MOCK_GIT_STATUS"; fi
  printf '%s' "$MOCK_CHANGED_PATHS"
}

source "$1"
