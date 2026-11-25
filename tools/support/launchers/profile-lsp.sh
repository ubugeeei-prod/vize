#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/support/launchers/profile-lsp.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -e
if [ "$1" = "lsp" ]; then
  exec @@COMMAND@@ "$@" --profile-json @@PROFILE@@
fi
exec @@COMMAND@@ "$@"
