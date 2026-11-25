#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/support/launchers/record-pid.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
exec 3<&0
@@COMMAND@@ "$@" <&3 &
native=$!
exec 3<&-
printf '%s\n' "$native" > @@PID_FILE@@
wait "$native"
