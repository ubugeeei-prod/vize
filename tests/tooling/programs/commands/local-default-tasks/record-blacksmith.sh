#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/local-default-tasks/record-blacksmith.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf '%s\n' "$*" >> "$FAKE_BLACKSMITH_LOG"
printf '%s\n' tbx_testbox_id
