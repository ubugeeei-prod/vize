#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize_canon/tests/programs/observe-checker-options.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for arg in "$@"; do
  if [ "$arg" = "--checkers" ]; then
    echo "error TS5023: Unknown compiler option '--checkers'."
    exit 1
  fi
done
exit 0
