#!/usr/bin/env bash
git ls-files -z -- '*.d.ts' '*.d.mts' '*.d.cts' > "$RUNNER_TEMP/declaration-inputs"
