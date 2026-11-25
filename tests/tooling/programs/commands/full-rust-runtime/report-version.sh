#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/full-rust-runtime/report-version.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[ "$#" = 1 ] && [ "$1" = --version ] || exit 9
printf 'Version @@VERSION@@\n'
