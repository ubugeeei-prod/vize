#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize_canon/tests/programs/report-missing-vite-types.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ "$1" = "--api" ]; then exec sleep 600; fi
echo "error TS2688: Cannot find type definition file for 'vite/client'."
exit 2
