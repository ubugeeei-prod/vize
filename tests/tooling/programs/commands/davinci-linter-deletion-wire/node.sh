#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/davinci-linter-deletion-wire/node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf '%s\n' "$@" > "$RECORDER_ARGUMENTS"
mkdir -p "$2"
printf '{}\n' > "$2/linter-deletion-readiness.json"
