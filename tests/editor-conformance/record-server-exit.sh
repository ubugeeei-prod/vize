#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/editor-conformance/record-server-exit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
code=0
"$@" || code=$?
printf '{"session":%s,"t":null,"event":"exit","code":%s}\n' "$VIZE_TAP_SESSION" "$code" >> "$VIZE_CONFORMANCE_TRANSCRIPT"
exit "$code"
