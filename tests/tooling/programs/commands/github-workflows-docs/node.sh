#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/github-workflows-docs/node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ "$1" = "--test" ]; then
  printf '%s\n' '# tests 8' '# pass 8' '# fail 0' '# cancelled 0' '# skipped 0' '# todo 0'
  exit 17
fi
touch "$READINESS_CONTROL_MARKER"
