#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-transport/tar.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf "%s\n" "$*" >> "$TAR_LOG"
exec /usr/bin/tar "$@"
