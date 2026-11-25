#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/support/check-fixtures/process-limit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
prefix=${1:--}
ulimit "${prefix}u" 2>/dev/null || ulimit "${prefix}p"
