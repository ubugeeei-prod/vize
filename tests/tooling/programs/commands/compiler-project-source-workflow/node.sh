#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/compiler-project-source-workflow/node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf "%s\n" "$@" > "$MOCK_ARGS"
printf "controlled failure\n" > "$3"
exit 23
