#!/usr/bin/env bash
set -o pipefail
vp pack 2>&1 | tee "$RUNNER_TEMP/n8n-whole-corpus-workflow/pack.log"
