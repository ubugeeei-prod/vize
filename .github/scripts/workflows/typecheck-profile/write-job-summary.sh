#!/usr/bin/env bash
if [[ -f "$RUNNER_TEMP/profile-out/summary.md" ]]; then
  head -c 900000 "$RUNNER_TEMP/profile-out/summary.md" >> "$GITHUB_STEP_SUMMARY"
fi
