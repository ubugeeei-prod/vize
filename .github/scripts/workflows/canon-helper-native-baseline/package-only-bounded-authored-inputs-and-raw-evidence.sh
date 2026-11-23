#!/usr/bin/env bash
set -euo pipefail
cd "$RUNNER_TEMP"
paths=(helper-baseline-custody)
if [[ -d helper-baseline ]]; then paths+=(helper-baseline); fi
if [[ -d helper-slot-triage ]]; then paths+=(helper-slot-triage); fi
tar --exclude=node_modules --exclude=project --exclude=tmp --exclude=cache --exclude=xdg-cache -czf helper-baseline-evidence.tgz "${paths[@]}"
