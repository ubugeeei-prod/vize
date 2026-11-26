#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/collect-playground-test-cases.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
packets=("$RUNNER_TEMP/test-cases-playground-browser.json")
vrt_report="playground/e2e/vrt/test-results/case-report.json"
if [[ "$RUN_VRT" == true ]]; then
  if [[ -s "$vrt_report" ]]; then
    node tools/support/compat/github/test-cases.ts playwright \
      "$vrt_report" "$RUNNER_TEMP/test-cases-playground-vrt.json"
    packets+=("$RUNNER_TEMP/test-cases-playground-vrt.json")
  elif [[ "$VRT_OUTCOME" == success ]]; then
    echo "::error::successful VRT produced no Playwright case receipt" >&2
    exit 1
  fi
fi
node tools/support/compat/github/test-cases.ts merge \
  "$RUNNER_TEMP/test-cases-playground.json" "${packets[@]}"
