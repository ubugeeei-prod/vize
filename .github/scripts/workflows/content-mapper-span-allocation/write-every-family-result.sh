#!/usr/bin/env bash
if [[ -f "$RUNNER_TEMP/span-evidence/summary.md" ]]; then cat "$RUNNER_TEMP/span-evidence/summary.md" >> "$GITHUB_STEP_SUMMARY"; fi
