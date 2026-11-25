#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize/tests/programs/report-checker-count.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
previous=''
for arg in "$@"; do
  if [ "$previous" = '--checkers' ]; then
    printf '%s\n' "$arg" >> "$VIZE_TEST_CHECKER_LOG"
  fi
  previous="$arg"
done
exit 0
