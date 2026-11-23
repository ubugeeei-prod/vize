#!/bin/sh
exec 3<&0
@@COMMAND@@ "$@" <&3 &
native=$!
exec 3<&-
printf '%s\n' "$native" > @@PID_FILE@@
wait "$native"
