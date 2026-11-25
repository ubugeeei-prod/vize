#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/rust-build-evidence/cargo.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf '%s\n' "$@" > "$ARGS_FILE"
printf '%s\n' '{"reason":"build-finished","success":@@EXIT_0@@}'
exit @@EXIT@@
