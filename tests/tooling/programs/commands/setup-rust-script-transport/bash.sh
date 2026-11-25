#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-transport/bash.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
printf "%s\n" "$@" > "$HELPER_LOG"
[ "$1" = "$EXTRACTOR" ] && [ "$4" = "$PRODUCTION_PIN" ]
if [ "$FIXTURE_DIGEST" = true ]; then
  exec /bin/bash "$1" "$2" "$3" "$FIXTURE_SHA"
fi
exec /bin/bash "$@"
