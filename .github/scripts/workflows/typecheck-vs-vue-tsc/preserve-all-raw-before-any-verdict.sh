#!/usr/bin/env bash
node "$VIZE_CI_FILES/preserve-all-raw-before-any-verdict.cjs"
tar -czf "$RUNNER_TEMP/typecheck-vs-vue-tsc-raw.tgz" -C "$RUNNER_TEMP/comparative" --exclude=target --exclude=source --exclude=provider/node_modules --exclude=vize --exclude=observations/misskey --exclude=observations/synthetic-project .
