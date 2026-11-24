#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-canonical-corpus/start-observer-test-build.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
nohup node tools/support/compat/github/canonical-corpus-shards.ts prebuild "$OBSERVER" > "$RUNNER_TEMP/canonical-prebuild.log" 2>&1 &
