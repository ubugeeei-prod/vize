#!/usr/bin/env bash
{
  echo "### TS-44 resource measurement (${RESOURCE_PRESET})"
  echo '```json'
  cat "${RUNNER_TEMP}/resource.json"
  echo '```'
} >> "$GITHUB_STEP_SUMMARY"
