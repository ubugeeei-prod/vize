#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/canon-helper-native-proxy/node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
: "${NATIVE_LAUNCHER_ARGS:?}"
printf '%s\n' "$@" > "$NATIVE_LAUNCHER_ARGS"
