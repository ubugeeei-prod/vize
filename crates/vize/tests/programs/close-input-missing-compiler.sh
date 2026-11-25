#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize/tests/programs/close-input-missing-compiler.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
exec 0<&-
echo "node:internal/modules/esm/resolve:271" >&2
echo "Error [ERR_MODULE_NOT_FOUND]: Cannot find module '/app/node_modules/vue/compiler-sfc'" >&2
exit 17
