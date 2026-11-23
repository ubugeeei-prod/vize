#!/usr/bin/env bash
printf 'Q16_OUT=%s/reported-q16\nQ16_PROVIDER=%s/reported-q16-provider\nQ16_CASE_SET=%s\n' "$RUNNER_TEMP" "$RUNNER_TEMP" "$Q16_REQUESTED_CASE_SET" >> "$GITHUB_ENV"
