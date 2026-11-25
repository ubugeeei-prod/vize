#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/support/check-fixtures/run-with-process-limit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
limit=$1
unsupported=$2
shift 2
ulimit -u "$limit" 2>/dev/null || ulimit -p "$limit" 2>/dev/null || exit "$unsupported"
exec "$@"
