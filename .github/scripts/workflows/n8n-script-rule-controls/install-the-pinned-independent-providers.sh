#!/usr/bin/env bash
set -o pipefail
vp install --frozen-lockfile --prefer-offline 2>&1 | tee "$RUNNER_TEMP/n8n-script-rule-workflow/install.log"
