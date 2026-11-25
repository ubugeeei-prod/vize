#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-prebuilt/bash.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [ "$1" != "$EXTRACTOR" ]; then exec /bin/bash "$@"; fi
printf '%s\n' "$@" > "$HELPER_LOG"
if [ "$FIXTURE_DIGEST" = true ]; then
  exec /bin/bash "$1" "$2" "$3" "$ARCHIVE_DIGEST"
fi
exec /bin/bash "$@"
