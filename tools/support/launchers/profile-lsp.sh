#!/bin/sh
set -e
if [ "$1" = "lsp" ]; then
  exec @@COMMAND@@ "$@" --profile-json @@PROFILE@@
fi
exec @@COMMAND@@ "$@"
