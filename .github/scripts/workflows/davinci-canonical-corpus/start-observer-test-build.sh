#!/usr/bin/env bash
nohup node tools/support/compat/github/canonical-corpus-shards.ts prebuild "$OBSERVER" > "$RUNNER_TEMP/canonical-prebuild.log" 2>&1 &
