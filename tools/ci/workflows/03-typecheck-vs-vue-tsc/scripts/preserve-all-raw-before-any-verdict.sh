#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-vs-vue-tsc/scripts/preserve-all-raw-before-any-verdict.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node "$VIZE_CI_FILES/preserve-all-raw-before-any-verdict.cjs"
tar -czf "$RUNNER_TEMP/typecheck-vs-vue-tsc-raw.tgz" -C "$RUNNER_TEMP/comparative" --exclude=target --exclude=source --exclude=provider/node_modules --exclude=vize --exclude=observations/misskey --exclude=observations/synthetic-project .
