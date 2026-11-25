#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/content-mapper-source-node22-workflow/node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ "$1" = "npm/native/scripts/native-build-receipt.mjs" ]; then printf "producer receipt\n" > "$MOCK_PRODUCER"; exit 0; fi
mkdir -p "$3"
printf "controlled consumer stderr\n" >&2
printf "%s\n" "$@" > "$MOCK_ARGS"
printf "failed outcome\n" > "$3/receipt.json"
exit 23
