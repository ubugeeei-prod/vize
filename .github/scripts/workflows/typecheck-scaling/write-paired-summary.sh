#!/usr/bin/env bash
if [[ -f paired-scaling/measurements/summary.md ]]; then
  cat paired-scaling/measurements/summary.md >> "$GITHUB_STEP_SUMMARY"
fi
