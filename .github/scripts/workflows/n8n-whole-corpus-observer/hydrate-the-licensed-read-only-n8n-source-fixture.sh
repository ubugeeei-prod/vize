#!/usr/bin/env bash
set -o pipefail
git submodule update --init --depth 1 -- tests/_fixtures/_git/n8n 2>&1 | tee "$RUNNER_TEMP/n8n-whole-corpus-workflow/hydrate.log"
git submodule status -- tests/_fixtures/_git/n8n > "$RUNNER_TEMP/n8n-whole-corpus-workflow/fixture-status.txt"
