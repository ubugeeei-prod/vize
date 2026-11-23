#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/sfc-option-runtime"
npm --prefix "$RUNNER_TEMP/sfc-option-runtime" install --save-exact --ignore-scripts --no-audit --no-fund vue@3.5.44
printf 'VIZE_NATIVE_SFC_RUNTIME_OPTIONS_VUE=%s/sfc-option-runtime\n' "$RUNNER_TEMP" >> "$GITHUB_ENV"
