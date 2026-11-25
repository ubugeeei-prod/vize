#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/v1-five-case-action/git.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test "$1" = rev-parse
test "$2" = HEAD
printf "%s\n" "$SOURCE_SHA"
