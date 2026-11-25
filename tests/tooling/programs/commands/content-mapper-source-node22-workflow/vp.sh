#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/content-mapper-source-node22-workflow/vp.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf "%s\n" "$@" >> "$MOCK_PACK"
if [ "$#" -ne 1 ] || [ "$1" != "pack" ]; then exit 91; fi
printf "controlled pack stdout\n"
printf "controlled pack stderr\n" >&2
exit "$PACK_STATUS"
