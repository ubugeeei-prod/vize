#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-replay/repeat-replay.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for repetition in 1 2 3 4 5; do
  "$1" content-mapper --reply-when-ready < "$2" > /dev/null
done
