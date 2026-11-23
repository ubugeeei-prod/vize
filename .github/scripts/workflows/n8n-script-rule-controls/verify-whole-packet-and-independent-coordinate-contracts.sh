#!/usr/bin/env bash
set -o pipefail
vp node --test tests/tooling/n8n-cli-config-script.test.mjs 2>&1 | tee "$RUNNER_TEMP/n8n-script-rule-workflow/contracts.log"
