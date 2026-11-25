#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize/tests/programs/close-input-compiler-error.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
exec 0<&-
echo 'authoritative compiler failure' >&2
exit 23
