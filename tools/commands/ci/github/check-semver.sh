#!/usr/bin/env bash
# Compare the checked-out PR merge against its actual base parent.
set -euo pipefail

BASELINE_REV="${BASELINE_REV:-}"
case "$BASELINE_REV" in 0000000000000000000000000000000000000000) BASELINE_REV="";; esac
# pull_request checkout is a merge commit. The event's base SHA can
# predate commits that landed on the base before this job started.
if git rev-parse --verify -q HEAD^2 >/dev/null; then
  BASELINE_REV="$(git rev-parse HEAD^1)"
fi
SEMVER_CHANGE_MARKER="$(cat "$RUNNER_TEMP/semver-change-marker.txt")"
SEMVER_ARGS=()
if printf '%s\n' "$SEMVER_CHANGE_MARKER" | grep -Eq '^[[:alnum:]_-]+(\([^)]+\))?!:|^BREAKING CHANGE:'; then
  SEMVER_ARGS+=(--release-type major)
fi
if [ -n "$BASELINE_REV" ]; then
  cargo semver-checks check-release --package "$1" --baseline-rev "$BASELINE_REV" "${SEMVER_ARGS[@]}"
else
  cargo semver-checks check-release --package "$1" "${SEMVER_ARGS[@]}"
fi
