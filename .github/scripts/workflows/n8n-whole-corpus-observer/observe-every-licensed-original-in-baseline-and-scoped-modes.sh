#!/usr/bin/env bash
set -o pipefail
vp node tests/tooling/support/n8n-whole-corpus-observer.mjs "$GITHUB_WORKSPACE" "$OBSERVER_OUTPUT" 2>&1 | tee "$RUNNER_TEMP/n8n-whole-corpus-workflow/observer.log"
