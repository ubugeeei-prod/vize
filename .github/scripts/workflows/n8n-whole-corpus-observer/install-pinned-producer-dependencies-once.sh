#!/usr/bin/env bash
set -o pipefail
vp install --frozen-lockfile --prefer-offline 2>&1 | tee "$RUNNER_TEMP/n8n-whole-corpus-workflow/install.log"
