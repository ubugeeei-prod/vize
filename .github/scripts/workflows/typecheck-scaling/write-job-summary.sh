#!/usr/bin/env bash
if [[ -f "$RUNNER_TEMP/scaling-out/summary.md" ]]; then cat "$RUNNER_TEMP/scaling-out/summary.md" >> "$GITHUB_STEP_SUMMARY"; fi
