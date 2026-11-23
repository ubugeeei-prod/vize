#!/usr/bin/env bash
set -o pipefail
vp node tests/tooling/support/n8n-cli-config-script-rules-acceptance.mjs "$SCRIPT_CONTROL_OUTPUT" 2>&1 | tee "$RUNNER_TEMP/n8n-script-rule-workflow/controls.log"
